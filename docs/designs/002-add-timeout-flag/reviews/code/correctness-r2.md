# correctness review r2

verdict: approve

Scope: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD` (base pin
recorded at `.pipeline.json:742`), judged against `02-proposal.md` (r4) and
`03-build-notes.md` (including its Round 2 section).
Checklist: `references/correctness.md` (code correctness — faithful to the
design). Previous round: r1 (`reviews/code/correctness-r1.md` — 1 major,
2 minors, 2 nits; the same finding set is recorded in
`.pipeline.json:552-593`). The r2 review attempt itself failed its reply gate
(`.pipeline.json:611-617`, `:732-738`), so this report replaces it and re-does
the r2 checklist from the tree.

Environment note: this review environment rejects **every** shell command —
`git diff`, `git log`, `npm run check`, `npx vitest` (`Permission denied:
shell`) — so the diff could not be re-derived and the gate could not be
re-executed. The review is therefore a line-by-line static trace of the HEAD
tree against the proposal plus the checked-in suites, cross-checked against
artifacts that *are* in the tree (`.pipeline.json` run log and gate log).
Findings below are graded accordingly; the r1 major that rested on the missing
gate evidence is re-graded, see checklist #7 and finding 1.

## Checklist walk

1. **Components/interfaces exist, are wired in, names/signatures match the
   proposal.** Re-verified independently in the current tree; every
   proposal-named piece is present and literal:
   `DEFAULT_REPLY_TIMEOUT_MS = 30_000` (`src/opencode-runner.ts:28`) and
   `MAX_REPLY_TIMEOUT_MS = 3_600_000` (`:31`) exported, `POLL_INTERVAL_MS = 250`
   untouched (`:25`), and no `POLL_TIMEOUT_MS` reference remains anywhere in the
   tree (grep); `PromptOptions.timeoutMs?: number` with the full runner-side
   contract in its docblock (`:123-137`, integer `1..MAX_REPLY_TIMEOUT_MS`,
   enforced by `runPrompt`, sub-250 ms legal with the ≤ one-interval overshoot);
   `runPrompt` resolves `options.timeoutMs ?? DEFAULT_REPLY_TIMEOUT_MS` (`:240`)
   and throws a `TypeError` naming the value *before* `createSession` (`:237-248`);
   `waitForReply(client, sessionID, timeoutMs)` keeps wall-clock deadline
   arithmetic (`Date.now() + timeoutMs`, `:290`) and its `null`-on-expiry
   contract (`:285-307`); `export class ReplyTimeoutError extends Error` in the
   Errors section (`:346-354`) keeps `'No response received from OpenCode'` as
   an exact prefix with `<n> = timeoutMs / 1000` and no data properties
   (`this.name` only — declared deviation 3); `runPrompt`'s throw site is now
   `throw new ReplyTimeoutError(timeoutMs)` (`:253-255`).
   `src/cli.ts`: module-private `EnvValues` (`:39-45`) exactly as specified,
   `CliConfig` with exactly one validated `timeoutSeconds: number` (`:32`),
   `CliArgs.timeout?: number` (`:55`), `configFromEnv(): EnvValues` reading
   `OPENCODE_REPLY_TIMEOUT_SECONDS` raw (`:76-84`), exported
   `resolveReplyTimeoutSeconds(flagValue: number | undefined,
   envValue: string | undefined): number` (`:101-104`) with the `^\d{1,5}$` gate
   before any `Number()` (`:108`), precedence flag → env →
   `DEFAULT_REPLY_TIMEOUT_MS / 1000` (`:115-136`), `JSON.stringify`-escaped
   reflection of env values (`:111`, `:130`); yargs
   `.option('timeout', { type: 'number', describe: … })` with **no** `default:`
   (`:167-174`); `main()` runs the resolver immediately after `parseArgs()`
   (`:182-189`) and before `createClient()` (`:197`) and before any
   `console.log` (`:203`), assigns `timeoutSeconds` inside the `CliConfig`
   literal (`:190-196`), extends the startup line with only
   ` waiting up to <n> s for the reply` (`:203-206`), keeps the "no password"
   hint verbatim (`:207-209`), and calls
   `runPrompt(client, { agent: config.agent, timeoutMs: config.timeoutSeconds * 1000 }, …)`
   (`:211-216`). `package.json:30-32` carries `"engines": { "node": ">=22.12.0" }`
   (mirrored at `package-lock.json:11-13`); README adds the `--timeout` row
   (`README.md:22`) and the Behaviour #3 rewording (`:35-37`). The README:29
   session-title drift is untouched (explicit non-goal). ✓
2. **Assumption register honoured; nothing silently flipped.** A1 (250 ms
   interval intact), A2/A5 (old message kept as exact prefix; the existing
   default-timeout test `tests/opencode-runner.test.ts:243-257` is unchanged and
   still matches by substring at `:253`), A3 (env reads stay in
   `configFromEnv()`), A4 (`timeoutMs` optional — all `runPrompt` call sites in
   `opencode-runner.test.ts` still pass an `options` object), A7/A8
   (`Number.isInteger` catches yargs' non-finite junk, `cli.ts:116`; gate before
   `Number()`), A9 (deadline is wall-clock arithmetic; only the fixed 250 ms
   `sleep()` uses a timer — `opencode-runner.ts:290,302,330-332`), A10
   (milliseconds internally, seconds at the CLI edge), A11 (`src/pipeline/`
   never calls `runPrompt`; grep of `runPrompt`/`waitForReply`/`timeoutMs` hits
   only `src/cli.ts`, `src/opencode-runner.ts` and the three suites), A13
   (module-level `console.error` + `process.exit(1)` under the `isMain` guard,
   `cli.ts:221-228`), A14/A16 (`1..3_600` s enforced at both boundaries), A19
   (the `vi.mock(..., importOriginal)` route landed and is compatible with the
   CLI's value imports, `tests/cli.test.ts:29-36`), A21 (`engines.node` added).
   No row was silently changed. ✓
3. **Deviations declared.** All five build-notes deviations are declared and
   check out against the tree: deviation 1 (gate not run *in the implement
   environment* — now superseded by the pipeline's own gate run, see finding 1),
   deviation 2 (precedence-dependent env range check — see finding 2),
   deviation 3 (`this.name` only, no data property), deviation 4 (message
   wording additions; messages single-line, pinned by
   `tests/cli.test.ts:229`), deviation 5 (A19 route (a) spike outcome, matching
   the mock factory in `tests/cli.test.ts:11-36`). The `package-lock.json`
   `engines` mirror beyond the proposal's "one `package.json` line" is declared
   at `03-build-notes.md:37-40`. No undeclared deviation found in the design's
   change set. ✓ (coverage gap on the range itself: finding 3)
4. **Scope creep.** The design's change set (`opencode-runner.ts`, `cli.ts`,
   `package.json` + lock mirror, `README.md`, the two new suites
   `tests/cli.test.ts` / `tests/opencode-runner-timeout.test.ts`) is exactly what
   the proposal asks for; no behaviour outside the proposal was added on the
   timeout path (no poll-interval knob, no exit-code 124, no pipeline change, no
   `clientFactory` seam — the recorded fallback was correctly not used). The
   out-of-scope `src/pipeline/` commits r1 identified are still inside the
   reviewed base (range hygiene, finding 3). ✓ (with finding 3)
5. **Build-notes claims checked against the code.** True by trace: the two-check
   contract for `timeoutMs = 100` (loop tests the deadline only *after* each
   reply check, `opencode-runner.ts:292-304` → exactly two `list` calls, then
   `within 0.1 s`) matches `tests/opencode-runner-timeout.test.ts:159-175`; the
   4 s and 30 s fake-timer cases match (`:67-85`, `:109-136`) — the pattern is
   the same one the pre-change test at `opencode-runner.test.ts:243-257` already
   relied on, which is independent evidence that vitest fakes `Date` here; the
   fail-closed set `Infinity`/`NaN`/`0`/`-5`/`3_600_001` all hit the `TypeError`
   before `createSession` so `create`/`prompt`/`list` stay at zero calls
   (`:140-154` vs source `:240-246`); empty `OPENCODE_REPLY_TIMEOUT_SECONDS=` is
   rejected by the gate before any `Number()` (`cli.ts:108` vs
   `cli.test.ts:99-106`); `1`/`3600` accepted and `3601` a range error
   (`cli.test.ts:127-140`); resolver-before-`createClient` and
   `timeoutMs = seconds * 1000` match `cli.test.ts:172-232`; the README table is
   prettier-stable as measured (all seven rows 99 chars, separator column widths
   12/32/45 equal the max cell widths of their columns). Two claims are
   inaccurate: finding 1 (gate claim, now stale) and finding 4 (the env-cleanup
   observation names the wrong test).
6. **Error paths from the design exist and fail closed.** ✓ Invalid
   `timeoutMs` at the exported API boundary → `TypeError` before any request
   (no unbounded poll); malformed env → throws in the resolver before
   `createClient()` and before any output, so a config error is exactly one line
   and side-effect free (asserted at `cli.test.ts:215-232`); malformed env is
   not masked by a valid flag (`cli.ts:108-113`); every failure still surfaces as
   `console.error(error)` + exit 1 (`cli.ts:221-228`, A13).
7. **Previous round's blocker/major addressed?** r1 raised no blocker. Its one
   **major** (finding 1: "`npm run check` was never executed, so the change is
   unverified") is **closed on the evidence now in the tree**, not by a code
   change: `.pipeline.json:712-717` records gate `testsGreen`, round 0,
   `proceed: true`, reason `"npm run check green"`, run by the pipeline runner
   after the implement stage landed the change set
   (`.pipeline.json:539-550`) and before this code-review round; no source file
   has changed since (no implement stage is recorded after it, and the tree still
   matches every anchor r1 cited). Since that gate is lint + format check +
   tests, Goal 4 ("existing tests pass unchanged") and Test strategy #12 are now
   evidenced, and my own trace found no source/test mismatch to contradict it.
   The residue — the build notes still assert the gate has *not* run and must be
   run by the caller — is a documentation inaccuracy, not an open major: finding 1
   (minor). r1's minors 2 and 3 and nits 5 and 6 were not acted on (no source or
   build-notes edits landed after r1) and are carried below; no new blocker or
   major surfaced in this re-check.

## Findings

1. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` and
   `:129-136` — the notes' gate claim is now stale/false: deviation 1 and the
   Round-2 paragraph both say `npm run check` was never executed and "must still
   be run by the caller", but the pipeline's own run log records
   `testsGreen` / `"npm run check green"` at round 0
   (`.pipeline.json:712-717`), i.e. the design's Test strategy #12 / Rollout #1
   gate was executed by the runner after the implementation landed. Checklist #5
   asks that build-notes claims be true, and this one no longer is; it also
   caused the previous r2 attempt to carry a major that the tree already
   refutes. (Caveat I could not resolve: the log line carries no counts and its
   timestamp is only ~3 s after the implement stage entry, so the summary itself
   is not verifiable here.) Fix: rewrite deviation 1 and the Round-2 paragraph to
   cite the gate-log entry (round, timestamp, `npm run check` = lint + format
   check + tests → green) and paste the summary block (test counts per suite,
   lint/format status, Node version) into `03-build-notes.md`; if the runner's
   log turns out to be wrong, re-run `npm run check` in `tools/orchestration`
   before merge.
