# Code review — compatibility seat — ratification pass c2

Scope: `git diff 4844669..HEAD -- ':(exclude)docs/designs'` (fix stage `f299913`,
test-update stage `aaf0796`, plus docs-only `3852b63`), read against the current
`02-proposal.md` and `docs/adrs/014-profiles-as-chains.md`. This pass checks only
that the post-approval revisions did not break what r1 approved. Verdict:
**approve** — no blocker or major findings; three minors from r1 remain open,
unchanged by these revisions.

## Checklist walk (compatibility)

1. **Platform code vs. the Compatibility matrix.** The revisions add no
   platform-specific code. The matrix rows stay as r1 read them; the three open
   minors (below) sit under the Windows and third-party-reader rows exactly as
   before — the revisions neither fixed them nor made them worse.
2. **Node floor.** N/A (Rust crate; std-only row unchanged, `Cargo.lock` untouched
   in this diff).
3. **Paths / case sensitivity.** Chain paths are still `Path::join` builds
   (chain_ops.rs:144); the case-sensitivity divergence is the carried minor #1.
4. **Guarded platform imports.** None added.
5. **Locale/layout reality.** Unchanged by the revisions; carried minor #2.
6. **Native dependencies.** None introduced.
7. **Test platforms.** Ran the full core suite on this Linux runner: 17 targets
   green, including `chain_ops.rs` (74 tests), `conformance.rs`,
   `grammar_closure.rs`, `reach_v2.rs`, `chain_guard.rs`, `version_gate.rs`
   (`cargo test -p bindforge-core --locked`). The Linux CI leg of Test strategy
   #9 is real and green; the Windows leg is still contributor-side only.

## What the revisions changed, and whether it broke anything approved

- **`CoreError::Cycle` Display text.** The range start shipped
  `chain derives from itself (\`Cycle\`)`; HEAD restores
  `chain derives from itself`, byte-identical to `main`
  (`git show main:crates/core/src/error.rs:135`). This *repairs* a violation of
  the proposal's "No `CoreError` variant is removed or changed" (The model) — the
  shipped `chain_guard.rs` pins `Cycle` only by variant, and the test-update
  stage's removal of the message-needle assertion in
  `tests/chain_ops.rs:1272` is not a weakening against the design: Test strategy
  #3 pins the variant, which the case still asserts, and the added assertions
  keep the A13 one-line/escape shape. No compat break.
- **Directory-major refusal.** `chain_ops` now calls the shipped shared helper
  `check_dir_version` (document.rs:1346, same as `reach.rs:160` and
  `placement.rs:160`), answering `InvalidValue` — which is what proposal
  Cross-file loading step 1 names (A5, basis `document.rs:787-802`) and what
  "Every other v2 refusal is `InvalidValue` or `MissingEntry`" requires. The
  `UnsupportedVersion` spelling at 4844669 was the deviation; the fix removes it,
  deletes the private `check_link_version` clone, and updates spec.md rule 2 to
  the shipped message shape ("names the file, the declared major, and the
  directory"). One spelling of one fault across every loader is strictly better
  for the third-party-reader row. `UnsupportedVersion` still means
  "major the format does not ship" (version_gate.rs green). No compat break.
- **Absent head is now `Io` carrying the path, not `ChainLinkMissing`.** This
  matches proposal step 1 (`parse_file(head)`, A13 "`Io` carries the path") and
  stops echoing an unvalidated file stem as a profile id; spec.md rule 3 and the
  error.rs doc now state the head/walk distinction consistently, and the new case
  at tests/chain_ops.rs:1378 pins `source_path` and the path-free message. A
  non-Rust implementer reads one coherent rule set. No compat break.
- **`merge` drops provenance labels from its output.** The proposal's merge
  paragraph specifies inheritance of `derives` and `for` only; inheriting an
  operand's `generated-by`/`seeded-from` onto a document that operand did not
  produce conflicts with ADR Decision Four. Removal plus the pin at
  tests/chain_ops.rs:2135 aligns code with design. No compat break.
- **`emit_composition` emits a kept reciprocal pair once instead of refusing.**
  The r1 code refused every composition that preserved an exchange; the proposal
  says such overlaps are compatible and output order is deterministic (first-patch
  touch order). The new case at tests/chain_ops.rs:2060 additionally pins
  merge-equals-stacked-chain resolution over the base — the merge paragraph's
  own contract. No compat break.
- **`serialize_profile_link` writes `kind`/`for` unconditionally for v2.** The
  v1 serialize branch is untouched (A15, conformance green); every model the
  crate produces carries `kind`, the writer no longer branches on filled fields,
  and the hand-built precondition is stated in the doc. The conformance
  round-trip (`serialize` → re-parse equality) passes for the whole v2 corpus.
  No compat break.

## Open findings carried from r1 (unchanged, still minor)

1. **[minor] Case-insensitive-filesystem divergence is still unpinned and
   unwritten.** `open_ancestor` (crates/core/src/chain_ops.rs:381-384) resolves a
   constructed path through `try_exists` — on Windows `Game-Base.toml` answers a
   `game-base` lookup — and the id rule at :406 compares against the constructed
   stem, never the name the filesystem matched. spec.md:70-74 asserts
   "profile-id ↔ file-name equality must be case-unique" as a fact the loader
   enforces; it does not, in that direction, and the suite still has no
   case-only-mismatch case. The fix stage did not take the r1 suggestion
   (byte-exact `read_dir` pin, or one contract sentence stating the divergence).
2. **[minor] `generated-by` character set: spec.md:46 still narrower than the
   reference reader.** The reader refuses the default-ignorable/joiner set
   (document.rs:1083 `check_generated_by` via `is_label_char` at :1110), which
   the proposal's entry table (proposal:236) names but spec.md's line ("no ASCII
   control or bidi-control characters") does not — and spec.md is the contract
   under the third-party-reader matrix row. No fixture or grammar case carries a
   soft-hyphen/U+200B label or any valid non-ASCII label. Unchanged by this
   range.
3. **[minor] Slug grammar still admits Windows reserved device basenames.**
   `check_profile_slug` (document.rs:1060) accepts `con`, `aux`, `prn`, `nul`,
   `com1`-`com9`, `lpt1`-`lpt9` as ids, which become file names at
   chain_ops.rs:144 and fork ids at :608; such a chain is unbuildable for the
   Windows contributor the matrix marks supported. Unchanged by this range; the
   r1 disposition (fix once where the grammar lives, or one sentence under
   Deferred plus the matrix note) still stands.

## FYI (no severity)

The new error.rs header declares a shipped arm's wording part of the pinned
contract; only variant-pins and fixture-message pins exist for `Cycle` (as on
`main` pre-014). A byte-pin is optional but would make that sentence literal.
