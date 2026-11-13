//! Shared fixtures for the `bindforge-transforms` suites.
//!
//! This is not a test target of its own: cargo compiles `tests/*.rs`, and each
//! suite pulls this module in with `mod common;` (the shape `crates/core/tests`
//! ships, ADR-004).
//!
//! # What the fixtures are
//!
//! Everything is the contract's own shape, never an implementation's
//! constructor: the loaded data a policy is applied over is the shipped contract
//! tree (the `reach-classes/v2` tables and the `ansi-us` geometry — the pair the
//! placement predicate is meant to run over) plus scratch documents in a temp
//! directory, read through `bindforge_core::parse`. The registry is a scratch
//! `v1/registry.toml`, because the match sets these cases pair over are the
//! loaded registry's ids, and a case that needs a different axis grouping passes
//! its own rows to `load_registry` (A3: the shipped registry is the only id
//! list, grown by PR — a unit suite does not pin itself to its future growth).
//!
//! Unit-level: loaded contract data and scratch directories, no network and no
//! live server (repo practice).
//!
//! # Why the module-level `allow(dead_code)` stays
//!
//! Each test target compiles this module on its own, and every target uses a
//! different subset of these helpers. A helper that stops being used anywhere is
//! caught by the suites' `proves:` comments, not by this attribute.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bindforge_core::{
    ActionRegistry, CoreError, Document, KeyboardGeometry, Manifest, Policy, Profile, ReachTables,
};
use bindforge_transforms::{Outcome, TransformReport};

/// The manifest pin every probe document carries — design 013's canonical
/// identity spelling `"<id>@<revision>"`, which `apply` pins the base against
/// (014's pin-equality rule, cross-referenced rather than restated).
pub const PIN: &str = "probe-game@1";

/// The base id every probe base carries, and the policy id every probe policy
/// carries. They differ by contract: `policy.id == base.id` is an entry refusal,
/// because a produced base cannot name its own input (Chosen design → The stack).
pub const BASE_ID: &str = "probe-base";
pub const POLICY_ID: &str = "probe-policy";

/// The registry the default probe match sets are computed over: `move.steer-*`
/// is the two-id family every relocate and polarity case pairs over,
/// `aim.camera` the one `continuous-lateral` id, and `combat.fire`/`ui.menu`
/// the two rows the mouse-button swap finds. No `move.jump-*` id exists here, so
/// that tag is the empty-match case.
pub const STEER_IDS: [&str; 2] = ["move.steer-left", "move.steer-right"];
pub const CAMERA_IDS: [&str; 1] = ["aim.camera"];
pub const NO_AXIS_IDS: [&str; 2] = ["combat.fire", "ui.menu"];

/// The default base layout. `move.steer-*`'s two source keys are `keyboard:a`
/// and `keyboard:s`, rook-adjacent on the shipped ANSI-US grid, and the ids are
/// deliberately *not* in byte order so the "produced `set` list preserves the
/// source base's element order" clause has teeth; the last two rows are the
/// `mouse:left`/`mouse:right` pair the mouse swap exchanges.
pub const DEFAULT_SET: [&str; 4] = [
    "move.steer-right keyboard:s",
    "move.steer-left keyboard:a",
    "combat.fire mouse:left",
    "ui.menu mouse:right",
];

/// The workspace root, from this crate's package root — the same derivation
/// `crates/core/tests` uses, so the suite finds the contract tree whatever the
/// test binary's cwd is.
pub fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .expect("crates/transforms sits directly under the workspace root")
        .to_path_buf()
}

/// The `schemas/` contract tree at the workspace root.
pub fn schemas_root() -> PathBuf {
    repo_root().join("schemas")
}

/// A scratch directory the readers can be pointed at, removed when it goes out
/// of scope — so every case lays out its own registry, base, and policy files
/// and cleans up after itself.
pub struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "bindforge-transforms-{label}-{}-{nonce}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("creating scratch dir {}: {err}", path.display()));
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes one file into this scratch tree and returns its path, creating the
    /// parent first.
    pub fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.path.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("creating {}: {err}", parent.display()));
        }
        fs::write(&path, text).unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
        path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// The shipped v2 tables and the shipped ANSI-US geometry — the pair
