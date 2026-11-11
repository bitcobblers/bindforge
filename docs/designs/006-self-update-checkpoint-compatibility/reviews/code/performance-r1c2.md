# Code review — performance seat, round 1 ratification pass c2

Scope: `git diff 4fc97b316349ed487b5ef49430175c04aabc1e61..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and ADR-006. This is a ratification pass:
I check only that the fixes landed after my r1 approval (notably `fix: stop committing
the run lock and honor runner-authored deletes in the ownership sweep` — `stageAll`,
lock-byte recording, `ensureLockHeld`, the `holdsRunnerBytes` delete-before-write
reordering — plus the earlier r1-fix additions: `loadState` seeding, the 8 MiB load
ceiling, `finalizeEscalate` tmp+rename marker, the sweep's recorded-bytes restore
branch) did not break the performance properties I approved.

## Verdict: approve

Zero blocker/major findings. The three r1 findings (two minor, one nit) are unchanged
in substance — none is regressed by the later fixes; the nit is marginally more true
(one extra call added in the same loop). Nothing in the post-approval code introduces a
new cost class beyond what was approved.

## What the later fixes change, performance-side

- **`stageAll` (runner.ts:353-357)** — every staging pass now runs `git add -A` plus a
  `git reset -q -- .pipeline.lock`: one extra git subprocess per staging pass (per stage
  boundary, plus the rare revert-commit pass). Same class and constant factor as the
  per-path git subprocesses already present; not event/keystroke path. No objection.
- **Lock-byte recording + `ensureLockHeld` (runner.ts:455-465, 527-575, 2294-2300)** —
  once per stage boundary: existsSync + readFileSync + JSON.parse of a ~100-byte file,
  occasionally a `wx` write. Bounded, negligible. The `runnerBytes`/`runnerDeleted`
  records are keyed by the four reserved names, entries replaced not accumulated — no
  new growth vector (checklist item 3: bounded).
- **`holdsRunnerBytes` reorder (state-store.ts:291-313)** — delete-first short-circuits
  to `existsSync` for runner-deleted paths instead of a failed `readFileSync`; a small
  strict improvement for the marker path, no change for the checkpoint path (still a
  full read + string compare per swept violation, my r1 finding 2, unregressed).
- **`loadState` 8 MiB ceiling (state-store.ts:132-155)** — one extra `stat` per load and
  a hard cap on load-side read/parse memory. This *bounds* the load-side cost of the
  unbounded checkpoint I flagged in r1 finding 1 (save side still uncapped, as flagged).
- **`finalizeEscalate` marker via tmp+rename (runner.ts:2859-2895)** — one extra
  rename+unlink per escalation only. Negligible.
- **Sweep restore branch (runner.ts:297-317)** — adds `namesReservedSegment(p)` again at
  305 inside the per-violation loop (see carried nit below).

## Checklist pass

1. **Event/keystroke path** — pass. All new synchronous fs sits at stage boundaries and
   escalation time, never per event/per seat.
2. **Complexity** — pass. No new O(n²); per-boundary costs are constant-factor.
3. **Unbounded growth** — carried r1 minor: `gateLog` still has no push-side bound and
   no stated bound; only consumer-side trims (`report.ts:225` last-8 denied,
   `statusLines` runner.ts:2948-2950 last-6). Unchanged by the later fixes.
4. **Startup** — pass; the load path now sizes before reading (8 MiB cap).
5. **Cache invalidation** — pass; the authorship record's mismatch→revert rule is now
   pinned by three new `ownership.test.ts` cases (edit-over-save, seed-skip,
   write-after-load still reverts).
6. **Benchmark/timing note** — the diff contains no timing note or benchmark-style test,
   so there is nothing to run. I attempted a direct measurement of the per-sweep
   read+compare against the run-003 fixture this round; the shell gate rejected the
   measurement commands, so the r1 estimate stands (fixture ~1.4 k lines, ~55 KB,
   analytical only).

## Findings

1. **minor** — *carried from r1, open, not regressed*. `runner.ts` gateLog push
   (now runner.ts:1883-1900) still unbounded and undocumented, while `stageCommits`
   keeps its explicit 200-entry window (runner.ts:411-415) named as §2 contract. The
   per-boundary `holdsRunnerBytes` full read+compare of the serialized checkpoint is
   still linear in that blob. The post-approval fixes did not add growth (no new
   high-frequency gateLog call sites; the 8 MiB load cap bounds only the load side).
   Fix unchanged: window gateLog at a named cap inside `gateLog()` or state the bound
   at 1883 and in §2's contract text.
2. **minor** — *carried from r1, open, not regressed*. `state-store.ts:304-310` still
   re-reads and fully re-compares checkpoint bytes on every swept violation. The
   delete-first reorder removed one wasted failed read for runner-deleted paths (a
   small win) but kept the full-compare for recorded-write paths; the proposed
   size-check-first refinement did not land. Acceptable to carry as approved.
3. **nit** — *carried from r1, marginally worse after the later fixes*. The per-violation
   loop recomputes work: `resolve(repoDir, p)` at 287 and 293 (and 311 on the delete
   fallback); `namesReservedSegment(p)` now runs up to three times per path — once in
   the filter (279 via `stagePathAllowed`), once at 294, and the restore branch added a
   third at 305. Trivially cheap against the git subprocesses, but the fix round added
   to the duplication rather than hoisting. Fix unchanged: hoist `abs`/`reserved` once
   per iteration and reuse at 287, 294, 305, 311.

## FYI (no severity, out of specialty)

`recordRunnerWrite` does not clear a prior `runnerDeleted` entry for the same path, so
within one process a recorded delete permanently outranks a later recorded write in
`holdsRunnerBytes` (state-store.ts:301-303). Cost-wise this short-circuit is a win; the
semantic edge (delete → re-write → sweep in one process) belongs to the correctness seat.
