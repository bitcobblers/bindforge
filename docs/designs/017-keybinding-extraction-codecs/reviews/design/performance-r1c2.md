# performance review r1 (dissent pass c2)

verdict: approve

Scope as directed: `git diff a4dda6a e068cf6` on `02-proposal.md`, then the full
current document. Re-checked the four performance findings of the first pass
against the changed text, plus only what the changed text makes true. Inputs
used: the proposal, `docs/adrs/017-keybinding-extraction-codecs.md`, the
Revision log, and read-only inspection of `crates/discovery/src/steam_common.rs`
(the `canonical_paths` rule at 363-380: canonicalize → require `steamapps` →
`BTreeSet` → byte order, which is what §Discovery plumbing's new `roots()`
sentence claims) and `schemas/README.md`'s budget family (the
`MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` shape the aggregate sentence copies, and
"exports all thirteen constants" at line 175, which §Read discipline correctly
leaves core's). `MAX_NATIVE_NAME_OCTETS`/`MAX_SLUG_OCTETS` = 128 confirmed in
`crates/core/src/placement.rs`, which is what makes the extract-side pair bound
(≤ 4096 pairs × ≈281 octets) land at ≈1.2 MiB, inside `MAX_SNAPSHOT_OCTETS` —
no gap between what the family can emit and what the loader will accept.

## Round-1 findings — all four closed

1. **major → closed.** §The first family: the remaining-depth prune is gone
   ("the walk prunes on nothing but depth and entry count"), and the Revision
   log records it ("nothing prunes by remaining depth: the walk's depth and
   entry-count caps are its whole bound"). Test strategy 6's
   "found at depth ≤ 8 and not at depth 9" is now consistent with the walk, and
   the flagship Proton layout — `AppData/Local/<Project>/Saved/Config/<Platform>/
   <Game>.ini` under the `steamuser` root, 7 path components — is inside
   `MAX_LOCATE_DEPTH` and is enumerated.
2. **major → closed.** Opens are now gated by the constant that exists: "At most
   `MAX_CANDIDATE_FILES` path-grammar matches per tree, in the walk's byte
   order, are opened … a further match is skipped and counted the way an entry
   past `MAX_LOCATE_ENTRIES_PER_TREE` is". `MAX_LOCATE_ENTRIES_PER_TREE` no
   longer prices content probes, and the 32-file cap bounds the opens, not just
   the retained set. Residue of the same accounting: Finding 1 below.
3. **minor → closed.** Both the resolver clause ("at most `MAX_USER_DIRS_PER_GAME`
   such trees in total across all roots") and the constant's Bounds cell
   ("across all of `ctx.roots`") state the union reading, and Test strategy 3's
   "a 17th tree over all of `ctx.roots`" matches it. The aggregate's `+1` term is
   now derivable from the budget table alone.
4. **minor → closed.** "`SnapshotEntryBudget`, checked as each pair enters a
   tree's working set, so the fold stops at `MAX_SNAPSHOT_ENTRIES + 1` pairs and
   names itself", mirrored in Test strategy 4's mid-fold breach. The transient
   peak is now priced: the fold can hold at most 4097 pairs, and lines that
   never become pairs are bounded by the reader's per-file cap, so nothing folds
   ~32 MiB of `+Bindings=` lines into strings before refusing. No new constant,
   no new refusal name.

## Findings

- [minor][new] §Read discipline and budgets — "since §The first family caps a
  tree's reads at `MAX_CANDIDATE_FILES` the product also ceilings the *opens*, so
  the worst case is `17 × 32 × MAX_BINDING_FILE_OCTETS` (544 MiB) per game" —
  the sentence counts one open pass per tree, but the family performs two: the
  content test opens every path-grammar match through `read::open_capped` and
  `read::decode`, and `extract` then "reads only located files, through
  `read.rs`" — necessarily a second open, since the claim payload is
  `UserCandidates { user: String, files: Vec<PathBuf> }` (paths, not bytes).
  Worst case per game is therefore `2 × (MAX_USER_DIRS_PER_GAME + 1) ×
  MAX_CANDIDATE_FILES` opens (1088) and ~1.1 GiB read, against the figure this
  section instructs `schemas/README.md` to copy into the budget family (544
  opens / 544 MiB) — the shipped aggregate is 2× low, in the one place the repo
  states a read cost as a number (the `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES`
  precedent at `schemas/README.md:132-133` is exact, not approximate). Fix: one
  clause naming both legs — "detect's content test and extract's fold are two
  open passes over the same ≤ `MAX_CANDIDATE_FILES` set, so the face reads at
  most `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES ×
  MAX_BINDING_FILE_OCTETS` (1.1 GiB) per game" — or replace the §The first
  family sentence "`extract` reads only located files, through `read.rs`" with
  `UserCandidates` carrying each located file's decoded bytes, so extract never
  re-opens and the single-pass figure becomes true at the cost of holding up to
  `MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` per tree. Either keeps the
  budget table, the constants and the refusal names unchanged.
- [minor][new] §`bindings/v1` (Bounds) with the §Read discipline table row —
  "`files` ≤ `MAX_CANDIDATE_FILES`, each entry at most `MAX_LOCATE_DEPTH` path
  components — the locate walk's own ceiling" versus "`MAX_LOCATE_DEPTH = 8` |
  directory levels the locate walk descends below a `UserTree` root": the two
  bounds are stated in different units and differ by one. A file in a directory
  at the deepest legal descent has 8 directory components plus its own name = 9
  path components, so the refusal Test strategy 10 pins ("a `files` entry over
  `MAX_LOCATE_DEPTH` components") fires on a located file Test strategy 6 says
  the walk finds ("the `Saved/Config/<dir>/` grammar is found at depth ≤ 8"), and
  Test strategy 11's "documents produced by the family all pass the loader"
  cannot hold at that boundary — the one clause whose stated purpose ("the locate
  walk's own ceiling") is to make the contract bound derivable from the walk.
  Minor, not major: the layouts this family exists to serve are 6–7 components
  deep, so the gap is at the cap's edge, not on a shipped path. Fix: state the
  unit once in the table row — "directory levels the locate walk descends below
  a `UserTree` root, so a located file's tree-relative path has at most
  `MAX_LOCATE_DEPTH` components counting the file (a file in the root itself is
  1)" — or restate the contract bound as `MAX_LOCATE_DEPTH + 1` components. No
  new constant, no change to the walk.

## FYI (no severity)

- Finding 2's off-by-one reads to me as a budget-unit question; the correctness
  and complexity seats will see the same text as a Test 6 / Test 10 / Test 11
  consistency question, and may anchor it differently.
- `canonical_paths` drops any root without a `steamapps` directory
  (`crates/discovery/src/steam_common.rs:363-380`) before `roots()` is handed to
  `GameContext`, so `SteamUserdata` is never enumerated under such a root. That
  only shrinks work, so it is not a performance finding, but the correctness seat
  may want to confirm A18's main root always qualifies for the Flatpak/Snap rows.
- The R1 Revision-log entry that closes the depth-prune theme is accurate as
  written; the removed clause leaves no orphan text behind in §The first family,
  §Read discipline or Test strategy 6.