/// `placement_is_legal` is meant to run over, and the only geometry that ships
/// (A20: a v1 policy is effectively ANSI-US-tested).
pub fn shipped_inputs() -> (ReachTables, KeyboardGeometry) {
    let tables =
        bindforge_core::load_reach_tables(&schemas_root().join("reach-classes").join("v2"))
            .expect("the shipped v2 reach tables must load");
    let geometry = bindforge_core::load_geometry(
        &schemas_root().join("keyboard-geometry").join("v1"),
        "ansi-us",
    )
    .expect("the shipped ANSI-US geometry must load");
    (tables, geometry)
}

/// Writes a scratch `v1/registry.toml` and loads it, so the match sets are the
/// loaded registry's ids and the axis kinds come from the registry's own
/// grouping (A2, A3).
pub fn load_registry(
    scratch: &ScratchDir,
    discrete_lateral: &[&str],
    continuous_lateral: &[&str],
    none: &[&str],
) -> ActionRegistry {
    let text = format!(
        "format = \"action-ids\"\nversion = \"v1\"\ndiscrete-lateral = \
         {}\ncontinuous-lateral = {}\nnone = {}\n",
        quoted_array(discrete_lateral.iter().copied()),
        quoted_array(continuous_lateral.iter().copied()),
        quoted_array(none.iter().copied()),
    );
    scratch.write("registry/v1/registry.toml", &text);
    bindforge_core::load_action_registry(&scratch.path().join("registry/v1"))
        .expect("a registry document the grammar defines must load")
}

/// The default probe registry.
pub fn load_default_registry(scratch: &ScratchDir) -> ActionRegistry {
    load_registry(scratch, &STEER_IDS, &CAMERA_IDS, &NO_AXIS_IDS)
}

/// A one-line string array quoting these elements.
fn quoted_array(elements: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let items = elements
        .into_iter()
        .map(|element| format!("\"{}\"", element.as_ref()))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

/// A `profile/v2` `base` document: its layout is its `set` list (A7).
pub fn base_text(id: &str, set: &[&str]) -> String {
    format!(
        "format = \"profile\"\nversion = \"v2\"\nid = \"{id}\"\nkind = \"base\"\nfor = \
         \"{PIN}\"\nset = {}\n",
        quoted_array(set.iter().copied())
    )
}

/// The same base pinned to a different manifest revision — the shape a
/// regeneration under a bumped manifest meets (Test strategy #8, A7).
pub fn base_text_pinned(id: &str, pin: &str, set: &[&str]) -> String {
    format!(
        "format = \"profile\"\nversion = \"v2\"\nid = \"{id}\"\nkind = \"base\"\nfor = \
         \"{pin}\"\nset = {}\n",
        quoted_array(set.iter().copied())
    )
}

/// A `profile/v2` `patch` document — never a legal argument to `apply`: a
/// transform applies to bases, and a personal patch sits above the stack
/// untouched (A1).
pub fn patch_text(id: &str, set: &[&str]) -> String {
    format!(
        "format = \"profile\"\nversion = \"v2\"\nid = \"{id}\"\nderives = [\"{BASE_ID}\"]\nkind = \
         \"patch\"\nfor = \"{PIN}\"\nset = {}\n",
        quoted_array(set.iter().copied())
    )
}

/// A `manifests/v1` document over the probe action table, with these constraint
/// entries appended verbatim — the loaded game facts the untouchable guard
/// reads (A6).
pub fn manifest_text(constraints: &[&str]) -> String {
    let mut text = String::from(
        "format = \"manifests\"\nversion = \"v1\"\nid = \"probe-game\"\nrevision = \
         1\ncodec = \"unreal-engine-5\"\nactions = [\"move.steer-left Turn left\", \
         \"move.steer-right Turn right\", \"aim.camera Mouse look\", \"combat.fire Fire\", \
         \"ui.menu Menu\"]\n",
    );
    for line in constraints {
        text.push_str(line);
        text.push('\n');
    }
    text
}

/// A `policy/v1` document whose stack is exactly these rows.
pub fn policy_text(id: &str, rows: &[&str]) -> String {
    format!(
        "format = \"policy\"\nversion = \"v1\"\nid = \"{id}\"\ntransforms = \
         {}\n",
        quoted_array(rows.iter().copied())
    )
}

/// The `Profile` a profile document carries.
pub fn profile_of(text: &str) -> Profile {
    let document = bindforge_core::parse(text)
        .unwrap_or_else(|err| panic!("a document the entry table defines must parse: {err}"));
    match document {
        Document::Profile(profile) => profile,
        _ => panic!("a `format = \"profile\"` document must parse into the profile arm"),
    }
}

/// The `Manifest` a manifest document carries.
pub fn manifest_of(text: &str) -> Manifest {
    let document = bindforge_core::parse(text)
        .unwrap_or_else(|err| panic!("a document the entry table defines must parse: {err}"));
    match document {
        Document::Manifest(manifest) => manifest,
        _ => panic!("a `format = \"manifests\"` document must parse into the manifest arm"),
    }
}

/// The `Policy` a policy document carries.
pub fn policy_of(text: &str) -> Policy {
    let document = bindforge_core::parse(text)
        .unwrap_or_else(|err| panic!("a document the entry table defines must parse: {err}"));
    match document {
        Document::Policy(policy) => policy,
        _ => panic!("a `format = \"policy\"` document must parse into the policy arm"),
    }
}

/// A profile's layout as `(action id, input id)` spellings, in `set` array
/// order — the comparison every produced-base case reads, including the
/// "element order is preserved in place" clause.
pub fn layout(profile: &Profile) -> Vec<(String, String)> {
    profile
        .set
        .iter()
        .map(|(action, input)| (action.as_str().to_string(), input.as_str().to_string()))
        .collect()
}

/// The report a stack answered for row `index`, failing unless the row is
/// reported at all and names its kind as the contract spells it.
pub fn report_at<'a>(
    reports: &'a [TransformReport],
    index: usize,
    kind: &str,
) -> &'a TransformReport {
    let report = reports
        .iter()
        .find(|report| report.index == index)
        .unwrap_or_else(|| {
            panic!(
                "row {index} must answer with a report — \"reports and no-ops\" is only a promise \
                 if the report exists; the reports are {:?}",
                reports
                    .iter()
                    .map(|report| (report.index, report.kind))
                    .collect::<Vec<_>>()
            )
        });
    assert_eq!(
        report.kind, kind,
        "the report for row {index} must name the row's kind as the contract spells it"
    );
    report
}

