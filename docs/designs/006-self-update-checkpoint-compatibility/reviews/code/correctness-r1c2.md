# Code review — correctness, r1 check 2 (ratification)

Scope as given: `git diff 640c289..HEAD -- ':(exclude)docs/designs'`.
FYI on scope: that literal range carries **no source or test paths** — between
640c289 and HEAD only `docs/designs` files (run records) landed. The code
revisions that landed after the r1c1 approval are commit **640c289**
(`stageAll` lock un-staging, lock byte recording in `acquireLock`/
`ensureLockHeld`/`releaseLock`, `holdsRunnerBytes` recorded-delete precedence,
a defensive `finalizeStage` guard; source: `runner.ts`, `state-store.ts`; no
test changes). I reviewed that delta against the full current
`02-proposal.md`, the ADR, and my r1c1 findings — the substantive change since
my c1 approval — and confirmed HEAD's code paths read identically for
everything else I approved.

## Checklist walk

1. **Components/interfaces vs proposal.** Unchanged by the delta except
   §1/§4 internals; `stageAll` keeps the single `git add -A` choke point
   (runner.ts:354) with `reset -q -- LOCK_FILE`, preserving A7 (terminal save
   lands inside the commit; `.pipeline.json` is still staged by
   `commitStage`). Pass.
2. **Assumption register.** A6/A10 lock posture untouched (`'wx'` create and
   pid liveness unchanged). A3 strengthened: the sweep's authorship skip now
   honors "absent because the runner deleted it" (state-store.ts:301-302),
   which is exactly the proposal §1 parenthetical; the previous fall-through
   resurrected the §3 marker from `headBefore`, and the fix restores
   proposal-faithful behavior. No silently changed assumptions. Pass.
3. **Deviations declared.** Input set for this pass excludes
   `03-build-notes.md`, so I could not re-read the deviation text; the delta's
   behavior is a direct implementation of review findings cited inline
   (security r1 #4 lock-fence hardening) and the same commit grows
   `03-build-notes.md` (+60 lines, visible in its stat). No *new* deviation
   kind introduced beyond the already-approved LOCK_FILE-reservation posture.
   Pass, within input limits.
4. **Scope creep.** `stageAll` and byte-recording for the lock extend the
   r1-declared lock fence; no new behavior beyond that. The
   `finalizeStage` `entry !== undefined` guard and the `let holder` type tweak
   are behavior-preserving. Pass.
5. **Build-notes claims.** Not openable in this pass's input set (FYI above);
   no claim contradicted by the visible diff.
6. **Error paths fail closed.** `gitOk` on the `reset` degrades at worst to
   today's (pre-fix) lock-committing behavior, not to a trust failure; the
   §3 guards in `finalizeDone`/`finalizeEscalate` are untouched and still
   escalate rather than crash. Pass.
7. **Previous blocker/major addressed.** My r1c1 finding was minor, not
   blocking; it is **still open** (see finding 1 — unchanged by the delta).

## Findings

1. **minor (carried from r1c1, still open, unchanged by this delta)** —
   `tools/orchestration/src/pipeline/runner.ts:2765-2798` + `2917-2922`:
   `finalizeDone` still flips the source ADR (`proposed` → `accepted`,
   2765-2774) before the guarded terminal `done` save; a
   `CheckpointWriteFailure` routes to `finalizeEscalate`, whose `commitStage`
   (2917-2922, now via `stageAll`) commits the accepted ADR alongside the
   `escalated` checkpoint and `ESCALATION.md`. Proposal Rollout #4 ties the
   flip to run completion; an escalated run has not completed. A later
   `finalizeDone` re-run takes `flipAdrStatus`'s null branch and only warns.
   Fix as stated in r1c1: perform the flip between the successful terminal
   save and the completion commit, never before the save. The 640c289 delta
   neither fixes nor worsens this.
2. **nit (introduced by this delta)** —
   `tools/orchestration/src/pipeline/state-store.ts:263-313`: a recorded
   delete outranks a recorded write forever — `recordRunnerWrite` never
   clears `runnerDeleted`. So if the runner deletes a reserved path and later
   rewrites it in the same process (real today: resume's §3 marker delete at
   runner.ts:2192, then a re-escalation's marker write at runner.ts:2884),
   `holdsRunnerBytes` returns `false` while the file holds exactly the
   runner's fresh bytes — any *later* sweep would revert the runner's own
   write. Today no sweep runs between those two actions (`finalizeEscalate`
   commits via `commitStage` without an ownership sweep, then the run ends),
   so it is latent, not active. Fix: make the records last-action-wins —
   `recordRunnerWrite` should `runnerDeleted.delete(abs)` — which also lets
   `holdsRunnerBytes` keep the delete-outranks-write read without stranding a
   stale delete flag.

## Verdict

approve — no blocker/major open; the post-c1 revisions (lock never staged,
runner-authored deletes honored, lock bytes recorded for the fence) align the
code more closely with proposal §1/§4 and break nothing I approved in r1c1.
