# security review r2

verdict: approve

Scope: whole document (`02-proposal.md`, as revised through r4). Checklist: `references/security.md` only.
Every security-r1 finding was re-verified **in the body**, not taken from the revision log: both
[major]s closed — runner-side fail-closed validation (`runPrompt` throws `TypeError` for non-integer /
`< 1` / `> MAX_REPLY_TIMEOUT_MS` before session creation, zero HTTP requests; Chosen design →
`src/opencode-runner.ts` bullet 3, Test #5) and env reflection (the `^\d{1,5}$` gate runs before any
`Number()` and before any echo, whether or not the flag is present, with `JSON.stringify`-escaped
reflection; resolver bullet 3, Test #8) — all four [minor]s closed (trust-classified "Data flow and
input trust"; format gate separated from range rejection; cap lowered to 3 600 s and sized from the
abuse model with the Risks row quantifying ≤ ~14 400 polls; banner logging rule + password hint kept
verbatim, never echoing a credential) — and the [nit] closed (resolver runs after `parseArgs()` but
before `createClient()` and any output; Test #11 pins the zero-side-effect config error). New material
added since r1 (`MAX_REPLY_TIMEOUT_MS` export, `EnvValues`, `engines.node`, README row, `vi.mock`
tests) introduces no new attack surface: the only value reaching deadline arithmetic is a doubly
validated integer, `engines` is advisory, and the mock tests are dev-only.

## Findings

- [nit] Test strategy #8 (02-proposal.md:128) — the validation matrix omits the set-but-empty env
  case (`OPENCODE_REPLY_TIMEOUT_SECONDS=`), a common shell typo. As designed it correctly throws
  (`envValue !== undefined` and `""` fails `^\d{1,5}$`, even alongside a valid `--timeout`), so this
  is fail-closed already — but the behavior is neither pinned by a test nor documented, and it
  surprises users who clear a variable by assigning empty. Fix: add `""` to #8's env case list and
  have the format-error message state that an empty value is an error (the README's `1–3 600`
  implies it; naming it removes the surprise).
- [nit] Chosen design → `src/cli.ts` → `resolveReplyTimeoutSeconds`, reflection bullet
  (02-proposal.md:84) — the claim that `JSON.stringify(value)` "escapes control and ANSI bytes" is
  stronger than reality: it escapes C0 controls (including ESC `\x1b`), `"` and `\`, but not DEL
  (U+007F), C1 controls (U+0080–U+009F, which include 8-bit CSI), or U+2028/2029. The path is safe
  only because the `^\d{1,5}$` gate rejects every non-digit env value before reflection and flag
  values arrive as numbers. Fix: reword so the gate is named as the load-bearing defense and
  `JSON.stringify` as defense-in-depth for C0 — otherwise a future gate relaxation could rely on the
  escaper alone.

## FYI (no severity)

- Checklist item 5 re-checked and **clean — not a blocker**: the design adds no new dependency, and
  no named/used dependency has a known unfixed advisory. (The GitHub advisory API fetch this round
  returned proxied junk, so the check was run against the OSV API plus the lockfile.) `yargs@17.7.3`
  (lockfile :3433): zero advisories, and its resolved `yargs-parser@21.1.1` (lockfile :3451) is above
  the 18.1.1 patched by GHSA-p9pc-299p-vxgp (CVE-2020-7608); `@opencode/client@2.1.4`
  (lockfile :879): zero advisories; `vitest`/`@vitest/mocker@5.0.5` (lockfile :3285 / :1604):
  outside the affected ranges of GHSA-82fw-gwwq-j7x9, GHSA-5xrq-8626-4rwp and
  GHSA-9crc-q9x8-hgqq, and the gate's `vitest run` starts no UI/API server, so none of the
  dev-server advisories apply; `zod@4.6.7` (lockfile :3473, on the reply-parse path though not
  named in the design) is ≥ 4.6.6, which fixed CVE-2023-54404, and ≥ 3.22.3 for CVE-2023-4316.
  A machine-checked `npm audit` step in Rollout #1 would be nice — release-process, not this design.
- Pre-existing and untouched, and the proposal now discloses it as a non-goal (02-proposal.md:21;
  Chosen design → `src/cli.ts` `main()` bullet): the startup line prints `config.baseUrl`
  unredacted, so a credential-bearing URL (`--base-url http://user:token@host` or
  `OPENCODE_BASE_URL`) lands in stdout on every run. The change adds only static text plus the
  validated integer to that line, so it does not worsen the leak, and the "no password was provided"
  hint never echoes a value — hence not a finding against this change set. r1's userinfo-redaction
  remedy stands as a separate `fix:` commit worth filing.
- Checklist items 3 and 6: the change set introduces no path handling, no shell interpolation, and
  no new filesystem/device capability; the only capability it touches is bounded HTTP polling, and
  the 3 600 s cap enforced at both the CLI resolver and the `runPrompt` boundary bounds it
  (≤ ~14 400 `message.list` polls at the ceiling). Item 7 holds end-to-end: bad flag, bad env, and
  raw-API values all reject cleanly (exit 1, or `TypeError` before any request), never partially
  applying and never polling open-ended.
- Editorial, not security (for the correctness/editing specialties): Revision log r3 repeats the
  "**Deliberately not changed this round**" bullet verbatim twice inside one line
  (02-proposal.md:160).
