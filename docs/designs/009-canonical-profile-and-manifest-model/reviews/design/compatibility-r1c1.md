# compatibility review r1 (dissent pass c1)

verdict: approve

Scope: the `02-proposal.md` diff `cc78a21 → 7b2ddc6` plus the full current document,
against `docs/adrs/009-canonical-profile-and-manifest-model.md` and the Revision log.

## Checklist walk

1. **Matrix statuses** — every row carries a status; the hub row now reads
   `no (v1) — out of scope`. No `unknown`, no empty cell. r0 finding closed.
2. **OS input APIs** — the design ships no input-event API (stated non-goal) and
   names no game config I/O (matrix `n/a` row), so the abstraction (a
   dependency-free data/format crate) is justified rather than assumed.
3. **Version floor** — N/A for Node; the Rust row pins 1.85.0 and adds no
   dependency (A9/A10 verified against `crates/core/Cargo.toml`, `rust-toolchain.toml`).
4. **Native modules** — none. `bindforge-core` depends on nothing,
   `bindforge-transforms` on the path crate. No silent native dependency.
5. **Keyboard reality** — v1 is US-ANSI glyph naming, non-ASCII names are
   ungrammatical, `pos:` is now explicitly v2-gated per format, dead keys are a
   manifest field, ISO/JIS and non-US layouts are rows with `no (v1)` + the v2
   requirement. One residual gap in the enumeration trail — finding 2.
6. **File paths** — no id becomes a path, and r1 extends the rule to manifest
   pins (index keyed on declared `game` + `revision`), so the r0 game-id gap is
   closed by both a grammar (`[a-z0-9-]+`, no dots) and a lookup rule. Walk is
   bounded by named public constants, entries taken by `DirEntry::file_type()`,
   cap on bytes read. Per-OS config dir conventions are absent because v1 sets
   no config home — consistent with ADR-009's "Not decided here: … where a
   user's personal policy document lives". Not re-opened.
7. **Platform test per supported row** — Test #14 names Linux and Windows; the
   Windows row's own pointer is stale — finding 1.

## Verified closures (r0 findings)

- Hub row `unknown` → `no (v1) — out of scope`, note kept. Closed.
- Round-trip split: the exemption is a hardcoded `dir.format == "reach-classes"`
  branch (`crates/core/tests/conformance.rs:63`), so r1's "all six formats
  re-serialize, `reach-classes` alone load-asserted" plus Rollout 2's
  `serialize` arms is the smaller change and keeps the harness frozen;
  `the_walk_picks_up_every_version_directory_the_contract_tree_ships`
  (`conformance.rs:322-335`) asserts a floor, not an exact set, so adding five
  format dirs needs no harness edit — Goals, Test #1, and Test #13 now agree. Closed.
- Game-id grammar added to the canonical Encoding rules (three shapes) and
  manifest resolution is an index lookup on declared `game` + `revision`;
  `data/manifests/<game>/` is declared cosmetic; Test #9 pins the traversal
  cases. Closed.
- `pos` is now namespace-gated by a v2 of every format carrying input ids, with
  `UnknownInput` for a v1 document; Deferred and both matrix rows agree. Closed.
