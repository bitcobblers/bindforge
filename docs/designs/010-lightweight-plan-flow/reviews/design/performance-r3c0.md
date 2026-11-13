# Performance review r3 — ratification c0

verdict: approve

Scope as given: `git diff c1b351d 92f71e0` over `02-proposal.md`, then the full current
document, with `docs/adrs/010-lightweight-plan-flow.md`, the Revision log, and my r3
performance findings as the only inputs. Checklist applied:
`references/performance.md` (latency/cost path, unbounded structure, bounded memory,
startup cost, cache/invalidation, baseline gap).

What the diff does to the performance surface, item by item:

- **Cost path / sessions per run (Rollout 6, Budget resolution).** The band arithmetic in
  Budget resolution is unchanged by this diff — 4–6 sessions per non-converging code round,
  the cap checked at pass entry ahead of the pass-backstop, phase-cumulative through
  `reviewSessionsUsed`, `maxRounds: 2` on the mutating `fix` loop, `maxReviews.code: 12`
  not chosen. The rewording ("starts with 4 sessions spent", "a round that died to the
  backstop at 6 spent sessions") preserves the meaning, and it still matches Test strategy
  #14 (a) 6 sessions, (b) 4 sessions, (c) 6/6. Nothing broken.
- **Baseline comparison (checklist #6).** `≤ 8 sessions, ≤ 96 min at the 12-min stage bound
  in the serial default` is arithmetically sound (2 + seats × 3 = 8; 8 × 12 min = 96) and is
  quoted against the research baseline correctly — Context's "43–96 stage sessions each"
  matches the ADR table (43 … 96), and run 003's 43 sessions / 2.2 h is the right comparator
  for the ADR's named target class (one CLI flag). No unexplained gap.
- **Startup cost (Selecting the flow, unchanged).** One `parseSource` read per entry, its
  result threaded to `state.flow` and to `sourceAdmissible` as an argument — the design
  still pays one frontmatter scan per entry rather than two, and Test strategy #17 pins
  **zero** client calls on the usage-error paths. Cost of a rejected run stays zero-session.
- **Bounded state (registry partial, A27, Deferred).** Every growth knob the design adds is
  registry data with a number: 80 plan lines, `maxReviews` `{design: 0, code: 6}`,
  `maxPassesPerRound: 3`, `maxRounds: 2`, 12-min stage timeout. The only over-cap path is
  the operator's explicit `--extend-budget`, state-side and disclosed. Memory growth
  (transcripts, ledger, checkpoint) stays bounded by those caps. No new unbounded
  structure; the shared `NNN` counter growth is a Risks row accepted by ADR-010.
- **Budget invalidation on resume.** `configFor` at run/resume/status and Test strategy #15
  asserting the *effective* numbers and seat list are untouched by the diff; the budget-drift
  row is unchanged.
- **Terminology changes (`heavier` → `full`, Goals comma, panel wording, plan-cap row).**
  Checked against the escalation strings: the plan gate prints `set flow: full and re-run
  with --force` and the light suffix reads `flow: light … promote with flow: full and re-run
  --force`, so the reworded Risks row "The escalation strings name the full flow" is true.
  No performance claim moved.

So: nothing in this diff breaks what r3 approved, and nothing it rewrote became false.
The two items below are the same minor residuals I carried in r3 — the diff touched the
sentences around them (Rollout 6's grants clause, the "The same data settles A23/A25"
clause) without reaching their arithmetic; I re-report #1 because the Revision log's
rebuttal ("states the escalated ceiling (≈ 9, plus grants)") is arithmetically unsound,
and #2 as an open residual the diff leaves in place. Neither gates.

## Findings

- [minor][re-raised] Rollout 6 — "a run that escalates legitimately sits above it, at
  **≈ 9 sessions** — plan 1, implement 1, the fix rounds, and at most the code phase's 6
  cumulative (Budget resolution)" — the band's value is still not the sum of the components
  the same sentence lists: 1 plan + 1 implement + at most 2 `fix` (`maxRounds: 2`) + 6 code
  cumulative = **10**, and the shape that escalates with no operator force — the round-entry
  backstop at 2/2 after two `fix` sessions, the code phase closing at its in-round 4 (the
  one-dissenter figure of #14(b), the section's own number) — is 1+1+2+4 = **8**, exactly
  #12's converging bound, so it does not sit above it. The Revision log records "states the
  escalated ceiling (≈ 9, plus grants)" as the decision; the rebuttal is unsound on the sum
  check, since the caps the design sets produce 8 (no force) and 10 (with the 6 cumulative),
  never 9 as a ceiling. The band is the datum Rollout 6 collects and A23/A25 settle against,
  so it is the number the operate step is graded on. The direction survives at both ends
  (8–10 sessions = 96–120 min at the 12-min bound, against run 003's 43 sessions / 2.2 h).
  Fix (text only): "a run that escalates legitimately sits at 8–10 sessions — 1 `plan` + 1
  `implement` + at most 2 `fix` + the code phase's 4 (round-entry backstop) to 6 (pass
  backstop) cumulative — 96–120 min at the 12-min bound, under run 003's 2.2 h; the low end
  ties #12's bound, so the escalated band separates from it only through the code phase's
  6-session cumulative (Budget resolution)."
- [minor][re-raised] Assumption register A25 + Risks row "12-minute stage timeout is too
  tight for a light `implement` (A25)" + Rollout 6 — "record sessions and wall clock with
  the ADR's counting rule … The same data settles A23/A25" — the diff reworded this clause
  and left the settlement claim naming data that cannot produce A25's own datum: the ADR's
  counting rule yields run totals (sessions = entries in `results`, wall clock = the sum of
  `durationMs`), while A25's basis states the missing datum is per-stage ("per-stage
  durations were never aggregated"), and a run total of 8 sessions in ≤ 96 min is compatible
  with one stage hitting the 12-minute knob — a timed-out stage surfaces as an escalation
  record, not as a duration in the total. `fix` carries the same 12-min knob and is named by
  neither A25 nor the Risks row, yet 2 of the 8–10 sessions in the escalated band Rollout 6
  measures are `fix` sessions. Fix: name `plan`/`implement`/`fix` in A25's wording and in
  that Risks row, and have Rollout 6 record per-stage durations beside the run totals —
  registry wording plus one recorded column, no knob change.
- [nit][new] Rollout 6 — "plus 1–2 sessions for each `--extend-budget` grant the operator
  buys" (the clause this diff reworded) — the per-grant figure is the operator's argument,
  not a bound the design sets: `--extend-budget <n>` adds `n` to `state.extensionSessions`,
  consumed one per over-cap session, so a grant of 5 buys 5 (A27). As written the band's
  ceiling reads as 1–2 per grant, which understates what the operator can buy. Fix: "plus
  the operator's `--extend-budget <n>` grants, each worth `n` over-cap sessions (A27)" —
  text only.

## FYI (no severity)

- A12's added clause "no document-stage primary agent reads, writes or shells" is cited to
  `designer.md:4-22` and `researcher.md:4-22` (both verified: transcripts read/edit denied,
  `shell: "*"` denied), and the `planner.md` copy is verbatim from `designer.md` — the r2
  inversion is closed. The universal quantifier over-reads the basis, though: `doc-sync` is
  a document-stage primary agent whose block denies `shell: "*"` and then **allows**
  `shell: "git diff*"`, so one document-stage agent does shell, narrowly. Security/correctness
  seat's phrasing call, not a performance matter.
- Budget resolution's new "A violation throws as a usage error at the config-read seam, as a
  bad `flow` value does" compares the error class, not the position: the bad-`flow` throw is
  placed pre-lock (Selecting the flow, #17 — no lock, no design directory, no session), while
  `validated` runs at the config read. If a caller-data violation lands after the lock, a bad
  `reviewSeats` costs a lock, a design directory and an entry checkpoint per attempt, which
  #17's zero-client-call assertion does not cover. Correctness seat's call.
