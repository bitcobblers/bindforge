# Code review — testing (round 3)

- Verdict: **approve** — 0 blocker, 0 major, 3 minor, 7 nit
- Range: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`
  (`baseRef`, `.pipeline.json:1109`). `git diff`/`git log` are refused in this
  review environment (`Permission denied: shell`), so the range was reviewed as
  the HEAD tree; the timeout change set is `src/opencode-runner.ts`,
  `src/cli.ts`, `tests/opencode-runner-timeout.test.ts`, `tests/cli.test.ts`,
  `tests/opencode-runner.test.ts` (the pinned guard), `package.json`,
  `package-lock.json`, `README.md`, cross-checked against
  `reviews/code/{correctness,security,compatibility,performance}-r3.md`, which
  independently anchor the same file set.
- Design: `02-proposal.md` (r4) Test strategy #1–#12; notes: `03-build-notes.md`
  (incl. its "Round 2" section)
- Checklist: `.opencode/skills/code-review/references/testing.md` (only)
- Previous round: `reviews/code/testing-r2.md` — approve, 0 blocker/major,
  findings 1–8 (2 minor, 6 nit). No `implement`/`fix`/`test-authoring` stage ran
  between r2 and r3 (`.pipeline.json:760-970`), so **every r2 finding is still
  open**; each below is marked carried or new and re-verified against the tree.

## Checklist walk

1. **Every numbered Test strategy item has a citing test.** `// proves: Test
   strategy #N` occurrences (18, grepped): #2
   (`tests/opencode-runner-timeout.test.ts:65`), #3 (`:87`), #4 (`:108`), #5
   (`:138`), #6 (`:156`); #7 (`tests/cli.test.ts:66`), #8 ×5 (`:73`, `:87`,
   `:97`, `:108`, `:117`), #9 (`:125`), #10 ×5 (`:8`, `:170`, `:181`, `:192`,
   `:203`), #11 (`:8`, `:213`). **Not cited: #1** (finding 4) and **#12** (a gate,
   not a test — finding 3). Assertion coverage matches the proposal where cited:
   #8's full matrix (`0`, `-5`, `2.5`, `Infinity`, `NaN`; env `"abc"`, `"30s"`,
   `"0x2d"`, `"1e4"`, `" 45\n"`, `"+45"`, set-but-empty `""`, ANSI `'\x1b[2J'`,
   flag `5` + env `"30s"`) at `cli.test.ts:75-123`; #9's `1`/`3600`/`3601` at
   `:127-140`; #10's four precedence routes at `:172-211`; #11's zero-`createClient`
   + zero-stdout + one-line message at `:215-232`; #5's five invalid values with
   zero `create`/`prompt`/`list` calls at `opencode-runner-timeout.test.ts:140-153`;
   #6's exactly-two-`list` + `within 0.1 s` at `:159-175`. Two proposal-named
   assertions are still unpinned: #5's "TypeError **naming the bad value**"
   (finding 5) and the env-range-under-a-flag behaviour (finding 2).
2. **Both promised layers exist.** Unit: `runPrompt` timeout cases through
   `stubClient()` + fake timers (`opencode-runner-timeout.test.ts:60-176`) and the
   pure resolver matrix (`cli.test.ts:65-141`). Integration: in-process
   `main(['node','cli', …])` with `vi.mock('../src/opencode-runner.js')` +
   `importOriginal` (`cli.test.ts:29-36`) — exactly the route #10 commits to; the
   factory returns a stub `PromptResult` and passes `extractReply` through, so
   `main()` does not die downstream at `src/cli.ts:217` after the asserted call,
   and `createClient` is a `vi.fn()` for #11's zero-call assertion. A19's recorded
   fallback (`clientFactory` seam) was correctly not needed
   (`03-build-notes.md:79-85`). ✓
