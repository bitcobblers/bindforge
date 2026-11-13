# Code review — testing seat (round 2, pass c0)

Design: `docs/designs/016-harness-evidence-and-read-rights/02-proposal.md` (ADR-016).
Scope reviewed: `git diff 22ce12bf09a600d3983f374294d9d5844b07af54...HEAD -- ':(exclude)docs/designs'`
(37 source/config files, 12 test files, ~3.9k insertions). Inputs: that diff, the proposal,
ADR-016, and the runner's gate record — which reports none for this run — plus my own
prior findings, of which I have none (first pass).

## Verdict: approve

Zero open blocker/major findings. The test work is unusually good: every numbered Test
strategy item (#1–#14) has at least one test citing it (`// proves: Test strategy #N`), the
new suites span the unit and integration layers the proposal promises, and I ran the
package check green myself. The open items are three minors (coverage gaps around two
defensive code paths and one literal promise of strategy #14) and three nits.

## Checklist walk

1. **Citations.** All 14 strategy items cited: #1 `gate-evidence-capture.test.ts`
   (legs/exit/tail-from-end/UTF-8 replacement/spawn-failure/elision leg/cargo-as-data),
   #2 `gate-evidence-checkpoint.test.ts` + `gate-evidence-render.test.ts` (round-trip,
   golden fixtures load with the field absent, `printable` + array bound at write and
   re-applied at render), #3 `independence.test.ts` (canonical spellings clean; sibling,
   other design dir, directory, glob, grep path, off-run git-diff, own+sibling, prefix-less
   token, transcripts-never, child-scoping all fail), #4/#5 `agent-shell.test.ts` (design
   seats run every compound gate leg; doc families allow `wc <file>`/`cat <path>` in star
   form and deny the toolchain; run-record fence + per-seat own-specialty re-open vs
   sibling denied; code-seat invariants kept for the design panel), #6 `document-cap.test.ts`
   + the new `proposal-size.test.ts` block (`documentSize` measures `01-research.md` for
   `research`), #7 `gate-evidence-prompt.test.ts` + render cap/order tests, #8
   `transcript-tokens.test.ts`, #9/#10/#13 `gate-evidence-integration.test.ts`, #11
   `own-report-seat.test.ts`, #12 `document-cap-integration.test.ts` (fail/retry/escalate
   with cap named; raising `docMaxLines` passes), #14 cited at
   `gate-evidence-capture.test.ts:203` (see finding 3 for how literally it is met).
2. **Layers.** Unit: the capture/checkpoint/render/prompt/cap/token/identity/agent-matrix
   suites. Integration: red-gate round trip, green-gate seat prompt, own-report seat
   through `resumePipeline`, oversized-document escalation, pre-016 resume. Both promised
   layers exist and the integration tests drive the real runner with stub clients.
