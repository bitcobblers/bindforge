# correctness review r1

verdict: request-changes

Scope: whole document (panel pass c0, round 1). Inputs: 02-proposal.md,
docs/adrs/003-parallel-inference-lane.md, Revision log (empty — initial r1).

Verification performed: every `verified` assumption row checked against its cited
code (panel.ts:162–178, gates.ts:81–168 and 260–273, runner.ts:96–135/272–277/
604–952/635/872–884/917–950/1126–1170/1183–1193/1304–1315, state-store.ts:157–173,
types.ts:19–23+48, stages.ts:107–111, package.json:30–32, src/cli.ts:101–137,
prompts.ts:50–51/89–90, docs/workflows.md:80–84/88–93) — all citations accurate.
Research URLs spot-checked: opencode.ai/docs/server (live; no concurrency
guarantee, supports A2), man7 rename(2) (live; atomic replace, supports A7),
MDN event loop (live — redirects to /Execution_model; run-to-completion,
supports A5). No dead or contradicting source.

## Findings

- [major][new] Pass-level fan-out in `drive`, bullet "Escalate after siblings
  settle" — "Accept this widened commit: siblings already in flight complete
  before escalation is visible (they would not have run at all under today's
  loop)." — the flow spawns every seat task up front (`await allSettled(seat
  tasks)`, per-seat `sem.acquire()` queued FIFO), so nothing prevents queued
  seats from acquiring and running after one seat exhausts its retry budget.
  At the **default N = 1** this means the whole panel still runs — serially,
  one seat at a time — after the first seat has already failed twice, before
  `finalizeEscalate` becomes visible. That contradicts the ADR this design
  implements (ADR-003 §Self-update: "Default N = 1 keeps ADR-002 behavior for
  gates, artifacts, and control flow") and the proposal's own text:
  Configuration ("DEFAULT_CONFIG.maxParallelReviews = 1 — today's behavior")
  and Rollout 1 ("every run keeps ADR-002 gates, artifacts, and control flow,
  with only the pass-commit shape changed"). It is not commit shape or timing:
  it changes which sessions the run buys and what artifacts it produces. Each
  executed attempt pushes a `state.results` entry, and `reviewSessionsUsed`
  (panel.ts:50–51) counts those, so post-failure siblings burn ADR-002's
  `maxReviews` budget (`runner.ts:1220–1241` pass-entry cap check) and change
  `pipeline status`/ESCALATION accounting; at N = 1 a worst-case escalation
  costs 5 seats × 2 attempts of serial wall clock instead of 2. Test strategy
  #5 ("escalation happens only after every sibling's task settles") codifies
  the widened behavior. Fix: add a pass-level `escalationPending` flag set by
  `runSeatWithRetry` on second failure in place of `StopDriving`; each seat
  task checks the flag immediately after `acquire()` and returns without
  running when it is set (release immediately). The barrier then waits only
  for in-flight tasks, so at N > 1 in-flight siblings still settle before
  `finalizeEscalate` (ADR §Execution 3 satisfied), while at N = 1 no unstarted
  seat runs — today's control flow exactly. Amend the "already in flight"
  sentence and test #5 accordingly.

- [minor][new] Two-phase split of `executeStage`, `finalizeReviewPass` —
  "finalizeReviewPass(opts, state, headBefore, stageName, stageKey, cfg)" and
  "commitStage keeps today's fallback message shape (`docs: <nnn-slug>
  <stageKey> stage`)" — a review pass has five seats, each with a distinct
  stageKey (`design-review:correctness`, …); the design passes one
  `stageKey` into a once-per-pass `commitStage` but never says which value it
  carries. Today's message shape is only defined for the per-seat key
  (runner.ts:277 uses the seat's `handle.stageKey`), so two implementers can
  emit `docs: 003-x design-review stage` or a leftover per-seat key — the same
  interface appears under two shapes. Fix: state the value explicitly, e.g.
  the pass commit's stageKey is the pass's stage name (`design-review` /
  `code-review` / `editorial-review`), making the message exactly
  `docs: <nnn-slug> design-review stage`; test #6 then asserts that exact
  string.

## FYI (no severity)

- `executeInference` is defined as "everything through `state.results.push`
  and the transcript write (lines 613–915) … No git mutation" — that span
  includes `proposalSize` (runner.ts:886–889), which shells out to `git diff
  --numstat`/`cat-file` (reads only, and unreachable for review stages since
  `DESIGN_DOC_STAGES` holds none). The claim is true but a one-clause
  "git reads allowed" note would make the two-phase contract unambiguous for
  the future-stage-kind risk the Risks table itself raises.
- A stray `.pipeline.json.*.tmp` left by a *crashed earlier run* (before
  `pipeline resume`) is still swept by the pass's `git add -A` — identical to
  today's exposure and arguably ADR-004's checkpoint-recovery territory; the
  drain invariant only covers in-flight writes, which is what ADR-003 asks.
  Not a finding.
- A5's cited MDN URL (`/Web/JavaScript/Event_loop`) now redirects to
  `/Reference/Execution_model`; content supports the claim, but updating the
  URL would keep the citation stable.
