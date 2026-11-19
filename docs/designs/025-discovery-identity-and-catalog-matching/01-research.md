# Research: Split discovered-game identity into a source key, a curated game id, and a friendly name (ADR-025)

## Summary

ADR-025 replaces ADR-024's name-derived discovery identity with three
separated identifiers: a source-unique lookup key `slug =
<source>/<external-id>` emitted by source plugins, a curated cross-source
`game_id` (the manifest `id` of ADR-013) that arrives only through a
user-confirmed candidate lookup, and a friendly name demoted to display and
search metadata. Discovery gains a connectivity gate and a per-user local
match store, both explicit overrides of ADR-022 stances. The external
identifiers the decision leans on are real and source-maintained — Steam's
AppID is documented by Valve as the unique id of an application, and Epic's
launcher records a `CatalogItemId` beside each install — but Epic's own docs
say a catalog item id is unique only *within a product namespace*, which
sharpen the ADR's own `025-R2` reservation: the `epic` external-id will
likely need its namespace. The codebase today is exactly the ADR-024 shape
the ADR describes as landing "today": slug from display name, a
`Namespace::Unknown` field, grammar-and-bound checks at both emit seams, and
no AppID read anywhere in the Steam plugins.

## Findings

### What the ADR changes against what is actually shipped

- `DiscoveredGame` currently has fields `slug`, `namespace: Namespace`,
  `display_name`, `install_path`, `platform`, `codec_id`; `Namespace` is a
  closed kebab-case serde enum whose sole value is `Unknown`, and its doc
  comment already says "ADR-025 owns any second value" — removing the enum
  (`025-C1`) breaks its serde/display tests directly
  (source: crates/discovery/src/game.rs).
- The `024-B1` bound already landed: `MAX_SLUG_OCTETS = 128` is exported
  from `bindforge_core`, and `crates/discovery/tests/zz_api_probe.rs` pins
  the value to 128 and even pins the public import line — the new
  `MAX_DISCOVERY_SLUG_OCTETS` constant (`025-B1`) beside it must be added to
  those API-probe pins (source: crates/discovery/tests/zz_api_probe.rs,
  crates/core/tests/placement.rs).
