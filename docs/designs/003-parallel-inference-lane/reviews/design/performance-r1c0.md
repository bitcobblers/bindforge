# performance review r1

scope: whole document (pass c0, round 1) — verdict: request-changes

Checklist pass: (1) event-path per-event cost — poll/nudge loops and the drained
writer are bounded and cheap; no git or sync IO is left inside the concurrent
region (verified: the only `spawnSync` sites outside the git helpers are the
resume backfill before `drive` and `writeEditorialDiff` at editorial pass entry,
both outside the fan-out). (2) O(n²) — panels are 1–5 seats, waiter queue and
barrier are O(|panel|); no unbounded quadratic surface. (3) memory — semaphore
waiter queue, writer chain, and in-memory results are bounded by the ADR-002
session budget. (4) startup — `resolveMaxParallel` is a regex + range check; no
new boot IO. (5) caching/invalidation — no cache introduced; the A7
last-landed-rename race is closed by the enqueue-ordered drained writer, and
test #7 pins it. (6) baseline — see nit finding.

## Findings

- [major][new] Chosen design → Pass-level fan-out in `drive` → "Escalate after
  siblings settle" — "it records the failure in `state.stageFailures` and the
  `stageFailure` gate entry (as today) and returns" + "Accept this widened
  commit: siblings already in flight complete before escalation is visible (they
  would not run at all under today's loop)." — the stated mechanism (all seat
  tasks created up front, each `await sem.acquire()` then run, barrier `await
  allSettled(seat tasks)`, nothing checking the recorded failure) makes **not-yet-started**
  seats run too, not just in-flight siblings. Today the loop short-circuits: on
  a seat's second failure `runWithRetry` calls `finalizeEscalate` and throws
  `StopDriving` (runner.ts:1158–1166), so later seats of the pass never launch.
  As specified, a c0 seat failing first at the default N = 1 still runs the
  remaining four seats — worst-case escalation wall clock grows from
  2 × `stageTimeoutMs` (~60 min at the 30-min default) to |panel| × 2 ×
  `stageTimeoutMs` (~300 min), and four doomed inference sessions consume server
  slots on a run that is already escalating. This is a latency/cost regression
  on the **default** path, and the design's own justification sentence
  ("siblings already in flight") misdescribes what the specified mechanism
  does. Fix: in each seat task, after `sem.acquire()` and before session
  creation, check a pass-level "retry-exhausted failure recorded" flag; if set,
  release the permit and return without prompting. The barrier then waits only
  for genuinely in-flight siblings, matching the design's stated rationale and
  the ADR-003 clause "escalates after its siblings settle" — and it stays one
  code path (the check is a no-op on the success path and at N = |panel|).
- [nit][new] Context / Rollout 4 — "ADR-002 budgets 40 design + 20 code
  sessions per run, so that serial lane is the run's dominant wall clock" — the
  proposal names the cost but never quotes the research/ADR payoff baseline
  (ADR-003 Consequences, research Performance baselines: a full-panel pass
  "shrinks from 5× to ceil(|panel|/N)× the serial wall clock"), so the "first
  post-merge parallel run" that the design makes the live validation of the
  lane has no expected number to check against. Fix: add the ceil(|panel|/N)×
  expected-pass-time sentence (with the N ≤ slots precondition) to Goals or to
  Rollout step 4, citing the ADR.

## FYI (no severity)

- The same escalation passage has a state-machine side outside my specialty —
  running seats today's loop would skip adds `stageFailure` gate entries and
  results/verdict records for seats that never ran pre-change; worth a
  correctness-seat look (my finding is scoped to the time/cost of running them).
