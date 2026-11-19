---
doc_type: task
status: pending
date: 2026-11-19
type: feature
functional_area: backend
associated_adr: ADR-026
depends_on: [T-005-registry-root-serve, T-006-registry-admin-face]
flow: light
---

# T-007: Caching registry mode — per-slug lookup policy and cache document

ADR-026 (`026-C6`, `026-C7`, `026-L4`, `026-L5`, `026-B1`–`026-B3`) defines
the user-hosted mode: a registry that holds only what its user registered
(T-006's face, over a local catalog document) plus a per-slug cache of
upstream lookups. No bulk transfer exists anywhere (`026-L3`).

Land in `apps/registry`:

- `registry serve --upstream <url>` (plain `http://`, validated like the
  CLI's endpoint rule but operator-configured, any host): the serving face
  is byte-identical to root mode (`026-C1` — a client cannot tell modes
  apart) while the answer comes from the lookup policy:
  1. a local entry for the slug answers with no upstream request (`026-L5`);
  2. else a cache entry younger than `--refresh-interval`
     (`REGISTRY_UPSTREAM_TTL_SECONDS = 3_600` default) answers as-is;
  3. else query the upstream's `/v1/candidates?slug=…`, cache `{slug,
     candidates, fetched_at}`, serve; a failed refresh over a stale entry
     serves stale and moves health `upstream` to `"stale"`;
  4. else (miss, upstream unreachable) the named `503` (`026-L4`) — never
     `200 []`.
- Cache document `026-C7`: `<data>/bindforge/registry/cache-v1.json` under
  the `025-C3` data-directory rule, records sorted by `slug`, load refused
  by name outside shape/bounds/`game_id` grammar (an untrusted file),
  `REGISTRY_CACHE_READ_BUDGET_OCTETS = 1_048_576`, eviction over
  `MAX_REGISTRY_CACHE_ENTRIES = 5_000` by oldest `fetched_at`, writes
  through the tmp+rename owner-only discipline.
- `GET /v1/health` reports `mode: "caching"`, `upstream: ok|stale|down`,
  `local_entries`, `cached_slugs`, `last_upstream_fetch`; upstream
  reachability is gauged against the upstream's `/v1/health`.
- `registry query <slug>` runs the policy once and prints the answer with
  provenance (`local | cache | upstream | stale-cache`).

Done when:
- The full policy table is unit-tested against an in-process loopback fake
  upstream (the `tests/common` `FakeCatalog` pattern) covering: local
  beats upstream, TTL-fresh cache answers without an upstream request
  (request-count assertion), stale cache refreshes, refresh-failure serves
  stale, miss+down answers the named 503, and eviction order.
- The cache file round-trips: refusal of hostile shapes (bad version, bad
  slug spelling, over budget, invalid `game_id`) names the file and rewrites
  nothing; a saved cache is byte-stable across processes.
- A subprocess test runs `discovery list` against a caching registry whose
  upstream is the root-mode registry from T-005 (both loopback), proving the
  three-process chain end to end.
- `cargo test --workspace --locked` passes.
