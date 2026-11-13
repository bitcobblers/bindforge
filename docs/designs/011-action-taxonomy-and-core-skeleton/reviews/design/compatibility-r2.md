# Compatibility review r2

verdict: approve

## Findings

None. This is the round-2 editorial confirmation pass; scope is
`reviews/design/editorial-r2.diff` (six hunks). All six are language-only —
a possessive ("ADR-014's profiles"), two serial commas, a hyphenated
compound ("build-script-generated table"), a comma-clarity rewrite of the
error-message echo sentence, and "gains" → "adds" with an em-dash appositive
around the ADR-009 retargeting clause. Each post-image was checked against
the current `02-proposal.md` and matches. Nothing in the changed text breaks
a compatibility position or falsifies a matrix row or test-strategy entry:

1. **Matrix** — untouched by the diff; re-checked: every row carries a
   status, no `unknown`, no empty cell.
2. **OS input APIs** — the stage targets none (no binary, no device I/O);
   input naming stays deferred to ADR-012, consistent with ADR-011's "Not
   decided here" clause. The diff adds no input-facing text.
3. **Version floor** — the `crates/core` goal hunk adds only a comma; the
   std-only / no-new-dependency / floor-1.85.0 posture is unchanged.
4. **Native modules** — none introduced; the error-echo hunk keeps the
   printable-character escaping so a non-ASCII id in a rejection message
   stays safe on any platform.
5. **Keyboard reality** — the Test strategy #1 hunk adds only the serial
   comma; the accept/reject case list is intact, including the non-ASCII
   rejection case that pairs with the `non-ascii-id.toml` fixture.
6. **File paths** — `parse_file(dir.join("registry.toml"))` untouched; the
   lineage hunk is docs-only and matches ADR-011's supersession of 009.
7. **Platform tests** — the matrix's two `yes` platform rows (Linux CI,
   Windows contributor) are still both named in Test strategy #7; the diff
   does not touch #7.

Deferred items re-checked only against the ADR per the pass rules: none of
the hunks contradicts ADR-011's Decision or Consequences.

## FYI (no severity)
- Lineage hunk — "the ones that describe future model ownership" moves from
  a restrictive `that`-clause to an em-dash appositive; the restrictor
  survives, so meaning is preserved — a language-seat confirmation, not a
  compatibility concern.
