# Transcript: code-review:testing r1c0

## Provenance
- stage: code-review:testing (round r1c0)
- agent: codereview/code-testing
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-12T23:11:08.744Z
- ended: 2026-11-12T23:17:55.216Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 60faf8c24438ed55121757057b578182d2815083...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/010-lightweight-plan-flow.md, and your own previous findings quoted below. Do not open
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
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/testing-r1c0.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"tools/orchestration/tests/pipeline/ (checklist item 6)","problem":"The checklist requires running `npm run check` in the affected package and pasting its tail. This seat has no shell access (every `shell` call returns `Permission denied: shell`), so the suite-green claim for `tools/orchestration` (lint + format:check + test) is unevaluated in this pass, not verified. Per the specialty checklist, an item that cannot be evaluated is a finding, not a pass.","fix":"Record the tail of `npm run check` run in `tools/orchestration` as evidence in the run (check runner output, or `03-build-notes.md`), so the suite-green claim is a transcript rather than an assumption."},{"severity":"minor","location":"tools/orchestration/tests/pipeline/gates.test.ts:631-642","problem":"Test strategy #7 and the design's Risks row 3 require the reserved-name precedence to be asserted for every stage class in both flows. The both-flow loop lists 10 stages and omits the document stages `proposal-revise` and `proposal-final` entirely (no reserved-name case anywhere) and `editorial-review` from the both-flow array. A precedence regression letting a `proposal-revise` session write `docs/designs/.../.pipeline.json` would pass the suite.","fix":"Add `proposal-revise`, `proposal-final` and `editorial-review` to the `stages` array at gates.test.ts:631 so the reserved-name loop covers the whole STAGE_NAMES set for both flows."},{"severity":"minor","location":"tools/orchestration/tests/pipeline/states.test.ts:339 and tools/orchestration/tests/pipeline/light-budget.test.ts:198","problem":"Light `maxRounds: 2` binds only via states.ts:202 (the runner skips the round-cap escalation for light at runner.ts:2419-2420 and the round charge at 2447,2450,2603). The only light assertion of that stop is a unit case at codeRound 6, which proves the branch exists but not that the registry's 2 is the stopping number; in the integration layer no light run ever reaches a second code round (every light-budget shape stops inside round 1 on the pass backstop). So the ADR's `fix, max 2 rounds` bound is unproven at its boundary.","fix":"Add an integration case with a panel that dissents once per round so a `fix` runs each round, and assert the run escalates entering code round 3 on the byte-equal fix-round string; add a unit case at codeRound 3 with configFor('light', DEFAULT_CONFIG) plus the mirror case at round 2 proceeding."},{"severity":"minor","location":"tools/orchestration/tests/pipeline/prompts.test.ts:464","problem":"The light `plan` stage's agent config is untested: the test asserts stageDef('plan').agent === 'design/planner' and that prompts/plan.md exists, but no test reads .opencode/agents/design/planner.md, even though the design makes its content a contract (designer.md:4-22 verbatim, A12's shape, and doc-sync's `git diff*` allow excluded). A missing planner file, a widened `edit \"*\"` allow, readable `transcripts/**`, a permitted shell, or the doc-sync `git diff*` line arriving is caught only by a live light run failing at its first session.","fix":"In prompts.test.ts, read .opencode/agents/design/planner.md and .opencode/agents/design/designer.md, assert the planner's permissions block equals designer.md lines 4-22, and assert it contains no `git diff` or `shell` allow."},{"severity":"nit","location":"tools/orchestration/tests/pipeline/flows.test.ts:229","problem":"The plan cap's non-integer arm is untested: validatePlanMaxLines (flows.ts:174-182) rejects non-integers as well as values <= 0, and the proposal's contract is absent-or-positive-integer, but the tests cover absence and 0 only.","fix":"Add `expect(() => configFor('light', { ...cliConfig(), planMaxLines: 12.5 })).toThrow(/planMaxLines/)`, and a NaN case."},{"severity":"nit","location":"tools/orchestration/tests/pipeline/light-flow.test.ts:360","problem":"The #12 session bound is one-sided: `sessionsCreated.length <= 2 + seats*3` (8) on a converging run that spends exactly 4 sessions, so a regression to 5-8 sessions passes and the number Rollout 6 reads against run 003's 43 is unconstrained.","fix":"Assert the exact session count for the converging run (toHaveLength(4)) alongside the ceiling, and apply the ceiling in the every-pass-dissent fixture (light-budget.test.ts:198) where it can bind."},{"severity":"nit","location":"tools/orchestration/src/pipeline/flows.ts:27","problem":"`SeatName` is exported with no consumer in src/ or tests/ (grep over tools/orchestration finds only the declaration), so it is an untestable addition that can drift from the SEAT_NAMES list it mirrors.","fix":"Delete it, or use it: `export const SEAT_NAMES: readonly SeatName[] = [...]` so the validator's seat set is type-checked against the panels."}]}
```
