# security review r3

verdict: approve

Scope: this change set only — `reviews/design/editorial-r3.diff` applied to
`02-proposal.md` (the proposed file already contains the applied hunks). The
document body and the r1/r2 closure state are context, not the target; the
previous security rounds walked the whole-document checklist
(`security-r1.md`, `security-r2.md`). Checklist: `references/security.md` only.

## Findings
- none

All seven hunks are language-only and security-neutral. Verified hunk by hunk
against the pre/post text and against the applied file at lines 5, 11, 55, 85,
126, 130, 160 (the `+` side of every hunk is present verbatim; no collateral
edits):

1. Context (02-proposal.md:5) — `honoring` → `honouring`. Spelling only; the
   sentence still names both untrusted config surfaces (`--timeout`,
   `$OPENCODE_REPLY_TIMEOUT_SECONDS`) unchanged.
2. Goals bullet 2 (02-proposal.md:11) — `honors` → `honours`. The
   fail-closed-at-the-API-boundary promise is byte-identical otherwise.
3. Options considered → Value semantics (02-proposal.md:55) — `unit-suffix
   syntax we are not adopting` → `…, which we are not adopting`. This is the
   paragraph carrying the abuse-side justification for the `1..3_600` cap
   (checklist item 6): the `timeoutMs / 250` poll math, ~121 polls at the
   default, ~14 400 at the ceiling, the rejected 86 400 s option ("too large a
   self-inflicted DoS surface") and the rejected no-cap option are untouched.
   No bound, number, or threat-model wording changed.
4. Chosen design → `resolveReplyTimeoutSeconds` bullet (02-proposal.md:85) —
   `avoid second hardcoded bounds` → `avoid a second set of hard-coded bounds`.
   The single-source-of-truth rule for the two bounds that both the CLI range
   check and the `runPrompt` boundary check derive from
   (`DEFAULT_REPLY_TIMEOUT_MS / 1000`, `MAX_REPLY_TIMEOUT_MS / 1000`) is
   unchanged. The adjacent format-gate and reflection bullets (lines 83–84)
   and the `main()` ordering/logging bullet (line 86) appear in this hunk as
   context only and are unmodified.
5. Test strategy #6 (02-proposal.md:126) — inserted `that`. The bounded
   sub-poll contract (two `message.list` calls, ≤ one-interval overshoot) that
   keeps an attacker-visible `timeoutMs` bounded is unchanged; Test #5 (the
   runner-side `TypeError` fail-closed pin closing security-r1's major) is
   context, untouched.
6. Test strategy #10 (02-proposal.md:130) — comma after `without the stub
   result`. Mock/test-only text; no runtime or logging surface.
7. Revision log r3 block (02-proposal.md:160) — deletes a verbatim duplication
   of the `**Deliberately not changed this round**` paragraph and completes the
   truncated tail (`fallback-trigger` → `fallback-triggering condition`).
   Diffed against the applied file: the retained copy preserves every
   security-bearing statement — the `1..3_600` cap kept on the abuse model
   (A16), `0`/negative rejection, the pre-existing unredacted `baseUrl` print
   left alone as a disclosed non-goal, `ReplyTimeoutError` carrying no data
   properties, and A8/A14/A17/A19 staying `unverified` (the r4 entry at line
   165 restates the same list). Nothing security-relevant was dropped; the
   duplication this removed was itself flagged as a defect in security-r2's
   FYI.

## Checklist walk-through (against the change set)
- 1 Trust boundaries: unaffected — the "Data flow and input trust" paragraph
  (02-proposal.md:88) is context in hunk @@ -82 and byte-identical; argv/env
  stay classified untrusted operator input, the server reply untrusted network
  input, with the covering validation named for each.
- 2 Parsing untrusted data: unaffected — no hunk changes the `^\d{1,5}$` gate,
  its precedence over `Number()`, the separate range check, or the
  `JSON.stringify`-escaped reflection (Test #8, line 128, untouched).
- 3 Path handling: not applicable — the diff adds no path, glob, or shell
  string; nothing interpolated.
- 4 Secrets/PII and logging: unaffected — the only edited line in that region
  (85) is the bounds-derivation sentence; the banner rule (line 86: static text
  plus the resolved integer, password hint never echoing a credential value,
  `baseUrl` non-goal) and the Risks rows (lines 102, 105) are untouched.
- 5 Dependencies: re-checked live this round and clean — the change set names
  and touches no dependency. OSV queries against the resolved lockfile versions
  return zero advisories for `yargs@17.7.3` (lockfile :3433),
  `yargs-parser@21.1.1` (:3451, above the 18.1.1 patched by GHSA-p9pc-299p-vxgp
  / CVE-2020-7608), `vitest@5.0.5` (:3285), `@vitest/mocker@5.0.5`,
  `@opencode/client@2.1.4` (:879) and `zod@4.6.7` (:3473). No known unfixed
  advisory, so nothing here is a blocker.
- 6 Privileges: unaffected — no new capability; the only capability stays
  bounded HTTP polling, still capped at 3 600 s (≤ ~14 400 `message.list`
  polls) at both the CLI resolver and the `runPrompt` boundary.
- 7 Fail closed: unaffected — no hunk weakens the reject-cleanly story: bad
  flag, bad env (including a malformed env alongside a valid flag), and raw-API
  `timeoutMs` values still reject with one error line / `TypeError` before any
  session or request (Tests #5, #8, #11 unchanged).

## FYI (no severity)
- security-r2's two `[nit]`s are untouched by this editorial diff and remain
  open as non-blocking items, correctly left for a technical revision rather
  than a language-only pass: (a) the set-but-empty env case
  (`OPENCODE_REPLY_TIMEOUT_SECONDS=`) is still absent from Test #8's matrix
  (02-proposal.md:128) — already fail-closed by `^\d{1,5}$`, just unpinned; and
  (b) the claim at 02-proposal.md:84 that `JSON.stringify(value)` "escapes
  control and ANSI bytes" still overstates the escaper (it covers C0 controls
  including ESC, not DEL, C1 U+0080–U+009F, or U+2028/2029); the load-bearing
  defense is the `^\d{1,5}$` gate. Rewording it changes technical meaning, so it
  is out of an editorial pass's remit — flagging so it is not lost.
- Pre-existing and disclosed as a non-goal (02-proposal.md:86, Risks): the
  startup line prints `config.baseUrl` unredacted, so a credential-bearing URL
  (`--base-url http://user:token@host` / `OPENCODE_BASE_URL`) lands in stdout on
  every run. The change set adds no text to that path; r1's userinfo-redaction
  remedy remains a separate `fix:` commit worth filing.
- For the proposal editor: the `honor→honour` and `hardcoded→hard-coded` edits
  align the document with its prevailing en-GB style; no US-spelling remnants
  found, so the security text now names things consistently (`hard-coded`
  everywhere, including the safety-critical `POLL_INTERVAL_MS` wording).
