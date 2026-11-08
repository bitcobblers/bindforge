# Build notes: 002-add-timeout-flag

Implementation followed `02-proposal.md` (r4) as written. What landed:

- `tools/orchestration/src/opencode-runner.ts`
  - `POLL_TIMEOUT_MS` replaced by exported `DEFAULT_REPLY_TIMEOUT_MS = 30_000`
    and `MAX_REPLY_TIMEOUT_MS = 3_600_000`; `POLL_INTERVAL_MS = 250` untouched.
  - `PromptOptions.timeoutMs?: number` added with the full runner-side
    contract in its docblock (integer `1..MAX_REPLY_TIMEOUT_MS`, enforced by
    `runPrompt`, sub-250 ms values legal with the ≤ one-interval overshoot).
  - `runPrompt` resolves `options.timeoutMs ?? DEFAULT_REPLY_TIMEOUT_MS` and
    throws a `TypeError` naming the bad value *before* `createSession` when it
    is not an integer in range — zero requests for invalid values (Test #5).
  - `waitForReply(client, sessionID, timeoutMs)` — same wall-clock deadline
    pattern; `null`-on-expiry contract unchanged.
  - New exported `ReplyTimeoutError` in the Errors section; message keeps
    `'No response received from OpenCode'` as an exact prefix and appends
    `within <n> s (set --timeout or $OPENCODE_REPLY_TIMEOUT_SECONDS to change
    it)` with `<n>` = `timeoutMs / 1000` as a plain JS number. No data
    properties.
- `tools/orchestration/src/cli.ts`
  - Module-private `EnvValues` (raw env bag, gains `replyTimeoutSeconds`)
    split from `CliConfig` (gains the one validated `timeoutSeconds: number`);
    `CliArgs` gains `timeout?: number`.
  - yargs gains `.option('timeout', { type: 'number', describe: … })` with no
    `default:` (A8).
  - Exported pure `resolveReplyTimeoutSeconds(flagValue, envValue)`:
    `^\d{1,5}$` format gate on the env value before any `Number()` call,
    running precedence-independently; flag → env → `DEFAULT_REPLY_TIMEOUT_MS /
    1000`; range check `1..MAX_REPLY_TIMEOUT_MS / 1000` separate from the
    format rejection, naming the source (`--timeout` reflects `String(flag)`,
    `$OPENCODE_REPLY_TIMEOUT_SECONDS` reflects `JSON.stringify(env)`).
  - `main()` runs the resolver immediately after `parseArgs()` and before
    `createClient()` and any `console.log`; startup line extended with only
    ` waiting up to <n> s for the reply`; the "no password" hint stays
    verbatim; `runPrompt` now receives `{ agent, timeoutMs: seconds * 1000 }`.
- `tools/orchestration/package.json` — `"engines": { "node": ">=22.12.0" }`
  (plus the matching `engines` entry mirrored into `package-lock.json`'s root
  package entry, so `npm ci` stays in sync — a one-line addition beyond the
  proposal's "one `package.json` line", required by the lockfile's presence).
- `tools/orchestration/README.md` — `--timeout` row added to the options
  table and Behaviour item 3 reworded, exactly as the proposal specifies.
  The pre-existing README:28 session-title drift was left untouched
  (explicit non-goal).

No test files were edited; the three existing suites
(`opencode-runner.test.ts`, `opencode-runner-timeout.test.ts`, `cli.test.ts`)
were treated as the contract.

## Deviations and notes

1. **Gate not executed (environment constraint).** Test strategy #12 /
   Rollout #1 require `npm run check` (lint + format check + tests) to run
   locally on Linux before merge. This implementation environment rejects all
   shell commands, so the suite, ESLint, Prettier, and `tsc` could not be
   executed. The change was instead verified by line-by-line static review
   against the checked-in tests (fake-timer flow, error types/messages, mock
   expectations, import order and printWidth compliance checked manually).
   The caller's runner must still execute `npm run check` in
   `tools/orchestration`.
   *Status: closed with execution evidence (Round 5). Two independent runs cover
   it: the pipeline's `testsGreen` gate logged `npm run check` green at
   `2026-11-08T06:09:48.887Z` (2.8 s after the implement stage's reply), and the
   round-3 code reviews executed the gate themselves at HEAD in
   `tools/orchestration` — ESLint clean, `prettier --check .` clean, `vitest run`
   → 7 test files / 91 tests passed, vitest 5.0.5, Linux
   (`reviews/code/testing-r3.md:80-99`, `reviews/code/compatibility-r3.md:13-24`).
   What stayed uncaptured is the Node version (`node -v` was refused in those
   environments) and any macOS/Windows run, which the design leaves unverified by
   decision. The earlier statement here and in Round 2 — that the suite "must
   still be run by the caller" — is superseded; see Round 5.*
2. **Env range check is precedence-dependent (interpretation).** The proposal
   pins only the *format* gate (`^\d{1,5}$`) as precedence-independent ("the
   gate runs whether or not `--timeout` is present"). Implemented literally:
   a well-formed but out-of-range env value (e.g. `--timeout 5` with
   `OPENCODE_REPLY_TIMEOUT_SECONDS=3601`) resolves to the flag without error,
   while any malformed env string throws even under a valid flag (Test #8's
   pinned case `flag 5 + "30s"` passes). If fully precedence-independent
   range checking is desired, that is a one-line move; no test pins it.
3. **`ReplyTimeoutError` sets `this.name = 'ReplyTimeoutError'`.** Not
   specified by the proposal; it adds no data property (the proposal's
   "carries no data properties" constraint concerns the duration) and keeps
   the message contract byte-identical.
4. **Resolver message wording.** The env format error includes "(plain digits
   only)" and both range errors include "(unbounded waits are not
   supported)" — the latter is the Risks-table mitigation ("resolver error
   names the accepted range and states unbounded waits are not supported")
   realized in the message text. Messages stay single-line (pinned by Test
   #11's `split('\n')` assertion).
5. **A19 spike outcome.** The pre-authored `cli.test.ts` uses the
   `vi.mock('../src/opencode-runner.js')` + `importOriginal` route (route a);
   the `src/cli.ts` as implemented is compatible with it (the CLI's
   value imports — `DEFAULT_BASE_URL`, `DEFAULT_REPLY_TIMEOUT_MS`,
   `MAX_REPLY_TIMEOUT_MS`, `extractReply` — flow through the real module in
   the mock factory), so the recorded fallback (a `clientFactory` seam on
   `main()`) was not needed.

## Round 2 (implement re-run)

The round-0 implement stage landed the full change set above but its final
reply was truncated before the verdict JSON (see
`transcripts/implement-r0.md`), so the stage re-ran. The re-run found the
implementation already complete and **made no source changes**; instead it
re-verified the tree against the checked-in tests:

- Every assertion in the three suites was traced through the current source:
  - `opencode-runner.test.ts` — the 243–257 default-timeout test still matches
    by substring because `ReplyTimeoutError` keeps the old string as an exact
    prefix; the unchanged poll loop keeps the "three `message.list` calls" and
    error-wrapping tests green.
  - `opencode-runner-timeout.test.ts` — the 4 s and 30 s fake-timer cases rely
    on vitest faking `Date` (default `toFake` set) together with the wall-clock
    deadline; the `timeoutMs: 100` case yields exactly two `list` calls (the
    deadline is tested only after each reply check) and `within 0.1 s`; the
    fail-closed cases (`Infinity`, `NaN`, `0`, `-5`, `3_600_001`) throw the
    boundary `TypeError` before `createSession`, so the `create`/`prompt`/`list`
    spies stay at zero calls.
  - `cli.test.ts` — precedence, the format gate (including the set-but-empty
    `OPENCODE_REPLY_TIMEOUT_SECONDS=` case, which the `^\d{1,5}$` gate rejects
    before any `Number()` call), the `JSON.stringify`-escaped reflection, the
    `1`/`3600` bounds with `3601` as a range error, and both `main()` routes
    (resolver-before-`createClient`, `timeoutMs = seconds * 1000`) all trace to
    the shipped `resolveReplyTimeoutSeconds`/`main`.
- Formatting was re-checked by hand against `.prettierrc` (`printWidth: 100`,
  `importOrder` node → external → relative with separation): all new/edited
  source lines fit, the README options table is prettier-stable (all seven
  rows are 99 characters; separator column widths 12/32/45 equal the max cell
  widths, the new `OPENCODE_REPLY_TIMEOUT_SECONDS` cell being the widest of its
  column), and `package.json` keeps 2-space JSON indentation.
- `package-lock.json`'s root package entry carries the same `engines` block as
  `package.json`, so `npm ci` stays in sync.

One observation about the checked-in suites, recorded rather than acted on
(test files are off-limits): `cli.test.ts`'s `afterEach` deletes
`OPENCODE_REPLY_TIMEOUT_SECONDS` only *after* each `main()` test, so the
"30 s default" case would inherit an ambient value if the variable happened to
be set in the shell that runs `npm test`. The gate is unaffected under a clean
environment.
*Correction (Round 5): the exposed test is the **first** `main()` test
(`cli.test.ts:172-179`), which runs before any `afterEach` has cleared the
variable; the "30 s default" case at `:204-211` is already protected by the
`afterEach` of the test before it — `code-review:testing` r3 finding 1 and
`code-review:performance` r3's FYI both name the first test. The hazard itself
is unchanged: an ambient malformed or set-but-empty value throws through the
precedence-independent format gate (`src/cli.ts:108`) and fails a test that has
nothing to do with it.*

Deviation 1 (gate not executed) still stands and is now re-confirmed: this
environment rejects **every** shell command — `git status`, `npm test`, and a
plain `cd`/`npm` invocation were each refused with `Permission denied: shell` —
so `npm run check` (and therefore Test strategy #12's Linux gate) must still be
run by the caller in `tools/orchestration`. Nothing in the repository's other
tests imports the changed symbols (a tree-wide grep for `POLL_TIMEOUT_MS`,
`waitForReply`, `runPrompt`, `timeoutMs` touches only `src/cli.ts`,
`src/opencode-runner.ts`, and the three suites), matching assumption A11.

Suggested commit message (Rollout #2): `feat: add configurable reply timeout
to orchestration CLI`. The commit recorded in `.pipeline.json` uses exactly
this message.

## Round 3 (doc-sync audit, 2026-11-08)

Doc-sync re-compared the branch diff (`fdfd2bc…HEAD`) against `02-proposal.md`
hunk by hunk and confirmed the build notes above against the shipped tree.
Additions and confirmations:

- **Shipped source = implement output.** `.pipeline.json` records no `fix`
  stage: all r2/r3 code reviews approved without changes, so nothing moved
  after the implement stage. The diff of `src/opencode-runner.ts` and
  `src/cli.ts` matches Deviations 1–5 exactly.
- **Test #1 confirmed mechanically:** `tests/opencode-runner.test.ts` does not
  appear in the diff at all — it truly passed the change with zero edits.
- **Unrecorded implementation detail** (not drift): `main()` obtains the
  resolver's env input by calling `configFromEnv()` a second time rather than
  threading the raw value out of `parseArgs()`. Env reads remain centralized
  in `configFromEnv()` as the design requires; no observable difference.
- **Scope note:** the same branch diff also modifies `src/pipeline/`
  (`gates.ts`, `runner.ts`, `prompts.ts`) and pipeline-adjacent tests. These
  belong to the orchestration-harness work developing the pipeline itself on
  this branch, not to design 002's change set; no 002 stage touched those
  paths, so the proposal's pipeline non-goal stands.
- **Gate status unchanged:** this audit environment also could not run
  `npm run check` (only `git diff` invocations are permitted), so Deviation 1
  remains open and is the caller's action item before merge.

The proposal now carries a dated "As-built" section summarizing the same
findings (drift items 1–3: lockfile `engines` mirror, `ReplyTimeoutError.name`,
resolver message wording).

Doc-sync verdict: `drifted: true` — three minor drift items, all documented
in "Deviations and notes" and mirrored in the As-built section; everything
else in the change set matches the approved design. Suggested commit message
for the doc-sync commit: `docs: record as-built audit of 002-add-timeout-flag
in proposal and build notes`.

## Round 4 (doc-sync verification pass, 2026-11-08)

A third doc-sync attempt re-ran the same comparison
(`git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`). The earlier
As-built/build-notes findings were **verified, not duplicated**: drift items
1–3 (lockfile `engines` mirror, `ReplyTimeoutError.name`, resolver message
wording) are re-confirmed against the actual diff hunks and remain the only
drift from the approved design. This pass adds what those passes left open or
understated.

- **Deviation 1 closed — the gate ran.** `.pipeline.json`'s `gateLog` records
  `{"gate": "testsGreen", "round": 0, "proceed": true, "reason": "npm run
  check green"}` at `2026-11-08T06:09:48.887Z`, 2.8 s after the implement
  stage's reply, on the tree that already contained `tests/cli.test.ts` and
  `tests/opencode-runner-timeout.test.ts`. `testsGreen` (`src/pipeline/gates.ts`)
  spawns `npm run check` in every package directory, so Test strategy #12 /
  Rollout #1's local Linux gate is satisfied for this change set, and
  `.pipeline.json` carries `"testsGreen": true`. Remaining gap (unchanged
  requirement): the gate discards the child's output (`stdio: 'ignore'`), so no
  run summary — test counts, lint/format status, Node version — was captured
  anywhere. That was `code-review:correctness` r1's major fix request and it is
  still unmet; re-running `npm run check` in `tools/orchestration` and pasting
  the summary is the only action left from Deviation 1.
- **Code-review history corrected.** Round 3's line "all r2/r3 code reviews
  approved without changes" is true but incomplete: `code-review:correctness`
  **round 1 returned `request-changes`** (1 major — the gate above; 2 minor —
  the precedence-dependent env range check, and the out-of-scope pipeline
  commits inside the review range; 2 nits — the format-gate error quoting the
  accepted range, and `configFromEnv()` being called twice). **No `fix` stage
  exists in `.pipeline.json`**, so the source never changed in response; rounds
  2 and 3 approved and `allCodeReviewsApproved` passed at round 3. Every one of
  those findings is already recorded here (Deviation 2, the second
  `configFromEnv()` note, As-built drift 3), so the as-built record stands — but
  the earlier wording made it look as though no reviewer had objected.
- **Provenance pinned to two commits.** The 002 code/manifest/README change set
  is exactly `8b4bc52` (`test:` — the two new suites, authored pre-implement)
  and `6a71e4b` (`feat: add configurable reply timeout to orchestration CLI` —
  `src/cli.ts`, `src/opencode-runner.ts`, `package.json`, `package-lock.json`,
  `README.md`). No later commit in the range touches those five paths, and
  `tests/opencode-runner.test.ts` appears in no commit at all (Test #1's "no
  edits" claim confirmed mechanically a second way).
- **Out-of-scope file list completed.** Round 3 listed only
  `src/pipeline/{gates,runner,prompts}.ts` plus "pipeline-adjacent tests". The
  range's non-002 content is those three files **plus**
  `tools/orchestration/prompts/doc-sync.md` (new doc-sync prompt template),
  `src/pipeline/stages.ts` (wires that template), `tests/pipeline-gates.test.ts`,
  `tests/pipeline-prompts.test.ts`, and the run-record artifacts in the design
  directory (`.pipeline.json`, `transcripts/`, `reviews/`, and `ESCALATION.md`,
  which exists at HEAD and is deleted in the working tree — the runner clears it
  when a run resumes), and one agent-config rename outside `tools/`,
  `.opencode/agents/codereview/code-complexity.md` →
  `code-compatibility.md` (78% similarity; the compatibility reviewer's agent
  definition — its round-1 stage recorded zero model turns, model `unknown`, an
  empty reply 27 ms after starting, and no `reviews/code/compatibility-r1.md`
  report). None of it is design 002
  output, so the proposal's
  "any change to `src/pipeline/`" non-goal still holds for 002's change set.
- **Why the run itself stopped.** `.pipeline.json` ends with
  `"status": "escalated"`, `codeBudgetUsed: 3`. Every escalation entry is a
  harness reply-format failure (`invalid-reply: not JSON` from research,
  proposal-revise, design-review:correctness, test-authoring,
  code-review:compatibility, code-review:testing, doc-sync) — not a design or
  code problem. The same range's harness commits address it (nudge-then-retry,
  JSON extraction from prose, verdict/severity synonym normalisation, and the
  new `doc-sync.md` template), which is why the two prior doc-sync attempts
  recorded their findings but never emitted the verdict JSON this stage needs.

Doc-sync verdict for this pass: `drifted: true` — the same three minor drift
items, unchanged and already documented; no new drift, no design decision
touched, and the acceptance gate is now evidenced rather than outstanding.
Suggested commit message: `docs: close 002-add-timeout-flag build-note gaps`.

## Round 5 (doc-sync completion, 2026-11-08)

Fourth doc-sync attempt over the same range
(`git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`, `baseRef` confirmed
in `.pipeline.json`). The earlier As-built notes were **verified against the
shipped source rather than re-narrated**: drift items 1–3 remain the complete and
only drift set; one of them was found understated and is completed below; two
Round 4 statements are corrected; and Deviation 1 is closed with the run summary
it was waiting for. No design decision was changed.

**Re-verified against source at HEAD** (by line, independent of the earlier
notes): `DEFAULT_REPLY_TIMEOUT_MS`/`MAX_REPLY_TIMEOUT_MS` exported and
`POLL_INTERVAL_MS` untouched (`opencode-runner.ts:25, 28, 31`); the
`PromptOptions.timeoutMs?` docblock contract (`:123-137`); the fail-closed
`TypeError` before `createSession` (`:240-248`); `waitForReply(client,
sessionID, timeoutMs)` keeping wall-clock arithmetic and the check-then-deadline
loop (`:285-307`); `ReplyTimeoutError`'s designed message byte-for-byte with
`timeoutMs / 1000` and the old string as an exact prefix (`:346-351`);
`EnvValues`/`CliConfig`/`CliArgs.timeout?` (`cli.ts:27-56`);
`resolveReplyTimeoutSeconds` with the `^\d{1,5}$` gate before any `Number()` and
a separate range check deriving `30`/`3600` from the runner constants
(`:101-137`); the `timeout` option with the designed `describe` and **no**
`default:` (`:167-174`); the resolver running before `createClient()` and before
any output (`:182-200`); the banner extended by only " waiting up to <n> s for
the reply" (`:203-206`); `timeoutMs: timeoutSeconds * 1000` at the call site
(`:211-216`); README row and Behaviour item 3 as specified (`README.md:22,
35-37`); `package.json:30-32` `engines` and its 3-line `package-lock.json`
mirror. Provenance re-confirmed: `8b4bc52` (the two new suites) and `6a71e4b`
(every 002 source/manifest/README file) are the only commits in the range
touching those paths, and `tests/opencode-runner.test.ts` appears in no commit.

**Completed — drift item 2 was understated.** The As-built record called
`this.name = 'ReplyTimeoutError'` (`opencode-runner.ts:352`) "behaviourally
additive only". It is a **second observable compatibility delta**: `error.name`
becomes `'ReplyTimeoutError'` for consumers that branch or log on `name`, and the
CLI's stderr first line gains the class-name prefix, breaking scrapers keyed on
`Error: …`. `code-review:compatibility` graded it minor in r2 and carried it open
in r3 (finding 1) because the proposal's Compatibility matrix ("Existing API
consumers" row) and Rollout #3 still name the extended message as the *single*
delta. **No `fix` stage ever ran**, so the shipped tree keeps the assignment.
Recorded as built in the proposal's As-built addendum; the matrix row is design
text and was left untouched.

**Deviation 1 closed with the summary the r1 major asked for** — two separate
executions, both green:
- Harness gate: `gateLog` → `{"gate": "testsGreen", "round": 0, "proceed": true,
  "reason": "npm run check green"}` at `2026-11-08T06:09:48.887Z`, 2.8 s after
  the implement stage's reply at `06:09:46.063Z`. Caveat from correctness-r3
  finding 1: that entry predates the branch's later harness commits and the gate
  discards child output, so it records only a verdict.
- Reviewer-executed at HEAD: `code-review:testing` r3
  (`reviews/code/testing-r3.md:80-99`) and `code-review:compatibility` r3
  (`reviews/code/compatibility-r3.md:13-24`) ran `npm run check` in
  `tools/orchestration`: ESLint clean, `prettier --check .` → "All matched files
  use Prettier code style!", `vitest run` v5.0.5 → **Test Files 7 passed (7),
  Tests 91 passed (91)**, Duration 857 ms, on Linux. The unedited guard at
  `tests/opencode-runner.test.ts:243-257` re-ran green, so Goal 4 and Test
  strategy #12's Linux gate are evidenced at the merged tree; macOS/Windows stay
  unexercised by decision (no CI is a non-goal). Still uncaptured: the Node
  version of that run (`node -v` refused).

**As-built behaviour the design left open (refines A8, not drift).** With yargs
17.7.3 as installed, a valueless `--timeout` parses to `undefined`, so
`resolveReplyTimeoutSeconds` treats it as "flag absent" and silently falls back
to env → 30 s; a junk value parses to `NaN` and a fractional one (`2.5`) to a
non-integer, and both are rejected by the range check. Measured in this pass with
`node --input-type=module` against the package's own `node_modules` (Node
v26.13.0 — probe runtime only, not the gate's). A8's "invalid input arrives as a
non-finite value" holds for bad values, not for a missing one; neither case is
pinned by a test, and the fallback is always inside the validated range.

**Corrections to Round 4** (its other findings stand):
- "the run summary … is still absent" — superseded by the reviewer-executed runs
  above; the summary now sits in the review reports and in Deviation 1's status.
- "Every escalation entry is a harness reply-format failure (`invalid-reply: not
  JSON` from research, proposal-revise, design-review:correctness,
  test-authoring, code-review:compatibility, code-review:testing, doc-sync)" —
  imprecise. `gateLog` holds exactly five `escalate` entries: proposal-revise,
  test-authoring, `code-review:compatibility` and doc-sync (all `invalid-reply:
  not JSON`) plus `code-review:testing` (`invalid-reply: Invalid option: expected
  one of "approve"|"request-changes"`). Not escalations: `research`
  (`steps-exhausted`, cap 15), `design-review:correctness` ×2, `implement`
  (`invalid-reply: not JSON`), and the newest entry `doc-sync: steps-exhausted
  (cap 25)` at `2026-11-08T07:05:10.528Z` — which, not a reply-format failure, is
  why this stage re-ran.
- "`.pipeline.json` ends with `"status": "escalated"`" — true when written; on
  resume the runner rewrote it to `"status": "running"` (`codeRound: 3`,
  `codeBudgetUsed: 3`, `lastStage: code-review`, `testsGreen: true`) and deleted
  `ESCALATION.md` from the working tree, where it still exists at HEAD.

**Recorded and deliberately not acted on** (open in the r2/r3 reviews; no `fix`
stage exists, so the source never changed in response): the precedence-dependent
env **range** check (Deviation 2 — correctness-r1 f2, testing-r3 f2:
`resolveReplyTimeoutSeconds(5, '3601')` returns `5`, unpinned by any test);
`configFromEnv()` read twice per run (correctness-r1 f5, performance-r2/r3 F1);
the format-gate error also quoting `1..3600` (correctness-r1 f4, drift item 3);
`JSON.stringify`-only reflection leaving DEL, C1 controls and printable non-ASCII
raw (compatibility-r3 f5); `ReplyTimeoutError`'s suffix naming CLI-only knobs to
library consumers — design-mandated text (proposal:72), compatibility-r3 f6; the
Risks-row cost model underpricing each poll's full `message.list` payload
(performance-r3 F3); `cli.test.ts` not hermetic against an ambient
`OPENCODE_REPLY_TIMEOUT_SECONDS` (testing-r3 f1, corrected above); the extended
banner and `error.name` unasserted (testing-r3 f6); CLI↔runner cap coupling
untested at `3600` → `3_600_000` (testing-r3 f10); Test strategy #1 and #12
carrying no `// proves:` citation (testing-r3 f4). Out of 002's change set but
inside the reviewed range: `src/pipeline/{gates,runner,prompts,stages}.ts`,
`tests/pipeline-*.test.ts`, `prompts/doc-sync.md` and the
`.opencode/agents/codereview/code-complexity.md → code-compatibility.md` rename —
range hygiene per correctness-r1/r3, security-r3 and compatibility-r3, none of it
touching the timeout path.

Doc-sync verdict for this pass: `drifted: true` — the same three minor drift
items, re-verified against source, with item 2's record completed and the
acceptance gate now evidenced by an executed `npm run check`. No design decision
changed. Suggested commit message: `docs: complete 002-add-timeout-flag as-built
record and gate evidence`.

## Round 6 (doc-sync re-verification, 2026-11-08)

The run's fifth doc-sync attempt (fourth recorded doc-sync pass) re-verified the
As-built record against a fresh checkout of the branch instead of duplicating
it. The shell — `git log`, `git diff`, `npm` alike — is refused in this
environment too, so verification is line-by-line against the checked-out tree
and the committed `.pipeline.json`, not a fresh replay of the
`fdfd2bc280482e9a878429f197619d5e794a075d...HEAD` range (`baseRef` confirmed at
`.pipeline.json:1249`).

- **The tree has not moved since Round 5.** Every anchor quoted there resolves
  to the same lines: `opencode-runner.ts:25, 28, 31, 123–137, 240–248, 285–307,
  346–351` (`this.name` at `:352`); `cli.ts:27–56, 101–137` (format gate `:108`),
  `167–174, 182–200` (second `configFromEnv()` at `:188`), `203–206, 211–216`;
  `README.md:22, 35–37`; `package.json:30–32` with the 3-line
  `package-lock.json:11–13` mirror; the unedited guard at
  `tests/opencode-runner.test.ts:243–257`; the `within 4 s` / `within 0.1 s` /
  fail-closed-value loop (`opencode-runner-timeout.test.ts:140`) /
  `vi.mock(…, importOriginal)` (`cli.test.ts:29–30`) anchors in the two new
  suites.
- **Drift set confirmed complete and unchanged:** As-built items 1–3 only
  (lockfile `engines` mirror; `ReplyTimeoutError.name`, already carried with its
  second-observable-delta completion; resolver wording additions at
  `cli.ts:111/:119/:130`). Deviations 1–5 statuses are all still accurate. Gate
  evidence re-anchored: gateLog `testsGreen` entry ("npm run check green",
  06:09:48.887Z) plus the reviewer-executed `npm run check` at
  `reviews/code/testing-r3.md:84–90` and `reviews/code/compatibility-r3.md:16–17`
  (ESLint + prettier clean, 7 files / 91 tests, vitest 5.0.5, Linux).
- **Clarification to the "why the run stopped" record** (harness bookkeeping,
  not 002 drift): this checkout's committed `.pipeline.json` ends at the run's
  terminal escalation — `status: "escalated"`, `updatedAt`
  2026-11-08T07:11:34.162Z, reason `stage doc-sync failed twice
  (steps-exhausted (cap 25))` — and `ESCALATION.md` is absent from this tree.
  Round 5's rewrite-to-`running` / "still exists at HEAD" described the runner
  worktree's transient uncommitted state, which never reached the committed
  file. The gateLog holds **six** `escalate` entries, not the five Round 5
  counted: the sixth is that terminal doc-sync escalation, which follows the
  07:05:10.528Z `steps-exhausted` stageFailure Round 5 quoted as the newest
  entry.
- The provenance claim (002 content is exactly `8b4bc52` + `6a71e4b`) could not
  be re-derived without git in this pass; it stands on the Rounds 4–5
  mechanical verification.

**Nothing remains open in the As-built audit.** The only unresolved items are
the review minors/nits listed under "Recorded and deliberately not acted on"
above; no gate requires fixing them (`allCodeReviewsApproved` passed at round 3,
no `fix` stage exists, and the merged tree is the implement stage's output).

Doc-sync verdict for this pass: `drifted: true` — the same three minor drift
items, verified against source at HEAD; no new drift, no record gap left, no
design decision touched. Suggested commit message: `docs: re-verify
002-add-timeout-flag as-built record at HEAD and close the audit`.

## Round 7 (doc-sync re-verification, 2026-11-08)

Sixth doc-sync attempt over the same range
(`git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`). As before, this
environment refuses **every** shell invocation (`git log`, `git diff`, and
`npm` all returned `Permission denied: shell`), so the earlier As-built notes
were re-verified line-by-line against the checked-out tree and the committed
`.pipeline.json` instead of a fresh hunk replay.

- **Drift set re-confirmed complete and unchanged — As-built items 1–3 only,
  verified against source, not re-narrated.** Item 1: `package.json:30–32`
  `engines: { node: ">=22.12.0" }` with the 3-line `package-lock.json:11–13`
  root-package mirror. Item 2: `this.name = 'ReplyTimeoutError'`
  (`opencode-runner.ts:352`) on top of the designed `super(...)` message
  (`:346–351`), carried with its Round 5 "second observable delta" completion.
  Item 3: resolver wording additions at `cli.ts:111` `(plain digits only)`,
  `:119` and `:130` `(unbounded waits are not supported)`.
- **All previously recorded source anchors resolve unchanged in this
  checkout:** `opencode-runner.ts:25, 28, 31, 123–137, 240–248, 285–307,
  346–351, 352`; `cli.ts:27–56, 101–137` (gate at `:108`), `167–174` (no
  `default:`), `182–200` (second `configFromEnv()` at `:188`), `203–206,
  211–216`; `README.md:22, 35–37` (the untouched pre-existing session-title
  drift, quoted at README:28 in the base tree, now sits at `:29` because the
  `--timeout` row added one line to the table above it); the unedited guard at
  `tests/opencode-runner.test.ts:243–257` (substring at `:253`);
  `opencode-runner-timeout.test.ts:140` (fail-closed value loop), `:170–171`
  (two-check contract + `within 0.1 s`); `cli.test.ts:29–30`
  (`vi.mock(…, importOriginal)`), `:172–179` (first `main()` test — the one
  the ambient-env hazard note points at) and `:204–211` (30 s default case).
  Deviations 1–5 and the Round 5 corrections all still stand as written.
- **Gate evidence re-anchored unchanged:** `reviews/code/testing-r3.md:80–99`
  and `reviews/code/compatibility-r3.md:13–24` — reviewer-executed
  `npm run check` at HEAD (ESLint clean, prettier clean, vitest 5.0.5,
  7 files / 91 tests, Linux; Node version not recorded, `node -v` refused).
- **Run-log refresh — supersedes Round 6's description of the committed
  `.pipeline.json`.** The committed run record has moved since Round 6 was
  written: it now holds `"status": "running"` (`:10`, the runner's resume
  rewrite), `lastStage: code-review`, `codeRound: 3`, `codeBudgetUsed: 3`,
  `testsGreen: true`, and its `gateLog` ends with a new trailing entry —
  `stageFailure` round 3, `doc-sync: invalid-reply: not JSON`
  (`:1254–1260`), at `1794122118194` = 2026-11-08T**07:15:18.194Z**, i.e. a
  doc-sync attempt that failed on reply format *after* the terminal escalation
  at `:1248` (07:11:34.162Z) Round 6 quoted as the end. `updatedAt` is that
  same timestamp (`:1263`), `baseRef` remains
  `fdfd2bc280482e9a878429f197619d5e794a075d` (line-shifted to `:1264` by the
  appended entries), the `gateLog` still holds exactly six `escalate`
  entries, and `ESCALATION.md` remains absent from this tree. This is harness
  bookkeeping from the run's resumed doc-sync attempts, not 002 drift; the
  002 source, manifest, README, and test files are untouched by it (and the
  two-commit provenance claim, `8b4bc52` + `6a71e4b`, remains un-re-derivable
  without git and stands on the Rounds 4–5 verification).
- **No 002 drift or record gap found by this pass; no deviation added,
  amended, or removed.** The proposal's As-built section and its addenda
  carry the same three items, verified rather than duplicated; the As-built
  audit remains closed, with only the listed review minors/nits open by
  decision (no `fix` stage exists, `allCodeReviewsApproved` passed at r3).

Doc-sync verdict for this pass: `drifted: true` — the same three minor drift
items and nothing else, re-verified line-by-line at this checkout; the only
delta since Round 6 is the harness run-log state refreshed above. Suggested
commit message: `docs: re-verify 002-add-timeout-flag as-built record and
refresh run-log state`.
