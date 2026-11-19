---
doc_type: adr
status: accepted
date: 2026-11-18
depends-on: [013, 022, 024]
ledger: yes
---

# ADR 025: Split discovered-game identity into a source key, a curated game id, and a friendly name

This ADR fills the slot `024-R1` reserved: its matching half — getting a
discovered game onto a real product id — is discharged here; its
publisher-namespace half is superseded by this ADR's decision (see
Alternatives) and recorded as such. This ADR amends the identity half of
ADR-024 — `024-C1`, `024-C2`, `024-L1`, `024-L2` gain their `amended-by`
per `021-L4` (Rollout, item 5) — while `024-B1` and `024-L3`, the shared
length bound and its stated-home discipline, stand untouched. ADR-022
predates the ledger, so two overrides against it are stated in prose: the
no-remote-service stance (a discovery scan now requires connectivity) and
the provisional `unknown/` spelling, which `024` had already moved to a
`Namespace` field and this ADR retires outright.

## Context

ADR-024, landed today, made a discovered slug an id the product can adopt
byte-for-byte. That fixes the `unknown/` grammar failure but inherits the
display name's own failures, which no grammar can fix. Returnal installed
on Steam and on Epic kebab to the same `returnal` and fuse only by guess;
two different games from two publishers can share a display name and split
apart only by the run-order `-2` suffix `024-L2` invented; a store that
retitles a game churns the slug before any match exists — the instability
ADR-024 itself conceded as its known cost. Identity carried by a name
derived from a string the store owns will keep leaking.

Each source already carries a per-install identifier the store maintains
for its own reasons: Steam's AppID rides every `appmanifest_*.acf`, and
Epic's launcher keeps a `CatalogItemId` in each install's `.egstore`
manifest. These are unique within their source and survive retail
retitling — everything a name-derived slug is not. They are also pure
third-party hygiene: re-listings mint new ids, they mean nothing across
sources (Steam's Returnal id and Epic's Returnal id are two ids for one
game), and manual installs have none. So a source id answers "which
install is this" but never "which game is this". That answer is curated
knowledge: a cross-source common id — which, in this product, already
exists and is the manifest `id` every profile reference already pins
(`013-L4`, `014`).

BindForge is an online service that hosts a catalog of games and their
profiles; the curation that maps source ids onto game ids is the
catalog's own data. The coherent shape is therefore three identifiers
with three distinct roles — a source-unique lookup key the plugin emits,
a curated game id the product joins on, and a friendly name demoted to
search metadata — with the join between the first two made by a candidate
lookup the user confirms, and the confirmation remembered locally.

## Decision

We will untangle the three identifiers: the **game id** (the manifest
`id`) becomes the only cross-source identity and arrives only by
curation — never derivation; the **slug** becomes the catalog lookup key
`<source>/<external-id>`, supplied by the source plugin and joined to
nothing directly; and the **friendly name** becomes display/search
metadata that joins nothing. Discovery will require connectivity, and
confirmed matches will persist locally, so a matched game keeps its game
id across scans and machines of the same user.

**Terminology.** The term *slug* takes its new sense — the catalog
lookup key — only for the discovery row. The shared slug grammar and the
`024-B1` bound keep their names and keep governing every product id
(manifest, profile, policy, codec, preset, geometry, action segment); no
other format's use of the word changes here.

### Contracts

- **025-C1** The discovery row's identity: `DiscoveredGame.slug` is the
  catalog lookup key `"<source>/<external-id>"`. The source is a closed
  token set, `steam` · `epic` · `local`; the external-id component is
  what the source calls the install — Steam's AppID, Epic's
  `CatalogItemId` once that backend lands (`025-R2`), and for `local` the
  kebab'd install directory name. The slug is opaque to the product: it
  is exempt from the shared slug grammar and the `024-B1` bound, capped
  instead by `025-B1`, and never stands as a path component (`025-L5`).
  The `Namespace` enum retires with its sole v1 value. Amends `024-C1`.
  → Rollout, item 1.
- **025-C2** The friendly name: a free-text normalization of the row's
  `display_name` that rides the sibling bound
  `MAX_NATIVE_NAME_OCTETS = 128` (`013-B4`); it is never end-user visible
  (the UI prints `display_name`), participates in no join, and carries no
  derivation law — `024-C2`'s algorithm retires as a law and survives at
  most as a display convention. Amends `024-C2`. → Rollout, item 1.
