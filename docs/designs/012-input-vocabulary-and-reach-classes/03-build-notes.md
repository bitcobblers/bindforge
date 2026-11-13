# Build notes — design 012 (input vocabulary, reach-class taxonomy, placement legality)

Implemented from `02-proposal.md` (r2), revised against the round-1 code reviews
(`reviews/code/correctness-r1c0.md`, `reviews/code/compatibility-r1c0.md`) in
the fix commit, extended by the test-update commit `9c1f601` that landed the
three test-side pins, and ratified by the dissent and ratification passes
(`reviews/code/*-r1c1.md`, `reviews/code/*-r1c2.md`). These notes describe the
tree at that point: complete as a source + contract-tree change, all five seats
approve, and the three cargo gate legs were run locally against it and pass —
see "Gate posture" for the re-run and "Carried review items" for what the seats
left open. The proposal's three new test targets (`tests/input.rs`,
`tests/placement.rs`, `tests/reach_v2.rs`) and the two proposed test-target edits
(`tests/conformance.rs`, `tests/grammar_closure.rs`) are in the tree, authored by
the test engineer; so is the 14th target and the extra test-side helpers, which
are outside the proposal's inventory — deviation 1.

`02-proposal.md` carries a dated As-built note (2026-11-13) recording the same
drift against the approved design — the per-format version gate, the
conditional load-time version rule, the geometry document variant, the edited
"Unchanged" files, the widened test inventory, the ASCII closure of the id
grammar, and the names the design left open — with the reason and the design
sections each touches. That note records what was built; it does not change the
design.

## What landed

Contract tree:

- `schemas/reach-classes/v2/` — `spec.md` (the canonical "Input ids" section,
  the entry table with `keys`, the taxonomy with its severity order, the pair
  rules), `left.toml` and `right.toml` grading all 48 members of the closed
  `keyboard:` set for each hand, and the fixtures the proposal names: valid
  `full-left-hand`, `minimal-hand`, `punctuation-glyphs`; invalid
  `missing-keys`, `unknown-class-reference`, `duplicate-input`,
  `pair-three-fields`, `non-string-element`, `mouse-input`, `word-not-glyph`,
  `uppercase-member`, `class-order-mismatch`, `unknown-class-id` — plus
  `non-ascii-member` (deviation 9).
- `schemas/reach-classes/v1/fixtures/invalid/keys-entry.toml` — pins that the
  v1 entry set refuses `keys`. v1's grammar, table data, and existing fixtures
  are byte-untouched.
- `schemas/keyboard-geometry/v1/` — `spec.md`, `ansi-us.toml` (the 48 graded
  ids on the ANSI rows: `q` = `0 1`, `a` = `0 2`, `z` = `0 3`, `space` = `3 4`),
  valid `basic-grid`, `sparse-grid`; invalid `unknown-entry`,
  `bad-coordinate`, `pair-two-fields`, `duplicate-input`.
- `schemas/profile/v1/fixtures/invalid/major-not-shipped.toml` — new: pins the
  per-format half of the version gate in the corpus (deviation 9).
- `schemas/README.md` — the format list gains `keyboard-geometry`, the scaffold
  paragraph names the two ADR-012 additions, one paragraph points at the
  input-id grammar's canonical home, and the versioning-rule bullet now states
  that the majors a document may declare are those of **its own format**.
- `schemas/reach-classes/v2/spec.md` — the id grammar says lowercase **ASCII**
  and names `non-ascii-member.toml`; the version-gate paragraph says the gate
  is per-format; the fixtures paragraph describes the stand-in directory the
  harness actually builds (named for the fixture's own major, completed from
  that same major directory) instead of the old "not named for a major"
  wording.
- `schemas/profile/v1/spec.md` — the version-gate paragraph states the
  per-format rule and names the new fixture (deviation 5).

`crates/core`:

- new `src/input.rs` — `InputNamespace`, `InputId::parse`/`namespace`/`as_str`,
  the three closed member sets (`MOUSE_MEMBERS`, `GAMEPAD_MEMBERS`, the 26 +
  10 + 11 glyph predicates plus the reserved word `space`), one refusal rule
  per set; the keyboard rule and the module doc say the set is lowercase ASCII,
  so a non-ASCII member is a refusal, never a fold.
