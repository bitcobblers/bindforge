# Performance review r2 (editorial confirmation pass)

verdict: approve

## Scope and inputs

Scope: the single editorial diff `reviews/design/editorial-r2.diff` (five hunks,
all language-only), read against `02-proposal.md`,
`docs/adrs/017-keybinding-extraction-codecs.md`, and the proposal's Revision log.
No prior performance findings were carried into this pass ("none — the editorial
phase is a single language pass per round"), so the ratification bar applies:
report only what the revisions broke or made false.

The five hunks, each checked for performance semantics:

1. §Codec family id — "at most" → "of at most `bindforge_core::MAX_SLUG_OCTETS`
   octets". Grammar only; the bound is unchanged and the constant is real
   (`crates/core/src/placement.rs:83`, `MAX_SLUG_OCTETS = 128`).
2. §Read discipline — "the product also *ceilings* the *opens*" → "also *caps*
   the *opens*". Synonymous verb swap; the arithmetic it asserts is unchanged and
   self-consistent: `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES`
   = 17 × 32 = 544 opens, × `MAX_BINDING_FILE_OCTETS` (1 MiB) = 544 MiB per
   game, matching the figure the section states.
3. Test strategy #3 — "9 and 8 digit directories" → "9- and 8-digit
   directories". Hyphenation only; the cap arithmetic the sentence pins is
   unchanged (9 + 8 = 17 candidate trees against `MAX_USER_DIRS_PER_GAME = 16`,
   so the 17th is past the cap).
4. Test strategy #6 — "other than the fixtures' others" → "other than those the
   fixtures use". Clarity fix; still asserts §The first family's "No
   platform-directory constant is assumed (A15)", which is performance-neutral
   and keeps the walk's pruning law at "nothing but depth and entry count".
5. Test strategy #16 — "and that the only `Cargo.lock` delta is…" → "and *pins*
   that…". Restores the missing test verb; it strengthens the pin on "no new
   external package and no resolved version moves" (§`crates/codecs`
   dependencies), which is a dependency-hygiene claim, not a runtime cost.

No hunk touches a budget constant, a walk bound, an ordering law, an IO posture,
or a cache, and no hunk makes a section this seat cited false. Nothing previously
approved was broken.

## Checklist disposition

1. **Input-path latency budget** — not applicable in the keystroke sense: this
   run ships no event path. The only cost centre is the offline CLI face, and the
   diff does not move it.
2. **O(n²) over unbounded n** — clear. `user_trees` is capped
   (`MAX_USER_DIRS_PER_GAME + 1` trees), the locate walk is capped by
   `MAX_LOCATE_DEPTH` and `MAX_LOCATE_ENTRIES_PER_TREE`, per-tree candidate sets
   by `MAX_CANDIDATE_FILES`; the per-record repetition the aggregate sentence
   names is linear in matching records. Unchanged by this diff.
3. **Memory growth bounded** — clear: no caches ship (Non-goals: "no snapshot
   caching or persistence"), pre-allocation length caps for both config files and
   the loaded snapshot (`MAX_BINDING_FILE_OCTETS`, `MAX_SNAPSHOT_OCTETS`).
4. **Startup cost** — clear: the face performs no network leg, and file reads are
   the bounded per-game figure above.
5. **Cache invalidation** — not applicable: nothing is cached.
6. **Research baseline quoted** — not re-checked in this pass:
   `01-research.md` was not among the inputs the caller allowed for this scope.

## Findings

None. Zero blockers, zero majors, zero minors.

## Evidence I could not obtain

- The caller offered the read-only `cargo` / `rustfmt` / `npm` legs for
  verification, but this session's shell access was denied outright (two
  attempts, including one `git diff*` form), so no toolchain leg was executed.
  Every number cited above was verified by reading the proposal's own arithmetic
  and by direct greps of `crates/core/src/placement.rs` (`MAX_SLUG_OCTETS`,
  `MAX_DISCOVERY_SLUG_OCTETS`) and `crates/core/src/toml.rs`
  (`MAX_DOCUMENT_BYTES = 1024 * 1024`). Nothing in this diff is a code-behaviour claim that needs
  a leg to confirm; the constants it names exist and read as quoted.

## FYI (no severity)

- §Read discipline's aggregate sentence (adjacent to hunk 2, but untouched by it,
  and untouched by the earlier hunks the Revision log records) counts **one** open
  pass per tree, while §The first family describes `detect` opening every
  path-grammar match through `read::open_capped`/`read::decode` for the content
  test and `extract` then reading the located files again through paths carried in
  `UserCandidates.files`. On my own reading that is 2 × 544 opens and ≈1.1 GiB per
  game rather than 544 / 544 MiB. It is **not** a finding of this pass — the diff
  did not create or alter it, and the round's Revision log shows the budget
  reading was deliberately settled at R1 — so it is recorded here as an
  observation for whoever owns a budget-text pass, where the smallest fix would be
  to have the aggregate sentence name the pass count it counts rather than to add
  a constant.
