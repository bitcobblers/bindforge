# Code review — performance seat — round 1, pass c0

- Change under review: `git diff a22cfd2694fde8d84be132c8d78faf09fda1adf4...HEAD -- ':(exclude)docs/designs'` (hosted mode, steam snapshot resolver, issued/mint/retired write paths, curation face).
- Inputs used: the diff, `02-proposal.md`, ADR-027, the runner's gate record (green at 2026-11-22T07:10:57Z), and a local run of the project's own 200k-record issued suite leg. No other review reports or transcripts opened.
- Checklist: `references/performance.md`.

## Checklist walk

1. **Request path (no sync fs/net, no per-event rebuilds, no avoidable allocations).** Serving reads comply: `State::candidates` (state.rs:292-334) takes only `RwLock` read halves and probes in-memory BTreeMap postings; the steam probe is one binary search (`SteamSnapshot::app`, steam.rs:201-206); the snapshot read guard is released before `mutations` is taken (state.rs:344-348), so no client waits on a store pull or an fsync of another document. The mint path's file IO on the request thread is `027-C4`'s persist-before-serve, by design. **One violation on the mint path — finding 1.**
2. **Complexity.** No O(n²) on an unbounded dimension. Held-set probes are three BTreeMap/binary searches per mint spelling (state.rs:410-412); `RetiredIds::contains`/`insert` are binary search + ordered Vec insert (≤250k, matches the design's named in-memory shape); the delta publish is exactly the design's one `sort_and_dedup` + linear two-way merge (steam.rs:265-316), never one insert per changed appid; index-build-once rule holds for the snapshot. `SteamStore::pull`'s per-page `collected.iter().max()` cursor scan (steam.rs:539) is O(pages × collected) but bounded by the 12×50k page grid — negligible, not flagged. `read_header_block`'s re-scan is bounded by the 64 KiB header budget — negligible.
3. **Unbounded growth.** Every registry-side collection is capped: issued at 200k records + 64 MiB octets (constants.rs:90,102), retired ids at 32 MiB, snapshot at 32 MiB with full-pull replace, and the in-memory snapshot swaps one `Arc` at a time. Bounded and documented.
4. **Startup.** `Config::build_with` reads exactly what boot needs: corpus (required), issued, retired, snapshot load-or-pull; `--no-steam` never reads the snapshot file. Document caps total the design's 144 MiB boot ceiling. Measured evidence: the suite's own 200k-record startup-load leg passed in **4.09 s** (debug profile, this machine): `test at_the_issued_cap_a_novel_key_is_refused_while_held_keys_answer ... test result: ok. 1 passed; 0 failed ... finished in 4.09s`.
5. **Cache invalidation covered by a test.** Yes: the age-bound posture (fresh → `200 []`/mint, stale → named `503`) is pinned in `hosted_lookup_policy.rs:392-434` and `steam.rs:207-220`; the refresh decision (`next_refresh_seconds` published/failed legs) in `steam.rs:1196-1252`; the delta merge's keep-unchanged/replace-changed semantics in `steam.rs:544-578`; the "held key answers with zero issued-document rewrites" mitigation leg in the end-to-end suite.
6. **Benchmark-like checks.** Ran the 200k leg above and quote it in finding 1. The design's Rollout-4 deliverable ("the issued-document rewrite measured at 100k and 200k records, recorded in §Budgets") has **no artifact in the reviewable diff** — no timing test, no recorded figure in any in-diff note; the only place it could live that I was allowed to see is the excluded `docs/designs` record.

## Findings

### 1. major — every mint deep-clones the whole issued catalog and rebuilds the entire postings index, the one thing §Concurrency rules out, under the global mutation lock

- Location: `apps/registry/src/state.rs:417-423` (`issued.clone()` → `rebuilt.upsert(...)` → `rebuilt.to_bytes()`), landing in `apps/registry/src/corpus.rs:361-365` (`Catalog::upsert` calls `rebuild_postings`, corpus.rs:280-291).
- Problem: the design's §Concurrency sentence is explicit: a writer "computes and serializes the new document straight into the write buffer (**a mint inserts the new key's posting rather than rebuilding the index**)". The code instead (a) deep-clones the whole `Catalog` — both the records map *and* the postings map — and (b) runs `rebuild_postings()` on every mint, walking all records and cloning every `source_key` and `game_id` string into a fresh BTreeMap, which makes the cloned postings copy pure waste. All of it runs inside the `mutations` mutex, which serializes mints and admin mutations across up to 64 request threads. At `027-B3`'s 200k-record cap that is, per *novel* key: a full model clone + a 200k-key index rebuild + a full ~30 MB re-serialization + the fsync'd rename, with several model-sized transients alive at once. The design's §Risks table prices only the accepted core of this (the whole-document rewrite) and names the posting-insert as the mitigation for the in-memory half; the extra clone+rebuild is unpriced and contrary to the named instruction. Evidence for scale: the suite's own 200k leg (which pays one O(n) DOM parse + records/postings build at startup per instance) takes 4.09 s in debug — a mint pays the same O(n) three times over per request, so near the cap each novel-key mint is a multi-hundred-ms-to-seconds hold of the one mutex. `027-S1` makes first-time steam keys the day-one traffic, so this is the ordinary path, not the abuse edge.
- Fix: make the mint's model update incremental — build the rebuilt document without `rebuild_postings` (add a `Catalog` seam that inserts the record and pushes the new key's posting into the existing postings in `game_id` order, since one mint adds exactly one posting), and avoid the deep postings clone when doing so (e.g. mutate the clone's postings in place, or hold the model in an `ArcSwap`-style single-slot swap). Either way, no mint may rebuild the index. Then land the Rollout-4 artifact the design requires: the measured issued rewrite at 100k and 200k records, recorded beside §Budgets' figure, so the deferred batched-write amendment defers against a number.

### 2. minor — `STEAM_PULL_DEADLINE_SECONDS` is not enforced while a page's header/body is being read, so the named deadline guarantee fails open against a dribbling peer

- Location: `apps/registry/src/steam.rs:721-747` (`read_header_block`), `steam.rs:787-857` (`read_body`/`fill`); the only deadline checks are `steam.rs:572-580` (between attempts) and `steam.rs:627-631` (connect), with the socket timeout set once at connect to `STEAM_READ_TIMEOUT.min(remaining)`.
- Problem: the design promises "a pull of either kind that has not published inside `STEAM_PULL_DEADLINE_SECONDS` fails by name as `steam pull deadline`". A peer that keeps the connection alive and dribbles ≤8 KiB per <30 s never trips a socket timeout and never reaches a deadline check: a single page can run on for hours (12 MiB body budget ÷ 8 KiB × ~30 s), and the startup path is pre-bind, so the process's bind is held correspondingly long instead of failing by name at 1,920 s. The suite's stall test (`steam.rs:929-961`) only pins the total-silence leg, where the connect-time socket timeout is what actually saves it.
- Fix: pass the deadline into `read_header_block`/`read_body` (or recompute the per-read socket timeout from `deadline − now` before each `fill`) and return `deadline_refusal()` when it is spent mid-page.

### 3. nit — the snapshot is published with `to_vec_pretty`, inflating the daily write and every later load by roughly half against the same 32 MiB cap

- Location: `apps/registry/src/steam.rs:247`.
- Problem: a 200k-app worldlist pretty-printed costs materially more octets than the compact spelling of the same `027-C3` shape — more write bytes on every daily publish, a larger file for the metadata gate, and a bigger parse at next boot, all inside the same `MAX_STEAM_SNAPSHOT_OCTETS`. The shape on disk is unchanged, so nothing is refused or misread; it is purely a paid-for-whitespace cost.
- Fix: if the pretty spelling is deliberate for operator inspection (the runbook reads the file), say so in one comment line and keep it; otherwise publish with `serde_json::to_vec`.

## FYI (no severity, other seats' territory)

- `load_or_pull_snapshot` (main.rs:368-401) boots on a present-but-*stale* snapshot rather than pulling, a deliberate deviation from the proposal's "load the snapshot if present, loadable, **and fresh**, else pull in full", self-documented as recorded in the build notes' §Deviations; the serving policy converts its steam misses to the named `503`, so no honesty is lost, but it is a deviation the correctness seat may want to weigh.

## Not applicable

- Checklist items with no candidate surface in this diff: keystroke paths (no client-side code changed); orchestration/Node paths (none touched).

## Verdict

**request-changes** — one major (finding 1, a design-instruction violation on the ordinary first-mint path with the design's own required measurement missing from the change), one minor, one nit.
