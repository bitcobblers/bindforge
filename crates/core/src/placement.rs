//! The placement-legality predicate (ADR-012, Decision Four).
//!
//! A placement moves one movement cluster to another region of the keyboard:
//! every action of the cluster keeps its internal shape and lands on a key the
//! other hand reaches. Decision Four makes that a checkable property rather
//! than an opinion — legal when (a) the shape is preserved and (b) no action's
//! reach class is degraded beyond a declared bound — and this module is that
//! predicate over *loaded data*: a `reach-classes/v2` pair of tables and one
//! `keyboard-geometry/v1` grid, never key lists in Rust. That is what makes
//! "adding a keyboard geometry is new table files, not new logic" true: the
//! shape rule reads the grid, so another geometry needs no code.
//!
//! Which placements a shipped policy chooses is ADR-015's, and this stays
//! region-agnostic — it answers "is this assignment legal at this bound", not
//! "where should this cluster go" — so ADR-015's transforms can precondition on
//! it. The hands are inputs to the predicate rather than facts derived from
//! data: nothing in a table says which hand *chose* to type a key, because the
//! left hand can rest on IJKL too.
//!
//! A refusal is a structured [`CoreError`] naming the id at fault, and it
//! always means the predicate could not be evaluated at all — an id with no
//! coordinate or no grade. A placement that evaluates and fails is a verdict
//! inside [`PlacementCheck`], with the deltas and degradations that explain
//! it: ADR-015's precondition has to be able to say why a placement is illegal,
//! which a bool cannot do. As elsewhere, no refusal names a local path.

use std::collections::BTreeMap;
use std::path::Path;

use crate::document::{Document, KeyboardGeometryTable, check_dir_version, parse_file};
use crate::error::CoreError;
use crate::input::{InputId, InputNamespace};
use crate::reach::{Hand, ReachClass, ReachTable, ReachTables};

/// One keyboard geometry: the name the grid answers to, and the coordinate of
/// every key it places.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardGeometry {
    name: String,
    coordinates: BTreeMap<InputId, (u32, u32)>,
}

impl KeyboardGeometry {
    /// The geometry's name: the value of its `geometry` entry and the stem of
    /// the file it is stored in.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The ids this grid places. The graded ids of a v2 table agree with this
    /// set — both are the 48 members of the closed `keyboard:` set — which is
    /// what keeps the definedness rule from ever firing on shipped data.
    #[must_use]
    pub fn inputs(&self) -> Vec<InputId> {
        self.coordinates.keys().cloned().collect()
    }

    /// The `(column, row)` one input sits at, or `None` when this grid does
    /// not place it.
    #[must_use]
    pub fn coordinate(&self, input: &InputId) -> Option<(u32, u32)> {
        self.coordinates.get(input).copied()
    }
}

/// `MAX_SLUG_OCTETS = 128` is the shared length bound for every id that rides
/// the slug grammar (`[a-z][a-z0-9-]*`) — a manifest `id`, a manifest `codec` or
/// preset id, a `profile/v2` `id` or `derives` element, a `policy/v1` `id`, a
/// keyboard geometry name, and one action-id segment, which keeps its stricter
/// shape rules and borrows only this number. The bound rides beside the grammar
/// at every one of these ids: `load_geometry` checks a geometry name against it
/// beside the `is_slug_name` check, before the name is joined onto a directory,
/// so no rider is enforced more narrowly than `024-B1` enumerates it.
/// The bound supersedes the DNS-label limit RFC 1035 §2.3.1 publishes, which
/// these specs used to state for themselves: each of the four now
/// cross-references this bullet instead of carrying a number of its own. The
/// value is the sibling of `MAX_NATIVE_NAME_OCTETS = 128` — one legal path
/// component on every platform this product names — and the amendment is
/// relaxation only: an id valid under the 63-octet text stays valid.
/// `schemas/README.md` is this bound's one stated home; this is its one
/// exported home.
pub const MAX_SLUG_OCTETS: usize = 128;

