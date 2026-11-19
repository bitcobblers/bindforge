---
doc_type: adr
status: accepted
date: 2026-11-19
depends-on: [013, 022, 025]
ledger: yes
---

# ADR 026: Serve the catalog from a public root registry, with caching user-hosted registries

This ADR fills the slot `025-R1` reserved: the catalog service itself — its
REST surface, its data, and the curation intake — is decided here for the
catalog half; its profile upload/download/sharing half is not, and is
reserved forward (`026-R2`). It amends the provisional endpoint admission
stated at the design targets of `025-C4` and `025-L1` — the loopback-only
rule, marked there as this ADR's to replace — and both units gain their
`amended-by` per `021-L4` (Rollout, item 6), while the wire contract
`025-C4` pinned (the request target, the response shape, the response
budgets) and the connectivity gate itself stand untouched. ADR-025 shipped
the client and stated, as its Rollout item 6, that the run ships no catalog
server; the CLI's refusal line names this ADR as the owner of that
transport. This run discharges that reservation: it ships the contract, the
ledger annotations, and — a recorded over-delivery the operator has
accepted, Rollout item 7 — the service itself: the registry application,
the catalog contract and seed, and the CLI amendment. The task-level split
planned at authoring time (`T-004`…`T-008`, per the ADR-023 decoupling of
intent from implementation) was never exercised; those task sources retire
against this branch.

**Terminology.** The *catalog* is the data: the curated set of games and
their source-key join facts. The *registry* is the service that serves a
catalog: a **root registry** holds the canonical catalog; a **caching
registry** holds its own registered entries plus a per-slug cache of what it
has fetched from an upstream registry. The already-shipped spellings that
name the data (`/v1/candidates`, `BINDFORGE_CATALOG_URL`) do not rename.

## Context

ADR-025 landed the discovery client whole and the server absent. Every
`discovery list` run today must pass the `025-L1` gate against a loopback
endpoint that, in practice, nothing serves: ADR-025's own Rollout item 6
recorded the deferral, and the only catalog a scan can reach is a test-fake
`TcpListener`. The provisional policy made sense while no service existed —
refusing everything beyond loopback meant a user could not be pointed at a
half-built thing by a flag — but it now forecloses the use the client was
built for: resolving discovered `steam/<appid>` keys to curated game ids
against a real catalog.

The catalog's shape is set by what it serves. Each game is curated knowledge
— a product game id (the manifest `id`, `013-L5`), a display name, and the
source keys (`025-C1` lookup keys) the stores maintain for it — and the
public root registry's catalog will grow with the community: tens of
thousands of games, each with several store keys. That size rules out the
naive hosting story: a user-hosted registry must not download the whole
catalog to be useful, because the whole catalog is large, it changes
constantly, and a per-user copy of all of it is mostly entries the user will
never query. What a user's scans actually ask is one slug at a time — the
candidate question `025-C4` already pins — so the registry's wire contract
should be exactly that question, served by a join the registry runs against
catalog data it holds locally: the root's canonical corpus, or a caching
registry's own registrations plus the handful of lookups it has pulled from
upstream. Explicit registration also answers the sources no public root can
ever curate: a `local/<dir>` install has a lookup key but no store-maintained
identity outside the user's own machine.

## Decision

We will ship one registry service with two modes — a **root** that serves
its canonical catalog corpus, and a **caching** mode that holds only what the
user registered plus a per-slug cache of upstream lookups — and pin its wire
surface to the per-slug candidate question `025-C4` already fixed, plus a
health route and a token-or-loopback-gated admin surface. No surface ever
transfers a whole catalog. The CLI's endpoint policy becomes what it always
provisionally meant: an explicitly configured `http://` endpoint, any host;
the connectivity gate, the response budgets, and the fail-closed discipline
carry over unchanged.

### Contracts

- **026-C1** The service: one binary, `registry` (new package
  `apps/registry`, std-only for its serving face), with `serve` and `query`
  subcommands. `registry serve --root --corpus <path>` runs the root mode
  over a catalog document; `registry serve --upstream <url>` runs caching
  mode. `serve` binds `--host` (default `127.0.0.1`) and `--port` (default
  ephemeral, printed at start). The serving face (`026-C2`) is byte-identical
  between modes; a client cannot tell which mode answered. → Rollout, item 2.
