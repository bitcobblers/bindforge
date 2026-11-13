//! The cross-file chain machinery of ADR-014 (design 014): `load_chain` reads
//! one profile chain out of the major directory its head sits in, `resolve`
//! answers what layout that chain produces against one manifest, and `fork`,
//! `diff`, and `merge` are chain operations over documents.
//!
//! What lives here is only what needs *several* documents in hand. Every rule
//! a single document can answer on its own — the entry set, the id grammars,
//! the `kind`/`derives` shape, the pin grammar, the delta budgets — is a
//! `document` parse rule, which is what keeps `schemas/profile/v2/fixtures/`
//! single-document like every other format's corpus (A16). `chain.rs` keeps the
//! shipped single-document guard byte-exact and this module applies the same
//! two refusals — a repeated id is `Cycle`, more than `MAX_CHAIN_DEPTH`
//! documents opened is `ChainTooDeep` — to the *resolved* chain, where a
//! two-file cycle is visible for the first time (A2, A15).
//!
//! # The one-directory rule
//!
//! Every link is `<profile-id>.toml` sitting directly in the head's own `vN/`
//! directory: no search path, no hub, and no second spelling of where an
//! ancestor lives. That closes the mixed-major question by construction (one
//! directory, one grammar), makes a hub upload bundle exactly one such
//! directory (ADR-018 transports this shape, it does not define it), and is
//! only safe because every `derives` element is slug-grammar checked at parse —
//! so the path this module builds is always a direct child, never `..` (A5,
//! A14). Where a player's personal chain lives on disk stays undecided (A11);
//! decided here is how a chain resolves once a head is given.
//!
//! # Determinism is a compatibility surface
//!
//! `resolve` implements the six rules design 014 states once, and a change to
//! any of them rides a future `profile/v3`: links apply deepest-first; within a
//! link the lists apply in the fixed `set` → `clear` → `swap` order and elements
//! apply sequentially in array order, which is data; last write wins, `clear`
//! unbinds, `swap` exchanges and an unbound side stays unbound; an unbound
//! action contributes nothing; output follows the manifest's `actions` row
//! order; an unbound base resolves to the empty layout.
//!
//! Refusals stay the contract's: every fault here is `InvalidValue`, `Cycle`,
//! `ChainTooDeep`, `Io`, or the one new variant `ChainLinkMissing`. The new
//! variant answers only a link this module's walk constructed from a `derives`
//! element and cannot find: it names the contract object, not a filesystem
//! detail (A18), and the id it carries is one the parser already accepted. A
//! head the caller named is `Io`, whose variant carries the path for
//! `source_path`, because a head's file stem has never been validated as a
//! profile id (A13). Each refusal is rendered by `error`'s single-line,
//! escaping, truncating path, and a local path never enters a message: the link
//! id and the file name say everything a reader needs. The directory-major rule
//! keeps the single spelling the shipped loaders share — `check_dir_version`,
//! same rule, same variant — so a chain cannot answer a variant no other loader
//! answers for the identical fault (A5).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::action::ActionId;
use crate::chain::MAX_CHAIN_DEPTH;
use crate::document::{
    Document, MAJOR_V2, Profile, ProfileKind, check_dir_version, check_profile_slug, parse_file,
};
use crate::error::CoreError;
use crate::input::InputId;
use crate::manifest::Manifest;

/// One profile chain: the documents `load_chain` opened, deepest base first …
/// head last. Only `load_chain` builds one, because a walk that never reached a
/// base is not a chain; mounting a lower rung is `load_chain`ing that link as
/// head, never editing this list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    links: Vec<Profile>,
}

impl Chain {
    /// The chain's documents, deepest first, head last.
    #[must_use]
    pub fn links(&self) -> &[Profile] {
        &self.links
    }

    /// The head — the document the chain was loaded from.
    #[must_use]
    pub fn head(&self) -> Option<&Profile> {
        self.links.last()
    }
}

