# security review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`, r1). Checklist: `references/security.md` only.

## Findings

- [major] Chosen design → `src/opencode-runner.ts`, 4th bullet (`const timeoutMs =
  options.timeoutMs ?? DEFAULT_REPLY_TIMEOUT_MS`) — the validation lives only in the CLI
  resolver, but `timeoutMs` arrives through the **exported** `PromptOptions` of the published
  `@bindforge/orchestration` API (the Compatibility matrix's "Existing API consumers" row). `??`
  only catches `null`/`undefined`, so a non-CLI caller can pass `Number.POSITIVE_INFINITY`,
  `NaN`, `0` or a negative value straight into the wall-clock deadline: `Date.now() + Infinity`
  and `Date.now() + NaN` both make `Date.now() >= deadline` permanently false, so `waitForReply`
  polls `message.list` every 250 ms **forever** — a fail-open, unbounded request loop and hung
  process, which is exactly what checklist item 7 forbids. The design's own huge-value safety
  argument (A9, A16, Risks row 4) assumes the resolver's `1..86_400` guard, which never runs for
  API consumers. Fix: make `runPrompt`/`waitForReply` fail closed — normalise `options.timeoutMs`
  (reject non-`Number.isFinite`, non-integer, `< 1`, or `> 86_400_000` with a `TypeError`, or
  clamp to a documented maximum) and state in the `PromptOptions` doc comment that the cap is
  enforced at the API boundary, not just at the CLI. Add a test: `timeoutMs: Infinity` and
  `timeoutMs: NaN` must not produce an endless poll loop.
- [major] Chosen design → `src/cli.ts`, 3rd bullet (`resolveReplyTimeoutSeconds` "throws errors
  that name the offending value and its source") + Data flow ("validation errors print and exit
  1") — the raw `OPENCODE_REPLY_TIMEOUT_SECONDS` string is reflected verbatim into stderr by the
  module-level `console.error(error)` (tools/orchestration/src/cli.ts:121, which prints message
  *and* stack). Env is untrusted input in practice (CI config, container image, sourced `.env`),
  so a value such as `OPENCODE_REPLY_TIMEOUT_SECONDS=$'\x1b[2J\x1b[31m…'` or one containing
  newlines/CR converts to `NaN` and is echoed back with live ANSI/control characters — terminal
  escape injection and spoofed/piloted CLI and CI-log output. Fix: never interpolate the raw
  string; validate against `^\d{1,5}$` (or `JSON.stringify()` the value) before quoting it, and
  say so in the design so the implementation cannot regress it.
- [minor] Chosen design → "Data flow" — no source is classified by trust (checklist item 1).
  argv and env are treated as "the operator's" without stating that both can be attacker-
  influenced (CI-provided env, argv from a wrapper script), and the proposal silently edits the
  network-facing `waitForReply`/`runPrompt` path without repeating that the OpenCode server
  response stays UNTRUSTED there (`parseContent` JSON-parses it; `main()` prints it). Fix: add
  one line to Data flow classifying argv/env as untrusted operator input and the server reply as
  untrusted network input, and note which validation covers each.
- [minor] Chosen design → `src/cli.ts`, 3rd bullet ("converts the env string",
  `Number.isInteger` as the validator) — `Number(envValue)` is far looser than "integer seconds":
  `"0x2d"` → 45, `"1e4"` → 10000, `" 45\n"` → 45, `"+45"` → 45 all pass, so the accepted value
  silently differs from what the user wrote and from the value the error message reports
  (checklist item 2). Fix: gate the conversion on an explicit `^\d{1,5}$` (or `^\+?\d{1,5}$`)
  regex before `Number()`, and keep the regex rejection separate from the range rejection in the
  message.
- [minor] Options considered → "Value semantics" + Risks row 4 (86 400 s bound) — the bound is
  sized only as a typo guard and the impact model is "a CI job hangs for hours". With the poll
  interval fixed at 250 ms, a valid `--timeout 86400` (or a hostile CI env value) issues up to
  ~345 600 `message.list` requests to whatever `OPENCODE_BASE_URL` names — sustained request
  pressure, not merely a slow job (checklist item 6, least capability). Fix: justify the cap from
  an abuse standpoint (or lower it, e.g. 3 600 s) and add a max-poll-attempts/total-requests note
  to the design so the loop's cost is bounded independently of the deadline.
- [minor] Chosen design → `src/cli.ts`, 4th bullet ("extends the startup log (line 107)") —
  line 107 already prints `config.baseUrl` unredacted, so a `--base-url
  http://user:token@host` (the env-var twin is `OPENCODE_BASE_URL`) prints credential material
  on every run; the design adds text to that same line without stating the logging rule for it
  (checklist item 4). Fix: state explicitly that the extended banner logs only the resolved
  duration and that any printed URL is redacted of userinfo; keep the "no password provided"
  hint wording as-is so `OPENCODE_SERVER_PASSWORD` is never echoed.
- [nit] Chosen design → `src/cli.ts`, 4th bullet + Data flow — the order of "resolve the
  timeout" relative to `createClient()` (tools/orchestration/src/cli.ts:101) and the startup
  banner (107) is not pinned, so an implementation could emit the success-shaped banner and build
  the auth-header client before rejecting the config. Fix: say the resolver runs first, before
  `createClient` and before any output, so a config error produces exactly one error line and no
  side effects.

## FYI (no severity)

- Dependency check (checklist item 5) came back clean and is not a blocker: no new dependencies
  are introduced. `yargs@17.7.3` resolves `yargs-parser@^21.1.1`, above the patched 18.1.1 for
  GHSA-p9pc-299p-vxgp (CVE-2020-7608 prototype pollution); GitHub's advisory DB reports no
  advisories for `yargs` or `@opencode/client`; the lockfile's `vitest`/`@vitest/mocker@5.0.5`
  are outside the vulnerable ranges of GHSA-82fw-gwwq-j7x9 (CVE-2026-84373), GHSA-5xrq-8626-4rwp
  and GHSA-9crc-q9x8-hgqq, and `npm test` runs `vitest run` (no UI/API server), so none of the
  Vitest dev-server issues apply. The Rollout gate is `npm run check` only; a dependency-audit
  step is arguably worth adding, but that is release-process, not this design.
- Path handling (checklist item 3) and new privileges (item 6) were assessed: the change set
  introduces no filesystem paths, no shell interpolation, and no new device/filesystem
  capability — nothing to fix.
- Pre-existing, outside this change set, for the correctness/compatibility reviews: `--password`
  on argv is visible in the process list; `parseContent` JSON-parses untrusted reply text with no
  size limit; `describeError` falls back to `JSON.stringify(error)`, which can serialise error
  objects (including anything the SDK attaches) into the printed message.
- The `1..86_400` vs `0`-means-unbounded decision (A14) is a UX/compat question for the
  compatibility reviewer; from a security standpoint rejecting `0` is the correct fail-closed
  choice and should stay.
