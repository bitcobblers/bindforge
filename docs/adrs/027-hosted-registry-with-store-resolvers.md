---
doc_type: adr
status: proposed
date: 2026-11-21
depends-on: [013, 022, 025, 026]
ledger: yes
---

# ADR 027: Serve the catalog from a hosted registry backed by store snapshots

This ADR fills the curation-intake half of `026-R2`: the registry's content
comes to know itself from the stores' own bulk listings instead of hand
registration, and the registry itself becomes a single hosted service rather
than software every user installs. It retires `026`'s user-hosted caching
mode and re-reserves the profile half of `026-R2` — profile
upload/download/sharing and auth beyond the single bearer token — to
ADR-028. The wire contract `026-C2` pins for clients — the candidate route,
response shape, and budgets — stands untouched, as does the shipped CLI
whole: `025`'s client keeps the registry as its only catalog seam, and no
store API is ever on a user's request path.

**Terminology.** The *hosted registry* is the one public instance, run by
the project, that every client points `BINDFORGE_CATALOG_URL` at. The
*curated corpus* is the `026-C3` document the operator curates. The
*issued-entries document* is the registry's own record of identities it
minted from store data. A *snapshot resolver* answers a source token's keys
from a locally held, periodically refreshed copy of that store's public
listing: the `steam` resolver is this ADR's, the `epic` resolver is a
reserved slot, and `local` is never resolved.

## Context

ADR-026 shipped a registry with two modes and an assumption. The modes —
a curated root and a user-hosted caching instance — are built, reviewed,
and green. The assumption was that useful catalog content arrives through
registration: curation upserts on a root, personal registrations on a
user's own caching daemon. Five shipped seed entries are that story's
entirety, and no public instance of either mode exists. The consequence is
a loop that runs but never lands: `discovery list` gates, scans, queries,
and resolves nothing, because the endpoint a user can honestly point at
answers `200 []` for every key anyone has ever asked.

Hand registration cannot be the primary content path at store scale. The
public Steam catalog is on the order of a hundred thousand games, Epic's
several thousand more; no operator types that, and no user runs a daemon
to register their own dozen. But at exactly that scale the stores make the
data available in bulk: Valve's keyed `IStoreService/GetAppList/v1` returns
the games-only catalog — appid, name, modification time — pageably, with an
incremental refresh mode, under a license that permits redistribution
through the licensee's named application, at a call volume one operator
refreshing a snapshot never approaches (`100,000` calls per key per day
against a daily pull of a handful of pages). The half of a catalog entry
that is *store knowledge* — which appid, which display name — needs no
curator at all; only the half that is *BindForge knowledge* — this game's
canonical id, its cross-store join, its manifest — does.

Epic cannot ride the same path: it publishes no public catalog API, its
unofficial listings speak storefront slugs while our `epic/` keys will
speak `CatalogItemId` (`025-R2`, unbuilt), and the discovery backend that
would mint those keys does not exist yet. And `local/<dir>` keys are, by
construction, unknown to every store; the question of how a player with
only local installs gets their keys resolved is the same question `025`
answered with local authority, and a separate, smaller amendment about
self-resolving local keys stands outside this ADR.

## Decision

We will run one hosted registry that every client points at, and give it
store-snapshot resolvers behind the cache seam `026` already built: a
queried key answers from the curated corpus, then from issued entries,
then from the resolver for its source token — the `steam` resolver
answering only from its locally held snapshot, never by fetching during a
request — and the first time a resolver answers for a key we will mint a
permanent identity for it and persist it as an issued entry. The admin
face remains, demoted from the content path to the curation path: it
promotes, corrects, and merges issued records into curated ones. The
user-hosted caching mode retires.

### Contracts