/// One resolved binding: an action of the joined manifest and the input the
/// chain binds to it. An action with no binding simply is not in the output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedBinding {
    pub action: ActionId,
    pub input: InputId,
}

/// Loads one profile chain from its head document.
///
/// The walk is iterative, and each step applies the shipped guards to what it
/// has opened so far:
///
/// 1. the head is read through the version gate and the parse budget, and its
///    `vN/` directory pins — and is pinned by — its declared major through the
///    one helper every shipped loader applies for that rule (A5). A head that is
///    not there is `parse_file`'s `Io`, which carries the path;
///    `ChainLinkMissing` is never the answer about a head, whose file stem has
///    not been validated as a profile id;
/// 2. each link's one `derives` element names the next document, always
///    `<major dir>/<slug>.toml`; an absent file is `ChainLinkMissing` naming
///    the id (a chain is a contract object — A18) and a document whose `id`
///    contradicts its file name is `InvalidValue`, the same location rule
///    `load_manifest` applies to a game directory;
/// 3. before any descent, a repeated id is `Cycle` and one more document past
///    `MAX_CHAIN_DEPTH` is `ChainTooDeep` — the guard's unit is documents
///    opened, one per link, and both refuse instead of descending (A1, A2);
/// 4. after the walk the deepest link is a `base` and every link above it a
///    `patch`: a label contradicting its position is `InvalidValue` naming the
///    link (Options: what `kind` means).
///
/// Every document opened — fixture, `data/profiles/`, a hub bundle, or a
/// personal chain — goes through one version gate, one byte budget, and these
/// chain guards, source-independent. Trust stays with adoption (ADR Decision
/// Four); nothing here decides whether a chain is *good*, only what it says.
pub fn load_chain(head: &Path) -> Result<Chain, CoreError> {
    let directory = head.parent().unwrap_or(Path::new("."));
    let mut links: Vec<Profile> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    // The head is the caller's path, so step 2's completeness refusal does not
    // apply to it: `parse_file`'s `Io` already says which path could not be read
    // (A13), and nothing here echoes an unvalidated file stem as a profile id.
    let expected = link_stem(head)?;
    let mut link = open_head(directory, head, expected)?;
    loop {
        if !seen.insert(link.id.clone()) {
            return Err(CoreError::Cycle);
        }
        let below = link_below(&link)?.map(str::to_owned);
        links.push(link);
        let Some(ancestor) = below else {
            break;
        };
        if links.len() >= MAX_CHAIN_DEPTH {
            return Err(CoreError::ChainTooDeep);
        }
        let path = directory.join(format!("{ancestor}.toml"));
        link = open_ancestor(directory, &path, &ancestor)?;
    }

    links.reverse();
    check_positions(&links)?;
    Ok(Chain { links })
}

/// Resolves a chain against the manifest it is pinned to: the deterministic
/// layout, in the manifest's `actions` row order, containing exactly the
/// actions some link leaves bound.
///
/// Three joins refuse by name before any layout is answered, because a chain
/// resolved against the wrong facts is the silent drift the design prices (A20):
/// every link's `for` must equal this manifest's `"<id>@<revision>"` exactly, id
/// *and* revision; every action id in any delta list must be a row of this
/// manifest; and a `seeded-from` must name a preset this manifest ships —
/// provenance must name a real thing, which is still not the same as anything
/// consuming it as trust (ADR Decision Four).
pub fn resolve(chain: &Chain, manifest: &Manifest) -> Result<Vec<ResolvedBinding>, CoreError> {
    check_joins(chain, manifest)?;

    let mut bound: BTreeMap<ActionId, InputId> = BTreeMap::new();
    for link in &chain.links {
        for (action, input) in &link.set {
            bound.insert(action.clone(), input.clone());
        }
        for action in &link.clear {
            bound.remove(action);
        }
        for (first, second) in &link.swap {
            // Rule 3: one exchange, both sides moving. An unbound side stays
            // unbound and the bound side keeps its binding, so a `swap` never
            // fabricates or destroys a binding no `set` or `clear` wrote.
            let left = bound.get(first).cloned();
            let right = bound.get(second).cloned();
            if let (Some(left), Some(right)) = (left, right) {
                bound.insert(first.clone(), right);
                bound.insert(second.clone(), left);
            }
        }
    }

    // Rule 5: the manifest's row order is the output order — no sort rule of
    // this module's own, following the "row order is data" precedent (A17).
    let mut layout = Vec::new();
    for row in &manifest.actions {
        if let Some(input) = bound.get(&row.action) {
            layout.push(ResolvedBinding {
                action: row.action.clone(),
                input: input.clone(),
            });
        }
    }
    Ok(layout)
}

