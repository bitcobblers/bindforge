# Research: Serving the catalog from a hosted registry backed by store snapshots (ADR-027)

## Summary

ADR-027 retires ADR-026's user-hosted caching mode, makes the one hosted posture
the only mode `registry serve` admits, puts a `steam` snapshot resolver behind
the existing cache seam, and mints a permanent `game_id` the first time the
resolver answers a key — with the stores held off the request path (`027-L1`)
and off the disclosure seam (`027-L4`), and Epic kept as an inert reserved slot
(`027-R1`). The content half is well-founded in primary sources:
`IStoreService/GetAppList/v1` is a keyed, appid-ordered, pageable games-only
listing with an `if_modified_since` delta mode, the license permits distributing
Steam Data to end users through the named Application, and the daily quota
(100,000 calls) is orders of magnitude above one operator's daily pull. The
identity half is where the ADR is weakest: `027-C5` mints from "`024-C2`'s"
derivation, which ADR-025 explicitly retired as a law and whose implementation
is deleted from the tree, and store display names are third-party-controlled,
renameable (≈2% of a 2020–2022 indie cohort within ~3.5 years) and often
non-Latin, so the derivation can legitimately come out empty and the plain slug
can be adopted by a rename. The engineering collision is transport: the pull the
ADR mandates is an HTTPS call to a keyed Valve endpoint, while `apps/registry`
links no TLS stack, its shipped endpoint policy refuses `https` by name, and its
shipped response parser caps a body at 65,536 octets against a page that is
megabytes. And `027-B3`'s 200,000-record mint cap sits barely above today's
Steam games count, so the "refuse new mints with a `503`" posture is an ordinary
operating state, not an abuse edge.

## Findings

### What the ADR changes against what is shipped

- The health document `027-C3` re-pins is a shipped, enumerated list: `health()`
  answers `status`, `mode`, `upstream`, `local_entries`, `cached_slugs`,
  `last_upstream_fetch`, and `mode` is the `Mode` enum with exactly two
  inhabitants (`root`, `caching`) — so `hosted` is a third spelling and the
  `upstream`/`cached_slugs`/`last_upstream_fetch` facts are the ones that retire
  while `steam_snapshot: {games, fetched_at} | none` is added
  (source: apps/registry/src/state.rs; docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C1`/`027-C3`).
- The named-refusal surface for the retired spellings is already flag-parsed in
  one place: `--root`, `--corpus`, `--upstream`, `--host`, `--port`,
  `--admin-token`, `--refresh-interval` plus `BINDFORGE_REGISTRY_ADMIN_TOKEN`,
  with startup refusals already named as a family; `--upstream` and
  `--refresh-interval` join that family, and `027-C1` adds no new spelling for
  the resolver flags it defers to `027-C3` (`--no-steam` and whatever names the
  snapshot path) (source: apps/registry/src/main.rs;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C1`/`027-C3`).
- `027-C1`'s retirement is a code-and-suite sweep, not a flag change: the
  caching half of the crate is `cache.rs`, `upstream.rs`, the `Mode::Caching`
  arms in `state.rs`, and the `cache-v1.json`/`catalog-v1.json` pair in
  `persist.rs`, and the shipped suites that pin the retiring behavior are
  `tests/caching_policy.rs`, `tests/cache_document.rs`,
  `tests/upstream_client.rs`, `tests/upstream_endpoint_gate.rs`,
  `tests/health_route.rs`, `tests/data_layout.rs` and
  `tests/cli_end_to_end.rs` — every one of which `027-C1` re-pins or retires
  (source: apps/registry/src/cache.rs; apps/registry/src/upstream.rs;
  apps/registry/src/persist.rs; apps/registry/tests listing).
- Registry-to-registry federation ("retires with the mode it served") is the
  whole of `026-C6`'s upstream ingest, including the choice to gauge the
  upstream through its `/v1/health`; nothing in `027` replaces it, so the
  hosted instance has exactly one content source: the corpus file, the issued
  document, and the snapshot (source: docs/adrs/026-registry-service.md
  `026-C6`; docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C1`).
- There is no timer seam today. The only periodic behavior in the shipped
  server is a staleness check evaluated inside a lookup on a request thread;
  `027-C3`'s "a background thread re-pulls incrementally at `027-B2`'s
  interval" is new machinery — a std thread plus a sleep loop, with its own
  lifecycle questions against the accept loop, the write lock, and the one-shot
  `registry query` posture (`027-C1`) (source: apps/registry/src/state.rs;
  apps/registry/src/serve.rs; docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C1`/`027-C3`).
