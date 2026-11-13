# performance review r1

verdict: request-changes

Scope: the whole of `02-proposal.md`. Inputs read: the proposal (incl. its Revision
log), `docs/adrs/010-lightweight-plan-flow.md`. No review reports or transcripts opened.

Checklist coverage:
1. Cost/latency budget — present and mostly quantified: `stageTimeoutMs` is a per-stage
   latency bound, `maxReviews`/`maxPassesPerRound`/`maxRounds` are the session bounds,
   Test strategy #12 states a session ceiling. One arithmetic defect → finding 1.
2. Structure costs — no unbounded n: 2 flows, 4/12 stages, 2/5 seats, a plan capped at
   80 lines and 10 files, bullet count scanned once per plan. No finding.
3. Memory growth — `FLOWS` is a 2-entry record; `configFor` allocates one record per
   entry call, not retained per pass; no cache is introduced. Bounded. No finding.
4. Startup cost — one finding (finding 2, nit): the run-start path reads the source
   document twice.
5. Caching/invalidation — the caches here are the seat stamp and the budgets derived
   from `state.flow`. Invalidation is stated and coherent: `flowOf` is the single read
   seam, the checkpoint is authoritative for a running run, promotion is a new full-flow
   entry save, `configFor` is the single resolver used by run/resume/status, so the
   resume-drift failure the ADR names is closed at one seam. No finding.
6. Research baseline — quoted by the design and consistent: "the six completed runs
   spent 43–96 stage sessions each", "56% of the six runs' sessions live in the design
   phase", "run 004 spent 14 of its 53 sessions in exactly that loop" against a light
   ceiling of ≤ 8 sessions (Test strategy #12). The 12-minute `stageTimeoutMs` has no
   baseline datum, and A25 says so explicitly ("per-stage durations were never
   aggregated") with Rollout #6 set to measure it — that gap is explained, so it is not
   a finding.

## Findings

- [major][new] Budget resolution — "the reachable shape is exactly `c0` + two dissent
  passes = 3 two-seat passes … so **`maxReviews.code: 6` is the binding stop**;
  `maxPassesPerRound: 3` is the per-round backstop that can only bind at the same point"
  — the session cost of a pass is not fixed at 2, so the binding stop is not a single
  number. The funnel the design keeps running unmodified prompts *only* the dissenters
  on a revision pass and *only* the stale approvers on a ratification pass (State graph
  delta: "revision to the dissenters only, ratification for stale approvals"; Test
  strategy #13 pins exactly that: "the dissent pass prompts only that seat, and the
  stale approver runs one ratification pass"). In the single-dissenter shape each pass
  after `c0` costs 1 session, so round 1 is `c0`(2) + dissent(1) + 3rd pass(1) = 4
  sessions when `maxPassesPerRound: 3` stops it — the pass backstop binds 2 sessions
  *before* `maxReviews.code: 6`, not "at the same point". Conversely, because the cap is
  checked at pass entry (A16), a 2-seat pass launched at 5 spent sessions takes the
  phase to 7, so "escalates at 6 code review sessions" (Test strategy #14) is the
  widest-dissent case only. The choice of 6 over 12 is still sound — every reachable
  shape is bounded in the 4–7 session band, well under the 43-session baseline — so the
  knob needs no change; the cost model that justifies it and the escalation point the
  tests assert are stated as exact when they are a range. Fix: state the light code
  phase's cost as a range — `sessions ∈ [4, 7]`: the cap check at pass entry admits a
  pass while spent sessions < 6, so a 2-seat pass can close the phase at 7; a 1-seat
  dissent/ratify pass makes `maxPassesPerRound: 3` the first stop at 4 sessions — and
  replace "can only bind at the same point" with that condition. Add the single-dissenter
  path to Test strategy #14 as a second case (escalation named by `maxPassesPerRound`
  at 4 sessions) so the test does not pin only the widest shape.

- [minor][new] Compatibility matrix — "`--max-parallel 1` (default, serial lane) | yes |
  Two seats = 2 × per-seat wall clock" — the design bounds light *sessions* (Test
  strategy #12: "total sessions ≤ 2 + seats × 3") but never states the light *wall
  clock*, while the baseline it argues from is quoted in both currencies (ADR-010's
  table: 43 sessions / 2.2 h for run 003). The wall-clock win is the operator-facing
  number, and the serial default is the shape most light runs will run in, so the design
  currently claims a 5× session cut with no corresponding wall-clock expectation and
  A25 conceding there is no per-stage duration datum. Fix: add one derived
  wall-clock bound to Goals or Rollout #6 — at the serial default, 8 sessions × 12 min
  = 96 min worst case vs run 003's 2.2 h baseline, so light must land under ~1.6 h —
  as the number Rollout #6's first light runs are checked against. No new mechanism; it
  replaces the unmeasured acceptance of A25 with a target the recorded data settles.

- [nit][new] Selecting the flow — "`runPipeline` reads the source text right after
  `deriveIdentity`" — the run-start path now reads and frontmatter-scans the source
  document twice: once pre-lock for the `flow` value, once at the existing post-lock
  `sourceAdmissible` gate that the design keeps ("the accepted value is recorded on the
  `sourceAdmissible` gate line"). Fix: parse once at the pre-lock point and pass the
  parsed result (flow value plus the fields `sourceAdmissible` already needs) into the
  post-lock gate, so the gate line records the value from the same parse — the design
  already needs that threading to print the accepted value, so this removes the second
  read rather than adding a step.

## FYI (no severity)

- The pre-lock `flow` read and the post-lock `sourceAdmissible` gate read the same file
  at two points with the lock taken between them; an edit in that window means the value
  validated is not the value admitted. Correctness's call, not mine.
- `plan` joining `CODE_STAGES` in `escalationPhase` (Reporting) is a phase-attribution
  change that also affects full-flow escalation text if `plan` is ever reachable there —
  correctness/reporting.
- `maxReviews.design: 0` alongside `reviewSeats.design: []` is two representations of
  "no design phase"; a seat list of zero plus a cap of zero must be the same authority
  for every consumer listed in A2. Compatibility/correctness.
- The shared `NNN` counter row (A17, Risks) is an ADR-010-accepted consequence; not
  re-opened.