/// Builds the appended patch a `fork` is: the caller's fresh slug id, `derives`
/// naming the chain's head, `kind = patch`, and the head's `for` inherited so
/// the forked chain loads against the same manifest. The delta lists are empty,
/// which makes the result a *document*, not a chain: `load_chain` refuses the
/// shell as the empty-patch case until the caller fills one element in.
///
/// Copying the chain's files is the consumer's move — core builds documents, not
/// filesystems, and storage location is ADR-017's.
pub fn fork(chain: &Chain, id: &str) -> Result<Profile, CoreError> {
    check_fresh_id(id, chain.links.iter())?;
    let Some(head) = chain.head() else {
        return Err(no_head("fork"));
    };
    Ok(new_patch(
        id,
        vec![head.id.clone()],
        head.for_manifest.clone(),
    ))
}

/// Builds the patch that turns `a`'s layout into `b`'s — the diff *is* the
/// grammar, so its output is an ordinary v2 patch that can be written beside
/// `a`'s head and `load_chain`ed as the new head.
///
/// Two preconditions refuse naming both heads: the chains' deepest links must
/// be the same *document* (equal parsed `Profile`s — an id is unique only within
/// one directory, and chain identity is the documents' contents), and their
/// resolved layouts must differ, because there is no legal non-empty patch
/// between chains that already agree.
pub fn diff(a: &Chain, b: &Chain, manifest: &Manifest, id: &str) -> Result<Profile, CoreError> {
    check_fresh_id(id, a.links.iter())?;
    let (head_a, head_b) = match (a.head(), b.head()) {
        (Some(head_a), Some(head_b)) => (head_a, head_b),
        _ => return Err(no_head("diff")),
    };
    if a.links.first() != b.links.first() {
        let message = format!(
            "`{}` and `{}` do not share their deepest link: `diff` answers what changed \
             between two chains over one base document",
            head_a.id, head_b.id
        );
        return Err(CoreError::InvalidValue {
            key: "derives".to_string(),
            message,
        });
    }
    let before = resolve(a, manifest)?;
    let after = resolve(b, manifest)?;
    if before == after {
        let message = format!(
            "`{}` and `{}` resolve to the same layout: a diff is a patch, and the patch with \
             no delta is refused",
            head_a.id, head_b.id
        );
        return Err(CoreError::InvalidValue {
            key: "set".to_string(),
            message,
        });
    }

    let mut set: Vec<(ActionId, InputId)> = Vec::new();
    let mut clear: Vec<ActionId> = Vec::new();
    for row in &manifest.actions {
        let before = before.iter().find(|binding| binding.action == row.action);
        let after = after.iter().find(|binding| binding.action == row.action);
        if before == after {
            continue;
        }
        match after {
            None => clear.push(row.action.clone()),
            Some(binding) => set.push((row.action.clone(), binding.input.clone())),
        }
    }

    let mut patch = new_patch(id, vec![head_a.id.clone()], manifest_pin(manifest));
    patch.set = set;
    patch.clear = clear;
    Ok(patch)
}

