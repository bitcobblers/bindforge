# Code review — testing seat — ratification pass c2 (r1)

Scope: `git diff 91dfbea2f0ab34c063a14036b3afdc1f4b4227d9..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and ADR-003. This pass checks only that the
post-r1 revisions broke nothing this seat approved in round 1.

## Diff under review (from the branch log)

- `fix: drop runner-executed formatter, stamp checkpoints at write sites, harden audit`
- `test: 003-parallel-inference-lane test-update stage`
- two `docs: … code-review stage` commits (docs/designs only — out of scope)

## Checklist result

1. **Proposal Test strategy coverage — pass.** Every numbered item #1–#9 still has at least
   one `// proves: Test strategy #N` citation after the test-update commit: #1/#4 in
   `pipeline-concurrency.test.ts` + `pipeline-parallel-fanout.test.ts`, #2/#3 fanout, #5
   `pipeline-parallel-escalation.test.ts`, #6 `pipeline-pass-finalize.test.ts` (incl. the
   editorial `-rN` naming case), #7 `pipeline-checkpoint-writer.test.ts`, #8
   `pipeline-cli-max-parallel.test.ts`, #9 `pipeline-parallel-integration.test.ts`.
2. **Unit + integration layers — pass.** #9 end-to-end at N = 2 with the N = 1 control-flow
   comparison (result order included) is intact.
3. **Revisions carry catching tests — mostly pass, one gap (finding 2).**
   - Checkpoint stamping moved to write sites (`runner.ts:417-431` `checkpoint()`,
     `runner.ts:1213-1216` `enqueueSnapshot`, `state-store.ts:159-190` no re-stamp): the
     contract is pinned — writer tests assert `updatedAt` survives the queue and an invalid
     snapshot leaves the previous checkpoint untouched (`pipeline-checkpoint-writer.test.ts:52-117`),
     and the integration test asserts the landed checkpoint is stamped
     (`pipeline-parallel-integration.test.ts:230-234`). All sequential writes go through
     `checkpoint()` (12 call sites checked); no unstamped `saveState` call remains in the runner.
   - Audit hardening (`independence.ts`): regression tests for the run-003 misfires (write
     content, denied reads, completed glob) and for the segment-matching bypasses named in
     the security r1 finding (`pipeline-independence.test.ts:132-220`). Pass.
   - **Formatter-step removal** (security r1 major 1): the fix deletes the step and the
     test block that previously *pinned the defective behavior*, leaving only a comment
     (`pipeline-fix-trigger.test.ts:15-20`, `runner.ts:267-285`). No test now prevents the
     exec's return — finding 2.
4. **Determinism — pass, with the r1-accepted caveat unchanged.** Stubs are in-process
   mock clients; the fan-out and integration fixtures stagger prompt windows by design
   (`15 + 20k` ms) so completion order is fixture-determined. The escalation N = 2 fixture
   remains timing-coupled but now **explicitly states the dependency** in the stub header
   and fixture comments (`pipeline-parallel-escalation.test.ts:122-131, 261-265`), and
   switches to the `invalid-reply` failure mode (`minNudgeIntervalMs: 0`) so the same-permit
   retry is genuinely what runs — this is the documented-margin option from my r1 finding 2;
   accepted, no new finding.
5. **Behavior-named tests — pass** (unchanged naming from r1).
6. **`npm run check` — NOT evaluated by this seat (finding 1).** The shell tool was
   permission-denied in this pass as well (three attempts, `Permission denied: shell`), so
   the suite's green status at default N = 1 is again unverified here. Per the checklist this
   counts as a finding, not a pass; the panel runner's independent `npm run check` in
   `tools/orchestration` decides, and a red run overrides this verdict.
7. **Untestable additions — no new ones from the revision.** Stamping sites all have
   consumers and tests; the audit fields have tests. The r1 unreachable
   `seats.length === 0` guard (`runner.ts:1198-1200`) is unchanged and still unmarked —
   carried nit.

## r1 findings — disposition after the revision

- Finding (minor, checklist #6, suite unrun here): **unchanged** — still open, finding 1.
- Finding (minor, N = 2 escalation timing coupling): **addressed** — the fixture now follows
  the proposed documented-margin option (explicit dependency comment in the stub header and
  at the fixture; failure mode pinned to `invalid-reply` with `minNudgeIntervalMs: 0`).
- Finding (minor, argv-level `--max-parallel` coverage): **not addressed** — still
  resolver-level only; carried as finding 3.
- Finding (nit, no test forcing the escalation sweep to throw): **not addressed** — the
  best-effort catch (`runner.ts:1311-1319`) has no forcing fixture; carried as finding 4.
- Finding (nit, unreachable `seats.length === 0` guard): **not addressed** — carried as
  finding 5.

## Findings

1. **minor** — `tools/orchestration` (whole diff), checklist item 6. This seat again could
   not run `npm run check` (shell permission-denied, three attempts); green at default N = 1
   is unverified here. Fix: rely on the panel runner's independent `npm run check`; red
   overrides this verdict.
2. **minor** — `tools/orchestration/tests/pipeline-fix-trigger.test.ts:15-20` /
   `src/pipeline/runner.ts:267-285` (checklist item 3). The fix for security r1 major 1
   (runner execs a model-plantable binary under `node_modules/.bin`) removed the old
   formatter-stub test and left only a header comment; nothing now fails if the exec is
   reintroduced. Fix: one regression fixture that plants a recording stub at
   `<pkg>/node_modules/.bin/prettier` (package dir touched by a stage), runs a pass that
   commits through `commitStage`, and asserts the stub never executed.
3. **minor** — `tools/orchestration/tests/pipeline-cli-max-parallel.test.ts` (carried r1).
   Coverage is still resolver-level only: the yargs coercion of `--max-parallel` and the
   resolve-before-createClient/lock ordering (`src/pipeline/cli.ts:161-170`) remain
   unexercised, while the file's own header asserts "all validation before side effects".
   The `--timeout` mirror in `tests/cli.test.ts:170-213` has both; the knob's header comment
   is the only "proof" here. Fix: as in r1 — one argv-level test for `run/resume
   --max-parallel 3` / `$OPENCODE_MAX_PARALLEL` landing in `cfg.maxParallelReviews`, and one
   that a malformed env rejects before `createClient`/lock.
4. **nit** — `tools/orchestration/src/pipeline/runner.ts:1311-1319` (carried r1). The
   best-effort escalation-sweep catch still has no test forcing `enforceOwnership` to throw
   (e.g. planted `.git/index.lock`) asserting the warning plus the terminal escalate commit.
5. **nit** — `tools/orchestration/src/pipeline/runner.ts:1198-1200` (carried r1). The
   `seats.length === 0` early return remains unreachable and remains unmarked as a
   defensive no-op.

FYI (no severity): the escalation lane fixtures now pin the `invalid-reply` failure mode
via `minNudgeIntervalMs: 0`, which makes them prove the same-session retry path rather than
the stage-timeout zombie path — a strict improvement over what r1 saw.

## Verdict

The revisions land their own test coverage and did not break any test-side guarantee this
seat approved in round 1. No blocker/major findings; the open items are one unevaluated
checklist item (delegated to the panel runner), one new item-3 gap on the formatter fix, and
carried r1 minors/nits.

**approve**