/// `MAX_DISCOVERY_SLUG_OCTETS = 128` is the length bound of a *discovery lookup
/// key* — the `<source>/<external-id>` string a discovery plugin reports for one
/// installed game (`025-B1`, ADR-025). It is a cap on a lookup key, not a
/// grammar-validated id: the key is exempt from [`is_slug_name`] by design,
/// because its `/` and its source token are what make it unusable as a product
/// id, and it is never a path component. It is the one bound the discovery emit
/// boundary checks, and it is set equal to [`MAX_SLUG_OCTETS`] and
/// `MAX_NATIVE_NAME_OCTETS` so the exported family reads as one number.
/// `schemas/README.md` is this bound's one stated home; this is its one exported
/// home.
pub const MAX_DISCOVERY_SLUG_OCTETS: usize = 128;

/// The slug grammar: `[a-z][a-z0-9-]*`. Named for what it is now that three
/// entry kinds share it — a geometry name, a manifest `id`, and a manifest
/// `codec` or preset id (ADR-013) — and applied to a document's `geometry`
/// entry, and to the name a caller hands `load_geometry` *before* it is
/// joined onto a directory, which is what keeps a `..` or a separator from
/// ever asking the filesystem for a path. It carries no length bound of its
/// own: the shared [`MAX_SLUG_OCTETS`] rides beside it for every id that
/// constant's document enumerates — a geometry name among them — so the
/// bound applies wherever the grammar does.
#[must_use]
pub fn is_slug_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    let Some((first, rest)) = bytes.split_first() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    rest.iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

/// The file-name rule for a geometry, beside `reach`'s `hand_matches_file_name`
/// and stated the same way: the file is `<geometry>.toml` and its `geometry`
/// entry names the same value. A pure function, so the rule is checkable
/// before a file is opened.
#[must_use]
pub fn geometry_matches_file_name(geometry: &str, file_stem: &str) -> bool {
    is_slug_name(geometry) && geometry == file_stem
}

/// The declared bound of a placement check, in severity steps: how many ranks
/// of the shared taxonomy a placement may degrade one action by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GradeBound(pub u32);

/// What a placement check decided, with the evidence beside it: the one common
/// `(dx, dy)` when every move agrees, each move's own delta, and each move's
/// degradation in ranks. A caller explains a refusal with these, which is why
/// the predicate answers a check rather than a bool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacementCheck {
    /// Clause (a) and clause (b) together.
    pub legal: bool,
    /// Clause (a) alone: every move carries the same delta and no two moves
    /// land on one key.
    pub shape_ok: bool,
    /// The bound this check was run at.
    pub bound: GradeBound,
    /// The common delta when the moves agree, `None` when they do not.
    pub delta: Option<(i64, i64)>,
    /// Each move's delta, in move order.
    pub move_deltas: Vec<(i64, i64)>,
    /// Each move's degradation — target rank minus source rank — in move order.
    pub degradations: Vec<i64>,
}

impl PlacementCheck {
    /// The worst degradation any move carries, or `None` for an empty move set.
    #[must_use]
    pub fn worst_degradation(&self) -> Option<i64> {
        self.degradations.iter().max().copied()
    }
}

