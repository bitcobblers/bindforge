# Code review — performance seat, round 1, pass c2 (ratification)

- Run: 026-registry-service
- Scope as given: `git diff 06d528febd4bd7d8b875553668b4ebab6a3bd097..HEAD -- ':(exclude)docs/designs'`
  (commits `f01a9394` docs, `a8652755` fix(registry): refuse an admin body declared as a non-JSON media
  type, `8e40cf38` test-update, `714effca` docs — the two code/test commits are the change under review).
- Inputs: that diff; `02-proposal.md` (full current document); `docs/adrs/026-registry-service.md`; the
  runner's gate record (2026-11-19T23:01:03Z, green: `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` exit 0, with
  a green leg's `test result: ok` tail); my round-1 findings.
- Pass type: ratification — I check only that the later fixes did not break what the panel approved in
  round 1, on the performance axis. No re-litigation of unchanged code.

## Verdict

**approve.** No blocker or major finding against this diff. The fixes are performance-positive or
performance-neutral; one nit on new code.

## What the diff changes, performance-side

1. **Header-block framing (`http.rs:328-368`)** — the whole-buffer `find_header_end` rescan after every
   byte is replaced with a one-octet incremental terminator test (`buffer.ends_with(b"\r\n\r\n")`).
   Equivalent by induction (bytes arrive one at a time, so the first occurrence of the terminator is
   always the one completing at the newest byte), and it removes the quadratic term I raised in round 1
   (finding 1): a maximal `MAX_REQUEST_HEADER_OCTETS` preamble now costs one comparison per octet instead
   of a windows(4) rescan from offset 0 (~5e8 window checks removed). The `MAX_REQUEST_HEADER_OCTETS`
   cap check is still performed every iteration, unchanged.
