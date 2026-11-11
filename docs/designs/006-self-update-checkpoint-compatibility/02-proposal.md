# Design: A trustworthy, cross-version-resumable checkpoint (ADR-006)

## Context

ADR-006 owns exactly three questions and no others: what a checkpoint written
by one runner version guarantees when a later version resumes it; whether the
worktree copy of `.pipeline.json` is trustworthy given every document/review
stage may write anywhere under `docs/**`; and what the runner does when the
checkpoint cannot be written (source:
[ADR-006](../../adrs/006-self-update-checkpoint-compatibility.md),
[research](./01-research.md)). Today none of the three has a mechanism:
cross-version behavior is the accidental discipline of optional fields
(`state-store.ts:21–27`), the checkpoint sits inside every doc stage's write
set (`gates.ts:260–273`), and `finalizeEscalate` writes its terminal record
through the same failing save path (`runner.ts:2565–2606`). This design
answers all three with four small changes in
`tools/orchestration/src/pipeline/` and deliberately no checkpoint schema
change: `version` stays `1`, following ADR-003's additive rule
(`docs/adrs/003-parallel-inference-lane.md:115–116`).

## Goals / Non-goals

- Goals:
  - Make every stage write to the checkpoint (or its tmp shape, or
    `ESCALATION.md`) a deterministic ownership violation, so the worktree copy
    and the committed copy can only carry runner-authored bytes (Q2).
  - State the cross-version resume contract as a rule set with tests —
    guaranteed loads, well-messaged refusals, golden fixtures for every
    historical checkpoint shape (Q1).
  - Guarantee a terminal record even when the checkpoint cannot be written:
    escalation survives its own failing save, and the split record is visible
    to `status` and explained by `resume` (Q3).
  - Close the lock's check-then-write race with an atomic exclusive create, so
    the "runner is the only writer" story holds across processes.
