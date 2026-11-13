# complexity review r1 (dissent pass c4)

verdict: request-changes

Scope: the diff `4cc9acc → 640d904` (02-proposal.md blobs), then the full current
document. Inputs: the proposal, `docs/adrs/010-lightweight-plan-flow.md`, the
Revision log, my r1 findings. No review reports or transcripts opened.

## r1 findings — status

- **major — the flat non-emptiness rule made `light` unresolvable** — **closed.**
  Budget resolution now reads "must keep `reviewSeats` non-empty **for every phase
  the flow's `stages` runs** — `code` for every flow ... and `design` under `full`",
  and the light exemption is the rule's condition, not a parenthetical
  ("A flow that names no design stage resolves the registry's `design: []` as
  written ... so `configFor('light', base)` resolves"). Test strategy #23 is scoped
  ("throws under `full` and **resolves** under `light`") and agrees with #1. One
  residue is left as a nit below (the `validated` pseudo-code's argument list).
- **minor — the escalation-suffix enumeration was not exhaustive** — **closed** in
  the form I proposed: one append at `finalizeEscalate`, gated on flow and phase.
  Verified the site: `finalizeEscalate` is `runner.ts:2845` and does compute
  `escalationPhase(state)` at `runner.ts:2861`; the four code-phase escalations
  named (`next.reason` at 2229, session-budget 2362, round-entry 2387) are all
  `finalizeEscalate` calls, so the choke point covers them, and
  `finalizeEscalate` is the only escalation writer in the drive loop. The
  "never fires" clause is gone and the cumulative-cap arithmetic checks out
  (`reviewSessionsUsed` counts every `code-review:` result, `panel.ts:50-51`; a
  `--stage code-review` force after a round dies at 6 spent hits `spent >= cap` at
  `runner.ts:2359` and escalates at 2362). One residue is a nit below (the gate's
  cited discriminator).
- **minor — Rollout 6's escalated bound** — **closed.** "≈ 9 sessions — plan 1,
  implement 1, the fix rounds, and at most the code phase's 6 cumulative ... plus
  one `--extend-budget` grant per over-cap pass" is derivable from the caps
  (1 + 1 + 1 fix + 6 code sessions = 9 in the shape where the cumulative cap binds
  at a later round entry), and the pointer to Budget resolution is now earned.

## Findings

- [major][new] Budget resolution — "The four funnel readers — `dissentingSeats`,
  `staleSeats`, `seatsConverged`, `seatSummary` (`panel.ts:55,65,74,181`) — gain
  the `cfg` parameter their callers in `runner.ts` and `states.ts` already hold,
  so no call site keeps a `*_SPECIALTIES` import" — the enumeration names four of
  the **five** readers of the seat list, and the missing one is the reader that
  chooses which seats launch. `funnelSpecialties(phase)` is called at
  `panel.ts:57,68,77,103,104,184`; lines 103-104 are inside `planPass`
  (`panel.ts:100`), whose `{ pass: 'full', seats: [...funnelSpecialties(phase)] }`
  is handed straight to `runReviewPass` as `seats` (`runner.ts:2453`) — the
  proposal's own A2 says "the pass loop itself takes seats from `planPass`". With
  only the four named readers threaded, `planPass` keeps the un-scoped
  `funnelSpecialties(phase)`, so a light run's first code pass launches the five
  `CODE_SPECIALTIES` seats: the light flow pays the exact cost ADR-010 exists to
  remove, `maxReviews.code: 6` is spent by pass one plus a partial, #12's
  `2 + seats × 3` bound and Rollout 6's yardstick stop holding, and the ledger then
  records five seats while `dissentingSeats`/`seatsConverged`/`seatSummary` read
  two — dissent from the three uncounted seats is invisible. It is also the site
  #9 pins ("a full pass names both seats"), so the test asserts behavior the
  change list never adds. `planPass` is the one reader that **already takes
  `cfg`** (`panel.ts:100`), so this costs zero new parameters — exactly the
  property the sentence's rationale ("the `cfg` their callers already hold")
  claims. Fix: name `planPass` in the enumeration (five readers,
  `panel.ts:55,65,74,100,181`), stating that its two `funnelSpecialties` sites
  (`panel.ts:103,104`) pass the `cfg` its signature already carries, so the
  full-pass seat list is the flow's panel. Registry/threading only; no new
  mechanism.
- [minor][new] Budget resolution — the `configFor` pseudo-code
  "`configFor(flow, base) = validated({ ...DEFAULT_CONFIG, ...base, ...FLOWS[flow].config })`" —
  the validation rule this round re-scoped is now flow-dependent ("the check is
  scoped to the phases the flow's `stages` runs", #23; "A flow that names no design
  stage resolves the registry's `design: []`"), but the canonical block passes
  `validated` the merged config **only**, where it can see no `stages`. An
  implementer who follows the block writes the unconditional two-key check again and
  reopens the defect this round closed: `configFor('light', base)` throws on the
  registry's own `design: []`. Fix: pass the flow (or its `stages`) into the
  helper — `validated(merged, flow)` — in the block, so the block states the rule
  the prose now carries. One argument; no new mechanism.
- [nit][new] State graph delta — "The light suffix is appended at **one** site —
  `finalizeEscalate` (`runner.ts:2845`), which already computes
  `escalationPhase(state)` — gated on `flowOf(state) === 'light'` and a code-phase
  escalation from `implement`/`fix`/`code-review`: ... and skips the plan stop" —
  the gate is right, but the cited discriminator cannot do the skipping once the
  Reporting section's change lands: `plan` joins `CODE_STAGES` in `report.ts`
  (`CODE_STAGES` feeds only `escalationPhase`, `report.ts:49-57`), so at the
  plan-size stop `escalationPhase(state)` is `code` and a phase-gated append
  doubles the promotion recipe onto a string that already names the flow and
  `--force`. Fix: one clause — the gate keys on the **stage**
  (`implement`/`fix`/`code-review`), and `escalationPhase` is not the discriminator
  because a light `plan` stop reads `code` by the Reporting change.

## Checklist

1. New module: `flows.ts` is ADR-010's named artifact; no in-repo equivalent
   (the budgets/seats are today split across `stages.ts`, `panel.ts`,
   `states.ts`). Nothing in the diff adds a package.
2. Moving parts → Goals: the diff's additions all trace — the `cfg` threading
   serves "seats declarable per flow" + the ADR's effective-config check; the
   single escalation site serves "an escalated light run reads as too big for the
   light flow". No new knob in the diff lacks a consumer.
3. Smaller design: the diff **is** the smaller design — the Options entry hands the
   ADR's rejected-alternatives record to the ADR (one authority), the suffix moves
   from three string edits to one writer, Ownership drops the run-scoped branch
   ("no new predicate"), Deferred names the extractor a future `planMaxFiles`
   would need. Net deletion of both prose and code paths.
4. New dependencies: none.
5. Thin first slice: Rollout 1 stays a no-op for every existing run (#3) and the
   `cfg` threading lands with it; nothing in the diff forces the light surface to
   ship at once.
6. Knobs/abstractions: `FlowDef.stages` gains a second consumer from this diff
   (the validation scope), `docStages` the meter, `config` the resolver. The
   seat resolver has one implementation and five readers — the enumeration above is
   the only gap.
7. Value: proportionate, and leaner than what I reviewed in r1. The mechanism
   count is unchanged (registry + one config field pair + one flow parameter + one
   gate + one append site); the diff removed duplication rather than adding
   machinery.

## FYI (no severity)

- Budget resolution calls the `--stage code-review` force "the recovery its comment
  names", but `states.ts:88-92`'s comment describes forcing a round after a round
  dies to a **crashed reviewer** — a scenario that leaves sessions below the cap, so
  it never reaches `runner.ts:2362`. The reachability argument for the
  session-budget escalation is the post-backstop force (mechanically sound: resume
  has no status gate, `runner.ts:2125-2148`), not the comment; the citation is
  loose. Correctness seat's call.
- Rollout 6: "one `--extend-budget` grant per over-cap pass" counts grants, and a
  granted pass entry launches 1–2 sessions (`runner.ts:2359-2371` decrements one
  grant per entry), so measured sessions can sit a seat above the grant count.
  Measurement yardstick only.
