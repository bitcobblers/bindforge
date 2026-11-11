# complexity review r2c2

verdict: approve

Scope: the diff between the two `02-proposal.md` blobs (Read-only git paragraph,
Risk row 1, Test strategy #13, Deferred bullet 3, Revision log), then the full
current document. Ratification: checked whether the revisions broke what was
approved in r2.

## Findings

- [minor][new] Read-only git for implement/fix/test-update seats (item 5) —
  "for this class `finalizeEscalate` (runner.ts:2008-2013) records the escalation
  in the run state instead of committing `ESCALATION.md`, so the escalation path
  runs no git over the plant" — the new control is specified as a branch inside a
  shared terminal path keyed on *which failure class* arrived. `finalizeEscalate`
  has ten call sites and takes only a reason string, so implementing "for this
  class" means pattern-matching that string — a third stringly-typed classifier
  beside `retryInSameSession`'s `startsWith('invalid-reply')`
  (runner.ts:1125): rewording or renaming the `git-plant-detected:` reason
  silently re-enables the very commit that runs `git add -A` over the plant, and
  nothing in the design pins the coupling. The repo already carries the shape for
  this: `finalizeReviewPass(repoDir, state, headBefore, stageName, commit:
  boolean)` (runner.ts:1093-1103) skips exactly that stage commit on the
  escalation path. Second, the sentence conflates two writes: the
  `ESCALATION.md` file write (runner.ts:1994-1999, plain `writeFileSync`, no git)
  and the terminal commit (2008-2013, the only git call in the function). Read
  literally, "instead of committing `ESCALATION.md`" also drops the operator's
  escalation report and makes the function's own console hint
  `report: ${reportRel}` (2004) point at a file that was never written — a larger
  remediation story than the control needs, since the resume path already deletes
  that marker (1409-1413).
  Fix: name it as one boolean on the escalate call — reuse the
  `finalizeReviewPass` `commit: boolean` shape — and pin the scope in the text:
  "the `git-plant-detected` class escalates with the report file still written
  and the terminal stage commit skipped (2008-2013), the only git call on that
  path". #13's "the escalation makes no commit and the marker stays absent" then
  reads as one mechanism, not as "no report".

## FYI (no severity)

- My r2 nit is untouched by this diff: Options considered — "Allow directory
  moves" — "file-only drops that whole rule class" still overclaims (validator
  rule 6's "no listed path a path-prefix of another" and rule 4's tracked-at-
  `headBefore` check both survive in file form). Out of this pass's scope, and
  the Revision log records no rejection of it; still nit, still open.
- Scope, for another seat: docs/adrs/005-runner-executed-file-moves.md's
  Decision section says of read-only git "It is deferred to a follow-up ADR.
  Items 1-4 stand without it," while the proposal's Context (unchanged text) says
  "This proposal also lands item 5". One of the two documents is stale; the
  new Deferred bullet ("the review seats' child-session `.git` route the
  `subagent` deny does not reach ... no follow-up yet") widens the gap against
  that clause. Not complexity, and not changed by this diff.
- The Risks row 1 phrasing "no escalation commit, so no runner git touches the
  plant" holds on the paths I checked — `finalizeStage` early-returns on a failed
  outcome (runner.ts:1046-1048) and `proposalSize`'s git calls run only when
  `outcome.ok` (971) — but it is a worktree-wide claim, and the same row also
  makes plant removal the operator's step before "the worktree's next git call".
  Whether the sentence is scoped tightly enough is the security/correctness seat's
  call.

## Value check

Proportionate. This revision removed a claim and a mechanism (pinning a plant
inert through env pins before the escalation's own commit) and replaced them with
one narrow skip on the escalation path plus an operator remediation step; the
shared env keeps its own consumers (out-of-repo config per A6, deterministic
committer identity per the Compatibility matrix). No new module, dependency or
knob; item 5 remains the only part of the design with disproportionate surface,
and that is a scope question, not this one.
