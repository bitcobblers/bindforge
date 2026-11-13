# Transcript: code-review:testing r1c0

## Provenance
- stage: code-review:testing (round r1c0)
- agent: codereview/code-testing
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T12:29:52.706Z
- ended: 2026-11-13T12:35:26.443Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff c26bf7f02371a1535a4d956d00294479b94d5532...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/012-input-vocabulary-and-reach-classes.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (testing seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/012-input-vocabulary-and-reach-classes/reviews/code/testing-r1c0.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"crates/core/tests/placement.rs:722 (a_move_naming_an_ungraded_or_uncoordinated_input_is_a_structured_refusal)","problem":"All four refusal cases are mouse:*/gamepad:* ids, refused by require_keyboard before the two keyboard-specific definedness branches can run; the coordinate() refusal (graded keyboard id absent from the grid) and the ungraded() refusal (keyboard id a loaded table does not grade, e.g. v1 tables) are reachable public behavior with no test exercising either, so the test name claims keyboard coverage it does not provide.","fix":"Add two cases: shipped v2 tables plus a scratch sparse geometry (basic-grid set) with move keyboard:i -> keyboard:q expecting the no-coordinate refusal, and load_reach_tables on reach-classes/v1/ fed to placement_is_legal expecting the ungraded refusal."},{"severity":"minor","location":"crates/core/src/document.rs:312 (serialize reach-classes arm)","problem":"The new v2 keys serializer arm and its `version == MAJOR_V1 && keys.is_empty()` guard are exercised by no test: the conformance walk skips re-serializing reach tables by contract and no unit test calls serialize on a v2 document, leaving the proposal's 'quoted already escapes \\ so the backslash glyph round-trips' claim unpinned.","fix":"Add a unit round-trip in tests/reach_v2.rs: parse a v2 table carrying \"keyboard:\\\\ off-reach\" and \"keyboard:, far\", serialize, re-parse, and assert Document equality."},{"severity":"minor","location":"schemas/reach-classes/v2/spec.md:152","problem":"The new spec states the corpus stand-in directory 'is not named for a major, so the directory/version rule above does not apply to it', but the conformance.rs this diff edited writes the fixture pair into <scratch>/<major>/ and loads from that segment; the language-neutral contract now contradicts the reference harness it defines (behaviorally compatible, but a third-party reader gets a false statement).","fix":"Reword the parenthetical: the harness stands the pair in a directory named for the fixture's own major, so the directory/version rule applies there; a directory not named for a major simply has no segment to compare."},{"severity":"minor","location":"crates/core/src/placement.rs:122 (PlacementCheck::worst_degradation)","problem":"Public method with no consumer in the crate, in no test, and not part of the proposal's PlacementCheck description — a knob without consumers under testing-checklist item 7.","fix":"Pin it (assert worst_degradation() equals the max of degradations in one predicate case) or drop it until ADR-015 consumes it."},{"severity":"nit","location":"crates/core/src/placement.rs:55 (KeyboardGeometry::inputs)","problem":"Public accessor with no consumer and no test; the geometry tests use coordinate/name and raw file text instead.","fix":"Use it in the 48-distinct-ids test or drop it."},{"severity":"nit","location":"crates/core/src/document.rs:532 (parse_keyboard_geometry version arm)","problem":"The keyboard-geometry spec states the declared version must name the directory's major and load_geometry calls check_dir_version, but every geometry test loads from a non-major 'grid' directory, leaving the segment-mismatch arm and the format's v1-only UnsupportedVersion arm untested.","fix":"Add one scratch case: a v1-declaring geometry document inside a v2/ directory (segment refusal) and a version = \"v2\" geometry document (UnsupportedVersion)."}]}
```
