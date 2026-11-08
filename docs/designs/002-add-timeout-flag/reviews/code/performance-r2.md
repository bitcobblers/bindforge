# Code review — performance (round 2)

- Diff range: `fdfd2bc280482e9a878429f197619d5e794a075d...HEAD`
- Design judged against: `02-proposal.md` (r4) + `03-build-notes.md`
- Scope of this review: the code this run produced — `869e633` (tests) and
  `0d7762a` (feat) — i.e. `src/opencode-runner.ts`, `src/cli.ts`,
  `package.json`, `README.md`, `tests/cli.test.ts`,
  `tests/opencode-runner-timeout.test.ts`.
- **Verdict: approve** (0 blocker, 0 major, 0 minor, 1 nit)

## Checklist results

1. **Wait/startup path — no sync fs or network, no per-call rebuilds, no
   avoidable allocations.** PASS with one nit. The hot-ish path (the 250 ms
   poll loop, `opencode-runner.ts:292–304`) is byte-for-byte the pre-existing
   shape plus a `timeoutMs` parameter — no new allocations, no fs/network
   added anywhere in the change. `resolveReplyTimeoutSeconds`
   (`cli.ts:101–138`) is a pure O(1) function (one regex test on a ≤5-char
   value, arithmetic); the `^\d{1,5}$` literal and the `CliConfig`/`EnvValues`
   bags are created once per process. Nit: the env bag is built twice per
   run — see F1.
2. **Complexity — no unbounded O(n²), maps/sets where identity lookup was
   intended.** PASS. The poll loop cost is O(timeoutMs / 250) and the design
   deliberately bounds it: `MAX_REPLY_TIMEOUT_MS = 3_600_000`
   (`opencode-runner.ts:31`) is enforced by the CLI resolver
   (`cli.ts:116`) *and* the runner boundary `TypeError`
   (`opencode-runner.ts:241–245`), capping the loop at ≤ 14,400
   `message.list` polls (3_600_000 / 250 = 14,400 exactly; ~121 at the 30 s
   default). No list→map conversion opportunities were missed — there is no
   repeated identity lookup anywhere in the change.
3. **Unbounded growth — caches/arrays/registries bounded or documented.**
   PASS. No cache, array, or registry was added. The one growth axis the
   change opens (poll iterations / nested pending promise frames in the
   recursive `poll()` at larger timeouts) is bounded at both boundaries
   exactly as the proposal's Risks row and A16 prescribe, and the
   `timeoutMs: 3_600_001` case is pinned as rejected
   (`opencode-runner-timeout.test.ts`, Test #5). Converting the recursion to
   a loop was explicitly "deliberately not changed" by the design (the frames
   are bounded by the same cap); I do not re-litigate it.
4. **Startup — boot parses/reads only what boot needs.** PASS, and
   deliberately so: the resolver runs before `createClient()` and before any
   `console.log` (`cli.ts:186–189`), so a config error emits zero HTTP and
   zero output — pinned by `cli.test.ts:215–232` (Test #11). The yargs
   option declares no `default:`, so no env coercion happens before the
   validator (`cli.ts:167–174`).
5. **Cache invalidation covered by a test.** N/A — evaluated, not
   unexamined: the change adds no cache or invalidation logic anywhere
   (grep of the diff: no memo/map/set structures beyond the pre-existing
   loop).
6. **Benchmark-like tests / timing notes — run and quote.** PASS, run:
   `npm test` in `tools/orchestration` → **7 files, 87 tests passed,
   Duration 701 ms**. This includes the fake-timer cases pinning the timing
   model: `timeoutMs: 4_000` → `within 4 s`, the no-option case staying
   unsettled at 29 s and rejecting with `within 30 s`, `timeoutMs: 100`
   performing exactly **two** `message.list` calls (Test #6's ≤ one-interval
   overshoot), and the five fail-closed `TypeError` cases with zero client
   calls — all finish in wall-clock milliseconds, proving the suite never
   really waits for the 30 s default. The design's ceiling figure (~14,400
   polls) is exact arithmetic on the shipped constants (3_600_000 / 250 =
   14,400; 30_000 / 250 + 1 = 121 baseline) and matches the Risks row; a
   live memory measurement of the ~14,400 pending frames could not be taken
   because this environment rejects ad-hoc `node`/`npx` invocations — the
   bounded-growth argument above stands on the enforced cap and the pinned
   rejection tests instead.

## Findings

### F1 — nit — `tools/orchestration/src/cli.ts:188`
**Problem:** `main()` calls `configFromEnv()` a second time (line 188) to get
`replyTimeoutSeconds`, although `parseArgs()` already built a fresh
`EnvValues` bag from `process.env` moments earlier (line 144, inside
`parseArgs`). The env is read and the bag allocated twice per process. Startup-only, one-time, negligible in practice —
hence nit, not minor.
**Fix:** hoist one bag: `const env = configFromEnv();` at the top of `main()`
and thread it into `parseArgs(argv, env)` (or read
`process.env.OPENCODE_REPLY_TIMEOUT_SECONDS` directly in `main()` if keeping
`configFromEnv()` single-purpose matters more).

## FYI (no severity, outside this specialty / outside this run)

- The diff range also carries `src/pipeline/gates.ts`, `prompts.ts`,
  `runner.ts`; those come from earlier branch commits (`f0a2e16`, `891567b`,
  `d7be29f`), not from this design run, and are an explicit non-goal of the
  proposal. I still gave them a performance scan: `extractJsonObject` is a
  single linear pass (candidates are non-overlapping), `verdictsFor` is
  O(specialties × results) with tiny n, and `enforceOwnership` runs a handful
  of git subprocesses per stage only when ownership is violated — no
  performance issues found there either.
- The build notes record that the implement stage could not run
  `npm run check`; in this review environment `npm test` (and therefore the
  vitest gate) does run, and it is green (87/87 in 701 ms). ESLint/Prettier/
  `tsc` still deserve a run by the merger.
