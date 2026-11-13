//! The stack: one policy's rows, applied in array order over one base (ADR-015,
//! design 015, Chosen design → The stack).
//!
//! A row is a *named, preconditioned* transform: it states what it is about to
//! do, and it does it only if every precondition fits the layout as it stands at
//! that point in the stack. That ordering is load-bearing — array order is
//! application order, so a row sees the output of the rows above it, and a row
//! can make the row below it fit or stop fitting (Options considered: Stack
//! order). Two channels carry the answer, and keeping them apart is what makes a
//! policy safe to run over somebody's layout:
//!
//! - **A row that does not fit is a verdict**, a [`NotApplicable`] naming one of
//!   the ten closed reasons, and the row writes nothing. `CoreError` means the
//!   *application* could not be evaluated at all — a patch as the base, a base
//!   pinned to a manifest other than the one it is applied against, a policy
//!   naming its own base, a base that binds one action twice (A1, A9). Turning a
//!   row's bad fit into a refusal would let one unhelpful row in a nine-row stack
//!   throw away the eight that fit; turning an entry fault into a verdict would
//!   let a patch through the stack.
//! - **A row is atomic.** Its preconditions are all checked against the working
//!   layout, and only then are its writes committed, so "never mangles" is
//!   structural rather than a hope: a failing row leaves the layout byte-equal
//!   with what the rows above it produced.
//!
//! The match rule is stated once, here, and every reason a report names hangs
//! off it: the match set is the loaded registry's ids the tag matches, in
//! `ActionId` byte order; the k-th matched id pairs with the k-th target in row
//! order; and the produced base's `set` keeps the source base's element order,
//! changing inputs in place rather than re-sorting the layout. Sorted containers
//! are also what makes *which* offender a report names deterministic, which is
//! why the offender is part of the determinism surface.
//!
//! The working layout answers each question once. An action has exactly one
//! binding in it — a base that binds one action twice is an entry refusal, so
//! the stack never reads a precondition off one copy of a binding while writing
//! another, which is how a hostile-but-legal base could make a row report
//! `Applied` over a layout nobody resolves to — while an input may serve several
//! actions, exactly as `resolve` keeps them, so occupancy is answered by every
//! holder of the key rather than the first one.
//!
//! Nothing here is a heuristic. Hands, bounds, and targets come from the row;
//! grades and grid coordinates come from the loaded contract tree; the untouchable
//! guard reads the loaded manifest's constraints. The one computation core
//! deliberately does not ship — whether a set of keys is one contiguous cluster —
//! lives here, because a "cluster" is a transform's concept, not a fact about a
//! keyboard (A5, A21).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bindforge_core::{
    ActionId, ActionRegistry, AxisKind, CoreError, GradeBound, Hand, InputId, InputNamespace,
    KeyboardGeometry, Manifest, PlacementCheck, Policy, Profile, ProfileKind, ReachTables,
    TagPattern, Transform, placement_is_legal,
};

/// The two members of the closed input vocabulary `swap-mouse-buttons` is
/// defined over. The row carries no tag precisely because this pair is closed:
/// a tag would be a second match language over ids the grammar already fixes
/// (A19).
const MOUSE_LEFT: &str = "mouse:left";
const MOUSE_RIGHT: &str = "mouse:right";

/// What a whole application answered: the produced base, and one report per row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    /// The new `profile/v2` base document — an ordinary chain citizen, gated by
    /// validation rather than trusted by provenance (A7, A18).
    pub base: Profile,
    /// One report per row of `transforms`, carrying that row's position.
    pub reports: Vec<TransformReport>,
}

/// What one row answered.
///
/// `index` is the row's position in the `transforms` array, which is the
/// position the author wrote and the position ADR-017's CLI and the hub display
/// point a player at. `kind` is the contract's spelling of the row's kind — the
/// same string `spec.md` states — so a report reads back as the row it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransformReport {
    pub index: usize,
    pub kind: &'static str,
    pub outcome: Outcome,
}

/// One row's answer: it fits, or it does not and says why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every precondition held and the row's writes landed.
    Applied,
    /// A precondition did not hold; the row wrote nothing.
    NotApplicable(NotApplicable),
}

