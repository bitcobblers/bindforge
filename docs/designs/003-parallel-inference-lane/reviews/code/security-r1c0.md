# Code review — security seat, round 1 pass c0

Scope: `git diff 73200b44...HEAD -- ':(exclude)docs/designs'` (25 files, ~2.5k
insertions). Judged against `02-proposal.md` (ADR-003 parallel inference lane)
and `docs/adrs/003-parallel-inference-lane.md`. Previous findings: none (first
pass for this seat at code level).

## Checklist results

| # | Item | Result |
|---|---|---|
| 1 | External input validated at boundary | Pass with nit — `resolveMaxParallel` (`cli.ts:66–101`) is strict fail-closed: `^\d{1,5}$` env gate even when the flag overrides, separate range check naming the source, all validation before client/lock/checkpoint side effects (`cli.ts:161–170`). Checkpoint load/save both run through `StateSchema` (`state-store.ts:145,170`); the new optional `findings` field is schema-validated and optional, so pre-change checkpoints still parse. Nit #4: re-injected findings are size-unbounded. |
| 2 | Path handling | Pass — every git invocation is `spawnSync('git', args[])` with array args; no string-interpolated shell. Design dir derived from regex-validated `nnn`/`slug` (`types.ts:151–152`, enforced on load). New writer's tmp names use `randomBytes(4)` inside the design dir (`state-store.ts:178`) — no traversal, no symlink-following path from untrusted data. |
| 3 | Injection | Major #1 (formatter execution) and minor #2 (audit substring bypass). New regexes are linear (`^\d{1,5}$`, `PRETTIER_EXT`); no backtracking surface. |
| 4 | Secrets / PII | Pass — no secrets in source, fixtures, or new tests (auth stays env→client, never logged). Minor #3: model-authored strings reach the terminal/log unsanitized (control bytes, not secrets). |
| 5 | New dependencies | Pass — none. The semaphore is the hand-rolled ~20-line counter + FIFO queue the design mandates (`concurrency.ts`, matches ADR-003 §Execution 3); `package.json` untouched in the diff. |
| 6 | Filesystem writes | Pass — checkpoint writes remain atomic tmp+rename; the new `createStateWriter` (`state-store.ts:201–221`) serializes in-pass writes in enqueue order and `runReviewPass` drains it before the pass's `git add -A` (`runner.ts:1285`) — exactly A7's fix, and pinned by `pipeline-checkpoint-writer.test.ts`. |
| 7 | Clean rejection | Pass — malformed config/env is a usage error before any side effect; corrupted checkpoints get a recovery instruction, not a stack trace (`state-store.ts:126–136`); escalation-path sweep is deliberately best-effort so `finalizeEscalate` still lands (`runner.ts:1291–1310`), matching the proposal's r1 security revision; `pipeline-pass-finalize.test.ts` / `pipeline-parallel-escalation.test.ts` pin the pre-escalation ownership sweep (out-of-policy stray never reaches the terminal commit). |

Fan-out itself is sound on this seat's terms: the pass window (`headBefore`
captured once, one sweep + one commit) removes the interleaving the ADR names;
`writer.drain()` before any git mutation removes the stray-tmp race; the
semaphore throws on a non-integer/<1 limit as a second gate behind the CLI
(`concurrency.ts:17–19`, `runner.ts:1203` falls back to `DEFAULT_CONFIG`).

## Findings

- **[major] `runner.ts:297–305` (`formatTouchedFiles`, new) — the runner executes a convention-resolved, git-ignored binary from the worktree.**
  `bin = join(pkg, 'node_modules', '.bin', 'prettier')` is spawned before every
  commit (via `commitStage` at `runner.ts:317`). The repo `.gitignore` ignores
  `node_modules/` entirely, so a planted `package.json` + `node_modules/.bin/prettier`
  under any probed package dir (`packageDirs` probes root, top-level, `tools/*`,
  `packages/*`) is invisible to `dirtyFiles`/`enforceOwnership` (which list only
  non-ignored files), never appears in a human-reviewed commit, yet
  `existsSync(bin)` + `spawnSync(bin, …)` runs it as the runner — whose privileges
  include the checkpoint, git authorship, and the lock — on that stage's own commit
  and every commit after. An implement/fix session is exactly the threat this
  change set defends elsewhere (coder agent permission `edit "*"`, ownership rule
  `!inTests` at `gates.ts:263–266`; independence.ts's premise that permissions are
  advisory against an injected model): a session steered by injected diff content
  can drop an undetectable payload the runner executes. The pre-existing
  `npm run check` channel runs model-editable scripts too, but those are at least
  committed and reviewable; this one is untracked forever. The new
  `pipeline-fix-trigger.test.ts` proves the mechanism (a `#!/bin/sh` stub in
  `tools/pkg/node_modules/.bin/` is executed). Not in the proposal at all — a
  drive-by addition.
  **Fix:** never exec a name-conventional path inside the worktree's `node_modules`.
  Either (a) run the formatter through the package manager from the repo root's own
  install (`spawnSync('npm', ['exec','--no-install','--','prettier','--',…], {cwd: repoDir})`
  or equivalent), (b) verify the formatter's install against the committed
  lockfile before allowing any stage-execution path to trust it, or (c) drop the
  auto-format step and let the `format:check` gate name the files (the deadlock
  this function papers over has a fix-loop ownership answer too). Whichever way,
  record why model-planted executables cannot ride into runner execution.