- Non-goals (each prevents scope creep at code review):
  - Fan-out, review rounds, gates, artifact contract — excluded by ADR-006
    ("Nothing here changes fan-out, review rounds, gates, or the artifact
    contract").
  - A schema v2 or a migration ladder: no breaking change is pending; v1 is
    kept by contract, not machinery.
  - A lock-ID nonce (Terraform `force-unlock` posture) or multi-run
    concurrency — see Deferred.
  - Relocating the checkpoint path — rejected below, and reserved segments in
    `moves.ts:134–147` stay as they are.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | Every document/review stage may write any `docs/**` path today, and coder stages may write anything except tests — the checkpoint path included | verified | gates.ts:260–273 |
| A2 | No stage prompt instructs writing `.pipeline.json` or `ESCALATION.md`; only the runner writes them | verified | grep of `tools/orchestration/prompts/` finds no reference; runner.ts:457–460, 2587–2591; state-store.ts:209–229 |
| A3 | `enforceOwnership` reverts every dirty or committed path outside the matrix, so fencing a path turns any stage write into a revert before that stage's `git add -A` | verified | runner.ts:233–294; finalizeReviewPass runs the sweep before `commitStage` (runner.ts:1654–1675, 316–338) |
| A4 | `escalationMarkdown` renders the last denied gate-log entries from the in-memory state, so the escalate reason survives a failed checkpoint save | verified | report.ts:218–225; gateLog before save at runner.ts:2576–2585 |
| A5 | POSIX `rename` fails `EISDIR`/`ENOTEMPTY` when a directory sits at the target; Node raises — a replaced checkpoint path is not repairable inside the write call | verified | https://pubs.opengroup.org/onlinepubs/9699919799/functions/rename.html ; https://nodejs.org/api/fs.html |
| A6 | Node's `'wx'` flag is an atomic exclusive create; exclusivity is not guaranteed on network file systems | verified (primitive) / unverified (operator FS class) | https://nodejs.org/api/fs.html ; research "Security landscape lens" |
| A7 | The committed copy is the newest good state at every stage boundary — terminal saves land before the commit — and is already the designated recovery anchor | verified | runner.ts:2554–2556, 2585, 2600–2605; state-store.ts:126–141; ADR-006:55–57 |
| A8 | Version refusal (`version !== 1`) is today the only cross-version behavior; the sole backfill on resume is `baseRef` | verified | state-store.ts:145–148; runner.ts:1990–1998 |
| A9 | Historical checkpoints (run 1 shape, run 3 live file with seats and reply-less failure records) parse under today's schema, so golden fixtures can pin them | verified | state-store.test.ts:171–192; docs/designs/003-parallel-inference-lane/.pipeline.json:399–409 |
| A10 | One pipeline process per repo is the invariant; today's `acquireLock` is check-then-write and admits two winners in a race | verified | docs/adrs/003-parallel-inference-lane.md:101–102; runner.ts:383–404 |
| A11 | The Node `'wx'`/rename primitives used need nothing above the existing `>= 22.23.4` engine floor | verified | tools/orchestration/package.json:30–31 |
| A12 | Node documents Windows-specific open-flag behavior (including `'wx'`/O_EXCL) and defers `rename` to POSIX rename(2); nothing in this repo exercises Windows | verified | https://nodejs.org/api/fs.html ; repo glob finds no `.github/workflows/` |

## Options considered

**Relocate the checkpoint to the repo root (the `.pipeline.lock` precedent).**
Rejected on two counts. It breaks the invariant that the terminal status is
saved BEFORE the commit so it lands inside it (A7; research open question 2),
and a root path is inside `implement`/`fix` stages' write set anyway
(`stagePathAllowed` admits anything but tests — A1), so relocation does not
establish trust; only fencing the write set does. The `LOCK_FILE` precedent
protects against model-initiated *moves*, not writes
(`tools/orchestration/src/pipeline/moves.ts:134–147`).

**Pin the checkpoint to the code revision that wrote it (Step Functions
posture).** Rejected: it contradicts the ADR's Decision — the whole point is
that the new runner resumes the old checkpoint. The startup code-freeze
(ADR-003 §Self-update) already pins each *run* to its frozen code; extending
that to resume would forbid the scenario the ADR exists for.

**Explicit migration-on-read with a version bump.** Rejected for now: no
breaking change is pending, and `version: z.literal(1)` means a bump orphans
every in-flight checkpoint by design (research Risks). The refusal seam at
`state-store.ts:145–148` is where a future ladder would mount; deferred.

**Digest-anchored trust inside the checkpoint (a content digest the loader
verifies).** Rejected: a digest stored inside a model-writable worktree is
forgeable by exactly the writers the fence must exclude, and the run-2
postmortem's "digest-anchored trust" appears only in an escalation chain that
never completed — no accepted design exists in this tree (research Risks).

## Chosen design

### 1. Fence runner-reserved paths in the ownership matrix — Q2

`stagePathAllowed(stage, path)` (`gates.ts:260–273`) gains a rule that precedes
the per-stage rules: return `false` when any `/`-separated segment of the path
is `STATE_FILE` (`.pipeline.json`), matches the `saveState` tmp shape
(`.pipeline.json.<8 hex>.tmp`, `state-store.ts:186`), or is `ESCALATION.md`,
compared segment-by-segment and case-insensitively the way the moves validator
already compares these names (`moves.ts:134–147`). The reserved names are
exported from `types.ts` next to `STATE_FILE`/`LOCK_FILE`.

The sweep is author-agnostic — `enforceOwnership` filters every touched path,
runner saves included (runner.ts:260–261, A3) — so it pairs the name rule with
an authorship test: a match is skipped while it still holds the bytes
`saveState` last wrote for it, kept in memory (or is absent because the runner
deleted it), and reverted otherwise (runner.ts:263–276). Consequences:

- A stage write to the checkpoint is a content mismatch, reverted before that
  stage's `git add -A`: neither copy can carry model-authored bytes again, and
  the run-2 "tampering or sanctioned write" finding stops being true because
  the only sanctioned bytes are the runner's own snapshot.
- The runner's own save is skipped, so boundary commits keep advancing the
  committed checkpoint (A7) even though the file is dirty at every sweep (drain
  at runner.ts:1871; `checkpoint()` at 2298, 2336); a new run's untracked
  checkpoint is committed by its first `commitStage`, and resume's marker
  deletion (runner.ts:1983–1989) is not restored from `headBefore`.
- A planted `.pipeline.json.<hex>.tmp` or directory named `STATE_FILE` matches
  nothing the runner wrote, so the sweep removes it and restores the file,
  keeping `git add -A` clean (state-store.ts:192–201).

Trust posture (the answer to Q2): the **worktree copy is the working state,
trustworthy inside a run by deterministic fencing plus single-flight** (§4);
the **committed copy remains the recovery anchor** when the worktree copy is
unreadable (A7). Pre-fence checkpoints (runs 001–007) stay a Risks item.

### 2. The cross-version resume contract — Q1 (rule set + fixtures, no mechanism)

Canonical statement, written into the `StateSchema` header in
`state-store.ts` and into `docs/workflows.md` for operators:

- **Guaranteed:** any runner loads and resumes any `version: 1` checkpoint
  written by an earlier runner. Evolution rules: new fields are added optional
  with behavior-safe defaults; no field is ever removed, retyped, renamed, or
  re-interpreted — `updatedAt` keeps "when this snapshot was taken"
  (state-store.ts:159–174) and `stageCommits` keeps its bounded-window meaning
  (runner.ts:345–355, types.ts:207–214) as named contract, not comment.
  Backfill at resume is used only where absence changes behavior (the
  `baseRef` precedent, runner.ts:1990–1998). This resolves research open
  question 7: re-interpreting old numbers under new control flow is forbidden
  by contract, and Test strategy #3 pins it per era.
- **May refuse:** an unknown `version` value, or a shape the schema rejects.
  Refusal never writes or deletes anything (already true: `loadState` is
  read-only), and its message follows the existing recovery-hint style: name
  the supported version plus the ADR's own two interim postures — restore the
  committed copy (`git checkout HEAD -- …`) or start a fresh run
  (ADR-006:55–57). Today's bare `unsupported pipeline state version`
  (state-store.ts:147) gains that wording; the v99 test's regex still passes.
- **A version bump is a separate, deliberate step:** introducing `2` requires
  per-version readers at the `state-store.ts:145–148` seam plus a golden
  fixture per era before the bump may ship; this design ships no schema change.

New fixtures directory `tools/orchestration/tests/pipeline/fixtures/checkpoints/`
holds a run-1-shape file, a run-2-era file (no `stageCommits`), and the live
run-003 checkpoint (with `seats`, `lastReviewPass`, and reply-less failure
records, `docs/designs/003-parallel-inference-lane/.pipeline.json`) as
load-and-resume inputs.

### 3. An escalation that survives its own failing write — Q3

- `saveState` wraps every write-path failure (the `mkdir`/`writeFile`/`rename`
  trio at `state-store.ts:185–189`) in an exported `CheckpointWriteFailure`
  class; a refused invalid snapshot keeps today's error and chain semantics
  (a refused write rejects its caller only — `state-store.ts:215–222`).
- The `drive` loop's catch (`runner.ts:2337–2342`) gains a branch: a
  `CheckpointWriteFailure` calls `finalizeEscalate` with the save error as
  reason, then stops driving — a structurally unwritable checkpoint escalates
  with a record instead of crashing without one (run-2 blocker). A seat's
  checkpoint rejection inside a pass already ends the run via
  `Promise.allSettled` rethrow (`runner.ts:1868–1876`); the same branch now
  converts that crash into an escalation. `runPipeline`'s entry save
  (`runner.ts:1938`) gets the same guard.
- `finalizeEscalate` (`runner.ts:2565–2606`) wraps its own `checkpoint()` in
  `try/catch`: on failure it still writes `ESCALATION.md` (the reason is
  already in the in-memory gate log the report renders from — A4), appends a
  line `checkpoint save failed: <msg>` to the report, still runs
  `commitStage` (committing at minimum the report), and prints the repair
  commands — `git checkout HEAD -- <designDir>/.pipeline.json`, with the
  `rm -r <designDir>/.pipeline.json && git checkout HEAD -- …` variant when a
  directory occupies the path (A5).
- **The split record is accepted, bounded, and visible** (SQLite's
  journal-existence posture, answering research open questions 4 and 5): when
  the save fails, the checkpoint keeps its last good save and `ESCALATION.md`
  is the terminal marker. `pipeline status` (`cli.ts:186–195`) checks for the
  marker alongside the loaded state and prints a split-record line when the
  checkpoint says `running` but the marker exists. `resumePipeline`
  (`runner.ts:1983–1989`) writes a `gateLog` entry
  (`terminalSaveFailed`, reason quoting the marker's failure line) before
  deleting the marker, so the new run's gate log explains the gap it repaired.
- For a directory at the checkpoint path, `loadState` names that repair
  instead of a raw errno: catch `EISDIR`/`ENOTDIR`/`ENOTEMPTY` alongside
  today's `ENOENT` handling (`state-store.ts:117–125`).

### 4. Atomic lock acquisition (supports the single-flight premise of §1)

`acquireLock` (`runner.ts:383–404`) replaces check-then-write with: try
`writeFileSync(path, info, { flag: 'wx' })` (A6); on `EEXIST`, read the holder
and apply today's pid-liveness logic unchanged — a live holder throws with
today's message (the Terraform fatal posture is already the behavior), a dead
holder is cleared with today's warning and one `'wx'` retry, and a racing
loser re-reads and throws. Two winners from the old `existsSync`+`writeFileSync`
race become impossible on local filesystems; stale-clear keeps a narrow race,
and the lock-ID nonce that would close it is Deferred.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The fence reverts a future stage that legitimately needs to write the checkpoint path | Low — no prompt references these names today (A2) | Medium — that stage's work is silently reverted | The revert appears in the ownership report and its commit body (runner.ts:277–291); fix the prompt, never re-open the fence |
| Pre-fence committed checkpoints (runs 001–007) can still carry model-authored bytes; trust then rests on human merge review, which no code enforces | Real for checkpoints that straddle the merge (research Risks) | High if exploited — seat findings re-enter prompts (runner.ts:2493–2502) | Interim posture unchanged and documented (ADR-006:55–57); the fence bounds the exposure to pre-merge history; golden fixtures pin parseability, not truth |
| Accepted split record confuses operators (marker says escalated, checkpoint says running) | Medium | Low | §3: `status` prints the split-record line; `resume` gate-logs it; ESCALATION.md names the failed save |
| `'wx'` exclusivity fails on a network/shared filesystem (A6) | Low for this repo's local worktrees (research Platform notes) | Medium — two writers return | Documented in the compatibility matrix; lock-ID nonce deferred |
| Escalation-with-failed-save leaves the tracked checkpoint deleted (directory-replacement case) until repair | Low — sweep usually catches it mid-stage | Low | §3 prints the exact `rm -r` + `git checkout` repair; Test strategy #7/#13 |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux | yes | Primary run and test environment, exercised by the `tools/orchestration` test suite; POSIX rename and `'wx'` semantics are the ones cited (A5, A6) |
| macOS | partial | POSIX semantics hold per the docs (A5, A6), but no test run in this change exercises it — see Test strategy (Platform) |
| Windows | no (unverified) | Not supported by this change and untested: no CI workflow exists (A12). Node documents platform-specific open flags (`'wx'`) and defers `rename` to POSIX rename(2); the reserved names inherit the moves validator's POSIX-shape rules (moves.ts:107–131) and the repair commands §3 prints are POSIX-shell only |
| Network / shared filesystem (NFS/SMB) | partial | Node documents `'wx'` (O_EXCL) exclusivity as not guaranteed on network file systems (A6); matrix row is a warning, not a support promise |
| Node `>= 22.23.4` (engine floor) | yes | No primitive used is above the floor; the version-check seam is unchanged in kind (package.json:30–31) |

## Test strategy

Unit (extend the existing suites in `tools/orchestration/tests/pipeline/`):

1. `gates.test.ts` — `stagePathAllowed` returns `false` for `.pipeline.json`,
   `.pipeline.json.a1b2c3d4.tmp`, and `ESCALATION.md` at any depth and in
   another case (`.PIPELINE.JSON`), for every stage class (document, coder,
   test); neighboring `docs/**` paths — including `.pipeline.json-notes.md` —
   stay allowed.
2. `ownership.test.ts` — a planted checkpoint edit, a planted writer-shaped
   tmp file, and a directory replacement of the checkpoint path (the
   `renameSync` fixture pattern already in that file) are all reverted; and a
   boundary whose only checkpoint change is the runner's own save commits that
   save rather than restoring it to `headBefore`.
3. New `checkpoint-compat.test.ts` — every fixture in
   `fixtures/checkpoints/` loads via `loadState` unchanged (run-1 shape,
   run-2 shape, run-3 live checkpoint with seats and reply-less failures);
   this is the load-side pin of the §2 contract.
4. `state-store.test.ts` — version `0`, `2`, and `99` all refuse; the message
   names the supported version and the operator postures; the file on disk is
   byte-identical after refusal.
5. `state-store.test.ts` — checkpoint path occupied by a directory: the
   `loadState` error message names the `rm -r` + `git checkout HEAD --`
   repair, not a bare errno.
6. `checkpoint-writer.test.ts` — a write-path failure (directory at target)
   surfaces as `CheckpointWriteFailure`; a refused invalid snapshot still
   rejects only its caller and the queue chain keeps landing later writes.
7. `escalation.test.ts` — `finalizeEscalate` with an unwritable checkpoint:
   `ESCALATION.md` is written, contains the gate reason and the `checkpoint
   save failed` line, the commit is still recorded, and no error escapes.
8. `runner.test.ts` — status rendering with checkpoint `status: 'running'`
   plus an `ESCALATION.md` marker shows the split-record line.
9. `runner.test.ts` — `resumePipeline` with marker + `running` checkpoint
   appends a `terminalSaveFailed` gate-log entry before deleting the marker.
10. `runner.test.ts` — `acquireLock`: a live holder blocks with today's
    message; a dead-pid holder is cleared and re-acquired; two concurrent
    `acquireLock` calls in a fixture repo admit exactly one winner.

Integration (mock client, temp git repos, as in
`parallel-integration.test.ts` / `stage-lifecycle.test.ts`):

11. A fixture checkpoint from an older era resumes with the current runner:
    driving continues after `lastStage`, `baseRef` backfills, and no refusal
    occurs — the self-update resume guarantee end to end.
12. A run interrupted between a stage commit and the next save resumes from
    the committed checkpoint, and the checkpoint is never written outside the
    runner's own write path (fence untouched).
13. Checkpoint path replaced by a directory mid-run: the run ends escalated
    (exit 2), not crashed, with `ESCALATION.md` naming the repair; after the
    repair, `resume` proceeds from the committed checkpoint.

Platform: the `tools/orchestration` test suite, run on Linux (this repo has no
CI workflow — A12), covers all of the above; the macOS, Windows, and network-FS
rows are untested in this run and recorded as such in the matrix (see Deferred).

## Rollout

1. Merge as one change in `tools/orchestration`: fence + contract text +
   fixtures + escalation guard + lock change. No schema field changes;
   `version` stays `1`; there is no migration and no data rewrite.
2. Runs in flight at merge time resume untouched (no shape change); the fence
   takes effect at their next stage boundary.
3. `docs/workflows.md` gains the operator section: the recovery posture for a
   replaced or unreadable checkpoint (`rm -r` variant), the split-record
   reading, and the cross-version refusal message.
4. Acceptance is the ADR's own scenario: the first self-update run (run 006
   itself is that run) resumes on the merged branch with new runner code
   against its pre-merge checkpoint; per the ADR, it flips `proposed` →
   accepted when that run completes.

## Deferred / out of scope

- Lock-ID nonce / stale-lock race closure (Terraform `force-unlock` posture) —
  no follow-up yet.
- Automatic digest reconciliation of worktree vs committed checkpoint at
  resume (the run-2 postmortem chain, never completed) — a dirty worktree copy
  is the *normal* state after the pass commit (runner.ts:2298), so "dirty ⇒
  tampered" fires on every resume; the §1 fence makes "dirty ⇒ runner" true.
- Schema `version: 2` and its migration ladder — owned by the future change
  that first needs a breaking field change; §2 fixes the seam it mounts at.
- Multi-run concurrency and per-run worktrees/locks — ADR-003 Alternatives.
- Windows support verification (a dated smoke job for `'wx'`/`rename`
  behavior) — no follow-up yet; the matrix row says `no (unverified)`.
- Fan-out, review rounds, gates, artifact contract — ADR-006 explicitly
  excludes them ("Nothing here changes fan-out…").

## Revision log

- r1 §1 Fence: the sweep is author-agnostic, so the rule now skips what the
  runner itself wrote in-window; A7, first commits and marker deletes hold.
- r1 §1 Fence: matcher narrowed to the `saveState` tmp shape and bound to
  segment-wise case-insensitive comparison (moves.ts:134–147); Test #1 pins it.
- r1 Compatibility matrix: Windows is `no (unverified)`, not `unknown`, so the
  POSIX-only repair commands stand; macOS is demoted to `partial`.
- r1 Test strategy (Platform): the Linux claim names the `tools/orchestration`
  suite, not a CI workflow this repo does not have (A12).
- r1 §2 May refuse: the message names the ADR's own postures (restore the
  committed copy, or start fresh) rather than a paraphrase of ADR-006:55–57.