/// Composes two patches that sit over one base into the one patch that does the
/// same work: the caller's fresh id, `derives` and `for` inherited from the
/// lower patch, and the two delta lists composed in chain order — `p1` below,
/// `p2` above — over every action either of them touches.
///
/// The composition is exact, not heuristic: it is worked out over symbols
/// (`p1`'s and `p2`'s effect on the base's own bindings, which `merge` never
/// reads) and then emitted in the fixed `set` → `clear` → `swap` order and
/// *re-checked* against those symbols. When an action's composed effect has no
/// one-patch form under that order — a lower `swap X Y` with both sides bound
/// and an upper `clear X`, where the net needs the `swap` to run first — the
/// whole merge refuses with `InvalidValue` naming the colliding action id: no
/// conflict markers outside the grammar, no silent guessing (A20). A
/// composition that cancels to the identity refuses as the empty-patch case.
///
/// Compatible overlaps are not conflicts: equal `set`s emit once, equal
/// `clear`s emit once, and a `set` under a `clear` composes to the lone `clear`.
/// An exchange the composition keeps emits exactly one `swap` element, whichever
/// half of the pair the touch order reaches first.
///
/// Two things the merged document does not carry. It carries no provenance
/// label: `generated-by` names what produced the document it is written on, and
/// neither operand produced this one, so a label taken from an operand would
/// state a fact no one established — the caller writes its own (Provenance
/// labels). And the composition assumes the base binds **both** sides of any
/// exchange either patch names; a merge that would need otherwise has no
/// one-patch form on the symbols, so it refuses rather than answering a layout
/// that depends on the base's contents `merge` never reads.
pub fn merge(p1: &Profile, p2: &Profile, id: &str) -> Result<Profile, CoreError> {
    check_operand(p1)?;
    check_operand(p2)?;
    check_fresh_id(id, [p1, p2].into_iter())?;
    if p1.derives != p2.derives {
        let message = format!(
            "`{}` derives `{}` and `{}` derives `{}`: `merge` composes two patches over one \
             base, or answers nothing",
            p1.id,
            base_spelling(p1),
            p2.id,
            base_spelling(p2)
        );
        return Err(CoreError::InvalidValue {
            key: "derives".to_string(),
            message,
        });
    }
    if p1.for_manifest != p2.for_manifest {
        let message = format!(
            "`{}` is pinned `{}` and `{}` is pinned `{}`: the pin equality rule is one pin for \
             one merge",
            p1.id, p1.for_manifest, p2.id, p2.for_manifest
        );
        return Err(CoreError::InvalidValue {
            key: "for".to_string(),
            message,
        });
    }

    let touched = merge_touch_order(p1, p2);
    let composed = compose(&touched, p1, p2);
    let delta = emit_composition(&touched, &composed)?;
    if delta.set.is_empty() && delta.clear.is_empty() && delta.swap.is_empty() {
        let message = format!(
            "`{}` and `{}` compose to the identity: the merged patch would carry no delta, and \
             the patch with no delta is refused",
            p1.id, p2.id
        );
        return Err(CoreError::InvalidValue {
            key: "set".to_string(),
            message,
        });
    }

    let mut patch = new_patch(id, p1.derives.clone(), p1.for_manifest.clone());
    patch.set = delta.set;
    patch.clear = delta.clear;
    patch.swap = delta.swap;
    Ok(patch)
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Opens the chain's head — the caller's own path, read through the same budget
/// and version gate as every other contract file. A head that is not there is
/// `parse_file`'s `Io`, whose variant carries the path, not `ChainLinkMissing`:
/// the completeness refusal belongs to a link the walk constructed, and a head's
/// file stem has never been validated as a profile id (A13, A18).
fn open_head(directory: &Path, path: &Path, expected: &str) -> Result<Profile, CoreError> {
    let document = parse_file(path).map_err(|err| name_the_link(err, expected))?;
    check_link(directory, expected, document)
}

/// Opens one link of the chain the walk constructed: the file the `derives`
/// element names, in the head's own directory, and then held to the two
/// chain-level rules that file name carries. An absent file here is the named
/// completeness refusal, because the id is parse-validated and the directory
/// rule already fixed where that document has to be (A18).
fn open_ancestor(directory: &Path, path: &Path, expected: &str) -> Result<Profile, CoreError> {
    if !path.try_exists().map_err(|err| read_fault(path, err))? {
        return Err(CoreError::ChainLinkMissing {
            id: expected.to_string(),
        });
    }
    let document = parse_file(path).map_err(|err| name_the_link(err, expected))?;
    check_link(directory, expected, document)
}

/// The two rules a chain link's file name carries: its declared major against
/// the `vN/` directory it was read from — the same rule `check_dir_version`
/// states for every shipped loader, answered with the same variant so the chain
/// cannot drift into a second spelling of it (A5) — and its `id` against its
/// name, which is the lookup every link above it resolves by.
fn check_link(directory: &Path, expected: &str, document: Document) -> Result<Profile, CoreError> {
    let Document::Profile(link) = document else {
        let file_name = format!("{expected}.toml");
        let message = format!("`{file_name}` must hold a profile document");
        return Err(CoreError::InvalidValue {
            key: "format".to_string(),
            message,
        });
    };
    check_dir_version(directory, &link.version, &format!("{expected}.toml"))?;
    if link.id != expected {
        let named = &link.id;
        let message = format!(
            "`{expected}.toml` holds the profile `{named}`: a chain link's file name must equal \
             its `id` byte-for-byte, because the id is the lookup every link above it resolves by"
        );
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message,
        });
    }
    Ok(link)
}