/// The verdict of a row that does not fit: a closed reason, the offender it
/// names, and — where the reason is a placement failure — the
/// [`PlacementCheck`] itself as the evidence.
///
/// The evidence is the point. A boolean cannot tell an author that their move
/// was one clean translation whose landing key sat four ranks past the declared
/// bound, which is exactly the answer a fix needs (A4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotApplicable {
    pub reason: NotApplicableReason,
    /// The id, tag, or key the row tripped over, as the contract spells it.
    pub offender: Option<String>,
    /// A `placement-illegal` verdict carries the predicate's own check.
    pub check: Option<PlacementCheck>,
}

/// The closed reason set `schemas/policy/v1/spec.md` states — the vocabulary
/// ADR-017's CLI and the hub display render, and the reason a third-party
/// implementer of `policy/v1` produces for the same fault. A fourteenth reason is
/// a `policy/v2` question, not a new enum arm (A11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotApplicableReason {
    /// The tag matches no id in the loaded registry (A3): the row is about
    /// nothing, and reporting that beats a silent no-op (A1).
    UnmatchedTags,
    /// A matched action has no binding in the working layout, so the cluster the
    /// row describes is not the cluster this base has.
    UnboundAction,
    /// The match set and the target list are different sizes, so the k-th-with-
    /// k-th pairing has no answer for a leftover.
    MatchCount,
    /// A source or target key is not a `keyboard:` input: grades and coordinates
    /// cover the keyboard only, so this is the transform's own check rather than
    /// the predicate's definedness refusal, which stays reserved for data gaps
    /// (A5).
    NotKeyboard,
    /// The matched keys are not one cluster: no rook-adjacency path in the loaded
    /// grid joins them, so there is nothing to translate (A21).
    NotContiguous,
    /// A target is bound to an action outside the match set. The cluster's own
    /// vacated keys stay legal to walk onto — that distinction is the whole
    /// difference between a translation and a displacement.
    OccupiedTarget,
    /// The placement predicate answered illegal, check attached: one translation
    /// rule broken, or a degradation past the row's declared bound (A4, A24).
    PlacementIllegal,
    /// The row reads or writes an input the manifest reserves, or touches an
    /// action the manifest hard-docks. The guard is unconditional and
    /// loaded-data-driven, never author-declared (A6, A23).
    ReservedInput,
    /// A `swap-polarity` match reached an id whose axis kind is not
    /// `discrete-lateral` — the rule 011's enum comment pre-announced and A2
    /// made load-bearing.
    AxisKind,
    /// A `swap-mouse-buttons` member has no bound action, and swapping one button
    /// into nothing is the mangling the ADR refuses.
    UnboundInput,
}

impl NotApplicableReason {
    /// The contract spelling of this reason — the string `spec.md` states and a
    /// non-Rust implementer writes, the way `ProfileKind::name()` answers its
    /// spellings today.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            NotApplicableReason::UnmatchedTags => "unmatched-tags",
            NotApplicableReason::UnboundAction => "unbound-action",
            NotApplicableReason::MatchCount => "match-count",
            NotApplicableReason::NotKeyboard => "not-keyboard",
            NotApplicableReason::NotContiguous => "not-contiguous",
            NotApplicableReason::OccupiedTarget => "occupied-target",
            NotApplicableReason::PlacementIllegal => "placement-illegal",
            NotApplicableReason::ReservedInput => "reserved-input",
            NotApplicableReason::AxisKind => "axis-kind",
            NotApplicableReason::UnboundInput => "unbound-input",
        }
    }
}