- **026-C2** The serving face: `GET /v1/candidates?slug=<percent-encoded>`,
  answered with exactly the response shape and budgets the shipped client
  pins (`200`, one decimal `Content-Length`, no `Transfer-Encoding`, body
  `[{game_id, name}]` within `025`'s caps), and `GET /v1/health` answering
  `200` with `{status, mode, upstream, local_entries, cached_slugs,
  last_upstream_fetch}`, where `upstream` is `none` (root mode), `ok`,
  `stale`, or `down` (caching mode). Any other route, and a malformed slug,
  is a `404`; a caching registry with nothing to answer from is the named
  `503` of `026-L4`. → Rollout, item 2.
- **026-C3** The catalog document: `{version: 1, revision, games: [
  {game_id, name, source_keys}]}` — `game_id` is a manifest id (shared slug
  grammar and bound, `013-L5`); `name` is curated display text bounded by
  `MAX_NATIVE_NAME_OCTETS` (`013-B4`); `source_keys` are `025-C1` lookup
  keys, each within `025-B1`; `revision` is a non-negative integer bumped by
  every mutation, carried for operator legibility and never used as a sync
  cursor. One spec, `schemas/catalog/v1/`; the root's corpus file, a caching
  registry's local entries file, and this ADR's seed are all instances.
  Loader refusals are the schema's, by name, as every contract document's
  are (`schemas/README.md` discipline). → Rollout, item 1.
- **026-C4** The admin face, present in both modes and mutating that
  instance's own catalog document: `POST /v1/admin/games` upserts one whole
  game record (`{game_id, name, source_keys}`), `DELETE
  /v1/admin/games/<game_id>` removes one, and `GET /v1/admin/stats` answers
  `{games, source_keys, revision}`. Every mutation validates against
  `026-C3` and refuses by name; every accepted mutation bumps `revision` and
  persists atomically through the tmp-and-rename discipline `matches.rs`
  establishes. A `405` names any method the route does not carry. → Rollout,
  item 3.
- **026-C5** The admin enablement and auth rule: the admin face is enabled
  when the listener binds loopback-only (a user's own daemon answers its own
  machine with no token friction) or when a bearer token is configured
  (`--admin-token` or `BINDFORGE_REGISTRY_ADMIN_TOKEN`) — a token is required
  for any non-loopback bind, which is the public root's posture. A present
  token must be answered with `Authorization: Bearer <token>` under a
  constant-time comparison. Disabled admin routes are `404`; enabled ones
  answer `401` for a missing or wrong token. → Rollout, item 3.
- **026-C6** The caching lookup policy, per queried slug: a local entry
  answers without any upstream request; else a cached entry within the
  `026-B3` freshness TTL answers as-is; else the caching registry queries the
  upstream's `/v1/candidates` for that slug and caches the answer with its
  fetch time — a failed refresh over a stale entry answers stale and moves
  `health.upstream` to `stale`; a miss with no reachable upstream is the
  named `503` of `026-L4`. Upstream reachability is gauged by the upstream's
  own `/v1/health`. `registry query <slug>` runs exactly this policy once,
  printing the answer and its provenance, for the operator. → Rollout, item 4.
- **026-C7** The cache document: `<data>/bindforge/registry/cache-v1.json`,
  where `<data>` is `025-C3`'s data-directory rule, holding `{version,
  entries: [{slug, candidates, fetched_at}]}` sorted by `slug` byte order
  under the same owner-only, tmp-and-rename, exclusive-create discipline as
  the match store; it is untrusted input at load and refused by name
  outside its shape, bounds, and the `game_id` grammar `025-C3` checks. An
  absent cache file is an empty cache, not an error. → Rollout, item 4.

### Laws

- **026-L1** The endpoint policy: the CLI's catalog endpoint comes from
  `--catalog-url` or `BINDFORGE_CATALOG_URL` and from nothing else, and the
  admitted endpoint is a literal-`http://` URL with a parseable host and
  port — any host, loopback or remote. The loopback-only admission and the
  refuse-non-loopback-resolution rule retire; the gate itself does not:
  `probe()` still resolves and connects before any plugin is constructed and
  fails closed with its named line (`025-L1` survives with its loopback half
  amended). Amends `025-C4` and `025-L1`. → Rollout, item 5.
- **026-L2** The disclosure boundary: a scan sends every queried source key
  to the configured endpoint, and the user's choice of that endpoint — the
  public root, their own caching registry, a local test — *is* the consent
  act; there is no other disclosure channel. A caching registry's own
  upstream traffic carries only the lookups its clients asked for, never
  client identity and never the bulk catalog. → Rollout, item 5.
