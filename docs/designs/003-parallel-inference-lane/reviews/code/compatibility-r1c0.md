# Code review — compatibility — round 1, pass c0

Diff reviewed: `git diff 73200b44dc6ae47e18fe9aba25cd7a0e9167136e...HEAD -- ':(exclude)docs/designs'`
Inputs: that diff, `02-proposal.md`, `docs/adrs/003-parallel-inference-lane.md`.

Verdict: **request-changes** — 1 major, 4 minor, 1 nit.

## Checklist result

| # | Item | Result |
|---|---|---|
| 1 | Platform code matches the Compatibility matrix | Findings 1, 2, 6 — the matrix's `N > 1` support story (operator guidance) never lands in the docs, and OS assumptions were added while the matrix's stated basis is "no OS-specific code added" |
| 2 | Node floor (`engines: >= 22.12.0`, A10) | Pass — everything new (`structuredClone`, `Promise.allSettled`, `AbortSignal.timeout`, `randomBytes`, `readdir`) is far below the floor; no engines change needed |
| 3 | Paths built with `path`/URL joins, case assumptions | Pass with nit 6 — `join(pkg, 'node_modules', '.bin', 'prettier')`, `relative()` used; the one hand-built prefix is the separator assumption in nit 6 |
| 4 | Guarded platform imports | Pass — no platform-only module is imported anywhere in the diff |
| 5 | Non-ASCII names / IDs survive the formats and IDs | Pass — `dirtyFiles` uses `--porcelain=v1 -z` (runner.ts:179-183), so raw UTF-8 paths survive, and prettier is spawned with argv (no shell); `resolveMaxParallel` reflects env bytes through `JSON.stringify`/`String`. Finding 3 covers the metacharacter case |
| 6 | Native/packaging dependencies introduced | Pass on npm deps — no new dependency, the semaphore is hand-rolled as designed. The new *external tool* invocation is a packaging story: findings 2 and 3 |
| 7 | Tests named in the plan run where they claim | Pass — Test strategy #2/#3/#4/#5/#6/#7/#8 map to `pipeline-parallel-fanout/-escalation/-integration`, `pipeline-pass-finalize`, `pipeline-checkpoint-writer`, `pipeline-cli-max-parallel`, `pipeline-concurrency`; `pipeline-pass-finalize.test.ts:200-252` covers N = 1 and N = 3 plus the editorial `-rN` naming. The POSIX-only parts (git, the `#!/bin/sh` prettier stub in `pipeline-fix-trigger.test.ts:80-86`) run exactly in the Linux CI the matrix claims; no test in the diff is skipped or platform-conditional |

## Findings

### 1. major — `docs/workflows.md:83-91` (and `tools/orchestration/README.md`): the operator-facing compatibility story for `N > 1` never lands, and the docs now contradict the shipped code

Problem. The change ships an operator-facing knob and the docs that carry its compatibility guidance are untouched:

- `docs/workflows.md:83-85` still states "Seats execute one at a time (single local inference lane); the order is fixed for reproducibility" — false for any run with `--max-parallel ≥ 2`, and the same paragraph never mentions that the commit shape changed at the default too (ADR-003 §Self-update requires that caveat not be claimed away).
- `docs/workflows.md:87-91` still lists **ADR-003 — parallel review lane (proposed)** under "### Proposed, not implemented", the exact entry Rollout step 3 says must move out of that list with the change.
- `--max-parallel` / `$OPENCODE_MAX_PARALLEL` appears nowhere in `docs/workflows.md` (including the CLI block at lines 95-100 the proposal names) nor in either README; the only place the `N ≤ server slots` guidance and the "each seat's deadline starts at its own prompt and counts queue waiting" note exist is a comment in `src/pipeline/cli.ts:56-64` and the yargs `describe` strings. The matrix row "Run with N > 1 — yes, conditional on the server exposing ≥ N slots" has no discoverable support story, and A1/A2 (the runner cannot verify slot count) make that guidance the only mitigation for the change's top-rated risk.
- The one `docs/workflows.md` hunk in the diff (lines 75-81) documents something else entirely (runner-side commit/format hygiene).

