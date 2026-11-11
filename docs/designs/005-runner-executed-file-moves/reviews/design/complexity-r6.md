# complexity review r6

verdict: approve

## Scope

The editorial-r6 diff (reviews/design/editorial-r6.diff) applied to
02-proposal.md, confirmed against docs/adrs/005-runner-executed-file-moves.md
and the Revision log. This is the single editorial-confirmation pass; the
complexity seat carried no findings in earlier rounds, so the test applied is:
does any revision in this diff introduce or reveal a complexity defect, break
previously approved text, or make a cited section false?

## Checklist pass

1. New package/module/interface: the diff adds none. The one new module named
   in the design (`moves.ts` with `validateMoves` + `applyMoves`) is
   unchanged by this diff; `MovePairSchema` is shared across the three stage
   reply schemas rather than duplicated — reuse, not new surface.
2. Moving parts vs. Goals: the diff adds no moving part. Every hunk is
   punctuation, line-wrapping, or phrasing (comma removal, `i.e.,`, em-dash
   apposition, heading-period placement, `optional-defaulting` → `optional,
   defaulting`, `made unappliable` → `can no longer be applied`).
3. Smaller design: nothing in the diff grows or shrinks the design; the
   schema hunk (line-wrap join) was checked byte-for-byte against the
   proposal's **Reply field** section — `min(1).max(200)` endpoints,
   `.default([]).max(64)` — identical. No bound or default silently changed.
4. New dependencies: none added by the diff; the design's dependency posture
   (no new runtime dependency, one `git ls-tree`, Node floor raise only) is
   untouched.
5. Deferrable second half: unchanged; the design still ships ADR-005 items
   1-4 only, with item 5 deferred exactly as the ADR's own Deferred clause
   states.
6. Config knobs / single-implementation abstractions: unchanged; Rollout #3's
   "no config knob or feature flag" survives the edit intact.
7. Value check: complexity remains proportionate — one optional-default
   reply field, one validator module, one guarded apply call inside
   `finalizeStage`, against the stated goal of unblocking every
   file-moving ADR (run 004's livelock, A12). Nothing in this diff moves
   that balance.

## Deferred-section check (skill step 5)

The Deferred item the diff's note flags — "Directory moves as single pairs —
this ADR-005 if a real change needs them" — was checked against the ADR:
Decision items 1-4 and Alternatives scope the change to file pairs and never
require directory moves, so the deferral does not contradict the ADR's
Decision or stated requirements. Not a complexity finding.

## Findings

(none)

## FYI (no severity)

- The diff appends an unresolved meta-comment to the end of the proposal —
  "Editor notes: Deferred / out of scope — ... is ambiguous; author to
  clarify." An open editorial TODO left inside the ratified document is an
  editorial/process-hygiene concern, not a complexity one; the reviewer of
  the next pass or the author should resolve the sentence's wording (it
  plausibly means "extend this ADR-005 to directory moves if a real change
  needs them") and delete the note.
