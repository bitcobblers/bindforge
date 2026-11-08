# Code review — security (round 3)

Range: `fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`
Design: `docs/designs/002-add-timeout-flag/02-proposal.md` (r4)
Checklist: `.opencode/skills/code-review/references/security.md` (security only)

## Scope reviewed

Re-reviewed from the tree at HEAD (shell is refused in this environment, so the
range was reconstructed from `03-build-notes.md` and the run record and every
finding was re-verified against file contents, not against the previous
round's text):

1. **The design's change set** — `src/cli.ts`, `src/opencode-runner.ts`,
   `package.json` (+ mirrored `engines` in `package-lock.json`), `README.md`,
   `tests/cli.test.ts`, `tests/opencode-runner-timeout.test.ts`.
2. **The pipeline group landed by the run's own record** (`src/pipeline/
   gates.ts`, `prompts.ts`, `runner.ts`, `tests/pipeline-*.test.ts`, and the new
   `.opencode/agents/codereview/code-compatibility.md`). The proposal names
   pipeline changes a non-goal (proposal:17, :23), but they sit inside the
   caller's range, so they are assessed under the security checklist. Scope
   attribution is an FYI, below.

`tests/opencode-runner.test.ts` is untouched — the "existing tests stay green"
anchor is intact.

## Round-2 findings: status

- **S1 (stack traces on new config errors) — still open.** `cli.ts:223–227`
  still does `console.error(error)`; re-raised below as R3-1.
- **S2 (`rmSync` non-recursive in `enforceOwnership`) — still open.**
  `runner.ts:111–112` is `git rm -f -r -- <p>` then `rmSync(resolve(repoDir, p),
  { force: true })` with no `recursive`; re-raised below as R3-2, now folded
  into a wider path-handling finding.

## Checklist results

1. **External input validated at the boundary — PASS.**
   `OPENCODE_REPLY_TIMEOUT_SECONDS` is a raw untrusted string gated by
   `^\d{1,5}$` before any `Number()` call (`cli.ts:108`), precedence-
   independently (flag `5` + env `"30s"` still throws — `tests/cli.test.ts:119–
   123`; set-but-empty `""` is rejected — `tests/cli.test.ts:99–106`). The flag
   is range-checked as an integer in `1..3600` (`cli.ts:116`). The exported API
   boundary re-validates independently: `runPrompt` throws a `TypeError` before
   `createSession` for non-integer / `<1` / `> MAX_REPLY_TIMEOUT_MS`
   (`opencode-runner.ts:240–246`), pinned with zero `create`/`prompt`/`list`
   calls for `Infinity`, `NaN`, `0`, `-5`, `3_600_001`
   (`tests/opencode-runner-timeout.test.ts:140–153`). No `eval`/`Function`/code
   evaluation on any new path. The network path (`waitForReply` →
   `findCompletedReply` → `parseContent`) is unchanged by this diff and keeps
   its JSON.parse + zod validation of untrusted server replies.
2. **Path handling — PASS for the design's change set; FAIL for the pipeline
   group (R3-3, R3-4).** The CLI/runner timeout change performs no path or
   shell work. In the pipeline group, `enforceOwnership` routes git's *text*
   output back into destructive git/FS calls without NUL delimiting.
   `writeEditorialDiff`'s snapshot path is safe: the `bindforge-editorial-`
   prefix is concatenated into the same path component as the slug, so a
   `../../` slug can't escape `tmpdir()` (`runner.ts:840`).
3. **Injection — PASS with two nits.** Env reflection is `JSON.stringify`
   escaped (`cli.ts:111`, `:130`) and pinned not to carry a raw `\x1b`
   (`tests/cli.test.ts:110–115`). Flag reflection is `String(flagValue)`
   (`cli.ts:119`) — I verified the load-bearing assumption A8 (unverified in the
   proposal) against upstream `yargs-parser`: for a key declared `type:
   'number'`, `maybeCoerceNumber` unconditionally applies `Number(value)`, so
   argv can only reach the resolver as a number, `NaN`, or (on a repeated flag)
   an array of numbers — never a raw argv string. All new regexes are fixed and
   bounded (`^\d{1,5}$` — JS `$` does not match before a trailing newline, so
   `"45\n"` is correctly rejected). Nits R3-6/R3-7 below.
4. **Secrets and PII — PASS.** Nothing credential-shaped in the diff's source
   or fixtures (only the pre-existing fake `'secret'` basic-auth fixture in the
   untouched suite — re-confirmed by grep). The extended banner adds only the
   validated integer (`cli.ts:203–206`); the "no password was provided" hint
   stays verbatim and never echoes a value (`cli.ts:207–209`); resolver errors
   reflect only the timeout env var.
5. **New dependencies — PASS (none).** `package.json` adds only the `engines`
   block (`:30–32`); the lockfile's root entry mirrors it (`package-lock.json:11–
   13`) and its dependency set matches `package.json` exactly — no new or
   bumped package, so no advisory surface to check.
6. **Filesystem writes — PASS with one minor (R3-5).** `saveState` writes the
   run checkpoint atomically (temp + rename, `state-store.ts:41–48`). The
   change set itself writes no files.
7. **Clean rejection — PASS with one minor (R3-1).** Config errors are
   side-effect free and single-line at the resolver level, pinned by
   `tests/cli.test.ts:215–232` (no `createClient` call, no stdout). The invalid
   `timeoutMs` path rejects as a rejected promise before any timer or request
   (`opencode-runner.ts:237–246`). The module-level print still carries a stack
   trace.

## Findings

- **[minor] R3-1 — `tools/orchestration/src/cli.ts:223–227`** (carried open
  from security-r2 S1): the four new throw sites (resolver `Error` at
  `cli.ts:109/117/128`, runner `TypeError` at `opencode-runner.ts:242`) all exit
  through `main().catch((error) => console.error(error))`, which prints a full
  stack trace for a plain config error — checklist item 7's "no stack traces to
  end users". The pattern is pre-existing and design-accepted (A13), so it stays
  non-blocking, but the change set extends it.
  *Fix:* `console.error(error instanceof Error ? error.message : error)` in the
  catch, keeping exit code 1.
- **[minor] R3-2 — `tools/orchestration/src/pipeline/runner.ts:111–112`**
  (carried open from security-r2 S2): `gitOk(['rm','-f','-r','--',p])` swallows
  its own failure, then `rmSync(resolve(repoDir, p), { force: true })` runs
  without `recursive: true`; `force` only suppresses `ENOENT`, so a directory
  whose `git rm` failed raises `EISDIR` and aborts the loop mid-revert —
  destructive partial application of the runner's own ownership boundary.
  *Fix:* `rmSync(..., { force: true, recursive: true, maxRetries: 2 })`, or fail
  the stage loudly when the revert command fails instead of crashing mid-loop.
- **[minor] R3-3 — `tools/orchestration/src/pipeline/runner.ts:83–113`**
  (checklist item 2, new code in range): `enforceOwnership` is documented as
  "the real boundary" against a model routing around agent-config permissions,
  but it parses git's human-readable output — `status --porcelain` sliced as
  `line.slice(3).trim()` (`:87`) and `diff --name-only` split on `'\n'`
  (`:100`) — and feeds those strings into `git cat-file -e <rev>:<p>`,
  `git restore -- <p>`, `git rm -f -r -- <p>` and `rmSync(resolve(repoDir, p))`.
  `core.quotepath` is a user-settable global that is never overridden here, so
  non-ASCII paths come back octal-escaped and quoted; rename entries arrive as
  `old -> new`; a path with leading/trailing whitespace is trimmed. The garbled
  string then matches nothing, so an ownership violation either silently escapes
  the revert (`force: true` hides the `rmSync` miss) or the destructive calls act
  on a wrong literal path.
  *Fix:* machine-readable git output — `git -c core.quotepath=false diff
  --name-only -z` and `status --porcelain -z`, split on `'\0'`, taking the new
  path for renames — and treat a failed revert as a hard stage failure.
- **[minor] R3-4 — `tools/orchestration/src/pipeline/runner.ts:505–506` with
  `:127–147`**: ownership enforcement runs only inside `if (outcome.ok)`. A
  stage that fails after writing files (steps-exhausted, invalid reply,
  timeout — all three happened in this very run, `.pipeline.json:974–1006`)
  leaves its out-of-ownership edits in the worktree unenforced; the next
  successful stage's `commitStage` then does `git add -A` (`:137`) and commits
  them under that stage's message. That is both a bypass of the boundary and an
  attribution laundering path for a constrained actor's edits.
  *Fix:* call `enforceOwnership` on the failure path too (using the same
  `headBefore` captured at `:353`), before recording the failure.
- **[minor] R3-5 — `tools/orchestration/src/pipeline/gates.ts:241–254`**:
  `stagePathAllowed` grants `implement`/`fix` every path except tests, which
  includes `.opencode/agents/**` — the permission definitions the runner treats
  as advisory precisely because "a model can route around agent-config
  permissions" — and `src/pipeline/**`, i.e. the enforcement code itself. A
  stage can therefore widen its own (and later stages') permissions and edit the
  guard that would catch it; the run record's own pipeline-group commits touch
  `.opencode/agents/codereview/code-compatibility.md`.
  *Fix:* deny both patterns for model-run stages
  (`/^\.(?:opencode)\/agents\//` and `/^tools\/orchestration\/src\/pipeline\//`
  → not writable by `implement`/`fix`), and have operator self-repair land
  outside the model-run path.
- **[minor] R3-6 — `tools/orchestration/src/pipeline/runner.ts:856–859`**
  (checklist item 6): `saveStateSync` writes `.pipeline.json` with a plain
  `writeFileSync`, bypassing the atomic temp+rename contract the async
  `saveState` documents in `state-store.ts:36–48`. A crash mid-write on the
  escalation path truncates the run's only progress checkpoint — partial write
  corrupting state is exactly what item 6 asks to prevent.
  *Fix:* route the sync path through the same tmp-file + rename sequence (or
  make the escalation path `await saveState`).
- **[nit] R3-7 — `tools/orchestration/src/cli.ts:111` and `:130`**: the
  rejection messages are echoed through `JSON.stringify`, which escapes C0
  controls (including `\x1b`), quotes and backslashes but leaves DEL, C1
  controls (U+0080–U+009F; 8-bit CSI is a live escape introducer in UTF-8
  terminals) and printable non-ASCII raw — and this is the path that echoes an
  arbitrary *rejected* value. Security-r1's rule is met because the `^\d{1,5}$`
  gate is the load-bearing defense; recorded so a future gate relaxation cannot
  lean on the escaper alone.
  *Fix:* reflect a shape instead of the bytes (`<rejected: 4 chars, not plain
  digits>`) or additionally escape `[\u007f-\u009f]`; keep the
  `not.toContain('\x1b')` assertions.
- **[nit] R3-8 — `tools/orchestration/src/cli.ts:119`,
  `tools/orchestration/src/opencode-runner.ts:243`**: both reflect an
  *unvalidated* value with `String(...)` even though the design's rule is that
  any reflected value is embedded via `JSON.stringify` (proposal:84). Safe today
  (see checklist 3: yargs coerces declared-number options through `Number()`,
  and the runner's own value is `options.timeoutMs ?? DEFAULT`), so this is
  rule-alignment plus defence-in-depth for non-CLI callers of the exported
  resolver, whose `timeoutMs` is `unknown` at runtime whatever TS says.
  *Fix:* `JSON.stringify(flagValue)` / `JSON.stringify(timeoutMs)` in the two
  messages (the runner message would render `"Infinity"` rather than `Infinity`
  — check the pinned substrings first, `tests/opencode-runner-timeout.test.ts`
  asserts the type and zero calls, not the message text).
- **[nit] R3-9 — `.opencode/agents/codereview/code-compatibility.md:20–31`**
  (new file in range): the agent's shell allow-list is prefix-wildcarded
  (`git diff*`, `git log*`, `npm run check*`, `npm test*`). Whether that is a
  narrow grant depends on whether the harness splits compound commands before
  matching; if it matches the whole line, `git diff …; <anything>` or
  `npm test && curl …` satisfies the pattern and the deny-all shell rule is
  bypassed. Worth one line of confirmation, since this file is the permission
  boundary for every future `code-compatibility` reviewer run.
  *Fix:* tighten to whole-command patterns (no trailing `*` past a complete
  command), or record in the file that the matcher is per-sub-command.

## Verdict

**approve** — zero open blocker/major security findings. Both design-review
security majors remain closed in code and pinned by tests: API-boundary
fail-closed validation (`opencode-runner.ts:240–246`,
`tests/opencode-runner-timeout.test.ts:140–153`) and env format gate +
escaped reflection (`cli.ts:108–113`, `tests/cli.test.ts:89–115`). The 3 600 s
cap bounds the poll burst at both boundaries (`3_600_000 / 250` ≈ 14 400 polls).
All seven checklist items were evaluable this round; the six minors are the
carried two plus four in the pipeline group, which is outside the design's
change set.

## FYI (no severity, out of specialty)

- The range still contains the pipeline-group commits (`src/pipeline/*`, the new
  agent config) that `02-proposal.md` names as a non-goal; correctness-r1/r2/r3
  carry the same range-hygiene finding with the two offered remedies. Re-pinning
  `baseRef` (`.pipeline.json:1012`) to `891567b` would make every reviewer's
  scope match the design.
- `main()` reads the environment twice (`cli.ts:144` inside `parseArgs`, `:188`
  for the resolver) — redundant, harmless; carried by performance-r2.
- The pre-existing unredacted `baseUrl` in the startup banner (`cli.ts:204`)
  stays as designed (explicit non-goal, proposal:21); not re-litigated here.
- `--timeout` with no value yields yargs' number-type placeholder `undefined`
  (`defaultForType(NUMBER)`), so the resolver silently falls back to env/30 s
  instead of reporting the malformed option. UX, not security: the fallback is
  always inside the validated range.
