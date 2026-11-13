# correctness review r1c6

Ratification pass: scope is the blob diff `4cc9acc → c1b351d` of `02-proposal.md`
plus the full current document, checked against ADR-010, the Revision log, and
this seat's r1c1 findings. Inputs used: those four only.

verdict: approve

Round-1 closures verified as real, not cosmetic:

- Finding 2 (the flat non-emptiness rule): the rule is now scoped to the phases
  `FLOWS[flow].stages` runs, `configFor('light', base)` resolves the registry's
  `design: []`, and #23 asserts the throw under `full` and the resolve under
  `light`. Closed, no new breakage: the table row, #1 and the prose agree.
- Finding 3 (the string enumeration missed a writer): the one-site
  `finalizeEscalate` gate on flow + escalating **stage** covers the red-tests
  limit (lastStage `implement`/`fix`), the pass-backstop and fix-round
  `next.reason`, the round-entry record and the session-budget record, and the
  stage discriminator correctly excludes the `plan` stop after the Reporting
  change. It also satisfies Risks row 7 (a light `implement` timeout names the
  flow). Closed; #8/#14's "asserted on the record" matches "no `states.ts`
  string changes".
- Finding 5 (planner grant vs. claimed confinement): Ownership now states the
  gate as `docs/**`, denies any run-scoped branch, and pins the widening as
  allowed in #7/#18 and Risks row 3; the "agent-config work" claim is gone.
  Closed as to the predicate; the residual is the ADR bookkeeping claim in
  finding 3 below.

## Findings

- [minor][re-raised] Budget resolution, "Which knob binds in the light code
  phase" — "The session cap is checked at **pass entry**, ahead of that
  backstop (A16), so inside the first code round the backstop prints first."
  — the conclusion is now right (the old "so it never fires" is gone) and the
  new cumulative-cap sentence is sound: `reviewSessionsUsed` counts every
  `code-review:` result, so the cap binds only at a pass entry in a *later*
  code round, which the `--stage code-review` force example reaches — #14(c)
  matches it. The causal clause is still the one my r1c1 finding 2 named: read
  literally, "cap checked at pass entry, ahead of the backstop" predicts the
  session-budget string at spent 6, which #14(a) says never prints in that
  round. The reason the backstop wins inside the first round is upstream:
  `decideNext` calls `planPass` (`states.ts:181`) and escalates the blocked
  pass-backstop reason (`states.ts:183`) at the top of the drive-loop
  iteration, before the review branch reaches `runner.ts:2357`. Fix: replace
  the `so`-clause's basis with that site — "the pass backstop is decided
  upstream, in `decideNext`'s `planPass` call (`states.ts:181-183`), so the
  iteration at spent 6 escalates on it and no pass entry happens at the cap."
