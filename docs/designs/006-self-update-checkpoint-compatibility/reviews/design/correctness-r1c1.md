# correctness review r1 (dissent c1)

verdict: approve

Scope: the blob diff da725a4..305aa36 of `02-proposal.md`, then the full current
document. Inputs per caller: proposal, ADR-006, revision log, my r1 findings.

## Checklist pass

1. Assumption register: A12 flipped to `verified`; fetched
   https://nodejs.org/api/fs.html — the docs do document Windows-specific open
   flag handling (`'w'`/`O_EXCL|O_CREAT` mapped to `CREATE_NEW`, "the behavior
   of some flags are platform-specific", the `'x'`/O_EXCL caveat) and point
   `fs.rename`/`renameSync` at POSIX rename(2). Basis supports the row. Repo
   glob confirms no `.github/workflows/`, supporting both the A12 basis and the
   Test-strategy Platform claim.
2. URL spot-check: nodejs.org/api/fs.html verified (also confirms the A6
   "exclusive flag might not work with network file systems" sentence).
   pubs.opengroup.org rename(2) row (A5) unchanged by this diff; verified r1.
3. Internal contradictions: §1 authorship test, §3 escalation path, Risks row 1,
   matrix, and Test #2 are mutually consistent after the revision. One citation
   of existing behavior is false (finding 1).
4. Edge cases: the tampered-but-untracked checkpoint case is unhandled
   (finding 2). Planted tmp/dir, marker delete, marker write→commit (no sweep
   between writeFileSync at runner.ts:2587 and commitStage at 2600) all check
   out against runner.ts:260–276, commitStage (git add -A at :327), and
   dirtyFiles (`--untracked-files=all --no-renames` at :209).
5. Interfaces: names consistent (`CheckpointWriteFailure`, `terminalSaveFailed`,
   ESCALATION.md path, `.pipeline.json.<8 hex>.tmp` matches
   `randomBytes(4).toString('hex')` at state-store.ts:186; v99 regex
   `/unsupported pipeline state version/` at state-store.test.ts:125 still
   passes with the appended sentence).
6. ADR questions 1–3 all decided (fence+authorship; rule set+fixtures;
   escalation survives failing save + split record). Deferred items consistent
   with ADR-006 Decision; none re-opened.
7. Previous blocker/minor/nit closed for real:
   - Blocker (sweep reverts runner's own checkpoint writes): the new
     authorship test (skip while a match holds the bytes `saveState` last
     wrote, or is absent because the runner deleted it) fixes all three
     failure modes I cited — boundary commits advance (dirty checkpoint skipped,
     committed by commitStage's `git add -A`), the untracked first checkpoint
     is skipped not removed, and resume's marker delete survives. Test #2's
     second clause now pins the correct behavior instead of the misfire
     output. The mechanism differs from my suggested fix but achieves it; the
     claimed line anchors (runner.ts:260–261, 263–276, 1871, 2298, 2336,
     1983–1989, 1938) all check against the code.
   - Minor (ADR-006:55–57 posture pair): §2 now names the ADR's actual pair —
     restore the committed copy (`git checkout HEAD -- …`) or a fresh run —
     and attributes it correctly.
   - Nit (tmp prefix breadth): matcher is now segment-wise equality or the
     exact tmp shape; `.pipeline.json-notes.md` stays allowed and Test #1
     pins it.

## Findings

- [minor][new] §1 Fence — "compared segment-by-segment and case-insensitively
  the way the moves validator already compares these names (`moves.ts:134–147`)"
  — the cited validator compares `STATE_FILE`/`LOCK_FILE` segments with plain
  `===` (moves.ts:136, 139), case-SENSITIVELY; only `.git` uses
  `toLowerCase()` (moves.ts:142). The claimed existing precedent for
  case-insensitive comparison of these names is false, and the revision log
  repeats the same mis-citation. The choice itself is fine (Test #1 pins
  `.PIPELINE.JSON`), but the Non-goals keep moves.ts:134–147 unchanged, so the
  matcher cannot "share" that comparison as the sentence implies. Fix: reword
  to "segment-wise like the moves validator (moves.ts:134–147), with
  case-insensitivity added here (the validator's only case-insensitive name is
  `.git`, moves.ts:142)", and correct the matching revision-log entry.
- [minor][new] §1 Fence consequences — "A stage write to the checkpoint is a
  content mismatch, reverted before that stage's `git add -A`" + "a new run's
  untracked checkpoint is committed by its first `commitStage`" — for a new run
  before its first `commitStage`, the checkpoint is absent from `headBefore`,
  and the cited revert (runner.ts:263–276) *removes* violations that are not in
  `headBefore`: a tampered untracked checkpoint is therefore deleted outright —
  no copy in HEAD, worktree, or memory-to-disk until the next `checkpoint()`
  re-save (runner.ts:2336) restores it from memory; a crash in that window
  (or a `commitStage` with no other dirty files) leaves nothing for §3's
  `git checkout HEAD --` repair to restore. The unconditional claim about the
  first commit is false in exactly the tamper case the fence targets. Fix:
  extend the authorship branch — a reserved-name match with a content mismatch
  that is absent from `headBefore` is re-saved from the in-memory PipelineState
  via `checkpoint()` instead of removed; pin with a Test #2 case.

## FYI (no severity)

- The authorship test requires the runner to remember in-window bytes and
  runner-deletion facts; the resume process must seed that record (e.g. from
  the bytes `loadState` just read) or its first sweep reverts a legitimately
  ahead-of-HEAD worktree checkpoint to `headBefore` until the first save
  rewrites it — transient and self-healing, but worth pinning for the
  implementation/test-strategy seats.