/// Apply one policy's stack over one base.
///
/// The arguments are the loaded values the stack joins — the same sextuple the
/// placement predicate takes, with a policy in place of a candidate profile —
/// and all six are required: game facts from the manifest, axis kinds from the
/// registry, grades and coordinates from the tables and the grid.
///
/// The `Err` channel is the entry channel, and only entry faults travel on it:
/// a `kind = "patch"` document as the base (a transform applies to bases, and the
/// personal patch sits above the stack, never opened and never rewritten, A1); a
/// base whose `for` pin differs from the manifest it is applied against (014's
/// pin equality, which is also what makes bumping pins a review, A7); a policy
/// whose `id` equals the base's, which would produce a base naming its own
/// input in a directory that holds one file per id; and a base whose `set`
/// answers one action twice, which leaves the stack no layout to read a
/// precondition off. Every one reuses a shipped `CoreError` variant (A9).
pub fn apply(
    policy: &Policy,
    base: &Profile,
    manifest: &Manifest,
    registry: &ActionRegistry,
    tables: &ReachTables,
    geometry: &KeyboardGeometry,
) -> Result<Applied, CoreError> {
    refuse_entry(base, manifest, policy)?;

    let mut stack = Stack {
        registry,
        tables,
        geometry,
        guard: Guard::new(manifest),
        layout: Layout::new(&base.set),
    };

    let mut reports = Vec::with_capacity(policy.transforms.len());
    for (index, row) in policy.transforms.iter().enumerate() {
        let outcome = match row {
            Transform::RelocateCluster {
                tags,
                from,
                to,
                bound,
                targets,
            } => stack.relocate(tags, *from, *to, *bound, targets)?,
            Transform::SwapPolarity { tags } => stack.swap_polarity(tags),
            Transform::SwapMouseButtons => stack.swap_mouse()?,
        };
        reports.push(TransformReport {
            index,
            kind: row.kind(),
            outcome,
        });
    }

    Ok(Applied {
        base: stack.produce(policy, base),
        reports,
    })
}

/// The entry refusals — the only faults that mean "the application could not be
/// evaluated", and therefore the only ones that refuse instead of reporting.
///
/// Each message is one bounded line naming what was refused: the kind, both pins,
/// the colliding id, or the duplicated action. The pin check is 014's rule re-used
/// rather than restated, spelled from the loaded manifest rather than parsed back
/// out of the base.
fn refuse_entry(base: &Profile, manifest: &Manifest, policy: &Policy) -> Result<(), CoreError> {
    if !matches!(base.kind, Some(ProfileKind::Base)) {
        let message = "`patch` is not a base: a transform applies to a base document".to_string();
        return Err(CoreError::InvalidValue {
            key: "kind".to_string(),
            message,
        });
    }

    let pin = format!("{}@{}", manifest.id, manifest.revision);
    if base.for_manifest != pin {
        let declared = base.for_manifest.as_str();
        let message = format!("`{declared}` is not `{pin}`, the pin this manifest answers");
        return Err(CoreError::InvalidValue {
            key: "for".to_string(),
            message,
        });
    }

    if policy.id == base.id {
        let id = policy.id.as_str();
        let message = format!(
            "`{id}` is both the base and the policy: a produced base cannot \
                               name its own input"
        );
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message,
        });
    }

    if let Some(action) = duplicated_action(&base.set) {
        let name = action.as_str();
        let message = format!(
            "`{name}` is bound twice in the base's `set`: a transform reads and \
             writes one binding per action"
        );
        return Err(CoreError::InvalidValue {
            key: "set".to_string(),
            message,
        });
    }
    Ok(())
}

/// The first action a base's `set` answers twice, if it answers any.
///
/// `profile/v2` calls a repeated action id data ("duplicates are data", and
/// `set-twice-last-element.toml` pins it valid) and lets `resolve` settle the
/// meaning last-writer-wins. That is an answer about a *chain*; a transform
/// stack has to read every precondition off one layout and write into it, so a
/// base holding two copies of one binding gives it two layouts to disagree
/// about. The stack refuses rather than choosing, because either choice is a
/// fact the row never checked: preconditions read one copy and writes land on
/// it while the other copy stays effective, so the row reports `Applied` over a
/// layout that changed nothing — the silent no-op the design refuses (Options
/// considered: Tags) and the hole the `occupied-target` guard and the atomicity
/// guarantee are meant to close. Naming the duplicated action is safe by the
/// same route as every other echoed id: it came through `ActionId::parse`.
fn duplicated_action(set: &[(ActionId, InputId)]) -> Option<&ActionId> {
    let mut seen: BTreeSet<&ActionId> = BTreeSet::new();
    set.iter()
        .find(|(action, _)| !seen.insert(action))
        .map(|(action, _)| action)
}

