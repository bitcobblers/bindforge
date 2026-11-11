# performance review r1

scope: whole document (panel pass c0, round 1)
inputs used: `02-proposal.md` (incl. its Revision log), `docs/adrs/006-self-update-checkpoint-compatibility.md`.
The Revision log holds one entry ("r1: (initial authoring) — no review findings yet"), so nothing in
this report is a re-report of a rejected finding. All findings below are `new`.

verdict: approve

## Checklist walked

1. **Input-path latency** — nothing in the design touches a per-keystroke/per-event path. The runner's
   hot analog is the checkpoint save path, and §3 only wraps failures there (`CheckpointWriteFailure`
   is allocated on the failure branch, not on the save path). Pass.
2. **Structure costs / O(n²)** — no nested iteration is introduced. `stagePathAllowed`'s new rule is a
   constant-time segment test evaluated before the per-stage rules; the ownership sweep stays linear in
   dirty paths; `acquireLock` is bounded at one extra read plus one `'wx'` retry (no retry loop). Pass.
3. **Memory growth** — one gap, reported below: the design adds a new persisted `gateLog` append with no
   stated bound, while it explicitly names the bound for the neighboring `stageCommits` window.
4. **Startup cost** — `version` stays `1`, so no reader ladder, no migration, no data rewrite
   (Rollout 1). `loadState` gains only errno-branch handling. Pass.
5. **Caching / invalidation** — no cache is introduced; the `status` split-record check is a fresh read
   per invocation, so there is no stale state to invalidate. Pass (nothing to evaluate as broken).
6. **Research baseline vs expected numbers** — the proposal quotes no cost baseline and states no expected
   numbers anywhere, so there is no gap to flag, but also no evidence the cost was measured or reasoned
   about. Reported as a nit; the research document is not in this seat's inputs, so the baseline itself
   could not be quoted here.

Positive note (checklist 3/4, no action): rejecting digest-anchored trust inside the checkpoint and
rejecting auto-reconcile digest compare at resume also avoid hashing `docs/**` at load/resume time. The
rejections are performance-positive as well as trust-positive.

## Findings

- [minor][new] §3 An escalation that survives its own failing write — "`resumePipeline`
  (`runner.ts:1983–1989`) writes a `gateLog` entry (`terminalSaveFailed`, reason quoting the marker's
  failure line) before deleting the marker" — this adds a new writer to a persisted, append-only array on
  the escalate → repair → resume cycle, and the design never says whether `gateLog` is bounded. It matters
  specifically for this design: its subject is checkpoints that outlive the run that wrote them and that
  operators repair and resume repeatedly, and `gateLog` is re-parsed by `loadState` on every resume,
  re-serialized on every save, and scanned by the escalation report (A4: "escalationMarkdown renders the
  last denied gate-log entries from the in-memory state"). §2 shows the house standard for exactly this
  concern — "`stageCommits` keeps its bounded-window meaning … as named contract, not comment" — so the
  silence reads as unexamined rather than deliberate. Fix: extend that §2 contract bullet (or add one
  clause to §3's split-record bullet) naming the `gateLog` growth limit the same way — either "windowed
  like `stageCommits`" or "append-only, bounded by the run's stage-and-attempt count, and therefore
  bounded at the byte size fixtures pin" — replacing the silence with the same named-contract sentence
  instead of adding a trimming mechanism.
- [nit][new] §1 Fence runner-reserved paths / §3 split record / §4 Atomic lock acquisition — "gains a
  first rule that precedes the per-stage rules", "checks for the marker alongside the loaded state",
  "on `EEXIST`, read the holder and apply today's pid-liveness logic unchanged" — the document adds three
  new per-operation costs (one segment test per swept path, one extra stat per `pipeline status` call plus
  one extra file read per resume, one extra read and one `'wx'` retry per contended lock acquire) and
  states no cost expectation for any of them, and no baseline or number appears anywhere in the document.
  The costs are almost certainly free, but a reader cannot tell whether that was checked or assumed, which
  is what the baseline item asks for. Fix: one sentence at the end of §4 or the head of Risks — "the fence
  is a constant-time segment check added to the existing per-path sweep; the split-record check is one
  `stat` per `status` call and one read per resume; the lock path gains one read and one retry only under
  contention; nothing here is on a per-event path, so no runtime budget applies" — and, if the research
  document carries a checkpoint size/load baseline, cite it there.

## FYI (no severity)

- §2 names "the live run-003 checkpoint (`docs/designs/003-parallel-inference-lane/.pipeline.json`)" as a
  fixture copy while §1's reserved-name rule still treats that in-run copy as runner-owned. The duplicate
  drift story (does the live file stay, who updates the fixture when the run's checkpoint is amended) is a
  complexity/maintenance question, not a performance one.
- §1's "`git add -A` clean without relying only on the drain barrier" makes the sweep revert tmp files
  shaped exactly like the runner's own in-flight `saveState` tmp files. Whether that can race a live
  save is a correctness question for another seat; its cost side is negligible.
- Risks row "Escalation-with-failed-save leaves the tracked checkpoint deleted … until repair" and §3's
  `rm -r` repair hint are operator-cost items; no performance action needed.