- `027-C4` names one lock ("the same write lock that serializes admin
  mutations"), but with `027` there are three writers of three documents in one
  process: the request path minting `issued-v1.json`, the refresh thread writing
  `steam-snapshot.json`, and the admin face writing the corpus. The shipped
  persist discipline is a *fixed* tmp name opened `create_new` at `0600` with
  rename-into-place, which is a loud failure rather than a race for a second
  *process* but says nothing about two *threads* in one process
  (source: apps/registry/src/persist.rs;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C4`/`027-C3`).
- The seed the hosted corpus starts from (`027-S1`) is six records, not the five
  the Context paragraph counts, and one of the six already carries an `epic/`
  source key spelled as a 32-hex-digit `CatalogItemId` — a key format
  `027-R1`/`025-R2` still call undecided — answered today by curation alone
  (source: data/catalog/v1/catalog.json;
  docs/adrs/027-hosted-registry-with-store-resolvers.md Context and `027-S1`/`027-R1`;
  docs/adrs/025-discovery-identity-and-catalog-matching.md `025-C1`/`025-R2`).
- The same six curated ids are exactly what `027-C5` would mint from the store
  display names (`Counter-Strike 2` → `counter-strike-2`, `Left 4 Dead 2` →
  `left-4-dead-2`, `Portal 2` → `portal-two`), which is the strongest available
  evidence that mint-then-promote converges on curated ids rather than colliding
  with them — for Latin-script names (source: data/catalog/v1/catalog.json;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C5`/`027-L5`).
- The ADR contradicts itself on the ordering `027-C4` fixes: the unit says the
  resolver's answer "is persisted as a whole issued record **before it is
  served**", while Rollout item 4 lists "**answer-before-persist** ordering
  under the write lock" (source: docs/adrs/027-hosted-registry-with-store-resolvers.md
  `027-C4` and Rollout item 4).
- Ledger work is mechanical but broad: `027` amends or retires `026-C1`,
  `026-C2`, `026-C6`, `026-C7`, `026-B1`–`026-B3`, `026-L4` (amended by
  `027-L3`), and splits `026-R2`, each needing `amended-by`/retirement lines
  under `021-L4`; `027-C5` additionally resurrects `024-C2`, which `025-C2`
  already amended out of existence, and the ADR does not say it revives a
  retired unit (source: docs/adrs/026-registry-service.md;
  docs/adrs/025-discovery-identity-and-catalog-matching.md `025-C2`;
  docs/adrs/021-decision-unit-ledger.md `021-L4`).

### The mint's basis is a retired algorithm whose code is gone

- `027-C5` grounds the mint in "`024-C2`'s" derivation, but `025-C2` amended
  that unit with the words "`024-C2`'s algorithm retires as a law and survives
  at most as a display convention" — the ADR therefore mints product identity
  from a rule the ledger currently records as retired
  (source: docs/adrs/025-discovery-identity-and-catalog-matching.md `025-C2`;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C5`).
- The implementation is deleted, not merely unmotivated: `derive_slug`,
  `resolve_collisions`, `crates/discovery/src/slug.rs` and the `Namespace` enum
  are gone from the shipped sources, and the suites that pinned the derivation
  table were moved out of any compiled target behind the never-default
  `retired-adr-024-suites` feature (source: crates/discovery/tests/retired/README.md;
  crates/discovery/tests/retirement_tests.rs; crates/discovery/Cargo.toml).
- The retirement gate scans only `crates/discovery/src` and
  `apps/discovery-cli/src` for the retired names, so a fresh implementation in
  `apps/registry/src` is legal under the gate — but it is *new* product code, and
  the retired table in `crates/discovery/tests/slug_tests.rs` is the behavioral
  record it should be re-pinned from (source: crates/discovery/tests/retirement_tests.rs;
  crates/discovery/tests/slug_tests.rs).
- `027-C5`'s one-line restatement ("lowercased, runs of non-alphanumerics
  collapsed to `-`, bounded to the slug budget") is *not* equivalent to the
  algorithm it cites. `024-C2` is an ordered seven-step algorithm: ASCII
  case-fold; strip Latin diacritics via NFKD with combining marks dropped;
  replace remaining characters outside `[a-z0-9]` with `-`; collapse hyphen runs;
  trim; **prefix `g-` to a leading digit**; and on an empty result fall back to
  the same normalization of the **install directory name**, refusing the game
  when that is empty too (source: docs/adrs/024-game-slug-identity.md `024-C2`).
- Two of those steps have no store-side analogue: a snapshot record has no
  install directory to fall back to, and the shipped grammar requires the first
  byte to be ASCII-lowercase and every byte to be `[a-z0-9-]`, so a store name
  that is CJK-, Cyrillic-, or emoji-only derives to the empty string and a
  digit-initial name without the `g-` prefix is grammar-invalid. Steam names of
  both kinds exist in volume — 36 of 121 renamed indies in one study were
  language changes, i.e. a non-English title gaining an English one
  (source: crates/core/src/placement.rs `is_slug_name`;
  docs/adrs/024-game-slug-identity.md `024-C2`;
  https://striketwo.games/blog/change-game-name-on-steam).
- The mint's collision spelling (`<slug>-steam-<appid>`) diverges from the
  retired `024-L2` spelling (`-2`, `-3`, … in display-name order). The divergence
  is the right direction — an ordinal suffix depends on which records already
  exist, so the same game can be minted differently at different times, whereas
  an appid suffix is a function of the record — but it means the grammar now has
  two documented disambiguation conventions in its history and the ADR does not
  retire the old one (source: docs/adrs/024-game-slug-identity.md `024-L2`;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C5`).