- The bound's one stated home is the `schemas/README.md` budget family
  bullet, which ADR-024 replaced the RFC 1035 §2.3.1 derivation with
  (source: schemas/README.md §"Input budget and chain bound";
  https://www.rfc-editor.org/rfc/rfc1035#section-2.3.1). `025-B1` is stated
  as "set equal … so the constant family reads as one, though it caps a
  lookup key, not a grammar-validated id", so the README bullet gains a
  sibling statement that is explicitly *not* a grammar bound
  (source: docs/adrs/025-discovery-identity-and-catalog-matching.md
  §Budgets; schemas/README.md).
- Slug emission is checked twice today: once inside the Steam scan core
  (`is_slug_name(&slug) || slug.len() > MAX_SLUG_OCTETS` → drop + count
  into `skipped`) and once as the CLI's last line in
  `refuse_invalid_slugs` (source: crates/discovery/src/steam_common.rs,
  apps/discovery-cli/src/main.rs). Under `025-L3` both checks become bound-
  only (`025-B1`), since `steam/123` fails the product slug grammar by
  design (source: docs/adrs/024-game-slug-identity.md §Context).
- The Steam plugins deliberately never read the AppID: the shared scan core
  reduces each parsed `appmanifest_*.acf` to `AppManifest { display_name,
  install_dir }`, and `steam_linux.rs`'s module doc states "AppIDs are never
  read" — a holdover of ADR-022's rejection of AppID usage
  (source: crates/discovery/src/steam_common.rs;
  crates/discovery/src/steam_linux.rs; docs/adrs/022-game-discovery.md
  §Alternatives). The ACF fixtures *do* carry `"appid"` keys and the crate's
  KeyValues parser already reads quoted `appid` values in tests, so emitting
  `steam/<appid>` is a reduction-step change, not a parser change
  (source: crates/discovery/tests/fixtures/steam/minimal/steamapps/
  appmanifest_900001.acf; crates/discovery/tests/keyvalues_tests.rs).
- `derive_slug` (the `024-C2` algorithm, retired to "at most a display
  convention" by `025-C2`) is a single module whose only consumers are the
  Steam core and its tests, so retirement is contained
  (source: crates/discovery/src/slug.rs; crates/discovery/src/
  steam_common.rs).
- The CLI prints columns `Slug | Namespace | Display Name | Install Path |
  Platform` in table mode and a byte-stable JSON array of `DiscoveredGame`
  in `--json` mode; T-003 pinned that JSON key set as a compatibility
  contract, and `025-C4`'s "candidates instead of prompting" JSON mode adds
  a second document kind to the same stdout stream — the output contract
  needs a deliberate re-pin (source: apps/discovery-cli/src/main.rs;
  docs/tasks/T-003-polish-discovery-cli-output.md).
- `DiscoveryError` has exactly two string-carrying variants, `PluginError`
  and `RegistryError`; the named connectivity refusal of `025-L1` has no
  home yet — a new variant (or new error type at the CLI/engine seam) is
  implied (source: crates/discovery/src/registry.rs).
- Discovery has no persistence of any kind and no data-directory concept
  anywhere in the crate, so the `025-C3` match store is the product's first
  per-user state file and must define that convention from zero
  (source: docs/adrs/022-game-discovery.md §Consequences; grep of
  crates/discovery).
- Ledger obligations ride rollout items 5–6: `024-C1/C2/L1/L2` gain
  `amended-by` lines per `021-L4`, and `024-R1` records ADR-025 as filling
  its slot with the publisher-namespace half superseded (source:
  docs/adrs/021-decision-unit-ledger.md §Laws; docs/adrs/
  024-game-slug-identity.md §Reservations; docs/adrs/
  025-discovery-identity-and-catalog-matching.md §Rollout).

### The external identifiers the decision names

- Steam: "Each App is represented by a unique ID called an App ID" —
  Valve's own documentation, and it is what the Steamworks API and Web API
  key on (source: https://partner.steamgames.com/doc/store/application).
  The store-side name that ADR-024's slug fought with rides the same app
  record (`name` beside `steam_appid` in the store's own appdetails API),
  which is exactly the rename-instability split ADR-025 exploits (source:
  https://github-wiki-see.page/m/Revadike/InternalSteamWebAPI/wiki/
  Get-App-Details).
- Steam ACF on-disk shape: file named `appmanifest_<appid>.acf`, root
  section `AppState` with `appid`, `name`, `installdir` as sibling string
  keys — the `<appid>` also appears in the file name, giving two agreeing
  sources for the id (source:
  https://github.com/leovp/steamfiles/blob/master/docs/acf_overview.rst).
- Epic launcher install metadata: each install is recorded as a JSON
  `.item` file in `%PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests`
  carrying `CatalogItemId`, `CatalogNamespace`, `AppName`, `DisplayName`
  and `InstallLocation` side by side (source:
  https://jayd.ml/games/2020-05-16/epic-games-store-steam-libraries.html),
  and `%PROGRAMDATA%\Epic\UnrealEngineLauncher\LauncherInstalled.dat`
  records each install as `NamespaceId` + `ItemId` + `AppName` (source:
  https://github.com/Utopia1916/EpicGamesManager). The in-game `.egstore`
  folder holds a `<guid>.manifest` plus a `<guid>.mancpn` JSON naming
  `CatalogItemId` and `CatalogNamespace` (source:
  https://git.amogus.cloud/mirrors/legendary/raw/commit/
  054a3ea7ebab94deb17925074d05e07176a25089/legendary/core.py).
- **Epic uniqueness (directly loads on `025-R2`):** Epic's glossary defines
  the catalog item ID as "A unique identifier for an audience item … Each
  catalog item ID is unique in a product" — product/namespace-scoped, not
  store-global (source: https://dev.epicgames.com/docs/glossary). The EOS
  `EOS_Ecom_CatalogItem` struct carries `CatalogNamespace` ("Product
  namespace in which this item exists") beside `Id`, and the Ecom Web API
  ownership endpoint keys on `nsCatalogItemId={{sandboxId:catalogItemId}}`
  — the pair, not the bare id (source:
  https://dev.epicgames.com/docs/api-ref/structs/eos-ecom-catalog-item;
  https://dev.epicgames.com/docs/web-api-ref/ecom-web-apis). Epic's own
  manifest API likewise looks up by `(namespace, catalog_item_id,
  app_name)` (source: legendary core.py, URL above).
- The manifest `id` as join target: ADR-013's `013-L4` makes the manifest
  identity its `id` plus revision, and `013-C2` pins the directory name to
  the `id` byte-for-byte — so the curated `game_id` of `025-C3` is already
  a grammar-checked, path-bearing id, which is what `025-L5`'s corpus
  boundary leans on (source: docs/adrs/013-game-manifests.md §Contracts,
  §Laws).

## Prior art

( lens: prior art — how comparable projects split "which install", "which
game", and "what to print" )

- **Playnite** runs the same three-role split, un-curated: each library
  plugin reports a provider id (`GameId` — "Gets or sets provider id. For
  example game's Steam ID") plus the importing `PluginId`, while the
  library's own database row is the identity it owns
  (source: https://api.playnite.link/docs/api/Playnite.SDK.Models.Game.html;
  https://api.playnite.link/docs/tutorials/extensions/libraryPlugins.html).
  Because no curated join step exists, the same game on two stores and the
  same title from different games both surface as duplicates the user
  fights — issues #217 ("Make same game available on multiple platforms
  appear as one entry") and #383 ("Better handling of game duplicates")
  (source: https://github.com/JosefNemec/Playnite/issues/217;
  https://github.com/JosefNemec/Playnite/issues/383).
- **GOG Galaxy 2.0** is the negative control for `025-L2`: its automatic
  cross-store merge fuses by title and both fuses wrongly ("GOG Galaxy
  combining two different games with the same name" — Final Fantasy IV
  Pixel Remaster merged with the 3D remake) and fails to fuse equivalents,
  driving a years-long user demand for a manual-merge control
  (source: https://www.gog.com/forum/general/gog_galaxy_combining_two_
  different_games_with_the_same_name; https://www.gog.com/forum/
  general_beta_gog_galaxy_2.0/manual_merge_for_any_games). This is the
  "fuse only by guess" failure ADR-025's Context cites, observed in the
  wild.
- **Wikidata** models the exact triple: a curated entity id (`Q49740`), a
  `title`/description as metadata, and the store id as a foreign key
  (`steam_appid`), to the point of exposing a reverse-lookup route
  `/v1/games/steam/{appid}` — a "candidates by source id" endpoint in all
  but the confirmation step (source:
  https://dataandapis.com/apis/video-games). **PCGamingWiki** does the same
  inside a Cargo table: curated game pages indexed by a `Game.Steam_AppID`
  column, queried as the lookup key
  (source: https://www.pcgamingwiki.com/wiki/PCGamingWiki:API).
- **MusicBrainz** is the long-lived proof of the curation posture ADR-025
  picks: identifiers (MBIDs) are assigned and merged by a community
  editorial process, never derived from names, and names stay mutable
  metadata — with the maintenance intake as the acknowledged permanent cost,
  which is what `025-R1` reserves rather than designs
  (source: https://musicbrainz.org/doc/MusicBrainz_identifier).
- **Format choice in this repo's lineage:** the slug-with-embedded-
  namespace (`unknown/<kebab>`) that ADR-022 shipped and ADR-024 half-
  retired is the only slash-bearing "id" in the product; every other
  id-family — the input id `<namespace>:<member>` of the format contract —
  uses `:` as its separator and states its grammar once
  (source: schemas/README.md §"one vocabulary spans the formats";
  docs/adrs/022-game-discovery.md §Context; docs/adrs/024-game-slug-
  identity.md §Context). No precedent exists yet for a `/`-bearing string
  in the product, and `025-L5` is what keeps it that way.

## Platform notes

( lenses: platform notes, standards )

- **Steam layout is stable across OSes for the external-id**: both backends
  parse `appmanifest_*.acf` under each library's `steamapps` (ADR-022's
  shipped decision), the Linux root coming from `libraryfolders.vdf` and
  the Windows one from the registry probe (source: docs/adrs/022-game-
  discovery.md §Decision; crates/discovery/src/steam_common.rs;
  https://github.com/leovp/steamfiles/blob/master/docs/acf_overview.rst).
  Community parsers note ACF key casing varies in the wild and recommend
  case-insensitive key comparison; the shipped KeyValues parser's public
  lookup used in tests is exact-case (`get_str("appid")`) — worth pinning a
  case-insensitive read for the appid specifically
  (source: https://reverseengineering.wiki/index.php/Format:Steam_client_
  data_files; crates/discovery/tests/keyvalues_tests.rs).
- **Epic has no Linux launcher**; third-party tooling on Linux reads the
  Windows `ProgramData` layout through a Wine prefix or keeps its own
  `~/.config/legendary/installed.json`
  (source: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/
  wiki/Troubleshooting; https://github.com/mtkennerly/ludusavi/issues/94).
  Whichever surface the `epic` token (`025-R2`) reads, the ADR's note that
  the backend "lands" later is load-bearing for this run — the closed token
  set `steam · epic · local` can ship with `epic` unpopulated
  (source: docs/adrs/025-discovery-identity-and-catalog-matching.md
  §Reservations).
- **Path-component relief and its limits.** With `025-L5`, source keys and
  friendly names stop being path components; only the game id keeps riding
  the 255-octet-per-component floor of ext4, APFS and NTFS
  (source: https://ext4.wiki.kernel.org/index.php/Ext4_Disk_Layout#
  Filename_Limits; https://learn.microsoft.com/en-us/windows/win32/
  fileio/naming-a-file#maximum-path-length-limitation; schemas/README.md
  ADR-024 bullet). The flip side: ext4 allows 255-octet file names, so a
  `local` install-directory name can kebab past `128 − len("local/")` and
  hit the named refusal at the emit boundary — dropped row, not truncated
  identity (source: ext4 wiki URL above; docs/adrs/025-discovery-identity-
  and-catalog-matching.md §Laws `025-L3`).
- **Case-insensitive filesystems and `local` slugs.** NTFS is "case-
  insensitive but case-preserving", so two install directories differing
  only in case kebab to one `local/<slug>`; `025-L3` explicitly makes `local`
  exempt from uniqueness, which is consistent, but the match store keyed by
  slug would hold one record for both (source:
  https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file).
- **Match-store location.** The per-user file (`025-C3`) has no existing
  product convention to follow; the two platform norms a Linux/Windows
  product reaches for are `XDG_DATA_HOME` (`~/.local/share`, fallback
  `~/.local/share/`) and the `FOOLDERID_RoamingAppData` known folder —
  plus the Flatpak/Vulkan-sandbox redirect (`~/.var/app/<id>/`) that
  game-adjacent tooling already handles (source:
  https://specifications.freedesktop.org/basedir-spec/latest/;
  https://learn.microsoft.com/en-us/windows/win32/properties/known-folders;
  https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/wiki/
  Troubleshooting).
- **URI spelling of the candidate query.** `025-C4`'s
  `GET <catalog>/v1/candidates?slug=<slug>` carries a `/` inside a query
  value; `/` is legal in a query component without encoding
  (RFC 3986 §3.4 allows `/` after the `?`), but the contract should say
  whether the slug arrives percent-encoded (`steam%2F123456`) or literal,
  since a server-side router that normalizes paths will see `?slug=steam/
  123456` as safe but a stricter gateway may not
  (source: https://www.rfc-editor.org/rfc/rfc3986#section-3.4).
- **Budget arithmetic for `025-B1` at 128:** `steam/` + a numeric AppID
  fits with enormous headroom; the Epic pair spelled
  `epic/<namespace>/<itemId>` is `5 + 32 + 1 + 32 = 70` octets on the
  observed 32-hex-char id shape — inside 128 (source: https://jayd.ml/
  games/2020-05-16/epic-games-store-steam-libraries.html; docs/adrs/
  025-discovery-identity-and-catalog-matching.md §Budgets).
- Nothing in this ADR touches the Node orchestration toolchain or the
  input-event path; no Node-floor or polling-budget consequence applies
  (source: AGENTS.md §Repo layout).

## Risks & unknowns

- **`epic/<CatalogItemId>` may be ambiguous at the source.** Epic's own
  docs scope catalog-item uniqueness to a product/namespace (see Findings);
  if namespaces are ever reused or items re-listed across namespaces, the
  `epic` external-id either collides or must grow to
  `epic/<namespace>/<itemId>` — the ADR already reserves this question as
  `025-R2` with owner `unassigned`, and the evidence here says the
  namespace-pair form is the safer provisional spelling if anything is
  emitted this run (source: https://dev.epicgames.com/docs/glossary).
- **Re-listing churn is asserted, not measured.** The ADR's claim that
  "re-listings mint new ids" (Steam AppID / Epic CatalogItemId) is the cost
  that keeps the confirmation UX honest, but no source surveyed here
  quantifies how often stores mint a fresh id for an existing game; the
  only hard external signal is that Epic's `AppName` codenames changed
  shape across eras (readable → UUID), hinting id hygiene is not frozen
  (source: https://jayd.ml/games/2020-05-16/epic-games-store-steam-
  libraries.html; docs/adrs/025-discovery-identity-and-catalog-matching.md
  §Context).
- **Disclosure surface.** Every scan queries the catalog for every
  unresolved slug — the user's installed-game inventory, keyed by
  third-party ids, leaves the machine on a schedule the user controls but
  cannot inspect; the ADR concedes this and narrows the query shape instead
  of gating it (source: docs/adrs/025-discovery-identity-and-catalog-
  matching.md §Consequences).
- **Amendment-within-a-day churn.** `024-C1/C2/L1/L2` and their fixtures
  re-pin on the same day they were written; the ledger mechanics
  (`021-L4` amended-by lines, `024-R1` half-superseded) are the part of the
  rollout most likely to be skimped and are mechanically checkable
  (source: docs/adrs/021-decision-unit-ledger.md §Laws; docs/adrs/
  025-discovery-identity-and-catalog-matching.md §Rollout item 5).
- **No server exists to test against.** `025-C4` is a client-side
  provisional contract until ADR-026 lands; the run needs an in-repo fake
  (fixture server or trait seam) to pin the connectivity gate, candidate
  rendering, and JSON passthrough without a live service (source:
  docs/adrs/025-discovery-identity-and-catalog-matching.md §Reservations).
- **Match-store shape is specified only by three field names.** Format
  (TOML vs JSON), corruption behavior, concurrent-scan writes, and the
  per-user path are all open; the product has no prior state file to copy
  (source: docs/adrs/025-discovery-identity-and-catalog-matching.md
  `025-C3`; grep of crates/discovery for any persistence).
- **Stale records on re-listing.** The store keys on `slug`; when a source
  mints a new external-id the old record goes unreachable (never resolved
  again, never deleted) — the ADR accepts the re-confirmation cost but not
  an explicit garbage-collection rule (source: docs/adrs/025-discovery-
  identity-and-catalog-matching.md §Consequences).

## Open questions

1. Epic external-id: does this run ship any `epic/` spelling at all
   (backend is `025-R2`, unassigned), and when it lands is it
   `epic/<CatalogItemId>` or `epic/<CatalogNamespace>/<CatalogItemId>`?
   Epic's glossary ("unique in a product") argues for the pair.
2. Match store: file format, exact path per OS (XDG data dir? `%APPDATA%`?
   the Flatpak redirect?), and behavior on unparseable content — refuse the
   scan, or start empty and re-confirm?
3. Connectivity gate placement: inside `crates/discovery` (engine) or in
   `apps/discovery-cli` before plugins run — and is the named error a new
   `DiscoveryError` variant or a CLI-level error type? (Two string variants
   exist today.)
4. Which Epic on-disk surface is canonical when the backend lands: the
   `ProgramData …\*.item` JSON, the in-install `.egstore/*.mancpn`, or
   `LauncherInstalled.dat`? They agree on fields but differ in freshness
   and sandboxing reach.
5. Does the candidate query percent-encode the slug's `/`
   (`steam%2F123456`), and is one HTTP call per unresolved slug required
   or may the client batch? The ADR's call shape is singular.
6. JSON-mode shape for candidate passthrough: candidates nested per row in
   the existing `DiscoveredGame` array, or a separate top-level structure?
   T-003 pinned the current array byte-for-byte, so any answer is a
   contract change to pin.
7. When a `local` install-directory name kebab-overs the 128-octet slug
   budget, is the named-refusal drop acceptable (game silently absent from
   its own machine, modulo stderr), or does the design want a fallback?
8. Friendly name: `025-C2` says it rides the row but is never user-visible;
   does it persist in discovery JSON output at all, and does the match
   store's `friendly_name` record feed search anywhere in this run, or is
   it write-only metadata until the UI exists?
9. Steam appid read: exact-case `appid` (current parser tests) or
   case-insensitive per community-parser warnings? The ACF fixture set is
   uniform, so a test gap is easy to create here.
10. Is there a cap or pagination story expected on candidate count
    ("zero or more candidates") for the CLI's confirm prompt before
    ADR-026 defines the server?