- new `src/placement.rs` — `KeyboardGeometry` (`name`, `inputs`, `coordinate`),
  `load_geometry` (grammar-checks `name` before joining it, then `parse_file`
  under the same budget), `geometry_matches_file_name`, `GradeBound`,
  `PlacementCheck` (`legal`, `shape_ok`, `bound`, the common `delta`, the
  per-move `move_deltas` and `degradations`, plus `worst_degradation()`), and
  `placement_is_legal` as definedness → shape (clause a) → bound (clause b).
- `src/reach.rs` — `InputGrade`, `ReachTable::grade`, `ReachTable::rank`, and
  `load_reach_tables` filing grades and applying the version-segment rule;
  `Hand` and `hand_matches_file_name` untouched.
- `src/document.rs` — `MAJOR_V2`, `supported_majors() = [v1, v2]` (the union
  over formats, kept for the majors-equality pin), `format_majors` +
  `check_format_major` — the version gate per format —,
  `REACH_CLASSES_V1_ENTRIES`/`REACH_CLASSES_V2_ENTRIES` chosen by declared
  version, `GEOMETRY_ENTRIES`, `REACH_CLASSES_V2_TAXONOMY`, the version-naming
  `reject_unknown_entry` message, `ReachClassTable.keys`,
  `KeyboardGeometryTable`/`GeometryKey`, the pair and triple encodings
  (`parse_input_grades`, `parse_coordinate`, `coordinate_value`), the
  `serialize` arms that write `keys` back (the reach arm branches on the
  version alone), `check_dir_version`, and `major_dir_segment` — the one
  major-directory predicate the loaders share.
- `src/action.rs` — `load_action_registry` reads the directory's major through
  `document::major_dir_segment`; its private `is_major_dir` is gone
  (deviation 5).
- `src/lib.rs` — re-exports and the public-surface list (unchanged by the
  round-1 fixes: `major_dir_segment` is `pub(crate)`).
- Untouched: `toml.rs`, `error.rs` (no new `CoreError` variant), `chain.rs`,
  `profile/v1`'s grammar table and table data, `action-ids/v1`,
  `reach-classes/v1` grammar and table data, `Cargo.toml`, `Cargo.lock`, both
  workflows, every pre-existing fixture. (`profile/v1/spec.md`'s version-gate
  paragraph is the one edit inside a format the inventory lists as unchanged —
  deviation 5.)

## Round-1 review fixes

| Finding | Fix |
|---|---|
| correctness 1 / compatibility 1 (major) — the crate-wide gate let `profile/v2` and `action-ids/v2` parse under the v1 grammar | `document::format_majors` + `check_format_major`: each arm's format is gated on its own majors (`reach-classes` is the only `[v1, v2]`), refusing with `UnsupportedVersion { supported: format_majors }`. The crate-wide check still binds first, so a major no format ships names the whole union and `supported_majors()` stays the union the majors-equality pin reads. The `keyboard-geometry` arm's own `version != v1` guard is deleted — the per-format gate subsumes it and refuses the same way, `UnsupportedVersion` naming `v1`. Pinned in the contract by `profile/v1/fixtures/invalid/major-not-shipped.toml`, `schemas/README.md`, and the `profile/v1` and `reach-classes/v2` spec paragraphs |
| correctness 2 (minor) — the reach serializer could print `keys = [...]` under `version = "v1"` for a hand-built model | the reach arm branches on the version alone: a v1 document never writes `keys`, so `serialize` output always re-parses |
| compatibility 2 (minor) — nothing in the corpus said the id grammar is ASCII | `schemas/reach-classes/v2/fixtures/invalid/non-ascii-member.toml`; "lowercase ASCII" in the spec's id grammar and the `keyboard:` bullet; the Rust member rule and module doc say ASCII |
| compatibility 3 (minor) — the v2 spec still described a stand-in directory that is not named for a major | the fixtures paragraph describes the harness's real layout: the stand-in carries the fixture's own major and the completing table comes from that same major directory, so the directory/version rule holds there |
| compatibility 4 (nit) — two `is_major_dir` predicates with two strictnesses | one `pub(crate) document::major_dir_segment(dir) -> Option<&str>`, used by `check_dir_version` (a `None` segment passes through) and by `load_action_registry` (a `None` segment is its existing refusal) |
| correctness 4 (minor) — these notes no longer described the tree | Gate posture and the deviation list rewritten against this tree, with the gate legs re-run |
| correctness 5 / compatibility 5 (nit) — `tests/zz_api_probe.rs` and `common::quoted_array` were undeclared | deviation 1 names both, and deviation 2 corrects the stale stand-in-directory justification |

