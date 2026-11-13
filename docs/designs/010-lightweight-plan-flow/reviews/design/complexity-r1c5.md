# Complexity review r1 (dissent pass c5)

verdict: approve

Scope: `git diff 640d904 c1b351d` of `02-proposal.md` (Options — seats; the
`configFor` block; the seat-reader enumeration; the budget-binding paragraph;
the State-graph-delta suffix paragraph; Rollout 6; the Revision log), then the
full current document, `docs/adrs/010-lightweight-plan-flow.md`, the Revision
log, and my r1 findings. Source read to confirm the changed text's citations:
`panel.ts`, `states.ts`, `report.ts`, `runner.ts`.

## Round-1 findings — all three closed by the changed text

- **major (the enumeration missed the launcher) — closed.** "Its five seat-list
  readers take the `cfg` their callers already hold: ... `planPass`
  (`panel.ts:100`) passes the `cfg` its signature already carries to its two
  sites (`panel.ts:103,104`), so a light full pass launches the flow's two
  seats". Confirmed against source: `planPass(state, phase, cfg)`
  (`panel.ts:100`) carries the config today and its two `funnelSpecialties`
  sites are `panel.ts:103,104`, so this is threading, no new mechanism. A2's
  reader list now names `planPass`, so A2 and the enumeration agree, and
  Options' "six `funnelSpecialties` call sites" (57,68,77,103,104,184) matches
  the five functions. The light full pass launches two seats, so #12's
  `2 + seats × 3` and Rollout 6's yardstop hold, and the ledger the three
  readers read is the ledger the pass loop wrote.
- **minor (`validated` saw no `stages`) — closed.** `}, flow)` with the comment
  "the seat rule is scoped to `FLOWS[flow].stages`" puts the flow in the
  canonical block, matching #23 and the prose; `configFor('light', base)` no
  longer reads as throwing on the registry's `design: []`.
- **nit (the cited discriminator cannot skip the plan stop) — closed as asked.**
  The stage is now the discriminator and the text says plainly that
  `escalationPhase` reports a light `plan` stop as `code` after the Reporting
  change. Confirmed: `CODE_STAGES` feeds only `escalationPhase`
  (`report.ts:35-57`, the sole reader), so a phase gate would have doubled the
  promotion recipe onto the plan-size string. The residual consequence of the
  new gate is finding 2 below.

## Findings

- [minor][new] Budget resolution — "Its five seat-list readers take the `cfg`
  their callers already hold: `dissentingSeats`, `staleSeats`, `seatsConverged`,
  `seatSummary` (`panel.ts:55,65,74,181`) gain the parameter" — the enumeration
  is one reader short of the live surface and one reader long of a consumer.
  Short: the sentence two lines above routes `states.ts`'s `specialtiesFor`
  through `seatsFor(phase, cfg)`, and that function has five call sites
  (`runner.ts:2322,2332,2339,2345,2617`); four are the full-flow editorial
  sites, and the fifth sits behind `reviewFilesFor` (`runner.ts:2600`) inside
  `stageVars` (`runner.ts:2621`) — a chain that holds no `cfg` today, so it
  gains two new parameters, not "the `cfg` their callers already hold". Long:
  `seatsConverged` has **no production caller** (its only consumer is
  `tests/pipeline/panel.test.ts:127,130`), so threading `cfg` into it is
  threading into test surface (checklist item 6: a parameter with no consumer).
  This matters because "callers already hold it" is exactly the argument
  Options uses to beat the rejected seat-list design, and it is the property
  that makes the change "threading only". Fix: state the threading surface
  honestly in one clause — the four live panel readers plus `specialtiesFor`'s
  five call sites, naming that `reviewFilesFor`/`stageVars` gain the parameter
  (or scope the delegation to the two config-governed phases and leave the
  pre-seat fallback on the constant) — and drop `seatsConverged` from the list
  of threaded readers, noting it is asserted through `planPass`. Threading
  bookkeeping only; no new mechanism.
