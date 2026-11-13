# Code review — compatibility, round 1 dissent pass c1

- Seat: compatibility (checklist `references/compatibility.md`)
- Diff under review: `git diff 48c80bde34da123b8dd0306343c361617d5cb227..HEAD -- ':(exclude)docs/designs'`
  (`docs/workflows.md`, `src/pipeline/cli.ts`, `src/pipeline/runner.ts`,
  `tests/pipeline/{cli,runner,restart-boundary-integration}.test.ts`; 545 insertions, 45 deletions)
- Read against: `02-proposal.md`, `docs/adrs/020-restart-boundary-for-harness-self-changes.md`, the
  runner's gate record (green at 2026-11-13T21:39:26Z)
- Purpose of this pass: check the last revision against MY round-1 findings. Independent read; no other
  seat's report opened.

## Verdict

**approve** — no open blocker or major. All three of my round-1 findings are fixed and pinned; the
residuals I can still see are nits, and the two new ones are corner-case/doc-level.

Checks I ran myself (read-only, the project's own test commands):

- `npx vitest run tests/pipeline/cli.test.ts tests/pipeline/restart-boundary-integration.test.ts`
  → 2 files, **15 passed** (the new integer pins and the driver-loop pin run, they are not skipped).
- `npx vitest run tests/pipeline/runner.test.ts --reporter=verbose` → the three new fixtures all
  execute and pass on Linux: `bounds a record-cap-sized window and keeps every path copy printable`,
  `escalates with no restart record when the save after the window fails`,
  `renders a planted record printable at the crossing and the status line`.

## Dissent check — my round-1 findings

**#1 (major) — the exit-code integers were unpinned. RESOLVED.**
`cli.test.ts` now imports `exitCodeFor` and pins the whole §3 table: `done → 0` (with a record beside
it and without), a standing record `→ 3`, a no-record non-done `→ 2`, `escalated → 2` both with and
without a record (the anti-collision row the matrix promises), and the thrown row `→ 1` through the
real entry point. `restart-boundary-integration.test.ts` now maps every seam result through
`exitCodeFor` — 3, 0, 2, 3, 3/3/0 for the consecutive-boundary run — and the driver-loop test's loop
condition is now `code === 3`, i.e. the suite drives the documented
`while [ $? -eq 3 ]` loop rather than a mirror of the state field. My hazard ("a future swap of 3/2
leaves the suite green while every driver automates past an escalation") no longer holds: a swap in
`exitCodeFor` (cli.ts:53-64) turns four assertions and five integration pins red. Residual, smaller:
see nit A below.

**#2 (minor) — `finalizeEscalate` shipped a standing record into an escalated result. RESOLVED.**
`runner.ts:3743-3754` clears `state.restartRequired` as the first act of `finalizeEscalate`, with the
rule stated at the site ("an escalation is the terminal record, so it supersedes a pause… it holds at
every site"). That covers every escalation exit, not only the two I named: the post-window
`checkpoint` that throws (`:3101`), the unhandled-throw path (`:3111`), a pre-commit gate stop
(`:2658` → `StopDriving`), `decideNext`'s escalate (`:2632`), and the entry-save failure (`:2433`).
Because the clear precedes the terminal `checkpoint()` at `:3786`, the escalated checkpoint itself
drops the key (`JSON.stringify` omits `undefined`), so no later-era or pre-020 reader ever sees a
pause beside `ESCALATION.md`. The new fixture reaches exactly the window I described — the gate
script replaces the checkpoint path with a directory between the window's `finalizeStage` (record set,
`:2015`) and the iteration's save (`:3090`) — and pins `status === 'escalated'`,
`restartRequired === undefined`, `ESCALATION.md` present with the save-failure line, no boundary
commit in the history, and no boundary line in the driver-facing status. Combined with the
`escalated → 2` pin, exit 2 is now pinned for that corner. One corner the sentence in
`docs/workflows.md` now over-promises: see nit B.

**#3 (nit) — the "changed N harness file(s)" count used the padded list. RESOLVED.**
`elisionMarker` (`:1889-1892`) and `surfacePathCount` (`:1897-1917`) make the marker a count line and
take N from the window's true total; every renderer now uses it — the `restart-boundary` gate entry
(`:3654-3658`), the boundary-commit body's stage name, the pause summary (`:3702-3703`), the crossing
entry (`:2545`), `restartBoundaryStatusLines` (`:3901`) — and `recordRestartBoundary`'s warn uses the
pre-bounding `paths.length` (`:1948`). The new cap fixture (34 surface paths, cap 32) pins 32 real
paths + one elision line and asserts `34 harness file(s)` with `not.toMatch(/33 harness file\(s\)/)`
in all three copies — the gate entry, the status line, and the printed driver summary. My wording
concern is closed.

## Compatibility checklist (my specialty), applied to this diff

1. **Platform code vs the matrix.** Nothing in this diff adds OS-specific source. The one new
   platform assumption is test-side: the escalate fixture's sabotage is a POSIX-relative path inside
   an `npm run check` script (`docs/designs/002-x/.pipeline.json`), which runs with cwd = the package
   dir (`gates.ts:485` runs `npm run check` at each `packageDirs` root), so it resolves to the
   fixture's checkpoint path. It is labelled Linux-only in the fixture's own comment; macOS behaves
   the same. Windows stays `no (unverified)` in the matrix. Consistent.
2. **Node floor (`>= 22.23.5`).** New source/test constructs — `String.replace` with `\p{Cc}`,
   anchored regex + `exec`, `??`, `Array.map/filter/slice`, `String.padStart`, `statSync`, top-level
   `await import()` in the test — are all in Node 22. No new primitive; matches the matrix row.
3. **Paths.** Source keeps `join`/`resolve` (`join(designDirRel, STATE_FILE)`,
   `join(repoDir, designDirRel)`); the record's paths stay repo-relative strings from `git diff -z`
   through `nulPaths`. The only concatenation is inside a test fixture's shell string, deliberate and
   commented. Case sensitivity: no new case-insensitivity assumption (the surface predicate and the
   elision marker are exact-match; `SURFACE_ELISION_RE` is anchored so a path that merely *ends* like
   the marker stays a path).
4. **Guarded platform imports.** The one new import is `printable` from `./independence.js` (cli.ts:10)
   — same package, platform-neutral. `cli.ts` importing `cli.js` at test load is safe: `isMain`
   (`:295`) keeps the bin block inert and `detectRepo()` runs only during parse.
5. **Locale/layout reality.** Non-ASCII harness paths survive: `printable` replaces only `\p{Cc}`
   (`independence.ts:443-445`), git's `-z` diff keeps raw bytes, and JS `.sort()` is deterministic for
   which paths the cap keeps. The cap fixture's control-byte name is *sanitized, not dropped*
   (`00-ctl\u0007.md` → `00-ctl .md`, still matched by `00-ctl`), and the planted-record fixture proves
   a loaded record re-sanitizes at the crossing entry and the status line. Good.
6. **Native dependencies.** None introduced.
7. **Tests run on the platforms they claim.** Verified by running the three files: 15 passes in
   `cli`+`integration`, and the three new `runner.test.ts` fixtures execute (not skipped) on Linux —
   the one platform the matrix claims the suite exercises. No `describe.skip`/`runIf` hides them.

## Findings

### A. minor-adjacent, filed as nit — the handler→`exitCodeFor` line is still pinned only by inspection
- Location: `tools/orchestration/src/pipeline/cli.ts:275` and `:288` (`return exitCodeFor(result)`);
  `tools/orchestration/tests/pipeline/cli.test.ts:283-385`
- Problem: after the fix, every row of §3's table is pinned at `exitCodeFor`, and one row (the thrown
  error → 1) is pinned through the real entry point. The two lines that connect a run/resume result to
  the table — the run and resume handlers' `return exitCodeFor(result)` — are reachable only in a live
  CLI, because `main` builds its own client (`createClient(String(args.baseUrl))`) and a test cannot
  hand it the stub the runner tests use. So a future edit that returns a literal from those handlers
  keeps the suite green. Much smaller than round 1 (the integers themselves, and the driver loop that
  reads them, are now pinned), so nit, not the major it was.
- Fix: make the seam injectable — pass the client factory (or a `runner` pair) into `main`/`dispatch`
  as an optional argument — then extend the exit-code describe with `main(['node','pipeline','resume',
  slug])` against the existing `lightStub` fixture asserting 3 for the pause, 2 for the red-gate run
  and 0 for the untouched run. If that seam stays shut for this round, record in `03-build-notes.md`
  that the mapping line is pinned by inspection and the integers by `exitCodeFor`'s tests, so the next
  reader does not over-read the pins.

### B. nit — the split-record restore can put a record-carrying checkpoint back on disk, against the new docs sentence
- Location: `tools/orchestration/src/pipeline/runner.ts:3853` (`finalizeEscalate`'s
  `restoreCommittedCheckpoint`) read against `docs/workflows.md:240-243`
- Problem: the new doc sentence is absolute — "a run that ends escalated … its checkpoint carries no
  `restartRequired` record". The in-memory clear at `:3754` guarantees that for the result, the exit
  code and any checkpoint this process writes. But ADR-006 §3's terminal-save recovery runs *after*
  the clear: when the escalated save fails, `restoreCommittedCheckpoint` lands HEAD's committed copy
  back (and `commitStage` at `:3855` commits it). If the run had crossed a boundary, HEAD's copy is the
  boundary commit's checkpoint — the one that *does* carry the record (Test strategy #4 pins exactly
  that). So the on-disk checkpoint beside `ESCALATION.md` can carry a standing record, and
  `pipeline status` prints both the split-record note (`:3965-3971`) and the
  `restart boundary: … changed N harness file(s)` line, telling the operator to run the resume the
  escalation was supposed to stop. Driver-visible contract is unaffected (exit is computed from
  in-memory state → 2, and the resume clears the record), so this is display/record honesty in a
  corner, hence nit. Reachable only when the terminal save fails on a run that had crossed a boundary.