/// The file stem a head path is named for. A path with no usable stem cannot
/// name a profile document at all; every link the walk builds is named from a
/// slug the parser already accepted.
fn link_stem(path: &Path) -> Result<&str, CoreError> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| CoreError::InvalidValue {
            key: "id".to_string(),
            message: "a chain link's file is named for the profile id it holds".to_string(),
        })
}

/// A failed read of a chain file: the path rides the variant, never the message
/// (`source_path` is the logging channel for it).
fn read_fault(path: &Path, err: std::io::Error) -> CoreError {
    CoreError::Io {
        path: path.display().to_string(),
        message: err.to_string(),
    }
}

/// Names the link a parse-level refusal came from. A link's own grammar fault
/// is also a fact about the chain — this is the link that cannot be loaded — so
/// the message carries the link id alongside the fault, and every refusal a
/// chain answers names the link it refused.
fn name_the_link(err: CoreError, link: &str) -> CoreError {
    match err {
        CoreError::InvalidValue { key, message } => CoreError::InvalidValue {
            key,
            message: format!("`{link}`: {message}"),
        },
        other => other,
    }
}

/// The one link below a loaded link, or `None` where the chain terminates. A v2
/// patch carries exactly one `derives` element — the shape rules refuse any
/// other count at parse — so a chain is one path, never a tree, and the walk
/// needs no backtracking.
fn link_below(link: &Profile) -> Result<Option<&str>, CoreError> {
    match link.kind {
        None => Err(link_fault(link, "a chain link is a `profile/v2` document")),
        Some(ProfileKind::Base) => Ok(None),
        Some(ProfileKind::Patch) => Ok(link.derives.first().map(String::as_str)),
    }
}

/// Step 4: the `kind` label checked against chain position — the bottom link is
/// the base every patch above it modifies. The per-document shape rules already
/// refuse the other contradictions, so this is the position half of the same
/// rule, stated where the whole chain is in hand.
fn check_positions(links: &[Profile]) -> Result<(), CoreError> {
    for (index, link) in links.iter().enumerate() {
        let expected = if index == 0 {
            ProfileKind::Base
        } else {
            ProfileKind::Patch
        };
        if link.kind != Some(expected) {
            let bottom = if index == 0 {
                "the bottom of a chain"
            } else {
                "a link above the bottom"
            };
            let message = format!(
                "`{}` carries `kind = {}` in {bottom}, where a `{}` link belongs",
                link.id,
                link.kind.map_or("no kind", |kind| kind.name()),
                expected.name()
            );
            return Err(CoreError::InvalidValue {
                key: "kind".to_string(),
                message,
            });
        }
    }
    Ok(())
}

