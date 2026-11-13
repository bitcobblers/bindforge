# Correctness review r1 (dissent pass c1)

verdict: approve

Scope: `git diff 17aee59..b08752d` of `02-proposal.md` plus the full current
document. Inputs: the proposal, the ADR, the revision log, and my r1 findings.

## Closure check of r1 findings (checklist item 7)

- [closed for real] Test strategy #5 one-step case — `home`→`thumb` (home key
  → `keyboard:space`): rank(thumb)=1, rank(home)=0, degradation 1; legal at
  `(1)`, illegal at `(0)`. Matches the pinned severity order
  `["home","thumb","stretch","far","off-reach"]`. The case is hand-independent
  because `space` grades `thumb` on both tables.
- [closed for real] Worked examples / Test strategy #5 hand attribution —
  recomputed all three pinned cases against the shipped TOML data:
  WASD→IJKL `left`→`right` uniform `(6,0)`, degradations 0,0,0,0
  (w stretch→i stretch; a,d,s home→j,l,k home); WASD→OKL (targets O,K,L,;)
  `left`→`right` uniform `(7,0)`, degradations 0,0,0,0 (o is `stretch` for the
  right, `;` is `home`); WASD→ESDF `left`→`left` uniform `(1,0)`, degradations
  0,0,0,0. All three legal at `GradeBound(0)`; the pinned deltas match.
- [closed for real] Version-segment check — now stated for **every** reach
  table at load, v1 directories included, with "v1-on-v1 passes trivially";
  Test strategy #3 now pins "refused, v1 directories included". Consistent
  with "the existing `reach_tables.rs` suite stays green unchanged".
- [closed for real] `InputGrade.class` is now `ReachClass`, one type with the
  `grade(&InputId) -> Option<&ReachClass>` lookup.
- [closed for real] Rule 4 now says "the shape delta when the moves agree and
  the per-move deltas when they do not", matching Test strategy #5's
  "reports per-move deltas" for the non-translation case.

## Cited-source spot-check

- A6 (https://en.wikipedia.org/wiki/Touch_typing) re-fetched: live; states
  home-row keys "ASDF" left / "JKL;" right — supports the revised table note
  ("the ADR's asymmetry fact made data (A6)") and the `;`-as-right-home
  grading. A6-adjacent pins (`;` home-right/off-reach-left, `'`
  stretch-right/off-reach-left, `space` thumb-both) match the shipped arrays.

## Findings

- [minor][new] Placement legality, worked examples + Test strategy #5 —
  "Moving an action from a `home` key to `keyboard:;`, `off-reach` for the
  source hand, is illegal at bound 1" and "Unit — the predicate, each case
  naming its `(from, to)` hands" — the revision promises a hand pair per
  case but this example names none, and its verdict is hand-pair dependent:
  rule 3 grades the target by the **to** hand, and `keyboard:;` is `home`
  for the right hand, so under `left`→`right` — the very left-hander's move
  this ADR motivates — the degradation is 0 and the move is legal even at
  bound 0, contradicting the example. ("off-reach for the source hand" is
  not the term the predicate uses.) Test strategy #5's matching case ("a
  source-`home` to `off-reach` move is illegal at `GradeBound(1)`") escapes
  only because "off-reach" reads there as the to-hand grade. Fix: pin the
  example's hand pair (e.g. `left`→`left`) or say "`off-reach` for the
  placement's to hand", and apply the promised per-case hand pair to the
  remaining #5 cases or scope the promise to the three WASD cases.

## FYI (no severity)

- Geometry entry set: "coordinates `0..=999` through checked integer
  conversion so an oversized literal is a grammar refusal" — the stated range
  also excludes negative literals, but the rationale clause names only the
  oversized direction; a wording nuance for another seat if it cares.
</content>
