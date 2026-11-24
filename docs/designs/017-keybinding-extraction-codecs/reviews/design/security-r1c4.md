# Security review r1c4 (ratification pass)

verdict: approve

Scope per caller: `git diff a4dda6a8 316ef46c` on `02-proposal.md`, then the full
current document, against the security seat's previous findings. Ratification
question only: did the revisions break what this seat approved, or make false a
section this seat cited? The report from an earlier attempt of this pass was not
present in the tree (`git diff --no-index` could not access it); the caller's
injected findings are this seat's record of what was open.

## Checks run on the changed text

- **Anchor-canonicalization claims (the basis this seat cited in r1).** The
  revision changes "`roots()` (the selected root candidates, in candidate
  order)" to "`roots()` is the selected set as the core holds it — canonicalized,
  existence-filtered, byte order, not the candidate list", and `GameContext.roots`
  to "core byte order". Verified against code: `SteamCore::with_roots` runs every
  root through `canonical_paths` (`crates/discovery/src/steam_common.rs:77-82`),
  which canonicalizes, drops entries that fail to canonicalize or lack a
  `steamapps` dir, and sorts/dedupes via `BTreeSet` (byte order)
  (`steam_common.rs:369-381`); libraries are likewise canonicalized
  (`steam_common.rs:100`). The changed text is *more* accurate than the text
  approved at a4dda6a, and §Config-path resolvers' dependence ("the anchors
  (`roots`, `library`) are already canonical from the scan") still holds.
- **Walk bounds after the `MAX_LOCATE_DEPTH` redefinition and the detect
  rewrite.** The walk still prunes on depth (`MAX_LOCATE_DEPTH` tree-relative
  components) and `MAX_LOCATE_ENTRIES_PER_TREE`, content-tests at most
  `MAX_CANDIDATE_FILES` grammar matches per tree through the capped reader, and
  skips-and-counts the rest; every open is length-capped before allocation and
  read-only. Malformed/over-budget input fails closed with a named refusal and a
  bounded, escaped echo. No new unbounded path, no new code-evaluation or
  write surface. Not broken.
- **Removed tail in §Read discipline** ("which turns a broken prefix (A17) or a
  whole-profile tree into a degraded `family-confirmed-files-absent` rather than
  a hang, an unbounded walk or a read outside the tree"): compression only — the
  degradation guarantee is still asserted in §Config-path resolvers, the
  Compatibility matrix NTFS/exFAT row, and the Risks table, all of which the diff
  leaves standing on the link-handling clauses. Not broken.
- **Contract additions** (`files` entry ≤ `MAX_LOCATE_DEPTH` components, loader
  refusal asserted in Test #10; `SnapshotEntryBudget` now checked against
  admitted pairs so removals free no room; fold stops at budget+1): strict
  tightenings on the untrusted-document load path and the fold's memory. Good.
- **Dedup by `UserCandidates.tree_root`**: absolute path explicitly
  "never rendered into a document"; probe-path scoping (§Local paths) unchanged
  from what this seat approved. Not broken.
- **Steam Deck row** dropped its stale "`$HOME` … layout" claim to match the
  host-config deferral — removes a false coverage claim. Good.
- **Dependencies (checklist #5):** the diff names no new external dependency and
  keeps A12's "no resolved version moves"; `cargo deny check advisories` stays a
  gate (Rollout item 9). Nothing new to advise-check.

## Findings

- [minor][re-raised] §Config-path resolvers (closing paragraph) — "Candidate
  paths are never canonicalized: the anchors (`roots`, `library`) are already
  canonical from the scan and no link is followed at an anchor or below one, so a
  walk never leaves the tree it started in." — Unchanged by this pass's diff, and
  unchanged in substance from the finding this seat carried at approval: the named
  checks (`DirEntry::file_type()` on the enumerated `<dir>`, `symlink_metadata`
  on a template root) still establish only that the tree root's *final* component
  is not a link, while the joined template path can carry a link at an
  intermediate component below the canonical anchor (`userdata` for SteamUserdata;
  `pfx`/`drive_c`/`users` for ProtonPrefix — A17 records prefix trees as "held
  together by symlinks"), which the OS follows during the join, and Test #6 pins
  only `.ini`, directory and tree-root links. This revision neither addressed
  nor worsened it — recorded for the run's ledger, not a regression, and as a
  minor it does not gate ratification. Fix (unchanged): one clause extending the
  existing link check up the fixed template chain (every component between the
  canonical anchor and the tree root `symlink_metadata`-checked; a link at any of
  them skips-and-counts the tree as a linked final component already does —
  templates are ≤ 6 components, no unbounded work), plus one `#[cfg(unix)]`
  template-intermediate link in the `symlinked-prefix` fixture asserted in
  Test #6.

## FYI (no severity)

- §Read discipline (aggregate sentence, changed by this diff): "since §The first
  family caps a tree's reads at `MAX_CANDIDATE_FILES` the product also ceilings
  the *opens*" — after the detect rewrite, a tree's grammar matches are opened
  once for the content test (≤ 32) and `extract` then reads the located files
  again "through `read.rs`" (≤ 32) with no stated byte reuse, so opens may be
  2× the stated `17 × 32` product (worst-case bytes 1088 MiB, not 544 MiB).
  Every individual read is still pre-capped and finite, so this is a budget-word
  precision issue for the performance seat, not a fail-open.
- §`bindings/v1` Bounds (clause touched by the diff): the new `files` bound states
  a component-count cap but no stated refusal of a `..` component in a loaded
  snapshot's `files` entry. Inert in v1 (nothing opens those paths — no apply
  face), so no security stake this run; the correctness seat may want the
  contract clause spelled before a consumer ever trusts the field.
