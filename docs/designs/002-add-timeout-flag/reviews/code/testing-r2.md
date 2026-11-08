# Code review — testing (round 2)

Scope: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD` (base pin
`.pipeline.json:899`), judged against `02-proposal.md` (r4) Test strategy
#1–#12 and `03-build-notes.md`. Checklist:
`.opencode/skills/code-review/references/testing.md`. Previous round: no
`reviews/code/testing-r1.md` exists, so checklist #3 is evaluated against the
blockers/majors raised by the *other* r1 code reviews
(`reviews/code/correctness-r1.md` — 1 major, 2 minors, 2 nits).

This report replaces the earlier file at this path, whose stage failed its
reply gate (`.pipeline.json:889-895` — `code-review:testing: invalid-reply: not
JSON`); its findings were never recorded in the run log, so this round re-does
the checklist from the tree.

Environment note (matters for checklist #6): this review environment accepts
`git diff`/`git log` but refuses every other shell command — `npm run check`,
`npx vitest`, `git show` all return `Permission denied: shell`. So the gate
could not be re-executed here; findings below grade that explicitly (finding 1),
and the rest is a line-by-line trace of the suites against the source.

## Checklist walk

1. **Every Test strategy item is cited.** Cited with `// proves: Test strategy
   #N`: #2 (`tests/opencode-runner-timeout.test.ts:65`), #3 (`:87`), #4
   (`:108`), #5 (`:138`), #6 (`:156`); #7 (`tests/cli.test.ts:66`), #8 ×5
   (`:73`, `:87`, `:97`, `:108`, `:117`), #9 (`:125`), #10 ×5 (`:8`, `:170`,
   `:181`, `:192`, `:203`), #11 (`:8`, `:213`). Not cited: **#1** (finding 4)
   and **#12** (a gate, not a test — covered by finding 1). Assertion coverage
   matches the proposal where cited: #8's full matrix (`0`, `-5`, `2.5`,
   `Infinity`, `NaN`; env `"abc"`, `"30s"`, `"0x2d"`, `"1e4"`, `" 45\n"`,
   `"+45"`, set-but-empty `""`, ANSI `'\x1b[2J'`, flag `5` + env `"30s"`) at
   `cli.test.ts:75-123`; #9's `1`/`3600`/`3601` at `:127-140`; #10's four
   precedence routes at `:172-211`. Two sub-assertions the proposal names are
   not pinned: #5's "TypeError **naming the bad value**" (finding 5) and the
   env-range precedence behaviour (finding 3).
2. **Both layers exist.** Unit: `runPrompt` timeout cases through `stubClient()`
   + fake timers (`opencode-runner-timeout.test.ts:60-176`) and the pure
   resolver matrix (`cli.test.ts:65-141`). Integration: `main(['node','cli',
   …])` in-process with `vi.mock('../src/opencode-runner.js')` +
   `importOriginal` (`cli.test.ts:11-36`), which is exactly the route proposal
   #10 commits to; build-notes deviation 5 records the A19 spike outcome, so
   the recorded `clientFactory` fallback was correctly not needed. ✓
