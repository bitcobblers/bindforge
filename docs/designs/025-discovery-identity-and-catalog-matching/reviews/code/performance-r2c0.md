# Code review — performance seat — round 2, pass c0

- Range: `git diff 7b036617dfa2d7895888b73c9b9412387b1d7e08...HEAD -- ':(exclude)docs/designs'`
- Design: `docs/designs/025-discovery-identity-and-catalog-matching/02-proposal.md`; ADR-025.
- Gate record (runner, 2026-11-19T03:30:34Z): all four legs green —
  `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked` all exit 0.
- Verdict: **approve** (zero blockers, zero majors; two nits).

## Checklist walk

1. **Event/keystroke path** — N/A in its strict form: this diff is a
   one-shot CLI scan, not an interactive path. No synchronous fs/network was
   added anywhere that runs per keystroke or per event; the fs and socket work
   is confined to the single scan flow the design mandates (gate → store load
   → enumerate → one query per distinct unresolved slug → render). Net of the
   retired `derive_slug` per-row folding and the retired `resolve_collisions`
   pass (`crates/discovery/src/steam_common.rs`, deletions at the old
   lines 260–350), the per-row emission cost went **down**: the row gate is
   now a single `slug.len()` comparison
   (`crates/discovery/src/steam_common.rs`, `discovered_game`, new lines
   ~245–260).
2. **Complexity** — no unbounded O(n²). The removed collision pass was the
   only super-linear pass over rows and it is gone. The store's per-row
   lookups are linear but bounded (see finding N2). `candidates_by_slug` and
   the resolved-slug snapshot are `BTreeMap`/`BTreeSet` — maps/sets where
   keyed lookup was intended (`apps/discovery-cli/src/main.rs:162–186`).
   One query per distinct unresolved slug, byte order, exactly the design's
   singular call shape; batching is explicitly deferred (`025-R1`).
3. **Unbounded growth** — every new container is bounded or documented:
   store file capped at 65,536 octets (`matches.rs:36`, enforced twice at
   `matches.rs:194–211`); response header block and body capped at 65,536
   (`catalog.rs:23`, enforced at `catalog.rs:299–303` and `353–357`) and
   candidate count at 256 (`catalog.rs:26`, `catalog.rs:380–385`); the body
   is only allocated after the `Content-Length` passes the cap
   (`catalog.rs:364`), so allocation follows the budget, not the peer.
   Store records never expire — that is `025-L4` by design, with GC
   explicitly deferred in the proposal. Documented, not a finding.
4. **Startup** — the run reads exactly what it needs when it needs it: the
   store is loaded once, after the gate and before enumeration
   (`main.rs:110–120`); no new boot-time parsing; plugin enumeration is
   unchanged. The `probe()` connect-before-enumeration is `025-L1`'s required
   ordering, not incidental work.
5. **Cache invalidation tested** — the match store is the run's only
   cache-like state and its invalidation rules are pinned: a slug the store
   resolves draws zero catalog requests (`cli_match_store_tests.rs:188–217`
   `a_valid_store_resolves_its_slugs_without_queries_or_rewrites`,
   `cli_confirmation_flow_tests.rs:215–254` where scan two adds no request
   and rewrites no byte, and the JSON-mode half at
   `cli_json_mode_tests.rs:191–194`); exactly one query per distinct
   unresolved slug (`cli_confirmation_flow_tests.rs:280–285`); a confirmed
   slug is prompted once for all rows sharing it (single record per slug,
   `matches.rs:281–300`).
6. **Benchmark-like test or timing note** — none exists in the diff (the
   only `Duration`s are the 2 s connect / 5 s read socket timeouts at
   `catalog.rs:29–30`, which are hang guards, not timing assertions).
   Nothing to run; the runner's green `cargo test --workspace --locked`
   leg (exit 0) covers the suites cited above.

## Findings

### N1 (nit) — `apps/discovery-cli/src/catalog.rs:283–304`: header block read one byte per syscall

`read_response` pulls the header block from the raw `TcpStream` with a
`[0u8; 1]` buffer, so every header byte is its own `recv` syscall — roughly
150–300 syscalls for a typical small response, and up to the 65,536-octet
budget before the over-budget refusal fires. Impact is small (one-shot CLI,
loopback-only authority by policy, bounded per response), so this is a nit,
not a defect that blocks. **Fix:** wrap the stream in a `std::io::BufReader`
at `catalog.rs:444` before calling `read_response` (the `Vec<u8>` scan for
`\r\n\r\n` then amortizes on buffer refills); the body `read_exact` already
reads in bulk and needs no change.

### N2 (nit) — `apps/discovery-cli/src/matches.rs:249–261,288`: slug identity lookup is a linear scan

The store is explicitly "keyed by `slug` alone" (module doc, `matches.rs:16`)
and `MatchStore::game_id`/`resolves`/`confirm` find records with
`records.iter().find(...)`, so `main.rs:162–174` costs
O(rows × records) per scan. Today that is harmless — the 65,536-octet read
budget bounds `records` to roughly a thousand entries and rows are
per-machine — and the proposal deliberately ships no dedup/index logic. Flag
only because the cheap fix is nearly free: `records` is already maintained in
`slug` byte order by `sort_records` (`matches.rs:358–360`), so
`binary_search_by_key` turns the identity lookup into O(log n) with no new
structure. **Fix:** replace the `find` in `game_id` and `confirm` with a
binary search over the sorted vec, or index once into a `BTreeMap<String,
usize>` after load. Not required this run.

## Out-of-specialty FYI (no severity)

`HttpCatalog::probe` (`catalog.rs:255–271`) carries a load-bearing
`addrs.len() == 1 || !is_connect_refusal(&error)` precedence for which
resolved address gets its connect tried and when the refusal fires; whether
that reads as the intended refusal semantics is a correctness-seat question,
not a performance one — the probe makes at most one successful connect, so
its performance surface is nil either way.

## Conclusion

No blocker or major findings. The change retires more per-row work
(`derive_slug`, the collision pass) than it adds, bounds every new
allocation against a named budget, and its cache rules are test-pinned; the
two nits are micro-efficiency polish on a loopback-only, one-shot path.
**Approve.**