- Fix: after `restoreCommittedCheckpoint` in the `saveFailure` branch, drop the stale pause from the
  recovered copy — strip `restartRequired` from the restored file (or re-save the cleared state once
  the path is healthy again) — or narrow the sentence in `docs/workflows.md` to name the ADR-006 §3
  exception ("…unless the terminal save itself failed: a split record recovers the last committed
  checkpoint, which may still carry it").

## FYI (out of specialty, no severity)

- `statusLines:3959-3961` adds a filter that hides every `restart-boundary` entry from the
  recent-gates block while a record stands; §3 says that entry "keeps the existing recent-gates block
  honest **without further mechanism**". For a run standing at its second boundary, the first
  boundary's pause entry is filtered too (its `-crossed` twin still reads), so the block no longer
  shows every boundary the run took. Design-fidelity call, not a platform one.
- `surfacePathCount` reads the total back out of the marker, so a model- or operator-authored file
  named exactly `<digits> further harness path(s) elided by the record cap` sitting at cap position
  33 would inflate a status count; and if `MAX_GATE_LEGS` ever changes, a record written under the old
  cap renders an understated count (the record carries no era marker). A separate bounded-count field
  on the record would make both impossible — the checkpoint grammar admits it as an optional field.
- The bin block's rejection handler (`cli.ts:300-303`) is now unreachable — `main` never rejects — and
  prints the message un-`printable`d; harmless dead branch.