3. **Revision rounds.** No testing-r1 round exists. The only r1 code-review
   **major** anywhere was correctness-r1 finding 1 ("`npm run check` never
   executed → Goal 4 and the merge gate unevidenced"). No test can catch an
   unexecuted gate, so the closure evidence is execution, not a test: the
   pipeline gate log records `gate: testsGreen` / `reason: "npm run check
   green"` (`.pipeline.json:861-867`) and a prior testing attempt at this path
   recorded a green run (7 files / 87 tests) at 01:30:57, i.e. after the last
   source commit in the range. The one r1 major that *is* test-shaped — the
   env-range precedence gap (correctness-r1 finding 2) — still has **no** test;
   finding 3. r1 minors 3/4/5 remain unaddressed (2, 3 are test-relevant:
   findings 3 and 6).
4. **Determinism.** No network, no live server, no ports, no `process.env`
   reads outside the resolver's own input, no `process.platform`/`os.EOL`/
   `path.sep` in the new suites. Every clock-sensitive test installs fake
   timers and restores real timers in a `finally` and in `afterEach`
   (`opencode-runner-timeout.test.ts:61-63`, `:82`, `:103`, `:133`, `:173`);
   Vitest's default fake set includes `Date`, which the wall-clock
   `Date.now() + timeoutMs` deadline depends on — the pre-existing guard at
   `opencode-runner.test.ts:243-257` already proved that dependency works in
   this toolchain (finding 7 asks for it to be made explicit). The fail-closed
   cases (`:140-154`) assert synchronous rejection before any timer exists, so
   they are clock-independent as the proposal requires ("real timers"). The one
   determinism defect is ambient-env coupling in the `main` suite: finding 2.
5. **Test names describe behaviour.** All names read as behaviour
   ("rejects with ReplyTimeoutError naming the configured 4 s duration",
   "keeps polling until 30 s elapsed when no timeoutMs is given", "lets
   --timeout win over $OPENCODE_REPLY_TIMEOUT_SECONDS"). The call-count name at
   `opencode-runner-timeout.test.ts:159` looks implementation-shaped but *is*
   the contract Test #6 pins (exactly two `message.list` checks), so it is
   correct here. ✓
6. **`npm run check` executed by this review.** Not possible here (see the
   environment note): no tail can be pasted from my own run. Finding 1.
7. **Untestable additions / unexercised branches.** `PromptOptions.timeoutMs`,
   `resolveReplyTimeoutSeconds`, `ReplyTimeoutError`, `DEFAULT_REPLY_TIMEOUT_MS`
   and `MAX_REPLY_TIMEOUT_MS` all have production consumers *and* tests; no dead
   code was added, and the `engines` line is a proposal-mandated config anchor,
   not a knob without a consumer. Two additions/branches are unexercised: the
   env-range-under-a-flag branch (finding 3) and the extended startup banner
   (finding 6).

## Findings

1. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` and
   `:129-136` — checklist item 6 could not be evaluated by execution here
   (`npm run check` refused: `Permission denied: shell`), so it counts as a
   finding rather than a pass. The in-tree evidence is second-hand and partly
   stale: the pipeline gate log's only green entry (`.pipeline.json:861-867`,
   `at: 1794118188887`) sits ~3 s after the implement commit `0d7762a`
   (`1794118186`) and therefore predates commit `d7be29f` (`1794118457`,
   "fix(orchestration): add the missing code-compatibility agent; fail fast on
   zero model turns"), which changed `tools/orchestration/src/pipeline/runner.ts`
   (+15/−1) — so that entry alone does not certify HEAD. The strongest evidence
   is the superseded report at this same path, which records running
   `npm run check` green at 01:30:57 (`Test Files 7 passed (7)`, `Tests 87
   passed (87)`, `Duration 688ms`), i.e. after `d7be29f`; I can cite it but not
   confirm it. Meanwhile deviation 1 and the Round-2 paragraph still state the
   gate "must still be run by the caller", which is now inaccurate either way.
   Fix: re-run `npm run check` in `tools/orchestration` (or let the runner's
   `testsGreen` gate re-confirm HEAD), paste the summary block — OS, `node -v`,
   per-suite test counts, lint/format status — into `03-build-notes.md`, and
   rewrite deviation 1/Round-2 to cite that run instead of deferring it.
2. **[minor]** `tools/orchestration/tests/cli.test.ts:144-168`, `:172-179` —
   the `main` suite is not hermetic against the caller's environment (checklist
   item 4). `beforeEach` (`:144-160`) resets only the runner mocks; the env
   clear lives in `afterEach` (`:167`), which has not yet run when the *first*
   `main()` test executes. If the shell running `npm run check` exports
   `OPENCODE_REPLY_TIMEOUT_SECONDS`, that test inherits it: a malformed or
   empty ambient value throws through the precedence-independent format gate
   (`src/cli.ts:108`) and fails the test, and a well-formed ambient value is
   masked there only because the test happens to pass `--timeout 2`. The suite's
   result thus depends on ambient state. Build-notes `:122-127` recorded this
   hazard against the wrong test and correctness-r2 finding 4 corrected the
   wording, but nobody fixed the suite. Fix: add
   `delete process.env[TIMEOUT_ENV]` to the `main` describe's `beforeEach` (or
   save/restore in `beforeAll`/`afterAll`), which also makes the build-notes
   observation moot.
3. **[minor]** `tools/orchestration/src/cli.ts:115-134` (docblock `:94-96`) —
   unexercised branch (checklist item 7), open since correctness-r1 finding 2:
   the env **range** check sits behind the `flagValue !== undefined` return, so
   `--timeout 5` with `OPENCODE_REPLY_TIMEOUT_SECONDS=3601` silently resolves to
   `5`. Nothing in `tests/cli.test.ts` pins that behaviour — the suite tests
   `resolveReplyTimeoutSeconds(undefined, '3601')` (`:137-139`) and
   `resolverMessage(5, '30s')` (`:120`), but never a *well-formed* out-of-range
   env value together with a flag — while the docblock directly above the code
   promises "a broken environment is a configuration error even when a valid
   flag overrides it". A reader of the suite cannot tell whether the silent
   resolve is intended or an oversight. Fix (test-stage, one line): add
   `expect(resolveReplyTimeoutSeconds(5, '3601')).toBe(5)` with a comment naming
   it as the precedence-dependent range behaviour, and correct the docblock
   wording to scope the precedence-independent promise to the format gate; or
   move the full env validation (gate + range) above the flag branch so the
   docblock, the Risks row and the tests agree.
4. **[nit]** `tools/orchestration/tests/opencode-runner.test.ts:243-257` —
   Test strategy **#1 has no citation anywhere in the tree**, which is the
   literal requirement of checklist item 1. The item is otherwise satisfied:
   `git log --name-only -1 869e633` shows the run's only test commit added just
   `cli.test.ts` and `opencode-runner-timeout.test.ts`, so the pinned guard is
   unedited and still matches the new `ReplyTimeoutError` message by substring
   (`:253` vs `src/opencode-runner.ts:349-352`). A `// proves:` comment cannot
   go in that file without violating the proposal's "passes with **no edits**".
   Fix: cite it from the new suite — e.g. above
   `tests/opencode-runner-timeout.test.ts:65`, add
   `// proves: Test strategy #1 (guard, unedited) — tests/opencode-runner.test.ts:243–257 keeps the 30 s default and the message prefix` —
   so the coverage map is complete without touching the pinned test.
5. **[nit]** `tools/orchestration/tests/opencode-runner-timeout.test.ts:138-154`
   — Test #5 pins the *type* and the zero-request consequence but not the
   message the proposal requires: "throws a `TypeError` **naming the bad
   value**". The source does name it (`src/opencode-runner.ts:241-246`), and no
   assertion would notice a regression that dropped the value, which is the
   whole point of the fail-closed report. Fix: in the loop body, add
   `expect(messageOf(error)).toContain(String(timeoutMs))` alongside
   `expect(error).toBeInstanceOf(TypeError)`.
6. **[nit]** `tools/orchestration/src/cli.ts:203-206` — the extended startup
   banner (` waiting up to <n> s for the reply`) is new user-visible output of
   this change set with no assertion anywhere: the four #10 tests inspect only
   the options handed to the mocked `runPrompt`, and #11 (`tests/cli.test.ts
   :215-232`) asserts only that *nothing* is printed on the error path. Fix: in
   the "passes --timeout 2" test, capture the spy (`const { log } =
   silenceOutput()`) and assert
   `expect(log.mock.calls.flat().join(' ')).toContain('waiting up to 2 s for the reply')`;
   do the same for the `this.name = 'ReplyTimeoutError'` deviation
   (`src/opencode-runner.ts:352`, build-notes deviation 3) with
   `expect((error as Error).name).toBe('ReplyTimeoutError')` if the deviation is
   meant to be part of the contract.
7. **[nit]** `tools/orchestration/tests/opencode-runner-timeout.test.ts:7-44`
   duplicates ~38 lines of fixtures verbatim from `tests/opencode-runner.test.ts
   :9-46` (`FactSchema`, `fakeSession`, `fakeReply`, `stubClient`). Two copies of
   the client shape now have to change together whenever the SDK envelope or the
   `waitForReply` signature moves, and a drift between them silently changes
   what each suite exercises. Fix: extract a `tests/helpers/opencode-stubs.ts`
   shared by both suites (test-stage work; the implement stage may not edit
   tests per the runner's ownership rule). Related: the fake-`Date` dependency
   of the timeout suites is implicit — Vitest fakes `Date` by default
   (`fakeTimers.toFake` defaults to everything global except `nextTick` and
   `queueMicrotask`), so pinning `vi.useFakeTimers({ toFake: ['setTimeout',
   'clearTimeout', 'Date'] })` or a `test.fakeTimers` entry would keep the
   deadline arithmetic explicit if the toolchain default ever narrows.
8. **[nit]** `tools/orchestration/tests/cli.test.ts:114` —
   `expect(message).not.toContain('[2J\n')` is a near-vacuous assertion: the
   message never contains that substring regardless of escaping, so it cannot
   fail. The real check is `:113` (`not.toContain('\x1b')`). Fix: drop it or
   replace it with an assertion that carries weight, e.g.
   `expect(message).toMatch(/^\$OPENCODE_REPLY_TIMEOUT_SECONDS .*"\u001b\[2J"$/)`
   or a `// eslint-disable`-free scan for control bytes
   (`// eslint-disable-next-line no-control-regex` aside):
   `expect(/[\u0000-\u001f\u007f]/.test(message)).toBe(false)`.

## Out-of-specialty FYI (no severity)

The reviewed range still carries the pre-run pipeline-infrastructure commits
`f0a2e16`, `891567b` and `d7be29f` (`src/pipeline/gates.ts|prompts.ts|runner.ts`
plus their own green suites), a subsystem proposal:17 names as an explicit
non-goal — range hygiene already carried as a minor by correctness-r1:112-129
and correctness-r2:176-193; their new tests
(`tests/pipeline-gates.test.ts:280-305`, `tests/pipeline-prompts.test.ts:115-141`)
are pure-function unit tests with behaviour names and no clock/network use, so
they raise no testing findings of their own, but they cite no Test strategy
item because they belong to no design in this run. Separately, three added test
lines exceed `.prettierrc`'s `printWidth: 100`
(`opencode-runner-timeout.test.ts:141`, `:159`, `cli.test.ts:215`); format
check is reported green, so Prettier accepts them — a formatting question for
whoever next touches those files.

## Verdict

**approve** — zero open blocker/major testing findings. Every implementable
Test strategy item (#2–#11) has a citing, behaviour-named, deterministic test
that traces cleanly to the shipped source; both the unit and integration layers
the proposal promises exist and use the route #10 committed to; the single r1
code-review major is closed by execution evidence rather than by a test. The
open items are one unevaluated-by-me gate (finding 1, with second-hand green
evidence in the tree), one hermeticity defect, one unpinned branch, and five
nits — none of which contradicts the design.
