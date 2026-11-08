# compatibility review r2 (code)

verdict: approve

Scope: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD` (`baseRef`
confirmed at `.pipeline.json:791`), judged against `02-proposal.md` (r4) and
`03-build-notes.md`. Checklist: `references/compatibility.md` (code
compatibility — platform/Node/paths/packaging).
Previous round for this specialty: **none landed**. `reviews/code/
compatibility-r1.md` does not exist; the two r1 attempts failed their reply
gate (`transcripts/code-review-compatibility-r1.md` and
`code-review-compatibility-attempt2-r1.md` carry empty replies;
`.pipeline.json:771-780` records `code-review:compatibility: invalid-reply: not
JSON` twice, then `escalate`). So checklist item 7 ("previous round's
blocker/major addressed?") has nothing to re-verify, and no earlier
compatibility finding is carried forward. The *design*-side compatibility
reviews (`reviews/design/compatibility-r1/2/3.md`) are treated as the design
contract the code is measured against.

Environment note: this review environment rejects **every** shell command
(`git diff`, `git log`, `npm run check`, `npx vitest` → `Permission denied:
shell`), so the diff could not be re-derived and the gate could not be
re-executed. The review is a line-by-line static trace of the HEAD tree against
the proposal, plus the artifacts that *are* in the tree (`.pipeline.json` run
and gate log, the three suites). The base-vs-HEAD attribution below is anchored
on the proposal's own verified line anchors (cli.ts:101/107/108–110/112/113/
121, `main(argv)` at :93), which match HEAD minus the added blocks; the
`isMain`/`fileURLToPath` entry guard at `src/cli.ts:221` is therefore
**pre-existing, not introduced by this change** (the proposal's A13 anchors
cli.ts:119–123 with `console.error(error)` at :121, i.e. the guarded shape
already existed at the base pin).

## Checklist walk

1. **Platform code matches the proposal's Compatibility matrix; OS-specific
   code behind explicit guards; every "supported" row has a code path.** ✓
   The change set introduces **zero** OS-specific code: the timeout path stays
   `Date.now()` wall-clock arithmetic + `setTimeout` polling
   (`opencode-runner.ts:240-246, 285-307, 330-332`), exactly the OS-neutral
   shape A12 argues and the Linux/macOS/Windows rows describe; nothing is
   behind a `process.platform` branch (tree grep for `process.platform`,
   `win32`, `os.EOL`, `path.sep` → no hits anywhere in `src/` or `tests/`).
   All three "Yes" rows are served by that one shared code path — there is no
   row whose path is missing. The "Pipeline CLI — Not applicable" row holds:
   no `timeoutMs`/`ReplyTimeoutError`/`OPENCODE_REPLY_TIMEOUT_SECONDS`
   reference exists under `src/pipeline/` (grep), and `runPrompt` stays
   confined to `src/cli.ts`, `src/opencode-runner.ts` and the three suites
   (A11). ✓ (evidence gap on the platform rows themselves: finding 2;
   in-range pipeline platform exposure: finding 3)
2. **Node floor: no API above the supported version.** ✓ The proposal's
   compatibility-r1-nit remedy landed: `package.json:30-32` declares
   `"engines": { "node": ">=22.12.0" }` and `package-lock.json:11-13` mirrors
   it into the lockfile's root package entry, so `npm ci` stays in sync. Every
   API the diff introduces sits far below that floor: `Number.isInteger`,
   `JSON.stringify`, `String()`, a plain `/^\d{1,5}$/` regex, `class extends
   Error`, and the runner's `Date.now()` arithmetic. A targeted grep for
   post-floor APIs (`structuredClone`, `toSorted`/`toReversed`/`toSpliced`,
   `findLast*`, `Object.groupBy`, `Array.fromAsync`, `AbortSignal.timeout`,
   `scheduler.`, `Intl.*`, `styleText`, `globSync`) returns nothing in `src/`
   or `tests/`. `tsconfig.json` stays `target`/`lib: ES2022`, `module:
   NodeNext` — matching the matrix's Node-toolchain row. A9 holds in code: only
   the fixed 250 ms `sleep()` uses a timer; the deadline is never a
   `setTimeout` delay, so the 2147483647 ms clamp can't touch `--timeout 3600`.
   ✓ (forward-looking gap in the floor's *type* anchor: finding 4)
3. **Paths built with `path`/URL joins, never string concatenation; case
   sensitivity flagged.** ✓ No path construction at all in the design's change
   set — the only new strings are error-message text. The pre-existing path
   code (out of this design's change set, in-repo) already uses joins rather
   than concatenation: `src/pipeline/cli.ts:17`
   (`join(fileURLToPath(new URL('../../prompts/', import.meta.url)))`),
   `src/pipeline/prompts.ts:27,63-65`, `state-store.ts:10,45-46`, and
   `tests/pipeline-prompts.test.ts:19` (URL join). Case-sensitivity: the new
   surface reads `process.env.OPENCODE_REPLY_TIMEOUT_SECONDS` (`cli.ts:82`),
   which Node looks up case-insensitively on Windows, and the value is matched
   with an anchored ASCII-digit regex — no case assumptions.
4. **Guarded platform imports.** ✓ No platform-only module is imported: the
   diff adds no imports at all except the ones already on `cli.ts:1-14`;
   `node:buffer`, `node:url` (pre-existing) and the vitest test APIs are all
   cross-platform, and `@opencode/client` is the pre-existing dependency.
   Nothing in the change set can break module load on another OS.
5. **Locale/layout reality — do formats and IDs survive non-ASCII?** Mostly ✓,
   with a nit. The value formats are ASCII-only by construction: the env gate
   `^\d{1,5}$` (`cli.ts:108`) is ASCII-only *because* it lacks the `u`/`v`
   flag, so Unicode digits (`٣٦٠٠`), locale grouping separators, NBSP and
   typographic minus all fail closed rather than being silently reinterpreted;
   flag values reach the resolver as numbers. No IDs or file names are
   introduced, and the new `ReplyTimeoutError` carries no identifier-bearing
   field. Two residuals: the reflected env value is only `JSON.stringify`
   -escaped, which leaves non-ASCII and C1 control characters raw — finding 5;
   and the README's `1–3 600` grouping (`README.md:22`) is a copy-paste trap
   the resolver then rejects with a quoted, escaped value, which is fail-closed
   and legible (no finding).
6. **Native dependencies introduced, packaging story, CI matrix.** ✓ No new
   dependency of any kind: `package.json:33-49` dependencies/devDependencies
   are byte-identical to the base except the added `engines` block, and the
   lockfile root entry mirrors it (`package-lock.json:11-13`). The lock's
   `os`/`cpu`-scoped entries (`@rollup/rollup-*`, `@esbuild/*`,
   `node_modules/fsevents` at `:2191`) are pre-existing transitive optionals of
   the vitest toolchain, not something this change introduced, so there is no
   new packaging or install-matrix burden. Packaging stays as Rollout #4
   describes: `files: ["dist"]`, `npm run build` output, no new binary, no
   `bin` entry.
7. **Tests named in the plan run on the platforms they claim; platform tests
   guarded, not skipped everywhere.** Partially evaluated — finding 2. The
   plan (Test strategy #12) claims exactly one executed platform (Linux, local
   gate — no CI exists in the repo, A20) and explicitly marks macOS/Windows
   "Yes — unverified". The code is consistent with that: the two new suites are
   platform-portable (argv-driven `main(['node','cli', …])`, `process.env`
   mutation with `afterEach` cleanup, `mkdtemp`/`tmpdir` nowhere in them, no
   shell or subprocess spawn, no fake-clock assumptions beyond vitest defaults
   — there is no `vitest.config.*` in the package, so the same defaults apply on
   every OS), and no test is guarded or skipped (`it.skip`,
   `skipIf`, `runIf` → no hits). Whether the gate actually *ran* could not be
   re-executed here; the only tree evidence is `.pipeline.json:761-766`
   (`gate: testsGreen`, `round: 0`, `proceed: true`, reason
   `"npm run check green"`), which names no OS, no `node -v`, and no test
   counts.

## Findings

1. **[minor]** `tools/orchestration/src/opencode-runner.ts:352` — a second
   observable delta for existing `@bindforge/orchestration` API consumers that
   the design's Compatibility matrix does not list. The matrix's "Existing API
   consumers" row (proposal:117) and Rollout #3 (proposal:138) both say the
   *one* observable delta is the extended message with the old string kept as
   an exact prefix. `this.name = 'ReplyTimeoutError'` adds a second one:
   `error.name` changes from `'Error'` to `'ReplyTimeoutError'` for code that
   branches or logs on `name`, and the CLI's stderr first line changes from
   `Error: No response received from OpenCode…` to
   `ReplyTimeoutError: No response received from OpenCode…`, which breaks any
   consumer scraping the CLI's stderr by line prefix. The change is declared
   (build-notes deviation 3) and keeps the message contract byte-identical, so
   it is graded minor, but the "one delta" claim in the matrix is now
   incomplete. Fix: either drop `this.name` (the subclass name is already
   available via `constructor.name` and `instanceof ReplyTimeoutError`), or
   keep it and add it to the matrix/Rollout #3 delta list as a second
   documented, prefix-safe delta.
2. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` +
   `tools/orchestration/tests/cli.test.ts` / `tests/opencode-runner-timeout.test.ts`
   — checklist item 7 could not be evaluated by execution, so it is a finding,
   not a pass. The matrix claims three supported platforms; the executed
   evidence in the tree is one gate-log line (`testsGreen` / `"npm run check
   green"`, `.pipeline.json:761-766`) that carries no platform, no Node
   version and no per-suite counts, and every shell command in this review
   environment was refused (`git status`, `npm run check`), so the Linux row is
   asserted rather than verified here and macOS/Windows remain inspection-only
   (as the design itself says). Nothing indicates a *platform* failure — the
   new suites are portable by construction — but the record cannot distinguish
   "green on Linux" from "green on an unknown machine". Fix: append the gate
   summary to `03-build-notes.md` (OS + `node -v` + per-suite test counts +
   lint/format status, and the `.pipeline.json:761-766` reference replacing
   deviation 1's now-stale "must still be run by the caller"), and re-run
   `npm run check` in `tools/orchestration` before merge if that log line
   proves wrong. Do not add a CI leg — introducing CI is an explicit non-goal
   (proposal:22), which is why the two rows stay "Yes — unverified".
3. **[minor]** `tools/orchestration/src/pipeline/gates.ts:190` (and
   `src/pipeline/runner.ts:71,79,650,841`) — POSIX-only process launches inside
   the reviewed range. `spawn('npm', ['run','check'], …)` cannot start on
   Windows without `shell: true` (the executable is `npm.cmd`), so the gate
   resolves `false` via `child.on('error')` rather than failing loudly;
   `spawnSync('git', …)` and `spawnSync('diff', …)` assume binaries that are not
   guaranteed on a stock Windows install. Those files sit inside the range the
   caller gave (the out-of-scope commits `f0a2e16`/`891567b` already recorded by
   `reviews/code/correctness-r1.md:112-129` and `correctness-r2.md:176-193`;
   this environment refused `git log`, so the per-line attribution is taken from
   those reviews, not re-audited). It is not part of the design's change set and
   touches no timeout code, but it is a live platform limitation inside the
   diff under review, and it makes the package's new `engines`/platform claims
   misleading for anyone who runs the pipeline tooling on Windows. Fix: state in
   `README.md`/build notes that the pipeline tooling is POSIX-only, or guard the
   launches (`process.platform === 'win32' ? 'npm.cmd' : 'npm'`, or
   `shell: true`), or re-pin `baseRef` (`.pipeline.json:791`) so the reviewed
   range contains only the design's change set.
4. **[nit]** `tools/orchestration/package.json:30-32,36` — the new floor is
   declared but its anchors are asymmetric. `engines.node >=22.12.0` is
   published metadata for a package whose shipped `dist/` code (`main:
   dist/cli.js`, `types: dist/cli.d.ts`) needs nothing above the ES2022 target,
   so the floor is a *toolchain* requirement (A21) that will warn — and with
   `engine-strict` fail — for library consumers pinned to older Node, while
   nothing user-facing documents it (`README.md` has no requirements line). At
   the same time `@types/node` is `^26.6.5`, i.e. the compile-time API surface
   runs four majors past the declared floor, and the ESLint config
   (`eslint.config.js`) has no compatibility rule, so the next change can
   compile cleanly against an API the floor rejects. Fix: add one README
   "Requirements" line stating the floor and that it is the toolchain (Vitest 5)
   floor, and pin `@types/node` to the floor major (`^22`) so the type anchor and
   the runtime floor agree.
5. **[nit]** `tools/orchestration/src/cli.ts:111,130` (asserted at
   `tests/cli.test.ts:110-115`) — the reflection escape covers C0 controls and
   ANSI `\x1b` but not the rest of the non-ASCII range. `JSON.stringify` leaves
   C1 control characters (U+0080–U+009F, e.g. CSI `Ð›`) and all printable
   non-ASCII bytes unescaped, and the format-gate error path is exactly the
   path that echoes an arbitrary rejected value. On terminals that honour 8-bit
   C1 in UTF-8 mode the C1 bytes are still live escape introducers, and on
   legacy Windows console code pages (cp437/cp1252) a non-ASCII value echoes as
   mojibake, making the very message meant to explain the bad value unreadable.
   This is beyond what security-r1's rule required (it is satisfied — no raw
   C0/ANSI byte reaches stderr), hence a nit. Fix: escape non-ASCII as well,
   e.g. `JSON.stringify(envValue).replace(/[\u007f-\u009f]/g, (c) =>
   '\\u' + c.charCodeAt(0).toString(16).padStart(4, '0'))`, or reflect
   `<rejected: ${envValue.length} chars, not plain digits>`; keep the
   `not.toContain('\x1b')` assertions.
6. **[nit]** `tools/orchestration/src/opencode-runner.ts:348-351` — the
   timeout report tells library consumers to use flags they do not have.
   `ReplyTimeoutError` is exported from the published API, and its fixed suffix
   reads `(set --timeout or $OPENCODE_REPLY_TIMEOUT_SECONDS to change it)`,
   which is guidance only for the `tools/orchestration` CLI; a programmatic
   caller's knob is `PromptOptions.timeoutMs`. Design-mandated text
   (proposal:72), so this is a design-level wording note rather than a
   deviation. Fix (design change first): extend the suffix to
   `(set --timeout, $OPENCODE_REPLY_TIMEOUT_SECONDS, or PromptOptions.timeoutMs
   to change it)` — the prefix contract (A5) and
   `tests/cli.test.ts`/`opencode-runner-timeout.test.ts` assertions all stay
   green, since only the suffix changes.

## FYI (out of specialty, no severity)

The banner line (`cli.ts:203-206`) now reads `… with a sample prompt waiting up
to 30 s for the reply...` with no comma or period where the base line ended, so
the sentence is a run-on for both the agent and no-agent forms — cosmetic, and
the exact text is what the proposal prescribes (proposal:86). Also: the format
and range errors both quote `1..3600` (`cli.ts:110-111,118,129-130`) even though
the proposal deliberately separates format rejection from range rejection
(proposal:82) — carried as a nit by `reviews/code/correctness-r1.md:130-137` and
`correctness-r2.md:206-214`; not repeated with severity here. And
`README.md:22`'s `1–3 600` thousands grouping invites the copy-paste value
`"3 600"`, which the gate does reject with an escaped quote — legible, so no
finding.
