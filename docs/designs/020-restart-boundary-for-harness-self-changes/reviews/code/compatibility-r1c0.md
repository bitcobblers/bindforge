# Code review — compatibility seat, round 1 pass c0

Range: `git diff 0179375b8d037833c6b6c5150312d88af0162211...HEAD -- ':(exclude)docs/designs'`
Design: `02-proposal.md` (ADR-020 restart boundary). Inputs: the diff, the
proposal, the ADR, the runner's gate record, this seat's own (empty) prior.

## Gate evidence used

The runner's gate (2026-11-13T21:02:48Z, green) ran `npm run check` to exit 0;
`check` is `lint && format:check && vitest run` (package.json:23), so the new
suites ran on Linux at the gate. I re-ran the five new/touched vitest files
myself: 84/84 passed. The cargo legs' `test result: ok. 0 passed` line is the
Rust suite, untouched by this change — consistent with the matrix's claim that
the orchestration suite is what exercises this change.

## Compatibility checklist walk

1. **Matrix vs code.** The matrix promises "no new primitive": the change adds
   only git plumbing (`diff -z --no-renames --name-only`, `commit`,
   `diff --cached --quiet`) through the existing `nulPaths`/`git`/`stageAll`
   helpers, one zod optional field, and console lines. No OS-specific branch
   was introduced, so "macOS partial / Windows unverified, nothing added" is
   honest: the Linux supported row has its full code path. Pre-020-reader row
   (A7) is implemented and pinned: additive-optional field, `status` and
   `version` untouched, era fixtures load with no key materialised, sibling
   key inert (state-store.test.ts). Pass.
2. **Node floor (>= 22.23.5).** New constructs: top-level `await import()` in
   test files (ESM TLA, fine), `structuredClone` (17+), `matchAll`, `\p{Cc}`
   with `u`, `fileURLToPath` — all below the floor. No new engines constraint
   needed. Pass.
3. **Paths.** `touchesExecutableSurface` compares git-output paths (always
   `/`-separated, real spelling) against `/`-joined constants — correct, and
   the docstring states it parses no path, matching `stagePathAllowed`. All
   new filesystem paths use `join`/`resolve`. Pass.
4. **Guarded platform imports.** None introduced or required. Pass.
5. **Locale reality.** `-z` output is unquoted UTF-8 (immune to
   `core.quotepath`), so non-ASCII template names reach the predicate and the
   record intact; `printable` replaces only `\p{Cc}` control bytes, preserving
   non-ASCII spellings into the checkpoint, gate log, commit body and status.
   Prefix matching is ASCII-safe over UTF-8. Pass.
6. **Native dependencies.** None added (package.json untouched). Pass.
7. **Tests run where claimed.** Plan claims Linux-only exercise; the gate and
   my own run both executed all eleven strategy tests (#1–#11 all present and
   green on Linux); no platform-conditional skip anywhere. See finding 1 for
   the one strategy test that does not assert what it names. Partial — see
   findings.

## Findings

1. **major — the exit-code contract itself is pinned by no test.**
   `tools/orchestration/src/pipeline/cli.ts:43` (`exitCodeFor`) is the whole
   of Decision 3's driver-compatibility promise: "2 keeps its exact current
   meaning — 'read ESCALATION.md' — so no existing driver automates past an
   escalation by code collision", and the risk table's mitigation is literally
   "distinct codes pinned both ways (pause→3, escalation→2)", naming Test #6.
   Test strategy #6 lists the four integers, but the shipped
   `tests/pipeline/cli.test.ts` asserts only the seam's *shape* (`status`,
   `restartRequired` presence) — `exitCodeFor` appears in no test (grep: only
   cli.ts), and `tests/cli.test.ts` predates and covers the other CLI. A
   future swap of 3/2 keeps the suite green while every documented driver
   loop (`docs/workflows.md`'s `while [ $? -eq 3 ]`) silently automates past
   escalations. Fix: in `cli.test.ts` (or a unit pin), assert
   `exitCodeFor({status:'done'}) === 0`, a standing-record result `=== 3`,
   `{status:'escalated'}` without a record `=== 2`, and that the run/resume
   handlers return `exitCodeFor`'s value — the integers, not just the fields.

2. **minor — an escalated run can still exit 3 while the pause record stands.**
   The record is set in `finalizeStage`/`finalizeReviewPass`
   (`runner.ts:1957`, `runner.ts:~2032`); between that and the loop-top check,
   `drive`'s per-iteration body runs checkpoint saves and escalation exits
   (`runner.ts:3015` inside the try whose catches call `finalizeEscalate` at
   `runner.ts:3020` and `runner.ts:3036`). If one of those fires after a
   surface-hit window, `finalizeEscalate` leaves `state.restartRequired` in
   place, and `runResult` (`runner.ts:2287`) ships
   `{status:'escalated', restartRequired}` — `exitCodeFor` maps the standing
   record to 3, so a run with a committed `ESCALATION.md` exits 3 and the
   documented driver loop auto-resumes it: the matrix's "no existing driver
   automates past an escalation" breaks in this corner, and the escalation
   checkpoint also renders the boundary status line beside the escalation.
   The author already recognized the shape — `finalizeRestartBoundary`'s
   save-failure path explicitly drops the record before escalating
   ("the escalation is the terminal record from here", `runner.ts:3592-3600`)
   — the generic escalation exits just don't apply it. Fix: clear
   `state.restartRequired` in `finalizeEscalate` (`runner.ts:3656`) so the
   terminal escalation never carries a pause, and pin it
   (escalate-after-surface-hit fixture → status 2, no record).

3. **nit — the cap's elision entry inflates the reported file count.**
   `boundedSurfacePaths` (`runner.ts:1861-1869`) appends the elision sentence
   *into* `record.paths`, so for > 32 hit paths the status line
   (`restartBoundaryStatusLines`, `runner.ts:3801`), the summary, and the gate
   reason print `changed ${record.paths.length} harness file(s)` = 33 when the
   true count was, say, 40, and one "file" is a sentence. §2's wording is
   "first N paths plus the count"; the shape matches the existing
   `boundedGateEvidence` posture (runner.ts:3182-3191), so this is a wording
   fidelity nit, not a drift. Fix: take the count from the pre-bounding
   total (`paths.length` before slicing) in the printed lines, or store the
   total on the record.

## FYI (no severity)

The new integration/unit tests compare git-output paths against
`join(...)`-built strings, which equal the `/`-separated git output only on
POSIX separators; consistent with the matrix's Windows row ("no (unverified)",
suite Linux-only), so not a checklist violation — flagging it only so a future
Windows verification round knows these tests are the first to need `split('/')`.

## Verdict

request-changes — one open major (finding 1), one minor (finding 2), one nit
(finding 3).
