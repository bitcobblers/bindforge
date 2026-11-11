# Code review — correctness — r1 c2 (ratification)

Seat: correctness. Pass: ratification c2 — check only that revisions landed
after the round-1 approval did not break what round 1 approved.

Inputs honored: `git diff 107c1672b6f33b0cae9117cdc751abb77f75bc68..HEAD --
':(exclude)docs/designs'`, `02-proposal.md` (full current document),
`docs/adrs/005-runner-executed-file-moves.md`. No other review reports or
transcripts opened.

## The diff under review

The scoped diff is **empty**: the only commit in the range (`b1c66ae docs:
005-runner-executed-file-moves code-review stage`) touches only
`docs/designs/` run records, which the range deliberately excludes. The only
post-approval source change in this area of history is the range's own base,
`107c167 test(orchestration): reformat the ADR-005 moves tests to clear
prettier` — a formatting-only reformat of `moves.test.ts` and
`moves-rollback.test.ts` (4 lines each, no semantic content). There are no
working-tree changes outside `docs/designs/` either (`git status` shows only
the run's `.pipeline.json`).

So: nothing in the source or test paths changed between the round-1-approved
state and HEAD. By construction, the later fixes cannot have broken what was
approved.

## Sanity confirmation that the approved state is intact at HEAD

Spot-checked (existence/wiring only, not a re-review):

- `types.ts:32,44` — `MovePairSchema` and
  `moves: z.array(MovePairSchema).max(64).default([])` present, matching the
  proposal's Reply-field contract.
- `moves.ts:285,318` — `validateMoves` and `applyMoves` exported, doc comments
  matching the validate-all-then-apply + guard-then-rollback design.
- `runner.ts:29,1072,1587` — wiring present: `executeInference` path calling
  `validateMoves`, finalize calling `applyMoves` ahead of the sweep/commit.
- `runner.ts:1416,1590,1701` — the three failure classes the proposal pins
  (`invalid-reply: moves — <issue>`, `moves-apply-failed:`,
  `moves-rollback-failed:` escalation) are all present and classified.

## Checklist

1. **Components named in the proposal exist and are wired** — intact at HEAD
   (spot-check above); unchanged since round 1. Pass.
2. **Assumption register honored** — no code change in scope, so no
   assumption could have been silently altered since round 1. Pass.
3. **Deviations declared in `03-build-notes.md`** — no new source delta to
   deviate; round-1 assessment stands. Pass (nothing new to declare).
4. **Scope creep** — the scoped diff contains no source behavior at all. Pass.
5. **Build-notes claims true** — no source change since round 1 to falsify
   them. Pass.
6. **Error paths exist and fail closed** — the three failure classes and the
   rollback escalation verified present at HEAD. Pass.
7. **Previous round's blocker/major findings addressed** — my previous verdict
   carried no findings; nothing to re-verify. Pass.

## Findings

None.

## Verdict

approve
