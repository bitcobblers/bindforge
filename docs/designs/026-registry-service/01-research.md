# Research: Serve the catalog from a public root registry, with caching user-hosted registries (ADR-026)

## Summary

ADR-026 discharges the server half of `025-R1`: one `apps/registry` binary with a
**root** mode serving a catalog corpus and a **caching** mode holding the user's
own registrations plus a per-slug upstream cache, serving exactly the per-slug
`GET /v1/candidates?slug=…` question the shipped client already pins, with a
health route and a token-or-loopback-gated admin face — and it amends the
CLI's loopback-only endpoint policy to admit any literal-`http://` host. The
hosting model has strong precedent in pull-through caches (Docker Distribution's
`proxy.remoteurl` + `ttl`, Verdaccio's `maxage` uplinks, nginx's
`proxy_cache_use_stale error timeout`), and the per-slug-not-bulk choice is the
opposite of what a public catalog owner actually publishes (Valve's
`GetAppList` returns the whole catalog), so the decision is a deliberate
divergence rather than an unexamined one. The ADR's two local budgets are
jointly unsatisfiable at their own caps (`026-B1` = 5,000 cache entries against
`026-B2` = 1 MiB with per-entry payloads the `025` candidate budget leaves
open-ended), and no shipped manifest exists for `026-S1` to draw its game ids
from. The two places the design pays most are the JSON-vs-TOML format contract
clash at `schemas/catalog/v1/` (the conformance walk and the majors-equality
test in `crates/core/tests/` both sweep every new `schemas/*/v*/` directory),
and the deferred-TLS posture: `026-R1` leaves bearer admin traffic *and* the
registry→upstream hop in plaintext, where the second one is not just a
disclosure problem but a route by which someone else's curation lands in the
user's corpus and in the match store.

## Findings

### What the ADR changes against what is actually shipped

- The thing `026-C2` must be byte-identical to is precisely characterized:
  `read_response` accepts only status token `200` (not any 2xx), rejects
  `Transfer-Encoding`, rejects a duplicated or non-decimal `Content-Length`,
  caps the header block *and* the body at `MAX_RESPONSE_OCTETS = 65_536`,
  refuses more than `MAX_CANDIDATES = 256` candidates, and `read_exact`s
  exactly `Content-Length` body octets; the client sends one request per
  connection with `Connection: close` and `Accept: application/json`
  (source: apps/discovery-cli/src/catalog.rs).
- The client never explains a non-`200` beyond the status line: a non-`200`
  becomes `Unreachable("the catalog answered \`<status line>\`")`. So the
  named `503` of `026-L4` reaches the user only as the status line the
  registry chooses to print, and any reason phrase or body the registry
  writes is discarded — the spec should pin the status line text, not just the
  code, if "cold fails closed, never empty" is meant to be legible
  (source: apps/discovery-cli/src/catalog.rs;
  docs/adrs/026-registry-service.md `026-L4`).
- `026-L1`'s retirement surface is three named places, all of which assert
  the loopback spelling today: `LOOPBACK_HOSTS = ["127.0.0.1", "[::1]",
  "localhost"]` plus the non-loopback-resolution refusal in `probe()`, the
  unit test `the_policy_refuses_everything_beyond_the_loopback_spelling`
  (which asserts refusal for `http://catalog.example` and
  `https://127.0.0.1:8080/`), and the subprocess suite
  `endpoint_policy_refuses_everything_beyond_the_loopback_spelling` (which
  iterates `http://catalog.example`, `http://catalog.example:8080/v1`,
  `http://0177.0.0.1/`, … and asserts one line naming `--catalog-url`,
  `BINDFORGE_CATALOG_URL` and ADR-026). The refusal string itself hard-codes
  "with a loopback http:// endpoint — ADR-026 owns the real transport", so
  `026` re-pins both the tests *and* the message it is named in
  (source: apps/discovery-cli/src/catalog.rs;
  apps/discovery-cli/tests/cli_catalog_gate_tests.rs).
- What `026-L1` keeps is separable and already written: `probe()` resolves the
  host, refuses an empty resolution set, then proves one
  `TcpStream::connect_timeout` before any plugin is constructed, with
  `CONNECT_TIMEOUT = 2s` / `READ_TIMEOUT = 5s`. The amendment deletes the
  `is_loopback()` loop and nothing else (source: apps/discovery-cli/src/catalog.rs).
