# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ b0170c152f10
- started: 2026-11-10T03:01:12.462Z
- ended: 2026-11-10T03:03:59.493Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/003-parallel-inference-lane/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/003-parallel-inference-lane.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/003-parallel-inference-lane.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/003-parallel-inference-lane/reviews/design/correctness-r1c0.md.
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design > Pass-level fan-out in `drive`, bullet \"Escalate after siblings settle\" — \"Accept this widened commit: siblings already in flight complete before escalation is visible (they would not have run at all under today's loop).\"","problem":"All seat tasks are spawned up front (per-seat `sem.acquire()` plus `await allSettled(seat tasks)`), so nothing stops queued seats from running after one seat exhausts its retry budget. At the default N = 1 the entire panel still executes serially after the first seat has failed twice, before escalation is visible. This contradicts ADR-003 §Self-update (\"Default N = 1 keeps ADR-002 behavior for gates, artifacts, and control flow\") and the proposal's own claims (Configuration: \"DEFAULT_CONFIG.maxParallelReviews = 1 — today's behavior\"; Rollout 1: \"with only the pass-commit shape changed\"). Each executed attempt pushes a `state.results` entry and `reviewSessionsUsed` (panel.ts:50-51) counts those, so post-failure siblings burn ADR-002's `maxReviews` session budget (checked at runner.ts:1220-1241) and change status/ESCALATION accounting; at N = 1 a failed-seat escalation costs 5x2 attempts of serial wall clock instead of 2. Test strategy #5 codifies the widened behavior.","fix":"Replace `StopDriving` inside `runSeatWithRetry` with a pass-level `escalationPending` flag set on second failure; each seat task checks the flag immediately after `acquire()` and, if set, releases and returns without running. The barrier then waits only for in-flight tasks, so ADR §Execution 3 (escalate after siblings settle) still holds at N > 1 while at N = 1 no unstarted seat runs — exactly today's control flow. Amend the \"already in flight\" sentence and test #5 to say escalation waits for in-flight tasks and skips unstarted seats."},{"severity":"minor","location":"Chosen design > Two-phase split of `executeStage` — \"finalizeReviewPass(opts, state, headBefore, stageName, stageKey, cfg)\" / \"`commitStage` keeps today's fallback message shape (`docs: <nnn-slug> <stageKey> stage`)\"","problem":"A review pass has five seats, each with a distinct stageKey (`design-review:correctness`, ...), but the design passes a single `stageKey` into a once-per-pass `commitStage` without saying which value it carries. Today's fallback message shape (runner.ts:277) is only defined for the seat's `handle.stageKey`, so the same interface appears under two shapes: an implementer could emit `docs: <nnn-slug> design-review stage` or a stale per-seat key, and test #6's \"message shape\" assertion is underdetermined.","fix":"Pin the value: the pass commit's stageKey is the pass's stage name (`design-review` / `code-review` / `editorial-review`), so the message is exactly `docs: <nnn-slug> design-review stage`; have test #6 assert that exact string."}]}
```