3. **Revision rounds: previously rejected blocker/major has a catching test.**
   The testing round raised none; across all r1/r2 code reviews the only
   **major** in the run is correctness-r1 finding 1 (`npm run check` never
   executed), which no test can catch — its closure is execution. That evidence
   was stale at r2 (the sole green gate entry, `.pipeline.json:1065-1069`
   `at: 1794118188887`, predates `d7be29f`). **This round I executed
   `npm run check` at HEAD and it is green** (tail in finding 3), and the recorded
   91 tests matches a static count of this tree (87 static `it(` in the 7 suites,
   one expanded by the 5-case loop at `opencode-runner-timeout.test.ts:140`). So
   the r1 major is closed at HEAD by execution; the residual testing gap is the
   absence of any in-repo record of it (finding 3). All eight r2 findings remain
   open and are carried.
4. **Determinism.** New suites: no network, no live server, no ports, no
   `process.platform`/`os.EOL`/`path.sep`, no wall-clock assertions. Every
   clock-sensitive test installs fake timers and restores real timers in both a
   `finally` and `afterEach` (`opencode-runner-timeout.test.ts:61-63`, `:82-84`,
   `:103-105`, `:133-135`, `:172-174`); the fail-closed cases (`:140-153`) run on
   real timers and assert synchronous rejection before any timer exists, as #5
   requires. Timer advances are sufficient and non-tight (4 250 ms vs a 4 000 ms
   deadline; 500 ms vs one 250 ms sleep; 29 000 + 2 000 vs 30 000 ms), and every
   rejected promise is attached (`void pending.catch(…)` at `:72`, `:164`;
   `pending.then(onFulfilled, onRejected)` at `:115-122`), so no
   unhandled-rejection flake. The module-level `isMain` guard
   (`src/cli.ts:221`) keeps `process.exit` inert under the test import ✓. Two
   defects: ambient-env coupling in the `main` suite (finding 1) and the implicit
   fake-`Date` dependency of the wall-clock deadline (finding 8).
