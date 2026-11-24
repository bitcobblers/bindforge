# compatibility review r2

verdict: approve

Scope: this is the single editorial confirmation pass of round 2, and my scope is
`docs/designs/017-keybinding-extraction-codecs/reviews/design/editorial-r2.diff`
— five one-line hunks, in §Codec family id, §Read discipline's aggregate
sentence, Test strategy items 3, 6 and 16. Ratification rule applied: a finding
only where the changed text breaks something previously approved or makes a
cited section false. Inputs used: `02-proposal.md`, `docs/adrs/017-keybinding-extraction-codecs.md`,
the Revision log, and the repository's read-only inspection surface. No prior
compatibility findings of mine were supplied, and the Revision log records no
rejected compatibility finding that these hunks reopen.

All five hunks are language-only; none adds, removes or weakens a platform
claim, a bound, an OS/API target, a dependency or a test assertion:

1. §Codec family id — "is `[a-z][a-z0-9-]*` **of** at most
   `bindforge_core::MAX_SLUG_OCTETS` octets". Grammar and bound unchanged.
   Re-verified against `schemas/manifests/v1/spec.md:56` ("the codec family id
   (slug grammar, at most the same shared `MAX_SLUG_OCTETS` bound)") and
   `crates/core/src/placement.rs:83` (`MAX_SLUG_OCTETS = 128`), which is A6; the
   sentence's `013-B4`-is-a-slip rider still matches `spec.md:121`
   (`MAX_NATIVE_NAME_OCTETS = 128` — `013-B4`). No compatibility delta.
2. §Read discipline — "the product also **caps** the *opens*" replacing
   "ceilings the *opens*". Same claim, plainer verb; the arithmetic still holds
   against the budget table (`MAX_USER_DIRS_PER_GAME = 16`, `MAX_CANDIDATE_FILES
   = 32`, `MAX_BINDING_FILE_OCTETS = 1_048_576` → `17 × 32` files, 544 MiB).
3. Test strategy #3 — "two fixture roots holding **9- and 8-digit** directories"
   replacing "9 and 8 digit". Same coverage; `9 + 8 = 17` against the cap of 16
   keeps the "17th being past the cap" assertion arithmetically true, and the
   enumeration-order clause still matches §Config-path resolvers'
   "keeping at most `MAX_USER_DIRS_PER_GAME` such trees in that enumeration order
   across all roots".
4. Test strategy #6 — "a platform directory name other than **those the fixtures
   use**" replacing the garbled "other than the fixtures' others". The
   previously unreadable assertion is now the one A15 needs: the locate walk must
   find a platform directory name it has no constant for. This strengthens
   compatibility coverage rather than narrowing it, and agrees with §The first
   family's "no platform-directory constant is assumed (A15)".
5. Test strategy #16 — "and **pins** that the only `Cargo.lock` delta is the
   `bindforge-codecs` stanza and the `discovery-cli` dependency edge" now states
   the §`crates/codecs` dependency bullet as an assertion. Checked against the
   tree: `members = ["crates/*", "apps/*"]` (`Cargo.toml:12`) so no glob change;
   `discovery` is the package name (`crates/discovery/Cargo.toml:2`); `serde` is
   a workspace dependency and `serde_json` already resolves
   (`Cargo.lock:317`); `discovery-cli` already depends on `discovery`, `serde`,
   `serde_json` (`apps/discovery-cli/Cargo.toml:7,12,13`). The pinned delta is
   reachable as written, so the pin cannot be false-by-construction.

## Checklist status

1. Compatibility matrix — every row carries a status (Yes / Yes-with-named-degradation
   / Yes-by-row / Compiles-claims-nothing / No / Not affected / Additive); no
   `unknown`, no empty cell. Not touched by the diff. Pass.
2. OS input APIs (X11 / Wayland / Raw Input / HID) — not applicable: this design
   touches no input API; it only reads files under resolved Steam roots and
   Proton prefixes, and the platform rows name the file-layout target per
   platform (Linux root-relative trees, Windows deferred to `017-R2`, macOS
   excluded by `Platform`'s two variants). Not touched by the diff.
3. Node version floor — not applicable to the diff; `tools/orchestration` is
   claimed unaffected and no Node surface appears in any hunk. (`check.yml:76`
   pins the Node leg at 22.23.5.)
4. Native modules — no native dependency is introduced; A12's "no new external
   package and no resolved version moves" is what hunk 5 turns into a pinned
   test, and that claim checks out above. Pass.
5. Keyboard reality (non-QWERTY, dead keys, IME, non-ASCII binding names) — the
   design's posture is verbatim native tokens bounded by the manifest's
   native-name rules (`crates/core/src/manifest.rs:336-359`: octet bound, no
   ASCII control character, no leading/trailing space), which admits non-ASCII
   tokens rather than refusing them; hunk 1 concerns family ids, whose ASCII slug
   grammar is already the manifest's `codec` grammar. Pass, unchanged.
6. File paths — separators/case are handled through `Path::components`
   component-wise matching and tree-relative byte-ordered `files`, not string
   concatenation; per-OS config-dir conventions (`%LOCALAPPDATA%`,
   `Documents\My Games`, macOS root, XDG host config home) are named and
   deferred to `017-R2`/`017-R4`, which does not contradict ADR-017's Decision
   (its `017-R2` reservation is exactly Windows launcher-context resolution).
   Not touched by the diff.
7. Test strategy names a platform test per supported platform row — Linux is the
   full leg (item 17), Proton by `proton-prefix`, the NTFS/exFAT degradation by
   item 6's link assertions, Flatpak/Snap by the two-root `user_trees` case
   item 3 restates; the Steam Deck row declares itself unexercised rather than
   claiming coverage. Items 3, 6 and 17 were edited or adjacent to edits and
   still satisfy this. Pass.

## Findings

(none — no blocker, major, minor or nit attributable to this round's diff.)

## FYI (no severity)

- §Read discipline's aggregate sentence counts *distinct files opened*
  (`detect` opens up to `MAX_CANDIDATE_FILES` per tree for its content test and
  `extract` reads the located ones again), so the octet figure is a per-open cap
  bound rather than a total-bytes-read figure — worth the performance seat's
  eye; pre-existing text, outside this pass's diff.
- `AGENTS.md` quotes the Node gate as 22.23.4 while `.github/workflows/check.yml:76`
  runs 22.23.5 — a repo-docs drift, not a proposal claim; the proposal names no
  Node version.