/// Row `index` fits, and says so.
pub fn assert_row_applied(reports: &[TransformReport], index: usize, kind: &str) {
    let report = report_at(reports, index, kind);
    assert!(
        matches!(report.outcome, Outcome::Applied),
        "row {index} (`{kind}`) fits this base and must report `Applied`"
    );
}

/// Row `index` does not fit: it reports `NotApplicable` carrying this closed
/// reason spelling. Returns the verdict's offender so the caller can pin what
/// the report names.
pub fn assert_row_not_applicable(
    reports: &[TransformReport],
    index: usize,
    kind: &str,
    reason: &str,
) -> Option<String> {
    let report = report_at(reports, index, kind);
    match &report.outcome {
        Outcome::NotApplicable(verdict) => {
            assert_eq!(
                verdict.reason.name(),
                reason,
                "row {index} (`{kind}`) must report the closed reason `{reason}`"
            );
            verdict.offender.clone()
        }
        Outcome::Applied => panic!(
            "row {index} (`{kind}`) does not fit this base: it must report `NotApplicable` with \
             reason `{reason}` and no-op, not `Applied`"
        ),
    }
}

/// The verdict names its offender — one of `needles`, since which sentence names
/// it is not part of the contract, but an unnamed offender is the report the
/// design refuses to ship.
pub fn assert_offender_names(offender: Option<String>, needles: &[&str], what: &str) {
    let named = offender.unwrap_or_default();
    assert!(
        needles.iter().any(|needle| named.contains(needle)),
        "the `{what}` verdict must name its offender, one of {needles:?}, got `{named:?}`"
    );
}

/// An entry refusal: `apply` answers `CoreError`, because the whole application
/// could not be evaluated — the refusal channel, never a verdict (A9).
pub fn assert_entry_refusal(err: &CoreError, needles: &[&str], what: &str) {
    let detail = err.to_string();
    assert!(
        needles.iter().any(|needle| detail.contains(needle)),
        "the refusal for {what} must name what it refused, one of {needles:?}, got `{detail}`"
    );
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "a refusal is one bounded line: {detail:?}"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()),
        "a refusal escapes control characters rather than emitting them: {detail:?}"
    );
}
