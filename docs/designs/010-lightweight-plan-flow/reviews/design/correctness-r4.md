# correctness review r4

scope: `reviews/design/editorial-r4.diff` (ratification/confirmation pass — the language-only
editorial pass of round 4). Inputs read: `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`,
the proposal's Revision log, the diff. No prior correctness findings in this run's editorial phase.

verdict: approve

## Findings

- [minor][new] Budget resolution — "`states.ts:152` keeps `DESIGN_SPECIALTIES` (editorial only, Risks row 4)"
  — the diff appended an `Editor notes:` line that leaves this cross-reference unresolved, and it is
  the one changed-adjacent claim in the diff that a reader cannot check: Risks row 4's headline is the
  resume budget-drift row, the editorial-round absence is row 1, and only row 4's trailing
  "Residual: a light call site … falls back to the five-seat panel" clause makes the row-4 citation
  defensible — i.e. the citation is true through a clause the parenthetical never names, so the two
  sections do not visibly agree. Fix: make the referent explicit, e.g. "(the one five-seat fallback
  left; editorial round only — Residual of Risks row 4)", and delete the editor note.
- [nit][new] Budget resolution — "`maxRounds: 2` bounds the mutating `fix` loop, on the runner's revise-round record"
  — "revise-round record" names this escalation nowhere else in the document: the same stop is "the
  fix-round backstop" (State graph delta, `case 'code-review'` backstop exhausted), "the fix-round
  string as `decideNext` writes it" (#8) and "the pass-backstop record" (#14). Checklist 5: one thing,
  several names; "revise round" is also a design-phase term the light flow never runs, so it can be
  read as a record outside the enumeration at "No `states.ts` string changes (#8)". Fix: reuse the
  established name — "… bounds the mutating `fix` loop, and its exhaustion prints the fix-round
  backstop string (State graph delta)".
- [nit][new] End of document, after the Revision log — "Editor notes: … author to clarify."
  — two open questions now sit in the proposal body as unowned prose, outside the Revision log, which
  is the run's record of what the author decided; a reader cannot tell whether the lines they cite are
  approved text. Fix: the author resolves the two flagged sentences (the fixes above are the smallest)
  and removes the notes, or moves them into the Revision log as open items.

## FYI (no severity)

- Rollout 6 (context line of the changed hunk, not changed by this pass) — "at ≈ 9 sessions — plan 1,
  implement 1, the fix rounds, and at most the code phase's 6 cumulative" — its own components sum to
  10 (1 + 1 + 2 + 6); the "≈" hedges it, so this is only worth folding in if the line is touched again.
- Overview "the six completed runs spent 43–96 stage sessions each" against ADR-010's "The seven runs
  on disk" table (six rows) is a counting-word difference in the source document, already handled by
  the Revision log's stage-count closure; not re-opened here.

## Checklist notes

- Changed text checked line by line against the sections it touches: the Overview "takes" wording, the
  four Goals parallelism edits, the `maxPassesPerRound: 3` sentence, the Ownership precedence sentence,
  the A28 intake clause, and the Rollout 6 "and against run 003" splice. None shifts a technical claim:
  the precedence order (reserved names first, widening confined to `implement`/`fix`, full keeps
  `!inTests`) still matches ADR-010's "Light stages" and A8/A9; the "knob that binds … pass-backstop at
  6 / 4 sessions" sentence still agrees with #14(a)/(b) and with the pass-entry cap ordering in A16;
  `2 + seats × 3` = 8 sessions and 8 × 12 min = 96 min in Rollout 6 still agree with #12.
- Assumption register and research URLs: not re-audited — no `verified` row and no citation is added,
  removed, or re-worded by this diff, and the register was the target of earlier rounds' closures.
- Deferred section: every deferral still tracks ADR-010's Deferred list; the one documented divergence
  (the cap's number in the registry, interpolated into the prompt, superseding the ADR's skill-carries-the-cap
  wording) is recorded as a deliberate decision in the Revision log; nothing here contradicts the ADR.
- No blocker or major open → approve.
