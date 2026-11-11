# Code review — correctness, round 1, pass c0

Scope: `git diff 846ca090053b43ec99295a7cedafb47910043df9...HEAD -- ':(exclude)docs/designs'`
Inputs: that diff, `02-proposal.md`, `docs/adrs/006-self-update-checkpoint-compatibility.md`,
`03-build-notes.md` (per the skill's step 1). No other review's report was read.
Verdict: **request-changes** — 1 major, 3 minor, 1 nit.

## Checklist walk

1. **Components named in the proposal exist, wired, right names/signatures.** Yes, all four
   changes are present and wired:
   `types.ts` exports `ESCALATION_FILE`, `STATE_TMP_RE`, `namesReservedSegment` next to
   `STATE_FILE`/`LOCK_FILE`; the reserved rule is the first statement of
   `stagePathAllowed` (gates.ts:261-269), so it precedes every per-stage rule and is shared
   with the moves validator (moves.ts:157). `state-store.ts` exports
   `CheckpointWriteFailure` and wraps the whole `mkdir`/`writeFile`/`rename` trio
   (state-store.ts:279-297); `drive`'s catch branches on it and stops driving
   (runner.ts:2445-2452); `runPipeline`'s entry save is guarded (runner.ts:2001-2011);
   `finalizeEscalate` guards its own save, appends the failure line, still writes the
   marker, still commits (runner.ts:2728-2771); `statusLines`/`renderStatus` take the
   marker flag and cli.ts:197 passes it; `resumePipeline` gate-logs `terminalSaveFailed`
   before unlinking (runner.ts:2066-2093); `loadState` names the repair for
   `EISDIR`/`ENOTDIR`/`ENOTEMPTY` (state-store.ts:143-152); the version seam carries the
   §2 contract text and the two ADR postures (state-store.ts:37-56, 173-183); `acquireLock`
   is a `'wx'` create with the unchanged pid-liveness logic and one retry
   (runner.ts:423-469). Fixtures and Test-strategy items #1-#13 all exist
   (`gates`, `ownership`, `checkpoint-compat`, `state-store`, `checkpoint-writer`,
   `escalation`, `runner`, `checkpoint-integration`).
2. **Assumption register honored.** A2 re-verified: `grep -i` over
   `tools/orchestration/prompts/` finds no reference to `.pipeline.json` or `ESCALATION.md`,
   so the fence fences no sanctioned writer. A5/A6: the errno set and the `'wx'` create are
   the ones cited. A8: no new backfill was added at resume — `baseRef` is still the only
   one. A12: repair strings stay POSIX. One register-backed claim fails in code — see
   finding 1 (`dirty ⇒ runner`, Deferred §Deferred rationale for rejecting digest
   reconciliation).
3. **Deviations declared.** The five real deviations (in-process repair, tmp unlink,
   authorship API, `acquireLock` export/defaults, entry-save guard) are listed in
   `03-build-notes.md` §Deviations. `restoreCommittedCheckpoint` is the one that changes
   behavior the proposal only asked to *print*, and it is declared and reasoned; its
   escalation test (`escalation.test.ts`) is consistent with it (HEAD holds nothing at the
   path there, so the printed posture stands). Finding 1 is a behavior the proposal asserts
   and the code does not deliver, and it is not declared.
4. **Scope creep.** Nothing in the diff is behavior the proposal did not ask for beyond the
   declared deviations. `statusLines` is a seam extraction for test #8, not new behavior.
5. **Build-notes claims.** Mostly true and unusually precise; two are now false at HEAD —
   findings 3 and 4.
6. **Error paths exist and fail closed.** Yes for the designed ones: seat rejections
   propagate through `Promise.allSettled` → `drive`'s branch (runner.ts:1931-1939);
   `drain()` rejections land there too; a refusal never writes (asserted byte-identical in
   `state-store.test.ts`); a planted tmp/dir is removed and the file restored, in either
   order. Gap on one path — finding 2.
7. **Previous round's blocker/major.** None: this is the first pass for this seat.

## Findings

### 1. major — the fence's authorship skip is per-process, so a resumed run reports its
### own newest checkpoint copy as a tampering violation

`state-store.ts:235-264` (`runnerBytes`, `holdsRunnerBytes`), consumed at
`runner.ts:284`. Trust lives in memory and `loadState` never seeds it, so in a fresh
process the run's own dirty checkpoint is an unmatched reserved path.

The dirty copy is the normal state: the loop-tail save (runner.ts:2440) runs *after*
that stage's `commitStage`, so a run interrupted mid-loop leaves the newest checkpoint
uncommitted. On resume, if the next step is a single stage (`proposal-revise`, `fix`,
`implement`, `doc-sync`, `test-*` — exactly where an interrupted design or code cycle
resumes), `finalizeStage`'s sweep (runner.ts:1673) runs *before* any save in this process
(`executeStage` takes `headBefore`, runs the stage, then sweeps; the loop's next
`checkpoint()` is at runner.ts:2440). `holdsRunnerBytes` finds no record → false → the
sweep `git restore`s the checkpoint to `headBefore` and pushes it into
`report.reverted`, which `applyOwnershipReport` prints as
`ownership: reverted 1 file(s) changed by <stage>: docs/designs/<run>/.pipeline.json`.
Review passes dodge it only accidentally: the loop saves at runner.ts:2333 before the
pass, so the pass sweep at 1724 skips.

That contradicts the proposal's own §1 consequence — "The runner's own save is skipped, so
boundary commits keep advancing the committed checkpoint (A7) **even though the file is
dirty at every sweep**" — and the Deferred rationale for refusing digest reconciliation,
"the §1 fence makes 'dirty ⇒ runner' true". It doesn't: across a process boundary,
dirty ⇒ tampered. The result is a deterministic false tamper report on the run's own
checkpoint on the ADR's central scenario (resume after an interruption, including the
self-update resume), which is the same ambiguity the fence exists to remove; the Risks row
that promises "the revert appears in the ownership report and its commit body" as the
detection channel now cries wolf. No state is permanently lost — the newer copy lives in
memory and the next save rewrites it — so this is major, not blocker. Not declared in
`03-build-notes.md`.

