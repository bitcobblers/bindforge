# correctness review r1c1 (dissent pass on the r1 revision)

verdict: approve

Scope: `git diff 5dc494e 9716bf4` of `02-proposal.md`, then the full current
document, against `docs/adrs/003-parallel-inference-lane.md` and the revision
log. Inputs per the caller: proposal, ADR-003, revision log, my r1 findings.

## r1 findings — closure check

- [closed] major — Pass-level fan-out escalation: the revision implements the
  proposed fix essentially verbatim. `runSeatWithRetry` now "sets a pass-level
  `escalationPending` flag" instead of `StopDriving`; the per-seat line is
  "await sem.acquire(); release + skip if escalationPending"; the diagram, the
  Risks row, and Test strategy #5 ("no unstarted seat runs") were all amended
  together. At N = 1 queued seats acquire, release, and return unprompted, so
  wall clock and the `maxReviews` budget behave as today; ADR-003 §Execution 3
  ("escalates after its siblings settle") still holds for in-flight siblings.
  No re-raised.
- [closed] minor — pass-commit stageKey: `finalizeReviewPass` drops the
  `stageKey` parameter; the text pins the value ("The commit's stage key is
  the pass's stage name... `docs: <nnn-slug> design-review stage`"); Test
  strategy #6 now asserts "the exact message". No re-raised.

## Checklist evidence

- URLs spot-checked: `https://opencode.ai/docs/server/` (live; lists no
  concurrency/slot guarantee — supports A2), MDN Event loop (live;
  single-threaded run-to-completion — supports A5), `rename(2)` man page
  (live; atomic replace of an existing `newpath`, no cross-writer ordering
  guarantee — supports A7).
- Revision log: all three r1 entries record accepted changes (escalation flag,
  escalation-path ownership sweep, macOS/Windows → `no`); no rejected
  findings to re-litigate.

## Findings

- [minor][new] Chosen design > Pass-level fan-out in `drive` — "**N = 1 is the
  same path** ... only commit shape and finalize timing change — one commit
  and one ownership sweep per pass, not per seat" — contradicts the escalation
  bullet revised in the same pass: "After the barrier, if the flag is set:
  drain the writer, run the pass `enforceOwnership` sweep, then
  `finalizeEscalate` once". Today's failed stage skips ownership enforcement
  — the sentence this revision deleted said exactly that ("a failed stage
  skips ownership enforcement today too"). So at N = 1 the escalation path now
  reverts out-of-policy sibling files that today land unfiltered in
  `finalizeEscalate`'s commit: that changes the escalation commit's *content*,
  which is neither commit shape nor finalize timing, so the "only ... change"
  enumeration and Rollout 1's "with only the pass-commit shape changed" are
  both now false. ADR-003 §Self-update promises N = 1 "keeps ADR-002 behavior
  for gates, artifacts, and control flow", carving out only commit shape and
  sweep timing; whether a new sweep on the escalation path falls inside that
  carve-out is ambiguous, and the design should state its reading rather than
  let the caveat undercount the diff. Fix: add the escalation sweep to the
  N = 1 caveat bullet and the Rollout 1 sentence as a third stated difference
  ("and the escalation path now runs one ownership sweep before
  `finalizeEscalate`, where today none ran"), or keep the sweep success-path
  only so escalation commit content stays exactly today's.
- [minor][new] Chosen design > Pass-level fan-out in `drive` — "The sweep
  reverts out-of-policy sibling files and escalation still lands; its commit
  carries only in-policy complete sibling reports." — the sweep enforces
  policy only, not completeness. Nothing states `escalationPending` is
  first-writer-wins-exclusive in effect: at N > 1 a second in-flight seat can
  also fail twice (the seat still completes attempt 2 before release, "Retries
  consume slots"), and its partial report is in-policy under the shared
  `docs/**` rule, so `finalizeEscalate`'s `commitStage` carries it — exactly
  what the r1 text this revision replaced admitted ("finalizeEscalate's own
  `commitStage` lands the siblings' partial reports in the escalation
  commit"). "complete" over-promises a filter the mechanism does not have.
  Fix: reword to "carries only in-policy sibling files, including any partial
  report from a retry-exhausted seat (as today)"; if partial reports must be
  excluded, specify the mechanism that detects them.
- [nit][new] Chosen design > Pass-level fan-out in `drive` — escalation bullet:
  the r1 version ended "then `finalizeEscalate` once ... and stop driving";
  the rewrite drops the "stop driving" clause and the flow diagram shows only
  the success branch (barrier → drain → `finalizeReviewPass` → verdicts →
  gate). A reader must infer that the run halts after escalation. Fix: restore
  "then stop driving" at the end of the escalation sequence (one clause).

## FYI (no severity)

- Configuration bullet: "optional, deliberately relaxing the ADR's required
  field" — the proposal openly deviates from ADR-003 §Configuration's
  `maxParallelReviews: number` declaration (making it `maxParallelReviews?:
  number`). Runtime default (1), opt-in, and CLI validation are all preserved,
  so this is a declared typing concession, not an internal contradiction —
  whether the ADR's exact field shape must be honored is a call for the
  compatibility/authority seat.
- Compatibility matrix macOS/Windows `unknown` → `no` is internally
  consistent with Test strategy #10 (Linux-only CI); it is a support-policy
  statement for another seat to bless.
