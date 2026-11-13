# Code review — correctness, round 1 pass c0

Scope: `git diff c26bf7f02371a1535a4d956d00294479b94d5532...HEAD -- ':(exclude)docs/designs'`
Inputs: that diff, `02-proposal.md`, `docs/adrs/012-input-vocabulary-and-reach-classes.md`,
`03-build-notes.md` (the checklist's deviation surface). No prior findings to re-check.
Verdict: **request-changes** (1 major, 3 minor, 1 nit).

## Checklist pass

1. **Components and signatures.** All present and wired: `src/input.rs`
   (`InputNamespace`, `InputId::parse`/`namespace`/`as_str`, the three closed
   member sets — 26 + 10 + 11 + `space`, 7 mouse, 18 gamepad), `src/placement.rs`
   (`KeyboardGeometry`, `load_geometry`, `geometry_matches_file_name`,
   `GradeBound(u32)`, `PlacementCheck`, `placement_is_legal(from, to, moves,
   bound, tables, geometry) -> Result<PlacementCheck, CoreError>`),
   `reach.rs` (`InputGrade`, `ReachTable::grade`), `document.rs`
   (`REACH_CLASSES_V2_ENTRIES`, version-picked entry set, `keys` parsing,
   serializer arm, `check_dir_version`), `lib.rs` re-exports and public-surface
   list, `schemas/reach-classes/v2/` (spec + both tables + the 3 valid and 10
   invalid fixtures named), `schemas/reach-classes/v1/fixtures/invalid/keys-entry.toml`,
   `schemas/keyboard-geometry/v1/` (spec + `ansi-us.toml` + 2 valid/4 invalid
   fixtures), `schemas/README.md` (format list + one pointer paragraph). The
   shipped `left.toml`/`right.toml` are the proposal's tables verbatim: 48 ids
   each, each once, `;` home / `'` stretch right and off-reach left, `space`
   thumb both; `ansi-us.toml` places exactly the same 48 (`q` = `0 1`, `a` =
   `0 2`, `z` = `0 3`, `space` = `3 4`). v1 grammar, table data and existing
   fixtures, `error.rs`, `toml.rs`, `Cargo.*` and both workflows are untouched.
2. **Assumption register.** A1–A14 hold in the code. A1: v1's set stays closed
   and `keys-entry.toml` refuses it naming the entry. A3: `\\` round-trips
   through `quoted`/`parse` (fixture `punctuation-glyphs.toml`, `sparse-grid.toml`).
   A8/A10: the 48 graded ids are the closed member set. A11: no new `CoreError`
   variant; the new refusals are `InvalidValue`/`MissingEntry`/`UnsupportedVersion`,
   one bounded line, no path.
3. **Deviations.** #2 (conditional `check_dir_version`), #3 (`Document::KeyboardGeometry`
   + serializer arm), #4 (geometry refuses a non-`v1` major), #5 (`is_geometry_name`
   in `placement.rs`), #6 (definedness reads the two consulted tables), #7 (field
   names), #8 (linear scans) are all declared and match the code. Two build-note
   claims are now false against this tree — finding 4.
4. **Scope creep.** See finding 5 (one extra test target, one extra shared test
   helper, neither in the delta inventory).
5. **Build-notes truth.** Finding 4.
6. **Error paths fail closed.** The refusal paths added by this diff are correct
   and pinned; one accept path this diff opens fails open — finding 1.
7. **Previous round.** No prior code-pass findings; nothing to re-verify.

Gate run locally against this tree: `cargo test --workspace --locked` green,
119 tests across 14 targets (input 6, placement 17, reach_v2 17, zz_api_probe 1,
conformance 8, grammar_closure 8, reach_tables 5, version_gate 4, and the
pre-existing targets unchanged).

## Findings

### 1. major — the version gate is still crate-wide, so `profile/v2` and
`action-ids/v2` documents now parse under the v1 grammar

`crates/core/src/document.rs:186-213`. `supported_majors()` returns `[v1, v2]`
and `parse` checks only that one global list, then dispatches on `format`.
`parse_profile` (`document.rs:379`) and `registry_from_entries` (`action.rs:179`)
never look at the version, so `format = "profile", version = "v2"` (or
`action-ids`) is now **accepted** and read with v1 semantics — `serialize` writes
`version = "v2"` back. Before this diff the gate refused any `v2` declaration, so
this is a newly-opened accept path, not pre-existing behavior.

The same diff shows the fix it wants: the geometry arm grows its own
`version != MAJOR_V1` refusal (`document.rs:531-537`) for exactly this reason,
and build note #4 records the hole for `profile`/`action-ids` as an observation.
`version_gate.rs:9-12` states the invariant being broken — "a document written
against a future major parsed with v1 semantics … is exactly what versioning
exists to prevent" — and `schemas/README.md:23-30` says a document declaring a
major the reader does not support is rejected, never read with another major's
grammar. There is no `profile/v2` directory, no grammar, and no fixture, so a
v2-declaring profile is a document core cannot support; it is untrusted input at
every entry point.

