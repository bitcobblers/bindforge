# Code review — security — round 1, pass c0

Range: `git diff 0179375b8d037833c6b6c5150312d88af0162211...HEAD -- ':(exclude)docs/designs'`.
Judged against `02-proposal.md` (ADR-020) and `docs/adrs/020-restart-boundary-for-harness-self-changes.md`.
Gate evidence at 2026-11-13T21:02:48Z: `npm run check`, `cargo fmt/clippy/test` all exit 0 (cargo test reports its 0-test workspace leg; the TS suite rides `npm run check`).

## Checklist walk

1. **External input validated at the boundary** — pass with caveat. `restartRequired` enters the checkpoint grammar as one additive-optional `z.object` (`state-store.ts:68–72`, beside `planStop`); `status`/`version` untouched, matching §2 and A6. Era fixtures pinned to load without the key materialising (`state-store.test.ts`), unknown sibling keys inert (A7). Read-side array size is unbounded, which the design explicitly sanctions (writer-side bounding, r2 revision log). See finding 2 for the one reader-side copy that skips the writer's sanitization.
2. **Path handling** — pass. `touchesExecutableSurface` (`gates.ts:566–570`) is pure prefix matching over repo-root-relative POSIX paths that reach it only as `git diff -z --no-renames --name-only` output (`runner.ts:1852–1857` through `nulPaths`); no normalization, no interpolation, and the `path === prefix.slice(0,-1)` arm catches a planted path that *is* exactly `tools/orchestration/src` (e.g. a symlink file at that name), so traversal-by-name still opens the boundary. No `..` or absolute path can appear in git plumbing output, and the honesty test proves the two prefixes cover every import-reached module and the `PROMPTS_DIR` read root (`executable-surface.test.ts`).
3. **Injection** — pass. All git calls go through `spawnSync('git', args[])` (`runner.ts:188–194`) — the commit body and gate-log lines carry model-authorable path text into argv, never a shell. Model-authored paths and the stage name go through `printable` at record creation (`runner.ts:1861–1868`, `1881–1883`), matching §2's "every copy takes printable" posture. No new regex over user-controlled input (the honesty test's `IMPORT_RE` walks the repo's own source; linear).
4. **Secrets/PII** — pass. New records carry harness paths, a stage name, and a timestamp; the gate-log/commit/summary text adds nothing else.
5. **New dependencies** — none in the diff (no `package.json`/lockfile change).
6. **Filesystem writes** — pass. Pause path follows the terminal-save pattern: guarded `checkpoint()` (tmp+rename) *before* the boundary commit, `restoreCommittedCheckpoint` + escalate on save failure (matching §2 and the ADR-006 §3 guard), and the boundary commit reuses `stageAll` with its lock-file reset.
7. **Failures reject cleanly** — the pause itself rejects cleanly (finalizeRestartBoundary clears the record before escalating when *its own* save fails), but the same guarantee does not hold at the other escalation sites — finding 1.

## Findings

### 1. major — an escalated run can carry the pause's exit code 3 and a stale `restartRequired` record

