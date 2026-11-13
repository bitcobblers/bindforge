# Security review r1c6 (ratification)

verdict: approve

Scope: `git diff 640d904 c1b351d` of `02-proposal.md` (Options — seats; Budget resolution —
canonical block + seat-list reader enumeration; State graph delta — the light-suffix gate;
Rollout 6; Revision log), then the full current document, plus
`docs/adrs/010-lightweight-plan-flow.md` and the Revision log. Claims in the changed text were
confirmed against `tools/orchestration/src/pipeline/panel.ts`, `runner.ts`, `report.ts`,
`states.ts`, `types.ts`.

## Findings

None. The revisions did not break anything the security seat approved, and did not make a
previously cited passage false.

## What the diff touched, checked against the security spine

- **The validation rule moved into the canonical block** (`validated({…}, flow)` with
  "the seat rule is scoped to `FLOWS[flow].stages`"). The block, the prose ("must name only known
  specialties, must keep `planMaxLines` **absent or** a positive integer … must keep `reviewSeats`
  non-empty **for every phase the flow's `stages` runs**") and the Revision-log entry
  (r1 + r1c4) now say the same thing, and #23 pins all four shapes. My r1 closure condition — the
  seat panel is caller data (A4's `opts.cfg` seam) and must be validated at resolution, not in
  prose — still holds. Fail-closed posture unchanged: "A violation throws at the bad-`flow` posture."
- **The seat-list enumeration gained the launcher, and the citation is real.** `planPass`
  (`panel.ts:100`) does already carry `cfg` in its signature, and its two `funnelSpecialties` sites
  are `panel.ts:103,104` exactly as the new text says; the five readers are `panel.ts:55,65,74,100,181`
  and `funnelSpecialties`' six call sites are `57,68,77,103,104,184` — so "six … call sites (A2)"
  and "its five seat-list readers" are both accurate. This *closes a hole in the security direction*:
  the seat list that launches sessions is the one `planPass` returns, so a light full pass can no
  longer launch the five-seat panel, and the code-phase session accounting behind
  `maxReviews.code: 6` matches the declared two seats.
- **The scoped non-emptiness rule keeps a consumer behind every panel it checks.**
  `reviewSeats.design: []` resolving under `light` is the vacuity shape (`planPass` → a zero-seat
  full pass, `seatsConverged` vacuously true, a board printing no dissent). Light declares no design
  phase, `assertStageInFlow` gates `--stage` against the same `stages` list the check is scoped by,
  `states.ts:152` keeps `DESIGN_SPECIALTIES` (so the registry's empty panel cannot reach the funnel
  through the editorial path), and Test strategy #12 asserts **no** `reviews/design/` in an
  end-to-end light run. No regression; the scoping is anchored to the same list the stage gate uses.
- **The suffix gate's new discriminator is implementable and its premise is true.**
  `state.lastStage` exists (`types.ts:232`, set to `${phase}-review` at `runner.ts:2462`),
  `finalizeEscalate` is `runner.ts:2845` and computes `escalationPhase` at `2861`; the strings the
  sentence enumerates are the session-budget escalation (`2362`) and the round-entry record
  (`2387`). The claim that `escalationPhase` reports a light `plan` stop as `code` is true, and
  `CODE_STAGES` is a module-local `Set` in `report.ts:35` read only by `escalationPhase`
  (`report.ts:50`) — `gates.ts` and `states.ts` import nothing from it, so the Reporting change
  ("`plan` joins `CODE_STAGES` … **for reporting only**") cannot widen the path gate. The
  ownership gate stays a `docs/**` document-stage rule, unchanged by this diff.
- **Untouched by the diff, and unchanged in the current document:** the pre-lock `flow:` validation
  (usage error, no lock / no design directory / no session), A28's intake site behind the `docs/`
  prefix rule (git plumbing output, no `.`/`..`, no absolute), the reserved-name precedence staying
  first in both flows, case-sensitivity, and #7/#18 pinning the `docs/**` widening as *allowed*.
  No new dependency is named, so the advisory/CVE item has nothing to check; no new platform
  surface (A20).

## Carried forward from r1 (unchanged by this diff — not counted against this pass)

- [minor, open] **Risks — Ownership row** — "ADR-010's Consequences records the widened surfaces".
  ADR-010's Consequences records exactly one write-surface widening ("`stagePathAllowed` … gains a
  flow parameter, widening the coder's write surface in one flow") and its only `plan` clause points
  the other way ("the `plan` stage inherits every small-model hazard … without the design panel's
  five pairs of eyes"). Rollout 4 cites the same sentence at the narrow scope. Fix unchanged:
  "ADR-010's Consequences records the coder's widened test surface; the `plan` stage's `docs/**`
  surface is accepted by this design, above." The Revision log's r1+r1c3 entry already attributes
  the acceptance to the Risks row, so the cell is the only place left citing the ADR for it.
- [nit, open] **Ownership — the security spine** — "A light `plan` that rewrites `docs/adrs/**`
  (the decision record) or a sibling run's design directory is therefore **not** reverted" omits the
  run's own source document (`docs/prompts/**` / `docs/adrs/**`), frontmatter included, i.e. the
  `flow:` key that Selecting the flow reads once per entry and that the plan-stop escalation names
  as the promotion recipe; #7's list gains the same example (`docs/prompts/<this-brief>.md`).

## FYI (no severity)

- Narrowing the suffix gate from phase to **stage** also drops the light suffix from plan-stage
  escalations that are not the size gate (e.g. the 12-minute `stageTimeoutMs` firing at `plan`);
  `escalationMarkdown`'s flow line still names the flow, but that record no longer carries the
  promotion recipe, and the r1 sentence dropped the clause that justified skipping the plan stop
  ("whose string already names the flow and the promotion recipe"), which covers only the
  oversized-plan string. (reporting / risk seat)
- Options: "the resolver that reads it validates it" — the validator is `configFor`'s `validated`;
  `seatsFor` is the reader. Unambiguous in Budget resolution, loose only in that one sentence.
