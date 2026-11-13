# Compatibility review r1c4 (ratification pass)

verdict: approve

Scope as given: `git diff b08752d2 92beeaa7` on `02-proposal.md` (three hunks: the
`reach.rs` paragraph in "The model", predicate clause 4 in "Placement legality", and
the Revision log entries), then the full current document. Inputs: the proposal,
`docs/adrs/012-input-vocabulary-and-reach-classes.md`, the Revision log, my r1
findings. No other reports or transcripts opened.

## Findings

None. The three changed passages do not break anything the compatibility seat
approved in r1, and they do not falsify the sections I cited in r1.

## Checklist re-check against the changed text

1. **Matrix rows all have a status** — all eight rows carry a status; no `unknown`,
   no empty cell. The r2 relabeling in the Revision log ("the non-Rust row commits
   to `yes (data only)`" stays recorded as an r1 decision) leaves the matrix body
   untouched by this diff.
2. **OS input APIs** — unchanged by this diff, and the stage ships no capture code:
   the abstraction (glyph ids under a stated ANSI-US geometry, geometry as table
   data) is the justification the ADR's Decision Three gives. Not affected.
3. **Node version floor** — not applicable to this repo; the analogue row
   ("Toolchain floor (1.85.0, edition 2024) | yes | std-only, no new dependency,
   `Cargo.lock` byte-unchanged") holds: the changed text adds no dependency and no
   build step.
4. **Native modules** — none; `Cargo.toml`/`Cargo.lock` stay in the Unchanged list.
   Unaffected by the diff.
5. **Keyboard reality** — unchanged (glyph grammar, ANSI-US assumption stated in
   `spec.md`, non-ANSI-US data and physical-position naming deferred). The diff
   does not touch the grammar or the geometry section.
6. **File paths** — the diff improves this, not breaks it: `load_geometry`'s
   grammar-checked `name` before the join (r1) plus the r2 "bounded signed deltas"
   wording; the rewritten sentence keeps `Hand` and `hand_matches_file_name`
   untouched, so the join/`format!("{name}.toml")` rule and the Windows
   case-collision rule (matrix row 2, Test strategy #8's Windows leg) still apply
   unchanged.
7. **A platform test per claimed-supported row** — Linux row: Test strategy #8 plus
   Rollout #5. Windows row: #8's contributor leg plus Rollout #5. Toolchain row:
   the three gate commands with `--locked`. Third-party v1-only reader row: #2, #6
   and #7 (`keys-entry.toml` refusal, `supported_majors()` equals `{v1, v2}` and the
   majors on disk, "`reach_tables.rs`'s v1 expectations pass byte-unchanged").
   The rewritten `reach.rs` sentence does not weaken this row: the load-time
   version-segment equality rule it describes is anchored by A2 (verified), whose
   basis is the contract's own rule that "declared majors equal the majors present
   under `schemas/*/v*/`" (`schemas/README.md:16-26`,
   `crates/core/tests/version_gate.rs:92-105`), and Test strategy #3 pins the
   refusal ("a document whose declared version differs from its loading
   directory's segment is refused, v1 directories included"). So "every v1 document
   declares `v1`, so the check adds no corpus refusal" is a consequence of a
   verified register entry rather than a bare assertion, and ADR Consequences'
   "keeping v1 refusals byte-exact" is not contradicted.

## Against my r1 finding

The r1 minor (Third-party reader, non-Rust row — support claim without a named
check in Test strategy #6) is untouched by this diff: the row's note and #6 are
byte-identical to the text I quoted. It is neither closed nor regressed, so per
this pass's scope I do not re-report it; it stays open as recorded in r1.

## FYI (no severity)

- Placement legality clause 4 now carries "the per-move coordinate deltas — one
  common `(dx, dy)` when the moves agree — and each move's degradation". That is
  consistent with clause 2 and with Test strategy #5's "the check reports per-move
  deltas", and it changes no platform claim; the shape of `PlacementCheck` itself
  is the complexity/correctness seats' business.
- Deferred items re-read only against the ADR: non-US geometry, physical-position
  naming, ungraded mouse/gamepad inputs, and the runner contract all track the ADR's
  "Not decided here" and Decision Three; none contradicts it, so none is re-opened.