Fix. In the same change: add the knob to the `docs/workflows.md` CLI block (`pipeline run/resume <slug> --max-parallel <n>`, `$OPENCODE_MAX_PARALLEL`, default 1) with the `N ≤ server slots` guidance and the per-seat deadline/queueing note; delete the ADR-003 bullet from "Proposed, not implemented"; rewrite the "one at a time" sentence to "up to N seats at a time (`--max-parallel`, default 1 — the serial lane), launch order fixed, one commit per pass". Add the knob to `tools/orchestration/README.md`'s flag table if that table is meant to list pipeline knobs, or say in the PR why it is not.

### 2. minor — `tools/orchestration/src/pipeline/runner.ts:281-306` (`formatTouchedFiles`): the formatter dependency is a hard-coded bin path, skipped silently

Problem. The new hygiene step shells out to `join(pkg, 'node_modules', '.bin', 'prettier')` — a devDependency bin shim reached by direct path instead of npm's resolution — and when that path does not exist the loop does a bare `continue` with no log line (line 298-300). Consequences:

- Under any hoisted install layout (npm workspaces, pnpm, yarn) or in a CI checkout without devDependencies, `packageDirs()` finds the package but the shim is one level up, so formatting silently stops happening and the run-003 deadlock this function exists to remove comes back with no signal at all — the comment "the gate will say so instead" is only half true: the gate goes red and the fix loop has no idea the formatter was never run. Today's repo has exactly one package with a local install (`tools/orchestration/package.json:39`), so this is latent, not live.
- `.bin/prettier` is the POSIX sh shim; on Windows npm writes `prettier.cmd`, so the existence check passes and `spawnSync` fails. That is tolerable against the matrix ("Windows: no"), but the matrix's stated basis is "no OS-specific code added", and this is OS-specific behavior.

Fix. Resolve the formatter the way npm does — walk up from `pkg` to the nearest `node_modules/.bin/prettier`, or `spawnSync('npm', ['exec', '--no', '--', 'prettier', '--write', ...], { cwd: pkg })` — and `console.warn` naming the package whenever it has dirty formattable files and no formatter was found, so the skip is never silent.

### 3. minor — `tools/orchestration/src/pipeline/runner.ts:281-303`: the formatter batch includes git-deleted paths and hands every path to prettier as a glob pattern

Problem. The file list comes from `dirtyFiles()`, which strips the porcelain status letters (`porcelainPaths`, runner.ts:167-172), so `D ` (deleted) and untracked-then-deleted paths are indistinguishable from modified ones and go to `prettier --write`. Prettier exits non-zero for a pattern that matches nothing ("No files matching the pattern were found") and treats each positional argument as a glob, so:

- one stage that deletes a `.ts`/`.md` file (an implement/fix stage removing a superseded module is normal) makes the whole `--write` batch for that package exit non-zero, and the `res.status !== 0` branch only warns — every other dirty file in that package is left unformatted, i.e. the deadlock case returns, silently;
- a dirty path containing glob metacharacters (`[`, `]`, `*`, `?`, `{`, `}` — legal in filenames, so a non-conventional slug or report name reaches it) is reinterpreted as a pattern, matching nothing or matching siblings it was never meant to touch.

`pipeline-fix-trigger.test.ts:87-108` only ever dirties *modified* files with plain names, so CI is green and the failure mode is latent.

Fix. Keep the status letters in the listing used by the formatter (or filter on `existsSync(join(pkg, p))`) so deleted paths are skipped, and pass the batch after `--` or format file-by-file via `--stdin-filepath` so argv are not glob patterns; add a fixture with one deleted `.ts` and one `[weird].ts` name asserting the rest still get formatted.