- **[minor] `independence.ts:77,91–98` — the independence audit's substring matching is trivially bypassable, and cross-seat report *writes* are not audited at all.**
  Detection is `value.includes('transcripts/')` / `'reviews/'`: a completed shell
  `ls docs/designs/003-x/transcripts` (no trailing slash — enumerates the run-records
  directory itself), a glob pattern ending in `/transcripts`, or command
  substitution (`cat "$(echo …/transcr; echo ipt/x)"`) all access or enumerate run
  records without matching. For review seats the audit flags *reads* of `reviews/`
  but a seat can `write`/`edit` a sibling's report file — that survives the pass
  ownership sweep (`docs/**` is in-policy) and corrupts the panel's run record,
  the same independence property the feature exists to protect. This is
  defense-in-depth over the agent-permission fence, so minor — but the module
  comment claims it "proves it", which the current matcher does not.
  **Fix:** match path *segments* (`/(^|\/)transcripts(\/|$)/`-style segment test,
  same for `reviews/`), and in review stages flag write/edit/patch parts whose
  path is under `reviews/**` and is not the seat's own handed `reportPath`.
  Document the residual bypass surface (obfuscated shells) as unprovable-by-design.

- **[minor] `independence.ts:135–137` + `runner.ts:850,1270,1234` — model- and server-authored strings reach the operator terminal, the checkpoint gate log, and `ESCALATION.md` unsanitized.**
  `brief()` truncates at 140 chars but keeps control bytes; a violation summary
  embeds model-chosen path/command text, and `stageFailure` gate entries embed
  server-derived failure strings — all printed via `console.warn` and re-displayed
  by `pipeline status`. An ANSI/control-sequence payload in a path or command can
  spoof or corrupt the operator's terminal output. `cli.ts:56–64` explicitly takes
  the opposite posture for config values ("reflected through JSON.stringify /
  String so no control or ANSI bytes can reach the terminal raw"); the same
  discipline should cover these paths.
  **Fix:** in `brief()` (and wherever `out.failure` is logged/persisted), strip or
  escape control characters, e.g. `s.replace(/\p{Cc}/gu, ' ')` or reflect via
  `JSON.stringify`.

- **[nit] `runner.ts:1852–1856` + `types.ts:11–16` — the findings re-injection channel is shape-validated but size-unbounded.**
  `prevFindings` embeds the seat's stored findings verbatim into the next pass's
  prompt, and they are persisted into every subsequent checkpoint snapshot
  (re-cloned, re-serialized on every write). `FindingSchema` puts no cap on
  `location`/`problem`/`fix`, so one oversized model reply permanently inflates
  the checkpoint and every later prompt of that seat. Checklist item 1 wants
  schema-validated **and size-limited**. **Fix:** bound the re-injected payload
  (truncate with an elision note in `reviewVars`) or add `.max(…)` to
  `FindingSchema`.

## FYI (out of specialty, no severity)

`docs/workflows.md` still says seats "execute one at a time" and still lists
ADR-003 under "Proposed, not implemented", and neither README documents
`--max-parallel` — Rollout step 3 of the proposal says those doc moves land
with this change; the compatibility seat's territory. Also FYI that
`formatTouchedFiles`, `fix-tests.md` selection, and the independence audit are
not in the proposal's design surface at all (drive-by scope), which the
correctness/compatibility seats will want to weigh.

## Verdict

`request-changes` — one open major (runner-executed, git-ignored, model-planted
formatter binary), three minors, one nit.
