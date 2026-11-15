# correctness review r2 (editorial confirmation pass, scope: reviews/design/editorial-r2.diff)

verdict: approve

## Scope and inputs

Scope is the r2 editorial diff only (5 hunks: Goals bullet 2, ¶1 last sentence,
¶8 template bullet, ¶8 manifests-spec bullet, Risks row 6). Inputs used:
`02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md`, `docs/adrs/013-game-manifests.md`,
`schemas/manifests/v1/spec.md` (as the basis rows the changed sentences cite),
and the Revision log. No review report or transcript was opened.

## What each hunk was checked against

- Goals — `The join, checked in both directions plus the laws 021-L3 and 021-L5
  make decidable`. Inserted `in` only; matches ADR-021 021-R2 ("checks the join
  in both directions") and ¶7's mapping (`depends-order` ← L3,
  `owner-missing`/`owner-unknown` ← L5). No meaning change.
- ¶1 — `parses no path it does not list`. Tense-only change from `it did not
  list`; agrees with the sibling `never writes`. No meaning change.
- ¶8 template bullet — `the "never re-specifies" test (021-L1)`. Verified
  against ADR-021:93–96 ("never re-specifies. The test: an entry is a unit only
  if deleting it from the spec loses no normative content"). The added noun is
  the ADR's own word, and it aligns this line with Non-goals ("No check of
  021-L1's `never re-specifies` test") and the Deferred bullet, so checklist 5
  (one thing, one name) is improved rather than broken. The note's unit list
  still covers exactly C1–C3 + L1–L5, with R1/R2 carried by the rollout items
  (¶8), consistent with A2's ten units.
- ¶8 manifests-spec bullet — `the pairing that 013-L2 and 013-R4 state — the
  seed set's (ADR-019), the codec's (ADR-017)`. Verified: `spec.md:14` does say
  "the seed set's and the codec's (ADR-016, ADR-017)" (A19's left side); 013-R4
  is `The seed preset set … Owner: ADR-019 (reserved)`; 013-L2 gives the owner
  pair `ADR-017, ADR-019` for the same two things, so the apposition after the
  dash is the pairing L2 ∪ R4 yields (013-R1 independently pins codec → ADR-017).
  `a sixth Backfill findings entry` is correct: 013 ships exactly five
  (lines 198–219), which is also what ¶6's "Backfill findings 5" cites. The
  reworded `the correction is recorded as` preserves the old `and recorded as`.
- Risks row 6 — `Both are editorial-only (no normative sentence changes)`.
  Dash-parenthesis change only; the quoted ADR precedent is verbatim
  ("mechanical, reviewable diff … recorded, not silently absorbed",
  021:142–144) and the row's "Both" still denotes the two named actions.

## Checklist disposition

1. Assumption register — the only basis rows the diff touches are A19 (re-checked
   above, supported) and the ¶1/A13 sentence around it; no status or basis was
   changed by the diff. No new `verified` claim was introduced.
2. Research URL spot-check — done for the citations in the sections the diff
   touches and their neighbours: MDN regular-expression guide (A15) live and
   supports look-behind; CommonMark citation: see FYI, pre-existing and outside
   the diff.
3. Internal contradictions — none introduced; hunk 3 removes a naming mismatch.
4. Missing edge cases — nothing in the diff changes behaviour, so no new edge
   case (empty input, absent file, collisions, concurrency) is opened.
5. Interfaces — no name or shape changed; `MAX_LEDGER_FINDINGS`, `ledgerGate`,
   the twelve codes and the `ledger-join` log name are untouched.
6. Does it decide what ADR-021 asked — unchanged by the diff; the taxonomy,
   both hook points and the corpus pass still answer 021-R2 and 021-R1.
7. Previous rounds — no r1 blocker/major was reworded by this diff; the Revision
   log still carries one current entry per decision, and no hunk contradicts a
   logged rejection (the `amends` symmetry and the ADR-022 reservation stay out).
   Deferred items were not re-opened; none contradicts ADR-021's Decision.

## Findings

None. The diff is language-only; every hunk preserves its technical claim, and
each surviving claim re-checked against the ADR, 013 and the manifests spec held.

## FYI (no severity)

- The CommonMark citation `https://spec.commonmark.org/0.31/` (Options
  considered, ¶4) returned 404 on fetch; the live page is
  `https://spec.commonmark.org/0.31.2/`. Pre-existing text, not part of this
  diff, so it is recorded for the record rather than raised as a finding in a
  diff-scoped pass.
- `013-L2`'s own parenthetical lists its two owners as "ADR-017, ADR-019" in the
  reverse order of the things it names, so ¶8's pairing depends on R4 (or
  elimination) rather than on L2's word order. Unchanged by this diff, and the
  dash apposition in the new sentence already pins the intended pairing.
