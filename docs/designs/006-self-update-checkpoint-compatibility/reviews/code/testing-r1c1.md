# Code review — testing seat, round 1 dissent pass (c1)

Scope: `git diff 4fc97b316349ed487b5ef49430175c04aabc1e61..HEAD -- ':(exclude)docs/designs'`
read against `02-proposal.md` (full current document) and
`docs/adrs/006-self-update-checkpoint-compatibility.md`.
Commits in range: 7f1845f, 273f241, fc2c59e, 640c289.
This pass checks first that the revision answers my round-1 findings; new
findings only where the changed code (mainly 640c289's lock machinery and
273f241's terminal-save guard) makes them true.

## Verdict

approve — zero open blockers/majors. Three minor and one nit carried below.

## r1 finding disposition

1. **[major] `acquireLock` concurrency test could not fail** — **answered,
   residual strictness gap (re-filed as minor #1 below).** The `Promise.all`
   fixture is gone; runner.test.ts:408-421 pre-writes a full live-holder
   `LockInfo`, calls `acquireLock`, and asserts both the refusal message and
   `readFileSync(...) === holder` byte-identity — exactly the in-process pin I
   offered as an alternative r1. It now catches the realistic forward
   regression (dropping `{ flag: 'wx' }` from runner.ts:464: the write
   succeeds, `tryAcquire` returns null, and the byte-identity assert fails).
   The test name ("a refused acquisition never clobbers the holder") now
   matches the body.
2. **[minor] live-holder refusal message unpinned** — **answered.** The
   fixture is a full `LockInfo` (runner.test.ts:379-384) and both refusal
   tests assert
   `/^another pipeline run holds the lock: slug=\S+ stage=\S+ pid=\d+$/`,
   which matches the live-holder throw at runner.ts:480-484. (The racing-loser
   re-read at runner.ts:497-505 stays unreachable in-process — that part of my
   r1 fix suggestion is not implementable without child processes; covered by
   minor #1's suggested child-race fixture if strictness is ever wanted.)
3. **[minor] content-vs-presence authorship skip untested** — **answered.**
   ownership.test.ts:303: `saveState` the advanced state, tamper over the same
   path, `enforceOwnership` reverts and restores `lastStage: 'proposal'`. The
   has(path)-skip implementation would now fail.
4. **[minor] resume marker-delete not pinned** — **answered twice.** Unit:
   ownership.test.ts:390 commits a marker, deletes + `recordRunnerDelete`, and
   asserts the sweep leaves it deleted. Integration:
   checkpoint-integration.test.ts:343-346 asserts the marker stays deleted
   after the second resume in a repo where the escalation commit made it
   tracked — this integration leg is the one that actually pins the
   delete-outranks-write ordering in `holdsRunnerBytes` (state-store.ts:290-303),
   because the first resume's `finalizeEscalate` recorded the marker's bytes
   in the same process. (Unit-level ordering pin: nit #4.)
5. **[minor] `restoreCommittedCheckpoint` restore branch indistinguishable**
   — **answered.** checkpoint-integration.test.ts:312-326 now asserts, before
   any operator repair: the path is a file again, `planted.md` is gone, the
   restored JSON is the committed runner-authored save, and `git show
   --name-only HEAD` names `ESCALATION.md` and no `.pipeline`-shaped path.
6. **[nit] doubled `checkpoint save failed:` stem** — **answered.**
   escalation.test.ts:251-255 pins the line shape
   (`/^checkpoint save failed: (?!checkpoint save failed)/m` plus the not-
   doubled negative), and runner.test.ts:348-351 pins the resume gate-log
   reason to the exact EISDIR line. The code-side dedupe landed at
   runner.ts:2862-2867 (correctness seat's share, done).

## New findings (from the changed code in this pass)

1. **minor** — `tools/orchestration/tests/pipeline/runner.test.ts:408-421`.
   Test strategy #10's third clause — "two concurrent `acquireLock` calls in a
   fixture repo admit exactly one winner" — is still pinned only by proxy:
   against the pre-design `existsSync`+`writeFileSync` pair the new test passes
   unchanged (the exists-check throws on the live holder before any write, so
   the holder's bytes survive just as well). What is now pinned is `'wx'`
   refusal semantics, not cross-process single-flight. Acceptable as the
   accepted in-process alternative, but the proposal clause literally has no
   two-acquisition test. Fix (if pursued): race two `execFileAsync` children
   that import `acquireLock` and exit 0/1; under `'wx'` exactly one exits 0
   deterministically (the assertion is deterministic for the current code),
   and the check-then-write pair admits both.
2. **minor** — `tools/orchestration/src/pipeline/runner.ts:536-574`
   (`ensureLockHeld`) and `runner.ts:353-356` (`stageAll`). The boundary
   re-assertion added for security r1 #4 has four branches — self-held return,
   foreign-live-pid refuse, missing/dead re-take, race-loser refuse — and no
   test reaches any throw. docs/workflows.md:136-142 promises operators that
   "if another live process has taken it, the run refuses to continue"; that
   contract is untested (checklist item 7). Likewise `stageAll`'s lock-unstage
   runs incidentally in the non-gitignored escalation.test.ts / budget.test.ts
   fixtures but is never asserted: checkpoint-integration.test.ts:83-85
   gitignores the lock, so its committed-file scan (:326) cannot catch the
   reset at runner.ts:355 being removed and the lock landing in a run commit.
   Fix: (a) resume-integration case that rewrites `.pipeline.lock` to a live
   foreign pid (`process.ppid`) before resuming and expects the run to end on
   the `lost the pipeline lock at a stage boundary` refusal, not complete;
   (b) in a non-gitignored full-run fixture, assert no commit on the branch
   names `.pipeline.lock` (e.g. scan `git log --all --name-only`).
3. **minor** — `tools/orchestration/src/pipeline/runner.ts:2782-2792`. The new
   `finalizeDone` guard converting a terminal `CheckpointWriteFailure` into
   `restoreCommittedCheckpoint` + `finalizeEscalate` (security r1 #3) is a new
   branch with no test: every directory-at-path scenario in the suite sabotages
   mid-run (checkpoint-integration.test.ts:289-295, drive-loop guard), never
   the done boundary. Dropping the try/catch would crash the completion path
   with no terminal record and leave the suite green — the exact run-2 class
   this design closes. Fix: a checkpoint-integration variant whose sabotage
   client replaces the checkpoint path at the final boundary, asserting status
   `escalated`, an `ESCALATION.md`, and a recorded terminal commit instead of
   an escape.
4. **nit** — `tools/orchestration/tests/pipeline/ownership.test.ts:390-405`.
   The marker-delete unit case never `recordRunnerWrite`s the marker's bytes
   before `rmSync` + `recordRunnerDelete`, so it pins only the plain
   delete-skip; the write-outranked-by-delete ordering (state-store.ts:290-303)
   is pinned today only through the integration leg. Cheap hardening: call
   `recordRunnerWrite(marker, body)` when planting the fixture marker, so the
   ordering is pinned at unit level too.

## Checklist

1. Strategy citations: #1, #2, #4, #10, #13 cited in the changed suites; the
   additions cite their r1 findings/deviations. #10's third clause: partial
   (finding 1). #3/#5-#9/#11/#12 unchanged since r1 and were verified then.
2. Unit + integration layers both present. ✓
3. Rejected r1 major now has a test that fails under the realistic regression
   (flag removal); full pair-revert strictness in finding 1. ✓ (partial)
4. Determinism: tmp dirs, mock clients, no clock/network dependence in the new
   tests; the 66 MB cap fixture is deterministic (state-store.test.ts:281-303). ✓
5. Test names describe behavior. ✓
6. `npm run check` in `tools/orchestration`: green (run this pass).
7. Untested additions: findings 2 and 3.

## `npm run check` tail (this pass)

```
npm notice run @bindforge/orchestration@1.0.0 test
npm notice run vitest run

 RUN  v5.0.5 /home/mike/repos/bindforge/tools/orchestration

mount: /tmp/bindforge-mnt-probe-D1FUj2: must be superuser to use mount.
       dmesg(1) may have more information after failed mount system call.

 Test Files  33 passed (33)
      Tests  357 passed | 2 skipped (359)
   Start at 22:17:26
   Duration  2.48s (tests 77%, import 19%, transform 3%, worker 1%)
```

Exit code 0. The mount line is pre-existing probe noise from a skipped test.
