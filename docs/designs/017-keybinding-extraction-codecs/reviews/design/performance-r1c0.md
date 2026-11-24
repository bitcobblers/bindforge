# performance review r1

verdict: request-changes

Seat: performance, full-panel pass c0, round 1. Scope: the whole of
`02-proposal.md`. Inputs used: `02-proposal.md`,
`docs/adrs/017-keybinding-extraction-codecs.md`, the proposal's Revision log,
and read-only inspection of `crates/discovery/src/{steam_common,steam_linux,keyvalues}.rs`
and the budget constants in `crates/core` to check the design's claims about the
code. No earlier review report or transcript was opened.

Checklist coverage: (1) latency/IO budget on the command path — findings 1 and 2;
(2) structure costs — finding 3; (3) memory growth — findings 1 and 2; (4)
startup cost — finding 3; (5) caching invalidation — not applicable, the run
ships no cache or table (`Non-goals`: "no snapshot caching or persistence"), and
the design performs no vocabulary join, so there is no translation table to
invalidate; (6) research baseline — not assessable, see *Missing evidence*.

## Findings

- [major][new] §The first family / §Read discipline and budgets — "`detect`
  walks each `UserTree` depth-first in byte order to `MAX_LOCATE_DEPTH`,
  collecting `*.ini` files whose path ends
  `Saved/Config/<one directory>/<name>.ini` ... capped at
  `MAX_CANDIDATE_FILES` per tree" — the cap bounds the *result set*, not the
  *traversal*, and two of the three `UserTree` roots are whole user-directory
  trees, not app-scoped ones: `NativeHost` is "`$XDG_CONFIG_HOME` when set and
  absolute, else `$HOME/.config`" (§Config-path resolvers) and `ProtonPrefix` is
  "`<library>/steamapps/compatdata/<appid>/pfx/drive_c/users/steamuser`" — the
  entire emulated user profile, `AppData` included. A real desktop `$HOME/.config`
  holds 10⁴–10⁶ entries and is routinely 8 directories deep, and the worst case is
  precisely the *zero-match* case — a game the family does not have natively —
  where the walk descends the full tree, `DirEntry::file_type()` on every entry,
  and never reaches the 32-file cap that would stop it early. No constant bounds
  directories or entries visited, so the command path has an unbounded synchronous
  walk per invocation; the design's own aggregate sentence counts only *opened
  files* and so reads as a bound where none exists. This is also a new cost
  shape in the product: the shipped scan is shallow — one `read_dir` of
  `<library>/steamapps` (`crates/discovery/src/steam_common.rs:195`) plus
  size-capped VDF/ACF reads — so there is no precedent whose posture already
  bounds a deep walk. Fix: add one design-time constant beside the other two
  (e.g. `MAX_LOCATE_ENTRIES_PER_TREE`, entries examined below a `UserTree` root,
  over-cap entries skipped and counted the way `steam_common`'s `skipped`
  counter already does, degrading to `family-confirmed-files-absent`), and name
  it in the §Read discipline budget table and in the aggregate sentence, so the
  walk has a stated ceiling independent of what the cap-on-found-files happens
  to catch. A prune note belongs in the same sentence: the grammar's four
  trailing components (`Saved/Config/<dir>/<name>.ini`) mean the walk need not
  descend a directory whose remaining depth budget is below that suffix.

- [major][new] §`bindings/v1` / §Read discipline and budgets — "`entries` ≤
  `MAX_SNAPSHOT_ENTRIES`" with the table row "`MAX_SNAPSHOT_ENTRIES = 4096` |
  one snapshot's entries | `017-B2`", read against "`Entries are keyed by the
  `Command` string verbatim ... and carry the distinct `Name` tokens verbatim as
  `inputs`" (§The first family) — the budget counts grouped *actions*, not
  bindings, so no constant bounds a snapshot's input tokens, its pair count or
  its serialized size. A 1 MiB INI is inside `MAX_BINDING_FILE_OCTETS`, so no
  file budget fires; ~15 000 lines of
  `+Bindings=(Name="KEY_<n>",Command="Action")` produce **one** entry carrying
  15 000 inputs, `SnapshotEntryBudget` never triggers, and ×`MAX_CANDIDATE_FILES`
  per tree the in-memory token set grows until it is bounded only by bytes read.
  Peak is worse than one document because `extract` returns every user tree's
  snapshot at once (`Result<Vec<BindingSnapshot>, ExtractionRefusal>`, §The codec
  face), across up to 18 trees. `017-B2`'s stated purpose is the pair count —
  "a game with more bindings than this does not exist today" (ADR-017 Budgets) —
  and the Risks table claims "five named budgets" as the mitigation for
  "Untrusted config files: enormous, deeply nested, UTF-16, symlinked out of the
  tree, over budget", which this unit choice defeats. Fix: define the `017-B2`
  count over distinct (`action`, `inputs` token) pairs rather than over
  `entries` array length — the same `SnapshotEntryBudget` check site, evaluated
  on the pair total — and state in §`bindings/v1` that `entries` array length is
  additionally bounded by it; no new constant and no new refusal name.