/// The working layout: the source base's `set` elements, in the source base's
/// element order, with their inputs rewritten in place.
///
/// Element order is not incidental — the produced base's `set` list preserves it,
/// which is what lets a consumer diff two bases as edits rather than as a
/// reshuffled list. Lookups are linear on purpose: a base is bounded by the
/// document budget, and a second index would be a second answer to one fact.
///
/// Two invariants, both settled before the first row runs, keep every lookup
/// below an exact answer rather than the first of several:
///
/// - **one element per action** — `refuse_entry` refuses a base that binds one
///   action twice, so `input_of` and `rebind` cannot disagree about which copy
///   of a binding a row means, and no write can hide behind a duplicate;
/// - **an input may serve several actions** — nothing in `profile/v2` forbids
///   that, and `resolve` keeps every holder, so occupancy questions ask
///   `holders_of`, never one answer at a time.
#[derive(Clone, Debug)]
struct Layout {
    elements: Vec<(ActionId, InputId)>,
}

impl Layout {
    fn new(elements: &[(ActionId, InputId)]) -> Self {
        Self {
            elements: elements.to_vec(),
        }
    }

    /// The input one action currently sits on, or `None` when the base does not
    /// bind it at all — an unbound action has no key to move (A10). One answer,
    /// because the entry refusal admits no base with a second copy of it.
    fn input_of(&self, action: &ActionId) -> Option<&InputId> {
        self.elements
            .iter()
            .find(|(bound, _)| bound == action)
            .map(|(_, input)| input)
    }

    /// The action one input currently serves first, or `None` when nothing does.
    /// `swap-mouse-buttons` reads it that way: the row names the two buttons, and
    /// the action it exchanges is the one it found on the button.
    fn action_on(&self, input: &InputId) -> Option<&ActionId> {
        self.elements
            .iter()
            .find(|(_, bound)| bound == input)
            .map(|(action, _)| action)
    }

    /// Every action one input currently serves, in element order. A base may
    /// bind one key to two actions — the grammar allows it and `resolve` keeps
    /// both — so "is this target free" is a question about all of them. A first
    /// answer alone is how a hostile-but-legal base slips a matched action onto
    /// a key a never-named action also holds.
    fn holders_of(&self, input: &InputId) -> impl Iterator<Item = &ActionId> {
        self.elements
            .iter()
            .filter(move |(_, bound)| bound == input)
            .map(|(action, _)| action)
    }

    /// Moves one action's binding, in place: the element stays where the source
    /// base put it and only its input changes. One element carries the action,
    /// so one write moves it.
    fn rebind(&mut self, action: &ActionId, target: &InputId) {
        for (bound, input) in &mut self.elements {
            if bound == action {
                *input = target.clone();
                return;
            }
        }
    }
}

/// The untouchable guard: the inputs and actions a row may never read or write,
/// read out of the loaded manifest (A6, A23).
///
/// Every mature system carries a set of keys the game will not let move, and the
/// ADR's clause — a hard-docked or exclusive-group key is never a transform
/// target — is only worth having if something enforces it. Here the set is data:
/// `dead-keys`, `docked-actions`, and the members of `exclusive-groups` are
/// reserved inputs, and the action side of a `docked-actions` row is a reserved
/// action. The guard is unconditional; there is no author-side opt-out, because
/// an accessibility surface a policy can opt out of is not one.
struct Guard {
    reserved: BTreeSet<InputId>,
    docked: BTreeMap<ActionId, InputId>,
}

impl Guard {
    fn new(manifest: &Manifest) -> Self {
        let mut reserved: BTreeSet<InputId> = manifest.dead_keys.iter().cloned().collect();
        for group in &manifest.exclusive_groups {
            reserved.extend(group.inputs.iter().cloned());
        }
        let mut docked: BTreeMap<ActionId, InputId> = BTreeMap::new();
        for row in &manifest.docked_actions {
            reserved.insert(row.input.clone());
            docked.insert(row.action.clone(), row.input.clone());
        }
        Self { reserved, docked }
    }

    fn reserves(&self, input: &InputId) -> bool {
        self.reserved.contains(input)
    }

