# Code review — performance seat, round 1 pass c0

Scope reviewed: `git diff fc28ab94f313d079222d61438bf5b9074a1b8039...HEAD -- ':(exclude)docs/designs'`
(64 files, +5986/−80). Inputs: that diff, `02-proposal.md`,
`docs/adrs/017-keybinding-extraction-codecs.md`, the runner's gate record
(green: `npm run check`, `cargo fmt`, `cargo clippy -D warnings`,
`cargo test --workspace --locked`), and the design's §Read discipline,
§The first family, §Config-path resolvers, §The CLI face budgets. No prior
findings — this is a first read.

Checklist applied: `references/performance.md` only.

## Verdict

**request-changes** — one major, four minor, three nit. Nothing here is a
correctness claim; each finding is a cost the change pays that the design's
own budget text either does not count or counts differently.

## Checklist walk

1. **Event/keystroke path** — no such path ships here (an offline CLI face).
   The analogue is the read path: `user_trees` → `locate` → `open_capped` →
   `decode` → fold, run once per matching record. No synchronous network
   anywhere on the face (confirmed: `bindings.rs:164-243` builds plugins and
   stops — no catalog client, no probe, no store, matching Options' A13
   divergence). Findings 1, 2, 3, 4, 6 are read-path costs.
2. **Complexity** — findings 3 (bounded-quadratic removal fold) and 5
   (re-detect across claim passes). Maps/sets are used where identity lookup
   was intended elsewhere: `Fold::seen` is a `HashSet`, `groups` dedup and
   `MergedClaim` tree-identity checks are linear over ≤ 17 items, the
   `scan()` library dedupe is a `BTreeSet`, `snapshot.rs` checks
   `files_raw.len()` before iterating it (`snapshot.rs:290`), and
   `user_trees` sorts the `<dir>` names once per root (`paths.rs:89`) rather
   than per candidate — those are all right.
