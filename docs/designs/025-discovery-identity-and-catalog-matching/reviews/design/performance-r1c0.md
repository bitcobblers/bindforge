# performance review r1

verdict: approve

Scope: whole document (`02-proposal.md`), full-panel pass c0. Inputs: the proposal,
`docs/adrs/025-discovery-identity-and-catalog-matching.md`, the Revision log, and
read-only inspection of `crates/discovery` / `apps/discovery-cli` (confirmed: no
`TcpStream`/timeout/network-dependency code exists today — all socket IO in this
design is new surface — and `derive_slug`/`resolve_collisions` exist today and are
retired by the design).

## Findings

- [minor][new] "Catalog client and gate" (`apps/discovery-cli/src/catalog.rs`) — "One GET per distinct unresolved slug — the ADR's singular call shape; batching is `025-R1`'s." with "TcpStream with a 2 s connect and 5 s read timeout", and "Scan flow and output" step 5 — "For the distinct unresolved slugs (those the store does not resolve), in byte order, one `candidates()` call each." — the design budgets every request but nothing at the scan level. The common first-scan case (absent store ⇒ every installed game is unresolved; a Steam library is routinely hundreds of slugs) runs sequentially with one fresh `TcpStream` (`Connection: close`) per slug; a mid-scan failure aborts, but a slow-but-successful loopback peer costs its full latency on every slug, so wall-time is N_unresolved × (resolve + connect + up to 5 s read) with no aggregate deadline and no stated expectation anywhere in Chosen design or Risks. Fix: one sentence at the timeout statement (or one Risks row) quantifying the bound — N_unresolved sequential requests, per-request worst case 7 s, why it is acceptable here (loopback-only peers, fail-closed on the first failure, user-initiated scan, concurrency/batching reserved to `025-R1`) — so the implementation and later reviewers inherit the intended budget instead of inferring it. The sequential shape itself is ADR-mandated (`025-C4`) and is not the finding; the missing scan-level statement is.

- [nit][new] "Catalog client and gate" — "The request is std-only: `TcpStream` with a 2 s connect and 5 s read timeout" — for the `localhost` authority (one of the three accepted hosts), std's connect timeout covers the connect syscall only; address resolution happens before it and is untimed, and as written each of the N requests repeats that resolution. Fix: resolve the authority once at `HttpCatalog` construction and carry the resolved address into every request, replacing per-request host resolution; one clause in the same bullet states the timeout's scope.

## Checklist coverage

1. Input-path latency budget: not applicable as to keystroke/event paths — nothing in this design touches the translation or binding event path. The new synchronous IO is the scan flow above (minor finding). The emit-path changes are net cheaper (`derive_slug`, `is_slug_name` at the boundary, and the `resolve_collisions` pass retire; the new boundary check is a length compare).
2. O(n²) with unbounded n: none found. `resolve_collisions` retires; store save is a sort; step 5 is a sort plus one request per distinct slug. Store membership structures are unspecified but n is install count, not an unbounded product dimension.
3. Memory growth: bounded where the design takes a position — records capped at 128-octet slug, `MAX_SLUG_OCTETS` game_id, `MAX_NATIVE_NAME_OCTETS` friendly name; response parse refuses over-65 536-byte `Content-Length` "before allocation". Store file growth over time is unbounded, but that is `025-L4`'s remember-without-expiry law mandated by the ADR; GC deferral contradicts nothing.
4. Startup cost: one small JSON read (match store) per scan, before enumeration; enumeration itself unchanged and network-free.
5. Caching invalidation: the store is trust-without-expiry by law (`025-L4`, "Records never expire and are re-trusted on every scan"), and the re-listing/re-mint consequence is the ADR's own stated cost, carried as a Low/Low Risks row — correctness of the invalidation policy is ADR-mandated, not a finding.
6. Research baseline: the caller-permitted inputs contain no quantitative baseline (01-research.md was not among them) and the proposal states no expected numbers to compare against. Recorded as missing evidence, non-blocking: every performance property needed to proceed (per-record bounds, body cap, fail-closed per-request timeouts) is establishable from the design text itself; the one expectation worth stating is the minor finding above.

## FYI (no severity)

- Table mode prompts once per distinct unresolved slug on a first scan — for a large library that is an hundreds-of-rounds interactive session; it is exactly what `025-L2` (no silent match) plus no-batching prescribes, but the correctness and complexity seats may want to confirm the acceptance of that shape rather than discover it at build time.