/// The `InvalidValue` for a link that contradicts its role, naming the link.
fn link_fault(link: &Profile, reason: &str) -> CoreError {
    let id = &link.id;
    CoreError::InvalidValue {
        key: "kind".to_string(),
        message: format!("`{id}`: {reason}"),
    }
}

// ---------------------------------------------------------------------------
// The load-time joins
// ---------------------------------------------------------------------------

/// The three joins `resolve` performs, each refusing by name.
fn check_joins(chain: &Chain, manifest: &Manifest) -> Result<(), CoreError> {
    let pin = manifest_pin(manifest);
    for link in &chain.links {
        if link.for_manifest != pin {
            let id = &link.id;
            let link_pin = &link.for_manifest;
            let message = format!(
                "`{id}` is pinned `{link_pin}`, not the manifest `{pin}` this chain is joined \
                 to: a chain re-pinned under links that still name the old revision refuses \
                 rather than re-resolving the same patch over changed game facts"
            );
            return Err(CoreError::InvalidValue {
                key: "for".to_string(),
                message,
            });
        }
    }
    for link in &chain.links {
        let Some(preset) = &link.seeded_from else {
            continue;
        };
        if !manifest.presets.iter().any(|listed| listed == preset) {
            let message = format!(
                "`{preset}` is no preset the joined manifest ships: provenance must name a real \
                 thing, even though nothing consumes it as trust"
            );
            return Err(CoreError::InvalidValue {
                key: "seeded-from".to_string(),
                message,
            });
        }
    }
    // The action join gets one lookup per resolution, not one scan of the
    // action table per touched action — the cost paragraph's "one per-resolution
    // lookup of the `MAX_MANIFEST_ROWS`-capped action table" (A21). Row order
    // stays untouched: this set only answers "is this a row?".
    let rows: BTreeSet<&ActionId> = manifest.actions.iter().map(|row| &row.action).collect();
    for link in &chain.links {
        for action in touched_actions(link) {
            if !rows.contains(action) {
                let text = action.as_str();
                let message = format!(
                    "`{text}` is no action of the joined manifest: grammar-valid is not joined \
                     to this game's action table"
                );
                return Err(CoreError::InvalidValue {
                    key: "actions".to_string(),
                    message,
                });
            }
        }
    }
    Ok(())
}

/// Design 013's canonical identity spelling — the pin every link carries and
/// the join every chain is checked against, id *and* revision (A7).
fn manifest_pin(manifest: &Manifest) -> String {
    format!("{}@{}", manifest.id, manifest.revision)
}

/// Every action id one link's delta lists name, in list order and deduplicated:
/// the set the manifest join runs over, and the set a composition works on.
fn touched_actions(link: &Profile) -> Vec<&ActionId> {
    let mut touched: Vec<&ActionId> = Vec::new();
    for (action, _) in &link.set {
        if !touched.contains(&action) {
            touched.push(action);
        }
    }
    for action in &link.clear {
        if !touched.contains(&action) {
            touched.push(action);
        }
    }
    for (first, second) in &link.swap {
        if !touched.contains(&first) {
            touched.push(first);
        }
        if !touched.contains(&second) {
            touched.push(second);
        }
    }
    touched
}

// ---------------------------------------------------------------------------
// The chain operations
// ---------------------------------------------------------------------------

/// The fresh id a chain operation is asked for: the slug grammar every v2 id
/// rides, and no collision with an id already in play — a chain looks its links
/// up by id in one directory, so a second link with one id is two answers to
/// one lookup.
fn check_fresh_id<'a>(id: &str, taken: impl Iterator<Item = &'a Profile>) -> Result<(), CoreError> {
    check_profile_slug("id", id)?;
    for link in taken {
        if link.id == id {
            let message = format!(
                "`{id}` is already a link id here: a chain looks its links up by id in one \
                 directory, so one id cannot name two documents"
            );
            return Err(CoreError::InvalidValue {
                key: "id".to_string(),
                message,
            });
        }
    }
    Ok(())
}