- **027-C1** The topology and the binary: the registry serves one hosted
  mode — `registry serve --root --corpus <path>` as `026-C1` spells it,
  extended by the resolver flags of `027-C3` — for all users; no instance
  is expected to run beneath a user. `026`'s caching-mode spellings
  (`--upstream`, `--refresh-interval`, the cache-document pair) become
  named startup refusals, `serve`'s health mode value becomes `hosted`,
  and `query <slug>` survives as the operator's one-shot run of the hosted
  lookup policy. Registry-to-registry federation retires with the mode it
  served. Amends `026-C1`. → Rollout, item 2.
- **027-C2** The resolver seam: a queried admissible key whose source
  token has an active resolver and no corpus or issued answer goes to that
  resolver; the closed token set (`025-C1`) partitions as `steam` (active,
  this ADR), `epic` (reserved slot, inactive — an inactive resolver is a
  named miss, never a fabricated answer), and `local` (never a candidate
  for any resolver). Resolvers answer `[{game_id, name}]` within the same
  caps the serving face already enforces. → Rollout, item 3.
- **027-C3** The steam snapshot resolver: the instance maintains
  `<data>/bindforge/registry/steam-snapshot.json` — `{version, fetched_at,
  apps: [{appid, name}]}` in ascending `appid` order, owner-only,
  tmp-and-rename, refused by name outside its shape, bounds, and the name
  bound `013-B4` — refreshed from
  `IStoreService/GetAppList/v1` games-only pages. The Web API key arrives
  only through the environment seam `BINDFORGE_STEAM_WEB_API_KEY`; a
  missing key is a named startup refusal unless the operator passes
  `--no-steam`, which serves with the resolver inactive. At startup the
  snapshot is loaded if present and within `027-B2`'s freshness, else
  pulled in full before the process binds; a failed pull is a named
  startup refusal; a background thread re-pulls incrementally
  (`if_modified_since`) at `027-B2`'s interval. Joins answer by exact
  `appid` lookup in the loaded snapshot. Amends `026-C2`'s health
  document: `mode` is `hosted`, the `upstream` fact retires, and a
  `steam_snapshot` fact carries `{games, fetched_at}` or `none`. →
  Rollout, item 3.
- **027-C4** The issued-entries document:
  `<data>/bindforge/registry/issued-v1.json`, a second `026-C3` catalog
  document with `026-C3`'s loader refusals and the same owner-only
  tmp-and-rename discipline; an absent file is an empty document. When a
  resolver answers for a key that has no curated or issued answer, the
  resolver's answer is persisted as a whole issued record before it is
  served, under the same write lock that serializes admin mutations.
  Issued records and curated records are separate documents and neither
  is ever merged silently into the other. → Rollout, item 4.