Three findings were the test engineer's and were left out of the fix commit;
the **test-update stage that followed it** (`9c1f601`) landed all three, so
they are in the tree now and this is the record of where each sits:

| Finding | Where it landed |
|---|---|
| correctness 1 / compatibility 1 — a `profile` (and `action-ids`, `keyboard-geometry`) document declaring `v2` | `tests/version_gate.rs::rejects_a_document_declaring_a_major_that_only_another_format_ships` — all three formats, each asserted naming that format's own majors, with a guard that the tree ships no `v2` for them — plus `the_invalid_corpus_ships_a_fixture_for_a_major_the_format_does_not_ship` pinning `major-not-shipped.toml` by path |
| compatibility 2 — nothing pinned the id grammar's ASCII half | `tests/input.rs::rejects_an_id_that_breaks_the_namespace_or_member_grammar` carries `keyboard:é`, and `tests/reach_v2.rs::the_v2_corpus_ships_the_fixture_that_refuses_a_non_ascii_member` pins `non-ascii-member.toml` by path and asserts the refusal echoes the member |
| correctness 3 — the repeated-target case never asked about distinctness | `tests/placement.rs::two_moves_onto_the_same_target_key_are_illegal` now runs `[a→s, a→s]` (both deltas `(1, 0)`, `delta == Some((1, 0))`) so the verdict can only come from `targets_are_distinct`, and keeps `a→s, d→s` as the pin for the common-delta half |

The helpers those cases needed are outside the proposal's delta inventory and
are declared in deviation 1: `tests/common/mod.rs`'s `format_majors` beside
`quoted_array`, and the edits to `tests/action.rs` and `tests/version_gate.rs`
that make both suites name *their format's* majors rather than the crate-wide
union.

## Gate posture (recorded, not asserted)

The `repo-practices` skill asks a code stage to run the gate legs itself and
quote the output. All three were run against this tree — first by the round-1
fix stage, then again here, after the test-update commit `9c1f601`, against the
tree at `HEAD` — and the numbers below are that second run:

- `cargo fmt --all -- --check` — exited 0 with no output (working tree clean).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` —
  `Finished \`dev\` profile [unoptimized + debuginfo] target(s)` with no
  warnings or errors.
- `cargo test --workspace --locked` — every suite green:

  ```
  test result: ok. 0 passed; 0 failed   (unittests src/lib.rs)
  test result: ok. 16 passed; 0 failed  (tests/action.rs)
  test result: ok. 14 passed; 0 failed  (tests/action_ids.rs)
  test result: ok. 4 passed; 0 failed   (tests/chain_guard.rs)
  test result: ok. 8 passed; 0 failed   (tests/conformance.rs)
  test result: ok. 8 passed; 0 failed   (tests/gate_contract.rs)
  test result: ok. 8 passed; 0 failed   (tests/grammar_closure.rs)
  test result: ok. 6 passed; 0 failed   (tests/input.rs)
  test result: ok. 11 passed; 0 failed  (tests/parse_budget.rs)
  test result: ok. 17 passed; 0 failed  (tests/placement.rs)
  test result: ok. 5 passed; 0 failed   (tests/reach_tables.rs)
  test result: ok. 18 passed; 0 failed  (tests/reach_v2.rs)
  test result: ok. 6 passed; 0 failed   (tests/version_gate.rs)
  test result: ok. 1 passed; 0 failed   (tests/zz_api_probe.rs)
  test result: ok. 0 passed; 0 failed   (doc-tests)
  ```

  122 tests across the 14 targets, 0 failures, 0 ignored — two more
  `version_gate.rs` cases and one more `reach_v2.rs` case than the run the
  fix-stage notes quoted (119, with 4 and 17). The correctness and
  compatibility seats re-ran the gate independently in their dissent passes and
  quote the same 122. The `version_gate.rs` pin
  `core_declares_exactly_the_major_versions_present_under_schemas` passes with
  `supported_majors() = [v1, v2]`, because the majors it reads are exactly the
  directories on disk. The two fixtures the round-1 fixes added ride the
  `conformance.rs` walk, which refuses both.