/// A chain operation operand must be a `profile/v2` patch: a v1 profile carries
/// no kind, no pin, and no deltas, so composing it would invent all three.
fn check_operand(operand: &Profile) -> Result<(), CoreError> {
    if operand.kind != Some(ProfileKind::Patch) {
        let id = &operand.id;
        let message =
            format!("`{id}` is not a `profile/v2` patch, and `merge` composes two patches");
        return Err(CoreError::InvalidValue {
            key: "kind".to_string(),
            message,
        });
    }
    Ok(())
}

/// The refusal for an operation asked of an empty chain — only `load_chain`
/// builds a `Chain`, so this is a defensive corner, not a documented one.
fn no_head(operation: &str) -> CoreError {
    CoreError::InvalidValue {
        key: "derives".to_string(),
        message: format!("`{operation}` needs a chain with a head"),
    }
}

/// The document a chain operation answers with: a v2 patch shell the caller
/// fills deltas into.
fn new_patch(id: &str, derives: Vec<String>, for_manifest: String) -> Profile {
    Profile {
        version: MAJOR_V2.to_string(),
        id: id.to_string(),
        derives,
        kind: Some(ProfileKind::Patch),
        for_manifest,
        set: Vec::new(),
        clear: Vec::new(),
        swap: Vec::new(),
        seeded_from: None,
        generated_by: None,
    }
}

/// How a patch's `derives` reads in a refusal naming a base mismatch.
fn base_spelling(patch: &Profile) -> String {
    if patch.derives.is_empty() {
        return "nothing".to_string();
    }
    patch.derives.join(", ")
}

/// What one action's binding is while a composition is being worked out: the
/// base's own binding — a symbol, because `merge` never reads the base document
/// — a concrete input some `set` wrote, or unbound.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Binding {
    Base(ActionId),
    Written(InputId),
    Unbound,
}

/// The three delta lists one composed result is written as.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ComposedDelta {
    set: Vec<(ActionId, InputId)>,
    clear: Vec<ActionId>,
    swap: Vec<(ActionId, ActionId)>,
}

/// The actions a merge composes over, in the order the proposal pins: every
/// action the lower patch touches, in its touch order, then the actions only the
/// upper patch touches, in its order. That order is the emitted element order.
fn merge_touch_order(p1: &Profile, p2: &Profile) -> Vec<ActionId> {
    let mut touched: Vec<ActionId> = Vec::new();
    for action in touched_actions(p1) {
        touched.push(action.clone());
    }
    for action in touched_actions(p2) {
        if !touched.iter().any(|listed| listed == action) {
            touched.push(action.clone());
        }
    }
    touched
}

/// Runs the two patches' deltas in chain order over symbolic bindings, `p1`
/// below and `p2` above, applying each link's lists in the fixed
/// `set` → `clear` → `swap` order and each list's elements in array order.
fn compose(touched: &[ActionId], p1: &Profile, p2: &Profile) -> BTreeMap<ActionId, Binding> {
    let mut state: BTreeMap<ActionId, Binding> = BTreeMap::new();
    for action in touched {
        state.insert(action.clone(), Binding::Base(action.clone()));
    }
    apply_lists(&mut state, &p1.set, &p1.clear, &p1.swap);
    apply_lists(&mut state, &p2.set, &p2.clear, &p2.swap);
    state
}

