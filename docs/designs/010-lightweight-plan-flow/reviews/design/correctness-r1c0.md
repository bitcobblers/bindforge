# correctness review r1

verdict: request-changes

Scope: whole document (full-panel pass c0, round 1). Inputs used: `02-proposal.md`
(incl. its Revision log), `docs/adrs/010-lightweight-plan-flow.md`, the four web
citations in the assumption register. No review reports or transcripts opened.

Checklist coverage: (1) assumption register — register correctly marks A23–A26
`unverified` and the body attributes those numbers to them, so no unverified
assumption is stated as fact; code-basis rows (A1–A20) were taken as given under
this panel's input restriction. (2) anti-hallucination spot-check — all four
cited URLs fetched live and each supports the claim it is cited for: SmartBear
("lightweight code review takes less than 20% the time of formal reviews and
finds just as many bugs") for A21; abseil ch09 ("only one LGTM is required",
"'Small' changes should generally be limited to about 200 lines of code",
"reviewed by precisely one reviewer") for A21; industrialempathy ("it is
absolutely possible to write a 1-3 page 'mini design doc'") for A22;
google eng-practices small-cls ("The number of files that a change is spread
across also affects its 'size'") for A22. No dead or contradicting source.
(3) internal contradictions — findings 1–5. (4) edge cases — finding 3.
(5) interfaces — finding 5. (6) decides the ADR — yes, the registry, frontmatter
gate, budget table, plan cap, status, promotion and test plan all answer
ADR-010's Decision. (7) revision rounds — Revision log's 13 r1 entries were read
first; none of the findings below is a rejection re-raised, and finding 1 is a
defect introduced by the closure of r1 OQ1/OQ2 (the single-resolver rule).

## Findings

- [major][new] Budget resolution / Test strategy #14 — "`maxParallelReviews:` //
  operator knob applied LAST" + "`--extend-budget` on the resume buys sessions
  under the light cap (the knob an operator actually buys, A16)" — the canonical
  rule orders `...base` then `...FLOWS[flow].config` and names exactly one knob
  applied after the flow partial (`maxParallelReviews`), so any budget increase
  that arrives on the same channel the CLI uses (A4's "CLI spread" and
  "`opts.cfg ?? DEFAULT_CONFIG`" seams) is overwritten by the flow partial for a
  light run. `--extend-budget` buys code-review sessions, i.e. raises
  `maxReviews.code` above the light `6`; applied through `base` it is silently
  discarded, so #14's assertion has no mechanism as the rule is written, and the
  budget table gives `maxReviews` no operator path at all. This is the design's
  own named hidden failure ("Budget drift on resume") in a new form: the operator
  extends, the run escalates again at 6. Fix: state where a resume-time extension
  lands — apply it to the *output* of `configFor` in the same posture as the
  `maxParallelReviews` line (e.g. `maxReviews: bump(resolved.maxReviews, extend)`),
  naming that this replaces "`...FLOWS[flow].config`" as the last writer of
  `maxReviews` for a light resume, and assert the extended number in #14.

- [minor][new] Budget resolution — "the reachable shape is exactly `c0` + two
  dissent passes = 3 two-seat passes" and "`maxPassesPerRound: 3` is the per-round
  backstop that can only bind at the same point" — the same paragraph says
  dissent passes are "≤2 seats each", and The `plan` stage / State graph delta
  keep the funnel's "revision to the dissenters only, ratification for stale
  approvals", so a one-dissenter round runs 1-seat passes: `c0` (2) + dissent (1)
  + ratification (1) = 3 passes at 4 sessions, where `maxPassesPerRound: 3` binds
  first and the terminal record is `fix-round backstop`, not the 6-session cap.
  Test strategy #13 is exactly that shape, and #14's "escalates at 6 code review
  sessions" only holds when both seats dissent every pass. Fix: qualify the
  claim — `maxReviews.code: 6` binds for the both-seats-dissent shape, and
  `maxPassesPerRound: 3` binds first at 4 sessions when a pass runs one seat —
  and say which escalation string the operator reads in each.

- [minor][new] The `plan` stage (oversized-plan gate) — "a missing section counts
  as 0 items (a faithfulness concern for the seats, not a gate)" — the file-count
  gate keys on the bullet count of `## Files to touch`, so a plan that names its
  files in prose, or under a renamed heading, counts 0 and passes the gate that
  exists precisely for the class the design fears; Risks row 2 asserts
  "`planMaxFiles` catches files-spanned", which the 0-count rule makes
  formatting-dependent. Fix: make the section required — deny with
  `the light plan requires a ## Files to touch section (N files, cap 10)` —
  replacing the "missing counts as 0" rule, keeping the bullet-count parse.

- [minor][new] Context / Chosen design / Compatibility matrix — "the same 13-stage,
  five-seat path" and "13-stage path, 5 seats, budgets unchanged" vs "`full`
  declares today's twelve stages" — the full flow's stage count is stated two ways
  in one document, and Test strategy #1 pins `FLOWS.full.stages` to "today's
  stages exactly", so the number is a contract, not prose. Fix: use the
  `STAGE_BASES` row count in all three places.

- [minor][new] Budget resolution / Options — "read `cfg.reviewSeats` and fall back
  to the five specialties when absent" vs "resolved from the flow exactly once" —
  the seat fallback is implemented twice (panel's `funnelSpecialties` and
  `states.ts`'s `specialtiesFor`), so `DEFAULT_CONFIG` is not the "sole default
  authority" for the seat panel, and the fallback direction is the unsafe one: any
  light call site handed a config literal that is not `configFor` output fans out
  to 5 seats and looks healthy. Fix: export one resolver from `flows.ts`
  (e.g. `seatsFor(phase, cfg)`) that both call sites use, so there is one fallback
  and one place to test, and name the residual "absent → full panel" direction in
  Risks row 4.

- [minor][new] Test strategy #6 vs Selecting the flow — "`flow: Light` and
  `flow: sideways` reach a denial" vs "throws `invalid flow "<value>" for <source>:
  expected full or light`" and "Unreadable files keep their existing
  `sourceAdmissible` denial" — #6 and #17 describe the same input with two
  different outcomes (source-gate denial vs pre-lock usage error); the prose picks
  the throw and reserves denial for unreadable files. Fix: change #6's verb to the
  pre-lock usage error and reserve "denial" for the unreadable-file case.

- [nit][new] The `plan` stage vs ADR-010 §Light stages — "It states no number" vs
  the ADR's "The `plan-note` skill carries the template and its length cap" — the
  design's placement (registry owns both numbers, prompt interpolates, skill
  carries the template) is the sounder choice and is argued in Options and
  Revision log r1 OQ4, but nothing records that it supersedes that ADR sentence,
  so the ADR and the proposal state the cap's home in two places. Fix: one clause
  in The `plan` stage (or Deferred) — "ADR-010's 'skill carries the length cap' is
  superseded: the skill carries the template, the registry owns the numbers".

## FYI (no severity)

- Goals state "Two deterministic stops for a too-big light run", while State
  graph delta records the funnel deadlock (`reviseNoChange`) as a third terminal
  record that "`maxRounds: 2`" makes likelier; wording/completeness, not a
  contradiction — consistency seat.
- ADR-010 places the bad-value case as "a source-gate denial, before the lock";
  the design makes it a pre-lock usage error because A18 shows
  `sourceAdmissible` runs after the lock. The reasoning is sound and the ADR's
  intent (no lock, no inference) is met, but the ADR's own sentence is now
  stale against the code — the ADR's owner, not this proposal, should record it.
- Test strategy names a target file only for #1; ADR-010's test plan maps the
  rest to `states.test.ts` / `gates.test.ts` per ADR-004's mirrored tree —
  test-strategy seat.
- `escalationPhase` puts `plan` in `CODE_STAGES`, which is shared with the full
  flow; unreachable there, and #10's byte-equality assertion covers it —
  compatibility seat.
