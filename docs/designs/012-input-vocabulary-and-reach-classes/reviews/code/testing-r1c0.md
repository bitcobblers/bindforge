# Code review — testing seat, round 1 pass c0

Diff reviewed: `git diff c26bf7f02371a1535a4d956d00294479b94d5532...HEAD -- ':(exclude)docs/designs'`.
Inputs: that diff, `02-proposal.md`, `docs/adrs/012-input-vocabulary-and-reach-classes.md`. No prior findings (first pass).

## Verdict: approve

## Checklist evaluation

1. **Strategy citations.** Test strategy #1–#7 each have at least one citing test:
   #1 `tests/input.rs` (5 citing tests + module doc), #2 `tests/reach_v2.rs`,
   #3 `tests/reach_v2.rs`, #4/#5 `tests/placement.rs`, #6 `tests/conformance.rs`
   module doc + `load_corpus_table` + `v2_is_a_supported_major_beside_the_untouched_v1`,
   #7 `tests/grammar_closure.rs` + `tests/placement.rs` (data agreement).
   #8 is the CI/platform evidence leg, not a code test; its three gate commands were
   run locally here and are green (tail below); the Windows leg is explicitly the
   merge-PR/contributor leg (A13, Rollout 5). Evaluated, not a code finding.
2. **Layers.** Unit (`input.rs`, `reach_v2.rs`, `placement.rs`) and integration
   (`conformance.rs` walk, `grammar_closure.rs`) both exist as promised.
3. **Revision rounds.** No previously rejected findings in this seat — N/A.
4. **Determinism.** No wall-clock assertions, no network, no servers, no ports.
   `ScratchDir` (pre-existing) makes unique temp paths from pid + atomic counter +
   nanos; naming, not behavior, depends on time. Same-label scratch dirs in loops
   are sequential per test and disjoint across concurrent tests. Clean.
