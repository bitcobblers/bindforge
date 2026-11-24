# performance review r1c4 (ratification pass)

verdict: approve

Scope as directed: the diff `e068cf61…316ef46c` of `02-proposal.md`, then the
full current document, judged only against what this seat approved in round 1
and the two findings it carried. Supporting inputs used: `02-proposal.md`,
`docs/adrs/017-keybinding-extraction-codecs.md` (§Budgets), the proposal's
Revision log, and the repo's budget precedent at `schemas/README.md:104-143`
with `grep` over `crates/core/src` to confirm the "thirteen constants" claim
(13 `pub const MAX_*` in core, `MAX_DOCUMENT_BYTES = 1 MiB`,
`MAX_CHAIN_DEPTH = 32`, `MAX_NATIVE_NAME_OCTETS = 128`, `MAX_SLUG_OCTETS = 128`
— all as the design describes). No `cargo`/`rustfmt`/`npm` leg was needed for
these findings and the shell leg was refused this session; nothing here rests on
an unverifiable code claim.

## Checklist

1. Input-path latency budget — no keystroke/event path exists in this design; the
   only cost centre is the offline `discovery bindings` face, whose read cost the
   aggregate sentence models. One open item, below.
2. Structure costs — every collection that drives a loop is a named constant
   (`MAX_CANDIDATE_FILES` 32, `MAX_LOCATE_ENTRIES_PER_TREE` 16 384,
   `MAX_USER_DIRS_PER_GAME` 16, `MAX_SNAPSHOT_ENTRIES` 4096); the revised
   `SnapshotEntryBudget` ("distinct pairs a tree's fold has ever admitted") is
   self-limiting — the check fires at 4097 admitted, so the retained
   distinct-pair set is bounded by the budget it enforces. No O(n²) over an
   unbounded n entered with these revisions.
3. Memory growth — bounded; `UserCandidates.tree_root` adds one `PathBuf` per
   located tree (≤ 17 per record). No cache is introduced (§Non-goals: no
   snapshot caching or persistence), so nothing to invalidate.
4. Startup cost — reads are pre-capped by metadata length before allocation
   (§Read discipline), unchanged by the revisions.
5. Caching — not applicable; the design ships no cache.
6. Research baseline gap — **evidence not available on this pass**:
   `01-research.md` is not among the inputs the caller named, so the design's
   numbers (544 MiB worst case, "located sets in the fixtures are 1–3 files")
   could not be compared to the surveyed baseline. Recorded, not made blocking:
   the figures are derived from the design's own constants and the repo's
   precedent aggregate shape, and the caller restricted the inputs.

## Findings

