# performance review r1 (pass c3 — ratification of the post-r1c1 revisions)

verdict: approve

Scope read: the blob diff `24b804a → 4cc9acc` of `02-proposal.md`, the full current
`02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`, the Revision log, and this
seat's r1 findings. Checklist applied: `references/performance.md` only.

Baseline the design quotes and is measured against (Context + ADR run table): the six
completed runs at 43–96 stage sessions; run 003 = 43 sessions / 2.2 h for one CLI flag.
The design's converging bound (#12's `2 + seats × 3` = ≤ 8 sessions, ≤ 96 min at the
12-min stage bound) is a 5.4× session cut against that baseline, and the diff now states
that plainly, so checklist item 6 (expected numbers vs. baseline, explained) is satisfied.
The diff also removed one of the two frontmatter reads per entry (the post-lock
`sourceAdmissible` gate now takes the parsed result as an argument), which is a strict
reduction in per-entry file IO. No per-session allocation, no structure growth, no cache:
the cost surface here is the session count and the per-session timeout, so the findings
below are all session-arithmetic.

r1 disposition:
- r1 #1 (the 7-session shape attributed to pure both-seats dissent) — **closed**. The
  Budget resolution paragraph now reads "4–6 sessions", "the last pass a round can enter
  starts at spent 4", and #14(a) asserts the pass-backstop record at 6 sessions. That is
  the fix asked for, and `maxReviews.code: 6` / `maxPassesPerRound: 3` / `maxRounds: 2`
  are unchanged, so the knob choice r1 approved stands.
- r1 #2 (the bound stated as converging-only) — **closed**. Rollout 6 now names
  "**converging**-run bound" and discloses that an escalating run sits above it.
- r1 #3 (the 12-min bound covers review seats, which no assumption carries) — **partly
  open**; the residual is the nit below.

## Findings

- [minor][regression] Budget resolution + Test strategy #14(a) — "The session cap is
  checked at **pass entry**, ahead of that backstop (A16), so it never fires:
  `maxReviews.code: 6` is the ceiling the backstop coincides with, not a string an
  operator reads" / "#14(a) … the session-budget string never printing (Budget
  resolution)" — the closure of r1 #1 replaced the false 7-arithmetic with a universal
  negative that is true only for the single-round shape. Session accounting is phase-side
  and cumulative across rounds (full's `design: 40` exceeds one round's 5 seats × 6 passes
  = 30, so the cap spans rounds), and `maxRounds: 2` lets the code phase span two
  `code-review` rounds: R1 `c0` 2 sessions + one dissent pass 1 = 3 spent, dissent routes
  to `fix`, R2 `c0` enters at spent 3 and closes at 5, a pass entered at 5 is admitted and
  closes at 7, and the next pass entry at 7 is refused by the cap — the session-budget
  record prints. So the cap is the binding stop on the multi-round shape, not a phantom,
  and the design's real phase ceiling is 7 sessions, not 6. Cost bound and knobs are
  unaffected; the terminated-shape attribution that Rollout 6's escalated band and #14
  read from this paragraph is the part that is now wrong. Fix: scope the sentence — "the
  pass backstop terminates a single-round phase at 4 or 6 sessions; a phase that spans
  both `fix` rounds accumulates past the cap, so the session-budget record is the terminal
  string there and the phase ceiling is 7" — and keep #14(a) as shape (a) (it is correct
  there) with the multi-round cap-firing case named as the shape that prints the
  session-budget record. Text plus one integration case; no knob, no mechanism.
- [minor][new] Rollout 6 — "a run that escalates legitimately sits above it, at **10–12
  sessions** (Budget resolution)" — the escalated band's top is one session above what the
  section it cites allows, and it is a number written in response to r1 #2, so the
  recorded-data target for the escalation path is off. Under the design's own arithmetic
  the fighting run is 1 `plan` + 1 `implement` + 2 `fix` (maxRounds 2) + the code-phase
  ceiling of 7 (the pass entered at spent 5, closing at 7) = **11** sessions; 12 is not
  reachable, because a pass entered at 6 or more is refused. The wall-clock reading is
  what the number is for: 11 sessions × 12 min = 132 min = run 003's 2.2 h exactly, the
  parity point this document concedes; 12 sessions reads as 144 min, i.e. the light flow
  losing to the baseline it exists to beat, which the cited section does not support.
  Fix: "a run that escalates legitimately sits above it, at 10–11 sessions ≈ 120–132 min
  at the same 12-min bound — parity with run 003's 2.2 h (Budget resolution)". Text only.
- [nit][re-raised] Assumption register A25 + Risks — "A 12-minute `stageTimeoutMs` is
  enough for the light flow's `plan`/`implement` stages" and "12-minute stage timeout is
  too tight for a light `implement`" — the diff's acceptance target ("≤ 96 min", i.e. 8
  sessions × 12 min) is dominated by code-review seats (6 of the 8 in #12's shape), while
  the only rows that register the 12-min knob's adequacy name `plan`/`implement`, and
  Rollout 6 records "sessions and wall clock" totals, which settle the aggregate but not
  A25's own stated datum ("per-stage durations were never aggregated"). Residual of r1
  #3, not a break introduced by this diff. Fix: name the code-review seats in A25's and
  that Risks row's wording, and have Rollout 6 record per-stage durations beside the run
  totals so the A25 settlement has its datum. Registry wording plus one recorded column.

## FYI (no severity)

- State graph delta — "The light suffix is appended to every code-phase escalation the
  light flow reaches: `decideNext`'s pass-backstop string … and fix-round string … and the
  runner's round-entry … backstop reached (2/2 rounds)" — if the session cap does fire
  (the multi-round shape above), the cap's escalation string is a fourth code-phase record
  and is not in the list of three, so ADR-010 §Status ("an escalated light run must read
  as 'this was too big for the light flow'") has an uncovered exit. Correctness seat's.
- Registry section + Rollout 1 — #3 was narrowed to deep-equality against a **complete**
  literal, while Rollout 1's step-1 no-op claim states "`configFor('full', base)` is
  deep-equal to `base`" unqualified; the partial-literal case the additive-shape guarantee
  rests on is now asserted nowhere. Complexity/compatibility seat's.