3. **Revision rounds.** My seat has no prior findings. Of the proposal's own r2 revision
   rows, the load-bearing one (whole shell token, prefix included; prefix-less spelling
   fails closed) is pinned by `independence.test.ts` ("fails closed on a prefix-less
   reviews token", "own plus a sibling in one command fails") — a test that would have
   caught the round-1 ambiguity.
4. **Determinism.** No network, no live server, no ports: all inference is stub
   `OpenCodeClient`s over `mkdtemp` git fixtures. `process.env.PATH` swaps (unresolvable
   npm; stubbed `cargo`) are restored in `finally` and vitest isolates per file. The
   `Date.now()` stamps in `own-report-seat.test.ts` exist because the audit's `since` is
   a real timestamp and are monotonic-guarded — deterministic in outcome. The
   `FLOWS.full.config` mutation in the cap integration test is restored in `finally`.
5. **Names.** Test names state behavior ("lets the seat re-read its own report without
   burning the attempt", "renders failing legs before green legs", "fails the seat that
   also opens a sibling report, naming the sibling"), not implementation steps.
6. **Check run.** Ran myself (there is no runner gate record for this run):
   `npm run check` in `tools/orchestration` (lint + format:check + vitest), working tree
   clean for that package. Tail:

   ```
    Test Files  51 passed (51)
         Tests  557 passed | 2 skipped (559)
      Start at 14:22:32
      Duration  7.32s (tests 88%, import 9%, transform 3%)
   ```

   Green. The `mount: /tmp/bindforge-mnt-probe-…: must be superuser` line above the
   summary is a pre-existing probe fixture's expected stderr, not a failure.
7. **Untestable additions.** `docMaxLines` has a consumer and a refusal test;
   `gateEvidence` is defaulted, rendered, and capped; the new agent rules are pinned by
   the shell matrix. One unreachable guard remains (finding 5, nit); the two untested
   defensive legs of `runLeg` are findings 2 and 6.

## Findings

1. **minor — Test strategy #14's "cargo legs run against the real workspace" is only met
   by stub.** `tools/orchestration/tests/pipeline/gate-evidence-capture.test.ts:203`
   exercises the cargo legs with a `fake-bin/cargo` PATH shim, and the green-gate
   integration tests (`gate-evidence-integration.test.ts`) use a node `check` script, so
   no test in the suite runs a real `cargo` binary; the strategy text says the cargo legs
   run "against the real workspace in the integration tests that need a green gate."
   The real command lines do run against the real workspace in CI's `rust` job
   (`.github/workflows/check.yml:28-33`) and in the shipped pipeline's own gate, and the
   stub deliberately keeps the host toolchain out of unit tests — but the suite itself
   does not close the loop the strategy sentence describes. Fix: either add one
   probe-guarded integration case that points `checkGate` at the repo's own workspace when
   `cargo` is on `PATH` (asserting the three `CARGO_GATE` legs and a real `test result:`
   line), or amend the fixture comment/build note to record that the real-workspace leg
   lives in the CI `rust` job and the runner gate, not the vitest suite.

2. **minor — the rolling 64 KiB tail buffer is never exercised.**
   `tools/orchestration/src/pipeline/gates.ts:369-375` drops oldest chunks once a leg's
   kept bytes exceed `MAX_TAIL_BYTES` (gates.ts:318), but the largest fixture leg emits
   ~4 KiB (`gate-evidence-capture.test.ts`, `big` mode), so the eviction loop never runs
   and a bug there (dropping from the wrong end, miscounting `kept`, splitting a code
   point across the eviction seam) would pass the whole suite while corrupting tails of
   chatty legs — exactly the artifact behavior claims cite. Fix: add a generator mode that
   writes ~200 KB (fill + `\nTAIL_END\n`) and assert the tail still ends at the leg's last
   line and stays ≤ `MAX_GATE_TAIL_CHARS`.

3. **minor — the run-record `glob`/`grep` deny set is pinned for only 10 of 17
   shell-holding agents.** `agent-shell.test.ts:340` loops `DOC_FAMILIES + DESIGN_SEATS`;
   the run-record blocks added to `code/coder.md`, `code/test-engineer.md` and the five
   `codereview/*` seats in this diff are asserted only for their *shell* leg
   (the `SHELL_HOLDING` deny test, which does cover all 17). Coder/test-engineer/code-seat
   `glob`/`grep` run-record denies are shipped config with no pin, so a future edit can
   drop them silently; `docs/workflows.md` now promises the denies for "every
   shell-holding stage agent." Fix: extend the `deniesResource` loop at
   `agent-shell.test.ts:340` to `CODE_AGENTS` and `REVIEW_AGENTS`.

4. **nit — signal-killed leg untested.** The proposal names two `exit: null` cases;
   `runLeg`'s signal branch (`gates.ts:406`, tail "killed by signal …") has no fixture.
   Fix: a mode that `process.kill(process.pid, 'SIGKILL')`s itself and an assertion on
   `exit === null` plus the spelled signal in the tail.

5. **nit — unreachable fail-closed guard.** `independence.ts:251`'s
   `tokens.length === 0 ||` is provably dead: whenever `REVIEWS_SEGMENT.test(command)` is
   true, at least one `TOKEN_BOUNDARY`-split token re-matches `REVIEWS_SEGMENT` (the token
   class is the segment class minus `/`, and a surviving `/` is itself a segment
   delimiter). The guard fails closed, which is why it is harmless, but per checklist #7
   a dead branch is a finding. Fix: drop it, or comment it as a defense-against-divergence
   between the two character classes so a reader does not expect it to be exercised.

6. **nit — `gateEvidenceText`'s render-time array re-bound not asserted directly.**
   `gate-evidence-render.test.ts` proves `printable` is re-applied at render and the
   block cap holds, but no case pushes more than `MAX_GATE_LEGS` legs through
   `gateEvidenceText` to show `boundedGateEvidence` re-runs there (strategy #2's "re-
   applies both"). Fix: pass 40 legs and assert the rendered block names the elision note
   while staying within `MAX_EVIDENCE_BLOCK_CHARS`.

## FYI (no severity)

- `gateEvidenceText(state, cfg)` accepts the resolved flow config and `void cfg`s it — a
  documented per-flow evidence-budget seam the shipped design deliberately does not
  price yet (Rollout 2: one constant). Not a knob without a consumer, just a seam ahead
  of its use.
- The uncommitted working-tree edits I noticed outside the diff range
  (`.opencode/skills/{repo-practices,research}/SKILL.md`, the run's `.pipeline.json`)
  are not part of `22ce12b...HEAD` and do not affect this package's check.
