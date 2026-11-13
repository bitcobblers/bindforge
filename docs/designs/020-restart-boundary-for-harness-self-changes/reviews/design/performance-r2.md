# performance review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`) against the
round-2 `02-proposal.md`. Inputs used: the diff, `02-proposal.md` (including its
Revision log), `docs/adrs/020-restart-boundary-for-harness-self-changes.md`, and
read-only checks of the pipeline sources. No earlier review report or transcript
was opened; no `01-research.md` (outside this pass's named inputs), so the
baseline for checklist 6 is quoted from the ADR, which is inside them.

Previous findings carried in: none. The diff is the single editorial pass of
round 2, and it changes no mechanism, no cost claim, and no number.

## Diff read

Every hunk is language-only: `ADR 020` → `ADR-020` (Non-goals, Options, §1 tail,
Risks row 4, Rollout 4, Deferred, Revision log); `in favor of` → `in favour of`;
`behavior` → `behaviour` (Deferred); two sentence reworks — "any fresh process
started from the run's worktree **already imports and renders** the changed
tree" (§2, resume paragraph) and "the line **that** `finalizeRestartBoundary`
prints" plus the "Status output:" bullet label (§3). Nothing in the diff touches
§1's detection rule, §2's record or bounding, the Risks numbers, the exit-code
table, or the Test strategy. The §2 rework replaces an ungrammatical
"has imported and renders" with the tense A1 actually supports; it neither adds
nor weakens a cost claim.

## Checklist

1. **Input-path latency** — no keystroke/event path is involved: the change sits
   in the orchestration drive loop. The added work is one `git diff -z
   --no-renames --name-only <headBefore>..HEAD` per finalize window, not per
   poll: the loop's polling cadence (`pollIntervalMs: 2_000`, stages.ts:118) is
   untouched, and the loop-top check reads in-memory state (`state.restartRequired`)
   before `decideNext` at runner.ts:2440–2441 (anchor verified). Rejecting the
   per-commit hooks (Options) is the cheaper of the two shapes: one subprocess
   per window instead of three sites. Not a finding.
2. **Structure costs** — the predicate is a two-prefix path test (O(1) per path,
   beside `stagePathAllowed`, gates.ts:559–589) over one diff listing bounded by
   the window. No product of bindings × devices × platforms, no repeated scan.
   ADR-003's fan-out keeps "the whole pass owns ONE git window" (runner.ts:2722–
   2725), so raising `maxParallelReviews` does not multiply the check.
3. **Memory growth** — the record is a single optional field holding `paths`, and
   §2 keeps it bounded: "the record keeps the `boundedGateEvidence` bounding
   (first N paths plus the count)". The shipped precedent is exactly that shape —
   `legs.slice(0, MAX_GATE_LEGS)` plus an "(gate legs elided)" count line
   (runner.ts:3057–3075) — so the checkpoint cannot grow with a model-authored
   file list. Cleared at resume entry, so at most one record per process. Bounded.
4. **Startup cost** — the design adds no boot-time file read: the field is one
   optional `z.object` in `StateSchema` (A6/A7), and a pre-existing checkpoint
   reads as "no boundary standing". A12's process-start cost is confirmed as a
   handful of local calls: `acquireLock`, `rev-parse HEAD`, `ensureBranch`, one
   checkpoint save (runner.ts:2263–2277).
5. **Caching invalidation** — this is the design's subject, and the invalidation
   it relies on is verifiable in the tree: templates are read from disk per stage
   (`loadTemplate`, prompts.ts:81–83) and pinned at stage entry for every attempt
   (runner.ts:2453–2467), the module cache is invalidated only by the fresh
   process, and the standing record is refused at the loop top before another
   window opens. The per-attempt `template @ sha` provenance is hashed from the
   rendered text (`templateHash`, prompts.ts:85–87). No cache is left stale by
   this change; nothing here adds a cache to invalidate.
6. **Baseline quoted** — A12 carries the ADR's own figure, "one more process
   start, which is seconds — far less than the escalation it replaces"
   (ADR-020:116–117), against the shipped stage cap `stageTimeoutMs: 30 * 60_000`
   with the light flow's 12 minutes as registry data (stages.ts:119–126). The
   design's expected numbers are the ADR's numbers, and the register marks the
   magnitude `unverified` ("mechanism verifiable in code, the magnitude never
   measured"), which is the explanation the checklist asks for rather than a gap.
   The mechanism half checks out in code; no unexplained numeric gap.

## Findings

None. No blocker or major, and nothing in the changed text makes a section
false or breaks a cost claim: the diff is language-only and its two reworks
match the anchors they cite.

## FYI (no severity)

- §2 — "the record keeps the `boundedGateEvidence` bounding (first N paths plus
  the count)" — the named function bounds gate *legs* (`GateEvidence`,
  runner.ts:3057–3075), so the implementer needs the same first-N-plus-count
  shape written for a `string[]`; memory stays bounded either way, so the cost
  question is settled — whether the helper gets its own name is a
  complexity/implementation call, not a performance one.
- The pass normalized `favour`/`behaviour` (Options, Deferred) while A6
  ("behavior-safe defaults") and A13 ("staleness behavior") still read American.
  Language consistency, for the editorial seat.
