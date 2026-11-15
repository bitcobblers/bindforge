# Performance review r2

verdict: approve

Scope: the editorial-r2 diff (`reviews/design/editorial-r2.diff`), read as the
authoritative text change through `git diff HEAD~1 HEAD --
docs/designs/021-decision-unit-ledger/02-proposal.md` (identical to
`HEAD~3..HEAD` for that file — the editorial round is the last commit). Inputs:
`02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md`, the Revision log, and
my own (empty) previous performance findings. Specialty applied: performance
checklist only.

The five diff hunks are all language-only:

1. Goals — "checked both directions" → "checked **in** both directions" (grammar).
2. ¶1 — "parses no path it **did** not list" → "…**does** not list" (tense; the
   read-only/parse-only posture is unchanged).
3. ¶8 template — "never-re-specifies (021-L1)" → "the "never re-specifies" test
   (021-L1)" (clarity; matches ADR-021 `021-L1`'s wording).
4. ¶8 spec — the garbled "corrected to the pairing `013-L2`/`013-R4` state gives"
   → "corrected to the pairing that `013-L2` and `013-R4` state" (grammar; the
   seed→`ADR-019` / codec→`ADR-017` pairing, i.e. A19, is unchanged).
5. Risks — em-dash aside "— no normative sentence changes —" → parenthetical
   "(no normative sentence changes)" (cosmetic).

None of these touch any performance-relevant claim. The gate runs at run entry
and at `finalizeDone` (pipeline steps), not on a keystroke/event path — so the
per-event-cost item is not applicable. The startup/corpus bound is untouched: A23
still bounds the scan to the 28-file handle corpus read once at entry and once at
finalize; A24 still makes the join a standing `npm run check` CI leg rather than a
per-run boot cost. There is no cache to invalidate, no allocation or table rebuild
added, and no number introduced or changed, so no gap opens against the research
baseline. The `MAX_LEDGER_FINDINGS` cap on denial output is unchanged. The A19
prose correction is editorial and not on the join path. No section I previously
cited was made false (I held none), and no revision broke an approved claim.

## Findings
(none — no blocker, major, minor, or nit performance findings)

## FYI (no severity)
- Hunk 4 turns the previously garbled "state gives" sentence into grammatical
  prose whose pairing now matches A19 and the `013-L2`/`013-R4` text it cites;
  the wording/meaning match is a correctness-seat concern, not a performance one —
  noted only because it was the most substantive of the five edits.