/// Load one keyboard geometry from the TOML file `<name>.toml` sitting directly
/// in one `schemas/keyboard-geometry/v<N>/` version directory.
///
/// `name` is matched against the `[a-z][a-z0-9-]*` grammar, and against the
/// shared [`MAX_SLUG_OCTETS`] bound, *before* it is joined onto `dir`, so a
/// name that arrived from somewhere else — a `..`, a separator, an uppercase
/// spelling — is refused without asking the filesystem anything and without
/// naming a path, and a grammar-valid name past the bound is refused the same
/// way because a keyboard geometry name rides the bound (`024-B1`). The loaded
/// document's `geometry` entry must agree with the file stem, as a reach
/// table's `hand` entry must agree with its own.
pub fn load_geometry(dir: &Path, name: &str) -> Result<KeyboardGeometry, CoreError> {
    if !is_slug_name(name) {
        return Err(CoreError::InvalidValue {
            key: "geometry".to_string(),
            message: format!("`{name}` is not a geometry name: expected `[a-z][a-z0-9-]*`"),
        });
    }
    if name.len() > MAX_SLUG_OCTETS {
        // The bound leads and the offender follows, as in every other
        // slug-bound refusal (`load_policy`): the echoed name is by definition
        // past the bound, so an echo-first message would truncate the number
        // away. A keyboard geometry name rides `MAX_SLUG_OCTETS` (`024-B1`), so
        // the bound is checked here beside the grammar, before the join.
        let message = format!("a geometry name is at most {MAX_SLUG_OCTETS} octets: `{name}`");
        return Err(CoreError::InvalidValue {
            key: "geometry".to_string(),
            message,
        });
    }
    let file_name = format!("{name}.toml");
    let document = parse_file(&dir.join(&file_name))?;
    let Document::KeyboardGeometry(table) = document else {
        return Err(CoreError::InvalidValue {
            key: "format".to_string(),
            message: format!("`{file_name}` must hold a keyboard-geometry document"),
        });
    };
    if !geometry_matches_file_name(&table.geometry, name) {
        let declared = table.geometry.as_str();
        return Err(CoreError::InvalidValue {
            key: "geometry".to_string(),
            message: format!("`{file_name}` declares geometry `{declared}`"),
        });
    }
    check_dir_version(dir, &table.version, &file_name)?;
    Ok(geometry_from(&table))
}

/// The predicate itself: one cluster's moves, the hand it leaves and the hand
/// it lands on, the declared bound, and the data both rules read.
///
/// A move pairs one cluster action's source key with its proposed target key.
/// Clause (a) wants one translation: the same signed `(dx, dy)` for every move
/// and no two targets colliding, so a rotation, reflection, or shear is illegal
/// whatever the grades say. Clause (b) ranks each move's class in the shared
/// taxonomy and allows a degradation up to the bound; an improving move has a
/// negative degradation and never blocks. An id that is not `keyboard:*`, that
/// the grid does not place, or that the relevant hand's table does not grade is
/// a refusal — the check could not run — rather than a verdict.
pub fn placement_is_legal(
    from: Hand,
    to: Hand,
    moves: &[(InputId, InputId)],
    bound: GradeBound,
    tables: &ReachTables,
    geometry: &KeyboardGeometry,
) -> Result<PlacementCheck, CoreError> {
    let from_table = table_for(tables, from)?;
    let to_table = table_for(tables, to)?;
    let mut move_deltas = Vec::with_capacity(moves.len());
    let mut degradations = Vec::with_capacity(moves.len());
    for (source, target) in moves {
        require_keyboard(source)?;
        require_keyboard(target)?;
        let delta = shape_delta(source, target, geometry)?;
        let degradation = degradation(source, target, from_table, to_table)?;
        move_deltas.push(delta);
        degradations.push(degradation);
    }
    let delta = common_delta(&move_deltas);
    let shape_ok = delta.is_some() && targets_are_distinct(moves);
    let allowed = i64::from(bound.0);
    let worst = degradations.iter().max().copied();
    let within_bound = worst.is_none_or(|value| value <= allowed);
    Ok(PlacementCheck {
        legal: shape_ok && within_bound,
        shape_ok,
        bound,
        delta,
        move_deltas,
        degradations,
    })
}

/// The table for one hand. A placement always needs both hands' tables — the
/// source's class comes from the hand that leaves, the target's from the hand
/// that lands — so a directory answering with one of them is a refusal.
fn table_for(tables: &ReachTables, hand: Hand) -> Result<&ReachTable, CoreError> {
    let found = tables.hand(hand);
    let Some(table) = found else {
        let name = hand.name();
        return Err(CoreError::InvalidValue {
            key: "hand".to_string(),
            message: format!("the reach tables hold no table for the {name} hand"),
        });
    };
    Ok(table)
}

/// Definedness, first clause: only a `keyboard:*` input has a coordinate and a
/// grade, so a mouse or gamepad id in a move is the refusal Decision Three
/// implies rather than a move that happens to fail.
fn require_keyboard(input: &InputId) -> Result<(), CoreError> {
    if input.namespace() == InputNamespace::Keyboard {
        return Ok(());
    }
    Err(CoreError::InvalidValue {
        key: "input".to_string(),
        message: format!("`{}` is not a keyboard input", input.as_str()),
    })
}

