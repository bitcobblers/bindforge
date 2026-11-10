# Code review — testing seat, round 1 pass c0

Inputs (per panel rules, independent read): the diff
`73200b4…HEAD -- ':(exclude)docs/designs'` (source + tests as they stand),
`02-proposal.md`, `docs/adrs/003-parallel-inference-lane.md`. No prior
findings to re-check (first pass for this seat). Specialty: `testing`
checklist only.

## Checklist walk

1. **Every numbered Test-strategy item has a citing test** — pass, for
   #1–#9. Citations found:
   - #1 `pipeline-concurrency.test.ts:16,38,58` (cap, FIFO order, oldest-waiter wake)
   - #2 `pipeline-parallel-fanout.test.ts:216` (N = 1/2/3/5; peak == min(N,|panel|), never over N)
   - #3 `pipeline-parallel-fanout.test.ts:232` (dissenters-only c1, stale-approvers-only c2 — both halves, exact create-order list)
   - #4 `pipeline-parallel-fanout.test.ts:260` (create order == handed order at N = 2, with a parallelism proof `maxInFlight == 2`)
   - #5 `pipeline-parallel-escalation.test.ts:211,251` (retry inside own permit — a release-and-re-acquire would launch the queued seat, which the test forbids; no unstarted seat runs; one sweep + one `finalizeEscalate`; `assertSweepRanBeforeEscalate` proves the sweep precedes escalation and the stray never reaches the terminal commit)
   - #6 `pipeline-pass-finalize.test.ts:201,228` (one commit + one sweep at N = 1 and N = 3, exact message `docs: 002-x design-review stage`; editorial pass at N = 3 with `-rN` naming and `rNcM` kept for design passes)
   - #7 `pipeline-checkpoint-writer.test.ts:49,67,86,102` (enqueue order, slower-first cannot land last, no `.tmp` after drain, writer goes through the validating `saveState`)
   - #8 `pipeline-cli-max-parallel.test.ts:30,37,44,58,69,77` (precedence, default 1, flag range, env format gate runs even under a valid flag, separate range error naming the source)
   - #9 `pipeline-parallel-integration.test.ts:201,231,250` (reports + transcripts exist, one pass commit carrying all five reports, no tmp after commit, `loadState` loads, once-per-seat results/ledger, one pass-gate entry, N = 1 vs N = 2 identical results incl. order and commit subjects)
   - #10 — not citable by a test (it *is* the `npm run check` run); see item 6 below, which I could not execute.

