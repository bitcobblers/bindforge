# Build notes — ADR-006 self-update checkpoint compatibility

Implementation of `02-proposal.md` in `tools/orchestration`. The package's
tests (written from the proposal) pass against this code. Everything below is
faithful to the proposal except the items under "Deviations".

## What landed

- **Reserved-path fence** (`types.ts`, `gates.ts`): `namesReservedSegment()`
  matches path segments case-insensitively against `ESCALATION_FILE`, exact
  `STATE_FILE`, and the temp pattern `STATE_TMP_RE`
  (`.pipeline.json.<8hex>.tmp`). The fence sits at the top of
  `stagePathAllowed`, so stage `allowedFiles` globs can never admit these
  paths.
- **Ownership sweep + authorship skip** (`runner.ts` `enforceOwnership`,
  `state-store.ts`): violation paths inside another design's directory are
  still reverted; violations on the pipeline's own reserved paths are skipped
  when the runner wrote the current bytes (`holdsRunnerBytes`, fed by
  `recordRunnerWrite`/`recordRunnerDelete` in `state-store.ts`), and
  otherwise restored from HEAD — deleting an on-disk directory stand-in first
  so the restore can land either order.
- **Write failure is an escalation cause** (`state-store.ts`
  `CheckpointWriteFailure`, `runner.ts`): `saveState` wraps
  `mkdir`/`writeFile`/`rename` in a `CheckpointWriteFailure` whose message
  stem is `checkpoint save failed:` and names the repair command
  (`rm -r <target> && git checkout HEAD -- <target>`) when the errno is one
  of `EISDIR`/`ENOTDIR`/`ENOTEMPTY`. `drive()`'s catch turns that error into
  `finalizeEscalate(...)` and stops.
- **Escalation-save guard** (`runner.ts` `finalizeEscalate`): the marker is
  always written; if the state save inside escalation also fails, its reason
  is appended to `ESCALATION.md` and the repair command is printed.
- **Split-record detection** (`runner.ts` `statusLines`, `cli.ts`):
  `statusLines(state, escalationMarkerPresent)` emits one line naming the
  split record when the marker exists but the persisted status is `running`;
  `resumePipeline` gate-logs a `terminalSaveFailed` entry quoting the
  previous failure before deleting the marker.
- **Lock hardening** (`runner.ts` `acquireLock`): exclusive create with
  `"wx"`, holder slug/stage/pid in the message, stale-lock clearance with a
  warning and a single retry, and a stable refusal naming the rival run if
  the retry loses the race.
- **Cross-version contract** (`state-store.ts` `loadState`, `StateSchema`
  header comment): unsupported versions are refused with a message naming
  the unsupported version and the two operator exits the code offers
  (restore the committed copy / start a fresh run); the runner never
  rewrites an incompatible checkpoint. The raw read is size-capped
  (`MAX_CHECKPOINT_BYTES`): an over-cap checkpoint is refused with the same
  postures before any bytes are parsed.
- **Operator docs**: `docs/workflows.md` gains a "Self-update checkpoint
  recovery" section covering the repair command, split records, and the
  cross-version refusal.

## Deviations from the proposal

1. **In-process checkpoint repair in `finalizeEscalate`.** The proposal says
   the repair command is _printed_ for the operator. `finalizeEscalate` also
   calls `restoreCommittedCheckpoint()`, which runs `git cat-file -e
HEAD:<cpRel>` and restores the committed bytes in-process only when HEAD
   holds the checkpoint file. Without this, the escalation commit would
   record the planted directory stand-in (`git add -A` sweeps it into the
   commit), corrupting the very checkpoint the operator was told to restore.
   When HEAD does not hold the path (first-run failure), the operator's
   stand-in is left untouched and only the printed command applies.
2. **`saveState` unlinks its temp file on write failure** before rethrowing,
   so a failed rename never strands a `.pipeline.json.<hex>.tmp` that the
   next sweep would have to explain. The proposal does not mention tmp
   cleanup.