- `026-C7`'s `<data>` is already a shipped rule: `data_dir()` = absolute
  `$XDG_DATA_HOME`, else absolute `$HOME` + `/.local/share`, Windows =
  absolute `%APPDATA%`, a relative value being *unset*; `store_path()` joins
  `bindforge/matches.json`, so the cache document adds a `registry/` level
  under the same tree. Both platform branches are unit-tested through a
  `StoreLayout` + environment-lookup seam rather than by running on two OSes —
  the same seam the cache path will want (source: apps/discovery-cli/src/matches.rs).
- The "tmp-and-rename discipline `matches.rs` establishes" that `026-C4`
  cites is stronger than the phrase: a *fixed* tmp name opened
  `create_new(true)` at mode `0600`, parent dir created `0700`, rename into
  place, re-`chmod` after the `umask`, and — pinned by
  `the_temp_name_is_opened_exclusively_only` — a *hard failure* rather than a
  write-through when a stale tmp entry exists. Two `registry serve` processes
  pointed at one corpus therefore fail loudly on the second save, which is a
  concurrency answer the ADR never states (source: apps/discovery-cli/src/matches.rs).
- The untrusted-at-load discipline `026-C3`/`026-C7` invoke is also already
  concrete: read-budget check on `metadata().len()` *before* the read, version
  equality check, per-field grammar and bound checks, refusal names the file,
  nothing on disk is rewritten, and every echoed value goes through
  `escape_and_truncate` so one hostile file cannot forge extra stderr lines
  (source: apps/discovery-cli/src/matches.rs; schemas/README.md §"Input budget
  and chain bound").
- The corpus-boundary law `026-L6` inherits a validated-grammar seam, not a
  new one: `game_id_is_valid` = `is_slug_name(id) && len ≤ MAX_SLUG_OCTETS`,
  applied at candidate parse (`partition_candidates`), at confirmation write,
  and at store load. `026-C4`'s admin intake and `026-C6`'s upstream ingest
  are two *new* sites for that same predicate, and the caching registry is the
  site where the input is remote (source: apps/discovery-cli/src/matches.rs;
  apps/discovery-cli/src/catalog.rs).
- Ledger mechanics for Rollout item 6 have a visible precedent form: the
  amended unit's own line gains a trailing annotation, as in
  `013-L5 … amended-by 024-B1 (ADR-024, per 021-L4)`; `021-L2` then requires
  every handle to resolve and every unit to carry at least one handle, and
  `021-L5` requires each reservation to name an owner or `unassigned` — which
  `026-R1`/`026-R3` satisfy by the latter and `026-R2` by ADR-027
  (source: docs/adrs/013-game-manifests.md `013-L5`;
  docs/adrs/021-decision-unit-ledger.md §Laws;
  docs/adrs/025-discovery-identity-and-catalog-matching.md Rollout 5).
- Rollout item 1 names tasks `T-004`…`T-008` for the implementation units;
  `docs/tasks/` currently holds only `T-001`–`T-003`, and `023-C2` fixes the
  name shape `docs/tasks/T-NNN-slug.md` with `doc_type: task`,
  `status: pending`, `type`, `functional_area` (source: docs/tasks
  listing; docs/adrs/023-decoupling-intent-from-implementation.md §Contracts;
  docs/tasks/T-003-polish-discovery-cli-output.md frontmatter as the working
  example).
- Workspace placement is free: `[workspace] members = ["crates/*", "apps/*"]`
  already matches an `apps/` crate, so `apps/registry` needs no workspace
  edit — and the file's own comment warns that a glob matching nothing aborts
  every gate command, which is the constraint that made `apps/*` legal only
  once an app crate existed (source: Cargo.toml).

### Where the format contract and the ADR disagree

- `schemas/README.md` states "Formats are TOML (ADR-011-015) in a documented
  subset of TOML v1.1.0", while `026-C3` describes the catalog document in
  JSON-shaped braces and `026-C7` names a `.json` file — the two shipped
  per-user documents (`matches.json`, and the candidate body) are JSON and
  live *outside* the formats contract. So `schemas/catalog/v1/` is either a
  new *kind* of entry in `schemas/` (a spec + fixtures for a non-TOML format)
  or the catalog document is TOML with JSON-shaped prose
  (source: schemas/README.md §"Formats"; docs/adrs/026-registry-service.md
  `026-C3`/`026-C7`; apps/discovery-cli/src/matches.rs).
- That choice is load-bearing because two shipped tests walk the tree
  mechanically. `conformance.rs` "walk[s] every
  `schemas/*/v*/fixtures/{valid,invalid}` directory" and
  `every_format_version_directory_ships_valid_and_invalid_fixtures` fails any
  version directory that does not ship at least one valid *and* one invalid
  fixture **as parsed by core's TOML path**; the walker
  (`crates/core/tests/common/mod.rs::fixtures`) selects `*.toml` files, so a
  directory of `.json` fixtures reads as an empty corpus. Separately
  `version_gate.rs` asserts "bindforge-core's declared majors must equal the
  majors present under `schemas/*/v*/`". A `schemas/catalog/v1/` directory
  therefore trips the harness unless the design either makes `catalog` a core
  format or states the carve-out in the harness and in `schemas/README.md`
  (source: crates/core/tests/conformance.rs; crates/core/tests/version_gate.rs;
  crates/core/tests/common/mod.rs).
- The budget family has one stated home and per-module exports:
  `MAX_DOCUMENT_BYTES`/`MAX_NESTING_DEPTH` in `toml.rs`,
  `MAX_SLUG_OCTETS`/`MAX_DISCOVERY_SLUG_OCTETS` in `placement.rs`,
  `MAX_NATIVE_NAME_OCTETS` in `manifest.rs`, chain/policy budgets in
  `chain.rs`/`document.rs`, with `schemas/README.md` carrying the bullet for
  each and asserting "the reference implementation exports all thirteen
  constants". `026-B1`–`026-B3` state no export location, and `026-B2`'s
  1,048,576 is byte-identical to `MAX_DOCUMENT_BYTES`, which the README treats
  as *already* the shared read cap — so the new constants either join the
  family (and the README's count changes) or the registry keeps private
  constants, which is the pattern `matches.rs` already broke with
  `STORE_READ_BUDGET_OCTETS = 65_536`
  (source: schemas/README.md §"Input budget and chain bound";
  crates/core/src/toml.rs; crates/core/src/placement.rs;
  crates/core/src/manifest.rs; apps/discovery-cli/src/matches.rs).
- `026-S1`'s id source does not exist: `data/manifests/README.md` states
  "The first shipped manifest lands as its own PR", `data/manifests/` holds
  only that README, and a tree-wide search finds no `manifest*.toml`. The seed
  corpus's "game ids drawn from the shipped manifest ids" has nothing to draw
  from, and `013-L5`'s name-agreement rule makes an id cheap to mint and
  expensive to change
  (source: data/manifests/README.md; tree search for `**/manifest*.toml` —
  none; docs/adrs/013-game-manifests.md `013-C2`/`013-L5`).

### The data the registry serves, in other people's catalogs

- The `026-C3` record shape is a well-trodden triple. IGDB's `external_games`
  row is exactly a source-key join fact: `uid` ("the other services ID for
  this game"), `external_game_source`, `platform`, `name`, and `game` (the
  curated IGDB id) — the difference from BindForge is that IGDB keeps source
  and id as two columns where this ADR keeps one string, `steam/<appid>`
  (source: https://api-docs.igdb.com/).
- The curated-join corpus at community scale is proven: Wikidata's
  `P1733` "Steam application ID" external identifier is carried by ~129,308
  items, maintained as public edits with constraint reports
  (source: https://www.wikidata.org/wiki/Wikidata:Database_reports/List_of_properties/all;
  https://www.wikidata.org/wiki/Property_talk:P1733).
- The catalog is bigger than the ADR's "tens of thousands": SteamDB's release
  counter reads 149,301 cumulative games with 27,496 released in 2026 alone
  (source: https://steamdb.info/stats/releases/).
- Valve's public answer to "serve the catalog" is the bulk surface this ADR
  rejects: `GET https://api.steampowered.com/ISteamApps/GetAppList/v2/`
  "Gets the complete list of public apps" in one call
  (source: https://partner.steamgames.com/doc/webapi/isteamapps). The
  divergence is coherent only because BindForge's client asks one slug per
  installed game and never browses; the ADR's `026-L3` rule should say that
  out loud, since the root's *operator* will be asked for a bulk export by
  exactly the people who read `stats`.

### Caching-registry precedent for the two modes

- Docker Distribution's pull-through cache is the structural twin of caching
  mode: a `proxy.remoteurl` plus a `ttl` (docs example `ttl: 168h`), caching
  "what the consumers asked for" rather than mirroring the upstream
  (source: https://distribution.github.io/distribution/recipes/mirror/).
- Verdaccio's uplinks name the two knobs `026-B3` and `026-C6` re-invent:
  `maxage` for freshness and `cache: false` for metadata-only caching, and its
  docs make "cached packages install without touching the network" the
  documented offline posture (source: https://www.verdaccio.org/docs/uplinks/;
  https://www.verdaccio.org/docs/caching/).
- The "serve stale only when the refresh failed" rule is standard HTTP cache
  vocabulary, not an invention: RFC 5861 registers `stale-if-error` and
  `stale-while-revalidate`, and nginx's documented recipe is
  `proxy_cache_use_stale error timeout http_500 http_502 http_503 http_504` —
  deliver the stale object instead of relaying the origin error
  (source: https://www.rfc-editor.org/rfc/rfc5861;
  https://blog.nginx.org/blog/nginx-caching-guide;
  https://nginx.org/en/docs/http/ngx_http_proxy_module.html).
- The precedent worth reading as a warning is eviction, which is exactly the
  `026-B1` rule: Distribution's mirror has a long-standing report of *not*
  clearing expired entries ("Mirror not clearing expired entries", issue
  #1994) despite documentation saying it should
  (source: https://github.com/distribution/distribution/issues/1994).
- The no-bulk alternative also has working precedent on the *other* side:
  Heroic/Legendary-family tooling consumes whole-store JSON catalogs (Epic's
  and GOG's own catalog dumps) rather than per-title lookups, which is what
  makes a launcher's offline browse possible
  (source: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher).
  BindForge's design buys its `026-B2`-bounded cache by giving that up.

### Amendment and consequences the ledger has to carry

- The ADR's own cost line — "the third ADR-era identity/policy surface in a
  week" — is verifiable: `024-C1/C2/L1/L2` were amended by ADR-025 on
  2026-11-18, `025-C4`/`025-L1` are amended by `026-L1` on 2026-11-19, and
  `021-L4` is what makes this legal without renumbering
  (source: docs/adrs/025-discovery-identity-and-catalog-matching.md frontmatter
  and §Amends; docs/adrs/026-registry-service.md frontmatter;
  docs/adrs/021-decision-unit-ledger.md `021-L4`).
- The gate suite's own invariant text is a compatibility claim the amendment
  must keep: "no test performs a non-loopback request" is asserted in the
  gate suite's header. Admitting any host does not license a test that dials
  out, so the registry's own suites must keep the in-process fake
  (`tests/common`'s loopback `TcpListener` helpers: `respond_to`,
  `candidates_body`, `closed_loopback_port`) as the only server in CI
  (source: apps/discovery-cli/tests/cli_catalog_gate_tests.rs;
  apps/discovery-cli/tests/common/mod.rs).

## Prior art

( lens: prior art — how comparable projects serve a curated catalog and a
caching hop )

- **Docker Distribution** solved "one service, two modes" the same way:
  the same binary/registry speaks as a mirror when `proxy.remoteurl` is set
  and as the origin otherwise, and the client cannot tell — the property
  `026-C1` states as "the serving face is byte-identical between modes" is the
  property that makes a mirror swappable for the origin in daemon config
  (source: https://distribution.github.io/distribution/recipes/mirror/).
- **Verdaccio** is the closest precedent for the *whole* ADR: a local registry
  that holds what you published (its "own registered entries"), proxies an
  uplink, caches the metadata for what you fetched, honors `maxage` for
  freshness, and keeps working offline — with the explicit posture that the
  proxy is a *cache of what you touched*, not a copy of npm
  (source: https://www.verdaccio.org/docs/caching/;
  https://www.verdaccio.org/docs/linking-remote-registry/).
- **Maven/JFrog-style "proxy repositories"** and npm mirrors generalized the
  same vocabulary (upstream, cached metadata, TTL, and a distinct
  "hosted" repository for what you curate yourself) — the two-mode split in
  `026-C1` is the industry default spelling, not a novel shape
  (source: https://www.verdaccio.org/docs/uplinks/ as the minimal statement;
  https://distribution.github.io/distribution/recipes/mirror/).
- **Catalog data models**: IGDB keeps the source-key table as its own
  endpoint (`/v4/external_games`) keyed by `(uid, source)`; PCGamingWiki keys
  its Cargo game table by `Game.Steam_AppID`; Wikidata carries the same fact
  as property `P1733` on curated entities. All three keep source and id
  separate from the display name, which is the split `026-C3` encodes as
  `{game_id, name, source_keys}` (source: https://api-docs.igdb.com/;
  https://www.pcgamingwiki.com/wiki/PCGamingWiki:API;
  https://www.wikidata.org/wiki/Property_talk:P1733).
- **Curation intake (reserved as `026-R2`)**: the long-lived models are
  MusicBrainz (assigned, merged, never derived, with editorial maintenance as
  the acknowledged permanent cost) and the wiki-style edit-and-correct model
  Wikidata uses for identifier claims. Both treat *dispute resolution*, not
  ingestion, as the expensive part — which is the honest scope of what ADR-027
  inherits (source: https://musicbrainz.org/doc/MusicBrainz_identifier;
  https://www.wikidata.org/wiki/Property_talk:P1733).
- **What comparable projects regret**: mirror staleness and eviction
  (Distribution #1994), health endpoints that never fail when the thing they
  watch is down (see Standards below), and — for the identity half — automatic
  cross-store merges, which GOG Galaxy's user base has spent years asking to
  be allowed to override. `026-L5`'s "local wins outright, never merged" is
  the direct lesson
  (source: https://github.com/distribution/distribution/issues/1994;
  https://www.gog.com/forum/general_beta_gog_galaxy_2.0/manual_merge_for_any_games).

## Platform notes

( lenses: standards, platform notes )

- **Toolchain floor**: Rust `1.85`, edition `2024`, pinned by
  `rust-toolchain.toml` and `[workspace.package] rust-version`; `serde_json`
  is already a dependency of the shipped client, so "std-only for its serving
  face" (`026-C1`) cannot mean "no dependencies" — it means no HTTP/TLS
  runtime. Reading it as "no `std::net` alternatives" is the safe
  interpretation
  (source: rust-toolchain.toml; Cargo.toml; apps/discovery-cli/src/matches.rs).
- **`026-C5`'s bearer scheme carries a normative requirement the ADR waives**:
  RFC 6750 states "TLS is mandatory to implement and use with this
  specification" and its recommendations say clients "MUST always use TLS
  (https) or equivalent transport security when making requests with bearer
  tokens". `026-R1` defers TLS to an edge, so the ADR should record that the
  admin face deviates from the scheme it names, rather than implying
  bearer-with-constant-time-comparison is the standard's answer
  (source: https://www.rfc-editor.org/rfc/rfc6750.html).
- **Constant-time comparison has a length leak if it is written naively**: the
  guidance is not to early-return on a length mismatch, and the ecosystem
  answer in Rust is `subtle::ConstantTimeEq`. A hand-rolled byte fold over the
  header value is the repo-shaped answer given the std-only stance
  (source: https://www.chosenplaintext.ca/articles/beginners-guide-constant-time-cryptography.html;
  https://developers.cloudflare.com/workers/examples/protect-against-timing-attacks/).
- **HTTP semantics the spec will be asked about.** RFC 9110 makes `PUT` the
  idempotent method and `POST` not idempotent (§9.3), requires a `405` to
  carry an `Allow` header listing the resource's methods (§15.5.6), treats
  request-syntax faults as `400` (§15.5.1) and semantically-bad-but-well-formed
  content as `422` (§15.5.21), and defines `503` as *temporary* server
  overload/maintenance permitting `Retry-After` (§15.6.4). Against that
  backdrop, `026-C2`'s "a malformed slug is a `404`" and `026-L4`'s `503` for
  "this instance has nothing for this slug" are both deliberate
  simplifications: they hide per-slug availability in status codes the
  standard reserves for other things, and `Allow` is unstated
  (source: https://www.rfc-editor.org/rfc/rfc9110.html;
  https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status/405).
- **A health route that always answers `200` is a status document, not a
  readiness gate.** Kubernetes' HTTP probes treat any 200–399 as success, so a
  caching registry reporting `upstream: down` still passes an ordinary probe;
  if the root is ever deployed under something that watches `/v1/health`, the
  `mode`/`upstream` fields are decoration. Decide whether `/v1/health` should
  answer non-`200` when `upstream` is `down` in caching mode, and whether
  `stats` (which is under `/admin`, so token-gated) is the only route that
  exposes corpus size
  (source: https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/).
- **Server-side HTTP framing is a new surface with a standard already
  written.** RFC 9112 §6.3 requires that a message with both
  `Transfer-Encoding` and `Content-Length` be treated as an error (and the
  connection closed), that an invalid `Content-Length` yield `400` and close,
  and that a body shorter than the promised length be treated as incomplete;
  the shipped client already *refuses* `Transfer-Encoding` on the response
  side, so the server must not emit it either
  (source: https://www.rfc-editor.org/rfc/rfc9112.html;
  apps/discovery-cli/src/catalog.rs).
- **Low-and-slow exposure is inherent to a thread-per-connection std server
  with no request-read timeout**, and the cheap-request/expensive-origin
  pattern is the general class: each cache miss costs a serial upstream fetch
  (`026-R3`), so an unauthenticated caching registry bound to a routable
  address turns one inbound request into one outbound lookup, and each unique
  slug is also a new cache entry that competes with the operator's useful
  entries under `026-B1`'s oldest-first eviction
  (source: https://www.cloudflare.com/learning/ddos/ddos-attack-tools/slowloris/;
  https://blog.qualys.com/vulnerabilities-threat-research/2011-11-02/how-to-protect-against-slow-http-attacks;
  docs/adrs/026-registry-service.md `026-B1`/`026-C5`/`026-R3`).
- **Query encoding is already decided by the client, and the server must
  match it**: `percent_encode` escapes everything outside RFC 3986 unreserved,
  so the wire spelling is `?slug=steam%2F900001` and control bytes travel as
  `%00`. RFC 3986 permits `/` unencoded in a query but requires the recipient
  to decode `%XX` consistently and warns about normalization and dot segments;
  `026-C2` says "percent-encoded" but does not state whether the registry
  percent-decodes before checking `025-B1`'s 128-octet cap (3× expansion is
  possible) or whether it decodes the `<game_id>` path segment of
  `DELETE /v1/admin/games/<game_id>` before matching
  (source: apps/discovery-cli/src/catalog.rs;
  https://www.rfc-editor.org/rfc/rfc3986.html).
- **Ephemeral port**: "default ephemeral, printed at start" is the
  bind-to-port-0-and-report-the-assigned-address convention; the IANA dynamic
  range is 49152–65535 but Linux's default local range starts lower, so the
  printed port is not predictable from the IANA table — the *printed* value is
  the contract (source: https://www.rfc-editor.org/rfc/rfc6335#section-6;
  https://en.wikipedia.org/wiki/Ephemeral_port).
- **Filesystem modes carry over unchanged**: the owner-only `0700` dir /
  `0600` file discipline in `matches.rs` is `cfg(unix)`-only with a documented
  no-op on Windows, so `cache-v1.json` and the corpus inherit "owner-only
  where unix modes apply"
  (source: apps/discovery-cli/src/matches.rs).
- Nothing in this ADR touches the input-event path or the Node orchestration
  toolchain, so no polling-budget or Node-version consequence applies; the
  `apps/*` workspace glob means the new crate joins the existing cargo
  fmt/clippy/test gates with no CI edit
  (source: AGENTS.md §Repo layout, §Test gates; Cargo.toml).

## Risks & unknowns

- **`026-B1` × `026-B2` do not close.** 5,000 cached entries against a
  1,048,576-octet load ceiling is ~210 octets per entry. A realistic entry
  (`"steam/123456"`, one short candidate, a numeric `fetched_at`) is
  ~110–130 octets, so the cap sits within ~2× of a *realistic* corpus — and
  the budget says nothing about the *legal* case, where each entry may carry
  up to `MAX_CANDIDATES = 256` candidates of `game_id ≤ 128` and
  `name ≤ 128` octets (~70 KB for one entry, i.e. ~15 entries exhaust the
  file). Whichever way the design resolves it — a per-entry candidate cap, a
  smaller `026-B1`, or a load that refuses the whole cache — the refusal path
  matters: `matches.rs`'s precedent is to refuse the *file*, which for a cache
  means a user's whole local lookup memory becomes one error line
  (source: docs/adrs/026-registry-service.md `026-B1`/`026-B2`;
  apps/discovery-cli/src/catalog.rs `MAX_CANDIDATES`;
  apps/discovery-cli/src/matches.rs `STORE_READ_BUDGET_OCTETS`).
- **Plaintext upstream hop is an integrity problem, not only a disclosure
  one.** `026-L2` frames the disclosure boundary as "the queried source key
  leaves", and `026-R1` as "link-visible" secrets. The third consequence of
  plain HTTP is that the *caching registry's* upstream answers are unauthenticated
  JSON: an on-path attacker controls which `game_id` a slug resolves to,
  that answer is persisted to `cache-v1.json`, and a `game_id` is the one
  identifier that "becomes a path component downstream". The grammar check
  narrows the payload but not the *identity*, so the attack is a legitimate
  id, chosen by the attacker, cached as the user's curation
  (source: docs/adrs/026-registry-service.md `026-C6`/`026-C7`/`026-L2`/`026-R1`;
  apps/discovery-cli/src/matches.rs header note and `game_id_is_valid`;
  docs/adrs/025-discovery-identity-and-catalog-matching.md `025-L5`).
- **The admin face's loopback auto-enable is a local-trust decision stated as
  a convenience.** "Enabled when the listener binds loopback-only" means every
  process that can reach `127.0.0.1:port` — including anything running as
  another user on a shared host with no `SO_REUSEPORT`/namespace isolation,
  and any browser-driven SSRF-shaped navigation — can upsert corpus records
  with no credential. The ADR does not say whether a configured token *also*
  applies on a loopback bind (both, or either)
  (source: docs/adrs/026-registry-service.md `026-C5`).
- **No request-side budgets anywhere.** `026-B1`–`026-B3` bound cache entries,
  cache read, and TTL; nothing in the ADR bounds the `POST /v1/admin/games`
  body, the request header block, per-connection request count, or the number
  of concurrently open connections — all of which the *client* side does bound
  (65,536-octet header and body, 256 candidates, 2 s/5 s timeouts)
  (source: docs/adrs/026-registry-service.md §Budgets;
  apps/discovery-cli/src/catalog.rs).
- **`fetched_at` is untrusted input but is also the eviction key and the TTL
  input.** `026-C7` makes the cache file untrusted at load, and `026-B1`'s
  eviction is "oldest `fetched_at`"; a load-time check that is not stated
  allows a future-dated entry to be immortal or a zeroed entry to be evicted
  first. Related: nothing states whether TTL arithmetic uses wall clock
  (`SystemTime`, which can jump backwards on NTP adjustment) or a monotonic
  clock, and the corpus/cache share the process
  (source: docs/adrs/026-registry-service.md `026-B1`/`026-B3`/`026-C7`).
- **Two new untrusted-shape files join a set the ADR itself calls out as a
  cost** — "their refusals must stay as named as the match store's" — and the
  match store's refusals are ~7 distinct pinned conditions across unit and
  subprocess suites today. The refusal vocabulary for the corpus (an
  *operator-authored* file that arrives by git PR, per the manifests'
  precedent) and the cache (a machine-written file) probably should not be the
  same set, and the ADR gives one rule for both
  (source: docs/adrs/026-registry-service.md §Consequences;
  apps/discovery-cli/src/matches.rs `every_untrusted_store_shape_is_the_named_refusal`;
  data/manifests/README.md).
- **A cold caching registry's first scan is a latency cliff with no stated
  bound on the registry→upstream hop.** The ADR keeps the client's serial
  per-slug queries (2 s connect / 5 s read each) and reserves prefetch
  (`026-R3`); `026-C6` additionally gauges upstream reachability via the
  upstream's `/v1/health`, which is a second round trip per cold query unless
  the health result is itself cached — a cost the ADR does not state
  (source: apps/discovery-cli/src/catalog.rs;
  docs/adrs/026-registry-service.md `026-C6`/`026-R3`).
- **Root-mode `--corpus` is a file an operator edits while the server runs.**
  The ADR fixes atomic *writes* through the admin face but never says whether
  the serving face re-reads the corpus, watches it, or is restart-only; with
  `revision` "never used as a sync cursor", there is no mechanism even to
  detect an out-of-band edit
  (source: docs/adrs/026-registry-service.md `026-C1`/`026-C3`/`026-C4`).
- **The amendment lands one day after the thing it amends, in the same
  constant family as two prior amendments** — the ADR counts this itself
  ("the third ADR-era identity/policy surface in a week"). The mechanical part
  most likely to be skimped is Rollout item 6's `amended-by` lines on
  `025-C4`/`025-L1` and the `025-R1` slot record, which `021-L2` makes a
  checkable join rather than prose
  (source: docs/adrs/026-registry-service.md §Consequences and Rollout 6;
  docs/adrs/021-decision-unit-ledger.md `021-L2`).
- **The ADR ships no product code but does change a user-visible refusal
  line.** Rollout item 5 is a *behavior* change to the CLI (remote hosts
  admitted) delivered by an ADR run whose stated output is "the contract and
  the two annotation edits and no product code". If item 5 really is deferred
  to `T-008`, then between this merge and that task the CLI still refuses
  non-loopback endpoints while `026-L1` states it must not — a documented
  divergence window the designer should either accept explicitly or reorder
  (source: docs/adrs/026-registry-service.md Rollout 1 and 5 and 7).

## Open questions

1. Is `schemas/catalog/v1/` a core TOML format (a `Document` arm, a
   `SUPPORTED_FORMATS`/`supported_majors` entry, `.toml` fixtures) or a JSON
   format that the conformance walk and the majors-equality test must be
   amended to carve out? Which files get edited, and does the
   "Formats are TOML" sentence in `schemas/README.md` change?
2. Is the catalog document TOML or JSON on disk, and is `version: 1` a bare
   integer (`026-C3` writes `version: 1`, every shipped format declares
   `version = "v1"`)? Which rule refuses an unsupported major, and does the
   version-gate error name "supported majors" for this format?
3. `026-S1`: `data/manifests/` is empty. Does the seed corpus mint ids that a
   future manifest PR must then adopt, or does a first manifest land first, or
   does the seed carry ids with no manifest at all (which is legal only if the
   registry never joins `game_id` against the manifest tree)?
4. Where do `026-B1`–`026-B3` live as exports — `crates/core` beside the
   thirteen-constant family, or private to `apps/registry` like
   `STORE_READ_BUDGET_OCTETS` — and do they get `schemas/README.md` bullets?
5. What bounds a *single* cached entry's candidate list, and what happens on a
   cache load over `026-B2`: refuse the file, start empty, or truncate to the
   newest `026-B1`-and-budget-satisfying entries?
6. Status-code precision: does a malformed slug stay a `404`, or move to
   `400`/`422`? Does `405` carry `Allow`? Does the `503` carry `Retry-After`,
   and what exact status line does it print, given the CLI shows the user
   nothing but that line?
7. Does `/v1/health` ever answer non-`200`, and does it *cost* anything (does
   answering `upstream: ok` perform a live probe, or read the last known
   state)? Is `/v1/health` ever gated?
8. What are the registry's own outbound limits — connect/read timeouts,
   response octet and candidate caps, redirect policy (follow or refuse?), and
   does it reuse the `025` client's response parser? Does it require the
   upstream's `/v1/health` before each cold query or cache that verdict?
9. Does the caching registry validate upstream `game_id`s against the
   `025-C3`/`013-L5` grammar *before* writing them into `cache-v1.json`, and
   does it validate `name` against `MAX_NATIVE_NAME_OCTETS`? `026-C7` hints at
   the `game_id` check only.
10. Concurrency: single-threaded accept loop or one thread per connection?
    What serializes an admin mutation against a cache write and against a
    serving read (readers can see a half-written document if the rename and
    the read are not ordered)? Is running two `registry serve` instances
    against one corpus a supported state or a refused one?
11. `026-C5` details: is the compared value the whole `Authorization` header
    or the token after `Bearer `; what is the maximum accepted header length;
    is the admin face enabled or disabled when a token is configured *and* the
    bind is loopback; and is `401` ever accompanied by `WWW-Authenticate`
    (RFC 6750 §3 spells that header)?
12. Does `registry query <slug>` write to the cache document as a side effect
    (026-C6 says it "runs exactly this policy once"), and does it exit
    non-zero on the `026-L4` cold case?
13. Does the router percent-decode `?slug=` and the `<game_id>` path segment,
    and is `025-B1`'s 128-octet cap applied to the decoded or the encoded
    form? Is the corpus keyed by the decoded string?
14. Is there any interim transport answer for the *upstream* hop specifically
    (e.g. `--upstream https://` admitted for the registry while the CLI stays
    plain-HTTP), or is `026-R1` a single blanket refusal?
15. Request-side budgets: max `POST` body, max header block, max candidates a
    root will ever compute for one slug before truncating (and is the answer
    ordered, so truncation is stable across requests)?
16. Does this run author `docs/tasks/T-004`…`T-008` as documents (per
    `023-C2`/`023-C4`), and if so are they written as part of this ADR run's
    rollout or left to the operator?
