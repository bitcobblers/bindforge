# Correctness review r1 (dissent pass c2)

verdict: request-changes

Scope: `git diff a4dda6a e068cf6` of `02-proposal.md`, then the full current
document, the ADR, the Revision log, and my round-1 findings. Checks run: read
`crates/discovery/src/steam_common.rs:70-129,365-389` (`with_roots` →
`canonical_paths` → `BTreeSet`, `steamapps` existence filter) and
`crates/core/src/{placement.rs,manifest.rs}` for `MAX_SLUG_OCTETS`,
`MAX_DISCOVERY_SLUG_OCTETS`, `MAX_NATIVE_NAME_OCTETS`; live fetch of A15's
Epic basis URL. No write leg run; no cargo leg run (the crate this design
describes does not exist yet, so no shipped test can confirm or deny it).

## Round-1 findings — status

- major, dedup key weaker than tree identity — **closed in substance**: §Determinism
  now dedups by `UserTree.root`, §The CLI face step 3 says "dedup by tree identity",
  Test #12 adds the two-libraries case. The closure is what opened new finding 1.
- minor, union cap on `MAX_USER_DIRS_PER_GAME` — closed (table row, resolver
  bullet, aggregate product and Test #3 agree); residual cut-order gap is nit 5.
- minor, "candidate order" — closed and verified true: `with_roots` keeps
  `canonical_paths(roots)` (canonicalized, `steamapps`-filtered, BTreeSet byte
  order), which is what the new "canonicalized, existence-filtered, byte order"
  and "core byte order" wording now state.
- minor, `!` operand grammar — closed in §The first family and Test #7; the
  entry-identity consequence is new finding 3.
- minor, A15 operator enumeration — closed; the fetched basis page lists exactly
  four special characters (`+`, `-`, `.`, `!`), so the row now matches its URL.
- minor, unbounded `files` entries — closed in §`bindings/v1` and Test #10; the
  bound's units are new finding 4.
- minor, ledger list / clause count — closed and now bijective: §Context's seven
  clauses, §Risks' "Seven clauses", §Rollout 8's hand-off, and seven Revision-log
  entries map one-to-one; the §GameContext entry is relabelled `R1:`.
- nit, "remaining depth" prune — closed (removed; "the walk prunes on nothing
  but depth and entry count").
- nit, `detect`'s missing refusal channel — closed ("located on path grammar
  alone and `extract` refuses it by name").
- nit, Steam Deck row — closed as proposed.

## Findings

- [major][new] Determinism (`017-L4`) — canonical + The codec face — "across
  documents, deduplicated by the `UserTree.root` path each was built from — tree
  identity, never rendered" (with §The CLI face step 3's "§Determinism's dedup by
  tree identity keeps one tree stated once" and Test #12's "with the registry and
  the `UserTree` vector shuffled in the test harness, yields byte-identical
  documents") — the dedup key the law now names is carried by no type in the
  design. `Detection` is `{family, evidence, per_user: Vec<UserCandidates>}` and
  `UserCandidates` is `{user, files}`; `extract` returns `Vec<BindingSnapshot>`,
  whose top-level keys are closed at `{format, version, codec, game, user,
  entries, provenance}` and `files` are tree-*relative*. So neither the face,
  which does the dedup at step 3, nor the ordering law can see a document's
  `UserTree.root`: the round-1 key (`user`, `provenance.files`) was exactly the
  payload the interface exposes, and the replacement key is not in it. Test #12
  compounds it: shuffling "the `UserTree` vector" and demanding byte-identical
  documents requires re-sorting documents by tree identity, which is the same
  missing field. Failure scenario as written: two records for one slug each
  return their `user_trees` documents and the face has nothing to compare but
  `user` plus relative paths — the round-1 collision this finding was raised to
  kill, re-opened one layer down. Fix: add tree identity to the claim payload —
  `pub struct UserCandidates { pub user: String, pub tree_root: PathBuf, pub
  files: Vec<PathBuf> }`, `tree_root` carried for dedup and ordering only and
  never rendered into a document (§Determinism already says "never rendered") —
  and state that `per_user` is built in `user_trees` order and `extract`'s `Vec`
  is positionally parallel to it; that replaces the implicit positional coupling
  and needs no change to `bindings/v1`'s closed key set.

- [minor][new] Determinism + `bindings/v1` + Test #12 — "two distinct trees that
  share a `user` label stay two documents" (with §`bindings/v1`'s "One JSON
  document per game per located user tree" and #12's "two `ProtonPrefix` trees
  sharing the `steamuser` label with different bindings — yield two documents") —
  document identity is now the located *tree*, which is a reading of `017-C3`'s
  "one document per game per local SteamID where user-scoped" that the run's own
  divergence record does not carry: §Context's seven clauses list `017-C3`'s
  "`inputs` spelling" only, and the Revision-log entry that speaks to document
  count says "`extract` returns one document per located user tree … `017-C3`'s
  per-user rule wins", which is now the opposite of what the section ships (C3's
  per-user rule does not win — per-tree does, and it can produce two documents
  for one user label). Consequence beyond the ledger: two documents can share
  every identity field a consumer can read (`codec`, `game`, `user`) while
  differing in `entries`, with `provenance.files` tree-relative and so identical
  too, so `--json` shows an unlabelled pair of same-user documents. Fix: keep
  the per-tree rule (it is the sound one) and make the record true — reword that
  Revision-log entry's basis to "read differently than `017-C3`'s per-local-
  SteamID wording", name `017-C3`'s document-identity clause in §Context's count
  and §Rollout item 8's hand-off, and add one clause to §`bindings/v1` stating
  that a consumer distinguishes same-`user` documents by nothing in the contract
  and must union their entries.

- [minor][new] The first family — `extract` — "a bare `!Key=` (no value) clears
  every pair accumulated for that key, `!Key=<tuple>` removes only the pairs it
  names" (with §`bindings/v1`'s "`inputs` may be empty when the file's own merge
  operators removed every input for an action" and "the (`action`, input-token)
  pair total ≤ `MAX_SNAPSHOT_ENTRIES`, which bounds `entries`", Options' "record
  every distinct (native action, native input) pair … grouped by action", and §Read
  discipline's "`SnapshotEntryBudget`, checked as each pair enters a tree's working
  set") — the fold still does not say whether an action whose pairs have all been
  removed survives as an entry with `inputs: []`, and the two passages answer
  opposite ways: §`bindings/v1`'s allowance says it survives, Options(b) derives
  entries from the surviving pair set, so an action with no pairs contributes
  nothing. The bare-clear is the operation that makes this the common case, and it
  decides the budget too: if emptied actions survive and the mid-fold check counts
  the *current* working-set size, a churn file (add 4096, remove 4096, add 4096
  others) passes the check and ships more than `MAX_SNAPSHOT_ENTRIES` entries, so
  "which bounds `entries`" is false; if the check counts distinct pairs *ever
  admitted*, entries are bounded and §Read discipline should say so. Fix: one
  clause in §The first family — "a `Command` whose pairs are all removed leaves no
  entry" (then delete §`bindings/v1`'s empty-`inputs` allowance and its "never for
  any other reason" tail) or the mirror of it — plus three words in §Read
  discipline fixing the check to distinct pairs admitted; and name the bare
  operand for the other three operators in the same sentence and in Test #7
  ("`-Bindings=`, `+Bindings=` and `.Bindings=` with no value are out-of-grammar
  and skipped"), which the new bare-`!` form otherwise leaves open.

- [minor][new] `bindings/v1` — the snapshot contract — "`files` ≤
  `MAX_CANDIDATE_FILES`, each entry at most `MAX_LOCATE_DEPTH` path components —
  the locate walk's own ceiling" (with §Read discipline's table, "`MAX_LOCATE_DEPTH
  = 8` | directory levels the locate walk descends below a `UserTree` root", and
  Test #6's "found at depth ≤ 8 and not at depth 9") — the new bound borrows a
  constant whose only definition counts *directory levels*, while the bound itself
  counts *path components*, and nothing in the document defines the depth of a
  file. Under the table's definition a file eight directory levels below the tree
  root has nine tree-relative components, so `detect` legitimately locates a file
  whose entry the loader then refuses — Test #10 ("a `files` entry over
  `MAX_LOCATE_DEPTH` components") and Test #11 ("documents produced by the family
  all pass the loader") cannot both hold. The layouts the fixtures model are at
  that boundary: `Documents/My Games/<Project>/Saved/Config/<Platform>/<name>.ini`
  is seven components under a Proton prefix root. Fix: state the units once, in
  the candidate's terms — "a located file's tree-relative path is at most
  `MAX_LOCATE_DEPTH` components, its last four being the grammar's
  `Saved/Config/<dir>/<name>.ini`" — and make #6 say "found at ≤ `MAX_LOCATE_DEPTH`
  components and not beyond" instead of an undefined "depth".

- [nit][new] Config-path resolvers — `src/paths.rs` — "one per digit-named `<dir>`
  in byte order, at most `MAX_USER_DIRS_PER_GAME` such trees in total across all
  roots" (with the section's return order, "in `TreeKind` declaration order then
  byte order of the tree's `root`", and Test #3's "a 17th tree over all of
  `ctx.roots`") — the cap is now stated over the union, but the order in which the
  16th tree is reached is not, and the section runs two different orders at once:
  the enumeration reads as roots-then-directory, the return order is byte order of
  the full tree path, and those disagree for prefix-adjacent roots (byte order
  puts `/home/a-b/userdata/…` before `/home/a/userdata/…` because `-` < `/`, while
  `roots()` byte order puts `/home/a` first). Which users the silent bound keeps is
  therefore open, and #3 does not pin it. Fix: name the cut order in that bullet —
  "the cap is applied while enumerating `ctx.roots` in `roots()` order and
  `<dir>` in byte order; the surviving trees are then returned in `root` byte
  order" — and say which two roots and directories #3's fixture uses.

## FYI (no severity)

- `017-B3`'s wording caps "one game's located candidate set per local SteamID"
  while the shipped constant caps a *user tree's* set; with the new per-tree
  documents one user label can hold two trees and so two 32-file caps. Another
  seat may want it on the ledger list beside the `017-C3` identity clause.
- Test #4's "`UserCandidates::push` refuses a 33rd file" and §The first family's
  new "a further match is skipped and counted" describe two overflow behaviours for
  one bound; both are defensible (type invariant vs walk policy) but the
  unreachable-in-production half is worth one parenthetical.
