# Code review — correctness seat, round 1, pass c0

Range reviewed: `73200b44...HEAD -- ':(exclude)docs/designs'`. Inputs: the
diff's source and test paths, `02-proposal.md`, ADR-003, and the build notes
as the deviation register the checklist requires (items 3 and 5). No previous
findings for this seat.

## Checklist

1. **Components exist and match.** `executeInference` (git-free inference,
   optional `templateText` — declared deviation 2), `finalizeReviewPass`
   (`(repoDir, state, headBefore, stageName, commit)` — declared deviation 4),
   `runReviewPass` in `drive` guarded by `next.kind === 'review'`
   (runner.ts:1490), `createSemaphore` (concurrency.ts, counter + FIFO waiter
   queue, permit handed directly to the oldest waiter — no barging),
   `createStateWriter` (state-store.ts:201–221, synchronous
   `structuredClone` at enqueue, one-at-a-time chain, drained at the barrier),
   `PipelineConfig.maxParallelReviews` optional with `DEFAULT_CONFIG` = 1
   sole default, `--max-parallel` on `run` and `resume` with local
   `resolveMaxParallel` (env format gate runs even under a valid flag; range
   check names the source; all validation before client/lock/checkpoint).
   Wired end to end. Pass entry captures `headBefore` once before any seat
   prompts (runner.ts:1632, 1500); barrier → drain → one sweep + one commit →
   verdicts → cycle bump → gate → sequential `saveState`, in the proposal's
   order. Retries hold the permit across both attempts and reuse the same
   session for `invalid-reply` only. The pass-level `escalation` flag is
   checked at acquire and between attempts, reproducing today's `StopDriving`
   skip set at N = 1; the escalation path runs the sweep best-effort before
   `finalizeEscalate` with today's exact message. Editorial twins in scope
   with `-rN` naming preserved.

2. **Assumption register honored.** A3 (disjoint report/transcript paths),
   A5 (only awaits at mutation boundaries), A6 (`spawnSync` git), A7
   (tmp+rename drained before `git add -A`), A8 (`finalizeReviewPass` passes
   `undefined` reply → fallback message `docs: <nnn-slug> design-review
   stage`, pinned by `pipeline-pass-finalize.test.ts`), A10 (no primitive
   above the floor) all check out against the code.

3. **Deviations.** The six build-notes deviations match the code (writer
   re-stamp dropped, template preloaded per pass, gate label renamed — and
   nothing branches on gate-log labels, so the label-only claim holds —
   finalize signature, best-effort escalation sweep, docs deferred to
   doc-sync). **But the diff carries two subsystems the proposal never asked
   for and the build notes never declare** — findings 1 and 2.

4. **Scope creep.** Findings 1 and 2.

5. **Build-notes claims.** Code-level claims verified. The deviation-1
   rationale is not realized in code — finding 3. The third-round "npm run
   check green at HEAD" execution claim could not be re-run in this seat's
   sandbox (shell denied); see FYI.

6. **Error paths fail closed.** Runner error in a seat → `allSettled` →
   rethrow → run ends as today; invalid checkpoint refused by the writer and
   the rejection surfaces (writer tests); escalation terminal on both the
   sweep-throws and sweep-succeeds paths; stray out-of-policy sibling file is
   reverted, never committed (escalation + finalize tests).

7. **Previous round.** No prior correctness findings.

## Findings

### 1. Major — the review-independence audit is in this diff but not in this design

`src/pipeline/independence.ts` (new, 137 lines), its wiring into every stage
attempt (`runner.ts:23, 740, 839–853`), the own-findings injection
(`runner.ts:1849–1856`), the new persisted `SeatRecord.findings` field
(`types.ts:107–114`, `state-store.ts:32–34`), the "never read transcripts"
lines in `prompts/implement.md` and `prompts/test-authoring.md`, the
`transcripts/**` read/edit denies in `.opencode/agents/code/coder.md` and
`test-engineer.md`, and `tests/pipeline-independence.test.ts` all landed in
this range (commits `d831541`, `d4cfbe8`). The proposal never asked for any
of it, and `03-build-notes.md`'s deviation list does not mention it. The
`findings` field also directly contradicts the proposal's promise that "no
`.pipeline.json` schema field is added by this design" — the field itself is
additive, optional, and `version` stays 1 (ADR-003 posture satisfied), but
the promise is on the record and the gap is not.

Fix: either carve this subsystem out of the branch under review, or add it to
`03-build-notes.md`'s deviation list explicitly (with provenance as a
mid-run harness change) so the panel can adjudicate it — including the new
optional checkpoint field — instead of inheriting it silently inside an
ADR-003 review.

### 2. Major — commit-time formatting and the trigger-sensitive `fix` template are also undeclared

Commit `4c59b79` adds `formatTouchedFiles` (`runner.ts:268–307`), called from
`commitStage` (`runner.ts:317`) so every commit — including every pass commit
— now reformats dirty prettier-known files, and `stageTemplate`
(`runner.ts:656–673`) with the new `prompts/fix-tests.md`, which changes
which prompt a `fix` stage receives based on `state.testsGreen`. Plus the
matching `coder.md`/`test-engineer.md` permission edits and the +7
documentation bullet in `docs/workflows.md`. This is runner behavior change
well beyond scheduling the fan-out; the proposal's finalize contract is
`enforceOwnership + commitStage`, and its non-goals list exists to keep
exactly this kind of addition from riding along undeclared.

Fix: same as finding 1 — declare both as declared-deviations with rationale
in `03-build-notes.md` (they read as legitimate self-repair for the format
deadlock), or split them from this change.

### 3. Minor — deviation 1's stated rationale is not implemented

The build notes say the landed snapshot "must carry the moment it was
*taken*"; but nothing assigns `state.updatedAt` except `freshState`
(`runner.ts:414–437`) — the writer's `enqueue` does not stamp it and neither
does `resumePipeline`. In production every checkpoint therefore carries the
run-creation timestamp: the queue-masquerade is gone, but so is the "last
save" signal the removed re-stamp used to carry. The checkpoint-writer tests
pass only because their fixtures hand-stamp snapshots 1000/2000/3000.

Fix: stamp `updatedAt` (on the clone) at `createStateWriter.enqueue` and
before the plain `saveState` calls — or correct the deviation text to say the
field becomes informational and frozen at run start.

## FYI

- This seat's sandbox denied shell, so the build notes' "npm run check green
  at HEAD (224 tests / 26 files)" was verified statically only, not re-run.
- The ADR-003 core (semaphore, writer, fan-out, escalation, pass finalize) is
  faithful to the proposal and pinned by tests matching Test strategy #1–#9;
  findings 1–3 are about what rides alongside it, not about the lane itself.

## Verdict

request-changes (2 major, 1 minor; no blockers).
