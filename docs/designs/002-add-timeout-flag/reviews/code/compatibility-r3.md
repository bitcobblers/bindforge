# compatibility review r3 (code)

verdict: approve

Scope: `git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD` (`baseRef`
confirmed at `.pipeline.json:962`), judged against `02-proposal.md` (r4) and
`03-build-notes.md`. Checklist: `references/compatibility.md` (platform matrix,
Node floor, paths, platform imports, locale, native deps, test-platform
reality). Previous round for this specialty: `reviews/code/compatibility-r2.md`
(approve; 3 minor + 3 nit, all non-blocking) — its findings are re-checked
against the current tree below and carried where still open.

Execution note (unlike r1/r2, this round **could** run commands): the design's
only acceptance gate, `npm run check` (lint + format check + tests,
package.json:23), was executed at this HEAD in `tools/orchestration` — ESLint
clean, `prettier --check .` → "All matched files use Prettier code style!",
`vitest run` → **Test Files 7 passed (7), Tests 91 passed (91)**, runner
`vitest v5.0.5`. Platform: Linux (kernel `7.3.3-1-cachyos`, read from
`/proc/version`). `git diff HEAD --stat` shows the only working-tree delta is
`docs/designs/002-add-timeout-flag/.pipeline.json` (the runner's own log), so
the run reflects the reviewed code. `node -v` itself was refused
(`Permission denied: shell`), so the Node version is not recorded; the floor is
nonetheless indirectly evidenced, since vitest 5.0.5 — the toolchain that sets
the floor (A21) — ran and passed.

## Checklist walk

1. **Platform code matches the proposal's Compatibility matrix; OS-specific
   code behind explicit guards; every "supported" row has a code path.** ✓
   The design's change set introduces **zero** OS-specific code: the wait path
   stays `Date.now()` wall-clock arithmetic plus a fixed 250 ms `setTimeout`
   sleep (`opencode-runner.ts:25, 285-307, 330-332`), exactly the OS-neutral
   shape A12 argues and the Linux/macOS/Windows rows describe. A grep for
   `process.platform`, `win32`, `os.EOL`, `path.sep`, `npm.cmd`, `shell: true`
   across `src/` and `tests/` returns **no hits** — nothing is guarded because
   nothing needs guarding, and no matrix row lacks a code path. The
   "Pipeline CLI — Not applicable" row still holds at HEAD: grep of
   `runPrompt|timeoutMs|OPENCODE_REPLY_TIMEOUT_SECONDS|ReplyTimeoutError`
   across the package hits only `src/cli.ts`, `src/opencode-runner.ts` and the
   three suites (A11), zero hits under `src/pipeline/`. **Attribution
   correction against r2 finding 3:** the POSIX-only process launches
   (`spawn('npm', ['run','check'])` at `src/pipeline/gates.ts:190`,
   `spawnSync('git', …)` at `src/pipeline/runner.ts:72,80,651`,
   `spawnSync('diff', …)` at `:842-844`) appear as neither `+` nor `-` lines
   anywhere in this range's diff, so they are **identical at `baseRef`** and
   were *not* introduced by this branch; r2 carried them as an in-range
   platform limitation, which was mis-attributed. They remain a real (pre-
   existing) limitation of the pipeline tooling — see the FYI line. The one
   *new* process-launch surface in the range is `enforceOwnership`'s git
   pathspec handling — finding 3. (Matrix-row evidence gap: finding 2.)
2. **Node floor: no API above the supported version.** ✓ The matrix's
   toolchain row is anchored in the tree: `package.json:30-32` declares
   `"engines": { "node": ">=22.12.0" }` and `package-lock.json` mirrors the
   same block into the root package entry, so `npm ci` stays in sync.
   `tsconfig.json:3-4,6` stays `target`/`lib: ES2022`, `module: NodeNext`,
   matching the matrix row. Everything the diff introduces sits far below the
   floor: `Number.isInteger`, `String()`, `JSON.stringify`, a plain
   `/^\d{1,5}$/` gate, `class extends Error` (`opencode-runner.ts:346`,
   native at ES2022, so `instanceof ReplyTimeoutError` works as the tests
   assert), and the runner's `Date.now()` arithmetic. A grep for post-floor
   APIs (`structuredClone`, `toSorted`/`toReversed`/`toSpliced`, `findLast*`,
   `Object.groupBy`, `Array.fromAsync`, `AbortSignal.timeout`, `scheduler.`,
   `styleText`, `globSync`) returns nothing in `src/` or `tests/`; the only
   late-vintage call in the range is `Array.prototype.at(-1)`
   (`gates.ts:96`, `runner.ts:409`), which is ES2022 — inside the declared
   lib. A9 holds in code: only the fixed `POLL_INTERVAL_MS` uses a timer, so
   the 2 147 483 647 ms `setTimeout` clamp can never truncate
   `--timeout 3600`. Residual asymmetry between the declared floor and the
   compile-time type surface: finding 4.
3. **Paths built with `path`/URL joins, never string concatenation; case
   sensitivity assumptions flagged.** ✓ For the design's change set there is
   no path construction at all — the only new strings are error/banner text and
   a yargs option name. The pipeline code in the range (out of design scope,
   in scope for this review) consistently uses `node:path`:
   `resolve(repoDir, p)` (`runner.ts:112`), `join(…)` at
   `runner.ts:176,332,630,671,840,851,862`, `prompts.ts:27,63-65`,
   `state-store.ts:10,15`, `cli.ts:17,51,95` — no `${dir}/${name}` style
   concatenation in changed lines. Case sensitivity: the new surface reads
   `process.env.OPENCODE_REPLY_TIMEOUT_SECONDS` (`cli.ts:82`), which Node
   resolves case-insensitively on Windows, and matches the value with an
   anchored ASCII-digit regex — no case assumption. `stagePathAllowed`'s
   regexes (`gates.ts:232,236`) assume `/`-separated paths, which is correct
   because git emits forward slashes on every OS. The one fragile
   path-in-the-new-code case is non-ASCII/quoting round-tripping through git's
   text output: finding 3.
4. **Guarded platform imports.** ✓ The diff adds no import that could break
   module load on another OS: the only new import block is the widened value
   import from `./opencode-runner.js` (`cli.ts:7-14`); the new test files
   import `vitest`, `zod`, and `import type { OpenCodeClient } from
   '@opencode/client'` (type-only, erased at compile). No `node:*` platform
   module, no optional/lazy platform require, so nothing needs a guard.
5. **Locale/layout reality — do formats and IDs survive non-ASCII?** Mostly ✓,
   two residuals. Every new value format is ASCII-only by construction: the
   env gate `/^\d{1,5}$/` (`cli.ts:108`) has no `u`/`v` flag, so `\d` is ASCII
   `\d` — Arabic-Indic digits, NBSP, U+2212, thin-space grouping and
   typographic separators all fail closed instead of being silently
   reinterpreted, and flag values reach the resolver as yargs numbers
   (`Number()`-parsed, locale-independent). No new file format, filename or ID
   is introduced by the design's change set, and `ReplyTimeoutError` carries no
   identifier-bearing field. Residuals: the reflected rejected env value is
   escaped only with `JSON.stringify`, which leaves C1 controls and printable
   non-ASCII raw (finding 5); and the resolver's messages print the bound as
   `1..3600` while `README.md:22` documents `1–3 600`, whose thousands space is
   a copy-paste trap — but the gate rejects `"3 600"` with a quoted, escaped,
   legible message, so it is fail-closed and recorded as an FYI, not a finding.
   The in-range pipeline code newly routes operator-authored non-ASCII paths
   through git's text output: finding 3.
6. **Native dependencies introduced, packaging story, CI matrix.** ✓ No
   dependency of any kind is added: `package.json:33-49` is byte-identical to
   the base except the `engines` block, and the lockfile diff is exactly the
   mirrored `engines` block (3 lines). The lock's `os`/`cpu`-scoped optionals
   (`@rollup/rollup-*`, `@esbuild/*`, `fsevents`) are pre-existing vitest
   transitives, untouched here, so there is no new install-matrix or binary
   story. Packaging stays as Rollout #4 says: `files: ["dist"]`, `npm run
   build` output, no `bin` entry — the new `engines` metadata is the only
   packaging delta. No CI was introduced (an explicit non-goal, proposal:22),
   consistent with A20 and Test #12.
7. **Tests named in the plan run on the platforms they claim; platform tests
   guarded, not skipped everywhere.** ✓ — and this round it is evaluated **by
   execution**, closing the gap r1/r2 had to carry as a finding. Test strategy
   #12 claims exactly one executed platform (Linux, local gate, no CI): that
   gate was run at this HEAD and is green (7 files / 91 tests, lint + format
   check clean, vitest 5.0.5). The two new suites are portable by construction
   — `main(['node','cli','--timeout','2'])` in-process argv (`cli.test.ts:174`),
   `process.env` mutation with `afterEach` cleanup, `vi.mock` + fake timers, no
   subprocess, no `mkdtemp`/`tmpdir`, no path or EOL assumption, no
   `process.platform` reference — and no test is skipped or guarded (`it.skip`,
   `skipIf`, `runIf` → zero hits anywhere in `tests/`). The macOS/Windows rows
   stay "Yes — unverified (OS-neutral path, A12)", which is what the design
   says they are; nothing in the suites would behave differently there, and
   adding a CI matrix leg to verify them is an explicit non-goal. The stale
   gate claim in the build notes is finding 2.

## Findings

1. **[minor]** `tools/orchestration/src/opencode-runner.ts:352` — carried open
   from r2 finding 1, still open at HEAD: `this.name = 'ReplyTimeoutError'` is
   a second observable delta for existing `@bindforge/orchestration` consumers
   that the design's Compatibility matrix (proposal:117) and Rollout #3
   (proposal:138) both still describe as the *one* delta ("the reply-timeout
   failure itself … extended message"). `error.name` now reads
   `'ReplyTimeoutError'` instead of `'Error'`, which changes (a) any consumer
   that branches or logs on `name`, and (b) the CLI's stderr first line, from
   `Error: No response received from OpenCode…` to
   `ReplyTimeoutError: No response received from OpenCode…`, breaking stderr
   scrapers that key on the line prefix. The message contract stays
   byte-identical and the deviation is declared (build-notes deviation 3), so
   it stays minor. Fix: either drop the assignment (`instanceof
   ReplyTimeoutError` and `constructor.name` already identify the class), or
   keep it and list it as a second, prefix-safe documented delta in the
   matrix's "Existing API consumers" row and Rollout #3.
2. **[minor]** `docs/designs/002-add-timeout-flag/03-build-notes.md:52-60` and
   `:129-136` — the platform/gate record for checklist 7 is now false in the
   tree: deviation 1 and the Round-2 paragraph still state that `npm run check`
   "could not be executed" and "must still be run by the caller". It was run
   against this HEAD during this review and is green (ESLint clean, prettier
   check clean, 7 test files / 91 tests passed, vitest 5.0.5, Linux). The
   Linux matrix row is therefore verified, but the artifact a later reader
   trusts says the opposite, and the macOS/Windows rows have no in-tree
   statement of what was and was not exercised. Fix: rewrite deviation 1 and
   the Round-2 paragraph to record the executed gate (Linux + lint/format/test
   status + per-suite counts, Node version if the runner can capture it) and
   keep the two unverified rows explicitly "not exercised — no CI, non-goal";
   do not add a CI leg.
3. **[minor]** `tools/orchestration/src/pipeline/runner.ts:97-113` (with the
   pre-existing `dirtyFiles` parsing at `:84-88`) — the newly added
   ownership-enforcement code routes file paths through git's *text* output and
   back into destructive git/FS calls without NUL delimiting or quote control:
   `git diff --name-only` is split on `\n` (`:99-101`) and
   `status --porcelain` lines are sliced as `line.slice(3).trim()` (`:87`),
   then each entry is passed to `git cat-file -e <rev>:<p>` (`:108`),
   `git restore -- <p>` (`:109`), `git rm -f -r -- <p>` (`:111`) and
   `rmSync(resolve(repoDir, p), { force: true })` (`:112`). Under
   `core.quotepath=true` (a user-settable global that the runner never
   overrides) any non-ASCII design dir, brief or report name comes back
   octal-escaped (`"docs/designs/003-\303\251…"`), and `--porcelain` rename
   entries arrive as `old -> new`; the escaped/garbled string then matches
   nothing (`gitOk` false → `rmSync` with `force: true` swallows the miss), so
   an ownership violation silently escapes the revert while the code reports it
   as handled, or acts on a wrong literal path. Paths containing leading/
   trailing spaces or quotes are mangled the same way by the `.trim()`. This
   is in the reviewed range but outside the design's change set, and the
   timeout path is unaffected, hence minor. Fix: make the git invocations
   machine-readable — `git -c core.quotepath=false diff --name-only -z` plus
   `status --porcelain -z` split on `\0` (renames: take the new path) — and
   keep `resolve()`; the `stagePathAllowed` regexes need no change since git's
   separators stay `/`.
4. **[nit]** `tools/orchestration/package.json:30-32,36` — carried open from
   r2 finding 4: the floor is declared but its anchors still disagree. The
   published floor `>=22.12.0` is a **toolchain** requirement (A21) for a
   package whose shipped `dist/` needs nothing above ES2022, yet `@types/node`
   is `^26.6.5` — four majors past the floor — so `tsc` will keep type-checking
   code against a Node 26 surface that the declared floor rejects (`skipLibCheck:
   true` and the absence of any compatibility lint rule in
   `eslint.config.js` mean nothing catches it), and `README.md` has no
   "Requirements" line telling consumers the floor at all. Fix: pin
   `@types/node` to the floor major (`^22`) or raise `engines` to match the
   type surface, and add one README requirements line naming the floor and that
   it comes from Vitest 5.
5. **[nit]** `tools/orchestration/src/cli.ts:111,130` — carried open from r2
   finding 5: reflection of a rejected env value uses `JSON.stringify` only,
   which escapes quotes, backslashes and C0 controls (including `\x1b`) but
   leaves DEL, C1 controls (U+0080-U+009F, whose 8-bit CSI is a live escape
   introducer on terminals in UTF-8 mode) and all printable non-ASCII raw —
   and the format-gate error path is exactly the path that echoes an arbitrary
   rejected value, including on legacy Windows console code pages where it
   prints as mojibake. Security-r1's rule is satisfied (no raw C0/ANSI byte
   reaches stderr, and `^\d{1,5}$` is the load-bearing defense), hence a nit.
   Fix: reflect a shape rather than the bytes (`<rejected: N chars, not plain
   digits>`) or additionally escape `[\u007f-\u009f]` and non-ASCII after
   `JSON.stringify`; keep the existing `not.toContain('\x1b')` assertions
   (`tests/cli.test.ts:104-108`).
6. **[nit]** `tools/orchestration/src/opencode-runner.ts:348-351` — carried
   open from r2 finding 6: the exported `ReplyTimeoutError` tells library
   consumers to use knobs they do not have — its fixed suffix reads
   `(set --timeout or $OPENCODE_REPLY_TIMEOUT_SECONDS to change it)`, while a
   programmatic caller's knob is `PromptOptions.timeoutMs`. Design-mandated text
   (proposal:72), so the code matches the design and this is a design wording
   note. Fix (design change first): extend the suffix to `(set --timeout,
   $OPENCODE_REPLY_TIMEOUT_SECONDS, or PromptOptions.timeoutMs to change it)`;
   the A5 prefix contract and every current assertion
   (`opencode-runner.test.ts`, `opencode-runner-timeout.test.ts:67-84`) stay
   green because only the suffix changes.

## FYI (out of specialty, no severity)

The POSIX-only process launches the pipeline tooling depends on (`gates.ts:190`
spawns `npm`, which on Windows is `npm.cmd` and resolves the gate to `false`
through `child.on('error')` rather than failing loudly; `runner.ts:842-844`
shells out to `diff`) are **pre-existing at `baseRef`**, not introduced by this
range — see checklist item 1 for the diff evidence. They still mean the shipped
`dist/pipeline/*` is POSIX-only while `engines`/the matrix make no OS claim, so
a README sentence would be cheap; this round deliberately does not re-raise it
as a finding against this change. Also: the range carries the runner's own
self-repair (`R078 .opencode/agents/codereview/code-complexity.md →
code-compatibility.md`) and pipeline-infrastructure edits (`gates.ts`,
`prompts.ts`, `runner.ts`, `tests/pipeline-*.test.ts`) that the proposal lists
as explicit non-goals (proposal:17,23) — range hygiene, already recorded by
`reviews/code/correctness-r3.md`; and `README.md:22`'s `1–3 600` grouping
invites pasting `"3 600"`, which the gate rejects legibly (no finding).
