# correctness review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`). Checklist: `references/correctness.md`.

Verification performed: every `verified` row of the assumption register was
checked against the cited code (`src/opencode-runner.ts`, `src/cli.ts`,
`src/pipeline/stages.ts`, `src/pipeline/runner.ts`, `tests/opencode-runner.test.ts`,
`README.md`, `package.json`). Line references in A1–A3, A5, A7, A10–A13, A15
all land on the claimed code. Four research URLs were spot-checked live:
`curl.se/libcurl/c/CURLOPT_TIMEOUT.html` (default `0` = never times out, long
seconds, last-set-wins — supports A6/research),
`postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNECT-CONNECT-TIMEOUT`
("in seconds (write as a decimal integer…); Zero, negative, or not specified
means wait indefinitely" plus the unset → env → built-in-default precedence —
supports A6, A14, research line 24),
`nodejs.org/api/timers.html#settimeoutcallback-delay-args` ("When `delay` is
larger than `2147483647` or less than `1` or `NaN`, the `delay` will be set to
`1`" — supports A9 verbatim) and `vitest.dev/api/expect` (string argument to
`toThrow` = "error message includes the substring" — supports A5). No dead or
contradicting sources. The load-bearing green-test claim holds against the real
test: `tests/opencode-runner.test.ts:253` uses `.rejects.toThrow('No response
received from OpenCode')`, a substring assertion, and `waitForReply` is
module-private, so the A5/A1/A11 chain is sound.

## Findings

- [major] Assumption register A17 + Test strategy #8–#9 — A17 is marked
  `verified`, but its cited basis (`tests/opencode-runner.test.ts:52–61`) is
  `captureFetch()`, a `vi.stubGlobal('fetch', …)` recorder that always returns
  `new Response('{}', { status: 500 })` and is only used to observe a single
  `session.create` request through a client the test builds itself. It never
  drives a successful round-trip, so it does not support the claim that
  "`main()` can be exercised end-to-end in tests without a real OpenCode
  server": that requires the real `@opencode/client` SDK (built inside `main()`
  via `createClient()`, with no injection seam) to decode success envelopes for
  `session.create`, `session.prompt` and `message.list` from hand-written stub
  bodies. As written, the only end-to-end proof that the flag survives argv
  parsing is not shown to be implementable. Fix: relabel A17 `unverified` and
  pick one concrete route — (a) `vi.mock('../src/opencode-runner.js')` so
  `main()`'s call is observed with the exact `timeoutMs` it passes (proves the
  argv → resolver → boundary path without the SDK), or (b) add a test seam
  (optional client/clientFactory parameter on `main()`) reusing the existing
  `stubClient()` helper, or (c) spike the exact stub response bodies the SDK
  codec accepts and record the result as the basis before claiming `verified`.
- [major] Chosen design → `src/cli.ts`, bullets 2 and 3 — the `CliConfig`
  interface is given two contradictory shapes for the same setting. Bullet 2
  adds a raw-string field `replyTimeoutSeconds?: string` to what `configFromEnv()
  returns` (today `configFromEnv(): CliConfig`, `src/cli.ts:50`), while bullet 3
  adds `timeoutSeconds: number` as a required `CliConfig` field that nothing
  populates: `configFromEnv()` supplies only the raw string and `main()`'s
  `const config: CliConfig = {…}` literal (`src/cli.ts:95–100`) keeps the
  resolved value in a local. The same value therefore appears under two names
  and two types in one struct, and the code as described does not typecheck.
  Fix: give the env bag its own type (`interface EnvValues { …; replyTimeoutSeconds?:
  string }` returned by `configFromEnv()`), keep exactly one `timeoutSeconds:
  number` on `CliConfig`, and have `main()` assign it from
  `resolveReplyTimeoutSeconds(...)` before the `× 1000`; state both type shapes
  explicitly in the section.
- [minor] Chosen design → `src/opencode-runner.ts`, bullet 5 (and "Missing edge
  cases") — the runner-side contract for out-of-range `timeoutMs` is left
  undecided, although `01-research.md:43` explicitly asks the designer to pin
  the sub-poll-interval behaviour down. Validation lives only in
  `resolveReplyTimeoutSeconds`, so a public-API caller of `runPrompt` passing
  `timeoutMs: 0`, a negative value, `NaN`, or anything below 250 gets one
  immediate `message.list` check, no sleep, `null`, and a `ReplyTimeoutError`
  reporting `within 0 s` (or `within -0.005 s`). Fix: state the runner contract
  in the `PromptOptions.timeoutMs` docblock — either reject non-finite or
  non-positive `timeoutMs` with a validation error, or explicitly document
  "`timeoutMs ≤ 0` means one immediate check then timeout" — and define how the
  message renders such values, plus a matching test case.
- [minor] Assumption register A6 — the row's human-facing-seconds claim cites
  `https://curl.se/libcurl/c/CURLOPT_TIMEOUT.html`, which documents the libcurl
  `CURLOPT_TIMEOUT` long-in-seconds option and never mentions `--max-time`; the
  flag name is unsupported by the cited basis. Fix: cite the curl command-line
  man page (`curl.1`, `--max-time`) for the flag, or drop the flag name from
  A6 and rely on `CURLOPT_TIMEOUT` + libpq `connect_timeout`, which the spot
  check confirms.
- [nit] Assumption register A5 — the citation anchor
  `https://vitest.dev/api/expect#tothrowerror` is stale: the current (v5.x)
  Expect page documents the matcher under `#tothrow` and marks `toThrowError`
  a deprecated alias. The substance (string argument = substring inclusion) is
  confirmed. Fix: cite `#tothrow` and, if worth noting, that the repo's test
  already uses `.rejects.toThrow` (`tests/opencode-runner.test.ts:253`).
- [nit] Assumption register A4 — the test call-site list is partial: `runPrompt`
  is also called at `tests/opencode-runner.test.ts:138, 147, 156, 168, 185, 198,
  211, 228, 238`. The conclusion (all call sites pass an `options` object, so an
  optional field is additive) still holds for every site. Fix: cite the full set
  or mark the list as representative.
- [nit] Chosen design → `src/opencode-runner.ts`, bullet 4 — the message
  template `within <n> s` does not define how `<n>` is derived from `timeoutMs`
  (`timeoutMs / 1000`), so non-multiple-of-1000 values reachable through the
  public API render as `within 4.5 s` while Test strategy #2 only pins `4 s`.
  Fix: state the formatting rule (divide by 1000, fixed rounding) so the
  message text and its tests are unambiguous.

## FYI (no severity)

- `README.md:28` says the session is titled "SDK connection test" while
  `SESSION_TITLE` is `Untitled Session` (`opencode-runner.ts:22`): pre-existing
  doc drift, untouched by this change set, but the Goals promise the README
  "stays truthful", so the implementer should leave it alone deliberately or
  fix it in a separate `docs:` commit.
- Rejecting `0`/negative and declining exit code 124 diverges from libpq,
  curl, and coreutils prior art. The proposal justifies both (bounded smoke
  test, existing exit-1 contract) and makes them visible in the README and the
  resolver error, so this reads as a deliberate, documented choice — worth a
  second look from the compatibility specialty.