Fix: make the gate per-format — a `supported_majors_for(format) -> &'static
[&'static str]` (or a `(format, major)` set) checked in each arm, `reach-classes`
being the only format with `[v1, v2]` — which also subsumes the geometry-specific
refusal at 531-537; keep `supported_majors()` as the union so
`core_declares_exactly_the_major_versions_present_under_schemas` still passes.
Add a `version_gate.rs` case pinning a `profile` document declaring `v2` refused
as `UnsupportedVersion`, and state the per-format rule in `schemas/README.md` so
third-party readers implement the same gate.

### 2. minor — the reach serializer can emit a v1 document carrying `keys`,
the case its own comment says must not happen

`crates/core/src/document.rs:310-316`. The guard is
`if table.version == MAJOR_V1 && table.keys.is_empty()`, so a
`ReachClassTable` with `version = "v1"` and a non-empty `keys` — constructible by
any consumer, since `ReachClassTable`'s fields are public — falls into the
`else` and prints `keys = [...]` under `version = "v1"`, a document v1's closed
entry set refuses to re-read (that is precisely what
`reach-classes/v1/fixtures/invalid/keys-entry.toml` pins). The comment above the
branch ("Only a document whose grammar carries `keys` writes it back: a v1
document that printed one would not re-parse") describes the intended rule; the
condition does not implement it.

Fix: branch on the version alone — `if table.version == MAJOR_V1 { head } else {
head + keys }` — so the writer's output always re-parses, as the proposal states
(only a v2 document writes `keys`).

### 3. minor — `two_moves_onto_the_same_target_key_are_illegal` never exercises
the targets-distinct half of clause (a)

`crates/core/tests/placement.rs:693-716`. The move set is
`a→s, d→s`, whose deltas are `(1,0)` and `(-1,0)`; `common_delta` returns `None`
and `shape_ok` is false on the common-delta rule, so the `targets_are_distinct`
check in `placement.rs:335-345` never decides this case, and the doc comment's
claim that the moves share one delta is untrue of them. The clause the design
pins ("`coordinate(target) − coordinate(source)` is the same `(dx, dy)` for every
move, **and targets are distinct**") is then unpinned: deleting the distinctness
check keeps this suite green.

Fix: pin it with a set that really has one delta and repeated targets — the only
such shape is a repeated source id, e.g.
`[(keyboard:a, keyboard:s), (keyboard:a, keyboard:s)]` — and assert
`check.delta == Some((1, 0))` beside `!check.shape_ok`, which separates the two
halves of clause (a).

### 4. minor — `03-build-notes.md` no longer describes this tree (checklist
item 5)

The notes' *Gate posture* block lists 75 tests across 10 targets and deviation #1
states "No test files were written or edited", that `conformance.rs` still
hard-codes `v1`, that `grammar_closure.rs` still carries three entry lists, and
that the worked examples are "pinned by nothing yet". Against this diff all four
statements are false: `tests/input.rs` (6), `tests/reach_v2.rs` (17),
`tests/placement.rs` (17) and `tests/zz_api_probe.rs` (1) exist,
`conformance.rs:121-127` takes the fixture's own major and compares grades,
`grammar_closure.rs` carries the `reach-classes/v2` and `keyboard-geometry/v1`
sets, and Test strategy #3/#5/#7 are pinned — the suite runs 119 tests green.
Deviation #2's justification ("the untouched `conformance.rs` harness … loads
every corpus table out of a `bindforge-core-reach-classes-corpus-load-*` stand-in
directory") is likewise stale: the harness now loads from `<scratch>/v1` or
`<scratch>/v2`, so the segment rule does bite there.

Fix: rewrite the Gate posture with the current per-target `test result:` lines
and rewrite deviation #1 into the deviations that actually remain (if any), so
the merge PR's evidence leg and the deviation record match the code.

### 5. nit — two additions outside the proposal's delta inventory, undeclared

`crates/core/tests/zz_api_probe.rs` (107 lines, new target) and
`crates/core/tests/common/mod.rs`'s `quoted_array` helper are not in the delta
inventory's Added/Edited lists, and the build notes' deviation #1 names only the
three proposed targets. `zz_api_probe.rs` is also the one new Rust target not
module-mirrored the way the proposal says its three are. Its content is honest
(it only resolves the promised public names), so this is bookkeeping: name both
paths in the notes (or fold the probe's assertions into `tests/reach_v2.rs` and
`tests/placement.rs`).

## FYI (out of specialty, no severity)

- `check_dir_version`'s "directory named for a major" carve-out (`document.rs`)
  is declared (build note #2) and now written into `reach-classes/v2/spec.md:152-154`;
  the harness feeds it a `v1`/`v2` segment, so a hard rule like
  `load_action_registry`'s may be affordable — the performance/compatibility
  seats' call.
- `ReachTable::grade`/`rank` and `targets_are_distinct` are linear scans, as
  build note #8 says; at 48 entries that is fine, and it is the performance
  seat's line to hold.