- [minor][new] §Read discipline and budgets — "one extraction opens at most
  `MAX_USER_DIRS_PER_GAME × MAX_CANDIDATE_FILES` files, so its worst-case read
  cost is `16 × 32 × MAX_BINDING_FILE_OCTETS` (512 MiB) for one game, offline,
  one game at a time" — the product omits two of the three template families:
  `MAX_USER_DIRS_PER_GAME` bounds only the `SteamUserdata` trees, while
  `ProtonPrefix` and `NativeHost` each add one more tree with its own
  `MAX_CANDIDATE_FILES` budget, and `extract` returns one document per tree. The
  shipped sentence, which the README amendment is told to copy ("A budget bullet
  states the five constants above, names `bindforge-codecs` as their exporter,
  and states the aggregate product", §Contract-tree amendments item 1), therefore
  understates the ceiling in `schemas/README.md`'s budget family by 64 files /
  64 MiB. The figure also ignores the per-record multiplier: step 3 runs
  `GameContext::from_record` → `registry::claim` → `extract` "Per matching
  record", and "More than one record matching (one game in two libraries) is not
  an error: each is processed and rendered" (§The CLI face) — so N matching
  records repeat the same `NativeHost` tree walk N times, since that tree's root
  does not depend on the record's root or library. Fix: state the product as
  `(MAX_USER_DIRS_PER_GAME + 2) × MAX_CANDIDATE_FILES` files (576 for the current
  constants), with the `+2` named as the `ProtonPrefix` and `NativeHost` trees,
  and add one clause that the worst case multiplies by the number of matching
  records; no new mechanism.

## Missing evidence (explicit)

- Checklist item 6 cannot be assessed at this seat: `01-research.md` was not in
  this pass's allowed inputs, so no research baseline can be quoted to test
  §Read discipline's numbers against. The only empirical figure the proposal
  offers for the walk and the caps is its own "located sets in the fixtures are
  1–3 files", which is a statement about fixtures and does not establish a
  traversal or entry-count bound for the real `$HOME/.config` and
  `drive_c/users/steamuser` roots findings 1 and 3 are about. I am not making
  this blocking on that ground; findings 1 and 2 stand on the text and the ADR's
  own budget wording.
- No wall-clock expectation is stated anywhere for `discovery bindings` (the
  design budgets bytes, not time), and the face is a synchronous CLI path that
  re-runs the whole Steam enumeration plus every resolver walk per invocation
  with no cache. Consistent with `Non-goals`' "no snapshot caching or
  persistence" and with ADR-017 `017-L5` ("only the bindings face pays for
  detection"), so not a finding on its own — recorded so the reviser can settle
  it in the same paragraph as finding 3.

## Verified, no finding

- The reader posture the design reuses is the right shape for the budget it
  claims: `crates/discovery/src/steam_common.rs:332-343` applies
  `keyvalues::MAX_DOCUMENT_BYTES` (`= 1024 * 1024`,
  `crates/discovery/src/keyvalues.rs:21`) against `metadata().len()` *before*
  allocating, so A4 and `read::open_capped`'s "metadata length ... **before**
  allocation" are faithful to the code, and `MAX_BINDING_FILE_OCTETS` keeps the
  same 1 MiB figure.
- `MAX_DISCOVERY_SLUG_OCTETS` and `MAX_NATIVE_NAME_OCTETS` (128) exist as cited
  in §`bindings/v1`'s bounds (`crates/core/src/placement.rs:83,85-92`), so the
  contract's bounds sit on exported constants rather than invented numbers.
- Memory that is genuinely bounded: the registry is a static slice iterated in id
  order (`pub static REGISTRY`), there is no translation table or cache to grow
  (the design deliberately performs no vocabulary join), `ScanRecord` adds two
  `PathBuf`s per already-materialized row with `discover()` as a projection, and
  every sort in §Determinism is O(n log n) over already-bounded collections.
  Checklist item 5 is not applicable — no cache, so nothing to invalidate.

## FYI (no severity)

- §`bindings/v1`'s "`user` one of the three spellings above" is not checkable by
  the loader from the document alone — the document carries no tree-kind field —
  while Test strategy #10 asserts a refusal for "a non-digit `user` under a
  userdata-shaped label"; that looks like the correctness seat's, not mine.
