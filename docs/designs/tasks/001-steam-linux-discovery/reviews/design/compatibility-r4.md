# Compatibility review r4

verdict: approve

Scope: the single editorial confirmation pass of round 4 — the diff
`reviews/design/editorial-r4.diff` (6 hunks) against `02-proposal.md`, plus
`docs/tasks/T-001-steam-linux-discovery.md`, the proposal's Revision log, and
this seat's (empty) previous findings. Inputs kept to those named; no other
run record opened.

## Diff assessment

All six hunks are language-only and carry no compatibility payload:

- Context — serial comma in "`normalize_slug`, `DiscoveryEngine`, or any `schemas/` spec";
  "settles each one" → "settles each such question" (still a promise about the open
  questions, all of which the Revision log lists as decided or deferred).
- A9 — comma after "so"; the deprecation/`var_os("HOME")`/relative-`XDG_DATA_HOME`
  claims are byte-identical to the approved text.
- §1 — "say more than `new()` does" → "…would"; the `new_without_default` +
  `-D warnings` claim stands (verified: the repo gate runs clippy `-D warnings`
  on the pinned 1.85.0 toolchain, and no `new()` is added).
- §2 — "the seam tests and `--steam-root` use" → "the seam that tests and
  `--steam-root` use"; "a thin wrapper over it" → "…over `from_roots()`".
  Antecedent made explicit; the canonical path rule it depends on is untouched,
  and path handling stays in `std` (`canonicalize`, `join`), not string concatenation.
- §4 — "behaviour" → "behavior" (house-style spelling only; the accepted/rejected
  grammar subset is unchanged).
- Test strategy #10 — serial comma in the three-leg platform gate; the leg names are
  unchanged.

No changed line asserts, weakens or moves a platform claim, an OS API choice, a
dependency/native-module claim, a path-convention claim or a platform test
obligation. Nothing previously approved was broken and no section cited by this
seat was made false.

## Compatibility checklist disposition

1. **Compatibility matrix** — N/A to this diff; re-checked incidentally: all 11 rows
   carry Yes/Partial/No, no `unknown`, no empty cell. Unchanged by the diff.
2. **OS input APIs** — N/A: the plugin reads files, it names no X11/Wayland/Raw
   Input/HID API; the abstraction is justified by scoping Windows/macOS to "No"
   (ADR-022), which the diff does not touch.
3. **Node version floor** — N/A: no Node code in the diff or in the design.
4. **Native modules** — §10's "no new dependency, `Cargo.lock` byte-identical" is
   untouched; verified `Cargo.lock` holds 32 `[[package]]` entries and the design
   adds no parser, `dirs`, `walkdir` or `tempfile`. No silent native dependency.
5. **Keyboard reality** — unchanged by the diff; binding-name handling stays inside
   the existing `normalize_slug` (crates/discovery/src/slug.rs), whose migration
   concerns the proposal defers consistently with ADR-022.
6. **File paths** — the only compatibility-bearing sentences changed in §2 replaced
   pronouns with names, not mechanisms; canonicalize-then-sort-then-dedup and the
   XDG/Flatpak/Snap candidate conventions are as approved.
7. **Test strategy names a platform test per supported platform** — #10's three legs
   are unchanged; `rust-toolchain.toml` pins `channel = "1.85.0"`, matching the text
   #10 cites. Read-only gates re-run on the current tree: `cargo fmt --all -- --check`
   clean, `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo test
   --workspace --locked` green (including the two existing CLI tests the diff's §8/
   Rollout #2 story depends on).

## Findings

None. Zero blocker and zero major findings open for this seat.

## FYI (no severity)

- The diff normalizes the last three serial-comma and British-spelling outliers, so
  the document's punctuation/style is now uniform — style consistency, which is the
  editorial seat's own measure, not a compatibility concern.
