# Complexity review r1 (panel seat c0, full-panel pass, whole document)

verdict: request-changes

Checked against `references/complexity.md` items 1–7. Inputs: `02-proposal.md`
(including its Revision log), `docs/adrs/010-lightweight-plan-flow.md`, and
`tools/`/`.opencode/` to test the reuse claims. No prior complexity report
exists for this run, so every finding here is `new`. The Revision log has no
entry that rejects any of the five items below.

Reuse found and credited (item 1): the `plan` stage reuses `kind: 'proposal'`
and its reply schema; the plan artifact reuses `02-proposal.md`, so the seat
stamp, the `docLines` meter, the faithfulness check and `reviews/code/` paths
stay untouched; `runReviewPass`/`panel.ts` run unmodified on a 2-seat list;
`STAGE_BASES` keeps the stage rows. New dependencies: none (item 4). Slicing:
the Rollout's four steps are genuinely ordered so step 1 is a no-op — item 5
is satisfied by the document's own plan.

## Findings

- [major][new] The `plan` stage (`testOwnership`) + Assumption register A14 —
  "the last rendered from the flow into `implement.md:4` and `fix.md:4-5` in
  place of the hard-coded \"Never edit test files\"" — the `fix` stage has a
  **third** template that the design never names, and it carries the same rule
  plus the escape hatch to the deleted stage. `runner.ts:1200-1208`
  (`stageTemplate`) returns `fix-tests.md` for any `fix` entered with
  `state.testsGreen === false`, with no flow branch; `prompts/fix-tests.md:8-10`
  says "a test file that is unformatted ... do not touch them: set
  testsNeedUpdate ... and the test engineer's stage fixes the files it owns"
  and `:14` says "Never edit test files; if a test itself must change, set
  testsNeedUpdate in your reply". A light `fix` entered from a red `testsGreen`
  gate (the path the light state graph keeps: `implement` green-gate → `fix`)
  is told it owns nothing in `tests/**` and is given a route to a stage that
  does not exist in light — precisely the prompt-contradicts-the-ownership-matrix
  defect A14 was raised to close, and `tests/pipeline/fix-trigger.test.ts:31,58`
  proves the template is live. The design's coverage list (A14, Rollout step 2,
  Test strategy #11: "`implement.md` and `fix.md` render the light ownership
  line") omits it, so the suite stays green over the gap. Fix: extend the same
  `{{testOwnership}}` variable to `fix-tests.md` — no new mechanism, the
  variable the design already introduces, rendered in the third template; its
  `testsNeedUpdate` sentence must render the light branch's "no test engineer
  exists in this flow". Add `fix-tests.md` to A14's site list, Rollout step 2,
  and Test strategy #11's render assertions.

