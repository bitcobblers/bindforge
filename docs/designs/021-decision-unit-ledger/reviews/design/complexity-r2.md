# complexity review r2

verdict: approve

Scope: the diff `reviews/design/editorial-r2.diff` (the round-2 editorial
confirmation pass), with `02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md`
and the Revision log as inputs. Previous complexity findings: none (editorial
phase is a single language pass per round).

## Findings

None.

All five hunks are language-only and preserve meaning against both the
pre-edit text and the source ADR:

- Goals — "checked **in** both directions": grammar only; the check set
  (¶7, `checkLedgers`) is untouched.
- ¶1 — "parses no path it does **not** list": tense fix; the invariant (read
  only listed paths, never write) is unchanged, no new surface.
- ¶8 template note — `the "never re-specifies" test (021-L1)`: now matches
  021-L1's own wording ("never re-specifies. The test: …") and the Deferred
  bullet; no new claim.
- ¶8 spec.md bullet — the repair of the broken "state gives" sentence keeps
  the same content (seed set `(ADR-019)`, codec `(ADR-017)`, A19) and the
  same consequence: the correction is recorded as a sixth `Backfill findings`
  entry, matching ADR-021's "recorded, not silently absorbed" clause. The
  correction rides the conformance commit already scoped in ¶8 — no added
  mechanism.
- Risks row — em-dash parenthetical to parentheses: punctuation only.

## Checklist

1. Reuse: n/a — diff adds no package/module/interface.
2. Moving parts → Goals: unchanged by the diff; no new part.
3. Smaller design: unchanged — nothing added to remove.
4. New dependencies: none introduced.
5. Deferrable second half: unchanged; Deferred section untouched.
6. Knobs / one-implementation abstractions: none added.
7. Value check: the diff makes zero complexity changes; the design's cost
   stays proportionate to ADR-021's stated goal of a mechanically checkable
   join.

Evidence note: the shell check legs (cargo/rustfmt/npm) were permission-denied
in this session, so no code-level re-verification was run. That creates no
gap for this pass: the diff asserts nothing new about the code — every hunk
edits prose grammar only — so no required property is left unestablished.

## FYI (no severity)

- Revision log carries no r2 editorial entry; whether language-pass edits
  belong in the log is an editorial-process question, not a complexity one.