5. **Behaviour-named tests.** All names read as behaviour ("rejects with
   ReplyTimeoutError naming the configured 4 s duration", "lets --timeout win
   over $OPENCODE_REPLY_TIMEOUT_SECONDS"). The call-count name at
   `opencode-runner-timeout.test.ts:159` is implementation-shaped but *is* the
   contract #6 pins, so it is correct here. ✓
6. **`npm run check` executed by this review — green.** Run in
   `tools/orchestration` at HEAD; tail:
   ```
   npm notice run eslint .
   npm notice run prettier --check .
   Checking formatting...
   All matched files use Prettier code style!
   npm notice run vitest run
    RUN  v5.0.5 /home/mike/.local/share/opencode/worktree/f852d4/misty-squid/tools/orchestration
    Test Files  7 passed (7)
         Tests  91 passed (91)
      Start at  01:51:39
      Duration  857ms (transform 49%, tests 28%, import 22%, worker 1%)
   ```
   So Goal 4 ("existing tests pass unchanged") and Test strategy #12's Linux gate
   are now evidenced by execution at the tree being merged — including the pinned
   guard at `tests/opencode-runner.test.ts:243-257`, untouched, whose substring
   `'No response received from OpenCode'` (`:253`) still matches the new
   `ReplyTimeoutError` prefix (`src/opencode-runner.ts:349`). `node -v` could not
   be run (denied), so the Node version is not recorded (finding 3).
7. **Untestable additions / unexercised branches.** `PromptOptions.timeoutMs`,
   `resolveReplyTimeoutSeconds`, `ReplyTimeoutError`,
   `DEFAULT_REPLY_TIMEOUT_MS` and `MAX_REPLY_TIMEOUT_MS` all have production
   consumers *and* tests; `engines` (`package.json:30-32`) is a proposal-mandated
   anchor, not a knob without a consumer; no dead branch was added to the timeout
   path. Still unexercised: the env-**range**-under-a-flag branch (finding 2), the
   extended startup banner and `ReplyTimeoutError.name` (finding 6), and the
   CLI-boundary junk-flag path the A8 safety argument rests on (finding 9).

## Findings

1. **[minor]** `tools/orchestration/tests/cli.test.ts:144-168` — carried open
   from testing-r2 finding 2 (checklist item 4): the `main` suite is still not
   hermetic against the caller's environment. `beforeEach` (`:144-160`) resets
   only the two runner mocks; `delete process.env[TIMEOUT_ENV]` sits in
   `afterEach` (`:162-168`), which has not yet run when the **first** `main()`
   test (`:172-179`) executes. With `OPENCODE_REPLY_TIMEOUT_SECONDS` exported in
   the shell that runs `npm run check`, that test inherits it: a malformed or
   set-but-empty value throws through the precedence-independent format gate
   (`src/cli.ts:108`) and fails a test that has nothing to do with the ambient
   value; a well-formed one is masked there only because the test happens to pass
   `--timeout 2`. (I could not demonstrate it by execution — an env-prefixed
   `npm test` invocation is refused here — but the ordering is unambiguous from
   the hooks.) `03-build-notes.md:122-127` records the same hazard against the
   wrong test (the "30 s default" case at `:204-211` is already protected by the
   preceding `afterEach`), so the tree's only note about it is misleading.
   Fix: add `delete process.env[TIMEOUT_ENV]` to the `main` describe's
   `beforeEach` (or save/restore in `beforeAll`/`afterAll`, which also avoids
   destroying an ambient value for sibling files in the same worker).
2. **[minor]** `tools/orchestration/src/cli.ts:115-134` (docblock `:91-99`) —
   carried open from correctness-r1 finding 2 / testing-r2 finding 3 (checklist
   item 7): the env **range** check still sits inside the env branch, behind the
   `flagValue !== undefined` return at `:115-123`, so `--timeout 5` with
   `OPENCODE_REPLY_TIMEOUT_SECONDS=3601` resolves silently to `5` and the broken
   environment is never reported. `tests/cli.test.ts` never pins it: `:137-139`
   covers `resolverMessage(undefined, '3601')`, `:119-123` covers
   `resolverMessage(5, '30s')`, nothing covers `resolverMessage(5, '3601')`. The
   behaviour matches the letter of the proposal (only the format gate is pinned
   precedence-independent, proposal:82) and is declared as `03-build-notes.md`
   deviation 2 — hence a coverage/docblock finding rather than a design
   contradiction — but the docblock sentence above it still promises "a broken
   environment is a configuration error even when a valid flag overrides it", and
   a reader of the suite cannot tell which behaviour is intended. Fix (test
   stage): pin it with
   `expect(resolveReplyTimeoutSeconds(5, '3601')).toBe(5)` plus a comment naming
   it as precedence-dependent range behaviour, and scope the docblock sentence to
   the format gate; or move the full env validation (gate + range) above the flag
   branch so code, docblock, Risks row and tests agree.
3. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` and
   `:129-136` — carried open from testing-r2 finding 1 / correctness-r1 finding 1,
   now substantially closed and narrowed: Test strategy #12 requires `npm run
   check` to pass on Linux, and I executed it at HEAD green (tail quoted above,
   checklist walk 6), which also supplies the closure evidence the r1 major
   lacked. What is still missing is any in-repo record of it: deviation 1 and the
   Round-2 paragraph still state the gate "could not be executed" and "must still
   be run by the caller", and no `testsGreen` gate entry exists after
   `.pipeline.json:1065-1069`. A later reader of the artifacts cannot tell that
   #12 was satisfied at the merged tree, and #12 has no citation anywhere either.
   Fix: rewrite deviation 1 and the Round-2 paragraph to record the executed run
   (Linux, lint/format status, per-suite counts — 7 files / 91 tests, and the Node
   version if it can be captured) and state that macOS/Windows remain
   unexercised because a CI matrix leg is an explicit non-goal. If the run had
   been red this would escalate to a major; it was not.
4. **[nit]** `tools/orchestration/tests/opencode-runner.test.ts:243-257` —
   carried open from testing-r2 finding 4 (checklist item 1): Test strategy **#1**
   still has no citation anywhere in the tree — the only item with no `// proves:`
   marker besides #12. The item is satisfied in substance: the guard is unedited
   (that file contains no `timeout`/`timeoutMs`/`ReplyTimeoutError` text at all)
   and passes by substring against the new message, and I re-ran it green. A
   `// proves:` comment cannot go in that file without violating #1's "passes with
   **no edits**". Fix: cite it from the new suite, e.g. above
   `tests/opencode-runner-timeout.test.ts:65` add
   `// proves: Test strategy #1 (guard, unedited) — tests/opencode-runner.test.ts:243–257 keeps the 30 s default and the message prefix`.
5. **[nit]** `tools/orchestration/tests/opencode-runner-timeout.test.ts:140-153`
   — carried open from testing-r2 finding 5: Test #5 pins the error *type* and the
   zero-request consequence, but not the message the proposal requires ("throws a
   `TypeError` **naming the bad value**", proposal:71). The source does name it
   (`src/opencode-runner.ts:242-245`), and no assertion would notice a regression
   that dropped it — which is the point of the fail-closed report. Fix: inside the
   loop, assert
   `expect(messageOf(error)).toContain(\`Invalid timeoutMs: ${String(timeoutMs)}\`)`;
   prefer the full prefix over a bare `toContain(String(timeoutMs))`, which
   `0`/`-5` would satisfy from the `1..3600000` range text alone.
6. **[nit]** `tools/orchestration/src/cli.ts:203-206` and
   `tools/orchestration/src/opencode-runner.ts:352` — carried open from
   testing-r2 finding 6: two pieces of user-visible output added by this change
   set have no assertion. The extended banner
   (` waiting up to ${config.timeoutSeconds} s for the reply…`) is only ever
   observed on the *negative* path (`tests/cli.test.ts:231` asserts nothing was
   printed); the four #10 tests inspect only the options handed to the mocked
   `runPrompt`. Likewise `this.name = 'ReplyTimeoutError'` — an undeclared-in-design
   deviation (`03-build-notes.md` deviation 3) that compatibility-r3 finding 1
   flags as a second consumer-visible delta — has no test. Fix: in the
   "passes --timeout 2" test capture the spy (`const { log } = silenceOutput()`)
   and assert
   `expect(log.mock.calls.flat().join(' ')).toContain('waiting up to 2 s for the reply')`;
   in the #2 test add `expect((error as Error).name).toBe('ReplyTimeoutError')` if
   the deviation is meant to be part of the contract.
7. **[nit]** `tools/orchestration/tests/cli.test.ts:114` — carried open from
   testing-r2 finding 8: `expect(message).not.toContain('[2J\n')` is near-vacuous —
   the resolver's message is single-line (pinned at `:229`) and never contains
   that substring whatever the escaping, so the assertion cannot fail. The load-
   bearing check is `:113` (`not.toContain('\x1b')`). Fix: drop it or replace it
   with an assertion that carries weight, e.g.
   `expect(/[\u0000-\u001f\u007f]/.test(message)).toBe(false)`.
8. **[nit]** `tools/orchestration/tests/opencode-runner-timeout.test.ts:7-44` —
   carried open from testing-r2 finding 7: ~38 lines of fixtures
   (`FactSchema`, `fakeSession`, `fakeReply`, `stubClient`) are duplicated
   verbatim from `tests/opencode-runner.test.ts:9-46`, so two copies of the SDK
   client shape must move together whenever the envelope or the `waitForReply`
   signature changes, and drift silently changes what each suite exercises. No
   `tests/helpers/` exists (glob: seven flat suites). Related: the fake-`Date`
   dependency of the wall-clock deadline (`src/opencode-runner.ts:290`) is
   implicit — there is no `vitest.config.*` and no `useFakeTimers({ toFake: … })`
   anywhere in the package (grep), so these suites rely on Vitest 5's default
   `toFake` set including `Date`. If that default ever narrows, the deadline never
   advances and the affected tests fail by 5 s timeout rather than by a named
   assertion. Fix: extract a shared `tests/helpers/opencode-stubs.ts` (test-stage
   work) and pin the fake set explicitly, e.g.
   `vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'Date'] })`.
9. **[nit]** `tools/orchestration/tests/cli.test.ts:81-85`, `:172-232` — new
   (checklist item 1/7): the only value A8 (register: **unverified** — "invalid
   numeric input reaches downstream validation as a non-finite value rather than
   failing in yargs") ever sees is a hand-written `Number.NaN` fed straight to the
   resolver. Nothing drives junk argv through the real `parseArgs()`: no test calls
   `main(['node','cli','--timeout','abc'])` (or `1e4`, or a repeated flag). That
   assumption is what makes the CLI-side range check the actual guard for
   operator input, so it belongs at the boundary, not only in a unit call. Fix: add
   a `main` case asserting `main(['node','cli','--timeout','abc'])` rejects with a
   message containing `--timeout`, `runnerMocks.createClient` never called and
   `log` never called — one test that also re-pins #11's side-effect-free property
   for the flag path.
10. **[nit]** `tools/orchestration/tests/cli.test.ts:172-211` vs
    `tests/opencode-runner-timeout.test.ts:140-153` — new (checklist item 7): the
    CLI↔runner unit boundary is only tested at `2`/`5`/default seconds. The
    coupling between the CLI cap (`maxSeconds = MAX_REPLY_TIMEOUT_MS / 1000`,
    `src/cli.ts:106`) and the runner cap (`MAX_REPLY_TIMEOUT_MS`,
    `src/opencode-runner.ts:241`) is untested at the top of the range, so a
    divergence (e.g. the CLI accepting `3600` while the runner rejects
    `3_600_000`) would regress silently into the runner's `TypeError` mid-run
    instead of the resolver's config error. Fix: add
    `await main(['node','cli','--timeout','3600'])` and assert
    `runPromptOptions()` matches `expect.objectContaining({ timeoutMs: 3_600_000 })`
    — the boundary value, not just a mid-range one.

## Out-of-specialty FYI (no severity)

The reviewed range still carries the pre-run pipeline-infrastructure commits
(`src/pipeline/gates.ts`, `prompts.ts`, `runner.ts`, plus
`tests/pipeline-gates.test.ts`, `tests/pipeline-prompts.test.ts` and
`.opencode/agents/codereview/code-compatibility.md`) — explicit non-goals of the
design (proposal:17, :23), graded as range hygiene by correctness-r3 finding 2 and
audited on its own merits by security-r3 findings 2–5 and compatibility-r3
finding 3. Their own new tests are pure-function unit tests with behaviour names
and no clock/network use, so they raise no testing findings of themselves, but
they cite no Test strategy item because they belong to no design in this run.
Separately, the earlier round's "three added test lines exceed `printWidth: 100`"
note is now moot: `prettier --check .` is clean at HEAD in my run above.

## Verdict

**approve** — zero open blocker/major testing findings. Test strategy #2–#11 all
have citing, behaviour-named, deterministic tests that trace to the shipped
source; both promised layers exist and use the route #10 committed to; #1's
unedited guard passes and #12's Linux gate is now green **by my own run at this
HEAD** (7 files / 91 tests, lint and format clean) — closing the run's only code
major with execution evidence at the tree being merged. The open items are two
hermeticity/coverage minors, one artifact-record minor, and seven nits, all
carried from r2 except findings 9 and 10; none contradicts the design, and no
implement stage ran between r2 and r3, so none of them could have moved.