- [minor][re-raised] §Read discipline and budgets — "since §The first family caps
  a tree's reads at `MAX_CANDIDATE_FILES` the product also ceilings the *opens*,
  so the worst case is `17 × 32 × MAX_BINDING_FILE_OCTETS` (544 MiB) per game" —
  this sentence and the two-pass structure behind it are **unchanged** by the
  revisions, so the figure is still 2× low: §The first family still opens every
  path-grammar match through `read::open_capped` and `read::decode` for the
  content test in `detect`, and `extract` then "reads only located files, through
  `read.rs`" — necessarily a second pass, since `UserCandidates` still carries
  `files: Vec<PathBuf>` (the revision added `tree_root: PathBuf`, which is an
  ordering key, "never rendered into a document", and does not carry bytes).
  Worst case per game is therefore `2 × (MAX_USER_DIRS_PER_GAME + 1) ×
  MAX_CANDIDATE_FILES` opens (1088) and ≈1.1 GiB read, against the one number
  §Contract-tree amendments item 1 instructs `schemas/README.md` to copy into its
  budget family — whose precedent shape the design itself cites and which is
  exact for a single pass (`schemas/README.md:131-133`: "one chain resolution
  opens at most `MAX_CHAIN_DEPTH` documents, so its total read cost is the
  product already stated by the two budgets, `MAX_CHAIN_DEPTH ×
  MAX_DOCUMENT_BYTES` (32 MiB)"). The Revision log's R1 §Read discipline entry
  restates what the aggregate counts ("the aggregate counts every tree") without
  addressing the number of passes, so the rebuttal does not reach this point; but
  the text is untouched, so this is not a regression and does not bear on
  ratification. Fix: the same one clause as r1 — "detect's content test and
  extract's fold are two open passes over the same ≤ `MAX_CANDIDATE_FILES` set,
  so the face reads at most `2 × (MAX_USER_DIRS_PER_GAME + 1) ×
  MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` (1.1 GiB) per game"; or make
  `UserCandidates` carry each located file's decoded bytes so `extract` never
  re-opens, which makes the single-pass figure true at the cost of retaining up
  to `MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` per tree. Either keeps the
  budget table, the constants and the refusal names unchanged.
- [nit][new] §Read discipline budget table — "`MAX_SNAPSHOT_ENTRIES = 4096` | one
  snapshot's distinct (`action`, input-token) pairs as admitted" — the row's
  subject ("one snapshot's … pairs") and its new modifier ("as admitted") sit in
  two different frames, and the prose below makes the enforced quantity the
  second one ("checked against the distinct pairs a tree's fold has ever admitted
  — a removal frees no room"), which can exceed a snapshot's surviving pair
  total. Harmless in direction — the enforced check is the stricter one, so the
  `bindings/v1` bound (`the (action, input-token) pair total ≤
  MAX_SNAPSHOT_ENTRIES`) still holds a fortiori, and no read cost changes — but
  item 1 of §Contract-tree amendments copies this table into `schemas/README.md`,
  where the ambiguity ships. Fix: state the checked quantity in the row, "the
  distinct (`action`, input-token) pairs one tree's fold may admit, which bounds
  its snapshot's surviving pairs" — no new constant, no change to the check.

## Closed from round 1

- [minor, round 1] `MAX_LOCATE_DEPTH` unit mismatch — **resolved, and resolved in
  the direction the budget table should carry**. The table now states the unit
  once ("components in a located file's tree-relative path — its last four are
  `Saved/Config/<dir>/<name>.ini`"), §The first family's walk now expresses the
  cut as a directory prune ("pruning a directory whose tree-relative path already
  has `MAX_LOCATE_DEPTH` components"), and the two consumers now read the same
  unit (§`bindings/v1`: "each entry at most `MAX_LOCATE_DEPTH` tree-relative
  components (§Read discipline's table)"; Test strategy 6: "located at
  ≤ `MAX_LOCATE_DEPTH` tree-relative components and not beyond"; Test strategy 10
  refuses "a `files` entry over `MAX_LOCATE_DEPTH` components"). The derivation
  closes exactly: a directory at 8 components is pruned, so the deepest located
  directory has 7 and the deepest located file has 8 = the contract bound = what
  the loader refuses above. Test strategy 11 ("documents produced by the family
  all pass the loader") can now hold at the boundary, and the walk got marginally
  cheaper (one fewer reachable level) rather than dearer.
- The `MAX_USER_DIRS_PER_GAME` wording change ("considered" → "kept", with the
  resolver keeping the first 16 in `roots()`-then-`<dir>` order, and Test
  strategy 3 pinning 9 + 8 fixture directories) keeps the aggregate's `17`
  correct: at most 16 `SteamUserdata` trees plus the one `ProtonPrefix` template
  reach `detect` per record.

## FYI (no severity)

- §Determinism and Test strategy 12 — the shuffle moved from the `UserTree` vector
  to "each `Detection.per_user` vector (its `tree_root`s with it)", while the
  dedup/re-sort key is `UserCandidates.tree_root` alone; §Config-path resolvers'
  total order is "`TreeKind` declaration order then byte order of the tree's
  `root`", and a byte-order sort of root paths alone does not reproduce
  declaration-first order, so the law's order is not derivable from the key the
  test now shuffles. Ordering law, not a cost model — correctness seat's.
- §`bindings/v1` Input spelling dropped "may be empty when the file's own merge
  operators removed every input … and never for any other reason", while §The
  first family now says "a `Command` whose pairs the fold has all removed leaves
  no entry" — so `inputs` is now always non-empty, but no contract clause says
  so; the fixture corpora are the format's conformance currency. Contract-language
  matter for the complexity/compatibility seats, not a cost matter.
- `017-B3` is now read per *tree* where the ADR words it "the cap on one game's
  located candidate set per local SteamID"; two `ProtonPrefix` trees sharing the
  `steamuser` label therefore get 32 candidates each (64 reads under one user
  label, across two records). The aggregate sentence is stated per record and
  says it repeats "per matching record the face walks", so this seat's figure is
  unaffected; ledger/compatibility seat's to weigh.