- **Location:** `tools/orchestration/src/pipeline/runner.ts:3656` (`finalizeEscalate`) with `runner.ts:2287` (`runResult`) and `tools/orchestration/src/pipeline/cli.ts:43` (`exitCodeFor`)
- **Problem:** `finalizeStage`/`finalizeReviewPass` set `state.restartRequired` at window end (runner.ts:1957, 2032), but several escalation paths can fire in the *same* drive iteration before the loop-top check (runner.ts:2551–2556) converts the record into the clean pause:
  - the iteration's own save `await checkpoint(designDir, state)` (runner.ts:3015, after `checkGate` for implement/fix) throws `CheckpointWriteFailure` → caught at runner.ts:3020 → `finalizeEscalate`;
  - an unexpected throw after the record is set → runner.ts:3036 `finalizeEscalate`;
  - a review pass that escalates *and* already ran `finalizeReviewPass(..., commit:false)` (runner.ts:2259), which still records the boundary — e.g. an operator surface commit landing inside the pass window (the case finalizeReviewPass's own comment at runner.ts:2028–2031 claims to cover).

  `finalizeEscalate` never clears the record, so the escalated checkpoint it saves carries `restartRequired` (a split record of exactly the ADR-006 §3 class), and `runResult` returns `{status:'escalated', restartRequired:…}`. `exitCodeFor` checks `done` then the record, so the escalated run exits **3 — the pause code** — directly contradicting §3's guarantee that "`2` keeps its exact current meaning — 'read `ESCALATION.md`' — so no existing driver automates past an escalation by code collision": the `workflows.md` driver loop (`while [ $? -eq 3 ]; do pipeline resume …`) then blindly resumes, and the resume's `restart-boundary-crossed` gate entry (runner.ts:2479) logs the stale record and clears it while the run drives on past an unaddressed escalation. `finalizeRestartBoundary`'s own save-failure branch (runner.ts:3601–3607) already states the right rule — "the record … must not reach the checkpoint the escalation goes on to write either" — but that clearing exists only there. The cli seam tests pin the four combinations separately; the escalated-with-record combination is unpinned.
- **Fix:** clear `state.restartRequired` at the top of `finalizeEscalate` (an escalation supersedes a pause, mirroring the boundary's own §3 route), and harden the mapping order in `exitCodeFor` to map `status === 'escalated'` to 2 before consulting the record. Pin it: a `runner.test.ts` case where the post-window save at runner.ts:3015 is sabotaged (the `checkpoint-integration.test.ts` directory-at-checkpoint trick) after a surface-touching window — assert the escalated checkpoint holds no `restartRequired` and `ESCALATION.md` stands — and a `cli.test.ts` case for escalated+record → 2.

### 2. minor — resume renders loaded checkpoint paths without re-applying `printable`

- **Location:** `tools/orchestration/src/pipeline/runner.ts:2477–2485`
- **Problem:** the `restart-boundary-crossed` gate entry interpolates `boundary.stage` and `boundary.paths` straight from the loaded checkpoint into the gate-log reason (committed into the run's history by the resume commit and printed). Those values are `printable` only when *this* code wrote them; the design's writer-side posture protects this process's records, not a checkpoint written by a later-era runner or hand-edited by an operator — a control byte in such a path would land raw in the gate log and terminal. The diff is inconsistent with itself: `restartBoundaryStatusLines` (runner.ts:3801–3810) wraps `record.stage` in `printable` on the same loaded value, and the boundary commit body/summary sanitize at write time.
- **Fix:** apply `printable(boundary.stage)` / `boundary.paths.map(printable)` at this render site (or sanitize the record once after `loadState`), matching the other two copies of the same value.

### 3. nit — the record cap's elision marker inflates the reported "N harness file(s)" count

- **Location:** `tools/orchestration/src/pipeline/runner.ts:1861–1869`, rendered at 1884–1889, 3570–3580, 3633–3638 (status line), 2483
- **Problem:** `boundedSurfacePaths` pushes the `"N further harness path(s) elided by the record cap"` line *into* `record.paths`, so every renderer that says `${record.paths.length} harness file(s)` overcounts by one in the capped case ("changed 33 harness file(s)" for 32 listed + marker). §2's posture is "first N paths plus the count" — the count should come from the input length, not the marker-padded list. (`boundedGateEvidence` keeps a marker leg too, but nothing renders its `legs.length` as a count, so this shape is new here.)
- **Fix:** keep the true total in a separate count (or exclude the marker from the rendered count).

## FYI (out of specialty)

The honesty test walks the import graph with a regex over string-literal specifiers, so a future computed `await import(expr)` would reach a module at runtime without the walk seeing it; no such import exists today and the design's Test #7 wording matches this implementation — worth remembering if a dynamic loader ever appears under `src/`.

## Verdict

request-changes — one open major (finding 1). Everything else of the security checklist passed, including the surface rule's path handling, the injection posture, and the guarded terminal-write shape.
