# Code review — performance (round 3)

- Range: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`
- Design judged against: `02-proposal.md` (r4) + `03-build-notes.md` (incl. its
  "Round 2" section)
- Checklist: `.opencode/skills/code-review/references/performance.md` (only)
- Previous round: `reviews/code/performance-r2.md` — verdict approve, 0
  blocker/major/minor, 1 nit (F1, double `configFromEnv()`). Before that, the
  design-round performance reviews (`reviews/design/performance-r1.md`/`-r2`).
- **Verdict: approve** (0 blocker, 0 major, 1 minor, 2 nits)

## Environment note (bears on checklist item 6)

This review environment rejects **every** shell command: `git diff …`, `git log
--oneline`, and `npm test` in `tools/orchestration` each returned
`Permission denied: shell`. The range therefore could not be re-derived and the
test suite could not be executed, so the timing figure the checklist asks me to
run and quote is unavailable at HEAD. Per the review rules that item counts as a
finding, not a pass (F1). Everything below was verified by line-by-line reading
of the HEAD tree; the anchors match those the r3 correctness/security reports
cite, confirming the tree still matches what performance-r2 reviewed.

## Checklist results

1. **Event/hot path: no synchronous fs or network, no per-call rebuilds, no
   avoidable allocations.** PASS for the design's change set. The wait path
   (`opencode-runner.ts:285–307`) is the pre-existing 250 ms poll loop plus one
   `timeoutMs` parameter: no fs/network call added, no structure rebuilt per
   iteration, and `findCompletedReply` (`:313–328`) keeps its single
   `message.list` + `data.find` per poll. The new boundary check
   (`:240–246`) is O(1) arithmetic before any request. `resolveReplyTimeoutSeconds`
   (`cli.ts:101–137`) is a pure O(1) function — one regex test on a ≤5-char
   value; the `/^\d{1,5}$/` literal and the `CliConfig`/`EnvValues` bags are
   allocated at most once per CLI run. The only avoidable allocation is the
   second env bag (F2, nit).
2. **Complexity: no O(n²) over unbounded n; maps/sets where identity lookup was
   intended.** PASS. The loop is O(`timeoutMs` / 250) and the bound is enforced
   at both boundaries — CLI range check `1..3_600` (`cli.ts:116`, `:127`) and
   the runner `TypeError` for `> MAX_REPLY_TIMEOUT_MS` (`opencode-runner.ts:241`,
   `MAX = 3_600_000` at `:31`) — so the reachable ceiling is exactly
   3_600_000 / 250 = **14,400** polls (vs 30_000 / 250 + 1 = **121** at the
   unchanged default), matching the Risks row. No repeated identity lookup was
   missed: nothing in the change set converts a scan into a map/set, and nothing
   needed it. The recursive `poll()` (`:292–304`) holds at most ~14,400 tiny
   pending frames at the cap; bounded by the same constant, and the design names
   it "deliberately not changed" — not re-litigated.
3. **Unbounded growth (caches, arrays, registries).** PASS. The change set adds
   no cache, array, or registry. The one growth axis it opens — poll iterations
   and their pending frames at a larger `--timeout` — is capped at both
   boundaries and pinned as rejected (`timeoutMs: 3_600_001`,
   `tests/opencode-runner-timeout.test.ts:140–154`). One documentation gap in the
   cost model behind that cap: the ceiling is priced in *requests*, but each
   request re-fetches the whole `message.list` body (F3, nit).
4. **Startup: boot reads/parses only what boot needs.** PASS, and more so than
   before. The resolver runs immediately after `parseArgs()` and before
   `createClient()` and before any `console.log` (`cli.ts:182–189`, `:197`,
   `:203`), so a config error emits zero HTTP and zero output — pinned by
   `tests/cli.test.ts:215–232`. The yargs option declares no `default:`
   (`cli.ts:167–174`), so no env coercion work happens before the validator.
   Nothing new is parsed at import time; `DEFAULT_REPLY_TIMEOUT_MS` /
   `MAX_REPLY_TIMEOUT_MS` (`opencode-runner.ts:28`, `:31`) are two constants.
5. **Cache invalidation present and covered by a test.** N/A — evaluated, not
   unexamined: no cache or invalidation logic exists anywhere in the change set
   (grep of `cli.ts`/`opencode-runner.ts` finds no memo/map/set/register added by
   this diff; the only `Set` in `tools/orchestration/src` is the pre-existing
   `touched` set in `src/pipeline/runner.ts:102`, outside the design's change
   set).
6. **Benchmark-like test / timing note: run it and quote the number.** **Could
   not be evaluated at HEAD** → F1. The tree contains the timing claims
   (`within 4 s`, unsettled-at-29 s / rejected-at-31 s, exactly two `message.list`
   calls for `timeoutMs: 100`, and the five zero-request `TypeError` cases —
   `tests/opencode-runner-timeout.test.ts:67–175`), and every one of them is
   fake-timer-based, so the suite's own wall-clock cost is the number worth
   quoting. performance-r2 quoted **7 files / 87 tests / 701 ms**, but that run
   covered `0d7762a`/`869e633`; `reviews/code/correctness-r3.md:149–165` shows
   the run's only green gate entry (`at: 1794118188887`) predates the in-range
   `d7be29f` (`1794118457`), so no recorded timing evidence covers HEAD, and I
   could not produce new evidence here.

## Findings

### F1 — minor — `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` (and `:129-136`)
**Problem:** checklist item 6 is unverifiable at HEAD. The build notes still say
`npm run check` "could not be executed" and "must still be run by the caller",
and the only in-tree green record predates the last in-range commit
(`1794118188887` vs `d7be29f` at `1794118457`, per
`reviews/code/correctness-r3.md:149-165`). So none of the timing contracts the
change ships — the ≤ one-interval overshoot, the two-check `timeoutMs: 100`
behaviour, the 14,400-poll ceiling that justifies the 3 600 s cap (A16) — has
executed evidence at the tree being merged. Not blocking: r2 did run the suite
green against the design's own change set, the post-gate delta touches no
timeout code, and the arithmetic behind the ceiling (3_600_000 / 250 = 14,400;
121 at default) is exact on the shipped constants.
**Fix:** run `npm run check` in `tools/orchestration` at HEAD and paste the
summary block into `03-build-notes.md` (OS, `node -v`, per-suite test counts,
Duration, lint/format status); rewrite deviation 1 and the Round-2 paragraph to
cite that run. If the re-run is red, this escalates to a major.

### F2 — nit — `tools/orchestration/src/cli.ts:144` and `:188`
**Problem:** carried open from performance-r2 F1, still unaddressed: `main()`
calls `configFromEnv()` a second time inside the resolver call (`:188`) although
`parseArgs()` built a fresh `EnvValues` bag from `process.env` moments earlier
(`:144`). `process.env` is read and a five-field bag allocated twice per run, and
the resolver validates a second, independent env snapshot rather than the one
yargs used for the other defaults. Startup-only, one extra allocation per
process — hence nit.
**Fix:** hoist one bag — `const env = configFromEnv()` at the top of `main()` and
thread it into `parseArgs(argv, env)` (or read
`process.env.OPENCODE_REPLY_TIMEOUT_SECONDS` directly at `:188` if keeping
`configFromEnv()` single-purpose matters more).

### F3 — nit — `tools/orchestration/src/opencode-runner.ts:313–317` (cost model: `02-proposal.md` Risks row, A16)
**Problem:** the diff raises the reachable poll count on an unchanged path whose
per-poll cost is not constant. `findCompletedReply` fetches the session's **whole**
message list every 250 ms and `data.find` scans it, so the cost of a wait is
O(polls × list size), not O(polls). The Risks row and A16 price the ceiling in
requests only ("up to ~14,400 `message.list` polls"), which understates the
`--timeout 3600` worst case: 14,400 *full list bodies*, each larger than the last
as the session accumulates messages. Bounded and opt-in, and the loop shape is
pre-existing and explicitly "deliberately not changed" — so this is a
documentation gap in the cost model behind the 3 600 s cap, not a code defect.
**Fix:** one clause in the Risks row / A16 stating that each poll re-fetches the
entire `message.list` body (so the ceiling costs 14,400 list payloads, payload
growing with session history), and keep the already-named adaptive-poll-interval
follow-up as the remedy if long timeouts ever become normal.

## FYI (no severity, outside this specialty)

- The range also carries the pipeline group — `src/pipeline/gates.ts`,
  `prompts.ts`, `runner.ts`, `tests/pipeline-*.test.ts`,
  `.opencode/agents/codereview/code-compatibility.md` (commits `f0a2e16`,
  `891567b`, `d7be29f`) — an explicit non-goal of the proposal
  (proposal:17, :23). I gave it the same performance scan as r2:
  `extractJsonObject` (`prompts.ts:115–147`) stays linear (candidates are
  non-overlapping via `i = text.indexOf('{', close + 1)`, and an unbalanced `{`
  `break`s at `:140-141`, so no O(n²) scan), `verdictsFor`
  (`gates.ts:78–102`) is O(specialties × `state.results`) with n bounded by
  stages × rounds, the runner's stage poll loop always sleeps
  `cfg.pollIntervalMs` (2 s, `stages.ts:38`) before re-listing
  (`runner.ts:377–417`) — no tight loop — and `enforceOwnership`/`commitStage`
  spawn a handful of sync git subprocesses per stage, while `saveState` /
  `saveStateSync` re-serialize the whole `.pipeline.json` after each stage. All
  stage-level; no performance findings.
- `cli.test.ts`'s `afterEach` clears `OPENCODE_REPLY_TIMEOUT_SECONDS` only after
  the first `main()` test has run, so an ambient value in the shell running
  `npm test` can affect that first test (correctness-r3 finding 4). Test-hygiene,
  not performance; it does not change the suite's runtime.
- The build notes' "gate not executed" deviation is the same one F1 rests on;
  ESLint/Prettier/`tsc` still deserve their run by the merger.