- [minor][new] State graph delta — "gated on `flowOf(state) === 'light'` and
  `state.lastStage` in `implement`/`fix`/`code-review`: one append covers every
  escalation those stages reach" — the gate is a hand-maintained triple of
  three of the light flow's four stages, and the revision deleted the clause
  that justified the exclusion ("skips the plan stop, whose string already
  names the flow and the promotion recipe"). What the triple excludes is not
  only the plan-size stop: every escalation a light `plan` reaches lands in
  `finalizeEscalate` with `lastStage: 'plan'` — the 12-minute `stageTimeoutMs`
  (`cfg.stageTimeoutMs`, `runner.ts:1343`), the stage-failure escalations
  (`runner.ts:2249,2283`), the gate escalations (`runner.ts:2569`) — so a light
  run that stalls in `plan` prints no flow, against the ADR's Status clause
  ("An escalated light run must read as 'this was too big for the light flow',
  not as an unexplained stall") and this design's own Risks row on the 12-minute
  knob ("the escalation names the flow, so an operator reads it as a light-flow
  limit", which covers `implement` only). It is also the one place the flow's
  stage list is written a second time, so a flow that gains a stage in the
  registry silently stops naming the flow — the registry-as-data goal the ADR's
  Consequences names. Fix: key the append on the **flow's declared stages**
  (`FLOWS[flow].stages`, registry data) and exclude the plan-size stop by the
  string the gate composes (that text already names the flow and `--force`),
  and say in The `plan` stage that a light `plan` timeout or stage failure
  carries the suffix. Same single append site, same count of clauses; it
  removes an enumeration rather than adding one.
- [nit][new] Flow registry / Budget resolution — "The one seat-panel resolver;
  `funnelSpecialties` and `specialtiesFor` delegate to it.
  `seatsFor(phase: ReviewPhase, cfg)`" — the resolver is typed over three
  phases while the config it reads governs two: `reviewSeats` has a `design`
  and a `code` key, and `ReviewPhase` also carries `editorial`, which
  `specialtiesFor` is called with (`runner.ts:2322-2345`). So the "one place"
  resolver needs a third branch that answers from the five-seat constant, and
  the document never says which constant it returns for a light run. Fix:
  type the resolver with the panel's own `FunnelPhase` (`'design' | 'code'`) —
  exactly the phases `reviewSeats` has keys for — and leave the editorial
  answer in `specialtiesFor`, where the design already puts it
  (`states.ts:152` keeps `DESIGN_SPECIALTIES`, editorial only). One type
  narrowed; no new code.

## Checklist

1. **New module/file reuse.** `flows.ts` is the ADR's mandated location;
   `seatsFor` delegates the existing `funnelSpecialties`/`specialtiesFor`
   (verified byte-identical at `panel.ts:31-32` and `states.ts:25-26`) instead
   of adding a third reader; `plan` reuses the `proposal` `StageKind`, the
   `proposal` reply schema, the existing `codereview/code-correctness` and
   `code-testing` agents, the `code-review.md` template, the `02-proposal.md`
   path, the `docLines` meter and `reviews/code/` paths. Nothing in `tools/` is
   re-implemented. Pass.
2. **Every moving part traces to a Goal.** `configFor` (budgets/seats resolve
   once), `seatsFor` (one read seam), `FlowDef.stages` (`--stage` gate,
   `assertStageInFlow`), `docStages` (the meter), `planMaxLines` (the
   deterministic stop), `{{testOwnership}}` (kills the `test-update` loop), one
   append site (Reporting). The one part that traces to no Goal is the
   `cfg` parameter on a reader nobody in production calls (finding 1).
3. **A 30–50% smaller design.** The reductions available are already taken: the
   files-spanned ceiling is Deferred, the skill carries no number,
   `prompts/plan.md` is a pointer, the seat rule lives in one resolver, the
   suffix in one site, `maxParallelReviews` needs no resolver code (spread
   order), no `lane` field, no second runner/grammar/matrix. What remains
   additive is a registry file, one agent, one pointer template, one skill, one
   gate and a flow parameter — that is the ADR's minimum. No smaller shape that
   still meets Goals G1–G6.
4. **New dependencies.** None; the `engines` floor and the platform surface are
   unchanged (A20, Compatibility row 1).
5. **Thin first slice.** Rollout 1 lands the registry, the resolver, the
   delegates, `flowOf` and `state.flow` as a no-op for every run, guarded by #3
   (`configFor('full', base)` deep-equal to `base`), and the light flow stays
   unselectable until Rollout 2–3. That is the right cut.
6. **Knobs with no consumer; single-implementation abstractions.** `maxRounds`,
   `maxReviews`, `maxPassesPerRound`, `stageTimeoutMs`, `reviewSeats`,
   `planMaxLines`, `docStages`, `stages` each name a consumer in the text. Two
   seams sit wider than their domain: the `seatsFor` phase type (nit) and the
   `cfg` parameter on a production-unconsumed reader (finding 1).
7. **Value.** Proportionate. Six new source artifacts and registry data buy a
   43-session, 5-seat, 12-stage path down to a 2-seat, 4-stage path with a
   ≤ 8-session converging bound; the complexity is concentrated in one resolver,
   one append site and one gate, and the security spine keeps its precedence.
   Approved.

## FYI (no severity)

- Budget resolution cites "`seatsConverged` vacuously true (A2)" as the empty
  panel's failure mode; production convergence is `planPass`'s `converged`
  branch (`panel.ts:110-111`), and an empty list lands in the full-pass branch
  (`panel.ts:103-104`) with `seats: []`. Correctness/citation seat's ground, and
  that paragraph is unchanged in this diff — validation's necessity stands
  either way.
- Rollout 6's "the 1–2 sessions each `--extend-budget` grant the operator buys
  launches" is an operator-chosen quantity (A27: `extensionSessions` is a
  count), not a cap the registry sets; the ≈ 9 ceiling should read as
  grant-count-agnostic. Performance/research seat's ground.
- Risks row 7's mitigation ("the escalation names the flow") is satisfied for
  `implement`/`fix`/`code-review` and not for `plan` — the ADR side of finding 2.
