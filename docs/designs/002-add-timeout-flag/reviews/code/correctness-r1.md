# correctness review r1

verdict: request-changes

Scope: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`, judged against
`02-proposal.md` (r4) and `03-build-notes.md`. Checklist:
`references/correctness.md` (code correctness — faithful to the design).
Previous code-review round: none (this is r1), so checklist #7 has nothing to
re-verify.

Environment note: this review environment accepts only `git diff`/`git log`
shell invocations; `npm`, `npx vitest`, `tsc`, `eslint` and `prettier` were all
refused with `Permission denied: shell`. Everything below is therefore a
line-by-line static trace of source against the checked-in suites, not an
executed run. That limitation is itself finding 1.

## Checklist walk

1. **Components/interfaces exist, are wired, names/signatures match.** All
   proposal-named pieces are present and match literally:
   `DEFAULT_REPLY_TIMEOUT_MS = 30_000` / `MAX_REPLY_TIMEOUT_MS = 3_600_000`
   exported (opencode-runner.ts:27-31), `POLL_INTERVAL_MS = 250` untouched;
   `PromptOptions.timeoutMs?: number` with the full runner-side contract in its
   docblock (opencode-runner.ts:120-137); `runPrompt` resolves
   `options.timeoutMs ?? DEFAULT_REPLY_TIMEOUT_MS` (240) and throws `TypeError`
   naming the value before `createSession` (241-246);
   `waitForReply(client, sessionID, timeoutMs)` keeps the wall-clock deadline and
   `null`-on-expiry contract (285-307); `export class ReplyTimeoutError extends
   Error` in the Errors section with the message text the proposal specifies,
   old string as an exact prefix, no data properties (346-354);
   `EnvValues` / `CliConfig.timeoutSeconds` / `CliArgs.timeout?` (cli.ts:23-56);
   yargs `.option('timeout', { type: 'number', describe: … })` with **no**
   `default:` (167-174); exported `resolveReplyTimeoutSeconds(flagValue,
   envValue)` with the exact signature (101-104); `main()` runs the resolver
   after `parseArgs()` and before `createClient()`/any `console.log` (181-201)
   and calls `runPrompt(client, { agent, timeoutMs: seconds * 1000 }, …)` (211-
   216); `package.json` `engines.node` and the README row/Behaviour #3 rewording
   match the proposal's wording. Goal 5 (`README.md` truthful) ✓; the README:28
   session-title drift was correctly left alone (explicit non-goal).
   No dangling `POLL_TIMEOUT_MS` reference anywhere in the tree (grep).
2. **Assumption register honoured.** A1 (250 ms interval unchanged), A4
   (`options` field optional — all 13 existing `runPrompt` call sites in
   `opencode-runner.test.ts` are untouched and compile), A5 (message prefix kept
   verbatim), A7/A8 (validated env conversion; `Number.isInteger` catches yargs'
   non-finite output), A9 (deadline is wall-clock arithmetic, only the fixed 250
   ms `sleep()` uses a timer), A10 (ms naming inside, seconds at the CLI edge),
   A11 (`src/pipeline/` does not call `runPrompt`), A13 (still `console.error` +
   exit 1 via the module-level catch, cli.ts:221-228), A14/A16 (`1..3_600`
   enforced at both boundaries), A21 (`engines.node` added). No row was silently
   flipped.
3. **Deviations declared.** Declared and true: deviation 3 (`this.name =
   'ReplyTimeoutError'`, opencode-runner.ts:352 — no data property added),
   deviation 4 (message wording additions, single-line messages), deviation 5
   (`vi.mock` + `importOriginal` route worked, `extractReply`/constants flow
   through the real module — consistent with cli.test.ts:29-36), the
   package-lock `engines` mirror, and deviation 1 (gate not run) — see finding 1.
   Deviation 2 is real but see finding 2. One coverage gap in the notes:
   finding 3.
4. **Scope creep.** The design's own change set (`opencode-runner.ts`,
   `cli.ts`, `package.json`, `package-lock.json`, `README.md`, the two test
   suites) is exactly what the proposal asks for — nothing extra was added to
   the timeout path. See finding 3 for `src/pipeline/` content inside the
   reviewed range.
5. **Build-notes claims checked.** Verified true by tracing: the two-check
   contract for `timeoutMs = 100` (immediate check → deadline tested only after
   each reply check → one 250 ms sleep → second check → expiry → `within 0.1 s`)
   matches `tests/opencode-runner-timeout.test.ts:159-175`; the 4 s / 30 s fake-
   timer cases match (346-354 formatting gives `within 4 s` / `within 30 s`);
   the fail-closed values `Infinity`/`NaN`/`0`/`-5`/`3_600_001` all hit the
   `TypeError` before `createSession` so `create`/`prompt`/`list` stay at zero
   calls; `OPENCODE_REPLY_TIMEOUT_SECONDS=` (empty string) is rejected by the
   `^\d{1,5}$` gate before any `Number()` (cli.ts:108); `JSON.stringify`
   reflection holds (110-111, 129-130); `1`/`3600` accepted and `3601` is a
   range error, matching `tests/cli.test.ts:127-140`; the README table is
   prettier-stable at 99 chars/row (columns 12/32/45 = max cell widths,
   `printWidth: 100`). One claim needs qualification: finding 3.
6. **Error paths fail closed.** ✓ `runPrompt` rejects an invalid `timeoutMs`
   before any HTTP request (no unbounded poll), the resolver runs before
   `createClient()` and before any output (asserted by
   `tests/cli.test.ts:215-232`), and every rejection surfaces as one error line
   with exit 1.

## Findings

1. **[major]** `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` (and
   proposal Test strategy #12 / Rollout #1) — the design's only acceptance gate
   was never executed, and the change is therefore unverified. Both new suites
   (`tests/cli.test.ts`, `tests/opencode-runner-timeout.test.ts`) were authored
   before the implementation existed, deviation 1 records that `npm run check`
   (lint + format check + tests) could not run in the implementing environment,
   and nothing on the branch shows it ran afterwards. Goal 4 ("existing tests
   pass unchanged") and Rollout #1's merge gate are unevidenced. My static trace
   found no source/test mismatch (fake-timer arithmetic, mock shape through
   `vi.mock(..., importOriginal)`, single-line messages, `isMain` guard keeping
   the module-level `process.exit` out of the suites), so this is unverified
   rather than broken — but a checklist item I could not evaluate counts as a
   finding, not a pass. Fix: run `npm run check` in `tools/orchestration`, paste
   the summary (test counts, lint/format status) into `03-build-notes.md`, and
   fix any fallout before merge.
2. **[minor]** `tools/orchestration/src/cli.ts:125-134` — the env **range**
   check sits behind the flag branch, so `--timeout 5` with
   `OPENCODE_REPLY_TIMEOUT_SECONDS=3601` silently resolves to `5` and the broken
   environment is never reported. The proposal pins only the *format* gate as
   precedence-independent, so this is the letter of the design, but it
   contradicts the rationale written next to it (proposal:82 and the docblock at
   cli.ts:94-97: "a broken environment is a configuration error even when a
   valid flag overrides it") and the Risks row "env typo … hard-fails the CLI".
   Declared as build-notes deviation 2, hence minor, not a deviation finding.
   Fix: validate the env value fully (format **and** range) before the flag
   branch, then apply precedence; or keep the behaviour and pin it with a test
   (`resolveReplyTimeoutSeconds(5, '3601') === 5`).
3. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:46-48` —
   the notes' coverage claim ("No test files were edited; the three existing
   suites … were treated as the contract") does not describe the whole reviewed
   range. The range also carries `src/pipeline/gates.ts`,
   `src/pipeline/prompts.ts`, `src/pipeline/runner.ts`,
   `tests/pipeline-gates.test.ts` and `tests/pipeline-prompts.test.ts` from
   commits `f0a2e16` and `891567b` — a subsystem the proposal names as an
   explicit non-goal ("Any change to the pipeline (`src/pipeline/`)",
   proposal:17). Provenance points to operator infrastructure rather than this
   run's output: both commits are the oldest in the range, authored immediately
   after the base pin (`fdfd2bc`, "pin the pipeline branch fork point as the
   review diff base") and before the run's first docs commit, and their content
   (stage-ownership enforcement, idle-session nudges, retry-verdict supersession)
   is unrelated to the timeout design. Fix: one build-notes line recording those
   two out-of-scope commits, or re-pin the review base to `891567b` so the
   review diff contains only the design's change set. If those commits turn out
   to be the run's own output, this becomes a scope-creep finding at major
   (undeclared, non-goal area) rather than minor.
4. **[nit]** `tools/orchestration/src/cli.ts:110-111` — the format-gate message
   states the range ("must be an integer number of seconds in 1..3600 (plain
   digits only)") even though the failure it reports is a *format* violation; the
   proposal deliberately keeps the two rejections separate (proposal:82). A user
   with `OPENCODE_REPLY_TIMEOUT_SECONDS=30s` is told about a range they never
   violated. Fix: keep the format error format-only, e.g.
   `` `$OPENCODE_REPLY_TIMEOUT_SECONDS must be plain decimal digits (integer seconds), got …` ``,
   and leave the range text to cli.ts:129-130.
5. **[nit]** `tools/orchestration/src/cli.ts:182,188` — `configFromEnv()` is
   called twice per run (once inside `parseArgs()`, again for
   `replyTimeoutSeconds`), so the resolver validates a different read of the
   environment than the one yargs used as defaults. Harmless in practice, but
   the design's "env reads centralized" intent (A3) is cleaner with one bag.
   Fix: have `parseArgs(argv)` take or return the `EnvValues` bag and pass it to
   `resolveReplyTimeoutSeconds`.

## FYI (out of specialty, no severity)

The pipeline commits in the range carry two details a future reviewer may want:
`extractJsonObject(text: string): unknown | undefined` in
`src/pipeline/prompts.ts` is a redundant union (`unknown | undefined` collapses
to `unknown`), and `enforceOwnership` in `src/pipeline/runner.ts` adds an extra
`fix: revert …` commit on top of the stage commit whenever a violation was
committed mid-stage, which changes what `commitStage`'s ownership boundary looks
like in history. Neither touches the timeout path.