2. **Request split into `RequestHead` + body (`http.rs`, `serve.rs`)** — the admin face is gated on the
   header block alone (enablement, route, bearer, `Origin`, `415` media-type) and a refused admin request's
   declared body is never buffered; it is discarded through `drain_unread_body` (`http.rs:379-399`):
   hard cap `UNREAD_BODY_DRAIN_OCTETS` = 65,536, 4 KiB scratch reads, 2 s read timeout
   (`constants.rs`). Bounded pieces, no buffering, bounded slot hold. This is a strict improvement over
   the approved round-1 shape (where every connection's declared body — up to
   `MAX_REQUEST_BODY_OCTETS` = 16,777,216, budget-checked first — was buffered before routing); the
   non-admin routes keep the approved round-1 behavior (budget check before buffering, §Budgets'
   sanctioned figure, §Risks' low-and-slow row unchanged in kind).
3. **Cache bounds (`cache.rs:141-151, 249-250`)** — a stored entry over `MAX_CANDIDATES` is a load
   refusal; `stage` truncates the staged list. O(1) / O(n) work on already-bounded input; closes a
   growth seam at the write and load boundaries. Positive.
4. **Response octet cap (`corpus.rs:60-110, 466-660`; `serve.rs:294-305`)** — `GET /v1/candidates`
   answers the largest prefix of the stable `game_id` window fitting `MAX_RESPONSE_OCTETS`. Linear in
   min(n, 256); `MAX_CANDIDATES` max-bound records ≈72,000 octets, the crossover near 231 candidates,
   are pinned by `corpus.rs` and `tests/candidates_join.rs` unit and subprocess legs (the arithmetic
   matches my own ≈283-octets-per-candidate estimate). Non-empty joins always keep ≥1 candidate, so this
   cannot turn a 200 into an empty answer. Nit below on the double serialization.
5. **Durability (`persist.rs:209-226`, `schemas/catalog/v1/spec.md`)** — `save` now `sync_all`s the tmp
   before the rename. This puts a synchronous fs on the write path: every admin mutation and every
   cache-miss `cache_lookup` (state.rs:380-395) holds the one write lock across serialize → write →
   fsync → rename. Checklist item 1 would flag this, but it is **design-sanctioned**: §Concurrency
   states the mutation serializes, writes the tmp, renames and swaps "all under the one write lock, …
   A reader may wait on one local write, and that latency is accepted", and the spec.md in this diff
   amends the atomic-persist discipline to spell the `sync_all` before the rename. Marked N/A by
   sanctioned deviation; `tests/persist_and_concurrency.rs::every_accepted_mutation_is_answered_only_after_the_document_holds_it`
   pins the ordering. No finding.
6. **Startup (`state.rs:170-186`, `corpus.rs:168-196`)** — `--corpus` now refused at start when absent
   (`load_required`). Startup-only refusal path; no boot-time work added. Positive/neutral.
7. **CLI `--flag=value` (`main.rs:129-190`)**, **`keys.rs` `Source::spells_lookup_key`** (one extra
   bounded scan of a ≤`MAX_SLUG_OCTETS` string) — negligible.
8. **Tests** — no sleeps added beyond the pre-existing pattern; the two new drain-shaped legs cost the
   2 s drain timeout each, bounded; the refresh-interval legs were moved off the racy 1 s budget. The
   runner's gate is green; the whole `cargo test --workspace --locked` leg passed in exit 0.

## Round-1 finding status (carried, for the record — none is a regression)

- **#1 (minor, http.rs framing)**: the quadratic terminator rescan is **fixed** as prescribed
  (incremental). Residuals from the same finding are unchanged, not worse: still one `read(1)` syscall
  per octet (`http.rs:330-348`) and the O(h²) duplicate-header scan (`http.rs:217-225`); both are now
  linear/quasi-linear against a 64 KiB cap rather than quadratic, i.e. strictly better than what I
  approved.
- **#2 (minor, cache.rs eviction loop)**: **unchanged** — `stage` still re-serializes the whole document
  per dropped entry and `drop_oldest` re-runs a full `min_by` scan (`cache.rs:258-266, 305-317`), under
  the cache write lock. Not worsened by the diff (the new truncate bounds entry size; the sanctioned
  fsync joins the already-counted save section of the lock hold).
- **#3 (minor, to_string_pretty)**: **unchanged** — `serialize` is still pretty (`cache.rs:282-301`);
  the spec.md diff documents the response-side arithmetic but does not restate the cache-capacity fit,
  so the §Budgets "026-B1's 5,000 binds ordinary answers" sentence still reads against ~1.5× pretty
  inflation as before. Same open state as approved.
- **#4 (nit, state.rs double clone)**: **unchanged** — two `cache.read()` + `.cloned()` on the fetch
  path (`state.rs:259-284`) plus the third read lock in `maybe_probe` (`state.rs:337`). Same open state
  as approved.

## Checklist disposition

1. Event path — no new synchronous fs outside the design-sanctioned persist discipline (item 5 above);
   the `candidates_answer` seam adds bounded work (nit below).
2. Complexity — no new O(n²); the diff removes one (header framing) and all new loops are bounded by
   64 KiB / 4 KiB scratch / 256 candidates / 65,536-octet discard.
3. Unbounded growth — improved (cache entry candidate cap at stage and load; response body capped;
   drain capped).
4. Startup — corpus-existence check only.
5. Cache invalidation — logic and its tests unchanged by this diff (fresh/stale/TTL legs untouched in
   substance; two legs re-timed for honesty).
6. Benchmark-like checks — the octet-crossover arithmetic is pinned by asserting tests
   (`corpus.rs`, `candidates_join.rs`, `wire.rs`), quoted by the runner's green
   `cargo test --workspace --locked` leg (exit 0); no separate timing note to run.

## Findings

### nit — `apps/registry/src/corpus.rs:60-110, 96-110` (`Candidate::response_octets` / `candidates_body`)

The response seam serializes the candidate window twice per request: `response_bounded` runs
`serde_json::to_vec` per candidate to measure it, then `candidates_body` re-projects the kept prefix to
`Value`s and serializes again. On the hot `GET /v1/candidates` path this doubles the per-candidate
allocation and serialization work (up to ~256 throwaway `Vec<u8>` allocations and a second array
serialization on a max-width answer). Bounded and small — hence a nit — but avoidable: serialize each
candidate's bytes once and sum those lengths while concatenating the kept prefix into the final array
buffer, or cache per-entry serialized length at index/build time. No behavior change intended; the
trim's stability and byte-identity are already pinned by the new tests.

## FYI (no severity, out of this pass's incremental scope)

The query-pair de-dup at `http.rs:275-277` (`query.iter().any` per pair over an attacker-sized request
line) is the same O(h²)-over-attacker-input family as my round-1 finding-1 residual; it is unchanged by
this diff and so not a finding here.
