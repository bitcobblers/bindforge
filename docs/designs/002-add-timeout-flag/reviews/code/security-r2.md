# Code review — security (round 2)

Range: `fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`
Design: `docs/designs/002-add-timeout-flag/02-proposal.md` (r4)
Checklist: `.opencode/skills/code-review/references/security.md` (security only)

## Scope actually reviewed

The range contains two code groups:

1. **The design's change set** — `src/cli.ts`, `src/opencode-runner.ts`,
   `package.json` (+`package-lock.json` engines mirror), `README.md`, and the
   new suites `tests/cli.test.ts`, `tests/opencode-runner-timeout.test.ts`.
2. **Pipeline self-repair commits landed mid-run** (`f0a2e16`, `891567b`,
   `d7be29f`) — `src/pipeline/gates.ts`, `prompts.ts`, `runner.ts`,
   `tests/pipeline-gates.test.ts`, `tests/pipeline-prompts.test.ts`, and the
   new agent config `.opencode/agents/codereview/code-compatibility.md`.
   The proposal declares pipeline changes a non-goal, but they are inside the
   caller's diff range, so they were reviewed under the security checklist
   too (scope question itself is an FYI, below).

`tests/opencode-runner.test.ts` (the pinned existing suite) is untouched by
the range — the "existing tests stay green" anchor is intact.

## Checklist results

1. **External input validated at the boundary — PASS.**
   `OPENCODE_REPLY_TIMEOUT_SECONDS` is a raw untrusted string and passes the
   strict `^\d{1,5}$` gate *before* any `Number()` call
   (`tools/orchestration/src/cli.ts:108`), precedence-independently (flag
   `5` + env `"30s"` still throws — `tests/cli.test.ts:119–123`); the flag is
   validated as an integer in `1..3600` (`cli.ts:116`); and the exported API
   boundary re-validates `timeoutMs` and throws a `TypeError` *before*
   `createSession`, with the fail-closed tests asserting zero
   `create`/`prompt`/`list` calls for `Infinity`, `NaN`, `0`, `-5`,
   `3_600_001` (`src/opencode-runner.ts:240–246`;
   `tests/opencode-runner-timeout.test.ts:140–154`). The security-r1 majors
   from design review are verifiably closed in code, not just in notes.
   The runner's own poll path only ever sees a validated integer, and the
   `3_600` s cap bounds the poll burst at both boundaries. No code evaluation
   anywhere on the new paths.
2. **Path handling — PASS.** The CLI change set performs no path or shell
   work. In the pipeline group, `enforceOwnership` builds no shell strings:
   all git calls go through `spawnSync('git', args[])`
   (`src/pipeline/runner.ts:70–80`) and violation paths are repo-relative
   porcelain paths consumed as single argv elements
   (`runner.ts:95–123`) — no traversal, no interpolation.
3. **Injection — PASS.** Env values reflected in resolver errors go through
   `JSON.stringify` (`cli.ts:111`, `cli.ts:130`), pinning test asserts no raw
   `\x1b` byte reaches the message (`tests/cli.test.ts:110–115`). Flag
   reflection is `String()` of a yargs-coerced number/`NaN`/number-array —
   never a raw argv string. All regexes in the diff (`^\d{1,5}$`,
   `TEST_PATH_RE`) are fixed, bounded, linear — no catastrophic backtracking.
   `extractJsonObject` (`pipeline/prompts.ts`) is a hand-written linear
   scanner, no regex, and its `JSON.parse` failures (including deep-nesting
   `RangeError`) are caught.
4. **Secrets and PII — PASS.** No credential material in the diff's source,
   fixtures (only the pre-existing fake `'secret'` basic-auth fixture), or
   new log lines. The extended banner adds only the validated integer
   (`cli.ts:203–206`); the "no password was provided" hint stays verbatim
   and never echoes a value; the resolver reflects only the timeout env var.
   The unredacted `baseUrl` print is pre-existing and an explicit design
   non-goal — not re-litigated here.
5. **New dependencies — PASS (none).** `package.json` adds only the `engines`
   block; `package-lock.json`'s diff is exactly the mirrored `engines` entry
   on the root package — no new or changed dependency versions, so no
   advisory surface to check.
6. **Filesystem writes — PASS with one minor** (finding S2 below). The CLI
   writes no files. `enforceOwnership` deliberately deletes only paths the
   stage was not allowed to touch, but its cleanup is destructive and not
   robust to partial failure.
7. **Clean rejection — PASS with one minor** (finding S1 below). The new
   rejection paths are side-effect-free and single-line at the resolver
   level — pinned by `tests/cli.test.ts:215–232` (no `createClient` call, no
   stdout). The module-level print still carries a stack trace, but that
   print is pre-existing and design-accepted (A13).

## Findings

- **[minor] S1 — `tools/orchestration/src/cli.ts:224–227`**: the new
  validation failures (resolver `Error` at `cli.ts:109/117/128`, runner
  `TypeError` at `opencode-runner.ts:242`) exit through the pre-existing
  `main().catch((error) => console.error(error))`, which prints a full stack
  trace for a plain config error — checklist item 7's "no stack traces to
  end users". Pre-existing pattern that the design deliberately keeps (A13),
  so non-blocking, but the change set extends it to four new throw sites.
  *Fix:* in the catch, `console.error(error instanceof Error ? error.message : error)`
  (or branch on the config-error types), keeping exit code 1.
- **[minor] S2 — `tools/orchestration/src/pipeline/runner.ts:110–111`**:
  `enforceOwnership`'s cleanup of a violation that is not in `headBefore`
  runs `git rm -f -r` (failure swallowed by `gitOk`) and then
  `rmSync(resolve(repoDir, p), { force: true })` without `recursive: true`;
  if `git rm` failed and `p` is a directory, `rmSync` throws `EISDIR`
  (force only suppresses ENOENT), aborting enforcement with a partially
  reverted worktree/index. Destructive partial application of the runner's
  own ownership boundary. Low likelihood, out-of-scope code, not
  model-exploitable beyond state the stage already had.
  *Fix:* `rmSync(..., { force: true, recursive: true, maxRetries: 2 })` or
  fail the stage loudly on revert failure rather than crashing mid-loop.

## Verdict

**approve** — zero open blocker/major security findings. Both design-review
security-r1 majors (API-boundary fail-closed validation; env format gate +
`JSON.stringify`-escaped reflection) are implemented and pinned by tests;
all seven checklist items pass except the two minors above.

## FYI (no severity, out of specialty)

- The range lands pipeline-runner changes (`src/pipeline/*`, new agent
  config) that `02-proposal.md` explicitly declares a non-goal — ownership/
  scope question for the compatibility reviewer and the run record, not a
  security defect.
- `main()` reads the environment twice (once via `configFromEnv()` inside
  `parseArgs`, once for the resolver at `cli.ts:188`) — harmless but
  redundant.
- `cli.test.ts`'s `afterEach` clears `OPENCODE_REPLY_TIMEOUT_SECONDS` only
  after each `main()` test, so the 30 s-default case inherits an ambient
  value in a dirty shell — already recorded in `03-build-notes.md`.
