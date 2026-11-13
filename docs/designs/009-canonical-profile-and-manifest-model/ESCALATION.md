# ESCALATED: 009-canonical-profile-and-manifest-model

Run stopped after stage `test-authoring` in the **code** phase
(round 0, budgets: design 2, code 0; status: escalated).
Reviewer sessions spent: design 24, code 0.

## Proposal size

proposal:  316 lines (+486/-170 across 7 design stage(s))

## Open blocking findings from code-review round 0

- (no blocking findings recorded; see the gate denials below)

## Recent gate denials

- stageFailure (r1): design-review:compatibility: independence violation: read a review report: /home/mike/repos/bindforge/docs/designs/009-canonical-profile-and-manifest-model/reviews/design/compatibility-r1c0.md
- design-review (r1): dissent pass c1 pending: correctness, compatibility, security, performance, complexity
- design-review (r1): dissent pass c2 pending: correctness, security, complexity
- design-review (r1): dissent pass c3 pending: correctness, security
- design-review (r1): ratify pass c4 pending: compatibility, performance, complexity
- editorial-review (r2): 2 reviewer(s) requested changes
- stageFailure (r0): implement: stage-timeout
- escalate (r0): stage implement failed twice (stage-timeout) — nudge/reply rule: one retry, then escalate

Fix the underlying cause (usually the brief or the proposal), then run:

`pipeline resume 009-canonical-profile-and-manifest-model`