## As-built (2026-11-10)

Record of what shipped against this approved design; the design text above is
unchanged. The change set (run 006, `tools/orchestration`) implements the four
pieces as designed — §1 fence, §2 contract text + era fixtures, §3 escalation
guards + split record, §4 `'wx'` lock — with `version` still `1` and no schema
change. Every deviation is enumerated with cause in `03-build-notes.md`
(deviations 1–19); the material drift, by section:

- **§1 Fence** — built, with four differences of substance: (a) the reserved
  set also contains `LOCK_FILE`, and single-flight is re-asserted at every
  stage boundary (`ensureLockHeld`, refuses on a live rival); cause: security
  r1 finding 4 — leaving the lock's *write* side open broke the §1 trust
  premise §4 supports. (b) The authorship test is the exported
  `recordRunnerWrite`/`recordRunnerDelete`/`holdsRunnerBytes`/`runnerBytesFor`
  API, and `loadState` seeds the record from the bytes it reads; cause:
  correctness r1 finding 1 — without seeding, "dirty ⇒ runner" died at the
  process boundary. (c) A tampered reserved path with nothing committed at it
  is restored from the runner's recorded bytes, not deleted (correctness r1
  finding 2), and a recorded delete outranks a recorded write (a resume
  deletes the marker a previous `finalizeEscalate` wrote; the sweep must not
  resurrect it). (d) `stageAll` is the single staging site and never stages
  the lock; the fence cannot depend on the operator's `.gitignore`. Known
  open gap (compatibility r1c2, minor, unfixed at ship): the
  `ESCALATION.md.<8hex>.tmp` shape introduced by the atomic marker write is
  not yet in the reserved-name set.
