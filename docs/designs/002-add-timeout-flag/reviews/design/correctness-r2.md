# correctness review r2

verdict: approve

Scope: whole document (`02-proposal.md`). Checklist: `references/correctness.md`.
Previous correctness round reviewed: `reviews/design/correctness-r1.md`.

Verification performed: every `verified` register row was re-checked against the
tree this round. Code anchors confirmed: `opencode-runner.ts` :20/:22/:25–26/
:101–117/:210–223/:253–274/:259–271 (the `poll()` shape — immediate check,
deadline tested only after each reply check — traced by hand: a `timeoutMs =
100` run performs exactly two `message.list` calls then returns `null`, so the
docblock bullet and Test #6's two-check / ≤ one-interval-overshoot contract are
correct; Test #4's 29 000/31 000 fake-timer boundary matches the 30 s default),
`cli.ts` :16/:26–31/:50–57/:93/:94–100/:101/:107–110/:112/:119–123,
`tests/opencode-runner.test.ts` (A4's 13-site call list matches exactly — 105,
119, 129, 138, 147, 156, 168, 185, 198, 211, 228, 238, 247, all passing an
`options` object; `captureFetch()` at :53–61 with `attempt()` at :63–66, always
`500`, inside A17's :53–66 anchor; the 243–257/:253 timeout-test anchors),
`stages.ts` :31/:39, `README.md` :16–21/:28/:34–35, `package.json` (:23 `check`
= lint + format:check + tests, :40 `vitest ^5.0.5`, :44 yargs ^17.7.3, 47
lines, no `engines`), the `runPrompt` grep confined to the three files A11
names (no pipeline usage), no `vi.mock` anywhere in `tools/orchestration`
(supports A19), and the hidden-file glob for `**/*.{yml,yaml}` returning zero
files (A20).

Four register-basis URLs spot-checked live this round (plus r1's earlier checks
of the Node timers page and the Vitest Expect page):
`https://curl.se/docs/manpage.html#-m` — anchor live, `\-m, --max-time
<seconds>` section exists, but it says "This option accepts decimal values"
with `--max-time 2.92` as an example: partially contradicts A6's "plain
integer seconds" claim (see finding),
`https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNECT-CONNECT-TIMEOUT`
— "in seconds (write as a decimal integer, e.g., `10`). Zero, negative, or not
specified means wait indefinitely" — verbatim, supports A6's integer part, A14,
and research line 23,
`https://vitest.dev/blog/vitest-5.html` — "Vitest 5 requires Vite >= 6.4.0 and
Node.js >= 22.12.0." — verbatim, supports A21. No dead sources.

Revision closure (checklist #7): correctness-r1's two `[major]` findings are
closed in substance, not reworded — A17 is now an honestly `unverified` row
whose stated assumption (the SDK-codec question is *avoided*, never exercised)
matches the real `captureFetch()` stub, and Test #10/#11 commit to the
`vi.mock('../src/opencode-runner.js')` + exported `main(argv)` route with the
`clientFactory` fallback recorded (A18/A19 verified as stated above); the
`CliConfig`/`EnvValues` contradiction is resolved — the raw env bag is a
separate module-private type and `CliConfig` carries exactly one validated
`timeoutSeconds: number` populated in the `main()` literal. All four r1
`[minor]`/`[nit]` fixes are present in the body: runner-side contract pinned
with Test #5/#6, `#tothrow` anchor, full A4 call-site list, `timeoutMs / 1000`
formatting rule.

Internal consistency: seconds cross the CLI boundary, milliseconds the
CLI↔runner boundary, consistently (`timeoutSeconds: number` ↔ `timeoutMs`,
`MAX_REPLY_TIMEOUT_MS / 1000` = the README row's `1–3 600` = Test #9's
`3_600`/`3_601`); the exact-prefix message contract holds across Options
considered, Chosen design, Risks, Compatibility, Rollout #3 and Test #1; the
`within <n> s` formatting rule renders Test #2's `4 s` and Test #6's `0.1 s`
correctly; the poll-volume arithmetic (121 at 30 s, 14 400 at 3 600 s, ~119×)
recomputes correctly; resolver placement (after `parseArgs()`, before
`createClient()` at :101 and before the first `console.log` at :107) matches
the real `main()` shape. Every ask of `docs/prompts/add-timeout-flag.md` is
decided (flag + env twin, 30 s default, honored inside `runPrompt`,
distinguishable clearly-reported failure, tests green, change inside
`tools/orchestration`), and all six research open questions have explicit
decisions.

## Findings

- [minor] Assumption register A6 (+ "Options considered" → Value semantics) —
  the row is marked `verified` for "plain integer seconds" but its own first
  citation contradicts the integer part: the curl man page's `-m, --max-time
  <seconds>` section states "This option accepts decimal values" and shows
  `curl --max-time 2.92`. Only libpq `connect_timeout` ("decimal integer") and
  `CURLOPT_TIMEOUT` (long seconds) back the integer claim, and the
  fraction-rejection rationale names coreutils as the sole floating-point
  outlier when curl is a second one. The decision itself still stands on
  libpq plus 250 ms poll granularity, so this weakens the citation, not the
  outcome. Fix: reword A6 to "seconds-valued human-facing interfaces
  (integer-typed in libpq `connect_timeout` and `CURLOPT_TIMEOUT`;
  `--max-time` is seconds-valued but accepts decimals)" and acknowledge curl
  as a second decimal-valued outlier in the Options rationale.
- [minor] Assumption register A10, basis — the anchor
  `tools/orchestration/src/pipeline/runner.ts:326` does not land on the
  claimed pattern: :326 is the `title:` line of the `session.create` call; the
  `start + timeout` wall-clock deadline is at `runner.ts:338`
  (`const deadline = startedAt + cfg.stageTimeoutMs`, tested at :378). The
  claim itself is true and the `stages.ts:31,39` anchors are exact. Fix: cite
  `runner.ts:338` (and :378) instead of :326.
- [minor] Chosen design → `src/cli.ts`, resolver bullet 1 (+ Test strategy
  #8) — the set-but-empty env case (`OPENCODE_REPLY_TIMEOUT_SECONDS=""`,
  common in container images and `.env` files) is decided only implicitly:
  `process.env` yields `""`, which is `!== undefined` and fails the
  `^\d{1,5}$` gate, so the run hard-fails even with a valid `--timeout`.
  That is consistent with the design's uniform fail-closed-on-broken-env
  stance, but nowhere does the spec or the #8 matrix state empty-vs-unset,
  the exact case checklist #4 calls out ("empty input"). Fix: state the
  empty-string case explicitly in the resolver bullet and add `""` to Test
  #8's rejection matrix.
- [nit] Chosen design → `src/cli.ts`, resolver bullet 1 (+ "Data flow and
  input trust") — the strictness rationale ("`Number()` is far looser than
  integer seconds: `"0x2d"` → 45, `"1e4"` → 10000") is argued only for env;
  the flag path has the identical silent coercion (yargs `type: 'number'`
  turns `--timeout 0x2d` into `45`, which then passes the range check) and
  the `number` parameter signature makes a format gate impossible there.
  Harmless in practice, but the stated rationale reads as if only env is
  exposed. Fix: one sentence recording the flag path's accepted coercion, or
  thread the raw argv token into the resolver for a symmetric gate.
- [nit] Test strategy #5 — "real timers, since the call must reject
  synchronously": `runPrompt` is `async`, so an invalid `timeoutMs` produces
  a rejected promise, never a synchronous throw; the call rejects
  immediately because validation precedes any timer scheduling, and fake vs
  real timers is irrelevant to that. Fix: drop the parenthetical or say "no
  fake timers needed: the promise rejects before any timer is scheduled".
- [nit] Revision log, r3 entry (02-proposal.md:160) — the r3 bullet contains
  the "**Deliberately not changed this round**" paragraph twice verbatim in
  the same line (grep finds the pair inside :160; only the second copy is
  followed by the register-status sentence). Log hygiene only — no design
  content is contradicted. Fix: delete the duplicated paragraph.

## FYI (no severity)

- Compatibility matrix: "TypeScript rejects the unknown `timeoutMs` property,
  so it constrains only future use of the new field" describes a recompile
  experience for existing consumers; whether that is the right compatibility
  framing belongs to the compatibility specialty, not to correctness.
- The README:28 session-title drift ("SDK connection test" vs
  `SESSION_TITLE = 'Untitled Session'`) is pre-existing and already explicitly
  handled by the design (leave alone or separate `docs:` commit) — noted in
  r1's FYI, correctly left untouched here.
