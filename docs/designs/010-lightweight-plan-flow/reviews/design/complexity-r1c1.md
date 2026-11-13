# complexity review r1 (dissent pass c1)

verdict: approve

Scope as given: the blob diff `9ae29b5..24b804a` of `02-proposal.md`, then the
full current document, judged against the complexity checklist. Inputs:
`02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`, the Revision log,
and my own round-1 findings.

## Round-1 findings — closure check

1. **major** `fix-tests.md` omitted from the ownership-variable sites — **closed**.
   A14 now names three templates with `runner.ts:1200-1208`; the `plan` stage's
   `stageVars` bullet renders `testOwnership` into `fix-tests.md:8-10,14` and
   states the light branch of the `testsNeedUpdate` sentence; Rollout 2 and Test
   strategy #11 both list the third template. Verified in code: `stageTemplate`
   returns `fix-tests.md` for `fix` with `!state.testsGreen`, no flow branch;
   `fix-tests.md:9-10,14-15` carry the rule and the test-engineer routing. No
   new defect introduced by the closure: a light `fix` that sets
   `testsNeedUpdate` and stays red re-enters `fix` under the light guard and
   dies on the existing budget record, whose report now prints the flow line.
2. **minor** dead `maxParallelReviews` key with a pinned `?? 1` — **closed**. The
   key is gone from the spread, the applied-last rule is one sentence about
   spread order, and #2 asserts the operator's value "by spread order alone,
   with no key written after the flow partial". `DEFAULT_CONFIG.maxParallelReviews: 1`
   stays the sole default (`stages.ts:83`).
3. **minor** seat rule implemented twice — **closed in substance**. One resolver
   `seatsFor(phase, cfg)`, both call sites delegate, the `states.ts:152` residual
   is named. Confirmed: `panel.ts:31-32` and `states.ts:25-26` are byte-identical,
   and `states.ts:152` is the `editorial-review` case, which light never reaches.
   The arrival path of `cfg` into the seat-consuming functions is unspecified —
   filed as the one finding below, on the changed text.
4. **minor** 10-file bullet-count gate — **closed**. `planMaxFiles` is deleted
   from the table, the gate signature, A24, the Risks row and Test strategy #5;
   the stop set is back to the ADR's two; the ceiling sits in Rollout 6 and
   Deferred, and the proxy problem is stated in Deferred.
5. **nit** five sections in two files — **closed**. `prompts/plan.md` is now a
   pointer and the sections live once in `plan-note/references/template.md`.
   Verified the cited pattern exists: `prompts/proposal.md` is exactly three
   lines and names no structure.

## Findings

- [minor][new] Budget resolution, seats paragraph — "`funnelSpecialties`
  (`panel.ts:31-32`) and `states.ts`'s byte-identical `specialtiesFor`
  (`states.ts:25-26`) delegate to it, so the `reviewSeats` read and the
  five-seat fallback live once, in one test" — the delegation has no route for
  `cfg`, and the route it needs is the signature fan-out the Options section
  says the design avoided. `funnelSpecialties(phase)` is called from
  `dissentingSeats`, `staleSeats`, `seatsConverged`, `planPass` and
  `seatSummary` (`panel.ts:57,68,77,103-104,184`), and only `planPass` takes
  `cfg`; `specialtiesFor(phase)` is called from `runner.ts:2322,2332,2339,2345,2617`,
  none with `cfg`. To delegate to `seatsFor(phase, cfg)`, four panel functions
  plus their call sites in `states.ts`, `runner.ts` and `report.ts` gain a
  `cfg` parameter, while Options records "A threaded parameter was rejected: six
  changed signatures". Left unstated, an implementer at `seatsConverged(state,
  phase)` with no `cfg` in hand re-derives the panel from `state.seats` keys, or
  keeps the `CODE_SPECIALTIES` import — the two-authority drift this paragraph
  claims to remove, and outside the reach of Test strategy #15, which asserts
  the resolver, not the wrappers that cannot reach it. Fix: add one clause
  naming the four signatures that gain `cfg` (`dissentingSeats`, `staleSeats`,
  `seatsConverged`, `seatSummary`) and stating that the effective config is
  already in scope at each of their call sites — `planPass`, `decideNext`, the
  status lines, `escalationMarkdown` — so nothing outside the funnel learns the
  seat list; and reword Options' rejection to what it rejects, a seat-list
  parameter threaded *into* the funnel from outside. No new mechanism.

## Checklist notes

- **1 (reuse)**: `plan` reuses `STAGE_BASES`, the `proposal` reply schema,
  `prompts/proposal.md`'s pointer shape, `plan-note/references/template.md`'s
  shape, the existing funnel, the two `codereview/*` agents. `flows.ts` is the
  only new module; nothing in `tools/` covers per-flow declarables.
- **2 (trace to a Goal)**: every moving part maps to a stated Goal — `FlowDef`
  to the declarable-registry Goal, `seatsFor` to the seat Goal, `planOutgrewFlow`
  to stop one, the flow line to the status Goal. `validated` is the one piece
  the ADR does not ask for; it buys the `code: []` vacuous-convergence stop that
  the seat-in-config choice creates, so it is a cost of an adopted part, not YAGNI.
- **3 (30–50% smaller)**: the shape left after deleting `validated` and
  `seatsFor` is smaller, but each deletion re-opens a defect already paid for
  (unvalidated config panel; two seat resolvers). No smaller design that keeps
  the Goals.
- **4 (dependencies)**: none new; the plan meter is the JS count at
  `runner.ts:179-180`, so the Windows/macOS `no` row's "no Unix dependency
  added" claim is accurate.
- **5 (defer the second half)**: Rollout 1–3 is the thin first slice (registry
  plus resolution, a no-op while `light` is unselectable), enforcement second,
  the files-spanned ceiling and every unverified constant (A23–A25) in Rollout 6.
- **6 (knobs/abstraction)**: `planMaxLines` has a consumer, the gate; the retired
  `planMaxFiles` is gone. `stages` is consumed by `assertStageInFlow`,
  `docStages` by the meter, `config` by `configFor`. The `funnelSpecialties`/
  `specialtiesFor` wrappers are thin delegation over one resolver, which is
  cheaper than changing five `runner.ts` call sites — the finding above is about
  how `cfg` reaches them, not about their existence.
- **7 (value)**: proportionate. Two flows over one state machine, one new
  module, two optional config fields, one gate, one rendered variable, no second
  runner, no second grammar, no per-stage flow record, no promotion machinery.
  The complexity is spent on the parts ADR-010 names as un-shed-able.

## FYI (no severity)

- Deferred: "if it lands it counts path-shaped tokens through the parser the
  ownership predicate already uses, not bullets" — `stagePathAllowed`/
  `ownershipIssue` validate a path string handed to them; no prose→path-token
  parser exists, so that sentence understates the future cost. Consistent with
  the ADR's two stops, so not a finding.
- The "Residual: a light call site handed a literal that is not `configFor`
  output…" sentence appears verbatim in Budget resolution and in the Risks row;
  two copies of one residual are two sentences to keep in step.
- In a light run `testsNeedUpdate` is never cleared (only `test-update` clears
  it), so a red `fix` re-enters `fix` to the budget limit. Owned by the
  correctness seat; the flow-naming escalation line covers the report.
