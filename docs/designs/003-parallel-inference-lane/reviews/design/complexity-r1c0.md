# complexity review r1

verdict: approve

Scope: whole document (`02-proposal.md`), full-panel pass c0, round 1. First
complexity pass; no prior findings, Revision log is empty (initial proposal).

## Checklist evaluation

1. **Reuse check (`tools/` inspected):** no in-process limiter, queue, or mutex
   exists anywhere in `tools/orchestration/src` (`.pipeline.lock` is a
   cross-process PID lock, not reusable for in-run fan-out; `independence.ts`
   is an audit helper). `p-limit` appears in `package-lock.json` only as a
   transitive dev-toolchain entry — not a direct dependency — so adopting it
   would *add* a dependency; the proposal's rejection of the library and the
   ~20-line in-process semaphore match the lane machinery's dependency-free
   style and are the right call. Nothing reusable was skipped.
2. **Every moving part traces to a Goal:** the two-phase split, `runReviewPass`
   fan-out, `createSemaphore`, `createStateWriter`, and
   `maxParallelReviews`/`--max-parallel` each map one-to-one to a Goals bullet
   and to an ADR-003 §Execution clause. No orphan machinery. The design
   actively *refuses* un-asked-for parts (no timeout↔N formula, no shared
   env-resolver extraction, no schema field, no adaptive path) — the Options
   section shows each was considered and cut.
3. **Smaller design (30–50%)?** No meaningfully smaller design meets the
   Goals: the ADR prescribes the shape (two-phase split, one pass-level
   finalize, size-1-semaphore degenerate path, drained write path). A design
   that dropped the writer (all in-pass writes deferred to after the barrier)
   would be smaller but would weaken today's per-seat crash resilience and
   contradicts ADR-003 §Self-update ("every checkpoint write made from inside
   a concurrent review pass"). The pass-level-finalize choice *removes*
   machinery (a git mutex) rather than adding it.
4. **New dependencies:** none. Verified above.
5. **Deferrable second half?** The rollout already keeps the merged slice
   thin by behavior (default N = 1, live validation post-merge). Slicing the
   change into "split first, fan-out later" would mean restructuring the same
   seat loop twice and would violate ADR-003 §Execution 4 ("N = 1 goes
   through the same fan-out … not a separate branch"). Not a finding.
6. **Knobs without consumers / single-implementation abstractions:**
   `maxParallelReviews` has exactly one consumer (the semaphore); no config
   knob is speculative. `createSemaphore` and `createStateWriter` are concrete
   utilities, not interface layers with one implementation. The seat-loop
   unification (design/code loop + editorial twin → one `runReviewPass`)
   *reduces* existing duplication.
7. **Value check:** the complexity looks proportionate. New surface is ~20
   lines of semaphore plus a small enqueue/drain chain; most of the change is
   relocating existing code (finalize out of `executeStage`, loop restructured)
   rather than new subsystems, and every added mechanism pays for a named race
   (checkpoint last-writer-wins, `git add -A` sweep) or an ADR-mandated
   invariant.

## Findings

None.

## FYI (no severity)

- The env-resolver *pattern* is now deliberately duplicated
  (`resolveMaxParallel` vs `resolveReplyTimeoutSeconds` at `src/cli.ts:101–137`);
  the deferral is consistent with the ADR, but a third knob would make the
  duplication worth revisiting — cleanliness seat's business, not mine.
- The widened escalation commit ("siblings' partial reports land in the
  escalation commit") is a behavioral/compatibility consequence worth its own
  seat's eye; from a complexity view it correctly reuses `finalizeEscalate`
  with no new machinery.
- `finalizeReviewPass(opts, state, headBefore, stageName, stageKey, cfg)`
  carries six positional params; not worth churn since it mirrors the moved
  call sites.
