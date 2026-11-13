# Code review — security, round 1 (dissent pass c1)

Range: `git diff 48c80bde34da123b8dd0306343c361617d5cb227..HEAD -- ':(exclude)docs/designs'`
(6 files: `docs/workflows.md`, `src/pipeline/cli.ts`, `src/pipeline/runner.ts`,
`tests/pipeline/cli.test.ts`, `tests/pipeline/restart-boundary-integration.test.ts`,
`tests/pipeline/runner.test.ts`), read against the full current
`02-proposal.md` and `docs/adrs/020-restart-boundary-for-harness-self-changes.md`.
Gate at 2026-11-13T21:39:26Z: green (`npm run check`, `cargo fmt --check`,
`cargo clippy -D warnings`, `cargo test --workspace --locked`).

Verdict: **approve**.

## Dissent check — my round-1 findings

**1 (major) — resolved.** An escalation reached in the same drive iteration that
set `restartRequired` used to carry the pause into the escalated record and the
exit code. Both halves of the requested fix are in place:

- `runner.ts:3743-3754` — `finalizeEscalate` clears `state.restartRequired` as
  its first action, before the terminal save, so the escalated checkpoint, the
  widened result, and `ESCALATION.md` agree that the escalation is the terminal
  record. `finalizeRestartBoundary`'s own save-failure branch keeps its explicit
  clear at `runner.ts:3668-3681` (comment now states the general rule).
- `cli.ts:53-64` — `exitCodeFor` answers `escalated → 2` *before* consulting the
  record, so even a checkpoint from the pre-fix revision cannot map to 3.

I verified the invariant rather than trusting the two sites: the record is set
at exactly one place (`runner.ts:1945`) and cleared at three (`2551`, `3673`,
`3754`), and `status = 'escalated'` is written only at `3772`, after the clear.
Every `finalizeEscalate` call site I checked either `return`s or throws
`StopDriving` (`2433`, `2441`, `2632`, `2658`, `2690-2697`, `2729`, `2933`,
`3101`, `3111`, `3601`, `3675`), so clearing never lets the stale process open
another window — ADR-020's liveness property survives the clear.

Pins: `runner.test.ts:1161` (the `check` gate sabotages the checkpoint path, so
the *post-window* save at `runner.ts:3090` throws after the window set the
record; asserts `result.restartRequired` undefined, no boundary commit anywhere,
restored checkpoint with no record, no `/restart/i` in the escalated status
output — the fixture is non-vacuous because the same stub with a green gate is
pinned pausing clean); `cli.test.ts:323` (escalated → 2 with the record beside
it or not); `restart-boundary-integration.test.ts:454` (`exitCodeFor(result)` is
2 for the escalated run) and `:476` (3 for the pause beside a red gate, where the
loop-top check at `runner.ts:2619` precedes `decideNext`, so that run ends
paused, not escalated — no conflict with §3's outranking).

**2 (minor) — resolved.** `runner.ts:2536-2551` re-applies the writer-side
posture to the loaded record at the crossing entry (`printable(boundary.stage)`,
`boundary.paths.map(printable)`), matching `restartBoundaryStatusLines`
(`3900`). Pinned at `runner.test.ts:1270`, where the load-bearing leg is
`expect(rendered).not.toContain(planted.stage)` plus the per-line control-byte
check on the status render. For the record: my r1 wording overstates the escape —
`gateLog` sanitizes `reason` at write (`runner.ts:2150`), so the crossing entry
was already printable on disk; the render-site copy is belt, not the hole. The
status-line copy was the real gap and is now both fixed and pinned.

**3 (nit) — resolved.** `surfacePathCount` (`runner.ts:1894-1917`) is the single
source for every "N harness file(s)" line: the `restart-boundary` gate entry
(`3654`/`3658`), the driver summary (`3703`), the status line (`3901`) and the
crossing entry (`2548`); the window-time notice uses the unbounded source length
(`1948`, `paths.length` before `boundedSurfacePaths`). The marker text is now one
named function (`elisionMarker`, `1889-1892`) and is recognized only by an
anchored regex at the cap position, so a planted path that merely ends like the
marker stays a path. Pinned at `runner.test.ts:938`: 34 surface paths render as
34 files in the gate entry, the status line and the captured console summary, the
stored list is 32 real paths plus one marker, and every copy is control-byte
free (the `0x07` probe sorts first, so the sanitization leg is not vacuous).

## Security checklist over the changed code

1. **External input at the boundary** — unchanged and adequate:
   `RestartBoundarySchema` (`state-store.ts:69-73`) validates `stage`/`paths`/
   `at` as typed, not coerced, so the new render sites' `string` assumptions
   come from the schema; a rejected shape loads nothing and writes nothing.
2. **Path handling** — the boundary commit body interpolates the printable'd
   path list into a `git commit -m` element (`runner.ts:3686-3697`); `git()` is
   `spawnSync('git', args)` with no `shell` (`runner.ts:188-190`, `233-235`), so
   a hostile file name cannot become a command. No `..`/absolute-path handling
   is introduced. The `checkScript` splice in the new test helper
   (`runner.test.ts:666-683`) is fixture-authored text inside the fixture's own
   `package.json`, with the quoting constraint stated; product code builds no
   shell string here.
3. **Injection** — `SURFACE_ELISION_RE` (`1895`) is anchored, literal apart from
   `(\d{1,9})`, no nested quantifier: no catastrophic backtracking, and it runs
   only on schema-validated strings. Reading a marker from a *loaded* record can
   only inflate a printed number (≤ 32 + 1e9); it drives no write, no path, no
   branch decision.
4. **Secrets/PII in logs** — the new `main` catch prints one `printable` line
   (`cli.ts:160`); the harness's throw messages carry the base URL and paths,
   never the basic-auth password, and no new log line names a credential.
5. **Dependencies** — none: the diff touches no manifest or lockfile.
6. **Filesystem writes** — no new primitive; the pause/escalate saves keep
   tmp+rename, and the record is cleared *before* the escalated save, so a
   successful escalated write also carries no stale pause record.
7. **Clean rejection** — improved: an invocation whose run never starts (no
   checkpoint, unreadable one, bad knob) now answers exit 1 with one printable
   line instead of a stack trace, on the same channel as the table's other rows
   (`cli.ts:149-163`, split from `dispatch` at `166`). I checked that throws
   before the lock (`resumePipeline`'s `loadState`, `runner.ts:2451-2454`) leave
   no lock behind — release is in the `finally` at `2565-2567`.

## Findings

1. **nit** — `tools/orchestration/src/pipeline/cli.ts:297-304`. `main`'s new
   catch documents `printable` as "the posture every other operator-facing copy
   takes", but the sibling rejection handler on the `isMain` path still prints
   `error instanceof Error ? error.message : error` raw. It is nearly
   unreachable now (`main` catches everything; it fires only if the catch itself
   throws, e.g. `String(error)` on an object with a throwing `toString`), so this
   is consistency, not an open hole. Fix: `console.error(printable(error
   instanceof Error ? error.message : String(error)))` in that handler, or delete
   it once `main` is the only entry point.

## Out of specialty (FYI, no severity)

The `statusLines` recent-gates filter (`runner.ts:3952-3961`) drops *every*
`restart-boundary` entry while a record stands, including an older crossed
boundary's entry in a multi-boundary run; the checkpoint's gate log and the
boundary commits still carry them, and the standing line renders the live
record — a display choice for the compatibility seat.
