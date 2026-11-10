# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ b0170c152f10
- started: 2026-11-10T03:11:20.041Z
- ended: 2026-11-10T03:13:04.625Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/003-parallel-inference-lane/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 5dc494e756f0a15c78eba7eaccfa35d40cdd7456 9716bf4c84f55bfe464605c41edaf3514210fd00` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/003-parallel-inference-lane.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design > Pass-level fan-out in `drive`, bullet \"Escalate after siblings settle\" — \"Accept this widened commit: siblings already in flight complete before escalation is visible (they would not have run at all under today's loop).\"",
    "problem": "All seat tasks are spawned up front (per-seat `sem.acquire()` plus `await allSettled(seat tasks)`), so nothing stops queued seats from running after one seat exhausts its retry budget. At the default N = 1 the entire panel still executes serially after the first seat has failed twice, before escalation is visible. This contradicts ADR-003 §Self-update (\"Default N = 1 keeps ADR-002 behavior for gates, artifacts, and control flow\") and the proposal's own claims (Configuration: \"DEFAULT_CONFIG.maxParallelReviews = 1 — today's behavior\"; Rollout 1: \"with only the pass-commit shape changed\"). Each executed attempt pushes a `state.results` entry and `reviewSessionsUsed` (panel.ts:50-51) counts those, so post-failure siblings burn ADR-002's `maxReviews` session budget (checked at runner.ts:1220-1241) and change status/ESCALATION accounting; at N = 1 a failed-seat escalation costs 5x2 attempts of serial wall clock instead of 2. Test strategy #5 codifies the widened behavior.",
    "fix": "Replace `StopDriving` inside `runSeatWithRetry` with a pass-level `escalationPending` flag set on second failure; each seat task checks the flag immediately after `acquire()` and, if set, releases and returns without running. The barrier then waits only for in-flight tasks, so ADR §Execution 3 (escalate after siblings settle) still holds at N > 1 while at N = 1 no unstarted seat runs — exactly today's control flow. Amend the \"already in flight\" sentence and test #5 to say escalation waits for in-flight tasks and skips unstarted seats."
  },
  {
    "severity": "minor",
    "location": "Chosen design > Two-phase split of `executeStage` — \"finalizeReviewPass(opts, state, headBefore, stageName, stageKey, cfg)\" / \"`commitStage` keeps today's fallback message shape (`docs: <nnn-slug> <stageKey> stage`)\"",
    "problem": "A review pass has five seats, each with a distinct stageKey (`design-review:correctness`, ...), but the design passes a single `stageKey` into a once-per-pass `commitStage` without saying which value it carries. Today's fallback message shape (runner.ts:277) is only defined for the seat's `handle.stageKey`, so the same interface appears under two shapes: an implementer could emit `docs: <nnn-slug> design-review stage` or a stale per-seat key, and test #6's \"message shape\" assertion is underdetermined.",
    "fix": "Pin the value: the pass commit's stageKey is the pass's stage name (`design-review` / `code-review` / `editorial-review`), so the message is exactly `docs: <nnn-slug> design-review stage`; have test #6 assert that exact string."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/003-parallel-inference-lane.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/003-parallel-inference-lane/reviews/design/correctness-r1c1.md.
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design > Pass-level fan-out in `drive` — \"**N = 1 is the same path** ... only commit shape and finalize timing change — one commit and one ownership sweep per pass, not per seat\"","problem":"Contradicts the escalation bullet revised in the same pass: \"After the barrier, if the flag is set: drain the writer, run the pass `enforceOwnership` sweep, then `finalizeEscalate` once\". Today's failed stage skips ownership enforcement — the sentence this revision deleted said exactly that (\"a failed stage skips ownership enforcement today too\"). At N = 1 the escalation path now reverts out-of-policy sibling files that today land unfiltered in `finalizeEscalate`'s commit, i.e. the escalation commit's content changes, which is neither commit shape nor finalize timing; the \"only ... change\" enumeration and Rollout 1's \"with only the pass-commit shape changed\" are both now false, and ADR-003 §Self-update (\"Default N = 1 keeps ADR-002 behavior for gates, artifacts, and control flow\", carving out only commit shape and sweep timing) is ambiguous about a new sweep on the escalation path. The design should state its reading instead of undercounting the N = 1 diff. Label: new.","fix":"Add the escalation sweep to the N = 1 caveat bullet and the Rollout 1 sentence as a third stated difference (\"and the escalation path now runs one ownership sweep before `finalizeEscalate`, where today none ran\"), or keep the sweep success-path only so escalation commit content stays exactly today's."},{"severity":"minor","location":"Chosen design > Pass-level fan-out in `drive` — \"The sweep reverts out-of-policy sibling files and escalation still lands; its commit carries only in-policy complete sibling reports.\"","problem":"The sweep enforces policy only, not completeness. Nothing prevents a second in-flight seat from also exhausting its retries at N > 1 (it still completes attempt 2 before releasing its permit, per \"Retries consume slots\"), and its partial report is in-policy under the shared `docs/**` ownership rule, so `finalizeEscalate`'s `commitStage` carries it — exactly what the replaced r1 sentence admitted (\"finalizeEscalate's own `commitStage` lands the siblings' partial reports in the escalation commit\"). \"complete\" asserts a filter the mechanism does not have. Label: new.","fix":"Reword to \"carries only in-policy sibling files, including any partial report from a retry-exhausted seat (as today)\"; if partial reports must be excluded, name the mechanism that detects incompleteness."},{"severity":"nit","location":"Chosen design > Pass-level fan-out in `drive` — escalation bullet ending \"...then `finalizeEscalate` once with today's `failed twice … one retry, then escalate` message\"","problem":"The r1 version ended \"then `finalizeEscalate` once ... and stop driving\"; the rewrite drops the statement that the run halts after escalation, and the flow diagram shows only the success branch (barrier → drain → `finalizeReviewPass` → verdicts → gate). Halting is now only inferable. Label: new.","fix":"Restore \"then stop driving\" as the final clause of the escalation sequence (or draw the escalation branch in the diagram)."}]}
```
