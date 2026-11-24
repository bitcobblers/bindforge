# Code review — performance seat, round 1 dissent pass c1

**Run:** 017-keybinding-extraction-codecs
**Seat:** performance
**Pass:** c1 (dissent — checks this seat's round-1 findings against the revision that answered them)
**Scope:** `git diff a8fd09ec6d6b8210574447f9b68e7cc199f73153..HEAD -- ':(exclude)docs/designs'`
(15 files, +504/−44), read against the full current `02-proposal.md` and
`docs/adrs/017-keybinding-extraction-codecs.md`.
**Inputs used:** the diff, `02-proposal.md`, ADR-017, the runner's gate record, this seat's
round-1 findings, and the project's read-only check commands run by this seat. No other
seat's report or transcript was opened (one repo-wide grep incidentally surfaced
`reviews/`/`transcripts/` line fragments; nothing in this verdict rests on them).

**Verdict: approve** — 0 blocker, 0 major, 0 minor, 2 nit.

## Re-check of this seat's round-1 findings

### 1. [major] `locate`'s open cap counted the located set, not the opens — **resolved**

`crates/codecs/src/families/unreal_ini.rs:187-190` adds `opened` beside `examined`;
:245-248 tests `if opened >= MAX_CANDIDATE_FILES { continue; }` *before*
`read::open_capped` and increments it for **every** grammar match the walk attempts,
independent of what the read or the content test returns; :261 pushes the reader-refused
match into the located set only after its slot was granted, so
`candidates.files.len() ≤ MAX_CANDIDATE_FILES` still holds for `UserCandidates::push`.
`locate` is per tree, so the cap is per tree, which is the unit `017-B3` and the README
bullet count. The shipped docs now match the code: `schemas/README.md:201-210`
("each tree's *opens* are capped at `MAX_CANDIDATE_FILES` independently of how many of
them locate") and `schemas/bindings/v1/spec.md:75-81`.

The missing pin landed: `crates/codecs/tests/unreal_ini.rs:426-476`
(`the_open_cap_bites_on_matches_that_never_locate`) puts `MAX_CANDIDATE_FILES` quiet
(`[/Script/Engine.GameUserSettings]`) matches plus one stating file sorted past them and
asserts `detect` claims nothing, with the in-cap control asserting the stating file does
locate. Run by this seat:

```text
test the_open_cap_bites_on_matches_that_never_locate ... ok
test result: ok. 27 passed; 0 failed; ... finished in 1.15s
```

Worst case per tree is back to 32 opens × 1 MiB, which is the arithmetic the design
accepted. The `Err(_) => {}` arm (`unreal_ini.rs:263-265`) additionally stops
an unstatable path from being located — it still consumes an open slot, which is the
"skipped and counted" law.

### 2. [minor] `detect`/`extract` double pass under-counted by the aggregate sentence — **resolved (documentation route)**

Both doc options I offered were on the table; the run took the second.
`schemas/README.md:201-212` and `schemas/bindings/v1/spec.md:84-92` now state
`2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS`
(1,088 MiB) and name which face pays which pass. Verified against the code: `locate`
reads every opened file for the content test (`unreal_ini.rs:249`) and `extract`
re-reads each located file (`:117-121` → `read_and_decode` :560-577) with no cache, and
`--probe` returns before `extract` (`apps/discovery-cli/src/bindings.rs:249-255`), so
one pass for probe and two for the extracting face is the true figure. The arithmetic is
right: 2 × 17 × 32 × 1 MiB = 1,088 MiB. A cache on `UserCandidates` remains the cheaper
future shape, but the budget statement is now honest, which is what the checklist's
"bounded or documented" asks for. See nit 2 for the one place the two figures still
disagree.

### 3. [minor] `Fold::remove` retained over the whole working set for every removal line — **resolved**

`unreal_ini.rs:358-369` builds the key once and returns when
`self.seen.remove(&key)` says the pair was never held, so the ~25k removal lines a
hostile 1 MiB file can carry are O(1) each; the `retain` is paid only for pairs actually
held. The lockstep the fix relies on is exact in the shipped code: `add` (:343-356)
inserts into `seen` and pushes to `pairs` only together, `remove` takes both, `clear`
empties both — so the early return cannot leave a pair behind that the old code removed
(no behavior change, which was the risk of a performance fix here).
`crates/codecs/tests/unreal_ini.rs:573-591` (`removals_of_absent_pairs_are_no_ops`) pins
2,000 removals of absent pairs as a no-op; this seat ran it: part of the 27-passed suite
above, 1.15 s for the whole file. Residual, accepted: an *effective* removal still pays
one `retain`, but effective removals are bounded by `admitted`
(≤ `MAX_SNAPSHOT_ENTRIES + 1`, and a re-add re-charges `admitted` because `seen.insert`
returns `true` again), so the quadratic term is budget-bounded (≈ 4.1k² pair compares per
tree) instead of file-size-bounded — the property the named budgets exist to give. The
`BTreeSet` shape stays available; it is no longer a finding.

### 4. [minor] The walk sorted the whole `read_dir` listing before applying the entry budget — **resolved (documentation route, as offered)**

`unreal_ini.rs:196-201` tests the budget at the top of the queue loop, so an over-budget
walk enters no further directory — the transient listing is now bounded by directories
actually entered, and the pending `queue` by entries examined
(≤ `MAX_LOCATE_ENTRIES_PER_TREE`). `crates/codecs/src/lib.rs:51-59` records the
distinction beside the constant ("bounds *examined entries*, not the walk's transient
per-directory listing … determinism of the examined set needs the sorted listing"), which
is the alternative my fix offered in place of `entries.take(room)`. The in-loop
stop-and-count at :219-221 is unchanged, so the law pinned at
`tests/unreal_ini.rs:371-400` (`the_locate_walk_stops_at_the_entry_budget`, still green in
the run above) is untouched. Accepted.

### 5. [nit] Registry pass 2 re-ran `detect` for a family pass 1 already ran — **resolved**

`crates/codecs/src/registry.rs:39-62` remembers the ids whose `detect` ran in pass 1 and
skips them in pass 2; `crates/codecs/tests/registry.rs:215-255` pins it on work done with
an `AtomicUsize`-counting decliner. Run by this seat:
`test pass_two_does_not_re_run_a_family_the_appid_pass_already_ran ... ok`
(`test result: ok. 7 passed; 0 failed; finished in 0.00s`). One `claim` now runs each
family's `detect` at most once, so `017-R4`'s routing data cannot double the tree walks.

### 6. [nit] One slug in two libraries paid the whole claim twice — **not resolved; carried (see nit 1)**

The fix memoizes per record-claim; the key includes `library`, which is exactly the field
that differs between the two records. Detail in nit 1 below.

### 7. [nit] `read::decode` copied a buffer the caller already owned — **resolved**

`crates/codecs/src/read.rs:57-80` adds `decode_owned(Vec<u8>)` and the no-BOM path moves
the buffer into `String::from_utf8(bytes)`; `decode` stays as the borrowed pure seam the
suites call, delegating through `to_vec()`. Both real call sites use the owned form
(`unreal_ini.rs:249`, `unreal_ini.rs:574`), so the full-size second copy is gone from both
paths that ever hold an owned buffer. Published-surface pin updated at
`crates/codecs/tests/zz_api_probe.rs:79-81`.

## Findings this pass

### nit 1 — the new claim memo cannot hit for the multi-record case its comment names
`apps/discovery-cli/src/bindings.rs:222-237` (carried from this seat's round-1 nit 6)

The comment asserts that "two records of one game in two libraries hand the resolvers the
same `(appid, library, roots)` anchors". They do not: `GameContext::library` is
`record.library` (`crates/codecs/src/context.rs:66`), and two libraries *is* the
difference between those two records — that is what `library` records. The two records of
`cli_bindings_tests.rs:406-442` (`one_slug_in_two_libraries_prints_two_probe_groups`, two
`--steam-root`s) therefore take different memo keys, both miss, and both pay a full
`registry::claim`. The duplicated work my round-1 nit named is untouched, because the
duplicated half is the `SteamUserdata` trees, which derive from `ctx.roots` + `ctx.appid`
alone (`crates/codecs/src/paths.rs:63-112`) and are byte-identical across those records;
only the `ProtonPrefix` tree differs (`paths.rs:117-132`). A hit needs two records with
the same appid *and* the same library *and* the same root set — a case the scan's own
library dedupe keeps from arising — so the map is inert in every shipped scenario while
its comment reads as the record that the redundancy is closed.

Cost, stated plainly: with one slug in N libraries the face pays N claims, each walking
up to `min(MAX_USER_DIRS_PER_GAME, N)` identical userdata trees, so the walk count is
N × min(16, N) against N distinct trees. This is disclosed by the aggregate bullet's
"repeating per matching record the face walks" (`schemas/README.md:210-212`), so it is a
documented cost, not an undocumented one — hence nit, unchanged from round 1. The
`Detection` clone on a hit and the per-record key clones (appid + library + all roots)
are bounded by the matched-record count and cost nothing measurable.

Fix: memoize at the level the cost lives at — a per-`claim` `locate` cache keyed by
`UserTree::root` inside `detect`'s caller chain, or split the CLI memo into a userdata
half keyed `(appid, roots)` and a proton half keyed `(appid, library)`. If neither is
wanted this run, delete `claim_route` and correct the comment to say the repeat is the
documented per-record cost: an inert cache plus a comment that misstates the anchors is
worse than either choice, because the next reader will trust it.

### nit 2 — the shipped aggregate and the proposal's aggregate sentence now disagree
`schemas/README.md:201-212`, `schemas/bindings/v1/spec.md:84-92` vs
`02-proposal.md:357-366` (§Read discipline and budgets)

Finding 2's documentation fix moved the shipped figure to
`2 × … = 1,088 MiB`; the proposal's own sentence still states
"`17 × 32 × MAX_BINDING_FILE_OCTETS` (544 MiB) per game" and §Contract-tree amendments
item 1 still tells the implementer to state "the aggregate product" in the README — which
is now a different product. The shipped number is the correct one (verified above), and no
test asserts either figure, so nothing breaks; the code phase cannot edit `docs/designs/`
either. The only exposure is the run record: a reader comparing the design to the contract
tree finds two worst-case numbers for one face.

Fix: no code change. Add the divergence to the Rollout 8 ledger hand-off (beside the
other "shipped reading differs from the wording" units) so the operator's copy of §Read
discipline gains the pass multiplier, and record in the run's own notes that the design's
544 MiB counts one pass (the `--probe` face) and 1,088 MiB counts the extracting face.

## Checklist

1. *Event/keystroke path* — **N/A.** No event loop; the face runs once per process
   (`bindings.rs:run`), and `bindings list`'s loop is untouched (`017-L5`).
2. *Complexity, no unbounded O(n²)* — locate opens are budget-bounded (finding 1);
   removal lines are O(1) unless effective, and effective removals are budget-bounded
   (finding 3); `extract` sorts groups and files per tree (bounded by 17 × 32);
   `entries()` folds through `BTreeMap`/`BTreeSet`. Residual open item is nit 1's
   per-record repeat of an identical walk, disclosed by the aggregate bullet.
3. *Unbounded growth* — every collection added or touched by this diff is bounded:
   `opened`/`candidates.files` ≤ `MAX_CANDIDATE_FILES` (now on the opens), `examined` and
   the pending `queue` ≤ `MAX_LOCATE_ENTRIES_PER_TREE`, `Fold.pairs`/`seen` ≤
   `MAX_SNAPSHOT_ENTRIES + 1` admitted, `claim_route` ≤ matched records (bounded by the
   scan). The new cache's key completeness is nit 1.
4. *Startup* — **N/A.** Nothing parses on boot; the budgets gate the only read paths and
   `--probe` skips the second pass.
5. *Cache invalidation covered by a test* — the one cache this diff adds
   (`claim_route`) is a pure-function memo (needs no invalidation within a run, and
   `claim` is documented pure at `registry.rs:41-44`), but it has no test and cannot hit;
   covered in nit 1. The pass-2 skip cache *is* covered
   (`tests/registry.rs:215-255`).
6. *Benchmark-like checks* — no benchmark or timing note ships. This seat ran the gates
   itself rather than resting on the runner's capture, whose only quoted line was
   `test result: ok. 0 passed` and so shows no per-test evidence:
   `cargo test --locked -p bindforge-codecs --test unreal_ini --test registry --test
   snapshot` → `test result: ok. 27 passed; 0 failed; … finished in 1.15s` (unreal_ini,
   including both new perf pins), and `--test registry` →
   `test result: ok. 7 passed; 0 failed; finished in 0.00s`. No measurable timing claim
   is made by the change; the removal-no-op pin is functional, not a benchmark.

## Notes (no severity)

- The new `snapshot.rs:305-323` traversal check re-walks `Path::components()` per
  `files` entry (≤ `MAX_CANDIDATE_FILES`, loader path only), the `ExtractionRefusal`
  `Display` change (`codec.rs:157-203`) allocates only on refusal rendering, and
  `tree_relative` (`unreal_ini.rs:548-555`) builds a small `Vec<String>` per located
  file: all trivial against the budgets above, none a finding.
- `registry.rs:60` skips pass-2 families by id; if a future registry slice ever declared
  two families under one id, the skip would cover both. The declaration order is pinned
  byte-sorted (`tests/registry.rs`), id uniqueness is not asserted — outside this seat,
  and unreachable with a one-family registry.