- **§2 Contract** — as written (header text, `docs/workflows.md` section, the
  three era fixtures, the refusal message). One addition not in the design:
  `loadState` size-caps the raw read (`MAX_CHECKPOINT_BYTES`) and refuses an
  over-cap checkpoint with the same postures before parsing (security r1
  finding 5).
- **§3 Escalation** — as written, plus: `finalizeEscalate` does not merely
  print the repair — it also restores the committed checkpoint in-process
  (`restoreCommittedCheckpoint`), because the escalation commit's `git add -A`
  would otherwise carry the planted stand-in into the terminal record
  (deviation 1); the marker is written atomically (tmp+rename) after clearing
  a planted directory, with the full report to stderr if it cannot land
  (security r1 findings 1–2); `runPipeline`'s entry save and `finalizeDone`'s
  terminal save carry the same guard (deviation 5/11); `saveState` unlinks
  its tmp on a failed write. Known open, non-blocking at ship: the
  `finalizeDone` guard branch is untested, and the ADR status flip still
  precedes the terminal save, so a done-boundary save failure commits
  `accepted` alongside an escalated record (correctness r1c1 minor, carried).
- **§4 Lock** — as written; `acquireLock` is exported with defaulted
  slug/stage (test seam), its bytes are recorded as runner-authored, and the
  boundary re-assert (§1(a) above) re-takes a missing/dead-holder lock with
  the same `'wx'` posture. Test strategy #10's two-concurrent-acquisition
  clause is pinned in-process (refused create never clobbers the holder)
  rather than by racing child processes; the full cross-process pin stays open
  as a testing-seat minor.
- **Test strategy** — #1–#13 all landed (`gates.test.ts`, `ownership.test.ts`,
  `checkpoint-compat.test.ts`, `state-store.test.ts`, `checkpoint-writer.test.ts`,
  `escalation.test.ts`, `runner.test.ts`, `checkpoint-integration.test.ts`);
  the suite is green at ship (`357 passed | 2 skipped`, the skips being the
  pre-existing EXDEV mount probe). The code-review panel approved r1c2 with
  only non-blocking minors, listed in `03-build-notes.md` "Ship state".