- [minor][re-raised] Ownership — "so the `flow` parameter parses no path; case
  is **not** normalized — as today." — the r1c1 finding-4 narrowing was not
  applied to this clause: the revisions rewrote the sentences around it (the
  A28 intake site) and re-asserted the blanket. `namesReservedSegment`
  lowercases each segment (`types.ts:195-207`), which is how the code comment
  at `gates.ts:294-300` reads ("no stage class … can own them at any depth or
  under another case"), and the same paragraph puts that rule first and
  winning, so a sentence saying case is not normalized "as today" is false for
  it. #7 already scopes the claim correctly ("case variants of
  `tests/`/`docs/` keep today's case-sensitive regex verdict"). Fix: scope the
  clause — "case is not normalized for the `tests/`/`docs/` regexes, as today;
  the reserved-name rule is case-insensitive by design".
- [minor][new] Risks, Ownership row mitigation — "ADR-010's Consequences
  records the widened surfaces" — the surfaces this row adds are new in this
  revision (`plan` rewriting `docs/adrs/**`, `docs/prompts/**` or a sibling
  run's design directory unreverted), and ADR-010's Consequences does not
  record them: it records `stagePathAllowed` gaining a flow parameter that
  widens **the coder's** write surface, the shared `NNN` space, promotion
  overwriting the plan, and the `plan` stage's small-model hazards. Nothing
  there states the plan stage's `docs/**` write surface, and Rollout 4 leans
  on the same reference for the same purpose. The one mitigation that defers
  the record to a document that does not carry it is the one a reader relies
  on. Fix: name the missing entry and the moment it lands — "ADR-010's
  Consequences gains the plan stage's `docs/**` write surface (this row) when
  the ADR is accepted; today it records only the coder's widened surface."
- [nit][new] Test strategy #7 — "`plan` allows **every** path under `docs/` —
  `docs/designs/**`, `docs/adrs/**`, `docs/workflows.md`, a sibling run's
  directory" — the same test's preceding clause denies "every reserved name —
  `.pipeline.json`, the lock, `ESCALATION.md`, a `saveState` tmp … for
  `plan`", and the checkpoint is `docs/designs/<slug>/.pipeline.json`: under
  `docs/`, so the two assertions are unsatisfiable together as literally
  written (`namesReservedSegment` runs first). Fix: "allows every
  **non-reserved** path under `docs/`".
- [nit][new] Budget resolution, seat-reader paragraph — "no call site keeps a
  `*_SPECIALTIES` import" — the next sentence says `states.ts:152` keeps
  `DESIGN_SPECIALTIES` (editorial only, Risks row 4), which is a call site
  that keeps one. Fix: scope it — "no seat-panel reader keeps a
  `*_SPECIALTIES` import (`states.ts:152`'s editorial constant is the only
  one left)".
- [nit][new] Rollout 6 — "a run that escalates legitimately sits above it, at
  ≈ 9 sessions — plan 1, implement 1, the fix rounds, and at most the code
  phase's 6 cumulative" — the named components sum to 10, and they cannot all
  be at their maximum at once: 6 cumulative code sessions is the pass-backstop
  shape, which dies inside the first round and spends **no** fix session
  (1+1+6 = 8), while the 2 fix rounds is the round-entry shape, which reaches
  only 4 code sessions (1+1+2+4 = 8). Both ceilings equal #12's `2 + seats ×
  3` = 8, so nothing pre-grant sits "above" that bound — the
  `--extend-budget` grants are the only thing that does, and the sentence
  lists them after. Fix: state the bound as `≤ 8` pre-grant (each escalation
  shape saturates at 8, not 10) and name the grants as what carries a run past
  it.
- [nit][re-raised] State graph delta vs. Budget resolution — "the deadlock stop
  … fires at 2 seats; with `maxRounds: 2` it is likelier to be the terminal
  record" vs. "`maxPassesPerRound: 3` is the record that prints on **every**
  shape of the round it starts" — the added "of the round it starts" narrows
  the claim to a round, which is the half that was vague, but it does not
  exclude the shape the neighbouring sentence names: the deadlock stop returns
  its `deadlock:` blocked reason from the same blocked site
  (`panel.ts:121-129`) and is that round's terminal record, so the pass-count
  string is not printed there. Fix: qualify — "every pass-count shape that is
  not the deadlock stop (`panel.ts:121-129`)".
- [nit][new] Options considered — "**The ADR's rejected alternatives, adopted
  unchanged here.** … each rejected there on the ownership-matrix and
  session-count grounds" — the delegation of the record to ADR-010 is fine, and
  the five names listed match the ADR's Alternatives exactly; the summary of
  the grounds is not true "each": "No plan artifact at all" is rejected on
  audit-record grounds (the seat stamp and the run record) and "`lane` as the
  field name" on a word-collision ground. Fix: drop the ground summary —
  "each rejected there on its own grounds; this design adopts the ADR's choice
  without re-arguing it".

## FYI (no severity)

- Context says a brief pays "the same twelve-stage, five-seat path"; ADR-010's
  Context says "13-stage path". The proposal anchors twelve to the
  `STAGE_BASES` rows and the author logged that count as settled; the other
  number lives in the source document's prose.
- Options' distinguishing cost for the threaded option is "a new parameter at
  each of the six `funnelSpecialties` call sites", while the adopted design
  threads `cfg` to the same readers (four gain a parameter, `planPass`'s
  callers thread it). The two costs are near-identical; the ADR effective-config
  check and the stale `CODE_SPECIALTIES` import are the real differentiators.
- #18's "one that writes through `docs/../src/x.ts`" is an agent-level (and
  therefore advisory) action; A28's intake claim covers what git reports, so
  that clause can only be exercised by writing outside the agent grant.
