# performance review r3c2

verdict: approve

Scope as given: the diff `92f71e0..4635f0d` of `02-proposal.md` (A12, the Budget
resolution validation paragraph, the `planner.md` bullet, one Compatibility-matrix row,
Test strategy #17, two Revision-log entries), then the full current document. Ratification
pass: findings only where the revisions broke something this seat approved in r3, or made a
section this seat cited false.

What the diff does to this seat's surface: it moves `validated` for a caller-supplied
`opts.cfg` to **pre-lock** (`runPipeline`/`resumePipeline`, once the flow is known, before
`acquireLock`/`freshState`) and widens A12 to name the `doc-sync` exception. Nothing in the
diff touches the knobs this seat's numbers run on — `stageTimeoutMs: 12 min`,
`maxReviews.code: 6`, `maxPassesPerRound: 3`, `maxRounds: 2`, the 4–6 band, `#12`'s
`2 + seats × 3` bound, Rollout 6, A25, the Risks timeout row. The pre-lock validation adds no
measurable cost (one object check on a config literal, no client call, no lock churn; the
frontmatter is still read once per entry, which the r3 text already counts as a read
reduction), and the baseline citation the checklist asks for is intact: Rollout 6 still
quotes run 003's 43 sessions / 2.2 h against the ≤ 8 sessions / ≤ 96 min expectation, and the
ADR's counting rule (sessions = entries in `results`, wall clock = the sum of `durationMs`)
is the one quoted. No checklist item is now unmet that was met in r3.

The three open performance findings from r3 sit in text this diff did not touch, so they
survive verbatim; the Revision-log rewrite closes half of the first one (it no longer records
the escalated ceiling as settled at ≈ 9) and leaves the Rollout 6 figure itself standing. They
are re-listed below as re-raised, with the label telling the reviser they are not caused by
this revision and are not ratification blockers.

## Findings

- [minor][re-raised] Rollout 6 — "a run that escalates legitimately sits above it, at ≈ 9
  sessions — plan 1, implement 1, the fix rounds, and at most the code phase's 6 cumulative
  (Budget resolution)" — the band's value is not the sum of the components the same sentence
  lists, and its low end is not above the bound it is said to sit above. 1 plan + 1 implement
  + at most 2 fix (`maxRounds: 2`) + 6 code-phase cumulative = **10**; the shape that
  escalates with no operator force — Test strategy #14(a), the pass-backstop record at 6
  spent sessions in round 1, which terminates the run before any fix — is 1 + 1 + 6 = **8**,
  and the one-dissenter shape #14(b) through a fix round and a second code round is
  1 + 1 + 1 + 4 + 2 = **9**. So the honest band is 8–10, and 8 is exactly #12's
  `2 + seats × 3` = 8, i.e. the escalated band separates from the converging bound only
  through the code phase's 6-session cumulative, not through the stage count. This is the
  datum Rollout 6 collects and A23/A25 settle against. Direction survives at both ends: 8–10
  sessions is 96–120 min at the 12-min bound, against run 003's 43 sessions / 2.2 h. Fix
  (text only): "a run that escalates legitimately sits at 8–10 sessions — 1 plan + 1
  implement + at most 2 fix + the code phase's 4 (round-entry backstop) to 6 (pass backstop)
  cumulative — 96–120 min at the 12-min bound, under run 003's 2.2 h; the low end ties #12's
  bound, so the escalated band separates from it only through the code phase's 6-session
  cumulative (Budget resolution)." Note for the reviser: this diff's Revision-log rewrite
  dropped the clause that recorded ≈ 9 as settled ("and states the escalated ceiling (≈ 9,
  plus grants)"), so the figure now has no record behind it at all — the same edit makes the
  residual easier to close, not harder.
- [minor][re-raised] Assumption register A25 + Risks row "12-minute stage timeout is too
  tight for a light `implement` (A25)" + Rollout 6 — "record sessions and wall clock with the
  ADR's counting rule … The same data settles A23/A25" — the recording rule named cannot
  produce A25's own datum. The ADR's counting rule is run-level (sessions = entries in
  `results`, wall clock = the sum of `durationMs`), while A25's basis states the missing
  datum is per-stage ("per-stage durations were never aggregated"); a run total of 8 sessions
  in ≤ 96 min is fully compatible with one stage hitting the 12-minute knob, and a timed-out
  stage surfaces as an escalation record, not as a duration inside that total. Compounding
  it: `fix` carries the same 12-min knob and is named by neither A25 nor the Risks row, yet
  2 of the 8–10 sessions in the escalated band Rollout 6 measures are fix sessions, so part
  of the measured band has an unregistered timeout adequacy. Fix: name plan/implement/fix in
  A25's wording and in that Risks row, and have Rollout 6 record per-stage durations beside
  the run totals so the A25 settlement has the datum it names. Registry wording plus one
  recorded column; no knob change.
- [nit][re-raised] Rollout 6 — "plus 1–2 sessions for each `--extend-budget` grant the
  operator buys" — the per-grant figure is the operator's argument, not a bound the design
  sets: `--extend-budget <n>` adds n to `state.extensionSessions`, consumed one per over-cap
  session (A27), so a grant of 5 buys 5. As written the escalated band's ceiling reads as
  1–2 sessions per grant, understating what an operator can buy and so what the measured band
  can show. Fix: "plus the operator's `--extend-budget <n>` grants, each worth n over-cap
  sessions (A27)" — text only.

## FYI (no severity)

- The `opts.cfg` pre-lock validation and A12's `doc-sync.md:23-25` `git diff*` exception are
  the security seat's ground; this seat notes only that the added validation path costs
  nothing measurable.
