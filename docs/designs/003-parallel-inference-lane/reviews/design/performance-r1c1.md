# Performance review r1c1 (dissent pass on the r1 revision)

verdict: approve

Scope: diff 5dc494e..9716bf4 of 02-proposal.md, then the full current document.
Inputs: 02-proposal.md, docs/adrs/003-parallel-inference-lane.md, 01-research.md,
the Revision log, and my r1c0 findings.

## Re-check of my r1c0 findings

- **[closed] major — unstarted seats running after a retry-exhausted failure.**
  The revised "Escalate after siblings settle" bullet and the loop diagram
  implement exactly the r1c0 fix: `runSeatWithRetry` "sets a pass-level
  `escalationPending` flag", and "A seat that acquires after the flag is set
  releases it and returns unprompted" (flag checked after `sem.acquire()`,
  before session creation — per the diagram line "per seat: await
  sem.acquire(); release + skip if escalationPending"). Consequences I checked:
  at N = 1 the escalation wall clock is again ~2 × stageTimeoutMs, matching
  today's `StopDriving` short-circuit (runner.ts:1158–1166); doomed seats no
  longer consume server slots; the barrier waits only for genuinely in-flight
  siblings, which matches ADR-003 §Execution 3 "escalates after its siblings
  settle". The r1c0 justification sentence that misdescribed the mechanism is
  gone. Test strategy #5 now asserts "no unstarted seat runs", so the property
  is pinned. No residual latency/cost regression on the default path.

- **[open] nit — missing payoff baseline (checklist item 6).** Re-raised below.

## Checklist sweep of the changed text

1. Event-path cost: flag check + semaphore acquire/release are O(1) per seat;
   seats are bounded (|panel| ≤ 5). No keystroke/event path involved. Pass.
2. O(n²): nothing in the changed text scales beyond the bounded panel. Pass.
3. Memory growth: seat-task array and writer chain bounded by in-pass events.
   Pass.
4. Startup cost: no new file reads/parsing added; `escalationPending` is a
   per-pass in-memory flag. Pass.
5. Caching invalidation: N/A (no caches touched).
6. Research baseline: still not quoted — see the nit below.

The new escalation-path step (writer drain + one `enforceOwnership` sweep
before `finalizeEscalate`) adds one `spawnSync` git scan per escalating pass;
bounded and negligible against the session wall clock it protects. No finding.

## Findings

- [nit][re-raised] Context / Rollout step 4 — "ADR-002 budgets 40 design + 20
  code sessions per run, so that serial lane is the run's dominant wall clock"
  / "The first post-merge parallel run … is the live validation of the lane" —
  the revision did not add the payoff baseline: the proposal still contains no
  occurrence of the ADR-003 Consequences / research "Performance baselines"
  number, that a full-panel or ratification pass "shrinks from 5× to
  ceil(|panel|/N)× the serial wall clock", so the run that the design makes the
  live validation has no expected number to check against (performance
  checklist item 6). The Revision log does not record this finding as rejected,
  so it is simply unaddressed. Fix: add one sentence to Goals or Rollout step 4
  quoting ceil(|panel|/N)× the serial wall clock with the N ≤ slots
  precondition, citing ADR-003 §Consequences.

## FYI (no severity)

- Chosen design → "Escalate after siblings settle" — "at N = 1 control flow is
  exactly today's" is now slightly overstated: at N = 1 the escalation path
  additionally runs `writer.drain()` and the new pass `enforceOwnership` sweep
  before `finalizeEscalate`, steps today's loop does not have (today a failed
  stage skips ownership enforcement). Cost is negligible; the claim's accuracy
  is correctness/compatibility territory.
- Configuration and CLI — making `maxParallelReviews` optional is flagged in
  the text as "deliberately relaxing the ADR's required field"
  (ADR-003 §Configuration writes it as a required `number`); default 1 is
  preserved, so the runtime behavior matches — flag-shape call belongs to the
  compatibility seat.