- [minor][new] Budget resolution — "`maxParallelReviews: // operator knob
  applied LAST / base?.maxParallelReviews ?? DEFAULT_CONFIG.maxParallelReviews
  ?? 1`" — this key is dead logic that hard-codes a third copy of a default.
  The budget table states the knob is "shared; `--max-parallel` applied last",
  so no `FLOWS[flow].config` partial sets it; `...base` in the spread already
  lands the operator's value after the flow partial, and `...DEFAULT_CONFIG`
  already supplies `1` (`stages.ts:83`). The `?? 1` literal is a pinned default
  that outlives `DEFAULT_CONFIG`: raise the package default to 2 and `configFor`
  keeps returning 1, contradicting the comment on the line above it, "remains
  the sole default authority (A4)". Fix: drop the key from the spread and add
  one clause — "no flow partial may set `maxParallelReviews`, which is why
  spread order alone satisfies the ADR's applied-last rule". Test strategy #2's
  "the operator's `maxParallelReviews` intact" assertion covers it unchanged.

- [minor][new] Budget resolution (seats paragraph) — "`funnelSpecialties(phase,
  cfg)` and `states.ts`'s `specialtiesFor(phase, cfg)` read `cfg.reviewSeats`
  and fall back to the five specialties when absent" — the two functions are
  byte-identical today: `panel.ts:31-32` and `states.ts:25-26` are the same
  `phase === 'code' ? CODE_SPECIALTIES : DESIGN_SPECIALTIES`. Threading `cfg`
  into both creates two implementations of the seat-resolution rule and two
  copies of the five-seat fallback — the drift the design cites against a cap
  in two places ("a cap in the skill *and* the registry is two authorities that
  drift"), rebuilt in code, and the effective-config test (#15) asserts the
  numbers, not the seat list, so a divergence is not caught. Fix: name one
  resolver — `specialtiesFor(phase, cfg)` delegates to
  `funnelSpecialties(phase, cfg)` (both are already imported by `runner.ts`),
  so the `reviewSeats` read and the five-seat fallback exist once. Add one
  clause noting that the direct `DESIGN_SPECIALTIES` use at `states.ts:152` is
  the editorial path, which light never reaches, so it needs no flow read.

- [minor][new] The `plan` stage (oversized-plan gate) — "when the `## Files to
  touch` section's bullet count exceeds `cfg.planMaxFiles`" — two costs for one
  mechanism. (a) ADR-010 fixes "Two deterministic stops" (the 80-line meter,
  the budget) and accepts as a cost that "nothing in the tool stops an operator
  from picking `light`"; the file ceiling is a third stop, and A24 concedes its
  constant is undatum'd — "no repo datum measures it". (b) The measured
  quantity is the bullet count the planner types, not the files the change
  touches, so the named class — "small-in-lines, wide-in-consequences" — clears
  the gate by grouping 20 files under 8 bullets: a knob whose signal is
  self-reported prose escalates on formatting while missing the risk it cites.
  Fix: make the counted quantity the section's path-shaped tokens (the same
  path strings `moves.ts`'s ownership predicate already reads), and land the
  ceiling in the data step the design already has — Rollout 6 settles A23-A25
  from the first light runs — so the first merged slice ships the ADR's two
  stops. If it lands now, state in the gate paragraph that bullet count is a
  proxy and record the proxy's known miss.

- [nit][new] The `plan` stage (skill bullet) — "Skill
  `.opencode/skills/plan-note/SKILL.md` (new): the template and its sections"
  and "Template `tools/orchestration/prompts/plan.md` (new) renders the five
  sections" — the five sections then live in two files, the shape the Options
  section rejects for the cap. The repo already has the single-source pattern:
  `prompts/proposal.md` is three lines that point at
  `.opencode/skills/design-proposal/references/template.md`, which holds the
  structure. Fix: give `plan.md` the same 3-line pointer shape and keep the
  section list once, in `plan-note/references/template.md`; the skill owns the
  sections, the registry owns the numbers.

## Item 7 — value check

Proportionate. The design buys the ADR's cost reduction without a second
runner, second grammar, second ownership matrix, second meter, second reply
contract or new `StageKind`; every new moving part (`flows.ts`, one
`Partial<PipelineConfig>`, three optional config fields, one read seam, one
resolver, three `decideNext` branches, one flow parameter on `stagePathAllowed`)
traces to a stated Goal, and the `full` flow's behavior is pinned by
`configFor('full', base)` deep-equal to `base`. The trimmable surface is
small — findings 3–5 are consolidation and deferral, not redesign — so the
complexity here is the registry's, not the feature's, and the ADR explicitly
buys that (the next flow becomes a registry entry).

## FYI (no severity)

- `FlowDef.stages` has exactly two consumers ("documentation + `--stage`
  gate"), so `assertStageInFlow` is its only behavior; that is a real guard
  (a forced `--stage research` must not write `01-research.md` into a light
  run), so it is not a knob without a consumer — flagging it is compatibility
  / correctness territory, not mine.
- The seat list reaches the funnel through three seams (`funnelSpecialties`,
  `specialtiesFor`, and a raw `DESIGN_SPECIALTIES` at `states.ts:152`); the
  design names two. The third is editorial-only, so light is safe — worth one
  clause for the implementer, and a report-path claim for the correctness seat.
- `docs/workflows.md` and ADR-001's ownership table handling (Rollout 4) is a
  docs-consistency question for another seat.