- **026-L3** No bulk transfer: no serving or admin route ever answers with a
  whole catalog document, in either mode; wire traffic is per-slug candidate
  lists within `025`'s response budgets, health and stats summaries, and
  single-record mutations. The root's corpus is read from disk by its own
  operator and travels only as far as its own process. → Rollout, item 2.
- **026-L4** Cold fails closed, never empty: a caching registry that holds
  neither a local entry nor a cache entry for a slug, with no reachable
  upstream, answers the named `503` — it never answers `[]`, because an
  empty candidate list states "the catalog has never heard of this key" and
  a lookup that could not be made is not that statement. The CLI's
  fail-closed handling of a non-`200` mid-scan (`025-C4`) carries the
  refusal onward unchanged. → Rollout, item 4.
- **026-L5** Local authority and staleness: a locally registered entry
  answers for its slug ahead of any upstream knowledge, is never merged with
  upstream candidates, and never expires (`025-L4`'s remembered-trust
  posture, held for curation); a cached upstream entry that has passed its
  TTL is refreshed, and serving it stale is admissible only when the refresh
  itself failed, where `health.upstream` names the staleness. → Rollout,
  item 4.
- **026-L6** The corpus boundary carries forward unchanged: the only
  identifiers that leave a registry on the wire are a queried source key
  (outbound, to the configured endpoint) and answers of `{game_id, name}`;
  friendly names and install paths never cross the registry seam
  (`025-L5`). → Rollout, item 2.

### Budgets

- **026-B1** `MAX_REGISTRY_CACHE_ENTRIES = 5_000` — cached slugs per caching
  registry; over the cap the oldest `fetched_at` is evicted. Candidate and
  per-slug response budgets are `025`'s (`MAX_RESPONSE_OCTETS`,
  `MAX_CANDIDATES`) and are not restated. → Rollout, item 4.
- **026-B2** `REGISTRY_CACHE_READ_BUDGET_OCTETS = 1_048_576` — the cache
  document's load ceiling, past which the file is refused by name like the
  match store's. → Rollout, item 4.
- **026-B3** `REGISTRY_UPSTREAM_TTL_SECONDS = 3_600` — default upstream
  freshness, overridable by `--refresh-interval` at the operator's call. →
  Rollout, item 4.

### Reservations

- **026-R1** TLS and any transport assurance: the v1 service and client
  speak plain HTTP, so an admin token or a queried source key over a public
  link is link-visible; public deployments may terminate TLS at an edge in
  front of the service, which this ADR neither requires nor models. Owner:
  `unassigned` — a rollout decision when the public root needs the
  assurance, not a slot this ADR can price. → Rollout, item 7.
- **026-R2** The rest of `025-R1` this ADR does not fill: profile
  upload/download/sharing, curation intake beyond the admin face, and auth
  beyond the single bearer token. Owner: ADR-027 (reserved). → ADR-027.
- **026-R3** Upstream prefetch and parallel hydration: a cold caching
  registry answers a first scan of N games with N serial upstream lookups;
  batching or prefetching is deferred work. Owner: `unassigned` — a rollout
  task, not an ADR slot. → Rollout, item 4.

### Seeds

- **026-S1** The seed catalog: `data/catalog/v1/catalog.json`, a
  `026-C3`-valid corpus opened with a handful of real games (game ids drawn
  from the shipped manifest ids, each carrying its `steam/<appid>` source
  key), so the root serves and the caching mode demonstrably joins on day
  one. → Rollout, item 1.

**Unit handles.** The handles in this ADR sit at the end of the Rollout list
items: one trailing code span of unit ids governs the line's claim, an id
anywhere else governs nothing.

Rollout:

1. The catalog document lands: `schemas/catalog/v1/` (format spec and
   fixtures, valid and invalid), the seed corpus at
   `data/catalog/v1/catalog.json`, and the validating loader. Items 1–5
   were planned at authoring time as implementation tasks beneath this ADR
   (`T-004`…`T-008`, per ADR-023); item 7 records that this run's build
   landed them all on this branch and the task sources retire.
   `026-C3 026-S1`
2. The root's serving face lands in `apps/registry`: `serve --root` answers
   `/v1/candidates` as the join over the loaded corpus, `/v1/health`, and
   the named `404`s, under the no-bulk and corpus-boundary laws.
   `026-C1 026-C2 026-L3 026-L6`
3. The admin face lands: the enablement and auth rule, the upsert and delete
   mutations with validating refusals, the atomic persist, and stats.
   `026-C4 026-C5`
