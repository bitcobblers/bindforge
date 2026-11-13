# Performance review r1 (c6 ratification pass)

Scope: the blob diff `4cc9acc..c1b351d` of `02-proposal.md`, then the full current document.
Ratification question: did the post-approval revisions break what the performance seat
approved, or make a section it cited false? Checklist applied: `references/performance.md`
(item 6 — design's expected numbers vs. the quoted research baseline — is the live item for
this design; items 1–5 are orchestration-cost analogues: sessions per stage, wall clock,
per-run config resolution).

verdict: approve

## Status of the seat's r1 findings

- r1 #1 (the "session cap never fires" universal negative) — **closed, correctly.**
  Budget resolution now scopes the claim ("inside the first code round the backstop prints
  first"), names the accounting that makes it phase-side ("`reviewSessionsUsed` counts every
  `code-review:` result in the run"), names the shape where the cap is the binding stop
  (a `--stage code-review` force after a round closed at 6 spent → `runner.ts:2362-2368`),
  and #14(a) is scoped to "not printing **in that round**" with the new #14(c) asserting the
  session-budget record at 6/6. The knob values and the 4–6 per-round bound I approved are
  untouched, and `maxReviews.code: 12`'s rejection still reads correctly now that it says
  "every **round** is bounded in the 4–6 band". No residual.
- r1 #2 (the escalated band, 10–12 sessions) — the number was replaced, and 12 is gone; the
  replacement is arithmetically inconsistent with its own decomposition. Re-opened as
  finding 1 below (new text, not the old number).
- r1 #3 (A25's datum) — still open; the sentence that asserts the settlement is inside the
  changed text. Finding 2 below.

## Findings

- [minor][new] Rollout 6 — "a run that escalates legitimately sits above it, at **≈ 9
  sessions** — plan 1, implement 1, the fix rounds, and at most the code phase's 6 cumulative"
  — the band's value is not the sum of the components the same sentence lists, and its
  claimed position is not above the bound it is measured against. The listed components are
  1 + 1 + 2 (`maxRounds: 2`, the fix rounds) + 6 (the code phase's cumulative ceiling) =
  **10**, not 9. And the shape that escalates without a force — the round-entry backstop at
  2/2 after two `fix` sessions, the code phase closing at its in-round 4 (Budget resolution's
  one-dissenter pass-backstop figure) — is 1 + 1 + 2 + 4 = **8**, which is exactly #12's
  converging bound (`2 + seats × 3` = ≤ 8), so it does not "sit above it". The escalated band
  is the recorded target for the escalation path, i.e. the datum Rollout 6 collects and the
  number A23/A25 settle against; as written it names one value no shape produces (9), hides
  the decomposition's ceiling (10), and asserts a separation from the converging bound that
  the 8-session shape contradicts. The direction of the claim survives at both ends — 8–10
  sessions at the 12-min bound is 96–120 min against run 003's 43 sessions / 2.2 h (132 min),
  so the light flow stays under the quoted baseline — only the band's arithmetic is wrong.
  No knob, no mechanism. Fix: state the band as the range its shapes produce: "a run that
  escalates legitimately sits at **8–10 sessions** — 1 `plan` + 1 `implement` + at most 2
  `fix` + the code phase's 4 (round-entry backstop) to 6 (pass backstop) cumulative —
  **96–120 min** at the 12-min bound, under run 003's 2.2 h; the low end ties #12's bound,
  so the escalated band separates from it only through the code phase's 6-session
  cumulative (Budget resolution)."
- [minor][re-raised] Assumption register A25 + Risks (row "12-minute stage timeout is too
  tight for a light `implement` (A25)") + Rollout 6 — "A 12-minute `stageTimeoutMs` is enough
  for the light flow's `plan`/`implement` stages" / "The same data settles A23/A25" — the
  settlement claim sits in the sentence this diff rewrote, and the data it names still cannot
  produce A25's datum. Rollout 6 records "sessions and wall clock with the ADR's counting
  rule" — run totals. A25's own basis states the missing datum: "per-stage durations were
  never aggregated"; a run total of 8 sessions in ≤ 96 min is compatible with one stage
  blowing the 12-minute knob (a serial lane is seats × per-seat wall clock, per the
  Compatibility matrix), so the aggregate does not settle a per-stage bound. Two light stages
  that carry the 12-min knob are not named by A25 or the Risks row: `fix` — and `fix` is in
  every shape in the escalated band above (2 of the 8–10 sessions), so the band Rollout 6
  measures is partly composed of a stage whose timeout adequacy is unregistered. Not a
  regression introduced by this diff; it is the r1 #3 residual plus the `fix` gap, and the
  diff's "**converging**-run bound" qualifier and the escalated disclosure cover the
  wall-clock half. Registry wording plus one recorded column; no knob change. Fix: name
  `plan`/`implement`/**`fix`** in A25 and that Risks row, and have Rollout 6 record
  **per-stage durations** beside the run totals, so the A25 settlement has the datum it names.

## FYI (no severity)

- ADR-010's Context counts "the **seven** runs on disk" over a six-row table and calls the
  full path a "**13**-stage path"; the proposal says "the **six** completed runs" and "today's
  **twelve** stages" (unchanged text, and the Revision log's r1 entry pins "the stage count is
  only the twelve `STAGE_BASES` rows"). One of the two documents is off by one on the stage
  count and the ADR is off on its own row count. Compatibility/correctness surface, and
  outside this seat.
- The Options row that now defers the ADR's rejected alternatives to `docs/adrs/010` is
  accurate: the ADR's Alternatives considered section carries all five named items (second
  runner, config-only variants, no plan artifact, keeping the test-authoring engineer,
  `lane`), each on ownership-matrix and session-count grounds. The deferral drops the
  proposal's copy of the baseline figures, but Context still quotes the research baseline
  (43–96 stage sessions across the six runs), so checklist item 6 stays satisfied.
