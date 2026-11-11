# Code review — compatibility seat, r1 pass c2 (ratification)

Scope: `git diff 4fc97b316349ed487b5ef49430175c04aabc1e61..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and ADR-006. This pass checks only
that the post-approval revisions did not break what the compatibility seat
approved in round 1. Inputs: the diff, the proposal, the ADR, my own r1 findings.

## Verdict

**approve** — one new minor finding from the revisions themselves; no blocker
or major, and no regression of anything approved in r1.

## Re-check of my r1 findings against the revised code

1. **tmp-shape matcher duplication (minor, was approved)** — unchanged.
   `STATE_TMP_RE` (types.ts:181) is still a second copy of the `saveState`
   naming rule (state-store.ts:326), and the fence tests still plant a
   hand-written 8-hex name. No regression; stays open as approved. It does
   feed the new finding below (a third copy of the same rule appeared).
2. **Absolute paths in store-side repair messages (minor, was approved)** —
   unchanged (state-store.ts:150/167/182/198/342); `CheckpointWriteFailure`
   still carries no structured `path`. No regression.
3. **Lock-concurrency test / claim (minor, was approved)** — addressed. The
   old `Promise.all` fixture is gone; runner.test.ts:359-421 now pins `'wx'`
   exclusivity on the primitive (a refused acquisition never clobbers the
   holder's bytes) and the docstring (runner.ts:445-454) names the residual
   stale-clear race with the lock-ID nonce deferred. The matrix's network-FS
   row remains accurate as written. Approved shape, now better tested.
4. **Reserved-name sets / moves-validator claim (minor, was approved)** —
   partially improved by the revisions: `LOCK_FILE` is now in
   `namesReservedSegment` (types.ts:201) and moves.ts rule 3 already funnels
   endpoints through `stagePathAllowed` (moves.ts:155-162), so the two sets no
   longer diverge in the dangerous direction. The comment's case-parity claim
   about `moves.ts` remains inaccurate (moves.ts:134-147 compares
   `STATE_FILE`/`LOCK_FILE` case-sensitively) and the matrix still has no
   case-insensitive-checkout row — both unchanged from what r1 approved, so
   not re-opened here.
5. **workflows.md wording (nit, was approved)** — fixed. The section (lines
   169-175) now names the unsupported version, the read version, and exactly
   the two postures the refusal message prints (state-store.ts:195-199); the
   "run the version that wrote it" paraphrase is gone and matches §2.

## Post-revision compatibility checks (checklist)

1. **Platform code vs the matrix** — the post-r1 code (lock bytes recording,
   `ensureLockHeld` boundary re-assert at runner.ts:2301/536-574, `stageAll`
   lock unstage at runner.ts:353-356, committed-checkpoint restore at
   runner.ts:2810-2826, the checkpoint size ceiling at state-store.ts:140)
   adds no OS-specific branch and no new platform claim; the matrix stands.
2. **Node floor** — nothing above `>= 22.23.4` (package.json:31): `rmSync`
   recursive, `stat`/Error-`cause`, `structuredClone` are all well inside it.
3. **Paths** — all new path work uses `join`/`resolve`; `git reset -q --
   LOCK_FILE` is a literal root-relative pathspec (no glob metachars in
   `.pipeline.lock`); case assumptions unchanged from r1.
4. **Guarded platform imports** — none added (`randomBytes`, fs sync APIs are
   cross-platform).
5. **Locale/layout** — the reserved-segment test is a pure ASCII
   case-folded comparison; non-ASCII names pass through unaffected.
6. **Native dependencies** — none introduced.
7. **Platform-named tests** — no new platform-specific tests; the only
   platform gates remain the pre-existing `runIf` cases in moves.test.ts.
   The new integration suite (checkpoint-integration.test.ts) pins the
   Linux-POSIX errno strings, consistent with the matrix's Linux-only claim,
   and its fixture gitignores `.pipeline.lock` exactly as the repo's
   `.gitignore` now does (deviation 9), so the boundary re-assert does not
   make the lock commit-visible — verified against the clean-status
   assertion at line 275.

## New finding (introduced by the revisions)

- **minor** — `tools/orchestration/src/pipeline/runner.ts:2880` (with
  `tools/orchestration/src/pipeline/types.ts:181-208` and
  `tools/orchestration/tests/pipeline/gates.test.ts:352-366`).
  The revised `finalizeEscalate` writes `ESCALATION.md` through a new
  runner tmp shape, `${markerAbs}.${randomBytes(4).toString('hex')}.tmp`,
  i.e. `ESCALATION.md.<8hex>.tmp`. That shape is **not** in the reserved-name
  set: `namesReservedSegment` matches `ESCALATION.md` exactly and
  `STATE_TMP_RE` only the `.pipeline.json.<8hex>.tmp` prefix, so neither the
  fence nor the sweep covers it. Residue from a kill between `writeFileSync`
  and `renameSync` — precisely ADR-006's kill-mid-escalation scenario, then a
  human resumes on the new branch — survives every boundary sweep and is
  committed by the next `git add -A`; a stage can likewise plant a
  `ESCALATION.md.<hex>.tmp` copycat anywhere under `docs/**` and it reaches a
  run commit. That breaks the §1 promise the fence was approved on ("the
  sweep removes it ... keeping `git add -A` clean") for the escalation
  marker's own tmp, and it is a third independent copy of the tmp-suffix
  naming rule (after `saveState` and `STATE_TMP_RE`, r1 finding 1).
  **Fix:** define the tmp shape once — a shared rule that a segment is
  reserved when it is `<reserved-name>.<8hex>.tmp` for any reserved name
  (`STATE_FILE` or `ESCALATION_FILE`), built from the same constants the
  writers use — so `STATE_TMP_RE` and the new marker tmp become instances of
  one grammar; add the `ESCALATION.md.<hex>.tmp` shape to gates.test.ts's
  `RESERVED_PATHS` and one sweep case.

## FYI (no severity, out of specialty)

`ensureLockHeld` (runner.ts:536-574) throws a plain `Error` at the drive-loop
top (runner.ts:2301); the drive catch (runner.ts:2547-2561) rethrows anything
that is not `StopDriving`/`CheckpointWriteFailure`, so a lock lost at a
boundary ends the run as an unhandled crash — no `finalizeEscalate`, no
`ESCALATION.md`, checkpoint stays `running`. That is the escalation-contract
seat's territory, not the compatibility seat's; flagged for it.
