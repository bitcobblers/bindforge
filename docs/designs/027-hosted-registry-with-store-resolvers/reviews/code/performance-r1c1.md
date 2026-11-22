# Code review — performance seat, round 1 dissent pass (c1)

- **Run**: 027-hosted-registry-with-store-resolvers
- **Seat**: performance (checklist: `.opencode/skills/code-review/references/performance.md`)
- **Scope as given**: `git diff 75cbc433ed8a546734c41d4eb928395373ab73e6..HEAD -- ':(exclude)docs/designs'`
  (21 files, +1333/−533), read against the full current `02-proposal.md` and
  `docs/adrs/027-hosted-registry-with-store-resolvers.md`, plus the runner's gate record
  (2026-11-22T09:51:31Z, green) and this seat's round-1 findings. No review report or
  transcript was opened.
- **Pass type**: dissent c1 — the revision under review was written to answer this seat's
  r1 findings, so those are checked first; anything else is reported only where the
  changed code makes it true.

## Verdict

**approve.** All three r1 findings are closed in the changed code (one major, one minor,
one nit). No blocker or major is open. What is left is one minor evidence item the design
itself asks for (Rollout 4's measured rewrite, not a code defect) and two nits about
boundaries the r1 fix newly moved.

## Re-check of this seat's round-1 findings

### 1. major — per-mint deep clone + full index rebuild + whole-model serialize under `mutations` — **closed**

`state.rs:411-490` no longer clones the issued `Catalog` or calls `upsert`. The mint now
takes:

- `Catalog::to_bytes_with` (`corpus.rs:355-378`) — the document spelled **against the live
  model**: one `Vec<&Record>` of `n+1` borrowed pointers with the new record spliced into
  its `game_id` slot, handed to the one shared document writer `write_document`
  (`corpus.rs:382-398`, extracted from the old `to_bytes`, so both spellings are one
  code path, not a copy).
- `Catalog::insert` (`corpus.rs:410-424`) — the postings are **inserted**: one `BTreeMap`
  entry per source key plus one ordered `Vec::insert` into the existing posting.
  `rebuild_postings` is not on the mint path.

That is §Concurrency's sentence, literally: a mint "computes and serializes the new
document straight into the write buffer (a mint inserts the new key's posting rather than
rebuilding the index), pays tmp-and-rename, and takes its document's `RwLock` write half
only for the model swap" (proposal:458-462). The write-half swap is at `state.rs:488`,
after the persist, with `mutations` still held, so the swap and the bytes agree.

Two things I checked to make sure the fast path is actually the mint's path and not a
decoy:

- **The `upsert` fallback cannot be taken by a mint.** `Catalog::insert` falls back to
  `upsert` (→ `rebuild_postings`, the O(n) rebuild) only for a `game_id` the document
  already holds. `mint::mint_id` (`mint.rs:89-125`) returns `IdentityHeld` when every
  spelling is held, and the `held` probe it is given (`state.rs:442-444`) covers
  `corpus.holds ∪ issued.holds ∪ retired.contains`, so the `game_id` it hands back is
  never one `issued` holds. The per-mint index rebuild is gone in fact, not just in the
  common case.
- **The incremental spelling cannot drift from the whole-document spelling**, which is the
  property this cheaper shape is only admissible with. It is pinned:
  `corpus::tests::the_incremental_document_spelling_is_the_whole_document_spelling`
  (`corpus.rs:574`, `:590`, including the replacement case). I ran it:

  ```text
  $ cargo test --locked -p bindforge-registry --lib incremental
  test corpus::tests::the_incremental_document_spelling_is_the_whole_document_spelling ... ok
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s
  ```

Per novel key the mint now pays: one O(n) document serialization and one O(n) tmp-write +
`sync_all` + rename (`persist.rs:250-296`) under `mutations`. It no longer pays a
records-and-postings deep copy or an O(n) posting re-clone on top. The remaining rewrite
is the risk §Risks:545 takes on by name ("every first-time key rewrites a multi-megabyte
issued document under the mutation lock … batched writes deferred"), mitigated by dedup
(policy step 2, `state.rs:318-328`), the cap, and Rollout 4's measurement — so it is no
longer a deviation from the design; it is the design's accepted residual. Checklist item 1
("no per-event table rebuilds") is satisfied on the request path.

### 2. minor — `STEAM_PULL_DEADLINE_SECONDS` unenforced mid-page — **closed**

The deadline is threaded into the readers: `attempt_page` passes it to
`read_header_block` (`steam.rs:704`, checked each 1 KiB window at `:884`) and to
`read_body` (`:722-727`), whose `fill` calls are both guarded (`read_body:959`, `:976`;
`fill:1015` → `within_pull_deadline:992`), returning `PageFailure::Final(deadline_refusal())`
— the named `steam pull deadline`, not a socket-error refusal. The dribbling peer I
described can no longer hold one page open past the deadline; the read socket timeout is
already `STEAM_READ_TIMEOUT.min(remaining)` (`steam.rs:747`), so a single blocking read
cannot carry the pull much past the bound either. `constants.rs:262-268` now states the
broader meaning, matching. Residual, worth a leg: see finding N2.

### 3. nit — `SteamSnapshot::publish` used `to_vec_pretty` — **closed**

`steam.rs:206` publishes with `serde_json::to_vec`, and the comment records why (the
`MAX_STEAM_SNAPSHOT_OCTETS` gate, the daily write, the later parse). The snapshot document
is now compact on every publish.

## Performance checklist

1. **Event/request path.** No new synchronous fs or network on a query. `027-L1` holds:
   `steam_lookup` (`state.rs:348-407`) reads the `Arc<SteamSnapshot>` slot and the loaded
   documents only; `SteamStore::production` (`steam.rs:530-547`) resolves nothing and dials
   nothing at construction (`main.rs:310-316`), which is an improvement on the previous
   startup that resolved the authority before it loaded anything. The mint's fs is the
   design's persist-before-serve, under `mutations`, not on a reader's lock.
2. **Complexity.** No unbounded quadratic. `to_bytes_with` is O(n) with a per-record
   string compare, dominated by the serialization it feeds; posting insert is
   O(log keys + posting length). `decode_page` is linear over a page; `normalize_store_name`
   (`steam.rs:1065-1071`) now delegates to `discovery::game::friendly_name`
   (`crates/discovery/src/game.rs:102-113`), which I read: `split_whitespace` + `join` +
   boundary clamp, linear — the substitution costs nothing at 50k names per page.
   `pull`'s per-page `collected.iter().max()` (`steam.rs:640`) is a re-scan, bounded by
   `STEAM_REFRESH_MAX_PAGES = 12`.
3. **Unbounded growth.** Bounded, and by named budgets: postings and records by
   `MAX_ISSUED_ENTRIES = 200_000` / `MAX_ISSUED_DOCUMENT_OCTETS`; the page read by
   `MAX_STEAM_PAGE_OCTETS` with the declared-length refusal before the drain
   (`steam.rs:951-956`); the header block by `MAX_RESPONSE_OCTETS`; `dial_addresses`'
   `Vec` is one resolver answer. No cache is introduced, so nothing else can go stale.
4. **Startup.** `startup_snapshot` (`steam.rs:1111-1143`) is load-only on the
   loadable-and-fresh arm and dials nothing there; `--no-steam` never builds a store or a
   `RootCertStore` (`main.rs:303-316`). Boot reads no more documents than before. The r1
   scale leg re-ran without regression (below).
5. **Cache invalidation covered.** The postings index is the only in-memory index; its new
   incremental update path is pinned against the whole-document spelling (quoted above).
6. **Benchmark-like legs — run and quoted.**

   ```text
   $ cargo test --locked -p bindforge-registry --test issued_entries \
       at_the_issued_cap_a_novel_key_is_refused_while_held_keys_answer -- --exact --nocapture
   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 3.88s
   ```

   That is the 200k-record leg in debug (fixture build + whole-document load + postings
   build + the cap refusal + `stats`): 3.88 s, against the 4.09 s I quoted in r1 — no
   regression from the revision. Note what it does **not** measure: at the cap the novel
   key is refused at `state.rs:439` *before* `to_bytes_with`, so no leg in the suite times
   an issued rewrite at scale (finding M1). The runner's gate record's single captured
   `test result: ok. 0 passed; …` line is the last target's tail (a target with no tests in
   it), not a suite that ran nothing; the gate is green on all four legs.

## Findings

### M1 (minor) — the issued-document rewrite Rollout 4 asks for is still not measured anywhere in the reviewed paths

- **Location**: `apps/registry/src/corpus.rs:355-378` and `apps/registry/src/state.rs:461`
  against `02-proposal.md` Rollout item 4 ("the issued-document rewrite measured at 100k
  and 200k records, recorded in §Budgets", proposal:730-731) and §Budgets:535-537, which
  still states the figure prospectively.
- **Problem**: §Deferred defers batched issued writes "against a number", and the mint's
  whole cost at `027-B3`'s cap is now exactly two things — the `to_bytes_with` serialization
  and one `persist::save` (with `sync_all`) — both under `mutations`. No leg produces that
  number: the only 200k-record leg refuses the mint at the cap check before serializing, so
  the suite's 3.88 s is load, not rewrite. I could not establish the figure from my inputs;
  the run's build notes are not an input to this seat, so if the measurement was recorded
  there this item is already closed and only the §Budgets line needs the number.
- **Fix**: land one `#[ignore]`d timing leg that boots against a 100k-record issued document
  and against a 200k-record one and asks one novel key, asserting nothing but printing
  `to_bytes_with` and `persist::save` elapsed times, and carry the two figures into
  §Budgets beside the memory figure — the shape is already there (`issued_entries.rs:355-411`
  builds the fixture); it stops one record short of a mint that serializes.

### N1 (nit) — the per-page DNS resolution sits outside the pull deadline it is now beside

- **Location**: `apps/registry/src/steam.rs:750` and `:793-814`.
- **Problem**: deferring resolution to `connect` is what keeps a fresh-snapshot boot off the
  resolver (Test strategy #17), and I do not want it back. But `to_socket_addrs` is now
  asked once per page *attempt* — up to `12 × 3` per pull — and it is the one blocking
  network act on the pull path with no bound: `connect` checks the deadline at `:743-746`
  and then resolves without a timeout. A host whose resolver is black-holed (rather than
  failing) can spend the resolver's own retry grid 36 times, which is how the named 1,920 s
  guarantee gains an unaccounted tail — pre-bind, that is the bind held longer than
  `constants.rs:262-268` promises. In practice a cached resolver makes it noise; it is a
  boundary the r1 fix moved, not a defect.
- **Fix**: either resolve once per pull and reuse the answer across the page loop's attempts
  (still never at startup), or note in `dial_addresses` that the resolver's own timeouts,
  not `STEAM_PULL_DEADLINE_SECONDS`, bound this leg.

### N2 (nit) — the mid-page deadline guard has no leg

- **Location**: `apps/registry/src/steam.rs:884` and `:1015` (the `within_pull_deadline`
  calls), with `apps/registry/tests/steam.rs:929` as the only deadline leg.
- **Problem**: `a_stalled_pull_is_cut_off_by_its_own_deadline` pins the total-silence peer —
  the case where the socket read timeout would have caught it anyway. Nothing dials a fake
  that dribbles a header block or body under the read timeout, which is precisely the peer
  this revision added the guard for; a future edit that drops one of the two call sites
  stays green.
- **Fix**: one loopback leg with `with_timing(page_backoff, short_deadline)` whose fake
  writes 8 bytes every 50 ms after the headers, asserting the refusal names `deadline` and
  that the elapsed time is the deadline, not a socket grid.

## FYI (no severity, outside this seat)

`.github/workflows/check.yml:66-109` builds `cargo-deny 0.20.2` from source with no cache on
every PR, inside its own 30-minute job — design-mandated by Rollout 3, but a cached scanner
binary or a `paths:` filter would buy the same screen for less wall clock.
