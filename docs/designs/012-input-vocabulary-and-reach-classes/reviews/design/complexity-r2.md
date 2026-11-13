# complexity review r2

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r2.diff` (5 hunks: Context,
Options considered, `input.rs`, `document.rs`, Worked examples). Inputs read:
`02-proposal.md` as revised, `docs/adrs/012-input-vocabulary-and-reach-classes.md`,
the Revision log. My seat has no findings from an earlier round — the editorial
phase is one language pass per round — so this is a confirmation pass: I checked
whether any hunk added, removed, or changed a moving part, and whether any hunk
made a previously-approved complexity judgement false.

Every hunk is wording-only. "with the existing version gate what refuses" →
"refusing", "and found no" → "and it found no", "like action ids do" → "as
action ids do", "as an echoed id" → "as an echoed id does", "not a special
case" → "not from a special case", "the ADR's Alternatives states" → "the ADR's
Alternatives section states", "US-ANSI" → "ANSI-US". No mechanism, entry,
fixture, type, or dependency is touched, and no hunk changes what a component
is responsible for.

## Findings

None.

## Checklist

1. **Reuse (new package/module/interface)** — the diff introduces none. The
   modules it touches are already in the delta inventory (`input.rs`,
   `placement.rs`, `reach.rs`, `document.rs`), and none of the reworded
   sentences moves a rule into a new place. The reworded passages still route
   through what already exists: the version gate (A2), the closed entry-set
   check and `reject_unknown_entry` (A1/A2), the existing printable-escape and
   truncate path used by action ids (A11), `quoted` for the backslash
   round-trip. Nothing to flag.
2. **Each moving part traces to a Goal** — unchanged by the diff. The two
   sentences the diff touches in `Options considered` are justification
   sentences for things already traced to Goals 1 and 3; "the existing version
   gate refusing a v2 document for a v1-only reader" still matches ADR Decision
   One, so no moving part gained or lost a Goal.
3. **A design 30–50% smaller** — the diff does not change the answer. The
   smallest credible cut remains the same one already available to the author
   and correctly deferred by the design's own structure: ship `reach-classes/v2`
   (the mapping ADR-012 actually owes) in the first slice and hold
   `keyboard-geometry/v1` plus `placement.rs` for a follow-up, since A5 makes
   the predicate a precondition for 015 rather than something 012 must ship to
   be coherent. The design defends keeping both (the Options entry on shape
   preservation, and Goals 3–4), so this stays an observation, not a finding.
4. **New dependencies** — unchanged: std-only, `Cargo.toml`/`Cargo.lock`
   untouched, no CI change (A12).
5. **Deferrable second half** — the diff neither opens nor closes a slice
   boundary; the Deferred section is untouched.
6. **Knobs with no consumer, layers with one implementation** — unchanged.
   `GradeBound` and `PlacementCheck` keep their named consumer (ADR-015, A5),
   and the one-geometry table format is the ADR's own stated consequence
   ("new table files, not new logic"), not a speculative seam. The reworded
   "not from a special case" line in Worked examples reinforces the
   data-over-branching posture rather than weakening it.
7. **Value check** — proportionate. Two new format directories and two new
   source modules are the minimum that ADR-012's Decisions One–Four name, and
   the diff adds zero complexity to that. Complexity is unchanged from the r2
   technical review: this is a language pass and it stayed one.

## FYI (no severity)

- Context now reads "an ANSI-US reference geometry" while assumption row A4
  still reads "a US-ANSI reference geometry" — the editorial hunk aligned the
  prose with the rest of the document (`Goals`, the grammar section,
  `ansi-us.toml`) but not the register row. Terminology consistency, for the
  consistency/editorial seat.
- The reworded `Options considered` sentence ("reasons the ADR's Alternatives
  section states; this design only adds that the id shape") is now the longest
  line in that paragraph relative to the file's wrap. Formatting/lint seat.
- The Revision log's r2 entry "`InputGrade.class` typed" is not visible in this
  diff — expected for an editorial-only pass, noted so the reviser is not
  chased for it.
