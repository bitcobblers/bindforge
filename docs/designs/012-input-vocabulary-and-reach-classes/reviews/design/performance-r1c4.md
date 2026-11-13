# performance review r1c4

verdict: approve

Ratification pass c4. Scope: the diff `b08752d..92beeaa` of `02-proposal.md`
(two hunks: "The model" reach-load paragraph, and predicate clause 4, plus
the r2 revision-log entries), then the full current document. Inputs per the
caller: the proposal, the ADR, and the revision log only — `01-research.md`
and other reviews were not opened, so checklist item 6 (research baseline
quote) was evaluated only against numbers stated inside the proposal itself.
No performance findings in any earlier round to re-check.

## Checklist pass over the changed text

1. **Input-path latency** — clause 4 changes only the shape of the
   `PlacementCheck` result ("the per-move coordinate deltas — one common
   `(dx, dy)` when the moves agree — and each move's degradation"). The
   predicate is a pure function over already-loaded data, consumed by 015's
   transforms at policy time, not on the keystroke/event path; the per-move
   vector is sized by the caller's `moves` slice. No allocation, rebuild, or
   synchronous IO added to any event path. The reach-load revision adds one
   string comparison per reach table at load — constant, load-time only.
2. **Structure costs** — no new nested iteration: the version-segment check
   is per-document, and "the loader compares no files (the ordinality's
   source is one rule per document)" is retained, so no cross-file canonical
   list comparison. Bounded coordinates (`0..=999`) and signed-integer deltas
   keep clause 2's arithmetic bounded. No O(n²) in bindings × devices ×
   platforms introduced.
3. **Memory growth** — grades (`48` pairs per hand) and geometry (`48`
   entries) are fixed-size corpus data; `PlacementCheck` carries at most one
   delta pair and one degradation per move. Bounded.
4. **Startup cost** — the r2 revision removes no IO and adds none; the new
   sentence ("`Hand` and `hand_matches_file_name` are untouched") states the
   r2 check reuses the already-loaded corpus rather than re-reading files.
5. **Caching** — no cache or memoization is introduced or changed in either
   hunk; nothing to invalidate.
6. **Research baseline** — the changed text states no expected numbers, so
   there is no gap against the baseline to flag; the baseline document
   itself was outside this pass's named inputs (noted, not a finding at this
   gate).

The r2 revision-log entries match the landed text ("version-segment equality
at load for every reach table, no v1 corpus outcome change"; "`PlacementCheck`
carries coordinate deltas and each move's degradation") — no closure claims a
performance cost the document contradicts. Nothing in the diff contradicts
docs/adrs/012-input-vocabulary-and-reach-classes.md from a performance angle
(Decision Four's "pure function over loaded data" is preserved; the deferred
runner contract and artifact-locating questions stay with their owner
documents).

## Findings

(none)

## FYI (no severity)

- "The model" — "every v1 document declares `v1`, so the check adds no corpus
  refusal" — an empirical claim about shipped corpus contents; not verifiable
  from this pass's named inputs and not a performance concern. Correctness /
  test-strategy seat territory if they have not already pinned it via the
  A11-basis sources.