- Fixture-name rule is stated as a stem rule with the extension separate
  (`.bindprofile` content, `.toml` fixtures), matching `is_fixture_name`
  (`crates/core/tests/common/mod.rs:352-366`). Closed (see finding 5 for the
  Windows row's wording).
- README gains the admitted keyboard-name list, and Rollout 1 carries it.
  Closed in substance; the cross-reference it created is finding 2.

## Findings

- [minor][regression] Compatibility matrix — Windows row, "exercised leg (Test
  strategy #13)" — the r1 renumbering moved the platform leg to #14; #13 is now
  "harness pickup and the version gate", which says nothing about case-insensitive
  filesystems. Checklist item 7 wants each supported row to name its platform
  test, and Rollout 7 already cites #14 correctly, so the matrix is the odd one
  out. Fix: that one cell → "exercised leg (Test strategy #14)".

- [minor][regression] Chosen design → Input vocabulary + Files created (README
  row) — "the single enumeration the graded set below and Test strategy #3
  iterate" / "the admitted keyboard-name list (the enumeration Test strategy #3
  iterates)" — Test #3 was rewritten in the same revision to iterate the
  **graded set**, which the Authoring bar defines by a different rule ("every key
  the shipped placements and the seed manifest's `default` rows reference,
  extended only by review"). So no CI check enumerates README's admitted
  keyboard-name list, and the list's role is unstated: the US-ANSI `yes (v1)`
  row's evidence trail runs through a sentence that no longer points anywhere.
  Fix: state README's list as the *admissible-name namespace* and the graded set
  as its reviewed subset, and drop "and Test strategy #3 iterate" from Input
  vocabulary plus the parenthetical in the README row (keep the narrowing the
  Revision log records); if the README list is meant to be fully graded, restore
  a clause in Test #3 that every admitted name is graded in both hands.

- [minor][regression] Adversarial posture — "New `CoreError` variants, each
  pinned by a fixture: … `DuplicateId`, `UnresolvedAncestor`, `MergeConflict`,
  `BadPin`, `IndexTooLarge`, `IndexTooDeep`" — a fixture is one document under
  `vN/fixtures/{valid,invalid}` (`schemas/README.md:23,41-42`;
  `crates/core/tests/conformance.rs:47-78`), which cannot carry these
  conditions: `IndexTooLarge`/`IndexTooDeep` are tree-shape conditions (added by
  this revision), and `MergeConflict`'s implementation and suite moved to the
  ADR-012 slice in this revision, so it has no v1 code path and no v1 pin at
  all. Rollout 1's "a valid+invalid fixture pair per format" cannot ship those
  cases as written. Fix: split the sentence — variants a single document pins are
  pinned by an `fixtures/invalid/` case; `DuplicateId`, `UnresolvedAncestor`,
  `IndexTooLarge`, `IndexTooDeep` are pinned by the multi-document / scratch-tree
  unit tests Test #1 and #9 name, and `MergeConflict` ships and is pinned in the
  ADR-012 slice, its rule staying in `patch/v1/spec.md`.

- [nit][new-in-changed-text] Chosen design → Manifests §5 (Entries) — "`default`
  = `[id, input]` (the emphasis fact, contract 1)" — contract 1 is the action
  taxonomy, which ADR-009 Decision 1 and this proposal's §1 ("the game fact is
  the action's default input, recorded as a manifest `default` row (contract 5)")
  both keep emphasis out of. Fix: cite contract 5.

- [nit][new-in-changed-text] Compatibility matrix — Windows row, "tree file
  names follow the shipped case-unique stem rule" — the shipped case-uniqueness
  check runs over `schemas/*/v*/fixtures/{valid,invalid}` only
  (`crates/core/tests/conformance.rs:277-292`); `data/profiles/` and
  `data/manifests/` names are unenforced, which is harmless exactly because ids
  never become path components. Fix: scope the wording to fixture names and say
  `data/` file names are hygiene, not load-bearing.

## FYI (no severity)

- Test #10 asserts a bad policy invocation is rejected "at parse … before
  `apply_policy` runs", i.e. in `bindforge-core`, while the registry carrying
  the arity and argument grammars is created in `crates/transforms` (Rollout 3),
  and that crate is the one that depends on core. The edge direction needs a
  home for the registry (complexity/correctness).
- The manifest index is keyed on declared `game` + `revision`, but no duplicate
  rule is stated for it: `DuplicateId` is defined over two documents declaring
  one profile `id`. Two manifests declaring one `game` + `revision` have no named
  verdict (correctness).
- `validate` checks every `default` value is a valid input id but not the
  `dead-keys` array's entries; a typo there is a game fact nothing in v1 reads
  (correctness).
- `MAX_CHAIN_DEPTH` reading is now exactly right against the shipped guard
  (`crates/core/src/chain.rs:27` is `derives.len() > MAX_CHAIN_DEPTH`): 32 links
  accepted, 33 rejected. Verified, no finding.
