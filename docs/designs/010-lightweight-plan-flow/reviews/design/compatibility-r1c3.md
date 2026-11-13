# compatibility review r1c3 (ratification of post-r1c1 revisions)

verdict: approve

Scope read: the blob diff `24b804a..4cc9acc` of `02-proposal.md`, then the full current
document, `docs/adrs/010-lightweight-plan-flow.md`, the Revision log, and this seat's
r1c1 findings. No review report, transcript or source file was opened.

## Checklist walk (compatibility)

1. Matrix rows — every row carries a status; no `unknown`, no empty cell. Windows/macOS
   read `no` with a named reason and a Deferred follow-up. Pass.
2. OS input APIs — the design adds no OS surface; the plan meter is stated as the JS line
   count, not a shell-out, so nothing new is Unix-dependent. Pass (A20 unchanged).
3. Node floor — matrix names `>= 22.23.4` and `no` below it; the diff adds no API above
   the floor. Pass.
4. Native modules — none introduced by the changed text. Pass.
5. Keyboard reality — out of scope for this design; unchanged. Pass.
6. File paths — the rewritten Ownership paragraph now states POSIX/repo-root-relative
   arrival **and** discloses "case is **not** normalized — as today", which is
   self-consistent on the only supported platform (linux, case-sensitive) and matches
   #7. No string concatenation added. Pass.
7. Platform test coverage — the matrix claims only linux; #21 names the linux suite at
   the `engines` floor. Pass.

## Round-1c1 findings — disposition

- `validated` invariant / absent-key case (**closed**). The sentence now reads
  "must keep `planMaxLines` **absent or** a positive integer (the full flow's config
  keeps it absent)", the knob table's `planMaxLines | absent (no gate)` is no longer
  contradicted, #5 keeps the no-gate assertion, and #23 pins "`planMaxLines: 0` throws
  where its absence passes". The absent-`reviewSeats` leg is pinned by #3 ("a complete
  literal without the new optional fields ... resolves to the five seats") and by the
  `seatsFor` five-seat fallback living once. `configFor('full', { ...DEFAULT_CONFIG,
  maxParallelReviews: 1 })` is exercised by #2's shape and #15.
- #21 "both compatibility rows" (**not addressed**, unchanged text) — re-raised as a nit.
- Matrix row's stage count (**not addressed** by the fix I proposed) — the Revision log
  closes the theme as "every mention is the twelve `STAGE_BASES` rows". That rebuttal
  rests on a count in `stages.ts` that is not in this seat's inputs, so it cannot be
  shown unsound; moved to FYI.

## Findings

- [minor][new] Test strategy #23 — "`reviewSeats.code: []` and an empty `reviewSeats.design`
  throw" — the design-panel clause names no flow, and the `validated` sentence it is
  pinning carves the opposite out for light: "must keep the two keys `seatsFor` reads
  non-empty — `reviewSeats.code`, and `reviewSeats.design` … `light`'s `design: []` is
  legal only because `light.stages` names no design stage". The knob table sets
  `reviewSeats.design | [] (no design phase)` in `FLOWS.light.config`, so a literal
  reading of #23 pins a test where `configFor('light', …)` throws — which contradicts #2
  (`configFor('light', …)` yields the light values), #15 and #12, and deletes the flow.
  The clause is only true for the flow whose `stages` runs design stages. Fix: qualify
  the clause — "an empty `reviewSeats.design` throws for `full`, whose `stages` runs
  design stages, and resolves for `light` (the registry's `design: []`)" — so #23 names
  the flow per clause, as its last clause already does. No new mechanism; the rule the
  section already states becomes testable both ways.
- [nit][new] Compatibility matrix / Rollout 1 — "`configFor('full', base)` is deep-equal
  to `base`; … (Test strategy #3)" — the registry paragraph narrowed this claim to
  "deep-equal to a **complete** `base` — the partial-literal case is Test strategy #3",
  and #3's second clause pins a partial literal by *compiles + resolves to the five
  seats*, not by deep equality. The two unqualified citations (matrix row "Existing ADRs
  and briefs", Rollout 1's no-op claim) now point at a test that is narrower than the
  sentence quoting it; both are true for the complete CLI-shaped config they describe, so
  nothing is false — the citation is imprecise. Fix: borrow the qualifier, "deep-equal to
  a complete `base`".
- [nit][re-raised] Test strategy #21 — "no macOS or Windows job exists, so both
  compatibility rows read `no` and no test claims them" — the matrix carries one merged
  row, `| Windows / macOS | no | …`; a reader cross-checking the anchor finds one row,
  not two. Unchanged by this revision. Fix: "so the `Windows / macOS` row reads `no` and
  no test claims it".

## FYI (no severity)

- ADR-010 Context calls the current path "the identical 13-stage path"; every proposal
  mention is now "twelve `STAGE_BASES` [stages]" (Context, registry, matrix row, Revision
  log). The proposal is internally consistent; the ADR still reads 13.
- Ownership: `plan`'s predicate rule is now `return inDocs`, `/^docs\//`, so at the
  predicate level a light `plan` may write any path under `docs/` — other runs' design
  directories, `docs/adrs/**`, `docs/workflows.md` — with confinement to
  `docs/designs/**` carried by the agent config. Ownership/security surface.
- Naming: the same code-phase record is "the fix-round backstop" in the State-graph row
  and #8, and "the runner's revise-round record" in Budget resolution, where the quoted
  string is `code review revise-round backstop reached (2/2 rounds)`.
- Rollout 6's escalated band "at 10–12 sessions (Budget resolution)" against Budget
  resolution's 4–6 sessions per non-converging round with `maxRounds: 2` plus the
  `plan`/`implement`/`fix` sessions — wall-clock arithmetic for the performance seat.