3. **Authorship helper API** (`recordRunnerWrite`, `recordRunnerDelete`,
   `holdsRunnerBytes` exported from `state-store.ts`). The proposal describes
   authorship tracking but not its shape; delete-tracking is needed so
   `resumePipeline`'s marker removal is recognised as runner-authored (the
   marker itself lives on a reserved path and would otherwise be swept).
4. **`acquireLock` is exported with defaulted slug/stage.** The tests call
   it with just the repo dir; the defaults (`unknown`/`start`) keep the lock
   record informative for entry saves.
5. **`runPipeline`'s entry save is also guarded**: a `CheckpointWriteFailure`
   at entry routes through `finalizeEscalate` and returns the resulting
   status, mirroring the mid-run rule rather than crashing the CLI.
6. **Fence matching is segment-wise**, not substring: a path _segment_ must
   match the reserved names exactly (case-insensitive), so legitimately named
   files such as `notes.escalation.md.bak` are not fenced by accident, while
   the reserved names are fenced at any depth.

## Notes for reviewers

- At the time these notes were first written no test files had been touched;
  that is no longer true of the full diff under review. Later test commits
  landed the fixes the red-gate section below reported for the test
  engineer — `approveClient` in `checkpoint-integration.test.ts` and
  `runner.test.ts` now models turn growth, `ownership.test.ts` dropped the
  unused `base` binding, and commit `4fc97b3` re-pinned the funnel
  assertions — so the package tests authored from the proposal are the
  acceptance record at HEAD.
- `moves.ts` was intentionally left alone — path rewriting for design
  renames is a non-goal of ADR-006.

## Red-gate diagnosis (fix round)

