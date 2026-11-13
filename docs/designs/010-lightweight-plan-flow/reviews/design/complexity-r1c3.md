# complexity review r1 (ratification c3)

verdict: request-changes

Scope: `git diff 24b804a 4cc9acc` on `02-proposal.md`, then the full current document.
Checked against the complexity checklist (reuse, Goal-trace, smaller-design sketch,
dependencies, sliceability, knobs with no consumer / abstraction with one implementation,
proportionality). Closure-generated defects are reported per the skill's rule that a
closure can break text it did not touch.

## Findings

- [major][new] Budget resolution, `validated` paragraph — "must keep the two keys
  `seatsFor` reads non-empty — `reviewSeats.code`, and `reviewSeats.design` …
  `light`'s `design: []` is legal only because `light.stages` names no design stage" —
  the rule is stated over **a resolved config**, unconditionally, and the light partial in
  the same section sets `reviewSeats.design: []` (budget table row:
  `reviewSeats.design | the 5 design specialties | [] (no design phase)`), so
  `configFor('light', base)` = `validated({…DEFAULT_CONFIG, …base, …FLOWS.light.config})`
  throws on the flow's own registry value: the light flow cannot resolve. The round-1 text
  was coherent here ("a non-empty seat list **for every phase the flow's `stages` runs**");
  the security closure rewrote it into a flat two-key non-emptiness rule and left the
  light exemption as a parenthetical that the rule forbids. Test strategy #23
  ("`reviewSeats.code: []` **and an empty `reviewSeats.design` throw**") reads as
  unconditional and so asserts a throw for exactly the config Test strategy #1 pins
  ("`reviewSeats.design` is empty" for `FLOWS.light`), and Rollout 1 ships `configFor` with
  this contract. The non-emptiness check has a real consumer for the **code** panel
  (`seatsConverged` is vacuously true on `[]`, as the paragraph says) and for **design** via
  the editorial path (`states.ts:152` passes `DESIGN_SPECIALTIES`); it has no consumer for a
  flow that runs no design phase, so the smaller contract is the scoped one.
  Fix: restore the scope in one clause — non-empty "for every phase the flow's `stages`
  runs", keeping `code` unconditionally non-empty since every flow runs code review — and
  scope #23 to `full` ("an empty `reviewSeats.design` throws under `full`; under `light` the
  registry's own `[]` resolves"). No mechanism added; the exemption already in the
  parenthetical becomes the rule's condition, and `validated` keeps its current inputs.

- [minor][new] State graph delta, escalation-suffix paragraph — "The light suffix is
  appended to **every** code-phase escalation the light flow reaches: `decideNext`'s
  pass-backstop string (`states.ts:183`) and fix-round string (`states.ts:187-194`), and
  the runner's round-entry `code review revise-round backstop reached (2/2 rounds)`" — the
  enumeration is not exhaustive, and the same revision's new claim that the session cap
  "never fires" is what justifies the omission. The cap is **per-phase cumulative**
  (`reviewSessionsUsed` counts every `code-review:` result in the run,
  `runner.ts:2358`/`panel.ts:50-52`), while `maxPassesPerRound` is per-round, so
  `maxReviews.code: 6` (one non-converging round's ceiling) is spent inside round 1 and any
  later code-round entry hits `spent >= cap` → the session-budget escalation at
  `runner.ts:2362-2368`, which is a code-phase escalation the light flow reaches and is not
  in the list. The reachability is the design's own recovery path: `--stage code-review`
  passes `assertStageInFlow` for light and forces a new round (`states.ts:88-92`), the case
  that comment calls "the obvious thing to force after a round dies to a crashed reviewer"
  — i.e. the run whose round just died to the pass backstop at 6 spent.
  Smaller design for the same requirement: all three strings (and the session-budget one)
  are written by one function, `finalizeEscalate(repoDir, state, reason, cfg)`
  (`runner.ts:2229`, `2362`, `2387`), which already computes `escalationPhase(state)` at
  `runner.ts:2861`. Appending the suffix there, gated on `flowOf(state) === 'light' &&
  phase === 'code'`, is **one** site that covers every code-phase escalation and none of the
  plan-stage ones (whose string already names the flow and the promotion recipe, so a
  choke-point append that ignored `phase` would duplicate that text). That replaces three
  string edits — the growth this round's r1c1 closure added — with one, and makes
  "every" true by construction.

- [minor][new] Rollout 6 — "a run that escalates legitimately sits above it, at 10–12
  sessions (Budget resolution)" — the number is not derivable from the section it cites, and
  the citation points at the arithmetic this diff rewrote. With `maxReviews.code: 6` as a
  phase-cumulative cap (see the finding above), a light run's code phase cannot reach 12
  sessions: plan 1 + implement 1 + fix rounds + at most 6 code sessions, so the escalated
  ceiling is ≈ 9 sessions plus one `--extend-budget` grant per over-cap pass, not 10–12.
  Fix: state the escalated ceiling as what the caps produce (code phase ≤ 6, whole run ≈ 9,
  plus extension grants the operator buys), and drop the "(Budget resolution)" pointer for
  the number that section does not support. Registry values only; no mechanism.

## FYI (no severity)

- Round-1 finding 1 (complexity, minor, `seatsFor(phase, cfg)` delegation) is untouched by
  this revision, not broken by it: the quoted sentence survives verbatim, and the residual
  it used to carry now lives in Risks row 4, which the new parenthetical
  "(residual: Risks row 4; `states.ts:152` is the editorial path)" points at, so the
  compression is honest. The gap is unchanged — `funnelSpecialties` has no `cfg` at
  `dissentingSeats`/`staleSeats`/`seatsConverged`/`seatSummary` (`panel.ts:55,65,74`; the
  post-delegation signatures that need it), only `planPass` carries `cfg`
  (`panel.ts:100`), `specialtiesFor` is called from `runner.ts:2345` with none — so an
  implementer at `seatsConverged(state, phase)` still has no stated route to the panel and
  can keep the `CODE_SPECIALTIES` import. Options' "A threaded parameter was rejected: six
  changed signatures" still describes the rejected design (a seat list threaded into the
  funnel from outside), not the four wrappers' `cfg` parameter. Not a ratification finding
  under the pass rules; noted so the reviser knows it is still open at minor.
- Value check (checklist 7): the design's added surface — one registry module, two optional
  config fields, one resolver, one seat resolver, one pre-lock validation, one gate, one
  template, one agent, one skill — is proportionate to what it buys (a 43-session run class
  drops to ≤ 8, ADR-010's Context table). No new dependency, no new package, no second
  runner, no abstraction with a single implementation beyond `seatsFor`, whose two
  delegates are the duplication it removes. The `maxReviews.code: 6` row is the one
  registry value the section now describes as printing nothing; it is fixed by the ADR's
  budget table, so the answer is the wording, not the number.
- The Ownership rewrite (Ownership — "`return inDocs`, i.e. `inDocs = /^docs\//`
  (`gates.ts:304,315)`) is a net simplification and verified: `gates.ts:315` is the
  document-stage `return inDocs`, `gates.ts:304` is that regex, so dropping the
  run-directory confinement claim removes predicate work and keeps the agent-config
  boundary; the Deferred note that a files-spanned stop needs a **new** extractor is the
  honest version of the previous claim.
