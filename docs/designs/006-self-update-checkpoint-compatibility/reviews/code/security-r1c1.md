# Security code review — dissent pass c1 (r1 dissent)

Scope: `git diff 4fc97b316349ed487b5ef49430175c04aabc1e61..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md` and `docs/adrs/006-self-update-checkpoint-compatibility.md`.
Checklist: `references/security.md` (code).

## Dissent check of my r1 findings

1. **major — marker path EISDIR after planted `ESCALATION.md/` dir — FIXED.**
   `finalizeEscalate` now clears a directory at the marker path before writing
   (`runner.ts:2877–2879`, the same existsSync/statSync/rmSync pattern as
   `restoreCommittedCheckpoint`) and wraps the write in a try/catch whose
   fallback prints the full report to stderr (`runner.ts:2885–2895`), so no
   EISDIR can escape the escalation path with no terminal record. Regression
   test exists: `escalation.test.ts:280–324` plants `ESCALATION.md/probe`,
   runs the sweep-then-escalate residue class, and asserts the marker lands
   as a file, no error escapes, no tmp/planted residue, and the escalation
   commit still records.

2. **minor — bare `writeFileSync` under an "atomic tmp+rename" comment — FIXED.**
   The marker is now written through `ESCALATION.md.<8hex>.tmp` + `renameSync`
   (`runner.ts:2880–2884`) with tmp cleanup in the failure branch, and the
   comment at `runner.ts:2845–2850` matches the code. `recordRunnerWrite(markerAbs, markdown)`
   keeps the sweep from reverting the runner's own marker, and
   `holdsRunnerBytes`'s recorded-delete outranks-recorded-write rule
   (`state-store.ts:291–313`) keeps `resumePipeline`'s marker delete
   (`runner.ts:2185–2192`) from being resurrected from `headBefore`.

3. **minor — `finalizeDone`'s checkpoint save outside any §3 guard — FIXED in code**
   (`runner.ts:2783–2792`: `CheckpointWriteFailure` → `restoreCommittedCheckpoint`
   → `finalizeEscalate`, other errors rethrow). See new minor below: the branch
   has no test coverage.

4. **minor — lock absent from the reserved-segment set — FIXED.**
   `namesReservedSegment` admits `LOCK_FILE` (`types.ts:195–208`, the set is
   STATE_FILE / ESCALATION_FILE / LOCK_FILE / `STATE_TMP_RE`), `stagePathAllowed`
   applies it before every per-stage rule (`gates.ts:261–271`), `stageAll` never
   stages the lock (`runner.ts:353–356`), and `ensureLockHeld` re-asserts it at
   each stage boundary inside the drive loop's try (`runner.ts:521–571`, called
   at 2301): missing/dead-pid lock is re-taken with `'wx'`, a live foreign holder
   refuses to continue rather than run unlocked beside a rival — the design's own
   fatal posture (§4) rather than the blind recreate my r1 fix text suggested;
   that is the safer reading and is documented at the call site. Tests:
   `gates.test.ts:336–364`, `moves.test.ts:362–366`, and the fixture notes the
   gitignored lock in `checkpoint-integration.test.ts:83–85`. The gitignored-lock
   blind spot of the sweep (`dirtyFiles` has no `--ignored`, `runner.ts:225–227`)
   is exactly what the boundary re-assert closes; the residual stale-clear race
   remains the design's Deferred lock-ID nonce, unchanged.

5. **minor — unbounded checkpoint read before parse — FIXED.**
   `loadState` stats the file and refuses over `MAX_CHECKPOINT_BYTES` (8 MiB)
   before `readFile`/`JSON.parse`, with the recovery-posture message style
   (`state-store.ts:132–172`); refusal is read-only. Test:
   `state-store.test.ts:278–302` (66 MiB gateLog, byte-identical file after
   refusal, no tmp residue).

## Checklist pass

1. Untrusted checkpoint: size-capped, schema-validated, no eval — ok.
2. Path handling: `designDirOf` derives from schema-enforced `NNN_RE`/`SLUG_RE`
   grammar (no traversal from checkpoint bytes); git invoked with argument
   arrays, no interpolated shell — ok.
3. Injection: `STATE_TMP_RE` is linear (no backtracking); escalation reasons
   sanitized through `printable()` before entering the marker/terminal — ok.
4. Secrets/PII: none in the diff — ok.
5. Dependencies: none added — ok.
6. Filesystem writes: checkpoint tmp+rename, marker tmp+rename with
   directory-clear and stderr fallback, lock `'wx'` create, stray tmps unlinked
   on failure (`state-store.ts:328–346`) — ok.
7. Hostile input rejects cleanly: EISDIR/ENOTDIR/ENOTEMPTY named with the
   `rm -r` repair; `CheckpointWriteFailure` converts to escalation at the entry
   save, the drive loop, the done boundary, and inside `finalizeEscalate` itself —
   ok except the untested done-boundary branch below.

## Findings

- **minor — `runner.ts:2783–2792`: the done-boundary save guard (r1 finding 3's
  fix) is untested.** `finalizeDone` is module-private and no integration test
  fails the save at that boundary: the mid-run directory test
  (`checkpoint-integration.test.ts:279–322`) plants the directory at the first
  prompt, so it escalates at the stage-tail save (2546), not at `finalizeDone`.
  The §3-class conversion at the completion boundary is therefore a defensive
  branch no test exercises. Fix: extend the integration fixture with a hook that
  replaces `.pipeline.json` with a directory only after the last code-phase
  stage completes (or export `finalizeDone` for a unit test in the shape of
  `escalation.test.ts:280`), and assert the run ends escalated with
  `ESCALATION.md` and a recorded commit, not a crash.

## FYI (out of specialty, no severity)

- `finalizeDone` flips the ADR to `accepted` (2765–2773) *before* the guarded
  save; if the done save now fails and the run converts to escalated, the
  escalation commit's `git add -A` commits the flipped ADR — an escalated run
  shipping an `accepted` ADR contradicts Rollout #4. Correctness/design seat's
  territory.
- `ensureLockHeld`'s live-rival throw exits mid-drive without a terminal record
  (status stays `running`, no marker) and a planted live-pid lock also blocks
  `resume` until the operator removes `.pipeline.lock`. This is the design's
  accepted Terraform fatal posture (§4), so not scored as a finding.

## Verdict

All five r1 security findings are closed in code; four carry dedicated
regression tests, the done-boundary guard is correct-by-inspection but
untested (minor, above). No open blocker or major findings.
