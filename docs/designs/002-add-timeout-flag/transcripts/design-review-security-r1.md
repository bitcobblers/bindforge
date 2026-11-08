# Transcript: design-review:security r1

## Provenance
- stage: design-review:security (round 1)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 143f0d087e11
- started: 2026-11-08T05:12:16.059Z
- ended: 2026-11-08T05:14:04.755Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/002-add-timeout-flag/02-proposal.md.
Apply only this change set: the whole document.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/002-add-timeout-flag/reviews/design/security-r1.md.
Reply with the JSON verdict object only.

Respond with exactly one JSON object that validates against this JSON Schema.
Output only the JSON object, with no markdown fences and no other text.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{
  "verdict": "request-changes",
  "findings": [
    {
      "severity": "major",
      "location": "Chosen design → `src/opencode-runner.ts`, 4th bullet (`const timeoutMs = options.timeoutMs ?? DEFAULT_REPLY_TIMEOUT_MS`)",
      "problem": "Validation exists only in the CLI resolver, but `timeoutMs` enters through the exported `PromptOptions` of the published `@bindforge/orchestration` API (the Compatibility matrix's \"Existing API consumers\" row). `??` only catches null/undefined, so an API consumer can pass `Number.POSITIVE_INFINITY` or `NaN`: `Date.now() + Infinity` and `Date.now() + NaN` both make `Date.now() >= deadline` permanently false, so `waitForReply` polls `message.list` every 250 ms forever — a fail-open unbounded request loop and hung process, contrary to checklist item 7. The design's own huge-value safety argument (A9, A16, Risks row 4) relies on the resolver's `1..86_400` guard, which never runs for non-CLI callers.",
      "fix": "Fail closed at the API boundary: in `runPrompt`/`waitForReply` reject non-finite, non-integer, `< 1`, or `> 86_400_000` `timeoutMs` with a `TypeError` (or clamp to a documented maximum), document the cap in the `PromptOptions` comment, and add tests that `timeoutMs: Infinity`/`NaN` cannot produce an endless poll loop."
    },
    {
      "severity": "major",
      "location": "Chosen design → `src/cli.ts`, 3rd bullet (`resolveReplyTimeoutSeconds` \"throws errors that name the offending value and its source\") + Data flow (\"validation errors print and exit 1\")",
      "problem": "The raw `OPENCODE_REPLY_TIMEOUT_SECONDS` string is reflected verbatim to stderr via the module-level `console.error(error)` (tools/orchestration/src/cli.ts:121, which prints message and stack). Env is untrusted in practice (CI config, container image, sourced `.env`), so a value such as `OPENCODE_REPLY_TIMEOUT_SECONDS=$'\\x1b[2J\\x1b[31m…'` or one with CR/LF converts to `NaN` and is echoed back with live ANSI/control characters — terminal escape injection and spoofed CLI/CI-log output.",
      "fix": "Never interpolate the raw string: validate it against `^\\d{1,5}$` before conversion or reflect it as `JSON.stringify(envValue)` so control characters are escaped, and state this in the design so the implementation cannot regress it."
    },
    {
      "severity": "minor",
      "location": "Chosen design → \"Data flow\"",
      "problem": "No data source is classified by trust (checklist item 1). argv and env are treated implicitly as \"the operator's\" without noting both can be attacker-influenced (CI-provided env, argv from a wrapper script), and the proposal edits the network-facing `waitForReply`/`runPrompt` path without stating that the OpenCode server reply remains untrusted network data there (`parseContent` JSON-parses it; `main()` prints it at cli.ts:114).",
      "fix": "Add one Data flow line classifying argv/env as untrusted operator input and the server reply as untrusted network input, naming which validation covers each."
    },
    {
      "severity": "minor",
      "location": "Chosen design → `src/cli.ts`, 3rd bullet (\"converts the env string\", `Number.isInteger` as the validator)",
      "problem": "`Number(envValue)` is much looser than \"integer seconds\": `\"0x2d\"` → 45, `\"1e4\"` → 10000, `\" 45\\n\"` → 45, `\"+45\"` → 45 all pass validation, so the effective value silently differs from what the user wrote and from the value quoted in the error message (checklist item 2).",
      "fix": "Gate the conversion on an explicit `^\\d{1,5}$` (or `^\\+?\\d{1,5}$`) regex before `Number()`, and keep regex-format rejection a distinct message from range rejection."
    },
    {
      "severity": "minor",
      "location": "Options considered → \"Value semantics for `0`, negative, fractional, and huge inputs\"; Risks row 4 (86 400 s bound)",
      "problem": "The 86 400 s cap is justified only as a typo guard, with impact modelled as \"a CI job hangs for hours\". With the poll interval fixed at 250 ms, a valid `--timeout 86400` — or a hostile CI-set env value — issues up to ~345,600 `message.list` requests at whatever `OPENCODE_BASE_URL` names: sustained request pressure, not just a slow job (checklist item 6, least capability).",
      "fix": "Justify the cap from an abuse standpoint (or lower it, e.g. 3 600 s) and bound the loop's cost independently of the deadline (max poll attempts / total requests) in the design."
    },
    {
      "severity": "minor",
      "location": "Chosen design → `src/cli.ts`, 4th bullet (\"extends the startup log (line 107)\")",
      "problem": "Line 107 already prints `config.baseUrl` unredacted, so `--base-url http://user:token@host` (or `$OPENCODE_BASE_URL`) prints credential material on every run; the design adds text to that same line without stating the logging rule for it (checklist item 4).",
      "fix": "State that the extended banner logs only the resolved duration, that any printed URL is redacted of userinfo, and that the \"no password provided\" hint keeps its current wording so `OPENCODE_SERVER_PASSWORD` is never echoed."
    },
    {
      "severity": "nit",
      "location": "Chosen design → `src/cli.ts`, 4th bullet; \"Data flow\"",
      "problem": "The ordering of the resolver relative to `createClient()` (cli.ts:101) and the startup banner (cli.ts:107) is not pinned, so an implementation could build the auth-header client and print the success-shaped banner before rejecting the config.",
      "fix": "Specify that `resolveReplyTimeoutSeconds` runs before `createClient` and before any output, so a config error yields exactly one error line and no side effects."
    }
  ]
}
```