    /// The manifest row docking one action, when one does: the row is what a
    /// refusal quotes, so the report can say which input the game pinned it to.
    fn docked_at(&self, action: &ActionId) -> Option<&InputId> {
        self.docked.get(action)
    }
}

/// One application in progress: the loaded data the rows ask questions of, and
/// the working layout they rewrite.
struct Stack<'a> {
    registry: &'a ActionRegistry,
    tables: &'a ReachTables,
    geometry: &'a KeyboardGeometry,
    guard: Guard,
    layout: Layout,
}

impl Stack<'_> {
    /// The match set: the loaded registry's ids the tag matches, in `ActionId`
    /// byte order — the one order that is stable across a registry round-trip, so
    /// the k-th-with-k-th pairing and the offender a report names are both fixed
    /// by the data rather than by a hash map.
    fn matched(&self, tags: &TagPattern) -> Vec<ActionId> {
        self.registry
            .ids()
            .into_iter()
            .filter(|id| tags.matches(id))
            .collect()
    }

    /// `relocate-cluster`: move one contiguous cluster of matched actions onto a
    /// list of target inputs as a single translation.
    ///
    /// The order of the checks is the order that costs nothing first: is the tag
    /// about anything at all, does every matched id have a binding, do the counts
    /// pair, is the cluster the base actually has a keyboard cluster the guard
    /// permits, is it one cluster, are the targets free of any action the row did
    /// not name, and only then the predicate work. Every fault names its
    /// offender; only `placement-illegal` carries the predicate's check.
    fn relocate(
        &mut self,
        tags: &TagPattern,
        from: Hand,
        to: Hand,
        bound: GradeBound,
        targets: &[InputId],
    ) -> Result<Outcome, CoreError> {
        let matched = self.matched(tags);
        if matched.is_empty() {
            return Ok(unmatched(tags));
        }

        let mut moves: Vec<(ActionId, InputId)> = Vec::with_capacity(matched.len());
        for action in &matched {
            let Some(source) = self.layout.input_of(action) else {
                let name = action.as_str();
                return Ok(fault(
                    NotApplicableReason::UnboundAction,
                    format!("`{name}` is matched by the row and bound by nothing in the base"),
                ));
            };
            moves.push((action.clone(), source.clone()));
        }

        if matched.len() != targets.len() {
            let spelling = tags.spelling();
            let detail = format!(
                "`{spelling}` matches {} ids and the row names {} targets: the pairing has no \
                 answer for the leftover",
                matched.len(),
                targets.len()
            );
            return Ok(fault(NotApplicableReason::MatchCount, detail));
        }

        if let Some(offender) = first_not_keyboard(&moves, targets) {
            return Ok(fault(
                NotApplicableReason::NotKeyboard,
                format!(
                    "`{offender}` is not a keyboard key: grades and coordinates cover the \
                         keyboard only"
                ),
            ));
        }
        if let Some(offender) = self.guard_fault(&moves, targets) {
            return Ok(fault(NotApplicableReason::ReservedInput, offender));
        }
        if !self.is_cluster(&moves) {
            let detail = cluster_detail(&moves);
            return Ok(fault(NotApplicableReason::NotContiguous, detail));
        }
        for target in targets {
            let Some(holder) = self.foreign_holder(target, &matched) else {
                continue;
            };
            let holder_name = holder.as_str();
            let target_name = target.as_str();
            return Ok(fault(
                NotApplicableReason::OccupiedTarget,
                format!("`{target_name}` is bound to `{holder_name}`, which the row never named"),
            ));
        }

        let pairs: Vec<(InputId, InputId)> = moves
            .iter()
            .zip(targets)
            .map(|((_, source), target)| (source.clone(), target.clone()))
            .collect();
        let check = placement_is_legal(from, to, &pairs, bound, self.tables, self.geometry)?;
        if !check.legal {
            let detail = placement_detail(&moves, targets, bound, &check);
            return Ok(illegal(detail, check));
        }

        for ((action, _), target) in moves.iter().zip(targets) {
            self.layout.rebind(action, target);
        }
        Ok(Outcome::Applied)
    }

    /// The action holding one target input that the row never named, if any.
    ///
    /// Every holder of the target is asked, not just the one the element list
    /// happens to carry first. A base may put two actions on one key — the
    /// grammar allows it and `resolve` keeps both — and answering with the holder
    /// that sits inside the match set is how such a base passes the occupancy
    /// check and then takes a matched action onto a key a foreign action still
    /// holds: the displacement the check exists to refuse (Chosen design → The
    /// stack). Walking onto the cluster's own vacated keys stays legal, because
    /// those holders *are* named by the row.
    fn foreign_holder(&self, target: &InputId, matched: &[ActionId]) -> Option<&ActionId> {
        self.layout
            .holders_of(target)
            .find(|holder| !matched.contains(*holder))
    }

    /// `swap-polarity`: exchange the bindings of exactly two `discrete-lateral`
    /// actions. A polarity flip is the one transform whose output is the same
    /// algebra the patch level already ships, so applied twice it is the identity
    /// (A10) — and an unbound side stays unbound, because a flip may not invent a
    /// binding for the half that has none.
    fn swap_polarity(&mut self, tags: &TagPattern) -> Outcome {
        let matched = self.matched(tags);
        if matched.is_empty() {
            return unmatched(tags);
        }
        if matched.len() != 2 {
            let spelling = tags.spelling();
            let detail = format!(
                "`{spelling}` matches {} ids: a polarity is a pair, so one or three has no answer",
                matched.len()
            );
            return fault(NotApplicableReason::MatchCount, detail);
        }
        for action in &matched {
            let kind = self.registry.axis_kind(action);
            if kind != Some(AxisKind::DiscreteLateral) {
                let name = action.as_str();
                let detail = match kind {
                    Some(kind) => format!(
                        "`{name}` is `{}`, and a polarity flip touches only `discrete-lateral`",
                        kind.name()
                    ),
                    None => format!("`{name}` is not an id the loaded registry lists"),
                };
                return fault(NotApplicableReason::AxisKind, detail);
            }
        }
        let moves: Vec<(ActionId, InputId)> = matched
            .iter()
            .filter_map(|action| {
                self.layout
                    .input_of(action)
                    .map(|source| (action.clone(), source.clone()))
            })
            .collect();
        if let Some(offender) = self.guard_fault(&moves, &[]) {
            return fault(NotApplicableReason::ReservedInput, offender);
        }

        let (Some(left), Some(right)) = (
            self.layout.input_of(&matched[0]).cloned(),
            self.layout.input_of(&matched[1]).cloned(),
        ) else {
            return Outcome::Applied;
        };
        self.layout.rebind(&matched[0], &right);
        self.layout.rebind(&matched[1], &left);
        Outcome::Applied
    }

    /// `swap-mouse-buttons`: exchange the actions currently bound to
    /// `mouse:left` and `mouse:right`.
    ///
    /// The row is input-wise where it reads and action-wise where it lands: the
    /// pair is a pair of inputs, and the exchange materializes as two `set`
    /// elements on the produced base. No grid and no grade is consulted — nothing
    /// here moves on a keyboard — so the row's whole precondition set is the
    /// vocabulary plus the manifest's constraints (A5, A19).
    fn swap_mouse(&mut self) -> Result<Outcome, CoreError> {
        let [left, right] = mouse_buttons()?;

        let mut pair: Vec<(ActionId, InputId)> = Vec::with_capacity(2);
        for button in [&left, &right] {
            let Some(action) = self.layout.action_on(button) else {
                let name = button.as_str();
                return Ok(fault(
                    NotApplicableReason::UnboundInput,
                    format!(
                        "`{name}` is bound to no action, and one button cannot be swapped \
                             into nothing"
                    ),
                ));
            };
            pair.push((action.clone(), button.clone()));
        }
        if let Some(offender) = self.mouse_guard_fault(&pair) {
            return Ok(fault(NotApplicableReason::ReservedInput, offender));
        }

        self.layout.rebind(&pair[0].0, &right);
        self.layout.rebind(&pair[1].0, &left);
        Ok(Outcome::Applied)
    }

    /// The guard over a relocate or polarity row: a matched action the game
    /// hard-docks, a source key it reserves, and — last, because a target is the
    /// row's own proposal rather than a fact about the base — a target key it
    /// reserves.
    fn guard_fault(&self, moves: &[(ActionId, InputId)], targets: &[InputId]) -> Option<String> {
        for (action, source) in moves {
            if let Some(docked) = self.guard.docked_at(action) {
                let name = action.as_str();
                let key = docked.as_str();
                return Some(format!(
                    "`{name}` is hard-docked to `{key}` by the manifest"
                ));
            }
            if self.guard.reserves(source) {
                return Some(self.reserved_detail(source));
            }
        }
        targets
            .iter()
            .find(|target| self.guard.reserves(target))
            .map(|target| self.reserved_detail(target))
    }

    /// The same guard over the mouse pair, where the input is the thing the row
    /// names: the button itself is checked before the action sitting on it, so a
    /// report says which button the game pinned.
    fn mouse_guard_fault(&self, pair: &[(ActionId, InputId)]) -> Option<String> {
        for (action, button) in pair {
            if self.guard.reserves(button) {
                return Some(self.reserved_detail(button));
            }
            if let Some(docked) = self.guard.docked_at(action) {
                let name = action.as_str();
                let key = button.as_str();
                let pinned = docked.as_str();
                return Some(format!(
                    "`{key}` serves `{name}`, which the manifest hard-docks to `{pinned}`"
                ));
            }
        }
        None
    }

    fn reserved_detail(&self, input: &InputId) -> String {
        let name = input.as_str();
        format!(
            "`{name}` is reserved by the manifest: a hard-docked, dead, or exclusive-group \
                 key is never a transform input"
        )
    }

    /// Whether the matched keys are one cluster: every key reachable from the
    /// first by rook steps through other matched keys. This is the computation
    /// core does not ship, because a cluster is a transform's concept — a base
    /// whose two steering keys sit three columns apart simply has no cluster to
    /// relocate, and saying so beats mirroring something across the grid (A21).
    ///
    /// A source the grid does not place is not "not contiguous": a missing
    /// coordinate is a fact the contract lacks, so this says yes and lets the
    /// predicate answer with its `InvalidValue` (A4, A5).
    fn is_cluster(&self, moves: &[(ActionId, InputId)]) -> bool {
        let mut grid: BTreeSet<(u32, u32)> = BTreeSet::new();
        for (_, source) in moves {
            let Some(coordinate) = self.geometry.coordinate(source) else {
                return true;
            };
            grid.insert(coordinate);
        }
        if grid.len() < 2 {
            return true;
        }
        let Some(start) = grid.iter().next().copied() else {
            return true;
        };
        let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
        let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
        seen.insert(start);
        queue.push_back(start);
        while let Some((column, row)) = queue.pop_front() {
            for next in rook_neighbours(column, row) {
                if grid.contains(&next) && seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        seen.len() == grid.len()
    }

    /// The produced base: an ordinary `profile/v2` base document, so a consumer
    /// puts it in a chain and gates it by validation rather than trusting the
    /// fact that a policy wrote it (A7, A18).
    ///
    /// `derives`, `clear`, and `swap` are empty — a base sits at the bottom of a
    /// chain and its layout *is* its `set` list — `for` is copied from the source
    /// base, whose pin equality already joined it to the manifest, and `id` is the
    /// policy's, which is why the entry refusal refuses a policy naming its own
    /// base. `generated-by` records what produced the document and is never read
    /// back as trust.
    fn produce(&self, policy: &Policy, source: &Profile) -> Profile {
        Profile {
            version: source.version.clone(),
            id: policy.id.clone(),
            derives: Vec::new(),
            kind: Some(ProfileKind::Base),
            for_manifest: source.for_manifest.clone(),
            set: self.layout.elements.clone(),
            clear: Vec::new(),
            swap: Vec::new(),
            seeded_from: source.seeded_from.clone(),
            generated_by: Some(format!("policy:{}", policy.id)),
        }
    }
}

/// The verdict shared by both tagged kinds when the tag reaches nothing: a
/// grammar-valid tag matching no loaded id is the row being about nothing, and
/// the report says so rather than leaving the author to guess (A1, A3).
fn unmatched(tags: &TagPattern) -> Outcome {
    let spelling = tags.spelling();
    fault(
        NotApplicableReason::UnmatchedTags,
        format!("no id in the loaded registry matches `{spelling}`"),
    )
}

/// A verdict with no evidence but its offender.
fn fault(reason: NotApplicableReason, offender: impl Into<String>) -> Outcome {
    Outcome::NotApplicable(NotApplicable {
        reason,
        offender: Some(offender.into()),
        check: None,
    })
}

/// The `placement-illegal` verdict: the reason, the line that explains which
/// clause failed, and the [`PlacementCheck`] itself as the evidence (A4).
fn illegal(detail: String, check: PlacementCheck) -> Outcome {
    Outcome::NotApplicable(NotApplicable {
        reason: NotApplicableReason::PlacementIllegal,
        offender: Some(detail),
        check: Some(check),
    })
}

/// The two members of the closed input vocabulary the mouse row is defined over,
/// read through the shipped grammar rather than trusted as a literal: the
/// vocabulary owns the rule, so a member the grammar stops admitting is a
/// refusal about the vocabulary rather than a row that quietly does nothing.
fn mouse_buttons() -> Result<[InputId; 2], CoreError> {
    let left = InputId::parse(MOUSE_LEFT)?;
    let right = InputId::parse(MOUSE_RIGHT)?;
    Ok([left, right])
}

/// The first key, source or target, that the reach and grid data cannot speak of
/// because it is not a keyboard input. Sources are asked first: they are facts
/// about the base, and a base that carries a gamepad cluster is the case an
/// author needs told about.
fn first_not_keyboard<'a>(
    moves: &'a [(ActionId, InputId)],
    targets: &'a [InputId],
) -> Option<&'a str> {
    for (_, source) in moves {
        if source.namespace() != InputNamespace::Keyboard {
            return Some(source.as_str());
        }
    }
    targets
        .iter()
        .find(|target| target.namespace() != InputNamespace::Keyboard)
        .map(InputId::as_str)
}

