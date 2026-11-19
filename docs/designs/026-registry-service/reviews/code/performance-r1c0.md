# Code review — performance seat, round 1 pass c0

Change under review: `git diff 217c46fda9eb95230c8df62233eca8ef7b6ffb71...HEAD -- ':(exclude)docs/designs'`
Design inputs: `docs/designs/026-registry-service/02-proposal.md`, `docs/adrs/026-registry-service.md`.
Gate record (runner, 2026-11-19T21:53:05Z): all four legs green — `npm run check`, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` (`test result: ok. 0 passed; 0 failed; …` is the tail of one captured leg).

## Checklist walkthrough

1. **Event/request path — no synchronous fs/network, no per-event rebuilds, no avoidable allocations.**
   Root-mode serving reads only the in-memory `Catalog` under a shared lock (`state.rs:224-235`); the index is built once at load and rebuilt only on admin mutations — no per-request table rebuilds. The caching path's per-lookup upstream fetch and write-through cache save are the design's own prescriptions (§Caching mode, §Budgets: "The file is rewritten write-through after each lookup that adds an entry"), and the network round trip correctly happens *outside* the cache write lock (`state.rs:279-289` vs `cache_lookup` at `374-389`). Avoidable allocations: see finding 4 (nit).
2. **Complexity — O(n²) where n is unbounded; maps for identity lookup.** The join is one `BTreeMap` probe plus a clone of ≤ `MAX_CANDIDATES` postings (`corpus.rs:218-231`); cache entries are a slug-keyed `BTreeMap`; no linear scans where a map was intended. One genuine quadratic-on-attacker-influenced-input site: finding 1. The eviction re-serialize loop is finding 2.
3. **Unbounded growth.** Cache is capped on both axes by the write-time dual caps (`cache.rs:237-244`, `constants.rs:37/41`); threads are capped by the slot ceiling with an over-cap-close at accept (`serve.rs:84-90`) and a release-on-Drop guard; the gauge is fixed-size; postings are bounded by `MAX_CATALOG_DOCUMENT_OCTETS`. Bounded, and each bound is documented in `constants.rs`.
4. **Startup.** Boot reads exactly what boot needs: the corpus and (in caching mode) the cache document, each size-gated by `fs::metadata` before the read (`persist.rs:236-252`, A22 discipline); the local-entries document is *not* created at startup, only on first write (`main.rs:196-218`). No eager upstream contact (`Upstream::new` is a parse, `upstream.rs:39-47`).
5. **Cache invalidation covered by a test.** Fresh/stale TTL arithmetic (`cache.rs:392-404`), entry-cap and octet-cap eviction (`cache.rs:340-389`, `tests/cache_document.rs:361/427`), clock-jump guard (`cache.rs:392`, `tests/cache_document.rs:507`), refresh-interval re-timing of the once-per-TTL probe (`tests/caching_policy.rs:326`), and "a health read never fires the probe" (`tests/upstream_client.rs:314-355`, which pins five cold lookups to exactly one `/v1/health` probe + five candidate requests — the round-trip amortization the §Risks row names). Covered.
6. **Benchmark-like test or timing note.** No wall-clock benchmark exists in the diff (the `Instant` uses in `tests/common/mod.rs` and `src/wire.rs` are readiness/deadline waits, not measurements); nothing to run. The probe-amortization pin above is the closest analogue and is counted under item 5. Not applicable beyond that.

## Findings

### 1. (minor) Request-header reader rescans the whole block per byte — quadratic in an attacker-chosen length up to 65,536

`apps/registry/src/http.rs:262-305` (`read_header_block` / `find_header_end`). The loop reads one byte per `stream.read()` and, after every byte, calls `find_header_end(&buffer)`, which runs `bytes.windows(4).position(…)` from offset 0. Total scan work is O(n²/8) window checks for a header block of n octets; n is bounded by `MAX_REQUEST_HEADER_OCTETS` = 65,536, but that bound is attacker-chosen per connection: a single request whose preamble places `\r\n\r\n` only at (or past) the cap costs ≈ 5×10⁸ window checks — roughly seconds of one core — plus 65,536 one-byte syscalls. With `MAX_CONCURRENT_CONNECTIONS` = 64, a low-and-slow sender can keep up to 64 threads spinning quadratically; §Risks bounds the *octets* of low-and-slow exposure but not this scan amplification.

Fix: make the terminator check incremental — after appending byte i, test only `buffer[i-3..=i]` (carry the match state across appends) instead of rescanning from 0; and read with a small buffered chunk (e.g. `read` into a 1–4 KiB scratch buffer, scanning only the newly appended tail) instead of one `read` per byte. Same family, same loop: the duplicated-header check at `http.rs:147-155` is O(h²) in header count over the same attacker-chosen block; an insertion into the existing `Vec` kept sorted, or a small `BTreeSet` of names, removes it.

### 2. (minor) Cache octet-eviction can re-serialize and min-scan the whole document hundreds of times per single lookup, under the cache write lock

`apps/registry/src/cache.rs:237-244` (`stage`) with `drop_oldest` (`cache.rs:284-296`), held under `self.cache.write()` for the whole stage→save→commit (`state.rs:374-389`). Each drop does an O(n) `min_by` scan and a full re-serialization of the staged document. The proposal sanctions "one extra pass per eviction", and each pass is bounded by `MAX_CACHE_DOCUMENT_OCTETS`; but a single staging can trigger many evictions at once: with the document sitting at the 1 MiB cap (the steady state write-through drives it there), one inserted entry at `MAX_CANDIDATES` ≈ 23 KiB can require ~150–300 drops, i.e. hundreds of full ~1 MiB serializations plus hundreds of 5,000-entry scans, all while every other lookup blocks on the cache read lock. The unit test `a_write_past_the_octet_cap_drops_oldest_until_the_buffer_fits` (`cache.rs:363-389`) already exercises ~40 serializations in one call — the loop's cost is not hypothetical.

Fix: keep the exact-buffer octet test, but find the eviction cut without full passes — measure each entry's serialized length once (or subtract incrementally as entries drop), evict oldest until the estimate is under the cap with headroom, then serialize once and only loop if that single check still fails. `drop_oldest`'s scan can ride along since entries staged from a `BTreeMap` can keep a running oldest-pointer instead of a per-drop `min_by`.

### 3. (minor) Pretty-printed cache serialization inflates the document ~2×, so `026-B2` evicts ordinary corpora before `026-B1` — contrary to the proposal's stated capacity arithmetic

`apps/registry/src/cache.rs:261-280` serializes with `to_string_pretty`. §Budgets argues "`026-B1`'s 5,000 is the cap that binds ordinary answers, `026-B2` only a corpus answering at `MAX_CANDIDATES`" using ~150-octet entries. A pretty-printed one-candidate entry measures ~250 octets (the shape is asserted in `cache.rs:325-337`), so ~4,000–4,500 ordinary entries already exceed 1,048,576 octets and `026-B2` starts evicting well before the 5,000-slug cap — the effective steady-state capacity of an ordinary cache drops by roughly a third versus the proposal's number, and every eviction additionally pays finding 2's re-serialize loop.

Fix: serialize the cache compactly (`serde_json::to_vec`), keeping the byte-order and cap semantics identical; if human-legibility is genuinely wanted, record in `spec.md` that `026-B1` is *not* the binding cap for ordinary corpora under the pretty spelling, and restate the fit arithmetic.

### 4. (nit) Cold lookup clones the same cache entry twice and takes the cache read lock three times

`apps/registry/src/state.rs:253-278` looks the slug up with `cache.entry(slug).cloned()` twice (step 2 for the fresh test, step 3 for the stale fallback) — on the fetch path that is two full `Vec<Candidate>` clones (up to 256 Candidates each) of the same entry — and `maybe_probe` (`state.rs:331`) takes the cache read lock a third time just for `is_empty()`. Fix: one read lock, one lookup, test freshness from the single borrowed/cloned entry, and pass the emptiness (or the gauge-due decision) from the same critical section.

## FYI (out of specialty, no severity)

- `http.rs:255` allocates the full declared `Content-Length` (up to the 16 MiB `MAX_REQUEST_BODY_OCTETS`) before any body octet arrives; 64 admitted connections × slow bodies = ~1 GiB resident. Framing-consistent with the proposal's declared-length check, but the security seat may want a look.
- The proposal states this run ships "no product code" (§Context/Goals), yet the branch carries the whole `apps/registry` crate plus the CLI-adjacent suites; the performance read here treated the code as the change under review per the caller's scope. The process/scope question belongs to another seat.

## Verdict

No blocker or major findings; four bounded items, all fixable locally. **approve**.