3. **Unbounded growth** — no caches, registries, or long-lived collections:
   `REGISTRY` is a `static` of size one, `MergedClaim.groups` is bounded by
   the 17-tree cap, no snapshot is persisted (design Non-goals: "no snapshot
   caching or persistence"). The one unbounded transient is a directory
   listing materialized before the walk budget applies — finding 4.
4. **Startup** — the face scans every resolved root and every
   `appmanifest_*.acf` to resolve one slug. That is the design's chosen
   shape (Options (c), §The CLI face step 1) and the `scan()` refactor did
   not add work: `libraries_in_root_with_skips` already ran `canonical_paths`
   per root (`steam_common.rs:165-180`), so dropping the outer
   `canonical_paths` over the union removes a second `fs::canonicalize` per
   library and moves dedupe to a `BTreeSet` of canonical paths — net fewer
   syscalls, same scan volume. `discover()` stays a projection
   (`steam_common.rs:156-158`), so `list`'s runtime is unchanged (`017-L5`).
   Accepted as designed, no finding.
5. **Cache invalidation** — **not applicable**: no cache, memo table, or
   persisted document ships in this diff, so there is no invalidation law to
   cover. The mirror image of that absence is findings 2 and 6, where the
   same bytes are read twice or N times because nothing is reused.
6. **Benchmark-like test or timing note** — the change ships a documented
   worst case (`schemas/README.md:202-208`, `schemas/bindings/v1/spec.md:74-78`:
   `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS`
   = 544 MiB read per game) but **no test or note that measures or counts it**,
   and no test counts file opens at all. That missing evidence is why finding
   1 is invisible to the suite: `crates/codecs/tests/unreal_ini.rs:262-280`
   builds 33 grammar matches that all state the section, which is exactly the
   one input where "matches located" and "files opened" coincide. I could not
   run the check commands or any timing myself — this seat's shell allow covers
   `git diff` only, and I write no file other than this report — so no measured
   number is quoted; every claim below is read off the code paths, with the
   arithmetic shown.

## Findings

### 1. major — the locate walk's open cap gates on the *located* set, so uncapped matches are opened and buffered whole

`crates/codecs/src/families/unreal_ini.rs:226-243`

```rust
if !grammar_match(&child_relative) { continue; }
if candidates.files.len() >= MAX_CANDIDATE_FILES { continue; } // cap on LOCATED
match read::open_capped(&path).and_then(|bytes| read::decode(&bytes)) {
    Ok(text) => { if states_bindings(&text) { let _ = candidates.push(path); } }
    Err(_) => { let _ = candidates.push(path); }
}
```

`candidates.files` grows only when a match *locates* (states the section, or
the reader refused it). A match that decodes cleanly but has no
`[/Script/Engine.PlayerInput]` section is opened, buffered whole, decoded, and
scanned — and does not move the counter that gates the next open. So the number
of opens per tree is bounded only by `MAX_LOCATE_ENTRIES_PER_TREE` (16,384,
`lib.rs:55`), not by `MAX_CANDIDATE_FILES` (32).

That contradicts two normative sentences this same diff ships:
`schemas/README.md:202-208` ("because a tree's reads are capped at
`MAX_CANDIDATE_FILES` the product caps the *opens* too — so the worst case is
… (544 MiB) read per game, offline") and `schemas/bindings/v1/spec.md:74-78`
(the same composite), and the proposal §The first family ("**At most
`MAX_CANDIDATE_FILES` path-grammar matches per tree** … are opened through
`read::open_capped` and `read::decode` for that content test; a further match
is skipped and counted"). The design's accepted-risk arithmetic (544 MiB/game)
therefore does not bound what ships: the practical worst case is every
`…/Saved/Config/<dir>/*.ini` under each walked tree opened and buffered at up
to 1 MiB each — reachable without any adversary, since a Proton prefix's
`drive_c/users/steamuser` tree reaches the grammar at 7 relative components
(`AppData/Local/<Game>/Saved/Config/<platform>/<name>.ini` ≤ `MAX_LOCATE_DEPTH`
= 8), so every game's config in that prefix, and every platform directory of
one game's `Saved/Config`, is read to answer one query. Each overshoot open
also pays the full `Vec<u8>` buffer plus `decode`'s copy (finding 2 doubles
that again for located files only).

Fix: count opens independently of locating. Add `let mut opened = 0usize;`
beside `examined`, test `if opened >= MAX_CANDIDATE_FILES { continue; }`
before `read::open_capped`, and increment it for every grammar match the walk
attempts to test; the skip-and-count then matches the README's "the product
caps the opens too". Pin it with the test that does not exist yet: N
grammar-matching `.ini` files that do **not** state the section plus one that
does, sorted past the cap, asserting the stating file is not located — the
current test's fixtures make the two counters indistinguishable
(`tests/unreal_ini.rs:262-280`).

### 2. minor — every located file is opened, buffered and decoded twice per run

`crates/codecs/src/families/unreal_ini.rs:232` (detect's content test) and
`crates/codecs/src/families/unreal_ini.rs:117-121` + `516-533`
(`extract`'s `read_and_decode`)

`Detection` carries paths, not contents, so `extract` re-reads and re-decodes
each of the ≤ 32 files per tree it was handed. Even with finding 1 fixed, the
octets a single `discovery bindings <slug>` run reads are **2 ×** the
`(16+1) × 32 × 1 MiB` = 544 MiB the shipped README states as the per-game
worst case, because the aggregate sentence counts one pass. (The design noticed
this hazard for the trait shape it rejected — ADR-017's Alternatives praise
`Detection` for not "forc[ing] the extractor to re-probe the tree a second
time" — but the re-*read* survived.)

Fix, either half: (a) carry the already-decoded text on `UserCandidates` (or a
private per-`detect` read cache keyed by path) so `extract` folds what `detect`
read, and keep `read_and_decode` for the located-on-path-grammar-only case the
proposal names; or (b) state the doubled pass honestly in `schemas/README.md`
and `schemas/bindings/v1/spec.md` — `2 × (MAX_USER_DIRS_PER_GAME + 1) ×
MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` — and note that `--probe` pays
one pass and the extracting face two. Whichever way, the README and the spec
must describe the run the code performs.

### 3. minor — `Fold::remove` scans the whole working set for every removal line, including removals of pairs the set never held

`crates/codecs/src/families/unreal_ini.rs:329-333`

```rust
fn remove(&mut self, action: &str, input: &str) {
    self.seen.remove(&(action.to_string(), input.to_string()));
    self.pairs.retain(|pair| pair.0 != action || pair.1 != input);
}
```

Two allocations for the set key, then an unconditional `retain` over
`self.pairs` (up to `MAX_SNAPSHOT_ENTRIES + 1` = 4,097 entries). Removal lines
consume no budget — a `-Bindings=`/`!Bindings=` line naming an absent pair
never touches `admitted` — so their count per file is bounded only by
`MAX_BINDING_FILE_OCTETS`: a 1 MiB file of ~40-byte removal tuples is ~25k
lines, each scanning ~4k pairs ≈ 10⁸ predicate evaluations, and `extract`
folds up to 32 located files per tree across up to 17 trees. The fold is
quadratic in (removal lines × working set) where the design's budgets were
meant to be the only bound; the whole thing is untrusted input (§Risks:
"enormous, deeply nested … over budget").

Fix: `seen` and `pairs` are maintained in lockstep by `add`/`remove`/`clear`,
so membership is exact — build the key once, `if !self.seen.contains(&key) {
return; }` before touching `pairs`, and reuse the one key for the set removal
and the retain predicate. Storing the working set in a `BTreeSet` keyed by
`(action, input)` makes removal O(log n) at no cost, since `entries()`
(`unreal_ini.rs:343-358`) already sorts by action and input byte order.

### 4. minor — a whole directory listing is collected and sorted before the walk's entry budget applies

`crates/codecs/src/families/unreal_ini.rs:194-206`

`read_dir`'s output is collected into `Vec<(String, DirEntry)>` and sorted for
byte order *before* the loop tests `examined >= MAX_LOCATE_ENTRIES_PER_TREE`,
so a single directory's size — not the budget the constant is presented as
capping (`lib.rs:53-55`: "entries the locate walk examines below one
`UserTree` root") — bounds the walk's transient allocation and its
`O(m log m)` sort. `proton_trees`' anchor (`<library>/steamapps/compatdata/…/
users/steamuser`, `paths.rs:117-137`) is exactly the kind of home tree that
holds a directory with a very large entry count, and the same shape repeats in
`user_trees`' `<dir>` collection (`paths.rs:72-89`, bounded in practice by the
SteamID directories).

Fix: test the budget at the top of the queue loop, before `read_dir` (an
over-budget walk should not enter another directory at all), and cap the
listing to the remaining budget (`entries.take(room)`) — determinism is not
lost, because which entries a budget-truncated walk examines is already
unspecified beyond the stop-and-count law (`tests/unreal_ini.rs:231-256`
asserts only that the hidden candidate is *not* located). Alternatively,
document beside `MAX_LOCATE_ENTRIES_PER_TREE` that the constant bounds
examined entries and not the transient listing.

### 5. nit — pass 2 re-runs `detect` for a family pass 1 already ran and that declined

`crates/codecs/src/registry.rs:40-60`

When a family's `appids()` contain `ctx.appid` and its `detect` returns `None`,
pass 2 iterates the same slice and calls `detect(ctx)` on that family again.
For `unreal-ini` a `detect` is the full `user_trees` enumeration plus a
`locate` walk of every located tree, so a table-declining family pays two
complete tree walks per record. Unreachable in shipped configuration
(`appids()` is empty, `unreal_ini.rs:67-71`, A21), live the moment `017-R4`
supplies routing data — which is the seam §The registry built "so `017-R4`
lands data into a seam instead of prompting a redesign".

Fix: collect the ids whose `detect` ran in pass 1 into a small set and skip
them in pass 2 (or iterate pass 2 over `ordered` minus the pass-1 candidates).

### 6. nit — a slug with k matching records walks and reads the identical trees k times before the dedup throws them away

`apps/discovery-cli/src/bindings.rs:224-235` with `137-148`

Each matching record gets `GameContext::from_record(record, roots)` where
`roots` is the whole plugin root set (`bindings.rs:187`), so two libraries
holding one AppID resolve the *same* `userdata/<steamid>/<appid>` trees, and
`registry::claim` redoes the complete `user_trees` + `locate` walk — including
finding 2's file reads — for the second record. `MergedClaim::absorb` drops the
duplicate groups only after that work is paid, and `extract`'s
`dedup_by(tree_root)` (`unreal_ini.rs:102-104`) dedups the documents, not the
walks. The design acknowledges the repeat in the aggregate's "repeating per
matching record the face walks", and k is normally 1–2, so this is the smallest
item here.

Fix: memoize the claim per `(appid, roots)` across the record loop, or skip a
record whose `(appid, plugin root set)` was already claimed, keeping the tree
identity as the key §Determinism already uses.

### 7. nit — `decode` copies the whole buffer it was handed, twice over on the read path

`crates/codecs/src/read.rs:58-72`

`String::from_utf8(bytes.to_vec())` and `String::from_utf8(bytes[3..].to_vec())`
allocate and memcpy a second full-size copy of a file that `open_capped`
already owns, and both call sites (`unreal_ini.rs:232`, `unreal_ini.rs:529-531`)
immediately discard the `Vec<u8>`. With finding 2's double pass that is four
full-file buffers per located file per run.

Fix: add `read::decode_owned(Vec<u8>)` (or let `decode` take `Vec<u8>`) so the
no-BOM path moves the buffer into `String::from_utf8`; keep the borrowed slice
for the pure seam the suites call directly.

## Out of specialty (FYI, no severity)

- `SteamCore::scan`'s dedupe move is a small win: `libraries_in_root_with_skips`
  already canonicalizes per root (`steam_common.rs:165-180`), so dropping the
  outer `canonical_paths` over the union removes one `fs::canonicalize` per
  library. Whether per-root ownership changes which root a shared library is
  attributed to, and the `library`-order change in scan order, are the
  correctness and compatibility seats' rows, not mine.

## Gate record

Green on all four legs as captured (`cargo test --workspace --locked` exit 0).
The record quotes one `test result: ok. 0 passed; 0 failed` line, which is not
enough to establish that the `bindforge-codecs` suites ran; that is the testing
seat's row. It does not affect any finding above: all seven are read off the
source, and finding 1's gap is precisely the one no existing test can see.
