# performance review r6

verdict: approve

Scope: `reviews/design/editorial-r6.diff` (single editorial confirmation pass), read
against `02-proposal.md` and `docs/adrs/005-runner-executed-file-moves.md` as the
caller named. Inputs per instruction were the proposal, the ADR, the Revision log,
the diff, and my previous findings (none — the editorial phase is a language pass).
`01-research.md` was not opened, per the caller's input restriction; checklist item
6 was evaluated against the baseline numbers the proposal itself quotes (A12).

## Checklist walk-through (diff scope)

1. Input-path latency: no keystroke/event path is involved; the only runner work
   added is `validateMoves` + `applyMoves` at reply-parse and finalize time. The
   diff does not touch this. The bounds sentence — "The bounds cap the validation
   and apply work one reply can make the runner spend" — and the bounds themselves
   (`max(200)` per endpoint, `.max(64)` per list) are byte-identical after the
   schema line re-flow; only the line wrap changed.
2. Structure costs: rule 6's disjointness comparisons are the only superlinear
   step, O(n²) over a list capped at 64 — bounded, unchanged by the diff.
3. Memory growth: the 64-pair cap and the additive `moves: []` default are
   untouched; hunk 10 ("optional, defaulting to `[]`") is a comma change that
   preserves the A13 additive-checkpoint claim.
4. Startup cost: unchanged; the diff adds no boot-time work. The one new git call
   (`git ls-tree`, rule 4) per `from` is already capped by the pair limit.
5. Caching: no cache is introduced or touched.
6. Baseline gap: the design's 64-pair cap sits above the quoted run-004 baseline
   (23 file pairs, A12) with no unexplained expected-numbers gap; the diff does not
   alter any number.

Wording hunks checked for semantic drift that could false a performance claim:
"a pair that can no longer be applied" (#11) preserves the TOCTOU-window test
meaning; "as a rename error" preserves the guard-refusal outcome class; the em-dash
parenthetical in "Wiring into retry" preserves that both returned issues and thrown
rule failures take the existing retry path — no new validation or IO per failure.
No revision reopens a Deferred item against the ADR.

## Findings

None. Every hunk is punctuation, line-wrapping, or heading style; no numeric bound,
complexity claim, IO count, or checkpoint-default statement changed.

## FYI (no severity)

- The diff appends an "Editor notes" block after the Revision log flagging the
  Deferred line "Directory moves as single pairs — this ADR-005 if a real change
  needs them" as ambiguous. The deferral itself does not contradict ADR-005 (its
  Decision only requires tracked-path `from`/non-existent `to`; directory moves are
  not required), so it is not a performance finding; the ambiguity is for the
  editorial/author track, not this seat.