2. **[minor]** `tools/orchestration/src/cli.ts:115-134` (docblock `:94-99`) —
   carried open from r1 finding 2: the env **range** check still sits inside the
   env branch, behind the `flagValue !== undefined` return, so `--timeout 5` with
   `OPENCODE_REPLY_TIMEOUT_SECONDS=3601` resolves silently to `5` and the broken
   environment is never reported. That is the letter of the proposal (only the
   format gate is pinned as precedence-independent) and it is declared as
   build-notes deviation 2 — hence minor rather than a deviation finding — but it
   contradicts the docblock rationale written directly above it ("a broken
   environment is a configuration error even when a valid flag overrides it") and
   the Risks row promising an env typo hard-fails the run, and no test pins the
   precedence-dependent behaviour. Fix: validate the env value fully (format gate
   **and** `1..maxSeconds` range) before the `flagValue !== undefined` branch and
   then apply precedence; or keep the behaviour, fix the docblock wording, and pin
   it with `expect(resolveReplyTimeoutSeconds(5, '3601')).toBe(5)`.
3. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:46-48` —
   carried open from r1 finding 3: the coverage claim ("No test files were
   edited; the three existing suites …") still does not describe the whole
   reviewed base, which also contains `src/pipeline/gates.ts`,
   `src/pipeline/prompts.ts`, `src/pipeline/runner.ts`,
   `tests/pipeline-gates.test.ts` and `tests/pipeline-prompts.test.ts` — a
   subsystem the proposal names as an explicit non-goal ("Any change to the
   pipeline (`src/pipeline/`)", proposal:17). Neither of r1's fixes (a build-notes
   line recording those commits, or re-pinning the base to `891567b`) was applied,
   and this environment refused `git log`, so r1's provenance conclusion (operator
   infrastructure predating the run's docs commits, hence range hygiene rather
   than this run's output) is accepted from r1 rather than re-audited. None of
   those files touches the timeout path (grep: no `timeoutMs`/
   `OPENCODE_REPLY_TIMEOUT_SECONDS`/`ReplyTimeoutError` reference in
   `src/pipeline/`). Fix: add one build-notes line naming the out-of-scope commits
   in the range, or re-pin `baseRef` so the review diff is only the design's
   change set. If they turn out to be this run's output, this escalates to a
   major undeclared non-goal violation.
4. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:122-127` —
   the Round-2 observation about the checked-in `cli.test.ts` is stated against
   the wrong test. `afterEach` deletes `OPENCODE_REPLY_TIMEOUT_SECONDS` after
   **every** `main()` test (`tests/cli.test.ts:162-168`), so by the time the "30 s
   default" case (`:204-211`) runs, an ambient value has already been removed by
   the earlier `afterEach` runs. The real exposure is the **first** `main()` test
   (`:172-179`), which runs before any `afterEach`: an ambient well-formed
   out-of-range value is masked there by `--timeout 2`, while an ambient
   malformed value throws through the precedence-independent gate and fails that
   test. Checklist #5: this is a build-notes claim that does not match the tree.
   Fix: name the first `main()` test (or move the env clear into `beforeEach`,
   which would make the note moot).
5. **[nit]** `tools/orchestration/src/cli.ts:110-111` — carried open from r1
   finding 4: the format-gate message states the accepted range ("must be an
   integer number of seconds in 1..3600 (plain digits only)") although the
   failure it reports is a *format* violation, blurring the deliberate
   format/range split (proposal:82); a user with
   `OPENCODE_REPLY_TIMEOUT_SECONDS=30s` is told about a range they never
   violated. Fix: keep the format error format-only (e.g.
   `$OPENCODE_REPLY_TIMEOUT_SECONDS must be plain decimal digits (integer
   seconds), got …`) and leave the range wording to `cli.ts:129-130`.
6. **[nit]** `tools/orchestration/src/cli.ts:144,188` — carried open from r1
   finding 5: `configFromEnv()` is called twice per run (once inside
   `parseArgs()`, again for `replyTimeoutSeconds`), so the resolver validates a
   second, independent env read rather than the bag yargs used for the other
   defaults — at odds with the "env reads centralized" intent (A3). Fix: have
   `parseArgs(argv)` take or return the `EnvValues` bag and pass that same
   snapshot into `resolveReplyTimeoutSeconds`.

## FYI (out of specialty, no severity)

Three new lines exceed `.prettierrc`'s `printWidth: 100`
(`tests/opencode-runner-timeout.test.ts:141` (102), `:159` (117),
`tests/cli.test.ts:215` (103); the two remaining >100 lines,
`src/cli.ts:170` and `tests/cli.test.ts:56`, are unbreakable string arguments and
are prettier-stable). The gate log reports `format:check` green, so Prettier
evidently accepts them; a future reviewer without shell access may want those
heads wrapped so the formatting claim does not depend on Prettier's trailing-
closure layout rule. Also: `tests/cli.test.ts:114`
(`expect(message).not.toContain('[2J\n')`) is a weak assertion next to the
`:113` control-character check, and `stubClient()` is duplicated across
`opencode-runner.test.ts` and `opencode-runner-timeout.test.ts` — both test-
quality observations, not design-fidelity ones. Finally, this report replaces the
earlier r2 file at the same path, whose run failed its reply gate
(`.pipeline.json:611-617`, `:732-738`); that version claimed "no executed-gate
evidence exists anywhere on the branch", which `.pipeline.json:712-717` refutes.
