# complexity review r4

verdict: request-changes

## Scope handling

This pass's named scope target is the diff file
`docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r4.diff`.
It does not exist at the given path (three direct read attempts against the
exact named path, absolute and relative, each returned "File not found").
Per the caller's instructions I did not list, glob, or grep the
`reviews/` directory to hunt for a renamed or relocated copy. The permitted
full-document inputs (`02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`,
and the proposal's Revision log) were read.

## Findings

- [blocker][new] Scope (editorial r4 diff) — "docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r4.diff" —
  the named scope target is missing, so the diff-scoped confirmation pass
  cannot be performed: checklist items 1–6 cannot be evaluated against the
  r4 editorial changes, and no "the revisions broke something approved"
  judgment can be anchored to changed text. A missing checklist item I could
  not evaluate counts as a finding, not a pass.
  Fix: regenerate (or re-deliver) the round-4 editorial diff at the named
  path and re-run this seat; alternatively the runner can explicitly rescope
  this confirmation pass to the whole `02-proposal.md`. No proposal text
  change resolves this — the defect is in the review input, not the design.

## Checklist notes (against permitted full-document inputs, not the diff)

Recorded so the value judgment is not left blank, and flagged as out of the
diff scope — these are not findings under this pass's rules:

1. Reuse: no new package; one new module (`moves.ts`) at the pipeline layer;
   wiring reuses the existing `safeParse` retry path, `stagePathAllowed`,
   `enforceOwnership`, `commitStage`, and the `.pipeline.lock` invariant.
   Nothing here reads as duplicating existing `tools/` capability.
2. Tracing to Goals: each moving part (reply field, validator, apply guard +
   rollback, sweep no-op, prompt lines) maps to a stated Goal or to an ADR-005
   Decision item; no moving part is visibly untraceable to a Goal.
3. Smaller design: file-only pairs, no new sweep branch, `fs.renameSync` over
   `git mv`, no feature flag — the shape is already near the minimal design
   consistent with the ADR's Decision.
4. Dependencies: none new (`fs.renameSync`, `path.posix`, one existing-helper
   `git cat-file -t`).
5. Deferrability: directory moves, case-only renames, read-only git are all
   already deferred; the merged slice is thin (inert at default `[]`).
6. Knobs/abstractions: no config knob, no single-implementation abstraction
   layer beyond the schema/validator split.
7. Value: proportionate to the feature (run 004's proven livelock, A12).

None of these observations is anchored in the r4 diff, which is the missing
input; that is why the pass cannot be approved.

## FYI (no severity)

- The missing-diff condition is a runner/inputs problem, not an authoring
  problem in `02-proposal.md`; the author should not be asked to edit the
  proposal on account of it.
