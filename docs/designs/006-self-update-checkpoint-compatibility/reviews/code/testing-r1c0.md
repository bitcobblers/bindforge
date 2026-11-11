# Code review — testing seat, round 1 pass c0

Range: `git diff 846ca090053b43ec99295a7cedafb47910043df9...HEAD -- ':(exclude)docs/designs'`
Judged against: `02-proposal.md` (Test strategy, §1–§4), `docs/adrs/006-self-update-checkpoint-compatibility.md`.
Checklist: `references/testing.md` only. No previous findings to re-check (first pass of this seat).

## Checklist walk-through

1. **Every Test strategy item has a citing test — pass.** All 13 numbered items carry at least
   one `// proves: Test strategy #N` citation: #1 `gates.test.ts:329-381` (10 stage classes ×
   10 reserved paths, plus the neighboring-path case incl. `.pipeline.json-notes.md`); #2
   `ownership.test.ts:239,254,268,286`; #3 `checkpoint-compat.test.ts` (4 cases, fixture names
   pinned by `readdir`); #4 and #5 `state-store.test.ts:226-283`; #6 `checkpoint-writer.test.ts`
   (3 cases); #7 `escalation.test.ts:212-267`; #8 `runner.test.ts:305-325` (`statusLines`);
   #9 `runner.test.ts:332`; #10 `runner.test.ts:355-401`; #11–#13
   `checkpoint-integration.test.ts:196,230,268`. All three fixtures §2 names exist on disk
   (`run-1-shape.json`, `run-2-era.json`, `run-3-live.json`).
2. **Unit + integration layers — pass.** Unit suites extended in place; the integration layer is
   new (`checkpoint-integration.test.ts`, mock client + temp git repos, the
   `parallel-integration.test.ts` shape the proposal names).
