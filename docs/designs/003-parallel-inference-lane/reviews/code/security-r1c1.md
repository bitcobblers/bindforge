# Code review — security — dissent pass c1 (round 1 re-check)

Scope: `git diff 91dfbea2..HEAD -- ':(exclude)docs/designs'`, read against the
full current `02-proposal.md` and ADR-003. (Note: the shell tool is not
available in this session; the review was performed on the full current
sources at HEAD, focusing on every path my r1 findings named plus the new
fan-out/writer/CLI/audit code, which is what this diff contains.)

Specialty: security checklist only (`references/security.md`).

## Re-check of my round-1 findings — all closed

1. **major — runner exec'd a convention-resolved binary from the worktree**
   (`formatTouchedFiles` → `node_modules/.bin/prettier` before every commit):
   **CLOSED.** The auto-format step is gone from `runner.ts`; the only
   remaining trace is the explanatory block at `runner.ts:267-285` ("the
   runner never executes a formatter") recording the planted-binary attack
   that motivated removal. Grepping the whole of `src/` finds no
   `spawnSync`/`execFile` of any worktree-resolved path: every `spawnSync`
   execs `git` from PATH with an argv array (no shell). The formatter-stub
   test that previously proved the mechanism was removed on purpose
   (`tests/pipeline-fix-trigger.test.ts:15-20`), and the replacement posture
   — red `format:check` gate summons `fix`/`test-update`, which run
   `npm run format` themselves from the committed `package.json` — is an
   ownership answer, not a runner exec. Matches my r1 fix option 3 (drop the
   step, let the gate name the files).

2. **minor — audit matched raw substrings; a seat could overwrite a sibling
   report**: **CLOSED in code.** `independence.ts:97-101` replaces the
   substring tests with whole-path-SEGMENT regexes delimited by
   start/end-of-string, `/`, and shell/JSON delimiter characters, so
   `…/transcripts` (no trailing slash), `…/transcripts;…`, and a bare
   `transcripts` argument all match, while a file *named*
   `transcripts-list.md` does not. Writes are now judged: completed
   `write`/`edit`/`patch` parts under `reviews/**` are compared by
   `reportKey` (from the `reviews` segment on, backslash-normalized, so the
   absolute path and the prompt-handed relative path identify the same
   report) against the seat's handed `reportPath` (`independence.ts:61,143-150,163-166`),
   and the runner hands it (`runner.ts:842-843`). The module header
   (`independence.ts:30-41`) now documents the residual obfuscation bypass
   (`$(printf …)`, split quotes, staged script files) as unprovable from the
   message list — exactly the honesty caveat my finding asked for. Segment
   bypasses named in r1 are pinned by tests
   (`tests/pipeline-independence.test.ts:185-220`). The write guard itself is
   untested — see new finding 1 below.

3. **minor — control bytes in model/server-authored strings reached the
   operator terminal**: **CLOSED.** `printable()` (`independence.ts:200-202`)
   strips `\p{Cc}` (C0+C1, so ESC/ANSI sequences cannot survive) and
   `brief()` applies it before truncation. It is applied at the choke points:
   the stage FAIL log line (`runner.ts:994`), `gateLog` for **every** entry
   including the pass fan-out's `stageFailure` reason (`runner.ts:1158` with
   `1242-1245`), `finalizeEscalate` before the gate log, terminal, and
   `ESCALATION.md` (`runner.ts:1983,2003`), `renderStatus` gate lines
   (`runner.ts:2035`), and `escalationMarkdown` findings and gate reasons
   (`report.ts:259,280`). This is now the same posture `cli.ts` takes for
   config values.

4. **nit — findings re-injection channel shape-validated but size-unbounded**:
   **CLOSED.** `boundedFindings` (`runner.ts:1821-1851`) caps the recorded
   payload (40 findings × 1 000 chars per field, each field also run through
   `printable`, plus an explicit elision marker recorded as a nit so it never
   gates), applied where verdicts are recorded into the seat board
   (`runner.ts:1673`) **and** again on the way into the prompt
   (`reviewVars`, `runner.ts:1906`), so a ledger written before the cap
   existed cannot smuggle a runaway finding into a prompt either. The
   checkpoint loader comment records the invariant (`state-store.ts:32-38`).

## New findings (only from changed code)

1. **minor — the sibling-report write guard fails open and no test exercises
   it.** The write check only runs when `ownReportPath` is a string; a caller
   without one "cannot judge writes" (`independence.ts:143-145`), and the
   runner passes `undefined` whenever `handle.vars['reportPath']` is not a
   string (`runner.ts:842-843`) — a renamed or missing template variable
   silently disables the entire write side of the audit with no signal.
   Nothing pins it: no test in `tools/orchestration/tests/` ever passes
   `ownReportPath` or asserts the `wrote a review report` violation (grep:
   the string appears only in `independence.ts`), and the two tests that read
   like write coverage (`pipeline-independence.test.ts:97-108, 222-233`) both
   omit `ownReportPath`, so they exercise the vacuous "cannot judge" branch
   and pass regardless of whether the guard works. This is the enforcement
   half of my r1 finding 2 — the property the whole feature exists to keep —
   resting on an untested wiring between `reviewVars` and the audit.
   **Fix:** add unit tests that (a) flag a completed `write` to
   `…/reviews/code/<other>-r1c1.md` while handed `…/security-r1c1.md`, (b)
   allow the own path and identify an absolute-path write of the own path as
   own, (c) assert `reviewVars` returns a `reportPath` for every review phase
   so the runner wiring cannot rot; optionally log a one-line warning when a
   review-stage audit runs with no handed report path.

## Checklist status

1. Boundary validation: replies zod-validated; seat findings now size-capped
   and control-stripped before persisting/injection; `resolveMaxParallel`
   fail-closed (`^\d{1,5}$` env gate even when the flag overrides; integer ≥ 1
   flag check; values reflected via `JSON.stringify`/`String`; all validation
   before any side effect — `cli.ts:55-101,161-166`). OK.
2. Path handling: no interpolated shell — every git call is `spawnSync` with
   an argv array; run-identity paths constrained by `NNN_RE`/`SLUG_RE`;
   transcript/report paths are runner-composed. OK.
3. Injection: audit regexes are linear (fixed segment + delimiter classes, no
   nesting/backtracking risk); no worktree-conventional binaries exec'd (see
   closed finding 1). OK.
4. Secrets/PII: none in the diff. OK.
5. New dependencies: none — the semaphore is in-repo (`concurrency.ts`),
   `package.json` unchanged. OK.
6. FS writes: checkpoint stays tmp-file+rename; the new
   `createStateWriter` snapshots with `structuredClone` at enqueue time,
   serializes renames in enqueue order, and drains before the pass's
   `git add -A` (`state-store.ts:209-229`, `runner.ts:1205-1216,1295`). OK.
7. Rejection posture: invalid checkpoints refuse with a recovery line, not a
   stack trace; invalid-state snapshots are rejected before writing
   (`state-store.ts:175-184`); semaphore rejects a non-integer/zero limit and
   `runReviewPass` defaults to the serial lane when the config field is
   absent (`runner.ts:1201-1204`). OK.

FYI (no severity, pre-existing, not made true by this revision):
`applyOwnershipReport` prints reverted file paths, and the revert-commit body
embeds them (`runner.ts:1017-1021,257-260`); a control byte inside a model-
planted *filename* would reach the terminal and git metadata unprintable.
Could be folded into the same `printable()` posture opportunistically.

## Verdict

All four r1 findings are closed in the changed code. One new minor (untested,
fail-open write-guard wiring). No blocker, no major open.
