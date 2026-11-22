# Design: Serve the catalog from a hosted registry backed by store snapshots

## Context

ADR-026 shipped a two-mode registry — a curated root and a user-hosted
caching instance — and five-to-six seed entries as its entire content story.
ADR-027 retires the caching mode, makes one **hosted** posture the only mode
`registry serve` admits (`027-C1`), puts a `steam` snapshot resolver behind
the cache seam the crate already has (`027-C2`/`027-C3`), and mints a
permanent `game_id` the first time a resolver answers a key (`027-C4`/
`027-C5`) — with stores held off the request path (`027-L1`) and off the
disclosure seam (`027-L4`), the miss posture made honest about snapshot age
(`027-L3`), and Epic kept as an inert reserved slot (`027-R1`). The wire
contract the shipped CLI pins (`025-C4`, `026-C2`'s candidate face) is
untouched. Link:
[`docs/adrs/027-hosted-registry-with-store-resolvers.md`](../../adrs/027-hosted-registry-with-store-resolvers.md).

## Goals / Non-goals

- Goals:
  - One hosted mode (`027-C1`): `serve --root --corpus` plus resolver flags;
    `--upstream` / `--refresh-interval` / the cache-document pair become
    named startup refusals; health re-pins to `mode: hosted` with the
    `steam_snapshot` fact.
  - A steam snapshot resolver (`027-C3`) whose request-path answer is an
    exact-`appid` lookup in a persisted, budget-bounded document, refreshed
    only by the operator's own bulk pull (`027-L1`, `027-L4`).
  - Issued entries (`027-C4`, `027-C5`): a second `026-C3` document, the
    mint rule with its collision and degenerate-name spellings,
    persist-before-serve under the mutation lock, and `027-L2` identity
    bookkeeping ensuring that deleted ids never return.
  - The miss's honesty (`027-L3`) pinned as a decision table, including the
    epic (inactive-resolver) and `local/` (never-resolved) postures.
  - The curation face (`027-C6`): `DELETE /v1/admin/issued/<game_id>` and
    the issued count in `stats`.
  - A keyed HTTPS pull that stays inside `027-C3`'s startup-pull and
    background-refresh contract without touching the std-only serving face.
- Non-goals (scope guards for code review):
  - No epic resolver (`027-R1`); no profile upload/download/sharing or auth
    beyond the bearer token (ADR-028, `027-R2`).
  - No client-side TLS or transport assurance (`027-R3`, carrying
    `026-R1`); no change to `apps/discovery-cli` at all — the wire contract
    survives verbatim.
  - No CLI self-resolution of `local/<dir>` keys (the separate amendment the
    ADR's Consequences leaves outside its ledger).
  - No corpus hot-reload (the `--corpus` file stays restart-only); no tooling
    to surface stale-name or dead-app issued records; no issued-document
    batched writes or raised mint cap (budget amendment territory, §Deferred).

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | The shipped health document is the enumerated list `status`, `mode`, `upstream`, `local_entries`, `cached_slugs`, `last_upstream_fetch`, and `Mode` has exactly the `root`/`caching` inhabitants. | verified | apps/registry/src/state.rs (`health`, `Mode`) |
| A2 | The persist discipline is a *fixed* `<document>.tmp` opened `create_new` at `0600`, parent `0700`, rename into place; `<data>` is absolute `$XDG_DATA_HOME`, else `$HOME/.local/share`, else `%APPDATA%`; a relative value is unset; catalog-document reads are metadata-gated against their octet cap *before* the file is read. | verified | apps/registry/src/persist.rs; apps/registry/src/constants.rs (`MAX_CATALOG_DOCUMENT_OCTETS`) |
| A3 | `IStoreService/GetAppList/v1` lives at `https://partner.steam-api.com`, requires the `key` parameter, returns `Game` items only unless opt-in `include_*` flags are set, orders by `appid`, continues pages with `last_appid` and `max_results` (default 10k, max 50k), and `if_modified_since` filters to items changed or added since a unix timestamp. Valve's page documents the request side only. | verified | https://partner.steamgames.com/doc/webapi/IStoreService |
| A4 | The unkeyed `ISteamApps/GetAppList/v2` is formally deprecated in favor of `IStoreService/GetAppList`, so the keyed endpoint is the only supported bulk surface. | verified | https://partner.steamgames.com/doc/webapi/isteamapps |
| A5 | The games-only Steam catalog is on the order of 164k–200k titles in late 2026 (third-party counts, not a Valve-published figure). | unverified | https://gamechosis.com/en/stats/; https://shelfwatch.net/stats; https://github.com/Producdevity/EmuReady/pull/342 |
| A6 | The Web API license permits redistributing store data to end users through the licensee's named Application, is personal and non-transferable, revocable at will, requires data provided on "as is" terms with disclaimers substantially equivalent to the Terms' Sections 7 and 8 and no-endorsement presentation, and allots 100,000 calls per key per day. | verified | https://steamcommunity.com/dev/apiterms |
| A7 | `apps/registry` depends on `std`, `serde_json`, `bindforge-core` and `discovery` only; its outbound client is hand-rolled HTTP/1.1 over `TcpStream`, and its shipped endpoint gate refuses `https` by name. `discovery::game::friendly_name` (no crate-root re-export) is that crate's only name normalization — whitespace collapse plus the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`, and nothing else: `split_whitespace` leaves non-whitespace control bytes (NUL, ESC, DEL, the C1 range) in place — admissible there because nothing in that crate parses or joins the result. | verified | apps/registry/Cargo.toml; apps/registry/src/upstream.rs; apps/registry/src/endpoint.rs; crates/discovery/src/game.rs (`friendly_name`); crates/discovery/src/lib.rs (module list) |
| A8 | `MAX_RESPONSE_OCTETS = 65_536` bounds both directions of the *candidate* face; `MAX_CANDIDATES = 256`; `MAX_CONCURRENT_CONNECTIONS = 64`, one thread per admitted connection. | verified | apps/registry/src/constants.rs; apps/registry/src/serve.rs |
| A9 | `024-C2` is an ordered derivation (case-fold; Latin-1/Latin-Extended-A single-base-letter fold; non-`[a-z0-9]` → `-`; hyphen-run collapse; trim; `g-` prefix for a leading digit; empty → fallback), was amended out of law by `025-C2`, its implementation is deleted, its suite table sits at `crates/discovery/tests/slug_tests.rs` behind the never-default `retired-adr-024-suites` feature, and the retirement gate scans only `crates/discovery/src` and `apps/discovery-cli/src`. | verified | docs/adrs/024-game-slug-identity.md `024-C2`; docs/adrs/025-discovery-identity-and-catalog-matching.md `025-C2`; crates/discovery/tests/retirement_tests.rs; crates/discovery/Cargo.toml |
| A10 | The `game_id` grammar is `is_slug_name` (first byte ASCII-lowercase, bytes in `[a-z0-9-]`) with `MAX_SLUG_OCTETS = 128`, and store display names are bounded by `MAX_NATIVE_NAME_OCTETS = 128` (`013-B4`), a figure whose stated rationale is one octet above the longest surveyed store title. | verified | crates/core/src/placement.rs (`MAX_SLUG_OCTETS`), crates/core/src/manifest.rs (`MAX_NATIVE_NAME_OCTETS`); docs/adrs/024-game-slug-identity.md `024-B1` |
| A11 | About 2% of a 5,854-release indie cohort carries a different store name within ~3.5 years, appids survive renames, and games have renamed existing appids to impersonate hits. | unverified | https://striketwo.games/blog/change-game-name-on-steam; https://www.gamingonlinux.com/2024/03/steam-had-a-bit-of-a-scam-problem-with-fakes-of-helldivers-2-and-palworld/ |
| A12 | `SystemTime` is not monotonic; clock steps can make a later measurement earlier, so freshness is wall-clock arithmetic with explicit tolerances. | verified | https://doc.rust-lang.org/std/time/struct.SystemTime.html |
| A13 | The shipped seed `data/catalog/v1/catalog.json` holds **six** records (the ADR's Context says five), one carrying an `epic/` key spelled as a 32-hex-digit `CatalogItemId` — a format `027-R1`/`025-R2` still call undecided — answered today by curation alone. | verified | data/catalog/v1/catalog.json |
| A14 | Cleartext transmission of a key in a URL query string (CWE-319/CWE-1428) and cleartext secrets in environment variables (CWE-526) are named weakness classes whose mitigations are transport encryption and protected-file custody respectively. | verified | https://cwe.mitre.org/data/definitions/319.html; https://cwe.mitre.org/data/definitions/1428.html; https://cwe.mitre.org/data/definitions/526.html |
| A15 | The shipped suites that pin the retiring caching behavior are `tests/caching_policy.rs`, `tests/cache_document.rs`, `tests/upstream_client.rs`, `tests/upstream_endpoint_gate.rs`, and the caching sections of `tests/health_route.rs`, `tests/data_layout.rs`, `tests/cli_end_to_end.rs`. | verified | apps/registry/tests listing |
| A16 | The cargo gate is one Linux job that cross-`cargo check`s the *whole workspace* for `x86_64-pc-windows-msvc` — legitimate there only because `cargo check` generates no code — and otherwise builds and tests on the host target; no macOS target is built at all; `README.md` presents the cargo gate as needing none of the listed system dependencies. | verified | .github/workflows/check.yml; README.md |
| A17 | The `GetAppList` response is an envelope whose `apps` list carries per-record `appid`, `name`, `last_modified`, and `price_change_number`, beside a `have_more_results` continuation flag. | unverified | third-party clients of the endpoint (e.g. https://github.com/Producdevity/EmuReady/pull/342) — Valve's page (A3) documents no response body |

Body claims below that rest on A5/A11/A17 stay worded as estimates from
third-party counts, single studies, and third-party clients of the endpoint;
the design's budgets and risk postures are the ADR's
own units, which stand regardless of where in the 164k–200k band the catalog
sits, and an envelope that turns out not to be A17's is the named
malformed-body refusal, not a silent misread.

## Options considered

**Plaintext pull to `api.steampowered.com` on port 80.** The public Web API is
documented as reachable over plain HTTP, which would keep the crate
TLS-free. Rejected: it puts the `key` in a URL query string on an
unencrypted link (A14, CWE-319/CWE-1428), and `027-R3`'s edge deferral
covers the *client-facing* hop, never this one.

**Out-of-process importer (`registry import-steam` run by operator cron)
writing the snapshot file.** Rejected: `027-C3` pins the pull *inside* the
instance — load-or-pull "before the process binds", "a background thread
re-pulls incrementally" — so a separate command amends the contract it is
meant to implement.

**In-process HTTPS pull over `rustls`** (chosen). The store hop gets
`rustls` (no default features, `ring` provider) plus `webpki-roots` at named
floors rather than "pinned versions": `rustls ≥ 0.23.46`
(RUSTSEC-2026-0346, `Debug` disclosing TLS secrets), `rustls-webpki ≥
0.103.16` / `≥ 0.104.0-alpha.8`, and `webpki-roots ≥ 1.0.10` with its 0.26.x
compat package resolving to a 1.x parent at least that high
(RUSTSEC-2026-0381/-0382, cert-path acceptance of distrusted intermediates —
the tooling to impersonate the authority carrying our key). The cargo trio
screens no advisories, so Rollout 3 lands a RUSTSEC screen beside them. All
of it sits behind the `steam.rs` pull seam only; HTTP/1.1 framing is
hand-rolled there exactly as `upstream.rs` hand-rolled it over `TcpStream`,
so the response discipline (caps by name, no `Transfer-Encoding`, no
redirects) stays ours. `ring` compiles C and assembly for the *target*
through a build script — not the "pure-Rust" stack this section claimed — so
Rollout 3 names its price: the shipped whole-workspace
`x86_64-pc-windows-msvc` compile leg (A16) drops `apps/registry`, which is
compile-checked on the host instead, and `README.md`'s prerequisites gain a C
toolchain. A higher-level client crate (e.g. `ureq`) was weighed and set
aside: the crate's whole outbound posture is named caps and named refusals,
and a dependency that owns its own framing would make several of this
design's refusals somebody else's behavior.

**Mint basis: fresh implementation of the retired `024-C2` steps in
`apps/registry`** (chosen), re-pinned from the retired table
(A9). The alternatives: reviving `024-C2` as a law and relanding code in
`crates/discovery` (rejected — `025-C2` amended it out, the retirement gate
exists precisely to keep that code dead, and `027-C5` needs a *mint rule*, not
a revived identity law), or minting bare `steam-<appid>` ids for everything
(rejected — `027-C5` names the store-name slug as primary, and the
convergence evidence — five of the six curated ids are exactly what the rule
would mint from those names, the seed's `portal-two` being the exception,
which the rule derives as `portal-2` — is what makes mint-then-promote
cohere rather than collide; Rollout 6 carries what deleting a curated record
costs when its appid later re-mints under the other spelling).

**Ordering: serve-before-persist.** Rejected: `027-C4`'s unit text
("persisted … before it is served") is canonical, and a served-but-unsaved
identity is the one double-mint path `027-L2` cannot survive. §Issued
entries' Ordering is the sequence.

**Partial snapshot serving.** Rejected: a refresh that exhausts `027-B4`'s
page budget with `have_more_results` still true is a knowingly incomplete
worldlist, and `027-L3` would certify "never heard of it" against it. A
partial pull fails the pull (startup: named refusal; background: keep the
old snapshot). Nothing else a legal pull is handed makes the snapshot short
of the games-only set (§The store pull normalizes, never drops).

**Miss postures: one shape for epic and local.** Rejected both
unifications: `200 []` for an `epic/` key is a fabricated "never heard of it"
from a registry with no Epic knowledge, and `503` for a `local/` key breaks
`026` root mode's shipped posture for a token that is "never a candidate for
any resolver" (`027-C2`). §The hosted lookup policy takes epic → named
`503`, local → `200 []`.

## Chosen design

### Hosted mode: flags, named refusals, and the retiring half (traces `027-C1`)

`apps/registry` keeps its crate and binary; the mode enum collapses to one
inhabitant, `Mode::Hosted`, whose `as_str()` is `"hosted"` — the startup
line prints `registry hosted listening on http://{bound}`.

- Startup requires `--root --corpus <path>` exactly as `026-C1` spells it,
  and additionally a resolvable `<data>` directory (§Issued entries' files
  live there; its absence is a named startup refusal mirroring the retired
  caching-mode rule — a posture change from root mode, which needed none),
  whose write lock the run then takes (§Concurrency's `write.lock`: finding
  it held is the named refusal `registry data directory is locked`).
- `--upstream <url>` and `--refresh-interval <seconds>` join the
  named-refusal family with lines that quote the flag and name the retirement
  (`--upstream is not a flag this registry runs: caching mode retired with
  ADR-027`), distinct from the unknown-flag refusal; the flags stay in the
  parser *only* to be refused, so a stale operator invocation explains
  itself. `endpoint.rs` (the `--upstream` gate) retires with them.
- The retired document pair is a named startup refusal by presence: either
  `<data>/bindforge/registry/cache-v1.json` or `catalog-v1.json` on disk is
  refused by name before anything binds, so an upgraded host cleans its
  data directory rather than carrying inert state — except at the path
  `--corpus` names, which is how an operator keeps reading their old entries
  across the upgrade (§Rollout 6 says whether they are copied or dropped).
- New flag: `--no-steam` (serve with the steam resolver inactive; the
  snapshot is never loaded and never pulled). The Web API key arrives only
  through `BINDFORGE_STEAM_WEB_API_KEY` (`027-C3`); a missing or empty value
  without `--no-steam` is a named startup refusal — required regardless of
  snapshot freshness, because the background thread is part of the run.
- Retired code: `cache.rs`, `upstream.rs`, `endpoint.rs`; the `Mode::Caching`
  and `Mode::Root` arms; the `Gauge`/`Verdict` upstream gauge;
  `Provenance::{Root, Local, Cache, Upstream, Stale}`; the caching constants
  and document names they own. Retired suites per A15; `health_route.rs`,
  `data_layout.rs`, and `cli_end_to_end.rs` re-pin. Registry-to-registry
  federation (the `026-C6` upstream ingest, including gauging through
  `/v1/health`) retires with the mode: the hosted instance's only content
  sources are the corpus file, the issued document, and the snapshot.
- `registry query <slug>` survives as the one-shot hosted lookup policy
  (§The hosted lookup policy): same startup as `serve`, including
  load-or-pull, no refresh thread, one lookup with its provenance (`corpus`
  or `issued`, or the shipped `(no candidates)` line with provenance `none`
  for the policy's two `200 []` branches), non-zero exit on every named-503
  posture. A `query` can mint and persist like a served query, so it takes
  the same `<data>` write lock and refuses to run beside a live `serve`
  (§Concurrency): a stopped-instance tool, which mints into whatever
  `<data>` it is pointed at (Rollout 6).
- Health re-pins to the enumerated list `{status, mode: "hosted",
  corpus_entries, steam_snapshot}`, where `steam_snapshot` is the `027-C3`
  shape verbatim — `{"games": <count>, "fetched_at": <unix>}` — or the
  string `"none"` (the resolver-inactive or no-snapshot spelling, following
  the retired `upstream: "none"` precedent). `upstream`, `cached_slugs`,
  and `last_upstream_fetch` are gone — the
  caching facts retire with the mode, and `steam_snapshot.fetched_at` is
  their successor. The issued count is deliberately **not** public health —
  it lands in `stats` (`027-C6`).

### The hosted lookup policy (traces `027-C2`, `027-L3`, `027-L5`)

For one decoded, admissible key (the shipped `keys.rs` admissibility rule
unchanged), after the candidate budgets the serving face already enforces:

1. **Corpus.** The curated join, if non-empty, answers alone and the issued
   document is never consulted for that key (`027-L5`: curation outranks the
   mint; the issued record stays visible to curation only). Provenance
   `corpus`.
2. **Issued.** The issued-document join, if non-empty, answers. Provenance
   `issued`. No mint, no write — a key that already has a record never pays
   the mint path again, at the cap or anywhere.
3. **`local/…`** keys are never a candidate for any resolver (`027-C2`) and
   the registry holds no local knowledge: `200 []`, exactly `026` root
   mode's shipped posture; the CLI's own `local` authority is `025`'s.
4. **`epic/…`** keys hit the reserved, inactive slot: the named `503`
   `epic resolver inactive` — `[]` would be the "never heard of it" claim
   this registry cannot make about Epic (`027-C2`).
5. **`steam/<external-id>`** keys go to the snapshot:
   - Resolver active and snapshot within `027-B2`'s freshness
     (`now − fetched_at ≤ STEAM_SNAPSHOT_MAX_AGE_SECONDS`), and the external
     id is a decimal `appid` present in the snapshot: the **mint path**
     (§Issued entries) answers — or is refused with a named `503` when the
     identity cannot be persisted (`027-B3`).
   - Fresh snapshot, no such appid (including a non-decimal external id —
     the snapshot knows every **game** appid the games-only pull asks for,
     A3): `200 []`. Every page was examined, none skipped, and every record
     the pull was handed is in it, so this is the claim the snapshot is
     entitled to make (`027-L3`).
   - Snapshot past its age bound, or no snapshot, or resolver inactive
     (`--no-steam`): the named `503` `steam snapshot stale` /
     `no steam snapshot` / `steam resolver inactive`. A cold
     store-resolver never answers `[]`.

Nothing in any branch dials a store (`027-L1`): the request path reads the
loaded snapshot only. `026-L4`'s general cold rule survives narrowed to
these named `503`s; its amended text and ledger line are Rollout item 1's.

### The steam snapshot resolver (traces `027-C3`, `027-B1`, `027-B2`, `027-L1`)

The instance owns `<data>/bindforge/registry/steam-snapshot.json` — the
`027-C3` shape verbatim:
`{version: 1, fetched_at, apps: [{appid, name}]}` in
ascending `appid` order, owner-only, written through the fixed-tmp-and-rename
discipline (A2).

Loader refusals, by name, before anything binds (the contract-document
discipline): `version ≠ 1`; `fetched_at` not a decimal unix stamp within the
shipped plausibility pair — older than `FETCHED_AT_PAST_TOLERANCE_SECONDS`
or further future than `FETCHED_AT_FUTURE_TOLERANCE_SECONDS` (the shipped
`026-C7` tolerance pair in `constants.rs`, reused); `apps` out of order, or
a repeated or non-decimal `appid` in it; a `name` over
`MAX_NATIVE_NAME_OCTETS` or carrying control bytes (the `013-B4` bound
`027-C3` names — the pull normalizes one before writing it, §The store
pull); malformed shape; and an over-`027-B1` file refused on `fs::metadata`
*before* the read
(A2's metadata-gated load discipline, as the shipped corpus read already
applies it). An out-of-order file is refused, not resorted (`026-C7`'s
sort-rule precedent).

In memory the snapshot is a sorted vector plus an appid index behind an
`RwLock`; a join is one exact-`appid` probe returning at most the single
`{game_id: <minted or issued id>, name}` candidate. Startup order (before
bind): load corpus (must exist, unchanged from `026`), then — unless
`--no-steam` — load the snapshot if present, loadable, and fresh, else pull
in full; a failed pull (or a present-but-unloadable snapshot that the pull
then cannot replace) is a named startup refusal. A stale snapshot does not
rescue a dead key at startup: the operator's degraded option is `--no-steam`
(serving with steam misses answered `503`, corpus and issued still served).
`--no-steam` never reads the file at all.

`027-L1` is enforced structurally, not by convention: the `SteamSnapshot`
type is read by `state.rs`'s lookup and written only by the pull seam; there
is no path from a request thread to the store client.

### The store pull (traces `027-C3`, `027-B4`, `027-L1`, `027-L4`; resolves open Q5–Q8, Q10)

A new `steam.rs` owns the pull. Its transport is a `StoreTransport` seam: in
production, a hand-rolled HTTP/1.1 request over a blocking `rustls` client
connection to the compile-time-pinned `https://partner.steam-api.com/IStoreService/GetAppList/v1/`
— no operator-configurable store URL, hostname verified against the pinned
authority, roots from `webpki-roots`; in tests, the same request/response
codec run against a loopback fake (plain TCP), so no test resolves or dials
a non-loopback name (the shipped gate discipline, A15's neighborhood), with
one TLS listener in the same family for the transport-assurance case
(§Test strategy #6).

Request rules, per pull (full or delta): `key` from
`BINDFORGE_STEAM_WEB_API_KEY`, `max_results=50000` **pinned explicitly on
every page** (A3's 10k default would trip the 12-page budget against a
~200k catalog, which is the research's Q5 finding — `027-B4`'s 12 pages are
measured against the 50k maximum: 600k slots, ≥3× A5's upper band),
`last_appid` continuation, no `include_*` flag ever sent — the pull asks for
the endpoint's default games-only set (A3), which is exactly the class the
policy's `200 []` certifies against — and for deltas `if_modified_since` =
the loaded snapshot's `fetched_at`. Response rules, all named refusals that
fail the *page*: non-`200`; `Transfer-Encoding`; a `3xx` (the shipped
upstream-client precedent); a body over `MAX_STEAM_PAGE_OCTETS`; a body that
is not the envelope A17 describes or carries no `apps` list; more than
`STEAM_REFRESH_MAX_PAGES` (12) pages while `have_more_results` is true; a
record whose `appid` is not a decimal integer. Nothing is *dropped*: a
record whose `name` breaks the `013-B4` bound or carries control bytes is
normalized at the pull seam instead — store display names are
attacker-chosen content, and failing the pull over one is remote, permanent
denial of the supply. The normalization is one std filter plus the shipped
clamp, in that order: drop the `char::is_control()` characters, then
`discovery::game::friendly_name` through that existing `discovery`
dependency (A7: the shipped rule is whitespace and clamping only, which is
why the filter is ours). A name that normalizes to empty stays empty: its
label renders blank and its identity is mint step 3's `steam-<appid>`
(§Issued entries). The record and its appid stay in the snapshot, and its
identity is paid once like any other key's (policy step 2). The publish line
counts the names it clamped (§Risks, §Deferred). Per page, up to
`STEAM_PAGE_ATTEMPTS` (3) attempts separated by
`STEAM_PAGE_RETRY_SECONDS`; a page failing all attempts fails the pull, and
a pull of either kind that has not published inside
`STEAM_PULL_DEADLINE_SECONDS` fails by name as `steam pull deadline`,
distinct from the over-`STEAM_REFRESH_MAX_PAGES` refusal and sized in
§Budgets to cover this grid with its retries, so a slow store costs a late
snapshot, not a guaranteed failure. A pull never publishes
partial results: a failed full pull is a startup refusal (or leaves the
previous snapshot untouched); a failed delta keeps the old snapshot serving
until `027-B2`'s age bound converts its misses to `503` — that is the
tolerance the ADR bought.

A full pull replaces the snapshot; a delta publishes via one linear two-way
merge of the loaded vector and the pulled changes plus one index build
(never one insert per changed appid), re-persisting once. Deltas never see
removals (A3's documented "changed or added" semantics) — removals are
handled by curation's delete, and the operator forces a full refresh by
deleting the snapshot file (§Rollout's runbook). `fetched_at` is the wall
clock read *before the pull's first page request*, never the publish-time
stamp: a stamp later than the data it certifies leaves the next delta's
`if_modified_since` above apps the pull never saw, and they stay unseen
until the operator deletes the snapshot; an overlap only re-fetches an item
the appid merge absorbs.

The refresh thread lives in `main.rs`'s `serve_command` only (never
`query`, never under `--no-steam`): sleep `STEAM_SNAPSHOT_REFRESH_SECONDS`
(86,400), delta-pull, on success tmp-and-rename, then publish the new model
under the snapshot `RwLock`; on failure keep the old snapshot and print one
escaped warning line, retrying after `STEAM_RETRY_SECONDS` — the only
consumer of that constant — until success.
The thread never takes the mutation lock (§Concurrency) and never touches
the corpus or issued documents — it writes one file and swaps one model.

Key custody: environment only, as `027-C3` pins (`BINDFORGE_STEAM_WEB_API_KEY`,
the `BINDFORGE_REGISTRY_ADMIN_TOKEN` precedent); no diagnostics carry the
request URI — every pull-failure line names the condition and status, never
the URL or the key (A14's CWE-319/526 mitigation split: transport here,
scrubbing in our own outputs; migrating both secrets to `0600` files is
§Deferred). `027-L4`: the only bytes this registry sends a store are the
fixed request lines above — no queried key, user or otherwise, ever appears
in a store request, which Test strategy #6 pins mechanically.

### Issued entries, the mint, and identity bookkeeping (traces `027-C4`, `027-C5`, `027-L2`, `027-L5`, `027-B3`; resolves open Q1–Q4, Q11)

**The document.** `<data>/bindforge/registry/issued-v1.json` is a `026-C3`
catalog document — same shape, same loader refusals (via the same
`corpus.rs::Catalog` type), same owner-only tmp-and-rename discipline; an
absent file is an empty document (unlike the corpus, which must exist). It
carries its own `revision`, bumped by every mint and every issued delete.
Its octet budget is private and separate: `MAX_ISSUED_DOCUMENT_OCTETS =
67_108_864`, sized for `027-B3`'s 200,000 records at typical ~150-octet size
(~30 MiB, A5's band) with headroom, enforced at load (metadata-gated) and
against the exact buffer a write renames.

**The mint rule (`027-C5`, restated in full).** `mint.rs::mint_id(display_name,
appid, held)` walks the retired `024-C2` steps (A9), re-pinned from the
feature-gated table at `crates/discovery/tests/slug_tests.rs`, as *new*
`apps/registry` product code (legal under the retirement gate, which scans
only the two discovery trees — A9):

1. ASCII case-fold; the finite Latin-1/Latin-Extended-A single-base-letter
   fold table; every remaining code point outside `[a-z0-9]` becomes `-`;
   hyphen runs collapse; leading/trailing `-` trimmed. No NFKD dependency:
   the fold is the same finite table the retired suite pinned.
2. A leading digit gains the `g-` prefix (the shipped grammar needs a
   lowercase first byte, A10) — the suite's `g-7-days-to-die` spelling.
3. Empty result (CJK/Cyrillic/emoji-only titles, which the research says
   exist in volume): the mint id is `steam-<appid>`, appid-unique and
   grammar-valid — no fallback chain.
4. Collision (`027-C5`'s spelling): if the derived id is already held — in
   the corpus **or** the issued document, or retired (below) — by a
   different record, the id becomes `<slug>-steam-<appid>`, which earns the
   same held ∪ retired check before it is served: corpus ids are
   operator-chosen grammar strings, so one can already spell that form.
5. Bound (`027-C5`'s "bounded to the slug budget", `024-C2`'s "cap at the
   shared bound"). The fold never exceeds the name's octets, but steps 2
   and 4 add them — 2 for `g-`, 7 plus the appid's digits for the suffix —
   onto a name the loader admits at exactly `MAX_SLUG_OCTETS`
   (`MAX_NATIVE_NAME_OCTETS = 128`, A10), so no mint output is in-budget by
   construction and the retired "length is never truncated" posture is not
   re-pinned here. A mint whose chosen id exceeds `MAX_SLUG_OCTETS` mints
   `steam-<appid>`, spelled from the queried key's external id: the shipped
   admissibility rule caps the *joined* key at 128
   (`MAX_DISCOVERY_SLUG_OCTETS`, `keys.rs`), so that id is at most
   6 + 122 = 128 octets — in budget by that cited cap, appid-unique, and
   held-set-checked like any other id; an id still over it is the named
   `503` `mint identity over bound`.
6. Any spelling the rule lands on that is held or retired by a different
   record refuses the mint with the named `503` `mint identity held`, and
   the key is curation's to solve. `024-L2`'s ordinal spelling (`-2`, `-3`)
   is **not** revived: an ordinal depends on which records exist when,
   where the appid suffix is record-stable.

**Ordering (resolving the ADR's internal contradiction).** The canonical
sequence is: *decide* the answer from the snapshot (read-only), then, under
the mutation lock (§Concurrency's scope, which is also what stops two
concurrent mints of one key from both passing the dedup check), check
dedup/cap, **persist the whole issued record**, swap the in-memory model,
and only then write the response out. This is `027-C4`'s
persist-before-serve, which is what makes `027-L2` hold: an identity served
but not persisted could mint twice, while one persisted but not served is a
silent record, the lesser breach and the one `027-B3` explicitly accepts. A
failed persist is a named `503` with model and file untouched.

**The cap (`027-B3`).** At `MAX_ISSUED_ENTRIES = 200_000` records, a *new*
mint is the named `503` `mint cap reached` — literally as the ADR spells it,
even when the snapshot knows the key: an identity that cannot be persisted
is not an identity. Keys already held answer from the issued document
forever (policy step 2 precedes the cap check), so the cap degrades only
*novel* keys. Promotion (corpus upsert, then issued delete) frees a slot,
and the runbook carries the arithmetic (§Risks).

**Retired ids (`027-L2`).** "Deleting a record never frees its id" needs
memory the `026-C3` shape cannot carry, so the instance owns one more small
document: `<data>/bindforge/registry/retired-ids-v1.json`,
`{version: 1, ids: [<game_id>…]}` in byte order, absent = empty, same
discipline, `MAX_RETIRED_IDS_OCTETS = 33_554_432` — ≈250k ids at the
128-octet id bound, >1M at typical ids — gated at load (metadata-gated) and
against the buffer a write renames. **Every** delete — issued or corpus — is
two persists under one mutation-lock hold, retired first. A run that dies
between them leaves `games ∩ retired ≠ ∅`: the legal, recoverable state —
the record keeps answering and the retried delete completes the removal —
safe because the append is a set insert, so an already-present id is a
no-op, never the duplicate that would break the file's byte order. A delete
whose write breaches that cap, on the rename-buffer measure above, is the
named refusal
`retired id list full` and mutates nothing; failing open there re-opens a
retired id, which is `027-L2` broken
by arithmetic. In memory it is an ordered insert with membership by binary
search, and a delete costs two bounded document rewrites, admin-only. The
mint's held-set is `games ∪ retired` across both catalog documents; an id in
the retired set is never minted again. `stats` does not report the retired
count, so the runbook reads the file's size (§Deferred).

### Concurrency: three writers, one mutation lock (traces `027-C4`; resolves the lock finding)

`027-C4` names one lock ("the same write lock that serializes admin
mutations"). Concretely: a single `mutations: Mutex<()>` serializes **corpus
admin mutations, steam mints, and issued deletes**, and that mutex — not the
documents' `RwLock` — is what keeps persist order from diverging from
mutation order. Inside it a writer probes the loaded models, computes and
serializes the new document straight into the write buffer (a mint inserts
the new key's posting rather than rebuilding the index), pays
tmp-and-rename, and takes its document's `RwLock` write half only for the
model swap. Readers (serving lookups) take read halves only, so no client
ever waits on an fsync. Lock order is
`mutations → corpus read → retired read/write → issued read/write` and
never the reverse: mints read the corpus for the `027-L5` suppression check
and both catalog documents plus the retired list for the held set. A delete
follows the same order — retired document first — and its two model swaps
are sequential (retired `RwLock` write half, swap, release, then the catalog
document's, never held together): a corpus delete's write half is the one
departure from the order as listed. The refresh thread — the snapshot's only
writer — takes only the snapshot lock, so there is no cycle.
Across processes the fixed-tmp `create_new` discipline (A2) is a per-write
guard, not a startup refusal, and cannot refuse a second writer; so `serve`
and `query` take `<data>/bindforge/registry/write.lock` under `create_new`
at startup and remove it on exit, and finding it present is the named
startup refusal `registry data directory is locked`: without it, two
persist-before-serve writers rename models read at their own startups over
each other's `027-L2` records. A crashed run leaves the file; the runbook
names deleting it.

### The curation face (traces `027-C6`)

`026-C4`/`026-C5` stand unchanged for the corpus. Additions, behind the
identical enablement/auth rule (loopback-or-bearer, same `401`/`404`
posture):

- `DELETE /v1/admin/issued/<game_id>`: percent-decode the segment, validate
  the `game_id` grammar (A10), refuse an absent record with the same
  `Absent` refusal shape the corpus delete answers, then move the record's
  id into the retired list and remove the record (bumping the issued
  `revision`) in two persists under one mutation-lock hold, retired first
  (§Issued entries). One issued record per call;
  no bulk retire (`026-L3` posture).
- `GET /v1/admin/stats` answers `{games, source_keys, revision, issued}` —
  the first three from the corpus as shipped, `issued` the issued-document
  record count. Counts only; no key or name lists (`026-L3`).
- **Promotion** is an operator sequence, not an operation: upsert the whole
  curated record (typically the issued record with `source_keys` extended),
  then `DELETE` the issued one. Until the delete lands, the corpus answers
  the key alone (`027-L5`), so the window is a no-op for clients; after it,
  the retired-id entry keeps the issued copy from ever returning (§Issued
  entries, retired ids).

### Budgets (new `constants.rs` family)

Private to `apps/registry` like the shipped family (the `026` precedent):
`MAX_STEAM_SNAPSHOT_OCTETS = 33_554_432` (`027-B1`, load ceiling and
publish-time check); `STEAM_SNAPSHOT_REFRESH_SECONDS = 86_400` and
`STEAM_SNAPSHOT_MAX_AGE_SECONDS = 604_800` (`027-B2`);
`MAX_ISSUED_ENTRIES = 200_000` (`027-B3`); `STEAM_REFRESH_MAX_PAGES = 12`
(`027-B4`, against the pinned 50k page); `STEAM_PAGE_MAX_RESULTS = 50_000`;
`MAX_STEAM_PAGE_OCTETS = 12_582_912` — the response *read* budget for one
page: 12 MiB, above the 50k-records-at-~240-octets estimate (A17's four
fields, `name` at `MAX_NATIVE_NAME_OCTETS`) the pinned `max_results` can
return, not the ~150-octet mid-estimate; not a worst case, since over-bound
store names make a page's true size unbounded and the over-cap page is the
residual §Risks carries; `STEAM_CONNECT_TIMEOUT = 2 s` /
`STEAM_READ_TIMEOUT = 30 s`; `STEAM_PAGE_ATTEMPTS = 3` separated by
`STEAM_PAGE_RETRY_SECONDS = 30`, inside `STEAM_PULL_DEADLINE_SECONDS =
1_920` ≥ the grid that family authorizes (`12 × (3 × 32 + 2 × 30) =
1,872`), so a stalled pull is cut off without `027-B4` being overridden
first; `STEAM_RETRY_SECONDS = 3_600` — the refresh
thread's between-pull gap only, no startup path sleeps it;
`MAX_ISSUED_DOCUMENT_OCTETS = 67_108_864`; `MAX_RETIRED_IDS_OCTETS =
33_554_432` (§Issued entries gives the id count it implies). The ADR states
no page budgets; these are the elaboration the research's Q6 demanded, kept
in the one file. Store quota: the refresh thread's worst day is
24 × 12 × 3 = 864 calls, ~115× inside A6's 100,000/day, and a startup pull
spends ≤36 of them with none of that spacing, bounded only by process
starts — the runbook's restart policy (§Rollout 6); no separate quota
machinery. The four document caps total 144 MiB of JSON (16 corpus + 64
issued + 32 retired + 32 snapshot) as the boot load ceiling; on top of it a
mint holds one model plus one serialized buffer, and a delta merge holds the old
snapshot, the merged model, one serialize buffer and the current page body.
Rollout 4 records the measured issued-document rewrite at 100k and 200k
records beside that figure, so §Deferred's batched writes defer against a
number.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The std-only binary grows a TLS stack (`rustls`/`ring`/`webpki-roots`) — the largest dependency this crate has ever taken, and `ring` compiles C/assembly for the target. | Certain (accepted) | Medium | Confined to `steam.rs`'s pull seam; the serving face, corpus, issued, and admin paths keep zero new deps; named version *floors* (not "pinned versions") plus a RUSTSEC advisory screen in the gate; Rollout 3 lands the CI-leg split and the `README.md` C-toolchain prerequisite (A16). |
| `027-B3`'s 200k cap sits inside A5's catalog band, so "refuse new mints with `503`" is an ordinary operating state, not an abuse edge. | Medium | High | Dedup makes held keys answer forever (cap degrades only novel keys); promotion frees slots; `stats.issued` makes the approach visible; raising the cap or mint-on-confirmation is a named budget amendment (§Deferred), not a surprise. |
| Write amplification: every first-time key rewrites a multi-megabyte issued document under the mutation lock (CWE-770 class), ≤64 threads deep (A8). | High | Medium | One mint per key *ever* (issued hits short-circuit at policy step 2); the cap bounds the document; §Concurrency keeps the file IO off the document's write half so readers never queue behind an fsync, and Rollout 4 measures the rewrite at 100k/200k records before the cap is called ordinary; batched writes deferred (§Deferred). |
| A rename-impersonation (A11) claims a plain slug by being first queried, serving an honest-but-wrong identity. | Medium | Medium | Curation outranks the mint absolutely (`027-L5`): one corpus upsert overwrites the answer for every key it names; `DELETE issued` + retired ids keep the impostor from returning. |
| Minted names freeze at mint time while ~2% of titles rename (A11); nothing surfaces the drift. | High | Low | Accepted: ids are load-bearing, names are display; correction is a corpus edit; surfacing tooling is explicitly §Deferred. |
| Delta pulls never see removals (A3), so issued records of delisted apps linger and answer. | Certain (documented endpoint behavior) | Low | Operator `DELETE` is the removal path; a forced full refresh (delete the snapshot file) cleans the snapshot side; issued orphan cleanup is §Deferred. |
| `013-B4`'s 128-octet name bound has no margin (A10) and store display names are attacker-chosen content (A11 records renaming): a 43-character CJK or 32-character emoji title reaches it, and a pull that refused or dropped such a record would be denied by remote content. | Medium | Low | The pull seam normalizes the `name` to the `013-B4`-legal form and keeps the record (§The store pull owns the rule), so the supply survives and the appid's misses stay certifiable; the cost is a shorter label, never a different identity, because the identity is paid once per key (policy step 2) and equal-prefix clamps resolve at mint steps 4–5. Names over the bound also make a page's true size unbounded, so an over-`MAX_STEAM_PAGE_OCTETS` page is the named residual: a page refusal that keeps the previous snapshot until `027-B2`'s age bound. Re-sizing the bound is an amendment (§Deferred); the pull's clamp count is the signal. |
| Valve can terminate a personally-licensed, non-transferable key without notice (A6), ending the snapshot supply. | Medium | High | `027-L3`'s named `503` is the honest degraded posture while corpus+issued keep serving — and, because a bound CLI fails the whole scan closed on any non-`200` (`025-C4`, §Compatibility matrix), the age of `health.steam_snapshot` is the operator's availability dial; the runbook carries the ToS obligations by name (§Rollout item 6) and records that the license forecloses key rotation. |
| A backwards clock step makes a snapshot look fresh (A12). | Low | Low | The load-time plausibility pair (the shipped `026-C7` tolerances in `constants.rs`) bounds a future-dated stamp to 86,400 s of false freshness against a 604,800 s age bound. |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux | Yes | The hosting target and CI runner: the host-target cargo trio builds and tests `rustls`/`ring`, and needs a C toolchain for `ring`'s build script (A16, `README.md` prerequisite per Rollout 3). `<data>` = `$XDG_DATA_HOME` else `$HOME/.local/share` (A2); owner-only `0700`/`0600` apply. |
| macOS | Dev only | Same Unix `<data>` branch and modes as Linux, and it is what a developer on a Mac runs; no gate builds or tests a macOS target and none is added, so nothing here promises a hosted macOS deployment. |
| Windows | Yes (compile-only) | `<data>` = `%APPDATA%`, relative = unset (A2); owner-only modes stay the documented no-op; owner paths `%APPDATA%\bindforge\registry\{steam-snapshot,issued-v1,retired-ids-v1}.json`. The shipped leg compiles the workspace for `x86_64-pc-windows-msvc` from the Linux runner (A16); Rollout 3 keeps `apps/registry` off that leg for `ring`'s target build script (§Options), so the crate's Windows coverage is host-compile plus the `DataLayout` seam. |
| Other / exotic Unix (BSD et al.) | No | Not a v1 target; the Unix branch and `std::net` may carry it; nothing exercises or promises it. |
| Bound CLI clients (any platform) | Yes | No change required or permitted: `025-C4`/`026-C2`'s candidate face, response shape, and budgets serve the CLI verbatim. The new `503`s render as the shipped status-line-only refusal *and fail the whole scan closed* (`025-C4`'s fail-closed rule), so a snapshot past `027-B2`'s age bound turns a user's `discovery list` into a refused run: snapshot age is user-visible availability, and the runbook reads it that way (§Risks). |

## Test strategy

Unit (in `apps/registry` unless noted; no unit resolves or dials a
non-loopback name):

1. **Mint table (`027-C5`):** the retired suite's survey vectors derive
   byte-identically (re-pinned from `crates/discovery/tests/slug_tests.rs`),
   including the `g-` leading-digit spellings and the fold table; a
   non-Latin-only name mints `steam-<appid>`; collision yields
   `<slug>-steam-<appid>` against a holder in either document; every output
   satisfies `is_slug_name` ∧ `≤ MAX_SLUG_OCTETS` — including the over-bound
   cases the arithmetic allows: a 128-octet ASCII name, a 128-octet
   digit-leading name, and a 128-octet *colliding* name all mint
   `steam-<appid>`; a suffixed id already held takes `steam-<appid>`, a
   held `steam-<appid>` refuses `mint identity held`, and an id still over
   `MAX_SLUG_OCTETS` after substitution refuses `mint identity over bound`.
2. **Snapshot loader (`027-C3`):** valid fixture loads; named refusals for
   `version ≠ 1`, out-of-order, repeated or non-decimal `appid` in `apps`,
   over-`013-B4` or control-byte `name`, `fetched_at` outside the
   plausibility pair, over-`027-B1` (metadata-gated, file never read), and
   malformed shape.
3. **Miss-posture decision table (`027-L3`, `027-C2`):** fresh snapshot +
   appid hit → mint path; fresh + appid miss (and a non-decimal external id)
   → `200 []`; stale snapshot → `503 steam snapshot stale`; absent → `503
   no steam snapshot`; `--no-steam` → `503 steam resolver inactive`; `epic/…`
   → `503 epic resolver inactive`; `local/…` → `200 []`; a curated steam key
   answers `corpus` with the issued copy invisible (`027-L5`).
4. **Mint write path (`027-C4`, `027-B3`):** persist-before-serve (an
   injected persist failure answers the named `503` with model, `revision`,
   and file untouched, and nothing served); dedup answers the existing record
   with zero writes; at `MAX_ISSUED_ENTRIES` a novel key is `503 mint cap
   reached` while a held key still answers; the octet cap refuses the same
   way; `revision` bumps per mint; two concurrent mints of one key leave one
   record and one id.
5. **Retired ids (`027-L2`):** issued and corpus deletes append to
   `retired-ids-v1.json` as the first of the hold's two persists; a derived
   id in the retired set takes the suffix spelling; an id retired under both
   spellings refuses `mint identity held`; a write that breaches
   `MAX_RETIRED_IDS_OCTETS` against the rename buffer answers `retired id
   list full` and mutates nothing, while a no-op re-delete — even against an
   at-cap list — still completes the removal; a planted `games ∩ retired`
   overlap keeps the record answering; absent file = empty set.
6. **Pull codec (`027-C3`, `027-B4`, `027-L4`):** against the loopback fake
   transport — requests carry `max_results=50000`, `last_appid`
   continuation, no `include_*` flag, `if_modified_since` on deltas, and nothing
   derived from any queried key (the `027-L4` assertion); `3xx`,
   `Transfer-Encoding`, over-`MAX_STEAM_PAGE_OCTETS`, a body that is not the
   A17 envelope or carries no `apps` list; more than
   `STEAM_REFRESH_MAX_PAGES` (12) pages while A17's `have_more_results` is
   true; a non-decimal `appid` each fail by name; a bound-breaking or
   control-byte `name` is control-stripped then clamped to the
   `013-B4`-legal form, an all-whitespace or all-control title stores empty,
   and its record stays in the snapshot; retries stop at
   `STEAM_PAGE_ATTEMPTS`, the gap between them is the page backoff and never
   `STEAM_RETRY_SECONDS`, and the deadline turns a stalled pull into
   `steam pull deadline`; a partial run never publishes; a delta merges by
   one linear two-way merge preserving order; a full pull replaces; **no
   failure line ever contains the key or the request URI**.
   **Transport assurance:** the *production* `StoreTransport`, reached
   through a test constructor taking the connect address and the name to
   verify separately (the compile-time-pinned authority stays the production
   value), driven against a loopback TLS listener presenting a self-signed
   cert *for the name being verified* and one for another name; it refuses
   both handshakes, so the first case fails on the chain, not the hostname.
   Without it, a verifier relaxed to accept the fake passes the whole suite
   with the key on an unauthenticated link; both cases dial `127.0.0.1`, so
   #17's no-outbound rule holds.
7. **Issued document:** `026-C3` refusals apply verbatim; absent file = empty
   document; the `MAX_ISSUED_DOCUMENT_OCTETS` load refusal is metadata-gated.
8. **Flags and startup (`027-C1`):** `--upstream`/`--refresh-interval`
   produce the retirement refusal (quoting the flag, naming ADR-027) before
   any bind; a leftover `cache-v1.json`/`catalog-v1.json` in the registry
   directory is a named refusal except at the path `--corpus` names; a second
   process against a `<data>` whose `write.lock` exists refuses before bind
   and leaves the file alone, and a run removes it on exit; missing key
   without `--no-steam` refuses; missing `<data>` refuses; `--no-steam`
   starts with health `steam_snapshot: "none"`.
9. **Health and stats:** health is exactly `{status, mode: "hosted",
   corpus_entries, steam_snapshot}` with the `{games, fetched_at}` and
   `"none"` spellings and none of the three retired facts; `stats` is
   `{games, source_keys, revision, issued}`.
10. **Admin issued route (`027-C6`):** auth posture identical to the corpus
    routes; absent id → the `Absent` refusal; success retires the id, bumps
    the issued `revision`, persists atomically; the deleted key answers from
    the corpus when curated, else re-mints under a suffix id that is *not*
    the retired one.
11. **Refresh scheduler (`027-B2`):** with injected clock and transport, the
    pull is due at 86,400 s, a failed pull retries at 3,600 s and keeps the
    old snapshot published; the thread's seam is a pure decision function —
    no unit sleeps.

Integration (subprocess; `XDG_DATA_HOME`/`APPDATA` redirection is the only
document-relocation seam, as shipped):

12. **Hosted end-to-end:** `serve --root --corpus <seed>` with a fresh
    planted snapshot (covering appids beyond the seed's six) and a
    configured, never-used key; a snapshot-known `steam/<appid>`
    mints and answers; the same key answers again with zero issued-document
    rewrites; a curated key answers with its corpus id; an unknown steam
    appid answers `200 []`; `epic/…` answers the named `503`; `local/x`
    answers `200 []`; response shape stays byte-exact to the shipped
    candidate contract (A8); a second `serve` against the same redirected
     `<data>` refuses at startup, and the first removes `write.lock` on exit
     (#8).
13. **Stale-snapshot posture:** a planted snapshot whose `fetched_at` is
    older than `027-B2`'s age bound turns steam misses into the named `503`
    while corpus and issued answers keep serving, and with `--no-steam` the
    same split holds with the inactive reason.
14. **Curation flow:** over the loopback socket, a corpus upsert shadowing an
    issued key, then `DELETE /v1/admin/issued/<game_id>`, then `stats.issued`
    decrement, then a re-query still answered from the corpus — and a corpus
    delete of that key makes the next query mint a suffix id, never the
    retired one (#5).
15. **Query one-shot:** `registry query` prints `corpus`/`issued`
    provenance, prints the shipped `(no candidates)` line with provenance
    `none` for either `200 []` branch, exits non-zero on every named-503
    posture, lands its mint in the redirected issued document, and refuses to
    start against a `<data>` whose write lock a live `serve` holds (#8).

Platform:

16. **Linux** is exercised directly by the cargo trio, `rustls`/`ring`
    included, on the host target; **Windows** is covered by the shipped
    `cargo check --target x86_64-pc-windows-msvc` leg (A16) — whose scope
    Rollout 3 narrows so `ring`'s target build script is never asked of a
    runner without a target C toolchain, with `apps/registry` compile-checked
    on the host instead — plus the `DataLayout`/env-injection seams for the
    `%APPDATA%` branch and the no-op owner-only modes, which assert under
    `cfg(unix)` where they are not no-ops. **macOS** has no leg at all, which
    is exactly why the matrix records it as dev-only rather than supported.
17. The whole suite keeps the shipped no-outbound rule: the store fake is an
    in-test loopback listener; nothing resolves `partner.steam-api.com` from
    a test (#6's TLS cases dial `127.0.0.1`); `npm ci && npm run check` in
    `tools/orchestration` is unaffected (no orchestration surface changes).
    The cargo job does change, per Rollout 3: the Windows leg's scope, and the
    advisory screen.

## Rollout

1. Ledger annotations (ADR Rollout 1, `021-L4`): `026-C1` and `026-C2` gain
   `amended-by` lines naming `027-C1`/`027-C3`; `026-C6`, `026-C7`,
   `026-B1`–`026-B3` gain retirement lines; `026-L4` gains its `amended-by
   027-L3`; `026-L5` records its caching half retiring with the mode while
   the `025-L4` local-authority posture stands; `026-R2` records this run
   filling the intake half with the profile half forwarded to `027-R2`
   (ADR-028); `024-C2` gains a cross-reference line that `027-C5` restates
   its steps as the registry's mint rule without reviving it as a law.
   `027-C3`'s snapshot and `steam_snapshot` health shapes ship verbatim.
2. The hosted mode lands: `Mode::Hosted`, the named refusals
   (flags, documents, key, `<data>`), the health re-pin, the retiring
   modules/constants/suites of §Hosted mode, and the seed corpus serving
   unchanged as `027-S1` — all six shipped records, including the one
   `epic/` key answered by curation alone (A13).
3. The steam resolver lands: `steam.rs` (document, loader, the
   `char::is_control` strip plus `discovery::game::friendly_name` clamp, the
   pull seam with the page backoff and
   the `steam pull deadline` refusal, refresh scheduler), the `rustls` +
   `webpki-roots` dependencies *at the floors
   §Options names*, the RUSTSEC advisory screen added to the gate beside them,
   the `check.yml` Windows-leg scope split that keeps `apps/registry`
   compile-checked on the host for `ring`'s build script, and the `README.md`
   prerequisite line that names a C toolchain (A16), then the startup
   load-or-pull, the `--no-steam` posture, the §Budgets family, and the
   `027-L1`/`027-L4` structural seams — Test strategy #2, #3 (fresh
   half), #6, #8, #11.
4. Issuance lands: `issued-v1.json`, `retired-ids-v1.json`, `mint.rs`, the
   persist-before-serve path under §Concurrency's lock scope, the `<data>`
   `write.lock` startup discipline, the cap and octet postures, the issued-document rewrite measured
   at 100k and 200k records, recorded in §Budgets,
   `027-L5` suppression, and the amended miss posture — Test
   strategy #1, #3 (stale half), #4, #5, #7, #12–#15.
5. The curation face lands: the issued delete route, the retired-list append,
   and `stats.issued` — Test strategy #9, #10, #14.
6. The runbook lands at `docs/registry-hosting.md`: the hosting posture and
   ephemeral-port contract; the key seam (environment only, CWE-526 stance,
   no-URL-in-logs rule); custody of the corpus, issued, retired-ids, and
   snapshot files — forcing a full refresh by deleting the snapshot, deleting
   a crashed run's leftover `write.lock` by hand, reading the retired list's
   size against its cap, and clearing the leftover `026` cache-document pair
   with **no migration**: an operator who wants
   those local entries copies them out first or upgrades with `--corpus`
   pointed at them (§Hosted mode); the stopped-instance invocation of
   `registry query` and the restart policy that bounds how many startup
   pulls a day can spend; `health.steam_snapshot`'s age reads as
   user-visible availability, because a bound CLI fails a whole scan closed
   on the `503`s it explains (§Compatibility matrix);
   the cap arithmetic (`stats.issued` against 200k, promotion frees slots,
   and what deleting a curated record costs when its appid later re-mints
   under the other spelling — §Options); the Steam ToS obligations by name —
   named-Application registration, key confidentiality and
   non-transferability (rotation foreclosed), the "as is" §7/§8-equivalent
   disclaimers, the no-endorsement presentation rule, and the privacy-policy
   and data-residency restriction the same Terms lead with; and the edge-TLS
   stance `027-R3` leaves to the operator.
7. The operator deploys the hosted instance against the runbook; the
   deployment is an operator act outside the repository's gates (`027-R3`).

## Deferred / out of scope

- The epic resolver: blocked on `025-R2` and the absence of any public Epic
  listing API — `027-R1`, owner `unassigned`; the reserved slot stays inert
  exactly as designed here.
- Profile upload/download/sharing and auth beyond the bearer token —
  `027-R2`, ADR-028 (reserved).
- Client- and registry-facing TLS / transport assurance — `027-R3`
  (carrying `026-R1`); this design's `rustls` hop does not extend it to
  clients.
- The CLI self-resolution amendment for `local/<dir>` keys — the separate
  small ADR the source document's Consequences leaves outside its ledger;
  nothing here preempts it.
- Moving `BINDFORGE_STEAM_WEB_API_KEY` and the admin token from environment
  to `0600` file custody (CWE-526 mitigation, A14) — no follow-up yet; both
  secrets should move together, once.
- Tooling to surface stale-name or dead-app issued records, or a retired-id
  count (`stats` grows counts only per `027-C6`) — no follow-up yet.
- Raising `027-B3`, mint-on-confirmation, or batched issued writes if the
  cap becomes steady state at real traffic — a budget amendment to this ADR,
  not a build-time improvisation.
- Re-sizing or widening `013-B4`/`027-C3`'s name bound if a store name
  breaches it in earnest — an amendment to those budgets; until then the
  breaching name is clamped at the pull seam and only its label shortens,
  and the pull's clamp count is the operator's signal (§The store pull).
- macOS as a hosting target, and a macOS CI leg to make the matrix honest
  about it — the platform stays dev-only (§Compatibility matrix, A16) until
  someone runs a hosted registry there.
- Corpus hot-reload for the promote-then-delete workflow (restart-only
  stands) — no follow-up yet.
- Formal ledger retirement of `024-L2`'s ordinal collision spelling (dead
  since `025-C2`; not referenced by live code) — `021`-housekeeping, no
  follow-up yet.
- Lifting the response-framing and persist blocks `steam.rs` copies from
  the retiring `upstream.rs` (and its shipped `discovery-cli`/`matches.rs`
  originals) into a shared crate — an
  implementation refinement, same posture as `026`'s deferral.

## Revision log

- r0→r2: mint basis `024-C2` retired and its code deleted (open Q1/Q2) — fresh `apps/registry` implementation of the steps, re-pinned from the retired table; `g-` prefix kept; install-dir fallback replaced by `steam-<appid>`; `024-L2` ordinals not revived (§Issued entries; Rollout 1 cross-reference). r2: the length claim now rests on a stated bound instead of an invented one — `steam-<appid>` is spelled from the queried key's external id, which `keys.rs` caps with the joined key at 128 octets — and an id still over `MAX_SLUG_OCTETS` after that substitution is the named `503` `mint identity over bound` (§Issued entries).
- r0→r1: keyed HTTPS pull vs a TLS-free crate (open Q6/Q7) — `rustls`+`webpki-roots` confined to the `steam.rs` seam; plaintext pull rejected (CWE-319); out-of-process importer rejected (would amend `027-C3`). r1: re-pinned as version *floors* plus a RUSTSEC screen, and as a C/asm build-script dependency rather than a "pure-Rust" stack — Rollout 3 splits the Windows compile leg and `README.md` gains the C-toolchain prerequisite (§Options, A16).
- r1→r2: store-hop authentication (§Test strategy #6) — the property no test had: the *production* rustls config refuses a loopback listener with a self-signed cert and one for another name; r2 names the seam (a `StoreTransport` constructor taking connect address and verification name separately, the authority still compile-time-pinned in production) and gives the self-signed cert the verified name, so the case fails on the chain and not the hostname.
- r0: `027-C4` vs Rollout-4 ordering contradiction — resolved as decide→persist→serve, `027-C4`'s text canonical; a failed persist is the named `503`, the record is never served.
- r0→r2: three writers, one named lock (research's concurrency finding) — one mutation mutex over corpus admin writes, mints, and issued deletes (§Concurrency); the mutex, not the document `RwLock`, carries persist order, so file IO moves off the write half and no reader waits on an fsync, and the cross-process refusal is a `<data>` `write.lock` taken at startup, since the fixed-tmp discipline is a per-write guard that never refuses a second process. r2: the order now names the write direction (`corpus read → retired read/write → issued read/write`), a delete's two model swaps are sequential and never held together, and the `write.lock` consequence is owned: `query` is a stopped-instance tool (§Hosted mode, Rollout 6).
- r0→r2: `027-B3` cap vs `027-L3` (open Q11) — the ADR's literal posture kept (novel keys `503` at cap; held keys unaffected); cap raise / batched writes deferred, now against Rollout 4's measured rewrite (§Budgets). r2: the memory figure counts the largest transient too — a delta publishes by one linear two-way merge plus one index build, and holds old model, merged model, serialize buffer and page body (§The store pull).
- r0→r2: `027-B4` vs the 10k default page (open Q5) — `max_results=50000` pinned every page; an exhausted budget with `have_more_results` fails the pull; partial snapshots never publish. r3: `MAX_STEAM_PAGE_OCTETS` is the page *read* budget — 12 MiB, which the 50k-records-at-the-A17-shape estimate sits inside — not a worst case — over-bound store names make a page's true size unbounded, so an over-cap page is the §Risks residual — and the counted store spend is the refresh thread's — startup pulls are bounded only by process starts, so the runbook carries the restart policy (§Budgets, Rollout 6).
- r1→r2: one retry constant for two tempos (§The store pull, §Budgets) — split: page attempts space themselves by `STEAM_PAGE_RETRY_SECONDS`, and `STEAM_RETRY_SECONDS` is only the refresh thread's between-pull gap, so no pre-bind path sleeps an hour. r2: `STEAM_PULL_DEADLINE_SECONDS = 1_920` now covers the grid that family authorizes (`12 × (3 × 32 + 2 × 30) = 1,872`) instead of overriding `027-B4`, and a deadline-killed pull is its own named refusal, `steam pull deadline`.
- r0→r4: over-bound store names (open Q3) — the pull seam owns the normalization: a `char::is_control()` strip, then the shipped `discovery::game::friendly_name` whitespace-and-clamp rule (A7: that rule drops no other control bytes, so crediting it with the strip would have the instance publish a document its own loader refuses), and a name that normalizes empty stores empty — the record stays, so `027-C3`'s snapshot shape ships verbatim and every appid miss stays certifiable; r0's named pull failure, r1's whole-snapshot mark, and r2's `dropped_appids` field with its constant, merge rules, loader refusals and `steam snapshot incomplete` policy branch are deleted — a bookkeeping subsystem the clamp makes redundant, whose cap breach returned the remote-chosen denial of supply it existed to prevent and whose listed appid could never be answered against `027-S1` (§The store pull, §The steam snapshot resolver, §The hosted lookup policy, §Risks). Re-sizing the bound stays deferred.
- r0→r1: wall-clock freshness (open Q9) — shipped `026-C7` plausibility pair reused for the snapshot; future-dated forgery bounded at one day against a seven-day age bound. r1: `fetched_at` is the wall clock read before the pull's first page, never the publish stamp, so a delta's `if_modified_since` cannot sit above apps the pull never saw.
- r0→r2: `027-L2` id reuse after delete (unasked by the ADR) — `retired-ids-v1.json` bookkeeping document; every delete retires its id; the mint avoids held ∪ retired. r3: the append is a set insert and its cap is measured against the buffer the write renames, so a no-op re-delete against an at-cap list still completes the removal while a breaching write refuses `retired id list full` and mutates nothing; the `games ∩ retired` overlap a crash between the two persists leaves is named as the legal, recoverable state it is — the record keeps answering and the retried delete completes the removal; the cap's implied id count is stated and the runbook watches the file (§Issued entries).
- r1: games-only request pinned (§The store pull, §The hosted lookup policy) — no `include_*` flag is ever sent, so `200 []` certifies against game appids; an unexpected envelope or a missing `apps` list is a named page refusal, and the response shape is carried as unverified (A17).
- r0: epic/local miss shapes (open Q14) — `epic` named `503`, `local` `200 []`; both unifications rejected with reasons (§Options).
- r1: convergence evidence was overstated (§Options) — five of the six seed ids are what the rule mints; `portal-two` is the exception, and the cost of deleting a curated record whose appid later re-mints is Rollout 6's warning.
- r1: platform claims made honest (§Compatibility matrix, §Test strategy #16) — Windows' row now names the shipped `x86_64-pc-windows-msvc` compile leg (A16) and its new scope, macOS is dev-only rather than "Yes", and the client row says the `503`s fail a bound scan closed.
- r1: `query`'s empty answer and the `--corpus` exception (§Hosted mode) — the two `200 []` branches print `(no candidates)` with provenance `none`, and the leftover-document refusal spares the path `--corpus` names.
- r0: key custody and diagnostics (open Q8) — environment seam per `027-C3`, no request URI or key in any output; file custody deferred for both secrets together.
- r0: ADR says five seed entries, tree ships six incl. an undecided-format `epic/` key (A13) — design follows the tree; the record stays curated and `027-R1`'s slot stays inert.

## As-built (2026-11-22)

Record of what the branch actually shipped against the approved design above
(`git diff a22cfd26…15eaa58e`). The design decisions are unchanged; each entry
names the drift, why the implementation went the way it did, and the section it
touches. Items the design shipped verbatim — the five-step lookup policy and its
`503`/`200 []` shapes, the `027-C3` snapshot document and its loader refusals, the
`027-C5` derivation with its fold table and suffix ladder, the three-document
layout and its budgets (every `constants.rs` figure above is the value shipped),
one `mutations` mutex with the stated lock order, `Mode::Hosted`, the four-field
health and four-count `stats`, `DELETE /v1/admin/issued/<game_id>`, the flag
refusals, and `027-L4`'s no-key-in-diagnostics rule — are not repeated here.

**Rollout items not delivered in this change set.**

1. **Rollout 1 (ledger annotations) — not done.** `docs/adrs/026-registry-service.md`
   carries no `amended-by`/retirement lines for `026-C1`/`026-C2`/`026-C6`/`026-C7`
   /`026-B1`–`B3`/`026-L4`/`026-L5`/`026-R2`, and `024-C2` carries no `027-C5`
   cross-reference; the only ADR edit in the range is an unrelated code-span fix in
   ADR-027's own `027-R1` line. Reason: no stage of this run owned `docs/adrs/`.
   Affected: §Rollout item 1; the ledger now understates the design's reach.
2. **Rollout 4's measurement — not done.** The issued-document rewrite was never
   timed at 100k and 200k records, so §Budgets' "Rollout 4 records the measured
   issued-document rewrite" and the write-amplification risk row's "measured before
   the cap is called ordinary" have no number behind them. The shape-level
   mitigations did land (`Catalog::to_bytes_with` serializes against the live
   model, `Catalog::insert` splices the posting instead of rebuilding the index,
   pinned equal to the whole-document spelling by a `corpus.rs` unit test).
   Affected: §Budgets, §Risks row 3.
3. **Rollout 6 (`docs/registry-hosting.md`) — not landed.** The file does not
   exist. This matters more here than in a normal run because three postures the
   design pushes into the runbook are the *only* recovery path in the shipped
   code: deleting a crashed run's `write.lock`, forcing a full refresh by deleting
   the snapshot, and reading the retired file's size against its cap. Affected:
   §Rollout item 6, §Concurrency, §Issued entries, §Compatibility matrix's client
   row.

**Retirement landed in place, not by deletion.** `apps/registry/src/{cache,
upstream,endpoint}.rs` are still on disk — declared by no target, referenced by no
code, and their constants are gone from `constants.rs` — and the four A15 suites
(`caching_policy.rs`, `cache_document.rs`, `upstream_client.rs`,
`upstream_endpoint_gate.rs`) stay in the tree with every leg `#[ignore]`d and a
reason comment naming what replaced them. §Hosted mode's "Retired code … Retired
suites per A15" is therefore satisfied in behavior and not in file inventory.
Reason: a stage may not `rm`; deletions are the runner's file moves. Affected:
§Hosted mode.

**Design deltas.**

- **A library target the design never named.** `apps/registry/src/lib.rs` is new
  and publishes `constants`, `corpus`, `keys`, `mint`, `persist`, `retired`,
  `state`, `steam`; `serve`, `admin`, `http`, `wire` stay binary-only. Reason: the
  startup clause (load only a present, loadable **and** fresh snapshot) plus the
  no-outbound rule leaves the stale- and absent-snapshot `503`s, the loader
  refusals, the pull codec and the transport-assurance pair with no route through
  a booting binary, so Test strategy #2, #3, #6 and #11 are pinned at the seam
  instead. Consequence worth knowing: the four stale-boot legs the fix round routed to `test-update` first drove their postures
  over a socket (`hosted_lookup_policy` ×2, `health_route`, `cli_end_to_end`) —
  and they are now pinned off it: the two `hosted_lookup_policy.rs` legs and
   `health_route.rs` against `State::hosted(..)` over a `SteamSnapshot::for_test`
   fixture, and the `cli_end_to_end.rs` leg over the reachable `--no-steam` case.
   Affected: §The steam snapshot resolver,
  §Test strategy #2/#3/#6/#11/#13.
- **Three unnamed refusals guarding the `200 []` claim.** A full pull that
  collected no records fails by name; a continuation page that carries nothing and
  then reports itself finished fails the pull; a snapshot whose `apps` is empty is
  a loader refusal. None is in §The store pull or the loader list. Reason: a store
  that answers `200` with no records (a key without the `IStoreService` grant, a
  gateway envelope) would otherwise replace a good snapshot and have the instance
  certify `200 []` for every game appid for `027-B2`'s whole age bound — the
  §Options rejection of partial serving, applied to the degenerate page. Affected:
  §The store pull, §The steam snapshot resolver, and the honesty of policy step 5's
  `200 []`.
- **A store `name` that is absent or not a string takes the empty-label posture**
  rather than failing the page — a third spelling of the "normalizes to empty" case
  §The store pull already fixes, so the record and its appid stay.
  Affected: §The store pull.
- **Page retries are conditional.** §The store pull's "up to
  `STEAM_PAGE_ATTEMPTS` (3) attempts" is spent only on transient failures (socket,
  connect, non-`200`, mid-response close); a framing or shape refusal
  (`Transfer-Encoding`, an over-`MAX_STEAM_PAGE_OCTETS` body, a non-A17 envelope, a
  non-decimal `appid`) fails the page on the first attempt. Reason: the same
  request earns the same answer, and the calls are not free under A6's quota.
  Consequence: the §Budgets grid (`12 × (3 × 32 + 2 × 30) = 1,872`) is a ceiling on
  attempts, which is the direction the deadline needs anyway. Affected: §The store
  pull, §Budgets.
- **The `3xx` refusal is not a separate name.** Redirects, `429`, `5xx` and every
  other non-`200` share one line, "the store answered *status* where only 200 is an
  answer". The design lists `3xx` as its own rule and Test strategy #6 names it
  separately; the suite pins the case (`redirect`) but not a distinct spelling.
  Also minor: the request path is spelled `/IStoreService/GetAppList/v1` without
  the trailing slash the design's URL carries. Affected: §The store pull's response
  rules, §Test strategy #6.
- **One appid, one key spelling.** A `steam/<external-id>` key is a snapshot probe
  only when the external id is the *canonical* decimal spelling, so `steam/007` and
  `steam/+7` fall to the fresh-snapshot `200 []` arm instead of parsing to appid 7;
  and the mint's `steam-<appid>` spellings are built from the snapshot record's own
  `appid`, not from the text of the queried key. §Issued entries step 5 spells the
  fallback "from the queried key's external id". Reason: `u64::from_str` accepts
  padded and `+`-prefixed spellings, and one appid must have exactly one key — the
  property step 3/step 5's "appid-unique" claim, the mint's held-set probes and
  `027-B3`'s one-mint-per-key arithmetic all stand on. Affected: §The hosted lookup
  policy step 5, §Issued entries step 5.
- **Step 5's bound fires on *exceeds*, not *reaches*.** A derivation landing on
  exactly `MAX_SLUG_OCTETS` is minted whole; only the `g-` prefix and the
  `-steam-<appid>` suffix pushing past it trigger the substitution. §Issued entries
  step 5 says "exceeds" and §Test strategy #1 lists a plain 128-octet name among
  the cases that "all mint `steam-<appid>`"; the two are inconsistent and the code
  follows the rule text, with the boundary named in
  `mint_rule.rs::mints_steam_appid_when_the_bound_would_be_breached`. Affected:
  §Test strategy #1.
- **`write.lock` is presence-only, and now carries a pid.** A present lock file is
  the named refusal whoever stamped it; the pid written into it is operator-facing
  text in the refusal's parenthetical, never input to a decision (the `/proc`-based
  stale-owner adoption an earlier round of the implementation carried was removed
  in code review). The design says
  "finding it present is the named startup refusal" and says nothing about the
  file's contents. Also: the lock is taken before the steam-key and
  leftover-document refusals, and a refused run's `Drop` removes its own lock, so
  the tree leaves no lock behind a failed start. Affected: §Concurrency.
- **The `--corpus` exception compares files, not spellings.** `same_document` uses
  device+inode on unix and a case-insensitive path compare elsewhere, so two
  spellings of one file spare one document. §Hosted mode names the exception without
  specifying its equality. Affected: §Hosted mode.
- **Owner-only is a ceiling.** `persist::owner_only_dir` clears group/other bits
  and never sets one, so a `<data>` the operator holds at `0500` stays there and a
  write into it fails closed. A2 states `parent 0700` as the mode a write *creates*.
  Reason: widening it would answer Test strategy #4's injected-persist-failure write
  `200` after overriding a deliberate read-only posture. Affected: Assumption A2,
  §Issued entries' write discipline.
- **Refresh cadence anchored on `fetched_at`, and a failed publish scores as a
  failure.** The thread waits `last_pull_at + gap − now` with `last_pull_at` seeded
  from the loaded snapshot's own `fetched_at`, so a boot onto a nearly-age-bound
  snapshot refreshes before `027-B2`'s bound instead of idling a full day; and a
  pull that produced a snapshot the *publish* refused retries on
  `STEAM_RETRY_SECONDS`. §The store pull says "sleep
  `STEAM_SNAPSHOT_REFRESH_SECONDS` … retrying after `STEAM_RETRY_SECONDS`". The
  decision is the pure `steam::next_refresh_seconds` Test strategy #11 pins.
  Affected: §The store pull's refresh thread.
- **Dependency floors, spelled as Cargo can say them.** `rustls =
  ">=0.23.46, <0.24"` with `default-features = false` and
  `features = ["ring", "std", "tls12"]`, `webpki-roots = ">=1.0.10, <2"`, and
  `rustls-webpki = ">=0.103.16, <0.104"` declared as a **never-imported direct
  dependency** whose only job is to stop a re-resolution landing the transitive
  verifier below the patched line. Two additions to §Options: the `<` ceilings
  (it names floors only), and choosing the `ring` provider over the default
  `aws-lc-rs` — the design named the provider, so this is the design read
  literally; the ceilings and the declarative dependency are the implementers'.
  Resolved set: rustls 0.23.46, rustls-webpki 0.103.17, webpki-roots 1.0.12,
  ring 0.17.14, no `aws-lc-rs`. Affected: §Options, §Rollout item 3.
- **The gate changed in one more way than §Test strategy #17 counts.** The RUSTSEC
  screen landed as a third job, `advisory-gate`, running
  `cargo deny check advisories` (advisories only) at the repository root — the
  shipped scheduled `advisories.yml` stays triage input — and the Windows leg
  became `cargo check --workspace --exclude bindforge-registry --target
  x86_64-pc-windows-msvc` plus an explicit host `cargo check -p
  bindforge-registry --all-targets` step. Consequence the design did not foresee:
  `crates/core/tests/gate_contract.rs` pins `check.yml`'s job set, so that core
  suite's pinned list widened to three jobs. The `README.md` C-toolchain
  prerequisite line landed as Rollout 3 has it. Affected: §Rollout item 3, §Test
  strategy #16/#17.
- **Warning output is ASCII-only.** `persist::echo_safe` (relocated out of the
  retiring `upstream.rs`) folds typographic punctuation to its ASCII spelling and
  escapes any remaining non-`char::is_ascii()` value as `\uXXXX`, so every stderr
  warning is one printable-ASCII line. §The store pull's stated rule is the
  narrower one — no diagnostics carry the request URI or key. Affected: §The store
  pull's key custody.
- **Test fixtures.** `apps/registry/tests/fixtures/store/` carries four committed
  DER files (self-signed keypair for `partner.steam-api.com`, one for
  `some-other-name.example`) so Test strategy #6's chain-vs-hostname pair runs
  against a real loopback TLS listener on both cases, per #6's "the first case
  fails on the chain, not the hostname". The design describes the cases; the
  committed binary fixtures are new. Affected: §Test strategy #6.

**Verification addendum (2026-11-22, second documentation pass).** This pass
re-ran the same comparison, and the branch tip is unchanged — the range is still
`a22cfd26…15eaa58e` — so **no new drift was found and no entry above is added to
or changed by it**; what follows records that the list was re-checked against the
tree instead of carried forward. Re-verified on the current files: the
`constants.rs` §Budgets family is at the values this proposal prints (every
figure in §Budgets appears verbatim, including `STEAM_PULL_DEADLINE_SECONDS =
1_920` and the reused `026-C7` tolerance pair); `apps/registry/Cargo.toml` spells
the floors and ceilings exactly as the "Dependency floors" entry records, with
`rustls-webpki` declared and never imported; `.github/workflows/check.yml` still
carries the three-job set with `--exclude bindforge-registry` on the
`x86_64-pc-windows-msvc` leg, the host `cargo check -p bindforge-registry` step,
and `cargo deny check advisories`; `README.md` carries the C-toolchain
prerequisite. The file inventory confirms the retirement-in-place entries:
`src/{cache,upstream,endpoint}.rs` are on disk and declared by no target (no
`mod` line in `main.rs`), `src/lib.rs` publishes the eight modules named, and the
four A15 suites are `#[ignore]`d leg by leg with a reason naming what replaced
them. All eleven named refusals of §The hosted lookup policy, §Issued entries and
§Concurrency exist with the spellings this document prints (`state.rs`'s
`reason()` table plus `store_lock.rs`'s `registry data directory is locked` and
`main.rs`'s two flag-retirement lines), health is the four-key set and `stats`
the four-count set, `steam.rs` requests `{STORE_PATH}?key=…&max_results=…` with
`STORE_PATH = /IStoreService/GetAppList/v1` (no trailing slash, as recorded) and
`last_appid`/`if_modified_since` continuation, publishes through
`SteamSnapshot::published(now, …)` with `now` read before the pull, merges a
delta by the one linear two-way merge, refuses an empty worldlist and a
record-less continuation page, and computes the thread's wait through
`steam::next_refresh_seconds` seeded from the loaded snapshot's own
`fetched_at` (`main.rs`). `store_lock.rs` carries no `/proc` liveness logic —
the pid is text only, as recorded. `docs/registry-hosting.md` still does not
exist, and `docs/adrs/026-registry-service.md` and
`docs/adrs/024-game-slug-identity.md` still carry no `027-*` annotation, so
Rollout items 1, 4's measurement, and 6 stand unre-delivered exactly as items
1–3 of "Rollout items not delivered above" state. Two precisions, wording only,
no posture change: (i) the owner-only ceiling entry's `persist::owner_only_dir`
is the public entry point and the bit-clearing itself is its private
`narrow_to_owner_only` helper; (ii) the "no diagnostics carry the request URI or
key" posture now also carries the failure's *identity* — `steam.rs::io_reason`
renders the rustls/io error chain into the pull-failure line — which is an
addition to what §The store pull says about diagnostics and stays inside it,
pinned by `tests/steam.rs::no_pull_failure_line_carries_the_key_or_the_request_uri`.
Affected: no design section; this addendum amends only the record.
