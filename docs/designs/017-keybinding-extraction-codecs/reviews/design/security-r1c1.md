# security review r1 — dissent pass c1

verdict: approve

## Dissent pass (c1) — disposition of my r1 findings

- **r1 major (symlinked tree anchor)** — closed at the anchor: §Config-path
  resolvers now skips "a tree whose own final component is a link
  (`DirEntry::file_type()` on an enumerated `<dir>`, `symlink_metadata` on a
  template root)", §Read discipline binds the rule ("never taken as a tree
  anchor"), Test #6 and the `symlinked-prefix` fixture carry a symlinked tree
  root, and the Risks/Compatibility rows repeat it. The revised premise that
  anchors are canonical is verified against the code: `SteamCore::with_roots`
  and the library enumeration run every root and library through
  `canonical_paths` (`crates/discovery/src/steam_common.rs:79,100,151,369`).
  One narrower residual survives in the new absolute sentence — the finding
  below.
- **r1 major (unbounded `inputs`, no document-octet cap)** — closed:
  `MAX_SNAPSHOT_ENTRIES` now counts (`action`, input-token) pairs, which bounds
  `entries` and every `inputs` array at once; `MAX_SNAPSHOT_OCTETS` = 4 MiB
  caps one loaded document with an `fs::metadata` pre-read check in
  `snapshot.rs` (the catalog/A4 posture); Tests #4 and #10 pin both. The pair
  re-count of `017-B2` strengthens the ADR clause rather than diverging from it
  (entries ≤ pair total ≤ 4096), so no ledger note is owed for it.
- **r1 minor (detect-time reads outside the reader)** — closed: `detect` opens
  candidates through `read::open_capped` and `read::decode`, and "an
  over-budget or undecodable match claims and refuses by name rather than going
  unseen"; the `bom-utf16` fixture is locatable through the same path.
- **r1 nit (`user` personal-data note)** — closed: `schemas/bindings/v1/spec.md`
  states `user` is launcher-local identity a sharing surface treats as personal
  data.
- **r1 nit (probe disclosure)** — closed: §Local paths names the OS-username and
  SteamID disclosure and scopes probe output to the operator; the Risks row
  matches.

Verified, no finding: no new external dependency, so checklist item 5 (dependency
advisories) adds no exposure; `cargo deny check advisories` cited in Rollout item
9 does exist in `.github/workflows/check.yml` (the `advisory-gate` job), so the
gate list's "per check.yml" is accurate. `MAX_NATIVE_NAME_OCTETS`,
`MAX_DISCOVERY_SLUG_OCTETS` and `MAX_SLUG_OCTETS` exist in core as the bounds the
format cites. `MAX_LOCATE_ENTRIES_PER_TREE` bounds the walk the roots-union
widened, and the aggregate-cost sentence was corrected for the second tree kind.
Fail-closed posture (named refusals, never an empty snapshot, never-write,
pre-allocation length caps) is intact through the revision; the exit-code
re-spacing (statuses 3/4/5 above clap's 2) is consistently reflected in the
CLI section, the Compatibility matrix and Test #13.

## Findings

- [minor][re-raised] §Config-path resolvers — "Candidate paths are never canonicalized: the anchors (`roots`, `library`) are already canonical from the scan and no link is followed at an anchor or below one, so a walk never leaves the tree it started in." — the named checks establish only that the tree root's **final component** is not a link (`DirEntry::file_type()` on the enumerated `<dir>`, `symlink_metadata` on a template root); the joined template path can still carry a link at an **intermediate** component below the canonical anchor — `userdata` for `SteamUserdata`, `pfx`/`drive_c`/`users` for `ProtonPrefix` — which the OS follows during the join, anchoring the walk in an outside tree while the sentence claims that is impossible. A17 says prefix trees are "held together by symlinks", so an intermediate link is the documented real-world shape, and nothing tests it (Test #6 pins `.ini`, directory and tree-root links only). This is narrower than the r1 major it survives — the r1 fix was implemented verbatim and the anchor case is closed; exploiting the residual needs write access to the launcher's own trees, and yields a grammar-filtered read of `.ini` files outside the context's trees with a bounded echo through `UnparsableEntry`/snapshot tokens. Fix: one clause extending the existing check up the template chain — every component between the anchor and the tree root is `symlink_metadata`-checked, and a link skips-and-counts the tree exactly as a linked final component does (fixed template paths, ≤ 6 components, no unbounded work) — plus one `#[cfg(unix)]` link at a template-intermediate component in the `symlinked-prefix` fixture, asserted in Test #6.

## FYI (no severity)

- §Determinism's new cross-document dedup keys on (`user`, `provenance.files`),
  and `files` are tree-relative: two genuinely different `ProtonPrefix` trees
  under different libraries share `user` = `steamuser` and plausibly share
  identical relative paths, so differing content could be stated once instead of
  twice. Correctness/complexity question for another seat, not a security one.
