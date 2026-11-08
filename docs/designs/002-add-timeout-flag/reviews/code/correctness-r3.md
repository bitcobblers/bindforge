# Code review — correctness (round 3)

Verdict: **approve** (zero open blocker/major findings; 4 minors, 2 nits, all
carried or sharpened from earlier rounds)

- Range: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`
  (`baseRef` = `fdfd2bc…`, `docs/designs/002-add-timeout-flag/.pipeline.json:914`)
- Design: `02-proposal.md` (r4); notes: `03-build-notes.md` (incl. its
  "Round 2" section)
- Checklist: `.opencode/skills/code-review/references/correctness.md` (only)
- Previous round: `reviews/code/correctness-r2.md` — verdict approve, no
  blocker/major; findings 1–6 (4 minor, 2 nit). Before that,
  `reviews/code/correctness-r1.md` — 1 major, 2 minor, 2 nit.

Environment note (bears on checklist #5/#7): this review environment rejects
**every** shell command — `git diff`, `git log`, `npm run check` all return
`Permission denied: shell` — so the diff could not be re-derived and the gate
could not be re-executed. The review is a line-by-line trace of the HEAD tree
against the proposal and the checked-in suites, plus the in-tree run record
(`.pipeline.json`) and the r1/r2 reports of rounds that *did* have `git` access
(`reviews/code/testing-r2.md:86-104` supplies the commit/timestamp evidence used
in finding 1). Findings that could not be evaluated by execution are recorded as
findings, not passes.

## Checklist walk

1. **Every component/interface the proposal names exists, is wired in, and
   matches its name and signature.** Re-verified against the HEAD tree; no drift
   from what r2 anchored:
   `DEFAULT_REPLY_TIMEOUT_MS = 30_000` (`src/opencode-runner.ts:28`) and
   `MAX_REPLY_TIMEOUT_MS = 3_600_000` (`:31`) exported, `POLL_INTERVAL_MS = 250`
   untouched (`:25`), and `POLL_TIMEOUT_MS` has zero hits tree-wide (grep over
   `tools/`); `PromptOptions.timeoutMs?: number` with the full runner-side
   contract in its docblock (`:123-137` — integer `1..MAX_REPLY_TIMEOUT_MS`,
   enforced by `runPrompt`, sub-250 ms legal, ≤ one-interval overshoot);
   `runPrompt` resolves `options.timeoutMs ?? DEFAULT_REPLY_TIMEOUT_MS` (`:240`)
   and throws a `TypeError` naming the value *before* `createSession` (`:237-246`);
   `waitForReply(client, sessionID, timeoutMs)` keeps the wall-clock deadline
   (`Date.now() + timeoutMs`, `:290`) and the `null`-on-expiry contract
   (`:285-307`); throw site is `throw new ReplyTimeoutError(timeoutMs)` (`:254`);
   `export class ReplyTimeoutError extends Error` in the Errors section
   (`:346-354`) with the proposal's exact message shape, `timeoutMs / 1000`
   formatting and no data properties (only `this.name`, declared deviation 3).
   `src/cli.ts`: module-private `CliConfig` with exactly one validated
   `timeoutSeconds: number` (`:27-33`) and `EnvValues` raw env bag (`:39-45`),
   `CliArgs.timeout?: number` (`:55`), `configFromEnv(): EnvValues` reading
   `OPENCODE_REPLY_TIMEOUT_SECONDS` raw (`:76-84`), exported
   `resolveReplyTimeoutSeconds(flagValue, envValue)` (`:101-137`) with the
   `^\d{1,5}$` gate before any `Number()` (`:108`), precedence flag → env →
   `DEFAULT_REPLY_TIMEOUT_MS / 1000` (`:115-136`), `JSON.stringify`-escaped
   reflection of env values (`:111`, `:130`); yargs `.option('timeout', {
   type: 'number', describe: '…'})` with **no** `default:` (`:167-174`, describe
   text byte-matches proposal:80); `main()` resolves immediately after
   `parseArgs()` (`:182-189`), before `createClient()` (`:197`) and before any
   `console.log` (`:203`); banner extended with the resolved duration only
   (`:203-206`); "no password" hint verbatim (`:207-209`);
   `runPrompt(client, { agent, timeoutMs: timeoutSeconds * 1000 }, …)`
   (`:211-216`). `package.json:30-32` carries `"engines": { "node": ">=22.12.0" }`
   with the mirror at `package-lock.json:11-13`; README adds the `--timeout` row
   (`README.md:22`) and the Behaviour #3 rewording (`:35-37`). The README:29
   session-title drift is untouched, as the proposal directs. ✓

2. **Assumption register honoured; no row silently flipped.** A1 (250 ms
   interval intact), A2/A5 (old string kept as an exact prefix; the pinned
   default-timeout test `tests/opencode-runner.test.ts:243-257` is present and
   unedited, its substring assertion at `:253` still matches, and that file has
   zero `timeout`/`timeoutMs`/`ReplyTimeoutError` hits — grep), A3 (env reads
   still in `configFromEnv()`), A4 (`timeoutMs` optional; every `runPrompt` call
   site still passes an `options` object), A7/A8 (`Number.isInteger` catches
   yargs' non-finite junk, `cli.ts:116`; gate precedes `Number()` at `:108`), A9
   (deadline is wall-clock arithmetic; only the fixed 250 ms `sleep()` uses a
   timer — `opencode-runner.ts:290,302,330-332`), A10 (ms internally, s at the
   CLI edge), A11 (zero `runPrompt`/`waitForReply`/`timeoutMs`/
   `ReplyTimeoutError`/`OPENCODE_REPLY_TIMEOUT` hits anywhere under
   `src/pipeline/` — grep, re-verified after the mid-run `src/pipeline/runner.ts`
   commit), A13 (module-level `console.error` + `process.exit(1)` under the
   `isMain` guard, `cli.ts:221-228`), A14/A16 (`1..3_600` s enforced at both
   boundaries), A19 (the `vi.mock(…, importOriginal)` route landed,
   `tests/cli.test.ts:29-36`), A21 (`engines.node` declared). ✓

3. **Deviations from the proposal appear in `03-build-notes.md`.** All five
   declared deviations check out against the tree (1: gate not run in the
   implement environment — accuracy issue, finding 1; 2: precedence-dependent env
   range check — finding 3; 3: `this.name`; 4: message wording, messages stay
   single-line, pinned at `tests/cli.test.ts:229`; 5: A19 spike outcome, matching
   the mock factory). The `package-lock.json` `engines` mirror beyond the
   proposal's "one `package.json` line" is declared (`03-build-notes.md:37-40`)
   and verified. What is *not* declared is the rest of the reviewed range —
   the out-of-design `src/pipeline/*` and `.opencode/agents/…` commits
   (`03-build-notes.md:46-48` still claims coverage of only the three suites):
   finding 2. No undeclared deviation was found **inside the design's change
   set**. ✓ (coverage gap: finding 2)

4. **Scope creep.** Inside the design's change set the diff adds nothing the
   proposal did not ask for: no poll-interval knob, no exit code 124, no
   `clientFactory` seam (the recorded fallback was correctly not taken), no
   pipeline change, no CI config. Behaviour the proposal implies but does not
   spell out — rejecting a set-but-empty `OPENCODE_REPLY_TIMEOUT_SECONDS=` —
   follows directly from the mandated `^\d{1,5}$` gate and is pinned by a test
   (`tests/cli.test.ts:99-106`); not creep. The out-of-design content in the
   range is the pipeline/agent-config group (finding 2, graded minor on the
   provenance ground established by r1 and accepted by r2: runner/operator
   self-repair, not this run's implement output — the run's own commits are the
   test commit and `0d7762a`/`869e633`). ✓ (with finding 2)

5. **`03-build-notes.md` claims are true.** True by trace: the two-check
   contract for `timeoutMs = 100` (`opencode-runner.ts:292-304` → exactly two
   `list` calls, then `within 0.1 s`) matches `tests/opencode-runner-timeout.test.ts:159-175`;
   the fail-closed set (`Infinity`, `NaN`, `0`, `-5`, `3_600_001`) hits the
   `TypeError` before `createSession` so the `create`/`prompt`/`list` spies stay
   at zero calls (`:140-154` vs source `:240-246`); the empty-env, `1`/`3600`,
   `3601`, precedence and resolver-before-`createClient` claims all trace to the
   shipped source; the README prettier claim is now verified directly — the
   options table's seven rows are each exactly 99 characters and no README line
   reaches 100 (`^.{99}$` → 7 matches, `^.{100,}$` → none); the
   `package-lock.json` mirror claim is true. Three claims are false or stale:
   the gate claim (finding 1), the coverage claim (finding 2), and the
   env-cleanup observation, which still names the wrong test (finding 4).

6. **Error paths from the design exist and fail closed.** ✓ Invalid `timeoutMs`
   at the exported API boundary → `TypeError` before any HTTP request
   (no unbounded 250 ms poll loop; `opencode-runner.ts:237-246`,
   `tests/opencode-runner-timeout.test.ts:140-154`); malformed env → throws in
   the resolver before `createClient()` and before any output, so a config error
   is one line and side-effect free (`cli.ts:182-189`,
   `tests/cli.test.ts:215-232`); a malformed env is not masked by a valid flag
   (`cli.ts:108-113`); expiry surfaces as a distinct type whose message keeps
   the old string as an exact prefix (`:346-354`); every failure still exits via
   `console.error(error)` + exit 1 (`cli.ts:221-228`, A13). The one asymmetry —
   a *well-formed but out-of-range* env value under a valid flag is never
   reported — is finding 3.

7. **Previous round's blocker/major addressed?** correctness-r2 raised no
   blocker and no major, so there is nothing its round mandated fixing, and no
   source, test or build-notes file has changed since it reported (the last
   recorded stages are the round-2 code reviews, `.pipeline.json:742-775`; the
   tree still matches every anchor r2 cited). All six r2 findings are therefore
   still open and are carried below. r1's single **major** (gate never executed)
   stays closed only for the commit it was run against: r2's closure argument
   "no source changes since the gate" is refuted by `reviews/code/testing-r2.md:91-95`
   (gate `at: 1794118188887` vs `d7be29f` at `1794118457`,
   `src/pipeline/runner.ts` +15/−1), so the gate no longer certifies HEAD —
   finding 1, still graded minor because the post-gate delta touches no timeout
   code and the design's own change set is what was gated. No new blocker or
   major surfaced in this re-check.

## Findings

1. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` and
   `:129-136` — carried open from r2 finding 1, sharpened: the notes still say
   `npm run check` "could not be executed" and "must still be run by the caller",
   and r2's rebuttal of that claim ("the runner's gate ran after the implement
   stage … with no source changes since") is itself wrong. The run record's only
   green entry (`.pipeline.json:870-875`, `gate: testsGreen`, `reason: "npm run
   check green"`, `at: 1794118188887`) predates `d7be29f` (`1794118457`,
   `tools/orchestration/src/pipeline/runner.ts` +15/−1 plus the new
   `.opencode/agents/codereview/code-compatibility.md`, present in the tree), and
   no later gate entry exists. So HEAD — the tree the caller asked me to review —
   has no in-tree execution evidence at all behind Test strategy #12 / Rollout
   #1, and I could not run the gate here (checklist item I could not evaluate, so
   it counts as a finding). Fix: re-run `npm run check` in `tools/orchestration`
   at HEAD and paste the summary block (OS, `node -v`, per-suite test counts,
   lint/format status) into `03-build-notes.md`, then rewrite deviation 1 and the
   Round-2 paragraph to cite that run. If the re-run is red, this escalates to a
   major: Goal 4 ("existing tests pass unchanged") would be unevidenced again.

2. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:46-48` —
   carried open from r1 finding 3 / r2 finding 3, unchanged: the coverage claim
   ("No test files were edited; the three existing suites …") still does not
   describe the reviewed range, which also contains `src/pipeline/gates.ts`,
   `prompts.ts`, `runner.ts`, `tests/pipeline-gates.test.ts`,
   `tests/pipeline-prompts.test.ts` and the new
   `.opencode/agents/codereview/code-compatibility.md` from `f0a2e16`, `891567b`
   and `d7be29f`. The proposal names both locations as explicit non-goals
   (proposal:17 "Any change to the pipeline (`src/pipeline/`)", proposal:23 "Any
   change outside `tools/orchestration`"). Attribution stays with the
   operator/runner self-repair group rather than this run's implement stage —
   r1's provenance reading, re-confirmed by the stage log (this run's only code
   stages are `test-authoring` and `implement`, `.pipeline.json:516-550`) — so
   this remains range hygiene, not undeclared scope creep, and it is not
   re-auditable here (no shell). None of those files touches the timeout path
   (grep of `src/pipeline/` for `runPrompt`/`timeoutMs`/`OPENCODE_REPLY_TIMEOUT`:
   zero hits), so nothing in the design's behaviour depends on them. Fix: add
   one build-notes line naming the out-of-scope commits inside the range, or
   re-pin `baseRef` (`.pipeline.json:914`) to `891567b` so the review diff is
   only the design's change set. If the attribution turns out to be this run's
   output, r1's stated escalation applies: major undeclared non-goal violation.

3. **[minor]** `tools/orchestration/src/cli.ts:115-134` (docblock `:94-96`) —
   carried open from r1 finding 2 / r2 finding 2, unaddressed: the env **range**
   check still sits inside the env branch, behind the `flagValue !== undefined`
   return at `:115-123`, so `--timeout 5` with
   `OPENCODE_REPLY_TIMEOUT_SECONDS=3601` resolves silently to `5` and the broken
   environment is never reported. It matches the letter of the proposal (only the
   format gate is pinned as precedence-independent, proposal:82) and is declared
   as build-notes deviation 2 — hence minor rather than a deviation finding — but
   it contradicts the rationale written directly above it ("a broken environment
   is a configuration error even when a valid flag overrides it") and the Risks
   row promising that an env typo hard-fails the run, and no test pins it
   (`tests/cli.test.ts:137-139` covers `undefined, '3601'`, `:120` covers
   `5, '30s'`, never `5, '3601'`). Fix: validate the env value fully (format gate
   **and** `1..maxSeconds`) before the flag branch and then apply precedence; or
   keep the behaviour, scope the docblock sentence to the format gate, and pin it
   with `expect(resolveReplyTimeoutSeconds(5, '3601')).toBe(5)`.

4. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:122-127` —
   carried open from r2 finding 4, unaddressed: the Round-2 observation still
   names the wrong test. `afterEach` clears `OPENCODE_REPLY_TIMEOUT_SECONDS`
   after every `main()` test (`tests/cli.test.ts:162-168`), so by the time the
   "30 s default" case (`:204-211`) runs, an ambient value has already been
   removed. The real exposure is the **first** `main()` test (`:172-179`), which
   runs before any `afterEach`: an ambient malformed/empty value throws through
   the precedence-independent format gate and fails that test; an ambient
   well-formed value is masked there only because the test happens to pass
   `--timeout 2`. Checklist #5: a build-notes claim that does not match the tree.
   Fix: name the first `main()` test, or make the note moot by clearing the
   variable in the `main` describe's `beforeEach` (`:144-160`).

5. **[nit]** `tools/orchestration/src/cli.ts:109-112` — carried open from r1
   finding 4 / r2 finding 5: the format-gate message states the accepted range
   ("must be an integer number of seconds in 1..3600 (plain digits only)")
   although the failure it reports is a *format* violation, blurring the
   deliberate format/range split (proposal:82);
   `OPENCODE_REPLY_TIMEOUT_SECONDS=30s` is told about a range it never violated.
   Fix: keep the format error format-only (e.g. `$OPENCODE_REPLY_TIMEOUT_SECONDS
   must be plain decimal digits (integer seconds), got …`) and leave the range
   wording to `:129-130`.

6. **[nit]** `tools/orchestration/src/cli.ts:144,188` — carried open from r1
   finding 5 / r2 finding 6: `configFromEnv()` is still called twice per run
   (once inside `parseArgs()`, again for `replyTimeoutSeconds`), so the resolver
   validates a second, independent env read rather than the snapshot yargs used
   for the other defaults — at odds with A3's "env reads centralized" intent.
   Fix: have `parseArgs(argv)` take or return the `EnvValues` bag and pass that
   same snapshot into `resolveReplyTimeoutSeconds`.

## FYI (out of specialty, no severity)

No new out-of-specialty observation this round beyond what the other r2 reports
already carry: `this.name = 'ReplyTimeoutError'` as a second consumer-visible
delta (`reviews/code/compatibility-r2.md:121-137`), the unasserted startup banner
and the `ReplyTimeoutError` suffix that names CLI knobs to library consumers
(`reviews/code/testing-r2.md:156-167`,
`reviews/code/compatibility-r2.md:203-214`), Test #5's unasserted "names the bad
value" (`reviews/code/testing-r2.md:148-155`), the banner run-on that the
proposal itself prescribes (`reviews/code/compatibility-r2.md:218-221`), and the
three added test lines over `printWidth: 100` — the README table, which I *could*
verify without shell, is prettier-stable at 99 characters per row.
