# compatibility review r1 (dissent pass c1)

verdict: approve

Scope: the blob diff `17aee59..b08752d` of `02-proposal.md`, then the full current
document. Inputs: `02-proposal.md` (incl. its Revision log) and
`docs/adrs/012-input-vocabulary-and-reach-classes.md`. Compatibility checklist
applied: matrix statuses, OS input APIs, toolchain floor, native modules,
keyboard reality, file paths, test-strategy coverage of supported rows.

## Closed

- r1 major, Compatibility matrix — the row now reads
  `| Third-party reader, non-Rust | yes (data only) | This stage ships no code,
  binary, or runner contract for any language, and the corpora are plain TOML
  any parser reads; the runner contract stays Deferred |`. That is a committed
  status that adds no mechanism, and it keeps the runner-contract gap in
  Deferred / out of scope, consistent with ADR-012's Consequences ("the corpus
  is language-neutral"). No `unknown` and no empty cell remains in the matrix.
  **Closed.**

## Checklist walk of the changed text

- **Matrix (1):** every row carries a committed status
  (`yes`, `yes (data only)`, `no (v1)`, `not applicable`). Satisfied.
- **OS input APIs (2):** unchanged by this revision; the design touches no OS
  input surface — it ships table data and a pure predicate over loaded data, and
  the OS-neutral vocabulary is the stated justification. Not re-opened.
- **Toolchain floor (3):** the revision's additions (`name` grammar check before
  the join, `0..=999` checked conversion, signed-integer deltas,
  `InputGrade.class: ReachClass`, `GradeBound(u32)`) are all std-only; the
  "std-only, no new dependency, `Cargo.lock` byte-unchanged" row stays true.
- **Native modules (4):** none; nothing in the diff introduces one.
- **Keyboard reality (5):** unchanged by the diff; non-US geometry, glyph
  naming, and modifier/named-key ids stay in the risk table and Deferred.
- **File paths (6):** the revision improves this — `name` is matched against
  `[a-z][a-z0-9-]*` *before* `dir.join(format!("{name}.toml"))`, so no
  separator or `..` can reach the join, and Test strategy #4 pins that refusal
  "before any file is read". Lowercase-only grammar also supports the Windows
  case-insensitivity row.
- **Test strategy vs. supported rows (7):** one gap, reported below.
- Other closures checked for collateral damage: version-segment equality for
  every reach table adds a refusal only for data no v1 corpus contains, and
  `reach_tables.rs` "byte-unchanged" plus "`v1` refusals stay byte-exact" stay
  coherent (v1-on-v1 "passes trivially"); naming the declared version in
  `reject_unknown_entry`, with the A11 escape-and-truncate path, preserves the
  byte-exact v1 message and bounds message growth. The dropped inline mentions
  of `class-order-mismatch.toml` and `unknown-class-id.toml` are still in the
  directory listing and still exercised by Test strategy #2, so the v2 spec-rule
  claim "each with a fixture or loader check" is not left unanchored.

## Findings

- [minor][new] Compatibility matrix + Test strategy — the row
  "`| Third-party reader, non-Rust | yes (data only) | … the corpora are plain
  TOML any parser reads …`" is now a support claim, and checklist item 7 wants a
  named check for every supported row. Every other `yes` row has one (#8 names
  the Linux CI runner and the Windows contributor leg; #2 and #6 pin the v1-only
  reader's refusals byte-exact). This row's claim rests only on assertion: #6's
  walk validates the corpora with BindForge's own parser, which is *stricter*
  than TOML, so a green walk proves conformance to the BindForge subset, not
  readability by a stock parser. Fix: anchor the claim in one clause of Test
  strategy #6 (or the row's note) — e.g. "every shipped corpus file stays inside
  the documented TOML subset `schemas/README.md` states, which is what makes the
  data readable to a third-party parser" — no new test target and no parser
  dependency, so "`Cargo.lock` byte-unchanged" stays true.

## FYI (no severity)

- The new coordinate bound `coordinates 0..=999 through checked integer
  conversion` is a forward-compat decision baked into `keyboard-geometry/v1`'s
  grammar (a future geometry needing a larger grid would ride a new major). It
  is ample for real grids and it is what keeps the signed-delta arithmetic
  overflow-free, so it is not a compatibility finding — but the complexity seat
  may want the bound justified in `spec.md` rather than only in the entry table.
