# Code review — performance seat, round 1 pass c0

Diff under review: `git diff 0179375b8d037833c6b6c5150312d88af0162211...HEAD -- ':(exclude)docs/designs'`
Judged against: `docs/designs/020-restart-boundary-for-harness-self-changes/02-proposal.md` and ADR-020.
Gate record (2026-11-13T21:02:48.947Z): `npm run check` green; cargo fmt/clippy/test legs green.

## Checklist evaluation

1. **Event/keystroke path — no sync fs/network, no per-event rebuilds, no avoidable allocations.**
   No new work enters any per-event path. The detection adds one synchronous
   `spawnSync git diff -z --no-renames --name-only <headBefore>..HEAD` per
   *finalize window* (`surfacePathsInWindow`, runner.ts:1846; called once each
   from `finalizeStage` runner.ts:1957 and `finalizeReviewPass` runner.ts:2032),
   not per commit, seat, or attempt — exactly the design's chosen "one net-diff
   check per window" (revision log r2). `spawnSync` matches the git-helper
   posture the finalize path already uses everywhere (runner.ts:188), so no new
   class of blocking primitive is introduced. I ran the leg's own command on the
   branch's feat window: `git diff -z --no-renames --name-only 507f6e5..893e7f2`
   yields 17 NUL-separated path entries — name-only output over a one-window
   range, milliseconds against the minutes-long window it closes (A12). The
   drive-loop check (runner.ts:2551) is an O(1) field test per iteration.
   The only allocation nit is gates.ts:568 (finding N1).

2. **Complexity.** `touchesExecutableSurface` (gates.ts:566) is `Array.some`
   over two constant prefixes — O(1) per path, no per-call table construction.
   `surfacePathsInWindow` is filter + sort: O(n log n) in the number of paths
   the window touched, bounded by what one stage committed. No O(n²) over any
   unbounded product; no identity lookup that should be a map/set. Pass.

3. **Unbounded growth.** The new state is a single record, not an array:
   `state.restartRequired` holds one `RestartBoundary`; the path list is bounded
   writer-side to `MAX_GATE_LEGS` (32) entries plus one count-bearing elision
   line (`boundedSurfacePaths`, runner.ts:1861), mirroring `boundedGateEvidence`
   — so the checkpoint cloned into every later save, the gate-log entry, the
   commit body, and the status line all carry a capped string list. The per-run
   boundary count is uncapped, but that is the proposal's explicit v1 posture
   ("Deferred: a per-run cap"; risk table row 4) and each boundary is
   gate-logged. Pass (documented-bounded).

4. **Startup.** `PROMPTS_DIR` was already computed at module load as an
   unexported const; exporting it adds nothing to boot (cli.ts:37). The new
   honesty test (`executable-surface.test.ts`) walks the import graph only at
   suite time — a BFS of ~O(files) `readFileSync` + regex passes over the
   package's sources, test-only cost. Pass.

5. **Cache invalidation logic present and covered by a test.** The staleness
   being invalidated is the process's own imported modules and stage-time
   templates. Detection sits at each finalize window; the record is cleared at
   `resumePipeline` entry (runner.ts:2474–2484) and the loop-top check refuses
   further windows while it stands. Covered: window classification both ways
   (src/template hit, docs-only miss, fully-reverted miss), clear-on-resume and
   drive-on, `statusLines` boundary line only while the record stands,
   checkpoint round-trip and inert-key load, exit-code seam — all present in
   the diff's test files and green in the gate record (`npm run check` exit 0).
   Pass.

6. **Benchmark-like tests / timing notes.** None ships in this diff, so there
   is no in-diff benchmark to run. The design's cost claim (A12: one extra
   process start of seconds against minutes-long windows) is self-declared
   unverified and the chosen test strategy contains no timing leg. I ran the
   added leg's command directly instead (finding under: 17 paths for the feat
   window, name-only output); an attempt to loop-time it was rejected by this
   seat's shell allowlist, so I quote the leg's shape and output size, not a
   wall-clock number. The mechanism's cost is structurally bounded (one
   subprocess per window, capped output); no finding raised.

## Findings

- **N1 — nit — `tools/orchestration/src/pipeline/gates.ts:566-569`**
  `touchesExecutableSurface` evaluates `prefix.slice(0, -1)` on every call,
  allocating two fresh strings per candidate path even though both exact-match
  spellings are constant. Not hot-path-critical (the predicate runs O(tens) of
  times per finalize window), so this is hygiene only. *Fix:* hoist the
  slash-stripped spellings to module constants beside
  `SURFACE_SRC_PREFIX`/`SURFACE_PROMPTS_PREFIX` and compare against them.

## Out-of-specialty FYI

- The gate record's cargo test leg reports `0 passed` — the Rust workspace
  carries no tests; the exercised suite is the TS one (`npm run check`, green).
  Noted for the panel's record, not scored here.

## Verdict

**approve** — zero blocker/major findings; one nit (N1).