Fix: seed the in-process record from what the loader read — in `loadState`, after the
schema check, `recordRunnerWrite(file, raw)` (or have `resumePipeline`/`runPipeline` save
once before `drive`). A stage write or delete afterwards still mismatches the record and
is still reverted, so the fence keeps its teeth; only the runner's own already-on-disk
bytes become trusted, which is exactly what the proposal claims.

### 2. minor — a tampered *untracked* checkpoint is deleted, not restored, and can leave
### the run with nothing to resume from

`runner.ts:294-304`. The reserved-name branch restores from `headBefore`; when the path has
no blob there — a new run's checkpoint, untracked until its first `commitStage` — the
`else` branch runs `git rm -f` and `rmSync`, so the stage's tamper destroys the run's only
checkpoint file until the next save (loop tail at 2440, or `finalizeEscalate`). A crash in
that window leaves no `.pipeline.json` at all: `pipeline resume` then reports
"no pipeline checkpoint". Proposal §1 promises the sweep "removes it and **restores the
file**". Fix: when the path is reserved and the runner holds recorded bytes for it, write
those bytes back (`writeFileSync(abs, runnerBytes.get(abs))`) instead of deleting, so the
run keeps its checkpoint whatever `headBefore` holds.

### 3. minor — the operator docs promise a refusal exit the code does not print

`docs/workflows.md` (new "A state file from another pipeline version" section) says the
refusal "says to run the version that wrote it, or to restore the checkpoint or start a
fresh run", and `03-build-notes.md:41-45` repeats it ("the two operator exits (re-run that
version, or …)"). `state-store.ts:176-181` offers only the two postures the proposal
specifies: restore the committed copy, or start a fresh run. The code follows §2; the docs
and the notes overstate it. Fix: drop the "run the version that wrote it" clause from
`docs/workflows.md` and the note, or add it to the message and §2's rule set.

### 4. minor — `03-build-notes.md`'s red-gate section no longer describes HEAD

Checklist item 5. The three items reported as living in test files the coder does not own
are fixed at HEAD: `approveClient` in both `checkpoint-integration.test.ts` and
`runner.test.ts` now models turn growth (`visible = max(0, min(lists - 1, prompts))`),
`ownership.test.ts:221` destructures only `dir` with the reason inline (the unused-`base`
ESLint error is gone), and commit `4fc97b3` re-pinned the funnel assertions the section
calls unfixable. Line 83's "No test files were touched" is also untrue of the diff under
review. Fix: mark that section as the diagnosis at the fix round and record what the
later test commit changed, so the next seat does not chase three already-closed items.

### 5. nit — doubled stem in the escalation report, and a doubled path in the repair line

`runner.ts:2741-2745` appends `checkpoint save failed: ${saveFailure.message}` while
`saveFailure.message` already begins with that stem (`state-store.ts:290`), so
`ESCALATION.md` reads `checkpoint save failed: checkpoint save failed: EISDIR …`. And
`state-store.ts:148-149` prints the same path twice (`${file}` and
`join(designDir, STATE_FILE)` are identical). Fix: append `printable(saveFailure.message)`
without the prefix, and reuse `file` in the repair text.

## FYI (no severity)

The fence matches reserved segments in *every* path, so a repo-committed test fixture
directory or file named exactly `.pipeline.json` / `ESCALATION.md` under
`tools/orchestration/tests/…` would be fenced and deleted by the sweep for the
`test-authoring`/`test-update` stages. Nothing in the tree does that today (the suite
makes those names under `mkdtemp`), but it is the shape a future fixture would reach for,
and the segment rule gives it no escape.
