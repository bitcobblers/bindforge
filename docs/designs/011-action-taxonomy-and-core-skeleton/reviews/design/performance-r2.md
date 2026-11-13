# performance review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (round 2 editorial confirmation pass).
Inputs read: `02-proposal.md` (with its Revision log),
`docs/adrs/011-action-taxonomy-and-core-skeleton.md`, and the diff. No prior
performance findings to re-check.

## Findings

None. Every hunk in the diff is a language-only edit (Oxford commas in Goals and
Test strategy #1, `build-script generated` → `build-script-generated`, `as first
exercise` → `as the first exercise`, `gains` → `adds`, `ADR-014` → `ADR-014's`,
and the error-message clause re-punctuated to "with an offending id echoed,
escaped to printable characters"). Re-checked against the applied file: the `+`
side matches, and no performance-relevant statement changed substance.

Checklist walk, per item, against the diff:

1. Input-path latency — untouched. Nothing in the diff adds cost to a
   keystroke/event path; the only mechanism the diff touches (the error echo in
   The model) sits on the `parse_file`/load path, not the event path, and the
   r1 revision-log entry ("an offending id is echoed escaped to printable
   characters") still matches the wording.
2. Structure costs — untouched; the diff adds no loop, table or cardinality.
3. Memory growth — untouched; the registry remains one bounded file whose length
   is bounded by the shared parse budget.
4. Startup cost — untouched; `no new dependency` / `Cargo.lock` byte-unchanged
   statements are outside the diff. The `build-script-generated` hyphenation is
   in the rejected option, whose rationale ("adds a build step whose correctness
   depends on `rerun-if-changed`") is unchanged.
5. Caching — not applicable; the diff introduces no cache or invalidation rule.
6. Research baseline vs. expected numbers — the diff changes no count, bound or
   number (`40 ids`, `63 octets`, the seven-namespace counts all sit outside the
   changed lines), so no gap between the design's numbers and the baseline is
   opened or closed by this pass.

## FYI (no severity)

- Placeholders and lineage — "the `ADR-009` references in the module docs of
  `crates/core/src/lib.rs`, `document.rs`, and `chain.rs` — the ones that
  describe future model ownership — are retargeted" — the edit turns a
  restrictive clause ("that describe future model ownership") into an appositive,
  which now reads as retargeting *every* ADR-009 reference in those three files
  rather than only the ownership ones. Precision/copy concern, not performance;
  flagging for the editorial seat.
