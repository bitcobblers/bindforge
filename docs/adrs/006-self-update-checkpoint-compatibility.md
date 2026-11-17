---
doc_type: adr
status: accepted
date: 2026-11-09
---

# ADR 006: Make the pipeline checkpoint resumable across a self-update

Carved out of ADR-003, which no longer carries any of it. **Precondition met
10 Nov — ready to schedule:** ADR-003 landed in run 003 and its parallel lane
was exercised once in later runs — 004, 005, and 007 launched review-pass
seats concurrently (their checkpoints show same-millisecond seat-session IDs;
`--max-parallel` is run-time config, not checkpoint state). Nothing in fan-out
depends on the property below, and no 006 run exists yet, so this ADR stays
`proposed` until one completes.

## Context

`pipeline resume` reads `docs/designs/<nnn-slug>/.pipeline.json` and continues
mid-run. When a run implements its own pipeline, that resume can happen after
the runner code on disk has changed, against a checkpoint written by the old
code. ADR-003 originally declared that transition a hard requirement.

Designing that requirement inside ADR-003 is what made the design run away. In
the attempt stopped on 9 Nov (run `002-run2`, 8 design rounds), 27 of 33
blocking/major findings — including the run's only blocker — were about making
the checkpoint trustworthy: concurrent-write ordering, trusting an in-worktree
file that every review stage may write, recovering when the checkpoint path is
replaced by a directory, and what to commit when the checkpoint cannot be
written at all. Those sections grew from 130 lines to 1,052 of a 2,696-line
proposal. Evidence: the run-2 postmortem, `docs/designs/002-run2/postmortem/
convergence-blockers.md`. The design directories were cleared on 9 Nov to reset
for a clean run, so read it from history:
`git show 3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md`.

## Decision

This ADR owns three questions, and no design is recorded here:

1. What a checkpoint written by one version of the runner must guarantee when a
   later version resumes it, and what it may simply refuse.
2. Whether the worktree copy of the checkpoint is trustworthy at all, given that
   document and review stages may write anywhere under `docs/**`, or whether
   resume should anchor to the committed copy, or move the checkpoint outside
   the writable allow-set entirely.
3. What the runner does when the checkpoint cannot be written, given that the
   escalation record itself is written through the checkpoint.

Nothing here changes fan-out, review rounds, gates, or the artifact contract.

## Consequences

Becomes possible: a self-update run can escalate after committing new runner
code and be resumed by a human on the new branch.

Accepted costs: ADR-003 lands without that property. The interim posture is
operator-facing — a resume that straddles the merge of ADR-003 may need
`git checkout HEAD -- docs/designs/<run>/.pipeline.json` or a fresh run.