- **025-C3** The local match store: a per-user file in the product's
  data directory holding one record per confirmed install — `slug`,
  `game_id` (the catalog's manifest id), `friendly_name` — written only
  by the confirmation `025-L2` admits. Two sources holding one game are
  two records sharing one `game_id`. → Rollout, item 2.
- **025-C4** Candidate lookup: a REST call `GET
  <catalog>/v1/candidates?slug=<slug>` returning zero or more candidates
  `{game_id, name}`. The CLI queries for every slug the store does not
  already resolve (`025-L4`) and puts the candidates to the user; in
  JSON-output mode it emits the candidates instead of prompting. The
  server-side contract is `025-R1`. → Rollout, item 3.
  amended-by 026-L1 (ADR-026, per 021-L4): the provisional loopback-only
  endpoint admission stated at this unit's design target — marked there as
  ADR-026's — retires; the lookup contract itself stands.

### Laws

- **025-L1** Connectivity gate: a discovery scan requires working
  connectivity and refuses with a named error before enumerating if the
  catalog is unreachable; regular use — resolving profiles, applying
  bindings, browsing the store's own data — must never require the
  network. (This overrides ADR-022's rejection of remote-service
  coupling for discovery; enumeration itself stays local.) → Rollout,
  item 3.
  amended-by 026-L1 (ADR-026, per 021-L4): the probe's
  refuse-non-loopback-resolution leg stated at this unit's design target
  retires; the fail-closed gate itself stands.
- **025-L2** No silent match: a `game_id` attaches to a discovered row
  only from a stored confirmed match or from a candidate the user chose;
  name similarity, duplicate names across sources, or a lone high-scoring
  candidate never complete the join by themselves. → Rollout, item 3.
- **025-L3** Emission discipline: each source plugin owns its
  external-id choice and answers for uniqueness within its source —
  `local` is exempt, its kebab'd names can duplicate, which is admissible
  precisely because the slug joins nothing by itself. The product checks
  only `025-B1` at the emit boundary, not the grammar. The `024-L2`
  `-2`/`-3` suffix rule retires: slug collisions are structurally gone
  and friendly-name duplicates are by design. Amends `024-L1` and
  `024-L2`. → Rollout, item 1.
- **025-L4** Remembered confirmations: a confirmed match persists across
  scans and is trusted without expiry; a slug the store does not resolve
  re-queries the catalog on every scan. (This retires ADR-022's
  no-persistence stance for match state only.) → Rollout, item 2.
- **025-L5** Corpus boundary: the game id is the only identifier of this
  ADR that enters a shipped artifact or a path component; source keys and
  friendly names stay in discovery output and the local store. A profile
  that pinned a source id would bind a shared artifact to one launcher.
  → Rollout, item 4.

### Budgets

- **025-B1** `MAX_DISCOVERY_SLUG_OCTETS = 128` — bounds the
  `<source>/<external-id>` string end-to-end; set equal to the shared
  `024-B1` bound so the constant family reads as one, though it caps a
  lookup key, not a grammar-validated id; one named constant beside
  `MAX_SLUG_OCTETS` in `crates/core`. → Rollout, item 1.

### Reservations

- **025-R1** The catalog service itself: hosting, the canonical REST
  surface, the curation intake by which new source-id↔game-id facts
  enter the catalog, auth, and profile upload/download/sharing.
  Owner: ADR-026 (reserved). → ADR-026.
  filled by ADR-026 (registry-service): its root and caching registry
  modes fill the hosting, candidate-surface, and curation-intake half;
  auth beyond the single bearer token and profile
  upload/download/sharing are reserved forward to ADR-027 (`026-R2`).
- **025-R2** The Epic discovery backend behind the `epic` token,
  including verifying whether `CatalogItemId` alone is Epic-unique or
  needs its catalog namespace. Owner: `unassigned` — a rollout task, not
  an ADR slot. → Rollout, item 1.

**Unit handles.** The handles in this ADR sit at the end of the Rollout
list items: one trailing code span of unit ids governs the line's claim,
an id anywhere else governs nothing.

Rollout:

1. The row identity lands in `crates/discovery`: `slug` carries the
   `<source>/<external-id>` spelling, the Steam plugins emit
   `steam/<appid>` from the ACF, the `Namespace` enum is removed from
   `DiscoveredGame`, the friendly name joins the row as display metadata,
   and `MAX_DISCOVERY_SLUG_OCTETS` lands in `crates/core` beside the
   `024-B1` constant and is checked at the emit boundary.
   `025-C1 025-C2 025-B1 025-L3 025-R2`
2. The local match store lands: the per-user record file, its read/write
   shape, and the remember-without-expiry rule. `025-C3 025-L4`
3. The catalog client lands in `apps/discovery-cli`: the connectivity
   gate, the candidate query for unresolved slugs, the user-chosen
   confirmation, and candidate passthrough in JSON mode.
   `025-C4 025-L1 025-L2`
4. The corpus boundary and the identity change are pinned: boundary
   fixtures assert no source key or friendly name reaches a manifest or
   profile; the `024` identity tests retire (namespace serde, derivation
   law, collision suffixes) and the slug-grammar tests keep their
   product-id meaning untouched. `025-L5`
5. The ledger annotations land: `024-C1`, `024-C2`, `024-L1`, `024-L2`
   gain their `amended-by` lines naming `025-C1`, `025-C2`, `025-L3`,
   `025-L3` per `021-L4`, and `024-R1` records this ADR as filling its
   slot with the publisher-namespace half superseded.
   `025-C1 025-C2 025-L3`
6. The deferral is recorded: the run ships no catalog server — the
   lookup contract `025-C4` stands as the client-side provisional
   spelling until the reserved ADR-026 states the service contract and
   the curation intake. `025-R1`

## Alternatives considered

**Keep ADR-024's name-derived identity.** A slug kebabbed from the
display name, adopted byte-for-byte as the manifest id on a future match.
Rejected: it is exactly the layer that retitles churn, same-name games
collide, and one game on two launchers fuses by guesswork; `024-L2`'s
run-order suffixes are the tell — a deterministic tie-break is being
asked to be an identity rule. The bound and constant work of `024`
(`024-B1`, `024-L3`) is kept precisely because it was orthogonal.

**Derive a publisher namespace** (the direction `024-R1` sketched): the
namespace is a curated publisher string and the pair disambiguates
same-name games. Rejected: publisher strings are third-party-owned and
spelled inconsistently across stores, so the namespace itself becomes
rename-unstable; and it cannot fix same-publisher same-title games at
all. The namespace concept keeps a value — `source` — because a source
token is short-lived-hygiene but *spelled by us*, so it stays stable.

**Key profiles by `source/id`.** The pair is genuinely unique per
install, so let it be the profile's join reference. Rejected: a shared
profile naming `steam/12345` only works for users who own the game on
that launcher, manual installs have no pair at all, and the corpus would
ship third-party identifiers into every shared artifact. The pair is
load-bearing at the install layer; the game id stays load-bearing at the
sharing layer.

**Let matching degrade offline.** Query the catalog opportunistically,
scan fine without it. Rejected: an optional network hides an identity
state machine (matched, unmatched, stale-catalog) behind a UI that looks
identical in two of its states; a named refusal makes the dependency a
visible, testable edge. Enumeration itself stays local, which is what
ADR-022's rejection was really guarding.

**Auto-match a single returned candidate without the user.** Rejected for
now: the catalog's intake (`025-R1`) is young and its confidence
unmeasured; confirmation is the cheap safeguard. If intake matures enough
to earn unattended joins, that rides `025-R1`'s owning ADR, not this one.

## Consequences

Becomes easy: retitling a store page costs nothing (the lookup key is
untouched); the same game on two launchers is two rows and one `game_id`
with no fusion heuristic at all; same-name games from different sources
need no suffix rule; the CLI can answer "which game is this" for
anything the catalog has heard of, with certainty, from a curated table
rather than string similarity.

Costs: a large part of ADR-024's identity half is amended within a day of
landing — fixtures and tests that re-pinned to `024` re-pin again;
discovery now fails closed on connectivity, and every scan is a
disclosure of the queried source ids to the service (the query shape of
`025-C4` is deliberately narrow); third-party catalog hygiene becomes a
user-visible event when a game is re-listed under a new id (a fresh
lookup, a re-confirmation); manual installs stay weakly identified,
reaching a `game_id` only through candidate confirmation like
everything else; and persistence of match state enters scope, closing
another of ADR-022's "none for v1" slots.

Not decided here: the catalog service and its intake (`025-R1`,
ADR-026, reserved); the Epic backend (`025-R2`); whether curated
confidence ever earns unattended matching; persistence of discovery
results beyond the match store (unchanged from ADR-022).