5. **Names.** Test names describe behaviors ("refuses a keys pair naming a class
   the document does not list"), not implementation steps. One name slightly
   overstates its coverage — finding 1.
6. **Gate run.** Ran the repo's three gate commands myself (the checklist's
   `npm run check` maps to this repo's `check.yml` rust job commands). All green;
   tail pasted below.
7. **Untestable additions.** Several — findings 1, 2, 4, 5, 6.

Also checked (not checklist items, and clean): strategy #2's ten v2 invalid
fixtures each carry one named fault and match their named unit-level case; the
`reach-classes/v1/fixtures/invalid/keys-entry.toml` pin exists and is asserted
both by `conformance.rs`'s walk and directly by
`the_v1_corpus_ships_the_fixture_that_refuses_a_keys_entry`; `reach_tables.rs`
is untouched in the diff and its 5 v1 expectations pass byte-unchanged (#7);
the worked-example deltas the proposal pins — `(6,0)`, `(7,0)`, `(1,0)`, the
4-rank home→off-reach gap, the 1-rank home→thumb step — were independently
recomputed from `ansi-us.toml` and the grade tables and match the asserts; the
zero-fixture and case-unique-name rules cover both new directories through the
unchanged walk; `version_gate.rs`'s majors-equals-dirs test now covers v2 with
no change (A2).

## Findings

### 1. minor — `crates/core/tests/placement.rs:722` (`a_move_naming_an_ungraded_or_uncoordinated_input_is_a_structured_refusal`)
All four refusal cases are `mouse:*`/`gamepad:*` ids, which are refused by
`require_keyboard` *before* the two keyboard-specific definedness branches can
run. The `coordinate()` refusal (a keyboard id the grid does not place) and the
`ungraded()` refusal (a keyboard id a loaded table does not grade — e.g. the v1
tables, which grade nothing) are reachable public behavior stated by the
predicate's doc and the geometry spec, but no test exercises either; the test
name claims the "ungraded or uncoordinated" coverage for keyboard ids it does
not provide.
**Fix:** add two cases: shipped v2 tables + a scratch sparse geometry (e.g. the
`basic-grid` set) with move `keyboard:i` → `keyboard:q` → expect the
no-coordinate refusal; and `load_reach_tables` on `reach-classes/v1/` fed to
`placement_is_legal` → expect the ungraded refusal.

### 2. minor — `crates/core/src/document.rs:312` (serialize reach arm)
The diff extends the `reach-classes` serialize arm (write `keys` back as pair
strings, guarded by `version == MAJOR_V1 && keys.is_empty()`), and the proposal
claims "`quoted` already escapes `\` so the backslash glyph round-trips" — but
the conformance walk deliberately skips re-serializing reach tables, and no unit
test calls `serialize` on a v2 document. Both arms of the new branch are
untested; the backslash round-trip through the serializer is an unpinned claim
(the corpus fixtures parse `\\`, nothing serializes it back).
**Fix:** one unit round-trip in `tests/reach_v2.rs`: parse a v2 table carrying
`keyboard:\\ off-reach` and `keyboard:, far`, `serialize` it, re-parse, assert
`Document` equality — this also pins that the v2 document's `keys` line comes
back.

### 3. minor — `schemas/reach-classes/v2/spec.md:152`
The new spec says "(A stand-in directory is not named for a major, so the
directory/version rule above does not apply to it…)". The `conformance.rs` this
same diff edited now does the opposite: it writes the fixture pair into
`<scratch>/<major>/` and loads from that segment ("the scratch directory the
pair sits in carries that same segment"). Both layouts behave identically with
the implementation, so no conformance fork opens, but the language-neutral
contract now contradicts the reference harness it defines.
**Fix:** reword the parenthetical: the harness stands the pair in a directory
named for the fixture's own major, so the directory/version rule does apply
there; a directory not named for a major simply has no segment to compare.

### 4. minor — `crates/core/src/placement.rs:122` (`PlacementCheck::worst_degradation`)
Public method with no consumer in the crate, in any test, and nowhere in the
proposal's `PlacementCheck` description (which pins deltas and per-move
degradations only). Checklist 7: a knob without consumers is a finding.
**Fix:** either pin it (assert `worst_degradation() == degradations.max()` in one
predicate case) or drop it until ADR-015 actually consumes it.

### 5. nit — `crates/core/src/placement.rs:55` (`KeyboardGeometry::inputs`)
Public accessor with no consumer and no test (`the_ansi_us_geometry…` and the
agreement test use `coordinate`/`name` and file text instead).
**Fix:** use it in the 48-distinct-ids test, or drop it.

### 6. nit — `crates/core/src/document.rs:532` (`parse_keyboard_geometry` version arm)
The keyboard-geometry spec states "A document's declared `version` must name the
major of the directory holding it", and `load_geometry` calls `check_dir_version`
— but every geometry test loads from a `grid/` (non-major) directory, so the
segment-mismatch arm, and the format's `v1`-only `UnsupportedVersion` arm, are
untested.
**Fix:** one scratch case — a geometry document declaring `v1` stood in a `v2/`
directory (expect the segment refusal) and a `version = "v2"` geometry document
parsed (expect `UnsupportedVersion`).

## FYI (no severity)

- `tests/zz_api_probe.rs` is a fourth test target beyond the proposal's delta
  inventory of three; it is not module-mirrored (ADR-004), though precedents
  exist (`gate_contract.rs`, `version_gate.rs`). Harmless compile-time surface
  pin; the empty-vec assert in it is a name-resolution pin, as its doc states.
- `grammar_closure.rs::the_invalid_corpus_ships_an_unknown_entry_fixture_for_every_format`
  now carves `reach-classes/v2` out of "every format" (documented inline, with a
  direct unit test covering the set); the name slightly lags the carve-out.

## Gate run (checklist item 6)

`cargo fmt --all -- --check` — no output (clean).
`cargo clippy --workspace --all-targets -- -D warnings` — clean (no warnings as errors).
`cargo test --workspace --locked` — tail:

```
     Running tests/reach_v2.rs (target/debug/deps/reach_v2-33bdec74c4bdc0c5)
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/version_gate.rs (target/debug/deps/version_gate-8261afbbfdd24ea5)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/zz_api_probe.rs (target/debug/deps/zz_api_probe-2a14cbdf7ccf7eec)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests bindforge_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

New targets' full lines: `input.rs` 6 passed; `placement.rs` 17 passed;
`reach_v2.rs` 17 passed; `conformance.rs` 8 passed; `grammar_closure.rs` 8
passed; `reach_tables.rs` 5 passed (untouched v1 expectations, #7);
`zz_api_probe.rs` 1 passed. 0 failures anywhere.
