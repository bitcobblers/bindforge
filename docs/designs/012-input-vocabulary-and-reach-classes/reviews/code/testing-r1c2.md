# Code review — testing seat — ratification pass c2 (round 1)

Scope: `git diff 2682fb1f740770fdb588db1a3ff69d66e9d1f3d4..HEAD -- ':(exclude)docs/designs'`
(the fix commit `eba75fc` and the test-update commit `9c1f601`), read against the full
current `02-proposal.md`, `docs/adrs/012-input-vocabulary-and-reach-classes.md`, and my
round-1 findings. Purpose: confirm the post-approval revisions did not break what the
testing seat approved.

## Gate runs (checklist item 6)

`cargo test -p bindforge-core`, tail:

```
     Running tests/reach_v2.rs (target/debug/deps/reach_v2-33bdec74c4bdc0c5)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/version_gate.rs (target/debug/deps/version_gate-8261afbbfdd24ea5)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/zz_api_probe.rs (target/debug/deps/zz_api_probe-7ccf7eec)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests bindforge_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Across the thirteen integration targets: 122 passed, 0 failed.
`cargo clippy -p bindforge-core --all-targets --locked`: `Finished` — no warnings.
`cargo fmt -p bindforge-core --check`: no output — clean.
(The r1 pins — `a_reach_table_declaring_a_version_that_disagrees_with_its_directory_is_refused`,
the reach `v2`-in-`v1` / `v1`-in-`v2` segment cases, the registry directory rule, the fixture
name and zero-fixture rules — all still pass.)

## What the revisions change

Per-format version gate (`format_majors`/`check_format_major` + crate-wide union first),
`major_dir_segment` consolidation of the two `is_major_dir` copies, removal of the
keyboard-geometry in-parser version guard (now covered by the per-format gate), the
serializer reach arm conditioned on `version == MAJOR_V1` alone, the ASCII-member
clarification (`keyboard:é`) with a corpus fixture and unit case, the rewrite of
`two_moves_onto_the_same_target_key_are_illegal` into a genuinely distinct-target case
plus a common-delta case, and the v2 spec's stand-in-directory rewording.

## Status of the r1 testing findings

| r1 finding | status after this diff |
|---|---|
| #1 `tests/placement.rs` — keyboard-specific definedness refusals (`coordinate()` miss on a graded keyboard id; ungraded keyboard id, e.g. a v1 table set) never exercised | **still open** — the refusal test is untouched by this diff (all four cases remain `mouse:*`/`gamepad:*`, refused before the keyboard branches run). Not broken by the revisions; carried, unchanged. |
| #2 `document.rs` reach-classes v2 serializer arm untested (backslash round-trip unpinned) | **still open, and re-scored** — the diff edits that exact arm (guard becomes `version == MAJOR_V1` alone); still no test calls `serialize` on a v2 document. See finding 1. |
| #3 `schemas/reach-classes/v2/spec.md:152` stand-in-directory language contradicting `conformance.rs` | **closed** — the spec now says the stand-in is named for the fixture's own major and the completing table comes from that same major, which is exactly what `load_corpus_table` does (`<scratch>/<major>/`, shipped pair from `schemas/reach-classes/<major>/`). A residual of the same false statement survives in a code comment; see finding 3. |
| #4 `worst_degradation()` knob with no consumer | out of scope for this pass — `src/placement.rs` is not in the diff; unchanged. |
| #5 `KeyboardGeometry::inputs()` accessor with no consumer | out of scope — unchanged. |
| #6 keyboard-geometry version arms untested | **closed on the parse half** — the in-parser guard was subsumed by the per-format gate and `rejects_a_document_declaring_a_major_that_only_another_format_ships` pins the `keyboard-geometry`/`v2` refusal. The loader segment-mismatch arm is shared code (`check_dir_version`) exercised through the reach `v2`-in-`v1`/`v1`-in-`v2` cases and the registry test; I do not re-score the geometry-specific scratch nit. |

Nothing approved in round 1 was broken: the whole gate is green, the r1 pins pass, and the
new corpus fixtures (`major-not-shipped.toml`, `non-ascii-member.toml`) are picked up by the
unchanged walk (fixture-name, zero-fixture, single-line-message cases pass).

## New findings from the revisions

1. **minor — `crates/core/tests/version_gate.rs:65` (`assert_version_rejection`), mirrored
   at `crates/core/tests/action.rs:158`.** The new per-format pins only assert that the
   refusal message *contains* each of the format's majors. A refusal that names the
   crate-wide union instead names those majors too, so the assertion passes under the very
   behavior the new contract language and the tests' own comments forbid (README: "refused
   naming that format's majors"; version_gate comment: naming the union "would advertise a
   `v2` no `profile` grammar exists for"). The fail-open itself (refuse, not parse) is
   pinned; the "naming that format's majors" half is not. *Fix:* destructure the variant —
   `CoreError::UnsupportedVersion { supported, .. }` — and assert `supported ==
   format_majors(format)`; a message-level `!contains` check cannot work because the
   refusal echoes the declared major.

2. **minor — `crates/core/src/document.rs:365` (serialize reach-classes arm).** This diff
   changes the writer's condition from `version == MAJOR_V1 && keys.is_empty()` to
   `version == MAJOR_V1` alone, and its comment now asserts "a v2 document always carries
   its grades"; still no test serializes a v2 reach document (the conformance walk skips
   reach re-serialization by contract), so the guard change and the proposal's
   "`quoted` already escapes `\` so the backslash glyph round-trips" claim remain unpinned.
   Carried from r1 #2 and re-scored because the changed line keeps it true. *Fix:* as in
   r1 — a `tests/reach_v2.rs` round-trip: parse a v2 table carrying
   `"keyboard:\\ off-reach"` and `"keyboard:, far"`, `serialize`, re-parse, assert
   `Document` equality.

3. **nit — `crates/core/src/document.rs:666-669` (`check_dir_version` doc comment).** The
   r1 #3 contradiction was fixed in `spec.md` but survives in this comment, which the diff
   edits around: it still says the directories "the conformance harness loads a corpus
   fixture in" are "not named for a major", while the harness writes the pair under
   `<scratch>/<major>/` and the spec now says the stand-in carries the fixture's own major.
   *Fix:* mirror the spec's rewording (a non-major directory is a caller's scratch choice
   with no segment to compare; the harness's stand-in is major-named).

## Verdict

approve — no blocker or major findings; the revisions close r1 #3 and half of #6, break
nothing previously green, and leave minors #1/#2/#4/#5 carried without regression.
