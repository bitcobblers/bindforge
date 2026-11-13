# compatibility review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`), checked against
`02-proposal.md` and `docs/adrs/012-input-vocabulary-and-reach-classes.md`. This is
the editorial confirmation pass; the compatibility seat had no open findings, so
the pass asks one question: does the diff break or make false any compatibility
claim? It does not.

## Findings

None.

Verification of each hunk against the compatibility checklist:

1. **Context hunk** — "the existing version gate what refuses" → "refusing": the
   version-gate claim stays identical to ADR-012 Decision One ("a v1 reader refuses
   a v2 document by naming the supported majors") and to the matrix row "Third-party
   v1-only reader | yes". No change in meaning.
2. **"US-ANSI" → "ANSI-US"** — terminology harmonization toward the ADR's own
   wording ("filled in for ANSI-US geometry") and the proposal's remaining usage
   ("ANSI-US set", "ANSI-US key grid", `ansi-us.toml`). This *improves* the
   geometry-assumption story the checklist's keyboard-reality item depends on: the
   reference geometry is now named one way everywhere, including in A4's Windows
   `VK_OEM_*` basis. No compat claim altered.
3. **Options hunk** — "now are rejected" → "are now rejected"; "the ADR's
   Alternatives states" → "the ADR's Alternatives section states": the citation now
   names a real ADR section (Alternatives considered), which still carries all three
   rejections (symmetric reach, pixel distance, physical-position naming). The id
   shape reserving physical-position naming stays consistent with ADR Decision Three
   and the Deferred section. No meaning change.
4. **Model hunks** — "like action ids do" → "as action ids do"; "as an echoed id" →
   "as an echoed id does": the A11 error-message behavior (printable-escape,
   truncating, path-free echoes) is untouched in intent; the error-contract claim
   ("No new `CoreError` variant") stands.
5. **Worked-examples hunk** — "not a special case" → "not from a special case": no
   platform or contract claim involved.

Checklist rows re-checked as still green on the shipped text (matrix complete with
no `unknown` and no empty cell; OS-neutral abstraction justified by A4 rather than
naming per-OS capture APIs, which this stage does not ship; std-only under the
1.85.0 floor; no native modules — `Cargo.toml`/`Cargo.lock` byte-unchanged; layout
reality handled via the stated ANSI-US geometry, the AZERTY risk row, and the
reserved physical-position extension point; the Windows row and Test strategy #8
cover the case-insensitive-filesystem platform claim). None of these were touched by
the diff.

## FYI (no severity)

- Options considered — the rewritten sentence now wraps past the document's
  apparent ~80-column prose rhythm ("...section states; this design only adds that
  the id shape"). That is a proposal-editing/formatting concern, not a compatibility
  one.