4. The caching mode lands: the lookup policy with its provenance, the cache
   document and its budgets and eviction, the TTL, the cold `503`, and the
   `query` subcommand; prefetch stays reserved.
   `026-C6 026-C7 026-L4 026-L5 026-B1 026-B2 026-B3 026-R3`
5. The CLI amendment lands in `apps/discovery-cli`: the endpoint policy
   admits any literal-`http://` host, `probe()` drops its loopback-resolution
   refusal and keeps the fail-closed connect, and the suites that pin the
   loopback-only refusals (`catalog.rs` units, the gate suite's policy
   section) re-pin to the admitted spelling. `026-L1 026-L2`
6. The ledger annotations land: `025-C4` and `025-L1` gain their
   `amended-by` lines naming `026-L1` per `021-L4`, and `025-R1` records
   this ADR as filling its slot with the profile half reserved forward to
   `026-R2`. `026-L1 026-R2`
7. The over-delivery is recorded: the Rollout planned at authoring time
   shipped the contract and these annotations with no product code, the
   run's build instead landed the registry application, the catalog
   contract and seed, and the CLI amendment of items 1–5 on this branch;
   the operator has accepted the green, reviewed implementation as this
   ADR's reference implementation and retired the five task sources rather
   than delete working code to match a plan; the transport assurance stays
   reserved. `026-R1`

## Alternatives considered

**A bulk sync surface** (`GET /v1/catalog` with a revision cursor, caching
registries mirroring the root's document): rejected outright. The public
root's catalog will be large and always growing; downloading it is exactly
the traffic this ADR's hosting model exists to avoid. A cache of the
slugs actually queried, plus the entries the user registered, holds
everything a user's scans can answer to, and holds it in kilobytes.

**Keep the loopback-only endpoint policy** and make the caching daemon the
mandatory local hop: rejected — it makes running a daemon a requirement to
reach the public catalog, and the user's stated posture is that the daemon
is optional. A user who trusts the public root points the CLI at it. The
policy was ADR-025's provisional half by its own words; amending it here is
the plan, not a deviation.

**Ship a stand-in catalog server for internal testing** (a small binary that
synthesizes candidates for any queried slug): rejected as the answer to
this ADR's gap — it papers over `025-R1` instead of discharging it, and the
in-process fakes in `tests/common/` already prove the client's behavior.
The internal-testing gap is the missing service, so the service is what
ships.

**TLS in v1** (https endpoints admitted, TLS in the std-only client, or a
rustls dependency): deferred rather than rejected — `026-R1` carries it.
Every v1 surface is plain HTTP so the service and client hold the std-only
posture and the disclosure model is stated in the open (`026-L2`); a TLS
story that matters needs a deployment story that matters, and that is the
reserved work.

**An admin read of the whole document** (`GET /v1/admin/catalog`): rejected
under `026-L3` — the no-bulk rule does not exempt the admin surface on the
root, where the document is the large one. Operators of the root read their
own corpus file; everyone else gets `stats`.

**Auto-blending upstream candidates with local registrations**: rejected —
an explicit registration is the user stating curation with intent, and
diluting it with the root's guess (or ordering one above the other in a
merged list) makes the answer depend on upstream churn. Local wins outright
(`026-L5`); the user edits their own entry to change it.

## Consequences

Becomes easy: running `discovery list` for real — pointed at the public root
or a local caching registry or a test registry, the client cannot tell and
neither does the user's flow; self-hosting a registry that costs nothing to
operate because it stores only what its users touched; giving `local/`-source
installs a curation path no public root can own; and ADR-025's refusal lines
finally naming a transport that exists.

Costs: `025-C4` and `025-L1` are amended one day after landing — the gate
suite's and `catalog.rs`'s pinned refusals re-pin, the third ADR-era
identity/policy surface in a week; v1 traffic is plaintext, so the ADR
states plainly what a chosen endpoint learns (`026-L2`) and `026-R1`
carries the assurance work; a cold caching registry's first scan pays N
serial upstream lookups (`026-R3`); two new local files (corpus, cache) join
the trusted-shape-at-refusal set and their refusals must stay as named as
the match store's; and the public root's operations — hosting, scale of the
join, corpus governance — are this ADR's neighbors, not its content.

Not decided here: the profile half of `025-R1` (`026-R2`, ADR-027 reserved);
TLS (`026-R1`); non-bearer auth or per-token identities; who curates the
root corpus and how disputes land; cache prefetch (`026-R3`).