## Carried review items

The round-1 seats approved with these still open; the behavior they touch is in
the tree, the pin is not, and none of them is a design question:

- `tests/reach_v2.rs` never calls `serialize` on a v2 document, so the proposal's
  "`quoted` already escapes `\`" round-trip claim and the version-only guard in
  the reach serializer arm are unpinned (testing seat, r1 #2, re-scored at c2);
  the harness skips reach re-serialization by contract, so the pin has to be a
  unit round-trip.
- The predicate's definedness refusals are pinned through `mouse:*`/`gamepad:*`
  ids only; a graded-keyboard id missing a coordinate, or a v1 table set with no
  grades at all, never reaches the `coordinate()`/`grade()` branches
  (testing seat, r1 #1).
- `PlacementCheck::worst_degradation()` and `KeyboardGeometry::inputs()` have no
  consumer yet (testing seat, r1 #4, #5) — both are ADR-015's handles.
- `document.rs::check_dir_version`'s doc comment still calls the conformance
  harness's stand-in directories "not named for a major"; since `9c1f601` the
  harness writes `<scratch>/<major>/`, which is what
  `schemas/reach-classes/v2/spec.md` now says. Comment-only drift, flagged by
  the testing and performance seats.
- `schemas/README.md`'s per-format sentence reads as "only the format's majors",
  while a major *no* format ships is still refused naming the crate-wide union;
  `document.rs`'s module doc states the layering honestly (compatibility seat,
  c1 nit).
- Cost nits accepted as written: the allocating `supported_majors()` membership
  test now duplicated by `format_majors`, the linear grade/rank scans, the
  `iter().any` duplicate-id rules, and `tests/common/mod.rs::format_majors`
  re-walking the contract tree per call (performance seat).

## Deviations from the design

1. **Test-side paths sit outside the proposal's delta inventory**, all owned by
   the test engineer: `crates/core/tests/zz_api_probe.rs` (a 14th target, 1 test,
   that resolves every ADR-012 name at the crate root; it is not module-mirrored
   the way the inventory's three targets are); `crates/core/tests/common/mod.rs`'s
   two new helpers — `quoted_array` (a textual reader of the `keys` arrays, needed
   because the closed member set carries the `,` glyph) and `format_majors` (the
   per-format majors read off the contract tree, shared by
   `tests/version_gate.rs` and `tests/action.rs`); and edits to
   `crates/core/tests/action.rs` and `crates/core/tests/version_gate.rs`, beyond
   the two edited targets the inventory names (`conformance.rs`,
   `grammar_closure.rs`) — both suites now assert against *their format's* majors
   instead of the crate-wide union, which is the test half of deviation 4. The
   inventory's Added and Edited lists name none of these; naming them here is the
   bookkeeping the round-1 nits asked for, widened once for the helper the
   test-update stage added after them.
2. **The version-segment check at load is conditional.** The design asks that a
   reach table's declared `version` equal its loading directory's version
   segment. `document::check_dir_version` enforces that whenever the directory
   is named for a major (`v<N>`), and passes when it is not. The
   `conformance.rs` harness now builds its stand-in directory *with* the
   fixture's own major segment, so the rule bites on that path; the lenient
   half still matters for a caller pointed at a directory that is not a major
   directory at all — `reach_tables.rs` loads its scratch tables straight out
   of `scratch.path()`, whose name is a `bindforge-core-*` label. A hard rule
   like `load_action_registry`'s would refuse that suite, whose paths this stage
   does not own. Stated in the v2 spec's Table data and Fixtures sections.
3. **The document model gained a fourth variant and a serializer arm** —
   `Document::KeyboardGeometry`, `KeyboardGeometryTable`, `GeometryKey`, and
   the `serialize` arm that writes the `keys` triples back. The proposal names
   only the reach-classes serializer arm; the geometry arm is forced by the
   harness it leaves unchanged, which re-serializes every valid fixture whose
   format is not `reach-classes`.
4. **The version gate is per-format, and the contract says so.** The proposal
   states only the crate-wide rule and the A11-shaped refusals; the two round-1
   majors showed the crate-wide rule alone opens an accept path
   (`format = "profile"` + `version = "v2"` read with the `profile/v1`
   grammar). `format_majors` is where a format's majors live,
   `check_format_major` applies it, and the geometry-specific guard the earlier
   round added is gone. `supported_majors()` stays the union, and the crate-wide
   check still runs first, so a major no format ships is refused naming every
   declared major (the spelling `version_gate.rs` pins today).
5. **`action.rs` and `profile/v1` are edited**, both under "Unchanged" in the
   delta inventory. `load_action_registry` now shares
   `document::major_dir_segment` instead of carrying its own copy of the
   major-directory predicate (compatibility nit 4 — one definition, each loader
   deciding what a non-major directory means); the behavior of its two
   refusals is unchanged. `schemas/profile/v1/spec.md`'s version-gate
   paragraph states the per-format rule and names the new invalid fixture.
6. **`is_geometry_name` lives in `placement.rs`**, with `document.rs` importing
   it, so the geometry-name grammar has exactly one definition and the import
   lists stay short. The proposal does not name a home for it; the grammar
   itself is the `keyboard-geometry/v1` spec's.
7. **Definedness in the predicate reads "graded by both hand tables" as "graded
   by the two tables the check consults"**: the source id is looked up in the
   from-hand table and the target id in the to-hand table. A same-hand shift
   (WASD→ESDF) consults one table twice, and requiring the *other* hand's grade
   for the same id would add a refusal that no shipped data can produce — the
   two tables grade the same 48 ids, which `reach_v2.rs` pins.
8. **Field and helper names the design leaves open**: `GradeBound(pub u32)` as
   a tuple struct; `PlacementCheck`'s `legal`, `shape_ok`, `bound`, `delta`,
   `move_deltas`, `degradations`, and `worst_degradation()`; `ReachTable`'s
   private `grades` and its `rank(&ReachClass)`; refusal wording throughout.
   `lib.rs` also re-exports `GeometryKey` and `KeyboardGeometryTable` from
   `document`, beside the `InputId`/`InputNamespace`/`InputGrade`/placement names
   the design lists — the geometry table type is public because `Document` gained
   the variant (deviation 3). The semantics the design fixes — one common
   `(dx, dy)` when the moves agree, per-move deltas, per-move degradation,
   structured refusals that name no path — are all present.
9. **Two invalid fixtures and three spec paragraphs the proposal's lists do not
   name**: `reach-classes/v2/fixtures/invalid/non-ascii-member.toml` (the
   corpus pin the compatibility seat asked for, following the precedent of
   `action-ids/v1/fixtures/invalid/non-ascii-id.toml`) and
   `profile/v1/fixtures/invalid/major-not-shipped.toml` (the corpus pin for
   deviation 4). The proposal's fixture inventories name neither, and the
   v2-spec wording changes (ASCII, per-format gate, stand-in directory) and the
   README bullet go beyond its "format list plus one pointer line".
10. `ReachTable::grade` and `rank` are linear scans, not indexes: the graded set
    is 48 entries and the contract gives no hot-path requirement. A caller
    loading the shipped tree can call the predicate per cluster without
    quadratic cost concerns; if 015 ever wants an index, it is a private change.