/// The keys a row was trying to move, spelled, for the report that says they are
/// not one cluster.
fn cluster_detail(moves: &[(ActionId, InputId)]) -> String {
    let keys: Vec<&str> = moves.iter().map(|(_, source)| source.as_str()).collect();
    let joined = keys.join(", ");
    format!(
        "the matched keys {joined} are not joined by rook-adjacency in the loaded grid: there \
             is no cluster here to move"
    )
}

/// The evidence line of a `placement-illegal` verdict. Which clause failed is
/// worth saying: a shape failure means the row did not describe one translation,
/// and a bound failure means it did but landed further out than the row allowed.
fn placement_detail(
    moves: &[(ActionId, InputId)],
    targets: &[InputId],
    bound: GradeBound,
    check: &PlacementCheck,
) -> String {
    let limit = bound.0;
    if !check.shape_ok {
        return "the move set is not one translation: `shape_ok` is false, so no bound makes it \
                legal"
            .to_string();
    }
    let worst = check.worst_degradation().unwrap_or_default();
    let mut named = String::new();
    for (index, ((action, source), target)) in moves.iter().zip(targets).enumerate() {
        let Some(degradation) = check.degradations.get(index) else {
            continue;
        };
        if *degradation > i64::from(limit) {
            let name = action.as_str();
            let from = source.as_str();
            let to = target.as_str();
            named = format!("`{name}` from `{from}` to `{to}` degrades by {degradation} ranks");
        }
    }
    if named.is_empty() {
        named =
            format!("the worst degradation is {worst} ranks past the declared bound of {limit}");
    }
    named
}

/// The four keys rook-adjacent to one coordinate: the grid is the only source of
/// adjacency, which is why another geometry needs no new logic (ADR-012).
fn rook_neighbours(column: u32, row: u32) -> Vec<(u32, u32)> {
    let steps = [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)];
    steps
        .iter()
        .filter_map(|(dx, dy)| {
            let Ok(column) = u32::try_from(i64::from(column) + dx) else {
                return None;
            };
            let Ok(row) = u32::try_from(i64::from(row) + dy) else {
                return None;
            };
            Some((column, row))
        })
        .collect()
}