### 4. minor — `tools/orchestration/src/pipeline/states.ts:29-39`: the gate-label rename changes the vocabulary of already-persisted gate-log values

Problem. `gateNameFor` now emits `design-review` / `editorial-review` / `code-review` instead of `allDesignReviewsApproved` / `allEditorialReviewsApproved` / `allCodeReviewsApproved`. Those strings are not display-only temporaries: they are written into `.pipeline.json`'s `gateLog` (`gate: z.string()`, state-store.ts:99), printed by `pipeline status` (runner.ts:1974-1976) and by the escalation report (report.ts:279), and `docs/adrs/001-code-driven-agent-pipeline.md:190` still documents the old names as the identity of the aggregated-verdict gates. The proposal's Compatibility matrix ("Checkpoints written before this change — yes — No checkpoint schema change") and Rollout step 2 ("No migrations") do not cover a value-vocabulary change, so a run whose checkpoint holds `allDesignReviewsApproved` entries and resumes on this code shows both vocabularies in one gate log, and any operator note or script keyed on the documented name stops matching. Nothing in the diff needed this: the pass gate's entry reason and the pass's stage name already carry the stage.

Fix. Revert `gateNameFor` to the ADR-001 names (the label change buys nothing the entry reason doesn't already say), or keep it and record it: update the ADR-001 gate table / `docs/workflows.md` and add one Rollout line saying gate-log labels change and old entries keep the old names.

### 5. minor — `tools/orchestration/src/pipeline/state-store.ts:156-173`: `updatedAt` loses its only writer, so a required persisted field silently stops meaning "last checkpoint"

Problem. The `state.updatedAt = Date.now()` line is removed and nothing else stamps it: `updatedAt` is written exactly once, at state creation (runner.ts:435-436). So after this change every checkpoint on disk carries `updatedAt === startedAt` for the whole run, while the schema still requires the field (`types.ts:224`, state-store.ts:107) and `version` stays 1. The rationale in the new comment is right for the writer (a queued snapshot must not masquerade as newer), but the fix drops the timestamp instead of moving it, which is an unannounced semantic change to a persisted artifact — the opposite of the matrix's "no checkpoint schema change" comfort. It also makes the new ordering test's assertion depend on hand-set values (`pipeline-checkpoint-writer.test.ts:52-84`) rather than on the writer recording when the snapshot was taken.

Fix. Stamp at snapshot time rather than at rename time: in `createStateWriter.enqueue` set `snapshot.updatedAt = Date.now()` before queueing, and restore the stamp in `saveState` for the non-pass sequential path (or have it stamp only when `state.updatedAt` is older than now). Either way both write paths keep meaning "when this snapshot was taken", and the writer test can assert the ordering without the fixture hand-setting `updatedAt`.

### 6. nit — `tools/orchestration/src/pipeline/runner.ts:290-296`: package-prefix matching assumes `relative()`'s separator equals git's

Problem. `prefix = rel === '' ? '' : \`${rel}/\`` builds the matcher from `relative(repoDir, pkg)` and compares with `p.startsWith(prefix)` against git porcelain paths, which are always `/`-separated. On a platform where `relative()` yields `\` the prefix never matches, so `formatTouchedFiles` becomes a silent no-op — harmless under the Linux-only matrix, but it is another OS assumption added against a matrix row that says "no OS-specific code added".

Fix. Normalize once: `const rel = relative(repoDir, pkg).split(sep).join('/')`, or state the Linux/POSIX assumption in the matrix row's Notes so the next reader knows the formatter is not portable.

## FYI (no severity, out of this seat's scope)

The diff carries substantial work the proposal does not describe — `src/pipeline/independence.ts` plus its wiring into the polling loop (runner.ts:839-852), `formatTouchedFiles`, the `stageTemplate`/`fix-tests.md` trigger, and the `.opencode/agents/*` prompt changes. Compatibility of those pieces is reviewed above only where it is compatibility; whether they belong in this change at all is for the scope seats and the editor.
