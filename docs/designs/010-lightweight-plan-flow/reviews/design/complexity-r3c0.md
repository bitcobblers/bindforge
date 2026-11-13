# Complexity review r3c0 (ratification)

verdict: approve

Scope: the diff `c1b351d8..92f71e02` of `02-proposal.md`, then the full current
document. Checklist: `references/complexity.md`. Inputs: the proposal, the ADR,
the in-document Revision log, my r3 findings.

## What the diff changes, in complexity terms

- **A12 + The `plan` stage** — the planner's permission list is now stated as
  `design/designer`'s list *verbatim*, and A12 carries the basis for it (the
  document-stage agents already deny `transcripts/**` read/edit and
  `shell: "*"`). The r3 text's bespoke "one line narrower" list is gone, and the
  width claim is deleted. Net: one fewer bespoke artifact, no new mechanism, and
  the copy is pinned to a checkable line range (`designer.md:4-22`). This is the
  cheapest available shape — the repo has no include mechanism for agent
  frontmatter, so a shared permission fragment would be a new mechanism, which
  the checklist prefers against.
- **Budget resolution / Which knob binds / Rollout 6 / Revision log** — the
  arithmetic, the knob table, the spread-order rule, the phase-cumulative cap and
  the seat-reader enumeration are untouched by substance; the edits are
  language-level ("starts with 4 sessions spent", "a `--stage code-review` force
  run after a round that died …", "the full flow" replacing "the heavier flow").
  Nothing that was approved moved.
- **Validation posture** — "A violation throws as a usage error at the config-read
  seam, as a bad `flow` value does" binds one error class to an existing posture
  (A4's `runner.ts:2068` seam) instead of naming a new one. Same code, tighter
  prose.
- **State graph delta** — one inserted word ("and **on** `state.lastStage`"); the
  gate's stage set is the same three stages, so the r3 observation about it is
  unchanged, not newly false.
- **Revision log** — the r1 entries are consolidated (complexity+security merged,
  the intake-site entry folded in) and the r2 planner-permissions entry added.
  Each consolidation keeps its closure text: `validated(merged, flow)` scoped to
  the phases the flow's `stages` runs with #23, the phase-cumulative cap,
  Rollout 6's measurement of #12's bound and the ≈ 9 escalated ceiling, #14's
  three shapes, A28 as the intake named and asserted in #18, case staying
  sensitive. No recorded decision was dropped, and no rejected finding was
  silently re-adopted.

## Checklist

1. **Reuse** — no new package, module or interface beyond `flows.ts`, which the
   ADR names. The new artifacts (`STAGE_BASES.plan`, `prompts/plan.md`,
   `planner.md`, `plan-note/`) each reuse an existing shape (proposal stage,
   proposal template, designer permission list, review-skill template layout),
   and the diff strengthened that (verbatim permission list). No reinvention.
2. **Trace to a Goal** — every moving part traces; the planner permission list
   traces to Goal 1 (one runner, shared transcripts/lock) and the ownership
   section; A12's added claim exists only to justify not designing a new list.
3. **A 30–50 % smaller design** — nothing in the diff grew the surface; the
   smaller-slice sketch from earlier rounds (registry + resolution seam first,
   Rollout 1) is what the document already ships. No sketch worth adding.
4. **New dependencies** — none. No package, no platform surface (A20 unchanged).
5. **Deferability of the second half** — Rollout 1 (registry + seam, a no-op with
   `FLOWS.light` unselectable) remains a thin first merge; the diff leaves the
   slicing intact.
6. **Knobs with no consumer / abstraction with one implementation** — the knob
   table is unchanged; `planMaxLines` has the gate, `docStages` has the meter,
   `reviewSeats` has the resolver, and `maxParallelReviews` is deliberately not a
   flow knob. `seatsFor` has two callers today (`funnelSpecialties`,
   `specialtiesFor`), which is the consolidation the r1 closure asked for, not a
   new layer.
7. **Value** — proportionate. The registry buys a per-flow stage list, seat
   panel, budget set and meter scope from data, over one state machine; the diff
   removes a bespoke permission list and ties one error class to an existing
   posture. Complexity is flat-to-down against what the feature brings.

## Findings

None. No revision in this diff breaks a property approved at r3, and none makes
a section cited at r3 false.

## Carried from r3 (non-blocking, unchanged by this diff, not re-fired here)

- **Budget resolution** — "Its five seat-list readers take the `cfg` their
  callers already hold" still enumerates four `panel.ts` readers plus `planPass`;
  the delegation of `states.ts`'s `specialtiesFor` carries a threading surface
  that paragraph does not name (its call sites, one behind a chain that holds no
  `cfg` today), and `seatsConverged` remains a reader with no production caller.
  Threading bookkeeping only; no new mechanism, so it does not gate ratification.
- **State graph delta** — the suffix gate is still the hand-written triple
  `implement`/`fix`/`code-review`, i.e. three of the light flow's four stages
  written a second time beside `FLOWS[flow].stages`; a light `plan` that stalls
  (stage timeout, stage failure, gate escalation) reaches `finalizeEscalate` with
  `lastStage: 'plan'` and prints no flow, which is the case the Risks row
  "the escalation names the flow, so an operator reads it as a light-flow limit"
  promises and ADR-010 §Status ("An escalated light run must read as 'this was
  too big for the light flow', not as an unexplained stall") asks for. The r1
  closure in the Revision log records gating on the escalating stage to avoid
  `escalationPhase`; it does not record deciding that plan-side stalls go
  unnamed. Keying the same single append on the flow's declared stages is one
  enumeration removed, not added.
- **Flow registry** — `seatsFor(phase: ReviewPhase, cfg)` is still typed over
  three phases for a panel the config declares for two; the editorial answer
  stays in `specialtiesFor` (`states.ts:152`). Narrowing to the panel's own
  phase union is one type, no code.

## FYI (no severity)

- `planner.md` is a verbatim copy of `designer.md:4-22`, so the two permission
  lists can drift. Nothing in Test strategy #1–#23 asserts agent frontmatter, so
  the equality is prose-only. Copying is the right call here (no include
  mechanism exists); if a seat wants it pinned, an agent-config assertion is the
  shape. Reporting-seat: ownership/security, not complexity.
- "No document-stage agent has **either** privilege today" reads as two
  privileges where A12 lists three verbs (reads, writes, shells) collapsed into
  two groups. Language pass territory, not technical.