3. **Revision-round pinning — n/a / pass.** No prior code-review blocker/major reached this seat.
   The r1 entries in the proposal's revision log are design-review revisions, and each has its
   pin (#1 pins the segment-wise case-insensitive matcher; #3/#11 pin the §2 contract).
4. **Determinism — pass.** No network, no live server, no ports; mock `OpenCodeClient` only,
   `pollIntervalMs: 1`, `stageTimeoutMs: 5_000`; temp dirs via `mkdtemp`; git is called with
   `-c user.name/-c user.email` so no operator config leaks; the `testsGreen` gate is fed a
   fixture `package.json` whose check is `node -e "process.exit(0)"` (local spawn, the pattern the
   existing lifecycle suites use); the lock tests use `process.pid` and pid `2_147_483_647`, both
   deterministic. No new case asserts on wall-clock.
5. **Names describe behavior — pass with one exception** (finding 1: "admits exactly one winner
   from two concurrent acquisitions" promises more than its body proves).
6. **`npm run check` in `tools/orchestration` — green.** Tail pasted below.
7. **Untestable additions / untested new branches — findings.** Three new code paths ship with no
   discriminating test: the §4 race, the `runnerDeleted` half of `holdsRunnerBytes`
   (`state-store.ts:244,253`), and `restoreCommittedCheckpoint`'s restore branch
   (`runner.ts:2687-2704`).

## Findings

### 1. major — `tools/orchestration/tests/pipeline/runner.test.ts:392-401`

The test meant to pin ADR-006 §4 cannot fail under the pre-fix implementation. `acquireLock` is
synchronous (`runner.ts:423`, `writeFileSync(..., { flag: 'wx' })`), so
`Promise.all([tryAcquire(dir), tryAcquire(dir)])` interleaves nothing: the first call creates the
lock file before the second call begins, and the second is refused by the pid-liveness branch
(the winner's own `process.pid` is the holder). Reverting §4 to the old `existsSync` +
`writeFileSync` pair leaves this test green, so the atomic-exclusive-create guarantee — §1's
single-flight premise — has zero coverage. Test strategy #10's third clause ("two concurrent
`acquireLock` calls in a fixture repo admit exactly one winner") is unpinned in practice, and the
test name overstates what the body proves.

Fix: make the race real, or pin the primitive. (a) Race two real processes on one temp dir —
`Promise.all` of two `execFileAsync(process.execPath, ['--import', 'tsx', '-e', ...])` children
that import `acquireLock` and exit 0/1 — assert exactly one exit 0 and that `.pipeline.lock`
holds that child's pid. (b) In-process, pin `O_EXCL` directly: pre-write `.pipeline.lock` with
`{ slug: 'other', pid: process.pid, stage: 'start', at: 0 }`, call `acquireLock`, expect the
throw *and* `readFileSync(...)` byte-identical to that pre-write — a plain `writeFileSync` would
have clobbered those bytes, `'wx'` cannot. Either shape gives the regression a red test.

### 2. minor — `tools/orchestration/tests/pipeline/runner.test.ts:370-381`

§4 promises "a live holder throws with today's message", and the operator story depends on that
refusal being recognizable. The case pins only `not.toMatch(/is not a function/)` and `/lock/i`
— an assertion that the seam exists, not that the contract holds. Any message rewrite, including
the new race-loser message (`runner.ts:462-465`), passes it.

Fix: assert the shape — `expect(message).toMatch(/^another pipeline run holds the lock: slug=\S+ stage=\S+ pid=\d+$/)`
— with the fixture lock written as a full `LockInfo` rather than `{ pid }`, and the same assertion
reused for the race-loser branch.

### 3. minor — `tools/orchestration/tests/pipeline/ownership.test.ts:239-296`

The §1 core case is missing: a stage editing the checkpoint **after the runner saved it in the
same process**. The two relevant cases cover "no recorded bytes → revert" (:239) and "recorded
bytes match → skip" (:286), never "recorded bytes exist and differ → revert". An implementation
that skipped on `runnerBytes.has(path)` instead of comparing content — trusting the map's presence
rather than the bytes — keeps the whole suite green, and that is exactly the tampering path the
fence exists for.

Fix: add a case in the same describe: `saveState(...)` the advanced state, `writeFileSync` a
tampered body over the same path, then `enforceOwnership(dir, base, 'design-review', advanced)`,
and assert `report.reverted` contains the checkpoint and the file is back to `headBefore`
(`lastStage: 'proposal'`).

### 4. minor — `tools/orchestration/tests/pipeline/runner.test.ts:332-352` (and `state-store.ts:244,253`)

Nothing pins §1's "resume's marker deletion … is not restored from `headBefore`", the only
consumer of `recordRunnerDelete`/`runnerDeleted`. In this test the marker is written *after*
`tmpRepo()` commits, so it is untracked: its deletion never appears in `dirtyFiles`, the sweep
never sees it, and `holdsRunnerBytes`'s `runnerDeleted.has(abs) && !existsSync(abs)` branch is
never reached with a real violation. In `checkpoint-integration.test.ts:275-310` the marker *is*
committed (by the escalation commit) and the branch does run, but nothing asserts the marker stays
deleted — so deleting `recordRunnerDelete(markerAbs)` from `runner.ts` would keep every test green
while a terminal marker is resurrected at the next boundary.

Fix: write **and commit** the marker in this fixture before resuming (that is what an escalation
commit does), then after the run assert `existsSync(marker) === false` and that
`git status --porcelain` still lists ` D docs/designs/002-x/ESCALATION.md` — the deletion survived
the sweep. Cheaper unit-shaped alternative: committed marker → `unlinkSync` +
`recordRunnerDelete` → `enforceOwnership(...).reverted` excludes it.

### 5. minor — `tools/orchestration/tests/pipeline/checkpoint-integration.test.ts:275-310`

`restoreCommittedCheckpoint` (`runner.ts:2687-2704`, called at :2765) has two branches and only
the no-op branch is asserted. `escalation.test.ts:212-267` covers "nothing committed at that path
→ leave the planted directory" (`HEAD:docs/designs/002-x/.pipeline.json` does not exist in that
fixture, and :266 asserts the directory is still there). Here the restore branch *runs* — the
checkpoint is in the base commit — but the test then performs its own `rmSync` + `git checkout`
(:298-305), so it cannot tell whether the runner repaired the path or the operator did, and it
never checks the promise the function's own docstring makes: that planted bytes never reach the
escalation commit's `git add -A`.

Fix: assert immediately after `expect(status).toBe('escalated')`: `readFileSync(checkpoint)`
parses and holds the last runner-authored save,
`existsSync(join(checkpoint, 'planted.md')) === false`, and `git show --name-only HEAD` names no
reserved path except `ESCALATION.md`. Keep the operator repair steps for the second resume leg.

### 6. nit — `tools/orchestration/tests/pipeline/escalation.test.ts:248-250`, `tools/orchestration/tests/pipeline/runner.test.ts:348`

The failure line's shape is never pinned, and it currently duplicates itself: `runner.ts:2743`
appends `` `\ncheckpoint save failed: ${printable(saveFailure.message)}` `` while `saveState`'s
message already begins `checkpoint save failed:`, so ESCALATION.md — and the `terminalSaveFailed`
reason that quotes that line — reads "checkpoint save failed: checkpoint save failed: EISDIR …".
`toContain('checkpoint save failed')` cannot see it.

Fix: pin the line once — e.g. `expect(report.match(/checkpoint save failed/g)).toHaveLength(1)`
or `/^checkpoint save failed: (?!checkpoint save failed)/m` — and assert the resume gate-log
reason equals that exact line. (The code-side dedupe belongs to the correctness seat; the test
should pin the shape either way.)

## FYI (out of specialty, no severity)

`runnerBytes`/`runnerDeleted` (`state-store.ts`) are module-level and never pruned — one entry per
checkpoint save for the life of the runner process. Harmless at these sizes; noted for the
correctness/performance seats.

## `npm run check` (tools/orchestration) — tail

Exit code 0.

```
npm notice run @bindforge/orchestration@1.0.0 test
npm notice run vitest run

 RUN  v5.0.5 /home/mike/repos/bindforge/tools/orchestration

mount: /tmp/bindforge-mnt-probe-qf5Ov7: must be superuser to use mount.
       dmesg(1) may have more information after failed mount system call.

 Test Files  33 passed (33)
      Tests  350 passed | 2 skipped (352)
   Start at 21:37:17
   Duration  2.34s (tests 76%, import 20%, transform 3%, worker 1%)
```

(The `mount:` line comes from a pre-existing probe in another suite, not a failure; lint and
`format:check` both passed before it.)

## Verdict

`request-changes` — one open major (finding 1: the §4 race is untested), plus five minor/nit
coverage gaps. Everything the proposal's Test strategy lists is cited and the suite is green; the
gaps are all of the "a regression would keep it green" kind.