/// One link's deltas applied to a symbolic state: rule 2 (fixed list order,
/// elements in array order) and rule 3's list shapes, over symbols rather than
/// inputs. The `swap` here always moves, because `Base` stands for the binding
/// the base document carries: **the model assumes the base binds both sides of
/// every exchange either patch names**, and a merge that would need otherwise
/// refuses (Options: `merge`'s conflict rule). That is what makes a `swap` under
/// a `clear` of one swapped action non-representable, and it is the deliberate
/// trade: `merge` never opens the base, so a composition whose answer depends on
/// whether the base binds one side — `set X a` under `swap X Y` — would be a
/// guess about a document it has not read, and it is refused instead. `resolve`
/// runs the same shapes over concrete bindings, where the all-unbound baseline
/// makes a side genuinely unbound and rule 3 says an unbound side stays unbound.
fn apply_lists(
    state: &mut BTreeMap<ActionId, Binding>,
    set: &[(ActionId, InputId)],
    clear: &[ActionId],
    swap: &[(ActionId, ActionId)],
) {
    for (action, input) in set {
        state.insert(action.clone(), Binding::Written(input.clone()));
    }
    for action in clear {
        state.insert(action.clone(), Binding::Unbound);
    }
    for (first, second) in swap {
        let left = state
            .get(first)
            .cloned()
            .unwrap_or(Binding::Base(first.clone()));
        let right = state
            .get(second)
            .cloned()
            .unwrap_or(Binding::Base(second.clone()));
        state.insert(first.clone(), right);
        state.insert(second.clone(), left);
    }
}

/// Writes the composition out as one patch's delta lists and proves the patch
/// means what the composition means: the emitted lists are re-applied over the
/// same symbols and every touched action must come back identical. Anything
/// that fails that check has no one-patch form under the fixed list order, and
/// `merge` refuses the whole result naming the colliding action rather than
/// shipping a patch that means something else.
///
/// A reciprocal pair is one element, not two. The touch order names both halves
/// of every exchange, so when the loop reaches the partner of a pair it already
/// emitted it steps past it; faulting that visit would refuse every composition
/// that *keeps* an exchange — a `swap` the two patches never both touch is a
/// compatible overlap, not a conflict — while a one-sided composition (a kept
/// `swap` whose other half a `clear` or `set` rewrote) still fails the
/// reciprocity check and refuses.
fn emit_composition(
    touched: &[ActionId],
    composed: &BTreeMap<ActionId, Binding>,
) -> Result<ComposedDelta, CoreError> {
    let mut emitted = ComposedDelta::default();
    let mut exchanged: BTreeSet<ActionId> = BTreeSet::new();

    for action in touched {
        match composed.get(action) {
            Some(Binding::Written(input)) => emitted.set.push((action.clone(), input.clone())),
            Some(Binding::Unbound) => emitted.clear.push(action.clone()),
            // The base's own binding, unchanged: this action needs no element.
            Some(Binding::Base(base)) if base == action => {}
            Some(Binding::Base(other)) => {
                let reciprocal = composed
                    .get(other)
                    .is_some_and(|partner| partner == &Binding::Base(action.clone()));
                if !reciprocal {
                    return Err(composition_fault(action, other));
                }
                if exchanged.contains(other) {
                    // Already emitted from the pair's other half: an exchange is
                    // one element, not two.
                    continue;
                }
                exchanged.insert(action.clone());
                emitted.swap.push((action.clone(), other.clone()));
            }
            None => {}
        }
    }

    // The proof: re-apply what was emitted and demand the same symbols back.
    let mut state: BTreeMap<ActionId, Binding> = BTreeMap::new();
    for action in touched {
        state.insert(action.clone(), Binding::Base(action.clone()));
    }
    apply_lists(&mut state, &emitted.set, &emitted.clear, &emitted.swap);
    for action in touched {
        if state.get(action) != composed.get(action) {
            let other = match composed.get(action) {
                Some(Binding::Base(other)) => other.clone(),
                _ => action.clone(),
            };
            return Err(composition_fault(action, &other));
        }
    }
    Ok(emitted)
}

/// The merge-conflict refusal: named by the colliding action id, one bounded
/// line, and no suggestion of a resolution — design 014 refuses here precisely
/// so no implementer invents conflict markers or a winner (A20).
fn composition_fault(action: &ActionId, other: &ActionId) -> CoreError {
    let first = action.as_str();
    let second = other.as_str();
    CoreError::InvalidValue {
        key: "swap".to_string(),
        message: format!(
            "`{first}` and `{second}` compose to an effect no one patch can write in the fixed \
             `set` then `clear` then `swap` order: `merge` refuses rather than guessing a winner"
        ),
    }
}