### The store data, at the scale the ADR assumes

- The endpoint is documented at `GET https://partner.steam-api.com/IStoreService/GetAppList/v1/`,
  requires `key`, and paginates: results "are returned in order of `appid`" and
  "subsequent requests should pass the last appid returned by the previous
  request as the `last_appid` parameter"; `max_results` is "Default 10k, max
  50k"; items carry `last_modified` (unix) and `price_change_number`, and
  `if_modified_since` filters "to only games that have been changed or added
  since a specific timestamp" (source: https://partner.steamgames.com/doc/webapi/IStoreService).
- Only `Game` items come back by default, and DLC/software/videos/hardware are
  opt-in flags — so the ADR's "games-only" posture is the endpoint's default, and
  a snapshot pulled without setting the flags is not comparable to one that sets
  them (source: https://partner.steamgames.com/doc/webapi/IStoreService).
- The bulk surface ADR-026's sibling research called Valve's answer is now
  formally deprecated — `ISteamApps/GetAppList/v2` is marked "Deprecated — this
  API can no longer scale to the number of items available on Steam. Please use
  IStoreService/GetAppList instead" — which is what makes `027`'s choice the only
  supported one rather than one of two (source:
  https://partner.steamgames.com/doc/webapi/isteamapps).
- Catalog size, four ways: 197,962 Steam titles and 7,275 Epic titles (3 Nov
  2026); 191,195 games plus 60,043 DLC; 194,645 unique Steam games; and a
  store-search count of 175,178 games against 282,299 total entries. The
  games-only `IStoreService` pulls third parties report are ~164k games after
  the v2 removal, against ~49k games in a 2021-era snapshot comparison — the
  same listing has roughly quadrupled what "games" means in five years
  (source: https://gamechosis.com/en/stats/; https://shelfwatch.net/stats;
  https://gamebrain.co/how-many-video-games-are-there;
  https://www.elitegear.io/games/wie-viele-spiele-gibt-es-auf-steam/;
  https://github.com/Producdevity/EmuReady/pull/342;
  https://github.com/woctezuma/steam-store-snapshots).
- `027-B4`'s 12 pages is ample against the 50k maximum (600k appid slots) and
  insufficient against the *default* page size: at the documented default of
  10k, a full games-only pull of a ~198k-title catalog needs ~20 pages, so a
  refresh that omits `max_results` trips its own page budget and stops early —
  and `027-L3` would then make the registry certify "never heard of it" against
  a knowingly incomplete snapshot (source:
  https://partner.steamgames.com/doc/webapi/IStoreService;
  https://gamechosis.com/en/stats/;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-B4`/`027-L3`).
- `027-B1`'s 33,554,432 octets is ~168 octets per record at `027-B3`'s 200,000
  records; a compact `{"appid":1234567,"name":"…"}` record at a plausible name
  length is well inside that, but the same document serialized with
  `serde_json::to_string_pretty` is several times larger, and the ADR states a
  load ceiling with no stated encoding rule to match it
  (source: docs/adrs/027-hosted-registry-with-store-resolvers.md `027-B1`/`027-B3`).
- The license is real but conditional, and its conditions are operator
  obligations, not code: "you may … distribute Steam Data to end users for their
  personal use via your Application", where the Application is the one
  "identified in your API key sign up form"; "You agree to keep your Steam Web
  API key confidential, and not to share it with any third party. This license is
  personal to you and specific to your Application"; data must be provided "on
  'as is' terms, with disclaimers substantially equivalent to those set forth in
  Sections 7 and 8"; the data may not be presented so as to appear Valve-endorsed
  or third-party-sourced; and "You are limited to one hundred thousand (100,000)
  calls to the Steam Web API per day" (source: https://steamcommunity.com/dev/apiterms).
- The same terms are personal and non-transferable ("You may not transfer,
  assign, or delegate the API Terms of Use and your Steam Web API key to any
  third party") and revocable at will ("Valve may change, suspend or discontinue
  the Steam Web API and suspend or terminate your use … at any time for any
  reason, without notice") — which makes `027`'s operator a single natural
  person in the license's eyes and makes the hosted instance's content supply
  terminable by a third party without cause
  (source: https://steamcommunity.com/dev/apiterms).
- Transport, per the same primary docs: "The public Steamworks Web API is
  accessed by making HTTP (port 80) or HTTPS (port 443) requests to
  `api.steampowered.com`", while "Steam also provides a partner-only Web API
  server hosted at `https://partner.steam-api.com`" that is recommended for
  availability — and it is the partner host the `GetAppList` page spells
  (source: https://partner.steamgames.com/doc/webapi_overview;
  https://partner.steamgames.com/doc/webapi/IStoreService).

### Where the shipped registry cannot make that call

- `apps/registry` depends on nothing but `std`, `serde_json`, `bindforge-core`
  and `discovery` — no HTTP client, no TLS, no async runtime — and the shipped
  outbound client is a hand-rolled HTTP/1.1 request over `TcpStream`
  (source: apps/registry/Cargo.toml; apps/registry/src/upstream.rs).
- The endpoint policy gate refuses `https` by name: admission is a literal
  `http://` prefix plus any host, which ADR-026 introduced deliberately
  (`026-L1`/`026-R1`). A Steam pull therefore either violates the shipped policy
  it inherits or requires a second, different endpoint policy for outbound
  store traffic (source: apps/registry/src/endpoint.rs;
  docs/adrs/026-registry-service.md `026-L1`/`026-R1`;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-R3`).
- The shipped response parser caps a body at `MAX_RESPONSE_OCTETS = 65_536`,
  refuses `Transfer-Encoding`, and enforces `MAX_CANDIDATES = 256`
  (`apps/registry/src/constants.rs`) — against a `GetAppList` page of 50k items,
  which is megabytes. The snapshot pull cannot reuse the caching client's bounds
  unchanged, and `027` states budgets for the snapshot *file* (`027-B1`) but none
  for the *page* it fetches (source: apps/registry/src/constants.rs;
  apps/registry/src/upstream.rs;
  https://partner.steamgames.com/doc/webapi/IStoreService;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-B1`).
- `027-R3` waves transport assurance at an edge, but that edge is in front of
  *clients*; nothing in `027` fronts the registry's own outbound hop to Valve.
  A plaintext pull puts the `key` in a URL query string on an unencrypted link —
  the class CWE-319/CWE-1428 covers, with the added property that query strings
  are what intermediaries and server logs retain
  (source: https://cwe.mitre.org/data/definitions/319.html;
  https://cwe.mitre.org/data/definitions/1428.html;
  https://partner.steamgames.com/doc/webapi/IStoreService;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-R3`).

## Prior art

( lens: prior art — how comparable projects publish bulk catalog data and mint
  permanent ids )

- **Bulk off the request path is the registry norm, not a BindForge invention.**
  crates.io's own answer to "I need the whole dataset" is the live index
  repository plus "a database dump every 24 hours", with the API reserved for
  interactive use and rate-limited to "a maximum of 1 request per second" plus a
  self-identifying `User-Agent` — the same split `027-L1` draws between the
  operator's bulk pull and the per-query serving face, and the same 24-hour
  cadence `027-B2` picks (source: https://rust-lang.github.io/rfcs/3463-crates-io-policy-update.html;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-L1`/`027-B2`).
- **Permanent, assigned, first-come names have a long operational record.**
  crates.io: "crates.io has a first-come, first-serve policy on crate names",
  "Crate deletion by their owners is not possible to keep the registry as
  immutable as possible", and content/deletion is "permanent and there is no
  basis to reverse" it — which is `027-L2`'s posture stated as policy, and the
  reason `027-L2`'s "deleting a record never frees its id" reads as ordinary
  rather than severe (source: https://rust-lang.github.io/rfcs/3463-crates-io-policy-update.html;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-L2`/`027-C6`).
- **…and its abuse class is name squatting, which those registries police with
  humans.** crates.io added an explicit prohibition on content that "exists only
  to reserve a name for a prolonged period of time (often called 'name
  squatting')" and reserves removal as the remedy. BindForge's mint differs
  structurally: the name is chosen by a third party (the store) and claimed by
  whoever queries first, so the registry's only defenses are `027-B3`'s cap and
  curation's delete/promote pair (source: https://rust-lang.github.io/rfcs/3463-crates-io-policy-update.html;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C5`/`027-B3`/`027-C6`).
- **Name-collision vocabulary.** PyPI resolves collisions by *canonicalizing*
  (`lower()` with runs of `._-` folded to one `-`, so `FrIeNdLy-._.-bArD` and
  `friendly-bard` are one name) — an equivalence rule, unavailable here because
  BindForge must keep two different games apart, not fuse them. Game catalogs
  that auto-derive slugs instead append a counter (the Twitch category for
  "Encounter: The Lost Cards" is `encounter-1`), and the recorded complaint about
  that spelling is precisely that the user cannot get the clean slug back after a
  rebrand — supporting `027-C5`'s choice of a stable, record-derived suffix over
  `024-L2`'s ordinal (source: https://packaging.python.org/en/latest/specifications/name-normalization/;
  https://discuss.dev.twitch.com/t/changing-my-url-slug-after-rebranding-on-igdb/64474;
  docs/adrs/024-game-slug-identity.md `024-L2`).
- **Store-snapshot tooling already exists and is the closest behavioral
  precedent.** Paging loops over `GetAppList` are written out in `steam.py`'s
  HTTP layer and in FronkonGames' Steam Games Scraper; the response shape
  (`apps[]` of `{appid, name, last_modified, price_change_number}` plus
  `have_more_results` and `last_appid`) is pinned in a Go client's API structs;
  an unofficial mirror documents the same parameter table. Third parties then
  *redistribute* the result as datasets, which is the same redistribution the
  ToS conditions on the named Application
  (source: https://github.com/Gobot1234/steam.py/blob/master/steam/http.py;
  https://github.com/FronkonGames/Steam-Games-Scraper/blob/main/SteamGamesScraper.py;
  https://pkg.go.dev/github.com/Jleagle/steam-go/steamapi;
  https://steamapi.xpaw.me/IStoreService;
  https://github.com/woctezuma/steam-store-snapshots;
  https://striketwo.games/blog/change-game-name-on-steam).
- **Renames are common enough to design around, and appids survive them.** A
  study of 5,854 paid Steam indie releases from 2020–2022 found 121 (2.1%)
  carrying a different store name today, and 253 (2.0%) across the full 12,948
  games it had records for; 36 of the 121 were language changes and only 25
  replaced the title outright, and "the app ID stays, so reviews and old links
  survive". A separate analysis of Steam's own charts calls name changes "somewhat
  common" and the game lifecycle "really complicated" to follow programmatically
  (source: https://striketwo.games/blog/change-game-name-on-steam;
  https://kalifi.org/2026/02/steambubbles.html).
- **Store names are an attack surface with an observed history.** Games have
  renamed existing appids to impersonate hits — "DO NOT SMILE renamed to
  Helldivers 2 … Another game, Figurality, did the exact same thing. Then the
  game Stolen Mushrooms, which attempted to be Palworld" — which in `027-C5`
  terms is an appid claiming a plain slug by being the first queried under it,
  and is honest-but-wrong rather than rejected
  (source: https://www.gamingonlinux.com/2024/03/steam-had-a-bit-of-a-scam-problem-with-fakes-of-helldivers-2-and-palworld/;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C5`/`027-L5`).
- **Epic really has nothing to snapshot, and what exists is unofficial.** The
  community client reaches `store-content.ak.epicgames.com/api/content/productmapping`,
  a `searchStore` GraphQL endpoint and `freeGamesPromotions`, and needs
  `cloudscraper` for anti-bot; Epic's own `catalog-public-service` requires a
  Bearer token; and the identifiers that reach users are storefront `AppName`
  values ("earlier `AppNames` are human readable, like `Fortnite` … later ones
  are some kind of UUID") rather than the `CatalogItemId` our `epic/` keys would
  speak. `legendary list` enumerates *an account's* games, not a public catalog
  — which is exactly why `027-R1`'s slot stays inert
  (source: https://epicstore-api.readthedocs.io/en/latest/_modules/epicstore_api/api.html;
  https://github.com/sd4rk/epicstore_api;
  https://www.postman.com/wokok/epic-games-store/documentation/48lskzd/get-list-searchable-elements;
  https://jayd.ml/games/2020/05/16/epic-games-store-steam-libraries.html;
  https://github.com/Heroic-Games-Launcher/legendary;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-R1`).
- **What ADR-026 bought and `027` gives back.** The pull-through-cache model
  ADR-026 copied (Docker Distribution's `proxy.remoteurl` mirror, Verdaccio's
  `maxage`/`cache: false` uplinks, and their documented offline "installs from
  cache" posture) is precisely the topology `027-C1` retires; `027` returns the
  registry to an origin-only service and moves freshness out of a per-user cache
  into the operator's snapshot interval (source:
  https://distribution.github.io/distribution/recipes/mirror/;
  https://www.verdaccio.org/docs/uplinks/;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C1`).
- **Staleness as a first-class answer.** HTTP caching already has the vocabulary
  `027-L3` invents: freshness lifetime versus age, heuristic freshness when the
  origin states none, and serving a stored stale response only under explicitly
  enumerated conditions (RFC 9111 §4). `027-L3`'s rule — a `200 []` is a
  *claim* the snapshot is entitled to make only inside `027-B2`'s bound, and a
  `503` outside it — is that idea with a hard bound instead of a heuristic
  (source: https://www.rfc-editor.org/rfc/rfc9111.html;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-L3`/`027-B2`).

## Platform notes

( lenses: standards, security landscape — the platform lens's input-handling,
  Wayland/X11 and Node-version concerns have no application to this ADR, which is
  the subject of the closing bullet )

- **No TLS exists in the product today, and `std` provides none.** The toolchain
  floor is Rust 1.85 / edition 2024 pinned by `rust-toolchain.toml`, and
  `apps/registry` carries no network dependency; an in-process HTTPS pull means
  adding a TLS stack (rustls, or a blocking client that wraps it) to the crate
  whose "std-only" reading ADR-026 already had to interpret as "no HTTP/TLS
  runtime". The alternatives the design will weigh are an out-of-process pull
  (an operator-run importer writing the snapshot file) or a documented edge
  proxy, both of which change who `027-C3`'s startup "pull in full before the
  process binds" belongs to
  (source: rust-toolchain.toml; Cargo.toml; apps/registry/Cargo.toml;
  docs/adrs/026-registry-service.md `026-C1` and §Alternatives considered
  ("TLS in v1"); https://github.com/rustls/rustls;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C3`).
- **The secret seam has a named weakness class.** CWE-526 is "Cleartext Storage
  of Sensitive Information in an Environment Variable": the recorded
  consequences are readability "by other processes with the execution context,
  including child processes", and insertion "into messages, headers, log files,
  or other outputs"; the documented mitigation is that "a properly protected
  configuration file" is "a better choice" than an environment variable.
  `027-C3`'s `BINDFORGE_STEAM_WEB_API_KEY` follows the shipped
  `BINDFORGE_REGISTRY_ADMIN_TOKEN` precedent, so whatever the design decides
  about file-vs-environment custody should be decided once for both
  (source: https://cwe.mitre.org/data/definitions/526.html;
  apps/registry/src/main.rs;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C3`).
- **The snapshot's `fetched_at` arithmetic is wall-clock arithmetic.** `027-B2`'s
  freshness test compares `fetched_at` against the current time, and `if_modified_since`
  is a `uint32` unix timestamp. `std::time::SystemTime` is documented "**is not
  monotonic**", with `duration_since`/`elapsed` returning `Result` precisely
  because "the system clock being adjusted either forwards or backwards" can make
  a later measurement earlier — so a backwards NTP step can make a snapshot look
  fresher than it is (re-arm `027-L3`'s `200 []`) and a forwards step can
  prematurely force `503` (source: https://doc.rust-lang.org/std/time/struct.SystemTime.html;
  https://partner.steamgames.com/doc/webapi/IStoreService;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-B2`/`027-L3`).
- **The name bound the snapshot inherits was chosen for store titles, which is
  a warning.** `027-C3` refuses the snapshot on "the name bound `013-B4`", and
  `024-B1`'s rationale for the 128-octet figure is that it is "one octet above
  the longest surveyed store display title" — i.e. the bound has no margin over
  the real distribution it was sized against, and the ADR does not say whether an
  over-long store name refuses the whole 32 MiB snapshot (a total resolver
  outage) or drops one record (a silent hole in `027-L3`'s honesty claim)
  (source: docs/adrs/024-game-slug-identity.md `024-B1`;
  docs/adrs/013-game-manifests.md `013-B4`;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C3`/`027-L3`).
- **Quota arithmetic is comfortable, and that is the point of `027-B4`.** A
  daily full pull plus hourly deltas is dozens of calls against the licensed
  100,000/day; `027-B4`'s "12 pages" is the whole quota posture "in one number"
  and there is ~6000× headroom at the documented maximum page size
  (source: https://steamcommunity.com/dev/apiterms;
  https://partner.steamgames.com/doc/webapi/IStoreService;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-B4`).
- **Minting is unbounded-throttling-shaped.** Every first-time key on the public
  unauthenticated `GET /v1/candidates` route becomes a document rewrite of
  `issued-v1.json` under the admin write lock, in a server that admits up to
  `MAX_CONCURRENT_CONNECTIONS = 64` one-request threads; the general class is
  allocation of resources without limits or throttling (CWE-770). The mint set is
  bounded by the store catalog rather than by attacker choice, which is a real
  mitigation — but write amplification is per-request regardless
  (source: apps/registry/src/constants.rs; apps/registry/src/serve.rs;
  https://cwe.mitre.org/data/definitions/770.html;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C4`/`027-B3`).
- Nothing in this ADR touches the input-event path or the Node orchestration
  toolchain, so no polling-budget or Node-version consequence applies; the new
  work stays inside `apps/registry` plus `docs/`, which the existing cargo trio
  and the `npm run check` gate already cover without a CI edit
  (source: AGENTS.md §Test gates; Cargo.toml).

## Risks & unknowns

- **The mint rule's cited algorithm does not exist as code and is recorded as
  retired.** `027-C5` → `024-C2` → amended away by `025-C2`, with the
  implementation deleted and its suites feature-gated out of every build.
  Whoever builds Rollout item 4 either revives the derivation in
  `apps/registry` (legal under the retirement sweep, which scans only
  `crates/discovery/src` and `apps/discovery-cli/src`) or invents a fourth
  spelling; the ADR should say which, and should state the mint's behavior on the
  empty-derivation and digit-initial cases `024-C2` handled by a fallback that
  store records do not have (source: docs/adrs/027-hosted-registry-with-store-resolvers.md
  `027-C5`; docs/adrs/025-discovery-identity-and-catalog-matching.md `025-C2`;
  crates/discovery/tests/retirement_tests.rs).
- **`027-B3` is not an abuse budget, it is the expected steady state.** Steam's
  games-only catalog is on the order of 164k–200k titles; every distinct
  `steam/<appid>` any user ever scans mints exactly one record, so 200,000
  issued records is roughly "every Steam game, once". At the cap, `027-B3`
  answers with a named `503` for keys the snapshot *does* know — which collides
  with `027-L3`'s promise that a within-bound snapshot is entitled to answer
  (source: docs/adrs/027-hosted-registry-with-store-resolvers.md `027-B3`/`027-L3`;
  https://gamechosis.com/en/stats/; https://github.com/Producdevity/EmuReady/pull/342).
- **Issued records are immortal but their names are not.** The mint is one-way
  by `027-L2`, so the `name` the resolver copied at mint time is what the
  registry serves forever unless an operator notices the rename — and ~2% of a
  5,854-game cohort renamed within a ~3.5-year window, meaning order of
  thousands of stale names on a ~200k catalog, with no mechanism in `027` to
  surface them (source: docs/adrs/027-hosted-registry-with-store-resolvers.md
  `027-C5`/`027-L2`; https://striketwo.games/blog/change-game-name-on-steam).
- **Nothing retires an issued record when the store drops the app.** The
  endpoint's delta mode is documented to return items "changed or added", never
  removed ones, and `027` gives only the operator's `DELETE` route as a
  removal path; whether a full (non-delta) refresh can therefore differ from a
  delta refresh in what it *deletes* is unasked in the ADR
  (source: https://partner.steamgames.com/doc/webapi/IStoreService;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C6`).
- **The transport gap is the likeliest place this design fails to build.**
  `027-C3` demands a keyed HTTPS pull (the documented host is `https://partner.steam-api.com`)
  inside a std-only binary whose only outbound client refuses `https` and caps
  responses at 64 KiB; `027-R3` disclaims transport for clients, not for this
  hop. Any resolution — a TLS dependency, a separate importer, or a local proxy —
  changes `027-C3`'s startup-pull and named-refusal contract
  (source: https://partner.steamgames.com/doc/webapi/IStoreService;
  apps/registry/src/endpoint.rs; apps/registry/src/constants.rs;
  apps/registry/Cargo.toml;
  docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C3`/`027-R3`).
- **One operator holds a personally-licensed, non-transferable key for a
  community service, under a revocable-at-will contract.** The ToS forbids
  sharing the key, is personal to a named Application, and lets Valve terminate
  without notice; the runbook (`027-C6`/Rollout 6) must therefore also carry the
  ToS's disclaimer obligation ("as is" terms with §7/§8-equivalent disclaimers),
  the no-Valve-endorsement presentation rule, and what the registry does when the
  snapshot goes stale because the key died
  (source: https://steamcommunity.com/dev/apiterms;
  docs/adrs/027-hosted-registry-with-store-resolvers.md Rollout 6).
- **Three documents, three writers, one named lock.** `027-C4` serializes
  issuance against admin mutations; the snapshot refresh thread is a fourth actor
  on the same data directory whose interaction with the fixed-tmp `create_new`
  discipline and with a concurrent `issued-v1.json` rewrite is unstated, as is
  whether two hosted processes against one data directory remain a loud failure
  (source: docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C4`;
  apps/registry/src/persist.rs).
- **Two contradictory orderings in the same ADR** (`027-C4`'s persist-before-serve
  versus Rollout 4's "answer-before-persist") decide the one property that
  matters for `027-L2` — an identity that was served but not persisted can be
  minted twice from a later query, and one that was persisted but not served is a
  silent record. This must be resolved, not averaged
  (source: docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C4`,
  `027-L2`, Rollout 4).
- **The inactive-resolver and never-resolved-token postures are stated twice,
  differently.** `027-C2` makes an inactive resolver "a named miss", `027-L3`
  speaks only of steam keys and snapshot age, and `026-L4`'s cold `503` still
  governs the general case; what an `epic/…` or `local/…` query answers on the
  hosted instance (named `503`? `200 []`? the same refusal?) is not pinned, and
  the shipped CLI renders whatever it is as only the status line
  (source: docs/adrs/027-hosted-registry-with-store-resolvers.md `027-C2`/`027-L3`;
  docs/adrs/026-registry-service.md `026-L4`; apps/discovery-cli/src/catalog.rs).
- **Facts in the ADR that the tree contradicts or that the tree has not caught
  up to:** "Five shipped seed entries" against six shipped records; a shipped
  seed `epic/` key in a format `027-R1` calls undecided; and Rollout item 1's
  ledger edits being the same class of work as the `amended-by` lines ADR-026's
  own research flagged as the most skimped step
  (source: data/catalog/v1/catalog.json;
  docs/adrs/027-hosted-registry-with-store-resolvers.md Context and Rollout 1;
  docs/designs/026-registry-service/01-research.md §Amendment and consequences).

## Open questions

1. Does `027-C5` revive `024-C2` (which `025-C2` retired, and whose
   implementation is deleted), or does `027` mint under a new derivation stated
   in full in this ADR? If it revives, where does the code live, and do the
   feature-gated `slug_tests.rs` vectors become the mint's test record?
2. What does the mint do for a store name that derives to the empty string
   (CJK/Cyrillic/emoji-only titles) — refuse that one record, mint
   `g-steam-<appid>`-style, or fall back to the appid? And is the `g-` leading
   digit prefix part of `027-C5`'s rule or not?
3. What does a snapshot do with a `name` over `013-B4`'s bound, or with a
   record whose `appid` is not a decimal integer: refuse the file, drop the
   record, or refuse startup? How is a dropped record reconciled with `027-L3`'s
   claim that a within-bound snapshot may answer `200 []`?
4. Is the snapshot's ascending-`appid` order load-bearing for correctness (a
   diff, a resume cursor) or only conventional, and does the loader refuse an
   out-of-order file?
5. Is `max_results` pinned explicitly in every pull, and is `027-B4`'s 12-page
   budget measured against the 50k maximum page or the 10k default? What happens
   when a refresh exhausts the budget with `have_more_results` still true — serve
   the partial snapshot, refuse it, or mark it stale so `027-L3` answers `503`?
6. What are the *page* budgets (response octets, records per page, connect/read
   timeouts, redirect policy, retry/backoff on `429`/`5xx`)? Is the 64 KiB
   `MAX_RESPONSE_OCTETS` reused for the store hop, and if not, which constant
   family does the new one join?
7. Where does HTTPS to `partner.steam-api.com` come from in a std-only crate, and
   does the answer stay inside `027-C3`'s "pull in full before the process
   binds" contract or move the pull to an operator-run importer? Is the key ever
   permitted on a plaintext link, including to `api.steampowered.com` port 80?
8. Is the `key` a query parameter (as documented) or a header, and what is
   scrubbed from the registry's own diagnostics so a failed pull cannot print it?
   Should the key stay in the environment (CWE-526) or move to a `0600` file for
   both this secret and the admin token?
9. Which clock does `027-B2`'s staleness test read, and what happens on a
   backward clock step: does a future-dated `fetched_at` re-arm `027-L3`'s
   `200 []`? Is `fetched_at` validated against plausibility at load, like
   `026`'s cache `fetched_at` was?
10. Who owns the refresh thread's lifecycle — does `query <slug>` start it, does
    `--no-steam` suppress it entirely, and does a failed background re-pull leave
    the old snapshot serving until `027-B2`'s age bound expires, or fail loudly?
11. At `027-B3`'s cap, does a key the snapshot knows answer from the snapshot
    without minting, or does the cap turn a known key into a `503`? Should the
    cap be raised above the catalog size, or should issuance be lazy in some
    other way (mint on confirmation rather than on first answer)?
12. Is minting per *queried key* unauthenticated and unthrottled? What is the
    cost ceiling per request when the mint rewrites a multi-megabyte issued
    document under the admin write lock, and is a per-key negative cache or
    batched write acceptable?
13. Does a *full* refresh delete snapshot records the store no longer lists, and
    does that retire or merely orphan the issued records minted from them? How
    would an operator find stale-name or dead-app issued records to curate
    (`GET /v1/admin/stats` gains only a count)?
14. What is the exact answer for an `epic/…` key (inactive resolver) and a
    `local/…` key on the hosted instance, which status code, and what does the
    shipped CLI show the user for each?
15. Which `026` units get `amended-by` versus retirement lines under `021-L4`
    (`026-C1`, `026-C2`, `026-C6`, `026-C7`, `026-B1`–`026-B3`, `026-L4`,
    `026-R1`, `026-R2`), and does `026-L2`'s endpoint-choice consent need a
    sentence now that the registry holds a third-party dataset rather than the
    user's own registrations?
16. Does the corpus file stay restart-only or become hot-reloadable now that the
    operator's whole curation workflow (promote, then delete the issued record)
    runs against a live hosted instance, and does `--corpus` custody differ from
    `issued-v1.json` custody in the runbook?
17. Does the runbook (Rollout 6) carry the Steam ToS obligations by name —
    named-Application registration, key confidentiality and non-transferability,
    the "as is" disclaimer requirement, the no-endorsement presentation rule —
    and does it state the operator-rotation answer the license forecloses?