/// Clause (a), per move: the signed coordinate delta over the bounded grid
/// coordinates.
fn shape_delta(
    source: &InputId,
    target: &InputId,
    geometry: &KeyboardGeometry,
) -> Result<(i64, i64), CoreError> {
    let (source_x, source_y) = coordinate(source, geometry)?;
    let (target_x, target_y) = coordinate(target, geometry)?;
    let dx = i64::from(target_x) - i64::from(source_x);
    let dy = i64::from(target_y) - i64::from(source_y);
    Ok((dx, dy))
}

/// Definedness, second clause: every id has a coordinate in the grid.
fn coordinate(input: &InputId, geometry: &KeyboardGeometry) -> Result<(u32, u32), CoreError> {
    let found = geometry.coordinate(input);
    let Some(coordinate) = found else {
        let text = input.as_str();
        let name = geometry.name();
        return Err(CoreError::InvalidValue {
            key: "input".to_string(),
            message: format!("`{text}` has no coordinate in the `{name}` grid"),
        });
    };
    Ok(coordinate)
}

/// Clause (b), per move: the rank of the target's class for the hand that
/// lands on it, minus the rank of the source's class for the hand that leaves
/// it. The rank is the class's index in the table's own `classes` order, so the
/// severity scale stays table data and this stays a pure function over it.
fn degradation(
    source: &InputId,
    target: &InputId,
    from: &ReachTable,
    to: &ReachTable,
) -> Result<i64, CoreError> {
    let Some(source_class) = from.grade(source) else {
        return Err(ungraded(source, from.hand()));
    };
    let Some(target_class) = to.grade(target) else {
        return Err(ungraded(target, to.hand()));
    };
    let previous = rank_of(from, source_class)? as i64;
    let next = rank_of(to, target_class)? as i64;
    Ok(next - previous)
}

/// Definedness, third clause: the hand's own table grades the id. A graded set
/// that stops covering a key is a vocabulary addition, not a placement that
/// fails.
fn ungraded(input: &InputId, hand: Hand) -> CoreError {
    let text = input.as_str();
    let name = hand.name();
    CoreError::InvalidValue {
        key: "input".to_string(),
        message: format!("the {name} hand's table grades nothing for `{text}`"),
    }
}

/// The rank of a class within the table that grades it. A document that parsed
/// cannot produce a grade whose class is unlisted — the pair rule requires every
/// class to be one its `classes` entry lists — so this is the definedness
/// refusal that would be.
fn rank_of(table: &ReachTable, class: &ReachClass) -> Result<usize, CoreError> {
    let found = table.rank(class);
    let Some(rank) = found else {
        let id = class.id.as_str();
        return Err(CoreError::InvalidValue {
            key: "classes".to_string(),
            message: format!("the `{id}` class is not listed by the table grading it"),
        });
    };
    Ok(rank)
}

/// Clause (a)'s other half: one common `(dx, dy)`, or `None` when the moves do
/// not agree on one. A set that does not agree is a rotation, reflection,
/// shear, or plain mismatch — illegal at every bound.
fn common_delta(deltas: &[(i64, i64)]) -> Option<(i64, i64)> {
    let first = deltas.first().copied()?;
    if deltas.iter().all(|delta| *delta == first) {
        return Some(first);
    }
    None
}

/// Clause (a)'s edge: two moves landing on the same key is not an assignment,
/// even when both deltas agree.
fn targets_are_distinct(moves: &[(InputId, InputId)]) -> bool {
    let mut seen: Vec<&InputId> = Vec::with_capacity(moves.len());
    for (_, target) in moves {
        if seen.contains(&target) {
            return false;
        }
        seen.push(target);
    }
    true
}

/// The loaded grid, keyed by id: a repeated id would have overwritten the
/// first, and the grammar refuses a repeat before this point.
fn geometry_from(table: &KeyboardGeometryTable) -> KeyboardGeometry {
    let mut coordinates = BTreeMap::new();
    for key in &table.keys {
        coordinates.insert(key.input.clone(), (key.column, key.row));
    }
    KeyboardGeometry {
        name: table.geometry.clone(),
        coordinates,
    }
}