> Status at HEAD: this section is the diagnosis as of the fix round and
> stands unedited as history. All three test-side items below were closed
> later by test commits (turn-growth stubs, the unused-binding lint fix, and
> commit `4fc97b3`'s funnel re-pins) — see "Notes for reviewers". The next
> seat should not chase them.

`npm run check` in `tools/orchestration` was red on three counts. One is
implementation-side and is fixed; the other two live in test files the coder
stage does not own, so they are reported for the test engineer rather than
patched around in `src/`.

- **Fixed here: `prettier --check` on `src/pipeline/runner.ts`.** The import
  block from `./types.js` listed `namesReservedSegment` before the type
  specifiers; the sort-imports plugin wants value specifiers last. Reordered
  only — `npm run format:check` now reports "All matched files use Prettier
  code style!".
- **Test file, not touched: `tests/pipeline/ownership.test.ts:223`.** ESLint
  `@typescript-eslint/no-unused-vars`: `tmpRepoWithCheckpoint()` destructures
  `const { dir, base } = tmpRepo()` and then returns a different `base` from
  its own `rev-parse`. Dropping the unused binding (or using it) clears the
  lint step that currently short-circuits the whole `check` chain.
- **Test files, not touched: four resume tests fail because their stub client
  never produces turn growth.** `approveClient` in both
  `tests/pipeline/checkpoint-integration.test.ts` and
  `tests/pipeline/runner.test.ts` returns one assistant turn plus one idle
  marker on _every_ `message.list` call, including the baseline read the
  runner takes right after the stage prompt (`runner.ts:1236–1238`). So
  `assistants.length > assistantBaseline` (`runner.ts:1295`) never holds, and
  the deliberate no-new-turn branch (`runner.ts:1305–1312`, "treat it as an
  empty reply") sets `rawText = ''`. Because those fixtures keep the default
  `minNudgeIntervalMs` (20 s) with `stageTimeoutMs: 5_000`, the pacing floor
  cannot fit before the deadline, `nudgeGate()` returns `'over'`, and each
  attempt ends as `stage-timeout` with 0 nudges — twice, then escalate. That
  is the runner's reply contract working, not a defect in it; the package's
  other fixtures model growth instead — `parallel-integration.test.ts:146–173`
  (`visible = max(0, min(lists - 1, prompts))`), and `budget.test.ts` and
  `parallel-escalation.test.ts` additionally set `minNudgeIntervalMs: 0` when
  a 5 s window must reach the nudge path at all.
- **Test files, not touched: two `created[0]` assertions contradict the funnel
  transition.** For a checkpoint whose `lastStage` is `design-review` and
  which carries no `seats` ledger — the run-2-era fixture and the hand-built
  `baseState` — `planPass` sees "no verdict recorded for any seat" and
  `decideNext` moves to `proposal-revise` (`states.ts:105–129`). That is
  pre-existing, documented funnel behavior ADR-006 does not touch (the design
  changes no gates, rounds, or fan-out), and it satisfies Test strategy #11's
  actual guarantee: driving continues _after_ `lastStage` and no refusal
  happens. `expect(created[0]).toContain('design-review:')` asserts a
  different next-stage guess.

Evidence that no `src/` change can close the remaining gap: the four failing
scenarios were re-run against this implementation with a stub that models
turn growth the way `parallel-integration.test.ts` does (temporary probe file,
since removed). The two escalation scenarios passed end to end — the
directory-at-the-checkpoint-path run ends `escalated` with `ESCALATION.md`
naming `checkpoint save failed` and the `git checkout HEAD --` repair, and
the repaired resume reaches `done`; the split-record resume records exactly
one `terminalSaveFailed` gate entry quoting the marker's failure line and
deletes the marker. The two funnel scenarios also reached `done` with
`baseRef` backfilled, no `ESCALATION.md`, and a clean `git status` (the §1
fence let every boundary commit carry the runner's own checkpoint save) — only
the `created[0]` expectation failed.

## Fix round 1 — code review r1 findings

Changes made against the three r1 code reviews (correctness, security,
testing). New deviations from the proposal, in addition to the six above:

7. **`loadState` seeds the §1 authorship record** (correctness r1 #1,
   major): after the schema check, the loader records the bytes it just read
   (`recordRunnerWrite(file, raw)`), so a resumed process starts trusting
   exactly what the last runner left on disk and the §1 claim "dirty ⇒
   runner" survives the process boundary. A stage write/delete afterwards
   still mismatches the record and is still reverted.
8. **Untracked reserved paths restore, not delete** (correctness r1 #2):
   when a tampered reserved path has no blob at `headBefore` and the runner
   recorded bytes for it (new `runnerBytesFor` export in `state-store.ts`),
   the sweep writes those bytes back instead of deleting the run's only
   checkpoint copy.
9. **`LOCK_FILE` added to the reserved-segment set, and the lock is
   re-asserted at stage boundaries** (security r1 #4, `ensureLockHeld` in
   the drive loop): the proposal's reserved set is the three checkpoint-
   adjacent names; adding the lock closes the writer-side gap the §1 trust
   posture leans on (the moves validator already named it reserved).
   A missing/dead-holder lock is re-taken with `'wx'`; a live rival makes
   the run refuse to continue. In real repos the lock stays gitignored and
   invisible to the sweep; in fixture repos without a `.gitignore` the
   sweep treats it like any out-of-ownership untracked file (unchanged
   from pre-fix behavior, where doc-stage sweeps already removed it).
10. **The escalation marker is written atomically and defends its path**
    (security r1 #1/#2): `finalizeEscalate` clears a directory planted at
    `ESCALATION.md` (the sweep cannot see the empty residue), writes
    through a tmp+rename pair, unlinks the tmp on failure, and prints the
    full report to stderr if the marker cannot land at all.
11. **`finalizeDone`'s terminal save gets the §3 `CheckpointWriteFailure`
    guard** (security r1 #3): on failure it restores the committed
    checkpoint and routes through `finalizeEscalate` instead of crashing
    the completion path (this is why `finalizeDone` now takes `cfg`).
12. **`loadState` size-caps the raw read** (security r1 #5): an over-cap
    checkpoint is refused with the restore/fresh-run postures before any
    parse.
13. **Escalation report line deduped** (correctness r1 #5): the appended
    failure line is `saveFailure.message` alone — its stem already begins
    `checkpoint save failed:`.
14. **Doc/notes overstatement fixed** (correctness r1 #3): the cross-
    version refusal names only the two postures the code offers (restore
    the committed copy / start a fresh run); "run the version that wrote
    it" was never in the message and is gone from `docs/workflows.md`.
15. `loadState`'s occupied-path and interrupted-write repair strings now
    interpolate the one `file` value instead of recomputing
    `join(designDir, STATE_FILE)` (correctness r1 #5, second half).

Still open for the test engineer (the test seat's gaps; no test file was
touched in this round): #1 the §4 race test cannot fail under the pre-fix
implementation (pin `O_EXCL` in-process or race real processes); #2 the
live-holder refusal message shape is unpinned; #3 "recorded bytes exist
and differ ⇒ revert" has no case; #4 nothing pins the resume marker
delete (`recordRunnerDelete`) surviving a sweep with a committed marker;
#5 `restoreCommittedCheckpoint`'s restore branch is asserted only through
operator steps; #6 the `checkpoint save failed` line shape is unpinned
(the code-side dedupe landed above, item 13).

> Status at ship: all six were closed by later test commits — #1's fix left
> a residual strictness gap re-filed as a non-blocking minor — with the
> per-item disposition in "Ship state" below. The next seat should not
> chase them.

## Fix round 2 — red `npm run check` (no review reports in this round)

`npm run check` in `tools/orchestration` was red on three counts; all three
were implementation-side, in `src/pipeline/`. No test file was touched. The
package is green at the end of this round: `lint`, `format:check` and
`vitest run` (357 passed, 2 skipped) all exit 0, and `npm run typecheck`
(which `check` does not chain) is clean too.

Gate causes and fixes, in the order the chain reported them:

- **ESLint `no-useless-assignment` at `runner.ts:505`** — `ensureLockHeld`
  initialised `let holder: LockInfo | null = null` although both the `try` and
  the `catch` assign before any read. The initialiser is gone; control flow
  still assigns on every path.
- **`tsc` TS2532 at `runner.ts:1724–1725`** — the moves-failure ledger amend
  indexed `state.results[i]` directly, which `noUncheckedIndexedAccess` makes
  possibly-`undefined`. The loop now takes the element into a local and
  checks it; same lookup, same break, no behaviour change.
- **Two `pass-finalize.test.ts` failures (`git status --porcelain` was
  `' D .pipeline.lock'`)** and one **`checkpoint-integration.test.ts`
  failure (`ESCALATION.md` still present after the resumed repair)**. Both
  are §1 bookkeeping bugs, recorded as deviations 16–18.

New deviations, continuing the numbered list:

16. **A recorded delete outranks a recorded write in `holdsRunnerBytes`**
    (`state-store.ts`). One process routinely does both to the same reserved
    path — `finalizeEscalate` writes `ESCALATION.md` and records its bytes,
    the next `resumePipeline` deletes that marker (§3) — and the old order
    tested the write record first: the read failed on the runner's own
    delete, `holdsRunnerBytes` answered `false`, and the next boundary sweep
    resurrected the marker from `headBefore`, quietly undoing the §3 repair
    and leaving a stale terminal marker next to a live run. The delete set is
    now consulted first, exactly the §1 wording ("…or is absent because the
    runner deleted it"). A stage that re-creates the path still mismatches
    and is still reverted.
17. **The run lock's bytes are recorded as runner-authored** —
    `acquireLock` (both write paths) and `ensureLockHeld` call
    `recordRunnerWrite`, and `releaseLock` calls `recordRunnerDelete`. The
    lock is a reserved name (deviation 9), so any repo that does not ignore
    it shows it to the sweep; without a record the sweep deleted the live
    lock at every boundary and the boundary re-assertion silently re-took it.
    With the record the sweep leaves it alone and restores these exact bytes
    if a stage deletes or rewrites the path.
18. **The runner never stages its own lock** — `stageAll()` is the single
    staging site (`commitStage` and the ownership revert-commit both used a
    bare `git add -A` before). Deviation 9's "the lock stays gitignored and
    invisible to the sweep" only holds when the operator's `.gitignore` says
    so: a repo without that line committed the lock in its terminal commit
    and then left ` D .pipeline.lock` dirty forever. `stageAll` runs
    `git add -A` and immediately unstages `LOCK_FILE` (`git reset -q --`), so
    the lock reaches no commit. A pathspec exclusion
    (`':(exclude).pipeline.lock'`) was tried first and rejected: git exits
    non-zero when a literal pathspec names an ignored, untracked file, which
    is the lock's normal state, so the fence would have depended on the
    ignore file again.
19. Comments that asserted the gitignore premise (`ensureLockHeld`'s doc and
    the drive-loop boundary comment) now state the real posture: the name is
    fenced, the bytes are recorded, and the path is never staged.

## Ship state (2026-11-10)

The code tree at review HEAD is byte-identical to `640c289` (`fix: stop
committing the run lock and honor runner-authored deletes in the ownership
sweep`); later commits land only run records under `docs/designs/`. The
package is green: `lint`, `format:check` and `vitest run` all exit 0 —
357 passed, 2 skipped (the skips are the pre-existing EXDEV mount probe
self-skipping) — and `typecheck` is clean. All five code-review seats
approved at r1c2 with zero blockers/majors. Test strategy #1–#13 are all
present in `tests/pipeline/` (the r1c2 testing seat re-pinned each cite).

Disposition of the six items left for the test engineer after fix round 1
(verified by the testing seat at r1c1, all landed in the test commits since):

1. §4 race pin — the `Promise.all` fixture is gone; `runner.test.ts:408-421`
   pre-writes a live holder and asserts the refusal plus byte-identity of the
   holder, which fails if `{ flag: 'wx' }` is dropped. Residual (non-blocking
   minor): proposal #10's literal "two concurrent `acquireLock` calls" clause
   is still pinned by proxy, not by racing child processes.
2. Live-holder refusal message — pinned by regex on both refusal paths.
3. Content-vs-presence authorship skip — `ownership.test.ts:303`: tampered
   bytes over a path the runner saved are reverted.
4. Resume marker delete surviving a sweep — pinned twice: unit in
   `ownership.test.ts:390+` and the second-resume leg of
   `checkpoint-integration.test.ts` (the integration leg is what pins
   deviation 16's ordering).
5. `restoreCommittedCheckpoint`'s restore branch — asserted directly in
   `checkpoint-integration.test.ts` before any operator repair (file
   restored, planted content gone, no checkpoint-shaped path in the
   escalation commit).
6. `checkpoint save failed` line shape — pinned in `escalation.test.ts:250-255`
   (single stem, not doubled) and the resume gate-log quote.

Non-blocking findings recorded by the r1 panel and **not** fixed in this
change set (each with its suggested follow-up in the report):

- `finalizeDone` still flips the source ADR `proposed → accepted` before the
  guarded terminal save, so a done-boundary `CheckpointWriteFailure` commits
  the accepted ADR next to an escalated record (correctness r1c1 minor,
  carried through r1c2).
- The `finalizeDone` save-guard branch, the `ensureLockHeld` throw paths, and
  `stageAll`'s lock-unstage have no test reaching them (security r1c1/r1c2,
  testing r1c1 #2/#3); they are correct by inspection and exercised only
  incidentally by fixtures.
- The `ESCALATION.md.<8hex>.tmp` marker tmp introduced by deviation 10 is not
  in the reserved-name set, so a killed escalation or a planted copycat of
  that shape can reach a commit (compatibility r1c2 minor).
- A recorded delete outranks a recorded write for the life of the process —
  `recordRunnerWrite` never clears `runnerDeleted`; latent today because no
  sweep runs between the marker delete and a later marker write
  (correctness r1c2 nit).
- `ensureLockHeld`'s boundary refusal throws a plain `Error` out of the drive
  loop (crash, not escalation) — flagged compatibility FYI, matches the
  proposal's silence on lock-loss at a boundary.
