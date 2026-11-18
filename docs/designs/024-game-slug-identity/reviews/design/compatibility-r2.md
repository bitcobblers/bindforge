# Compatibility review r2

verdict: approve

Scope: the editorial round-2 diff (`reviews/design/editorial-r2.diff`), the
single language pass of the round, as a confirmation pass. Inputs: the diff,
`02-proposal.md`, `docs/adrs/024-game-slug-identity.md`, the Revision log, and
this seat's previous findings (none — the editorial phase is one language pass
per round). No review reports or transcripts opened; no shell was available to
this session (`Permission denied: shell` for every command, including `pwd`), so
the code claims touched by changed hunks were verified with read-only `read` /
`grep` on repository sources instead of the cargo and npm legs.

## Diff-by-hunk compatibility read

Every hunk is wording-only; none moves a compatibility-relevant claim:

- Context / Goals / Identity shape / Specs / Rollout / Deferred: sentence
  splits, article and comma insertions, `` `024-L1` `` / `` `024-L3` `` code
  spans, "Resolver" → "A resolver", "install-dir" → "install-directory". No
  status, bound, grammar or dependency claim changes.
- Shared grammar and bound: "Delete the private `MAX_SLUG_OCTETS` in
  `policy.rs`, `manifest.rs` and `document.rs` and the private
  `MAX_SEGMENT_OCTETS` in `action.rs`" restructured into per-file clauses —
  still exactly the four private copies, which `grep` confirms:
  `crates/core/src/policy.rs:37`, `manifest.rs:63`, `document.rs:220`
  (`MAX_SLUG_OCTETS = 63`) and `action.rs:38` (`MAX_SEGMENT_OCTETS = 63`), plus
  `action.rs:44` `SEGMENT_SHAPE_RULE` "expected 1-63 octets …" as the
  interpolated-message site the bullet names.
- Options, fold table: "nothing is transliterated multi-character" → "nothing
  maps to more than one character". Equivalent statement; the non-Latin
  behaviour (`ß`/`æ`/`ø` → `-`, non-Latin-only → `None`) is unchanged, so the
  keyboard-reality position is unchanged.
- Collision suffix: "the same named-refusal path as an un-derivable name" →
  "… used for an un-derivable name". Same refusal path, same bound re-check.
- Emission: "checks the final value with" → "against"; the `skipped`-counter
  clause now reads "fires only when a plugin emitted nothing at all
  (`steam_common.rs:101`)" — `steam_common.rs:101` is
  `if !self.roots.is_empty() && games.is_empty() && skipped > 0`, so the
  reworded claim is accurate, and the crate path `bindforge_core::is_slug_name`
  matches the package name `bindforge-core` (`crates/core/Cargo.toml:2`).

## Checklist

1. Compatibility matrix — pass, and untouched by this diff: all six rows carry
   a definite status (`Yes`, `Compile-verified, with a named gap`, `Yes, no CI
   leg`, `Yes, with a known behaviour change`, `Not affected`, `Yes`); no
   `unknown`, no empty cell. The matrix revision-log entry records the
   deliberate choice to commit to definite statuses rather than leave
   verification vague, and the reasoning is sound — no rebuttal to re-open.
2. OS input APIs — not applicable: the design ships no input capture; the only
   OS-facing surface added is the existing `cfg(windows)` `winreg` seam,
   unchanged (`crates/discovery/Cargo.toml:6-13` confirms deps are `serde` plus
   `winreg` under `cfg(windows)`, so the one new edge is in-workspace).
3. Node version floor — pass: the diff adds no Node code; the
   `tools/orchestration` row keeps its "Not affected / `npm ci && npm run check`
   unchanged" status, and the distinct `nnn-slug` run-identity grammar stays out
   of scope.
4. Native modules — pass: no new native or third-party dependency; `std`-only
   derivation, one workspace edge (`discovery → bindforge-core`).
5. Keyboard reality — pass: rewording preserves the fold boundary (Latin-1
   Supplement + Latin Extended-A single-base-letter), the explicit
   no-multi-character rule, and `-` for everything unmapped.
6. File paths — pass: unchanged; the grammar-before-filesystem discipline
   (`/`, `.`, `..` refused) is untouched.
7. Test strategy vs matrix platforms — pass: unchanged by the diff; #11 states
   the Linux-only suite, the Windows `cargo check --target
   x86_64-pc-windows-msvc` compile leg, and no macOS CI leg, which is exactly
   what the matrix's Windows and macOS rows now claim. No row claims a
   verification level the test strategy does not back.

## Findings

None. The editorial pass introduced no new compatibility exposure and broke no
section this seat approved.

## FYI (no severity)

- Test strategy #1's diacritic line and the `COLUMNS` line in "CLI boundary and
  output" run long relative to the surrounding prose; line-wrapping is the
  editorial seat's, not this one's.