- **027-C5** The mint rule: an issued record's `game_id` is the
  shared-grammar slug (`024-C2`'s) of the store display name — lowercased,
  runs of non-alphanumerics collapsed to `-`, bounded to the slug budget —
  and where that slug is already held by a different record,
  `<slug>-steam-<appid>` instead. The mint is one-way: the slug of a name
  a store later renames is never re-derived, and the rule below the mint
  is `027-L2`. → Rollout, item 4.
- **027-C6** The curation face: `026-C4` and `026-C5` stand unchanged for
  the corpus, plus `DELETE /v1/admin/issued/<game_id>` retiring one issued
  record under the same auth rule, and `GET /v1/admin/stats` gaining the
  issued count. Promotion is the operator upserting a whole curated record
  (typically the issued record with `source_keys` extended) and then
  deleting the issued one; a curated record for a key suppresses issuance
  for that key permanently, per `027-L5`. → Rollout, item 5.

### Laws

- **027-L1** Nothing on the request path: the registry makes no store call
  while answering a query; resolvers answer from their persisted snapshot
  alone, and refresh traffic is the operator's own bulk pull, not a
  per-user proxy. A design that dials a store mid-query violates this
  law, whatever its latency looks like. → Rollout, item 3.
- **027-L2** Minted identity is immortal: a `game_id` once issued is never
  re-derived, rewritten by a store rename, or reused for another game;
  curation corrects a record's keys and name around a stable id, and
  deleting a record never frees its id for reissue. Match stores hold
  these ids, so this law is what makes minting admissible at all. →
  Rollout, item 4.
- **027-L3** The miss's honesty, amending `026-L4` for the hosted mode: a
  steam key with no corpus, issued, or snapshot answer is `200 []` only
  while the snapshot is within `027-B2`'s staleness bound — inside it,
  "never heard of it" is a statement the snapshot is entitled to make;
  past it, with no snapshot, or with the resolver inactive, the answer is
  the named `503`. A cold store-resolver never answers `[]`. → Rollout,
  item 4.
- **027-L4** The disclosure seam: a store ever sees the operator's bulk
  catalog pull and nothing else — no queried key, user or otherwise, is
  ever sent to a store by the registry. Client-to-registry disclosure
  remains `026-L2`'s endpoint-choice consent, unchanged in kind. →
  Rollout, item 3.
- **027-L5** Curation outranks the mint: a curated corpus record answers
  its keys alone; where corpus and issued records both name a key, the
  corpus answers and the issued record is visible to curation only, never
  merged into an answer. → Rollout, item 4.

### Budgets

- **027-B1** `MAX_STEAM_SNAPSHOT_OCTETS = 33_554_432` — the snapshot's
  load ceiling, past which the file is refused by name like every other
  contract document. → Budgets.
- **027-B2** `STEAM_SNAPSHOT_REFRESH_SECONDS = 86_400` and
  `STEAM_SNAPSHOT_MAX_AGE_SECONDS = 604_800` — the refresh interval and
  the age past which the snapshot is stale enough that `027-L3`'s empty
  answer is no longer honest. → Budgets.
- **027-B3** `MAX_ISSUED_ENTRIES = 200_000` — issued records held; at the
  cap new mints are refused with the named `503`, because an identity
  that cannot be persisted is not an identity. → Budgets.
- **027-B4** `STEAM_REFRESH_MAX_PAGES = 12` — pages per refresh at the
  endpoint's 50k cap, the whole quota posture for the keyed list in one
  number, orders of magnitude inside the licensed daily allowance. →
  Budgets.

### Reservations

- **027-R1** The epic resolver: blocked on `025-R2` (the discovery backend
  that mints `epic/` keys and pins whether `CatalogItemId` is the key) and
  on the absence of any public Epic listing API to snapshot; the resolver
  seam's reserved slot stays inert until both land. Owner: `unassigned`. →
  `025-R2`
- **027-R2** The profile half of `026-R2` this ADR does not fill after
  taking the intake half: profile upload/download/sharing and auth beyond
  the single bearer token. Owner: ADR-028 (reserved). → ADR-028
- **027-R3** Transport assurance: `026-R1` carries forward unchanged —
  plain HTTP in v1, TLS terminated at an edge the deployment fronts, the
  operator's decision to make. Owner: `unassigned`. → Rollout, item 7.

### Seeds

- **027-S1** The hosted instance's corpus starts as the shipped
  `026-S1` seed; every steam key in it answers from curation, and every
  other steam key answers from the snapshot on day one, so the hosted
  registry is useful the moment it binds without any bulk import landing
  in this repository. → Rollout, item 2.

**Unit handles.** The handles in this ADR sit at the end of the Rollout
list items: one trailing code span of unit ids governs the line's claim, an
id anywhere else governs nothing.

Rollout:

1. The ledger annotations land: the `026` units this ADR amends or whose
   rules retire gain their `amended-by` and retirement lines per
   `021-L4`, and `026-R2` records this ADR as filling the intake half with
   the profile half forwarded to `027-R2`. `027-C1 027-C3 027-L3 027-R2`
2. The hosted mode lands: `serve` gains the hosted posture and the
   caching-mode spellings become named refusals, the health document
   re-pins, and the suites that pin caching-mode behavior retire or
   re-pin; the seed corpus serves unchanged. `027-C1 027-S1`
3. The steam snapshot resolver lands: the snapshot document, the startup
   pull, the refresh thread, the environment key seam, the budgets, and
   the laws that keep stores off the request path and off the disclosure
   seam. `027-C2 027-C3 027-L1 027-L4 027-B1 027-B2 027-B4`
4. Issuance lands: the issued document, the mint rule and its collision
   spelling, answer-before-persist ordering under the write lock, the
   curation-outranks law, the amended miss posture, and the issued-count
   budget. `027-C4 027-C5 027-L2 027-L3 027-L5 027-B3`
5. The curation face lands: the issued delete route, the extended stats,
   and the promotion flow's refusal shapes. `027-C6`
6. The operator runbook lands in `docs/`: the hosting posture, the key
   seam, the corpus/issued file custody, and the edge-TLS stance this
   ADR leaves to the operator. `027-R3`
7. The hosted instance is deployed by the operator against this runbook;
   the deployment itself is an operator act, outside the repository's
   gates. `027-R3`

## Alternatives considered

**Hand curation as the primary path**, the direction the thread arrived
from — an import tool POSTing curated records to the admin face. It lost
on scale and permanence: the store half of the catalog is derivable data
the operator would otherwise re-maintain forever, and every gap it misses
is an inert scan for some user. Curation stays, at the identity and join
half, which is the part that needs a human.

**Lazy store proxying** — the registry dials Steam on a miss, caches the
answer, serves it on the next scan. It lost on `027-L1`: a public service
whose per-query behavior is a live third-party call inherits someone
else's latency, throttle, and shutdown into our failure surface, and it
makes every user's key the operator's problem at request rates rather
than the operator's own bulk pull at refresh rates. The snapshot trades
staleness — priced in `027-B2` — for a request path that never lies and
never blocks on a stranger.

**Per-source records with a merge operation**, letting Steam and Epic
answers exist as separate entries the registry fuses later. It lost
because the fusion still requires the same-game recognition that building
the joined record requires, with a window in between where one game
answers differently per queried key and match stores confirm whichever
identity arrived first. Attaching a newly discovered source to an
existing identity is already expressible as a whole-record curation
upsert (`026-C4`), so the merge mechanism buys nothing the curation face
does not already do.

**Discovery CLI speaking to stores directly.** It lost on the consent
seam: `026-L2` makes the user's choice of registry endpoint the one
disclosure decision, and a store client in the CLI is a second endpoint
the user never chose, pointed at by a flag that names a different one.

**Shipping Epic in v1.** It lost on facts: no public listing API exists to
snapshot, the key grammar the resolver would answer is reserved to
`025-R2`, and the discovery side that mints `epic/` keys is unbuilt. The
seam is shaped for it; the slot stays inert.

## Consequences

The loop stops being idle: a fresh install pointing at the hosted
instance resolves its whole Steam half on the first scan, and the
BindForge-specific value — ids, joins, manifests — becomes the visible
difference between a minted record and a curated one. Three burdens
arrive with that. Identity minting becomes product surface: slugs minted
today are load-bearing forever (`027-L2`), collision spellings will show
up in real match stores, and the mint-cap is an abuse posture this design
takes seriously rather than discovers later. The operator becomes a real
role: one person or rotation holds a Steam key, a host, the corpus and
issued files' custody, and the edge-TLS decision `027-R3` waves through.
`026`'s caching-mode code and suites retire against this branch, shrinking
the binary's exposed surface to the one topology anyone actually runs. The
CLI is untouched — the wire contract it pins survives verbatim — and
`local/` players keep `025`'s local-authority posture, with a separate
small amendment proposed (outside this ADR's ledger) letting the CLI
self-resolve `local/<dir>` keys to their own source key so no hosted
service is ever required to play on one's own machine.
