# Complexity review r4

scope: `reviews/design/editorial-r4.diff` (the round-4 editorial language pass)
inputs read: `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`, the proposal's Revision log, the named diff
verdict: approve

Checklist walked against the diff only. The editorial pass adds no module, no interface, no
dependency, no knob, and no deferral change: `flows.ts`, `configFor`, `seatsFor`, the `light`
partial, the ownership widening, and the Deferred list are untouched in substance. Every hunk is
wording: the Context verb, three gerund rewrites in Goals, the `maxPassesPerRound` sentence, two
conjunction insertions in Ownership and Rollout 6, and the `no absolute path` clarification (which
keeps the property that the `flow` parameter parses no path, so no new path extractor is implied).
Complexity looks proportionate to what the feature buys — one registry file, one resolver, one
rendered template variable, one gate — and the diff does not change that reading. Two wording
defects in the changed text touch knob/consumer precision, neither of which blocks.

## Findings

- [minor][new] Budget resolution → "Which knob binds in the light code phase" — "`maxRounds: 2` bounds
  the mutating `fix` loop, on the runner's revise-round record" — the diff ships this sentence with an
  open editor note ("author to clarify") inside the Revision log, so the canonical budget section ends
  round 4 with the fix loop's bounding knob having no named consumer site: "the runner's revise-round
  record" names no writer, and the sentence sits in the paragraph whose subject is the *pass*-backstop
  record at `panel.ts:113-119`, so a reader cannot tell which record prints the fix-round stop
  (checklist item 6 — a knob whose consumer the design does not state). Fix: replace the trailing
  clause with the site this proposal already cites in State graph delta — "`maxRounds: 2` bounds the
  mutating `fix` loop, and the round-entry record (`runner.ts:2387`) names the flow" — then delete the
  editor note. Words only; no new mechanism, and it replaces the vague clause rather than adding to it.
- [nit][new] Budget resolution → "Which knob binds in the light code phase" — "`maxPassesPerRound: 3`
  is the knob that binds, and it prints the same record on **every** shape of the round it starts" — the
  newly added clause states the binding knob unqualified, while the sentences immediately after assign
  binding to the phase-cumulative cap (`maxReviews.code: 6`) at the entry of a *later* code round; the
  section's own analysis splits binding per-round vs. per-phase, so the absolute phrasing is the one
  reading an operator can carry into an escalation and misread. Fix: scope the clause — "`maxPassesPerRound: 3`
  is the knob that binds inside a round" — the paragraph's existing text already covers the cumulative
  cap, so nothing is added.

## FYI (no severity)

- The first editor note asserts the `states.ts:152` `DESIGN_SPECIALTIES` exposure is Risks row 1; row 1
  is about the light flow having no design panel at all, while the five-seat-panel residual that constant
  represents is the one recorded in row 4 (Budget drift, "falls back to the five-seat panel"). The
  pointer's correctness is the clarity/compatibility seats' call, and it is an editorial-note text, not
  design content.
- Context says "the six completed runs spent 43–96 stage sessions each" while ADR-010's Context reads
  "The seven runs on disk" above a six-row table — a count mismatch with the source document, in a line
  the diff did not change.
- Rollout 6's escalated-run estimate "at ≈ 9 sessions — plan 1, implement 1, the fix rounds, and at most
  the code phase's 6 cumulative" sums to 10 with both fix rounds; pre-existing prose, hedged by "≈", and
  the diff's change in that hunk is the conjunction "and against".
