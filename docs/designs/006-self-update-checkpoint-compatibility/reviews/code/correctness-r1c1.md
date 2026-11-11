# Code review — correctness seat, r1 dissent pass c1

Diff: `git diff 4fc97b3..HEAD -- ':(exclude)docs/designs'`, read against the
full current `02-proposal.md` and ADR-006. This pass first re-checks the
five findings I raised in round 1, then checks only what the changed code
makes newly true.

## Re-check of round-1 findings

1. **major — fence authorship skip not seeded in a fresh process**
   (`state-store.ts:208-217`). **Fixed.** `loadState` now calls
   `recordRunnerWrite(file, raw)` after the version check and schema
   `safeParse`, seeding the in-process record with exactly the bytes the
   last runner left on disk. The resume scenario I described (dirty
   checkpoint from a mid-loop interruption, single-stage resume, sweep
   before any save) now hits `holdsRunnerBytes` → true and is skipped:
   no false `ownership: reverted … .pipeline.json` report. A stage write
   after the seed still mismatches the record and is reverted, so the
   fence keeps its teeth, and "dirty ⇒ runner" (Deferred rationale, §1)
   is now true across the process boundary. The seeding is pinned by a
   case in `ownership.test.ts:320`. Recorded as build-notes deviation 7.

2. **minor — tampered untracked checkpoint deleted instead of restored**
   (`runner.ts:297-317`). **Fixed.** In the no-blob-at-`headBefore`
   branch, `namesReservedSegment(p) && runnerBytesFor(abs)` writes the
   recorded bytes back instead of `git rm -f` + `rmSync`, matching §1's
   "removes it and restores the file". New `runnerBytesFor` export
   (`state-store.ts:277-279`). Pinned by
   `ownership.test.ts:364-366` ("restores recorded bytes over a tampered
   untracked checkpoint"). The planted-directory case composes correctly:
   the directory is cleared at `runner.ts:294-296` first, then the
   recorded bytes land. Recorded as deviation 8.

3. **minor — docs/build-notes overstated the refusal message**
   (`docs/workflows.md:169-175`). **Fixed.** The operator section now
   names exactly what the shipped message carries (`state-store.ts:195-199`):
   unsupported version, "this runner reads version 1", and the two
   postures — restore the committed copy, or start a fresh run. The
   "run the version that wrote it" clause is gone. Recorded as deviation 14.

4. **minor — red-gate diagnosis no longer true of HEAD**
   (`03-build-notes.md:85-92, 96-102`). **Fixed** (run record, outside the
   source diff but the claim is checkable): the notes now mark the
   diagnosis as history and name what the later test commits closed;
   verified on disk — `approveClient` in `checkpoint-integration.test.ts`
   models turn growth (`visible = max(0, min(lists-1, prompts))`, line
   166), `ownership.test.ts:222-225` destructures only `dir`.

5. **nit — duplicated message stems / path** (`runner.ts:2863-2869`,
   `state-store.ts:165-167, 180-182`). **Fixed.** The appended marker line
   is now `printable(saveFailure.message)` with no second prefix (and the
   `terminalSaveFailed` quote at `runner.ts:2174` still matches the single
   stem); both repair sentences interpolate the one `file`. Recorded as
   deviations 13 and 15.

Checklist item 7 (previous round's major actually addressed): yes — #1 is
addressed in the code itself, not papered over, and the fix preserves the
revert path for genuine stage writes.

## New finding from the changed code

- **minor** — `tools/orchestration/src/pipeline/runner.ts:2765-2791`
  (`finalizeDone` + deviation 11's new save guard). The ADR status flip
  (`flipAdrStatus` → `writeFileSync`) runs *before* the terminal `done`
  save. With the new `CheckpointWriteFailure` branch, a done-path save
  failure now escalates and `commitStage` **commits** the flipped
  source-ADR (`status: accepted`) alongside an `escalated` checkpoint and
  marker — previously the crash left the flip dirty and uncommitted. That
  contradicts Rollout #4's "flips `proposed` → accepted when that run
  completes": an escalated run has not completed, yet the accepted ADR
  reaches history, and a later `finalizeDone` re-run takes the
  `patched === null` branch and warns "ADR status not flipped
  (unexpected format)". Fix: move the flip after the save succeeds — it
  must precede `commitStage`, not the save, so flipping between the
  successful `checkpoint()` and the completion commit keeps the ADR in
  the done commit and leaves an escalated commit without the flip.

## Checklist

1. Components named in the proposal exist and are wired: fence at
   `gates.ts:261-271` with reserved names/segment matching in
   `types.ts:161-202`; §2 contract text in the `StateSchema` header
   (`state-store.ts:37-58`) and fixtures per era; §3
   `CheckpointWriteFailure` + drive/entry/done guards + marker write +
   split-record status (`runner.ts:2952-2958`, `cli.ts:197-198`) + resume
   gate log (`runner.ts:2166-2193`); §4 `'wx'` lock
   (`runner.ts:455-506`). Pass.
2. Assumption register honored (A2-A7 unchanged in kind; A6 `'wx'` used). Pass.
3. Deviations: all new behavior beyond the proposal (lock reservation and
   boundary re-assert, size cap, marker path defense, recorded-delete
   precedence, lock-bytes recording, never-staging the lock, in-process
   committed-checkpoint restore) is declared as deviations 1-19 in
   `03-build-notes.md`; each was checked against the code and matches. Pass.
4. Scope creep: the additions are declared and bound to the proposal's
   own premises (single-flight §4, §1 trust posture); no undeclared extra
   behavior seen.
5. Build-notes claims checked against the code — deviations 7, 8, 13, 14,
   15, 16, 17, 18 all match what is on disk. (FYI: shell was not available
   to this seat, so the "package green at HEAD" claim was not re-executed;
   the red-gate items it names were verified fixed by reading the files.)
6. Error paths fail closed: refusals write nothing; write failures
   escalate with a record; lock loss refuses. Pass.
7. Round-1 major/minors: addressed as above.

## Verdict

**approve** — zero open blocker/major; one new minor (finalizeDone
flip-before-save) recorded, none of it blocks the design's guarantees.
