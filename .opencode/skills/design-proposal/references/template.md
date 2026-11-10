# Design: <title>

## Context
What is being built and why, in a few sentences. Link the source document.

## Goals / Non-goals
- Goals: bullet list.
- Non-goals: bullet list. Non-goals prevent scope creep at code review.

## Assumption register
| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | <claim> | verified / unverified | <URL or file:line> |

## Options considered
For each option: one paragraph, plus why it was rejected.

## Chosen design
Components, data flow, and interfaces. Name real packages and paths
(e.g. `tools/orchestration/src/pipeline/`). Include a diagram only if it
carries information the prose does not.

## Risks
| Risk | Likelihood | Impact | Mitigation |

## Compatibility matrix
| Platform / environment | Supported? | Notes |

## Test strategy
1. Unit: <what must be proven>
2. Integration: <what must be proven>
3. Platform: <which platforms must be exercised>

## Rollout
Steps from merged branch to shipped feature; migrations; defaults.

## Deferred / out of scope
One line per concern this design deliberately does not answer, naming the
document that owns it (or "no follow-up yet"). An entry here is not a
weakness; it is the boundary of the decision.

## Revision log
Ledger of this run's decisions, kept current — not a transcript. One entry per
finding that is still material, at most two lines, disposition first. When a
round supersedes an earlier decision, replace that entry in place.

- r<N>: <finding id> — addressed by <deleting | rewriting | deferring> <what>
- r<N>: <finding id> — rejected: <reason>
- r<N>: <finding id> — deferred to <ADR>