2. **Unit + integration layers where promised** — pass. Unit: concurrency,
   writer, resolver, fan-out/escalation/finalize over stub clients and temp
   git repos (the temp-repo layer the proposal itself prescribes for #6/#9).
   Integration: `pipeline-parallel-integration.test.ts` end-to-end
   `resumePipeline` at N = 2 and N = 1.

3. **Revision-round regressions covered** — the proposal's r1 closures are
   pinned in code: `escalationPending` skip-unstarted and the pre-escalate
   ownership sweep both have tests (`pipeline-parallel-escalation.test.ts`,
   stray probe + clean tree + stray absent from the escalation commit).
   N = 1 "same path, different commit shape" is asserted at the commit level
   (`pipeline-pass-finalize.test.ts:205`).

4. **Determinism** — mostly pass: mock client everywhere (no network, no
   port), temp dirs, no wall-clock assertions. The stub delays are ordering
   devices; under Node's timer-order guarantees the fan-out peaks and the
   integration result-order identity are safe (consecutive prompt windows
   are 20 ms apart and every step between is microtasks/sub-ms fs). One
   test does depend on a wall-clock *budget* — finding 2.

5. **Behavior-named tests** — pass. Names state the guarantee ("holds the
   retry inside its permit and escalates once at N = 2", "lands three
   concurrent enqueues in enqueue order"), not implementation steps.

6. **`npm run check` run by this seat** — could NOT evaluate. Shell access
   is permission-denied for this review session (two attempts, both
   rejected), so I could not run the suite or paste its tail. Per the
   checklist this counts as a finding, not a pass — finding 1. The panel's
   independent runner check governs; if it is red, that overrides this
   seat's verdict.

7. **Untestable additions** — no config knob without a consumer:
   `maxParallelReviews` is consumed by `runReviewPass` and behaviorally
   exercised at N = 1/2/3/5. Two untested defensive branches noted as nits
   (findings 4, 5).

## Findings

1. **[minor] Suite green not verified by this seat** —
   `tools/orchestration` (whole diff). The seat's shell is denied, so
   checklist item 6 could not be executed; my read is static only. Fix:
   the panel runner's independent `npm run check` in `tools/orchestration`
   stands in for this; a seat with shell access should paste the tail.

2. **[minor] `tests/pipeline-parallel-escalation.test.ts:257-266` relies on
   a timing budget, not a signal** — the N = 2 fixture only proves its
   claim if the failing seat's two attempts (≈ 90–120 ms of 5 ms timers +
   microtasks) all land inside the sibling's 600 ms prompt window; under
   CI event-loop starvation (parallel vitest workers, low-CPU container)
   the sibling could settle first, the queued security seat would get the
   freed permit, and the exact `creates` assertion fails as a false alarm.
   Fix: make the interleaving explicit instead of raced — have the stub
   resolve the sibling's prompt only after it observes correctness's
   second attempt (prompt count ≥ 2 for that key), or bump the margin and
   say why.

3. **[minor] `tests/pipeline-cli-max-parallel.test.ts` covers the resolver
   only, not the argv → yargs → cfg wiring** — the proposal says the knob
   mirrors `--timeout`, and the `--timeout` mirror in `tests/cli.test.ts`
   includes the end-to-end chain (`argv -> yargs -> resolver -> …`, line
   170) and the "resolver runs before `createClient()` and any side
   effect" ordering test (line 213). No equivalent exists for
   `--max-parallel` / `$OPENCODE_MAX_PARALLEL`: the yargs number coercion
   (junk input → NaN) and the validation-before-client construction in
   `src/pipeline/cli.ts:163-170` are unexercised. Fix: add one argv-level
   test for `run --max-parallel 3` / `OPENCODE_MAX_PARALLEL` fallback
   landing in `cfg`, and one that a bad env rejects before any client/lock
   side effect, mirroring `tests/cli.test.ts`.

4. **[nit] `src/pipeline/runner.ts:1301-1309` best-effort escalation sweep
   has no failing test** — the catch that keeps escalation landing when the
   sweep throws is the guard the proposal's r1 security revision added,
   but nothing forces `enforceOwnership` to throw. Fix: a fixture that
   plants `.git/index.lock` (or an unresolvable `headBefore`) before the
   escalation pass and asserts the warn + `finalizeEscalate` anyway.

5. **[nit] `src/pipeline/runner.ts:1198-1200` `seats.length === 0` early
   return is unreachable** — `planPass` and `specialtiesFor` never hand an
   empty seat list, so it is dead defensive surface. Fix: drop it, or
   comment it as a guard and leave it (it cannot be tested through
   `runReviewPass` since that is not exported).

## FYI (out of specialty, no severity)

`docs/workflows.md` and both READMEs gained no `--max-parallel` /
`$OPENCODE_MAX_PARALLEL` entry, `N ≤ server slots` guidance, or removal of
the ADR-003 "Proposed, not implemented" bullet, which the proposal's
Configuration section and Rollout step 3 require the change to land — my
grep of `docs/` finds the knob only in `src/pipeline/cli.ts`, yargs
describes, and the run's own design directory.

## `npm run check` tail

Not run: the review seat's shell tool is permission-denied
(`Permission denied: shell`, two attempts). Verdict below therefore treats
green as unverified pending the panel runner's independent check.

## Verdict

approve — no blocker/major defects in the test surface itself; the open
items are the seat's unverifiable check run (minor, covered by the panel
runner), one timing-budget flake risk, and one missing mirror-wiring
test.
