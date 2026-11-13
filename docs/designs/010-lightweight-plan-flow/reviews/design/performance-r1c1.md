# Performance review r1 (dissent pass c1)

Scope: the diff `9ae29b5 → 24b804a` of `02-proposal.md`, then the full current document.
Inputs: the proposal, `docs/adrs/010-lightweight-plan-flow.md`, the Revision log, and my
r1 findings. No review reports or transcripts opened.

verdict: approve

## r1 finding status

- **major — light code-phase cost has no single binding stop** — **closed.** Budget
  resolution now states the cost as a band — "a pass costs one or two sessions and a round
  that does not converge costs **4–7 sessions**" — names `maxPassesPerRound: 3` as the first
  stop at 4 sessions for the single-dissenter shape, and names the pass-entry overshoot for
  the 2-seat shape. Test strategy #14 now has both shapes. The knob choice survives, as I
  argued it would. Residual: the 7 is attached to the wrong dissent shape (finding 1 below).
- **minor — no light wall-clock expectation** — **closed.** Rollout 6 now carries a derived
  bound against run 003's 43 sessions / 2.2 h. Residual: the bound's scope is the converging
  shape only, and it depends on an unregistered per-session assumption (findings 2 and 3).
- **nit — two frontmatter reads per entry** — **closed, as specified.** "parses the source
  text once, right after `deriveIdentity`", carries the result on the `deriveIdentity`
  result, and `sourceAdmissible` "takes the same result as an argument instead of
  re-reading: one frontmatter read per entry". The post-lock gate no longer costs a read,
  and the value stamped is the value that cleared the pre-lock check. No re-open.

## Findings

- [minor][new] Budget resolution / Test strategy #14(a) — "both seats dissenting admits a
  2-seat pass at 5 spent and closes the phase at 7" / "(a) both seats dissent every pass —
  escalation on the session-budget record at 6 spent (not 20), the phase closing at 7" —
  the shape named cannot reach 7 under the design's own cap semantics. With the cap checked
  at pass entry against sessions spent (A16), a pure both-seats-dissent round is a run of
  2-seat passes: 2, 4, 6 spent, so the last pass is admitted at 4 spent and the phase closes
  **at 6** — escalation and closure land on the same number. 5 spent only occurs when a
  1-seat pass lands in between, i.e. the mixed dissent/ratification shape (`c0` 2 + 2-seat
  dissent 2 + 1-seat ratify 1 = 5, then an admitted 2-seat pass closes at 7). As written,
  #14(a) pins a test to a session count the described shape does not produce, and the
  cap-binding shape is attributed to the wrong dissent pattern. Fix: in Budget resolution
  attach 7 to the mixed shape — "a pass entered below the cap overshoots it: the
  dissent/ratification alternation reaches 5 spent and closes at 7; the pure both-dissent
  shape closes at 6 on the session-budget record" — and in #14(a) assert closure at 6, with
  the 7 overshoot pinned as its own alternating-dissent case so the pass-entry rule stays
  covered. No knob change, no mechanism added.
- [minor][new] Rollout 6 — "check them against the derived bound — ≤ 8 sessions, ≤ 96 min at
  the 12-min stage bound in the serial default, against run 003's 43 sessions / 2.2 h" — the
  stated bound is the converging shape's, not the worst case the same document derives. 8
  sessions is Test strategy #12's `2 + seats × 3` (plan + implement + 6 code sessions). The
  escalated shapes the design costs out in Budget resolution add the mutating `fix` rounds:
  1 plan + 1 implement + 2 `fix` (`maxRounds: 2`) + 6–7 code sessions = **10–11 sessions**,
  i.e. 120–132 min at the 12-min bound — the top of that band is run 003's 2.2 h exactly, so
  at the serial default a light run that fights is at parity with the baseline the design
  claims to beat. The acceptance number is fine for the runs Rollout 6 targets; it is stated
  as *the* derived bound, so the first light runs have no expectation for the escalation path
  and the design's headline win goes unqualified. Fix: name the shape — "≤ 8 sessions / ≤ 96
  min for a run that converges (Test strategy #12's bound); an escalated round reaches 10–11
  sessions ≈ 120–132 min at the same bound, at parity with run 003's 2.2 h" — so the recorded
  data has a target for both shapes. Text only.
- [minor][new] Rollout 6 + A25 + Risks — "≤ 96 min at the 12-min stage bound" against
  A25's "A 12-minute `stageTimeoutMs` is enough for the light flow's `plan`/`implement`
  stages" and the Risks row "12-minute stage timeout is too tight for a light `implement`" —
  the new wall-clock bound multiplies 12 min by *every* session, and the majority of the
  light flow's sessions are code-review seats (6 of the ≤ 8 in #12's shape), while A25 and
  the Risks row register the timeout's adequacy for `plan`/`implement` only. The acceptance
  target therefore rests on a per-session duration assumption for the flow's dominant session
  class that no assumption row and no risk carries, and A25 concedes durations were never
  aggregated — so the number Rollout 6 settles is the number nothing in the design bounds.
  Fix: widen A25's wording and that Risks row to name the code-review seats as the sessions
  the 12-min bound covers, and have Rollout 6 record per-session durations for review passes
  beside the run totals. Registry wording plus one recorded column; no mechanism.

## Checklist coverage

1. Event/keystroke path — n/a: the flow adds no interactive path; nothing added per event.
2. Structure costs — seat lists and stage lists are fixed-width per flow; no unbounded
   product introduced. No O(n²).
3. Memory growth — no new cache or table; `validated` and `seatsFor` are pure per call.
4. Startup cost — closed: one frontmatter read per entry, threaded to `state.flow` and
   `sourceAdmissible`; the plan size meter is the JS count at `runner.ts:179-180`, no shell-out.
5. Caching / invalidation — n/a: no cache introduced; the flow is read from the checkpoint
   through the single `flowOf` seam, so there is no derived copy to invalidate.
6. Research baseline — quoted in Context (43–96 sessions) and Rollout 6 (43 sessions /
   2.2 h); the gaps between the design's numbers and that baseline are findings 1–3 above.

## FYI (no severity)

- The Deferred entry "A files-spanned ceiling on the plan (`planMaxFiles`) … the ADR fixes
  two deterministic stops" is consistent with ADR-010 §When a light run is too big, whose two
  stops are the 80-line plan gate and budget exhaustion — the file cap was this design's own
  r1 addition. Not a contradiction; not re-opened.
- Context says "the six completed runs spent 43–96 stage sessions each"; ADR-010's table says
  "The seven runs on disk" and lists six rows (43–96 sessions, 2.2–7.1 h). The range matches
  the table; the count and the wall-clock range are prose-accuracy questions for another seat.
- The Windows/macOS row and Test strategy #21 flipping from `unknown` to `no` removes a shell
  dependency from the size meter's justification — cost-neutral, and it makes the 96-min
  bound's serial-lane assumption explicit, which is what findings 2 and 3 attach to.
