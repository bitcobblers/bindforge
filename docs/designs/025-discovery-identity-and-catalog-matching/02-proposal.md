# Design: Split discovered-game identity into a source key, a curated game id, and a friendly name (ADR-025)

## Context

Source: `docs/adrs/025-discovery-identity-and-catalog-matching.md`, built on
`docs/designs/025-discovery-identity-and-catalog-matching/01-research.md`.
ADR-024 landed today and made a discovered slug an id the product can adopt
byte-for-byte; this design replaces that identity half. A discovered row's
`slug` becomes the catalog lookup key `<source>/<external-id>` — for Steam,
`steam/<appid>` read out of the `appmanifest_*.acf` the scan already parses —
and that key joins nothing: it is exempt from the shared slug grammar and from
`024-B1`, and rides its own bound, `025-B1`. Cross-source identity becomes the
curated `game_id` (an ADR-013 manifest `id`), which arrives only from a
candidate the user confirms and is remembered in a new per-user match store.
The display name's normalized form becomes a `friendly_name` row field that is
never user-visible and joins nothing. Discovery gains a connectivity gate that
refuses by name before enumerating. The run ships no catalog server: the
lookup is the client-side provisional spelling until ADR-026 (`025-R1`) states
the service contract.

## Goals / Non-goals

- Goals:
  - `DiscoveredGame.slug` carries `<source>/<external-id>`, built by the one
    constructor that is the only place that spelling exists, over the closed
    token set `steam · epic · local` (`025-C1`; canonical statement: Chosen
    design, "Row identity").
  - The Steam plugins read the ACF `appid` and emit `steam/<appid>`; a row
    whose appid is missing or not a numeric id is refused at the plugin, never
    given a name-derived identity.
  - `Namespace` and its serde spelling retire, `derive_slug` and the `-2`
    collision pass retire, and the `024` identity tests retire with them
    (Rollout 4).
  - `friendly_name` rides the row as display metadata under
    `MAX_NATIVE_NAME_OCTETS` (`025-C2`).
  - `MAX_DISCOVERY_SLUG_OCTETS = 128` lands beside `MAX_SLUG_OCTETS`, is the
    only thing the emit boundary checks, and is stated in `schemas/README.md`
    as a lookup-key cap and explicitly not a grammar bound (`025-B1`, `025-L3`).
  - A per-user match store, a connectivity gate with one named refusal, a
    candidate query per unresolved slug, user-chosen confirmation, and candidate
    passthrough in JSON mode (`025-C3`, `025-C4`, `025-L1`–`025-L4`).
  - The corpus boundary is pinned: no source key or friendly name reaches a
    manifest, a profile, or a path component (`025-L5`).
  - The ledger amendments land and are gate-pinned (Rollout 5).
- Non-goals:
  - No catalog server, hosting, auth, canonical REST surface, or curation
    intake (`025-R1`, ADR-026).
  - No Epic backend, and no `epic/` spelling is emitted this run (`025-R2`).
  - No `local` discovery backend; the token is declared, unpopulated.
  - No TLS-capable or authenticated transport (it is ADR-026's, with auth).
  - No auto-matching of a lone candidate, no candidate scoring, no batching of
    the query.
  - No search index or UI over `friendly_name`; no garbage collection of stale
    store records; no persistence of discovery results beyond the match store.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | `DiscoveredGame` is `{slug, namespace: Namespace, display_name, install_path, platform, codec_id}`, and `Namespace` is a kebab-case serde enum whose sole value is `Unknown` | verified | crates/discovery/src/game.rs:29-51 |
| A2 | The Steam scan core reduces each ACF to `AppManifest { display_name, install_dir }` and never reads `appid`; `steam_linux`'s module doc states "AppIDs are never read"; both plugins share this core | verified | crates/discovery/src/steam_common.rs:26-30,333-348; crates/discovery/src/steam_linux.rs:6-7 |
| A3 | ACF fixtures carry a quoted `appid` sibling of `name`/`installdir`, and the shipped KeyValues reader looks every key up case-insensitively (`eq_ignore_ascii_case`), so an exact-case appid read is not needed | verified | crates/discovery/tests/fixtures/steam/minimal/steamapps/appmanifest_900001.acf:3; crates/discovery/src/keyvalues.rs:47-62 |
| A4 | Identity is checked twice today: `derive_slug` + `is_slug_name`/`MAX_SLUG_OCTETS` in `discovered_game`, and again in the CLI's `refuse_invalid_slugs`; `resolve_collisions` runs once per plugin over its own rows | verified | crates/discovery/src/steam_common.rs:238-261,278-326; apps/discovery-cli/src/main.rs:115-129 |
| A5 | `is_slug_name` is `[a-z][a-z0-9-]*` over bytes and therefore refuses any `/`; `MAX_SLUG_OCTETS = 128` sits beside it and is the only exported slug bound | verified | crates/core/src/placement.rs:83,95-105; crates/core/src/lib.rs:116-117 |
| A6 | `MAX_NATIVE_NAME_OCTETS = 128` is exported, and `schemas/README.md` states the budget family and claims the reference implementation "exports all twelve constants" | verified | crates/core/src/lib.rs:112-115; schemas/README.md:99-145 |
| A7 | The CLI prints the five columns `Slug \| Namespace \| Display Name \| Install Path \| Platform` and `--json` prints one compact `serde_json` array of `DiscoveredGame`; T-003 pins that output as one valid JSON document with a fixed key set | verified | apps/discovery-cli/src/main.rs:32-38,86-98; docs/tasks/T-003-polish-discovery-cli-output.md:33-39 |
| A8 | `DiscoveryError` has exactly two string-carrying variants and the engine swallows plugin errors, so a named connectivity refusal has no home in `crates/discovery` today | verified | crates/discovery/src/registry.rs:1-5; crates/discovery/src/engine.rs:20-33 |
| A9 | `crates/discovery` has nine modules, none of them persistence or data-directory related, and its deps are `serde`, `bindforge-core`, `winreg` (Windows only) plus a `serde_json` dev-dep | verified | crates/discovery/src/lib.rs:1-9; crates/discovery/Cargo.toml:6-22 |
| A10 | The CLI package's deps are `discovery`, `bindforge-core`, `clap`, `serde`, `serde_json`, and those are the workspace's only external deps (`clap` and `serde_json` per-package, not workspace-level), so the new modules need no `Cargo.lock` change; the toolchain is pinned to 1.85.0 and `Cargo.lock` is committed and gate-used with `--locked` | verified | apps/discovery-cli/Cargo.toml:6-13; Cargo.toml:20-24; rust-toolchain.toml:7; .github/workflows/check.yml:38 |
| A11 | CI runs the full test suite on `ubuntu-latest` only; Windows code is `cargo check --target x86_64-pc-windows-msvc` (no test binary runs), and macOS has no leg | verified | .github/workflows/check.yml:17-38 |
| A12 | `conformance.rs` walks every `schemas/*/v*/fixtures/invalid/` and asserts each fixture is refused with a structured single-line message, so a new invalid manifest fixture is gated without a new test | verified | crates/core/tests/conformance.rs:198,283 |
| A13 | A ledger amendment is pinned in the shape "`amended-by 024-B1 (ADR-024, per 021-L4)." on the amended unit's entry, gated by `ledger_annotations.rs` reading the corpus through its `ledger_unit` helper | verified | docs/adrs/013-game-manifests.md:114-117; crates/core/tests/ledger_annotations.rs:56-70,158-181 |
| A14 | Steam documents the AppID as the unique identifier of an application, and it is what Steam's own APIs key on | unverified | https://partner.steamgames.com/doc/store/application (via 01-research.md Findings) |
| A15 | Epic's `CatalogItemId` is unique only within a product namespace, so the `epic` external-id will likely need `epic/<namespace>/<itemId>` | unverified | https://dev.epicgames.com/docs/glossary (via 01-research.md Findings, Risks) |
| A16 | The per-user data directory a Linux/Windows product reaches for is `$XDG_DATA_HOME` (fallback `$HOME/.local/share`) and `%APPDATA%`, and a Flatpak/Snap sandbox is already handled by `XDG_DATA_HOME` pointing into `~/.var/app/…` | unverified | https://specifications.freedesktop.org/basedir-spec/latest/; https://learn.microsoft.com/en-us/windows/win32/properties/known-folders (via 01-research.md Platform notes) |
| A17 | `ureq`'s default features are `rustls` + `gzip`, its rustls provider is `ring`, and its dependency tree is ~37 crates with 9 direct deps — so the assumption that its build needs a C toolchain for the *target* (and so the CI Windows compile-check leg in A11 with it) is plausible but unexercised here | unverified | https://docs.rs/ureq (TLS and features sections) |
| A18 | `/` needs no percent-encoding inside a query value under RFC 3986 §3.4, but nothing in this repo has ever put a `/`-bearing string in a query, and no server-side reading of it is specified anywhere | verified (RFC) / unverified (server reading) | https://www.rfc-editor.org/rfc/rfc3986#section-3.4; 01-research.md Platform notes |
| A19 | A store re-mints a store id for an existing game occasionally ("re-listings"), which is the cost that keeps re-confirmation honest; nothing surveyed quantifies it | unverified | docs/adrs/025-discovery-identity-and-catalog-matching.md §Consequences; 01-research.md Risks |
| A20 | The shipped corpus carries no `/`-bearing identifier today: `data/` holds two READMEs and no valid schema fixture declares an `id` containing a `/` | verified | data/profiles/README.md, data/manifests/README.md (the tree's only files); grep of `schemas/` for `id = "` with a `/` in the value: no matches |

## Options considered

**Transport for the candidate query.** (a) Depend on a maintained blocking
client (`ureq`) and speak `https` now — rejected: the service does not exist
(`025-R1`), auth is explicitly ADR-026's contract, so an unauthenticated TLS
client is a client written against an incomplete contract; and on A17's
unverified but plausible reading it puts a C-toolchain build script in the
`x86_64-pc-windows-msvc` compile-check leg CI already runs, which is the gate
this run must not break. (b) Ship no transport at all — a trait plus an
in-process fake — rejected: ADR-025's Rollout 3 says the client lands, and
without a wire-level client the `025-C4` spelling and the query's encoding are
never exercised. (c) Chosen: a std-only HTTP/1.1 client that this build
restricts to a loopback authority, with `https` and any non-loopback `http`
refused by name. It is the whole `025-C4` client — URL spelling, request,
bounded response parse, JSON model — testable end-to-end against a
`TcpListener` in the test process, with zero new dependencies, and it cannot
disclose an installed-game inventory to a remote host before ADR-026 defines
the transport, auth, and hosting it owns. The named refusal for a real endpoint
is the honest edge; the TLS transport is Deferred.

**Where the connectivity gate lives.** (a) Inside `crates/discovery`'s engine —
rejected: the engine swallows plugin errors (A8), it would gain an HTTP
dependency and a network concept ADR-022 deliberately kept out of enumeration,
and `DiscoveryEngine` is not even the CLI's aggregation path. (b) Chosen: in
`apps/discovery-cli`, the only shipped scan driver, which is also where ADR-025's
Rollout 3 puts the client. Enumeration stays local and network-free in
`crates/discovery`, which is what ADR-022's rejection was really guarding.

**Where the `game_id` lives.** (a) A field on `DiscoveredGame` — rejected:
plugins know nothing about matches, so the field would be a lie at the plugin
seam and would put curated, machine-local state in the type a third-party plugin
constructs. (b) Chosen: the CLI composes the row with its match when it renders,
so the plugin row keeps exactly the fields a source can know, and the store
stays the only place a `game_id` is kept.

**Match-store format and location.** (a) A TOML document read by core's
schema-document reader — rejected: that reader is the schemas contract's
parser, with its budget and format table, and a runtime state file is not a
schema document. (b) One file per confirmed slug — rejected: no atomic
whole-store read, and directory enumeration for a two-field record. (c) Chosen:
one JSON document, `serde_json` (already a CLI dependency), sorted by `slug` for
a byte-stable file, written to a temp name and renamed.

**A Steam row with no usable appid.** (a) Fall back to the number in
`appmanifest_<n>.acf` — rejected: two sources of truth for one identifier, and
the ACF's `AppState.appid` is the one the store itself maintains. (b) Fall back
to a name-derived slug — rejected: that is `024-C2`, retired by `025-C2`, and a
row with a name-derived slug would join on nothing while looking like an id.
(c) Chosen: refuse the row through the existing count-only skip path, exactly as
an unusable `installdir` is refused today.

**A friendly name past its bound.** (a) Refuse the row — rejected: a display
string that joins nothing would delete a real install from its own machine's
listing. (b) Keep it unbounded — rejected: `025-C2` rides the sibling bound by
name. (c) Chosen: clamp to the longest prefix that is both within
`MAX_NATIVE_NAME_OCTETS` and on a `char` boundary. Truncation is admissible
here, and only here, because `friendly_name` is never joined to anything: a
truncated display string costs a shorter label, not a different identity.

## Chosen design

### Row identity — `crates/discovery/src/game.rs`

- `pub enum Source { Steam, Epic, Local }` with `as_str()` spelling the closed
  token set `steam`, `epic`, `local`, and no serde derive: the row carries the
  composed key, never the token, so `Source` is construction-side only.
- `Source::lookup_key(&self, external_id: &str) -> String` is the one place the
  `<source>/<external-id>` spelling exists. Every shipped row is built through
  it, which is why the emit boundary may check only `025-B1` and not the shape
  (`025-L3`): a plugin cannot spell the key differently without using another
  function.
- `DiscoveredGame` becomes `{ slug, friendly_name, display_name, install_path,
  platform, codec_id }`: `namespace` and the `Namespace` enum (with its
  `Display` and kebab-case serde) are deleted, and `friendly_name` sits after
  `slug`.
- `fn friendly_name(display_name: &str) -> String` states the only remaining
  normalization in this crate: trim leading and trailing whitespace, collapse
  interior whitespace runs to one space, then clamp to
  `bindforge_core::MAX_NATIVE_NAME_OCTETS` at a `char` boundary. It is metadata,
  not a derivation law: nothing parses it, and it is not user-visible (the UI
  prints `display_name`).
- `derive_slug` retires: `crates/discovery/src/slug.rs` is deleted, along with
  its `pub mod slug;` line in `lib.rs`. Nothing else consumes it (01-research.md
  Findings), and the friendly-name rule above is a whitespace rule, not a
  kebabbing rule.

### Emission — `crates/discovery/src/steam_common.rs`

- `AppManifest` gains `appid: String`. `parse_manifest` reads
  `app_state.get_str("appid")` — already a case-insensitive lookup (A3), which
  answers the research's exact-case question — trims it, and requires 1–20
  ASCII digits; anything else (`None`, empty, non-numeric, an embedded `/`) is
  the same skip-and-count outcome as an unusable `installdir`.
- `discovered_game` builds `slug` with `Source::Steam.lookup_key(&appid)` and
  the row's `friendly_name` from `display_name` (whose absent/empty-name
  fallback to `installdir` is unchanged). The identity gate becomes bound-only:
  `slug.len() > bindforge_core::MAX_DISCOVERY_SLUG_OCTETS` is the refusal
  (`025-L3`); `is_slug_name` and `MAX_SLUG_OCTETS` lose their imports here,
  since `steam/123` fails the shared grammar by design.
- `resolve_collisions` and its in-module unit tests are deleted: `025-L3`
  retires the suffix rule, because distinct installs of one Steam source carry
  distinct AppIDs (A14) and so leave no collision to break, while friendly-name
  duplicates are by design. The `(slug, install_path)` sort stays, which keeps
  the same game in two libraries as two rows and makes the row order the byte
  order of `steam/<appid>`.
- `steam_linux.rs`'s module doc loses the "AppIDs are never read" clause (A2,
  its only statement). The `steam` token is the only one with a shipped emitter:
  `epic` waits on `025-R2`, `local` on a backend this run does not write.

### Bound — `crates/core` and `schemas/README.md`

- `placement.rs` gains `pub const MAX_DISCOVERY_SLUG_OCTETS: usize = 128;`
  beside `MAX_SLUG_OCTETS`; its doc comment states that it caps a lookup key,
  not a grammar-validated id, that the key is exempt from `is_slug_name`, and
  that `025-B1` names it. `lib.rs` re-exports it and its public-surface
  paragraph names it as the thirteenth exported constant.
- `schemas/README.md` gains a sibling bullet in the exported budget family:
  ADR-025 adds `MAX_DISCOVERY_SLUG_OCTETS = 128`, the cap on the
  `<source>/<external-id>` discovery key, set equal to the two 128 bounds so the
  family reads as one and is explicitly *not* a slug-grammar bound — the
  discovery key is exempt from the grammar and never a path component; the
  bullet also records ADR-025's second, non-schema borrowing of
  `MAX_NATIVE_NAME_OCTETS` as the `friendly_name` display bound. The existing
  `MAX_SLUG_OCTETS` bullet is untouched: its rider set stays exactly as
  `024-B1` enumerates it, and the exemption lives in the new bullet so the two
  cannot drift. "all twelve constants" becomes thirteen.
- `crates/core/tests/zz_api_probe.rs` resolves the new constant in value
  position beside the existing family; `crates/discovery/tests/zz_api_probe.rs`
  is re-pinned to `Source`, `lookup_key`, and the new field set.

### Match store — `apps/discovery-cli/src/matches.rs` (new)

- `MatchStore` is one file, `<data>/bindforge/matches.json`, where `<data>` is
  `$XDG_DATA_HOME` when set and absolute (else `$HOME/.local/share`) on
  Linux/other, and `%APPDATA%` on Windows, a relative value being unset. The
  directory is resolved from an environment lookup closure, mirroring
  `steam_common::absolute_env` and `steam_linux::host_candidates`, so the rule is
  unit-tested without touching the process environment (A16). A Flatpak or Snap
  install needs no extra rule: its `XDG_DATA_HOME` is already the redirect.
- File shape: `{ "version": 1, "matches": [ { slug, game_id, friendly_name } ] }`,
  records sorted by `slug` byte order so a re-save is byte-stable; the store
  holds one record per distinct slug — two installs of one game are two records
  sharing one `game_id` — and there is no dedup logic at all because the store
  is keyed by `slug` alone.
- Load: an absent file is an empty store. Unparseable content, an unknown
  `version`, a file over a `65_536`-octet read budget, a `slug` past `025-B1`
  or not spelled `<token>/<external-id>` with `<token>` in `Source`'s closed
  set, or a `game_id` failing `is_slug_name` + `MAX_SLUG_OCTETS` is the named
  refusal `error: match store unreadable: <path>` before
  enumeration, and nothing on disk is rewritten; the user's remedy is to remove
  the file. `game_id` is validated at all three places it is seen —
  candidate parse, confirmation write, store load — because it is the one
  identifier here that becomes a path component (`025-L5`), and both the store
  file and the server response are untrusted inputs.
- Save: only after at least one confirmation, through
  `matches.json.tmp` (opened exclusively, never through a pre-existing entry)
  + `std::fs::rename`, after `create_dir_all` on the parent; the `bindforge/`
  directory and both files are created owner-only — 0700/0600 where unix modes
  apply — so the confirmed inventory is never a `umask` accident.
  Records never expire and are re-trusted on every scan (`025-L4`).

### Catalog client and gate — `apps/discovery-cli/src/catalog.rs` (new)

- `struct Candidate { game_id: String, name: String }`, `Deserialize` with
  unknown fields ignored.
- `trait CatalogClient { fn probe(&self) -> Result<(), CatalogError>;
  fn candidates(&self, slug: &str) -> Result<Vec<Candidate>, CatalogError>; }`
  is the seam every behavior below is written against, so the CLI's own tests
  drive it with an in-process fake and one subprocess test drives the real
  implementation over a socket.
- `HttpCatalog` is built from `--catalog-url` (a global clap flag beside
  `--json`) or `BINDFORGE_CATALOG_URL` — both untrusted config sources, like
  the store's env seam. The pure `fn parse_endpoint(base: &str) ->
  Result<Endpoint, CatalogError>`, the `candidates_url` of endpoint policy,
  admits exactly: a literal `http://` prefix; an authority that is non-empty,
  carries no `@`, and runs to the first `/`, `?`, or `#`, with an optional `:`
  and digits-only port; and a host that is exactly `127.0.0.1`, `[::1]`, or
  `localhost` — no other IP form parses. Anything else, or no endpoint at all,
  is `CatalogError::TransportUnavailable` naming the flag, the env var, and
  ADR-026 as the owner of the real transport.
- `fn candidates_url(base: &str, slug: &str) -> String` is pure:
  `<base>/v1/candidates?slug=<encoded>`, percent-encoding every byte outside
  RFC 3986's unreserved set, so `steam/900001` travels as `steam%2F900001`
  (A18: the literal `/` is legal but a normalizing gateway is not, and the
  encoded form is unambiguous). One GET per distinct unresolved slug — the
  ADR's singular call shape; batching is `025-R1`'s.
- The request is std-only: `TcpStream` with a 2 s connect and 5 s read timeout,
  `GET <path> HTTP/1.1` with `Host`, `Accept: application/json`, and
  `Connection: close`. The response parse refuses anything it has not been
  told to understand: status other than `200`; a `Transfer-Encoding` header
  (header names matched case-insensitively); not exactly one `Content-Length`,
  accepted only as 1–5 ASCII digits naming at most `65_536` body octets; a
  header block over the same `65_536`-octet budget before its blank line; a
  body that is not valid `Vec<Candidate>` JSON or holds more than `256`
  candidates. Each is `CatalogError::Unreachable(reason)`.
- `probe()` asks the authority, not an endpoint: resolve the host/port and
  `TcpStream::connect_timeout` each address the resolver returns, after
  refusing unless every returned `SocketAddr`'s `IpAddr::is_loopback()` — the
  property is established by address, not by name, so a `hosts` entry for
  `localhost` cannot send the query outward. It therefore invents no path
  ADR-026 owns, and it is what enforces `025-L1` "before enumerating". A
  mid-scan query failure is the same named refusal, and the run fails closed
  with no partial table.
- Two named refusals, one gate: `error: catalog unreachable: <reason>` for a
  probe or transport/parse failure, `error: catalog transport unavailable:
  <reason>` for the endpoint policy. Both go to stderr, both exit non-zero, and
  every value a refusal interpolates — a slug, a candidate `name`, a rejected
  `game_id`, the store `<path>`, the connect `<reason>` — goes through
  `bindforge_core::escape_and_truncate`, so no outside string forges a line or
  leaks a path.
- A candidate whose `game_id` fails `is_slug_name` + `MAX_SLUG_OCTETS` is
  dropped with one named stderr refusal line; the surviving candidates keep the
  response's order.

### Scan flow and output — `apps/discovery-cli/src/main.rs`

`Commands::List` becomes, in order:

1. Build `HttpCatalog` from the flag or env (endpoint policy), then `probe()`:
   failure is the named refusal and `ExitCode::FAILURE`, before any plugin is
   constructed (`025-L1`).
2. Load the match store (its named refusal also precedes enumeration, so a
   broken file can never be half-written by a later confirmation).
3. Build the plugin list and enumerate exactly as today — the only local step.
4. The emit boundary stays `refuse_invalid_slugs`, its body becoming the
   `025-B1` bound alone, its refusal line text unchanged, and its ADR-024
   shared-grammar doc comment reversed with the rest of that grammar.
5. For the distinct unresolved slugs (those the store does not resolve), in
   byte order, one `candidates()` call each. A slug the store resolves is never
   queried (`025-L4`).
6. Table mode: per distinct unresolved slug with candidates, one prompt on
   **stderr** — stdout carries only the table — reading one line from stdin
   through the pure `fn selection(answer: &str, count: usize) -> Option<usize>`
   (`1`-based; blank, `n`, out-of-range, or EOF is no match). A chosen candidate
   writes the one record for the slug, (`slug`, `game_id`, friendly name of the
   first sharing row in `(slug, install_path)` order); nothing is ever stored
   without that choice, and name similarity or a lone high-scoring candidate
   never completes the join (`025-L2`). The answer applies to every row sharing
   the slug — the store holds one record per distinct slug — so one Steam
   library scanned twice is prompted once.
7. JSON mode: no prompt, no prompt text, no store writes; candidates are
   emitted instead, per row.
8. Render. `COLUMNS` becomes `["Slug", "Game Id", "Display Name", "Install
   Path", "Platform"]` — `Namespace` retires, `Game Id` prints the resolved id
   or `-`, and the two ACF-sourced cells — `Display Name`, `Install Path` —
   render through `escape_and_truncate` in place of today's raw `row_cells`
   pass-through, so a hostile ACF byte cannot emit a terminal control sequence.
   The JSON row is one document per run (T-003's clause holds) with the
   key set re-pinned deliberately:

   ```json
   {"slug":"steam/900001","friendly_name":"Example Game","display_name":"Example Game",
    "install_path":"/steam/steamapps/common/Example Game","platform":"SteamLinux",
    "codec_id":null,"game_id":null,
    "candidates":[{"game_id":"example-game","name":"Example"}]}
   ```

   `candidates` is `[]` for a slug the store resolved, and `null` `game_id` is
   the unmatched state; `friendly_name` is discovery output, which `025-L5`
   allows.
9. `ExitCode::SUCCESS`.

### Corpus boundary and ledger

- `crates/discovery/tests/boundary_tests.rs` (new): for every slug the Steam
  fixture trees emit, `!is_slug_name(slug)` and the slug is inside `025-B1` —
  the structural proof, for the source key only, that it cannot become a
  product id, because the grammar refuses it (A5); the friendly-name half of
  `025-L5` rests on the match store rule that only `game_id` is ever an id or
  path component — no grammar assertion is made about friendly names.
- `schemas/manifests/v1/fixtures/invalid/id-source-key.toml` (new), whose `id`
  is `steam/900001`: refused by A12's existing conformance walk, no new test.
- A corpus sweep asserting no file under `data/` or
  `schemas/*/v*/fixtures/valid/` contains `steam/`, `epic/`, or `local/` —
  Rollout 4's "no source key or friendly name reaches a manifest or profile",
  pinned literally against the shipped corpus (A20).
- Ledger: in `docs/adrs/024-game-slug-identity.md`, `024-C1`, `024-C2`,
  `024-L1`, `024-L2` gain `amended-by` entries naming `025-C1`, `025-C2`,
  `025-L3`, `025-L3` in A13's shipped shape, and `024-R1` gains the line
  recording ADR-025 as filling its slot with the publisher-namespace half
  superseded. `crates/core/tests/ledger_annotations.rs` gains the gate that
  reads those five claims off the corpus through its existing `ledger_unit`
  helper; `024`'s prose and `024-B1`/`024-L3` stay untouched as history.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| No reachable catalog exists, so every real scan refuses with the named error in this build | Certain (ADR-026 has no service; `025-R1`) | Low (there is nothing to reach) | The refusal names the flag, the env var, and ADR-026; the whole client path is exercised against a loopback fake; the transport, auth and hosting are Deferred to `025-R1` |
| `024`'s identity fixtures and tests re-pin a second time, one day after landing | Certain (Rollout 4) | Low | Deletions are named (`slug.rs`, `slug_tests.rs`, `collision_tests.rs`, `resolve_collisions`, `Namespace`); the API probe is re-pinned in the same commit; the gates run at branch scope |
| Every scan discloses the user's installed-game inventory keyed by third-party ids | Certain by design, once a service exists | Medium | This build can only reach a loopback authority; one narrow singular query per slug, no batching, user-initiated scans; TLS, auth and the disclosure surface are ADR-026's (`025-R1`) |
| An untrusted `game_id` (server response or hand-edited store) reaches a path component | Medium (both inputs are untrusted) | High | `is_slug_name` + `MAX_SLUG_OCTETS` at candidate parse, at write, and at load; `025-L5` keeps the source key and friendly name out of artifacts entirely; echoes through `escape_and_truncate` |
| A hand-rolled HTTP/1.1 response parse mishandles a hostile or odd response | Low (loopback peers only) | Medium | Only `200`; exactly one numeric `Content-Length` ≤ 64 KiB; `Transfer-Encoding` refused case-insensitively; header block and body octets and candidate count all capped before allocation; each refusal is a pinned unit case; a maintained client replaces it when ADR-026's transport lands |
| A store mints a fresh external-id for an existing game, leaving an unreachable store record (A19) | Low | Low | Accepted by ADR-025: a fresh lookup and a re-confirmation; no garbage collection is specified (Deferred) |
| A scripted table run blocks on the confirmation prompt | Medium | Low | Prompts go to stderr, stdin EOF and blank input decline deterministically, `--json` never prompts |
| Two install directories differing only in case collapse to one `local/<slug>` on a case-insensitive filesystem, and the store holds one record for both | Low (no `local` emitter ships this run) | Low | `025-L3` makes `local` exempt from uniqueness by design; the rule is stated where the token is defined |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux (ext4) | Yes | The full suite runs here (A11); the XDG data-dir rule, the Steam scan, the store round-trip, and the loopback client are all exercised on this leg |
| Windows (NTFS) | Compile-verified, with a named gap | `cargo check --target x86_64-pc-windows-msvc` runs no test binary (A11); the `%APPDATA%` branch of the store path is unit-tested through the env-lookup seam on Linux, not run on Windows |
| macOS (APFS) | Yes, no CI leg | The XDG fallback path is the same rule; nothing in this design is macOS-specific; runtime verification level is Test strategy #15's |
| Flatpak / Snap Steam installs | Yes | The `host_candidates` root probe (`crates/discovery/src/steam_linux.rs`) is unchanged; the store follows `XDG_DATA_HOME`, which is already the sandbox redirect (A16) |
| Source `steam` | Yes | `steam/<appid>` from `appmanifest_*.acf`, read case-insensitively (A3), refused when absent or non-numeric |
| Source `epic` | Declared, unpopulated | The token exists so the closed set is complete; no backend and no `epic/` spelling ships this run (`025-R2`, A15) |
| Source `local` | Declared, unpopulated | The token exists; no local backend is written this run, so its kebab'd external-id rule is stated but unpinned |
| Remote catalog service | No (named refusal) | This build's client reaches a loopback authority only; `https` and non-loopback `http` refuse by name. Hosting, auth, the canonical REST surface and the server-side reading of `?slug=` are ADR-026's (`025-R1`) |
| Third-party schema readers | Not affected | No published grammar, bound, or fixture is tightened; the one README addition states a non-grammar cap, and the `MAX_SLUG_OCTETS` bullet's rider set is unchanged (`024-L3`) |
| `tools/orchestration` (Node) | Not affected | Nothing in the pipeline reads discovery output; `npm ci && npm run check` is unchanged |
| Rust toolchain and dependencies (pinned 1.85.0) | Yes | No new dependency and no `Cargo.lock` change: the client is std-only (A10), so the Windows compile-check leg gains no build script |

## Test strategy

1. Unit: `Source::as_str` / `lookup_key` — the three tokens spell `steam`,
   `epic`, `local`; `lookup_key` emits exactly one `/` and composes
   `steam` + `900001` as `steam/900001`; the row's JSON carries no `source` and
   no `namespace` key.
2. Unit: `friendly_name` — trims, collapses whitespace runs, and clamps a
   multi-byte name to the longest `char`-boundary prefix inside
   `MAX_NATIVE_NAME_OCTETS`; a 128-octet name is untouched and a 129-octet name
   loses one octet without panicking or splitting a `char`.
3. Unit: appid extraction — `"appid" "900001"` yields the slug `steam/900001`;
   key casing (`AppId`, `APPID`) is accepted; a missing key, an empty value, a
   non-numeric value, an embedded `/`, and a 21-digit value each yield no row
   and a `skipped` delta of exactly one, asserted with a `SteamRoot` helper that
   can write an ACF with no `appid` key.
4. Unit: bound-only boundary — a slug of exactly `MAX_DISCOVERY_SLUG_OCTETS`
   survives `refuse_invalid_slugs` and a 129-octet one is refused, untruncated,
   with today's escaped line text; and `steam/1` — grammar-invalid by design —
   survives, which the retired grammar check refused; the bound skip in
   `discovered_game` is pinned separately as an in-module `AppManifest`
   construction with an over-long appid — the only reachable route there,
   defense in depth.
5. Unit: store path — `$XDG_DATA_HOME` absolute wins, a relative value is
   treated unset and falls back to `$HOME/.local/share`; the Windows branch
   resolves `%APPDATA%` the same way; all through the env lookup closure.
6. Unit: store load — absent file is empty; a valid file yields records; a
   truncated JSON body, `version: 2`, a file over the 65 536-octet budget, a
   `slug` past `025-B1` or spelled outside `<token>/<external-id>`, and a
   `game_id` of `../x` each produce the named refusal naming the path, and the
   file's bytes are unchanged afterwards; a newline in `$XDG_DATA_HOME` still
   yields one escaped line.
7. Unit: store save — nothing is written without a confirmation; a confirmation
   writes the file with records sorted by `slug`; re-saving the same store is
   byte-identical; no `.tmp` file remains; on unix the directory and both files
   are owner-only (0700/0600) and the tmp open is exclusive.
8. Unit: `selection` — `"1"` and `"2"` map to indices, `" 2 "` is accepted, and
   `""`, `"n"`, `"0"`, `"3"` over two candidates, and EOF all answer `None`.
9. Unit: `candidates_url` and endpoint policy — `steam/900001` becomes
   `?slug=steam%2F900001`, a slug with a space or `%` is encoded once (never
   double-encoded), and `http://127.0.0.1:PORT` is accepted while `https://…`
   and `http://catalog.example` raise `TransportUnavailable` naming the flag, the
   env var and ADR-026; an unset flag and env raise the same name; so does
   `parse_endpoint` on `http://127.0.0.1:8080@host.example/`,
   `http://0177.0.0.1/`, `http://:8080/`, and `http://127.0.0.1:PORTx/`.
10. Unit: response parse — a `200` with a small JSON body yields the candidates
    in response order; a `404`, a `500`, a `Transfer-Encoding` header (also as
    `transfer-encoding`), a missing, duplicated, or non-numeric `Content-Length`,
    a header block over 65 536 octets, a body over 65 536 octets, a
    257-candidate body, and a non-JSON body each raise `Unreachable` naming the
    reason; a `game_id` of `../x` in a `200` body is dropped with one
    `escape_and_truncate` echo and no path.
11. Integration (CLI, subprocess): the gate refuses before enumerating — no
    listener behind `--catalog-url` gives exit non-zero, one named stderr line,
    and stdout carries neither the table header nor any fixture row, which is
    what proves "before enumerating".
12. Integration: the confirmation round-trip against a `TcpListener` fake in the
    test process — scan one answers the prompt with `1`, writes one record, and
    prints `-` in `Game Id`; scan two prints the id, makes zero requests (the
    fake asserts the count), and rewrites nothing; a run whose answer is `n`
    leaves no record; two fixture slugs confirmed to one `game_id` are two
    records; an ACF fixture `name` carrying an ESC byte renders with no raw
    control byte in the table.
13. Integration: `--json` mode — one parseable JSON document with exactly the
    key set in Chosen design (including `game_id: null` and a nested
    `candidates` list), no prompt text on stdout, and no store file written;
    the retired `"namespace"` key is asserted absent.
14. Integration: retirement — `derive_slug`, the `Namespace` type, the
    `namespace` field, and `resolve_collisions` resolve nowhere in
    `crates/discovery`'s or `apps/discovery-cli`'s `src/` or in any assertion
    of any suite; `tests/slug_tests.rs` and `tests/collision_tests.rs` are
    gone; `crates/discovery/tests/zz_api_probe.rs` resolves the new surface,
    and the fixture-tree expectations read `steam/<appid>`.
15. Platform: the full suite runs on the Linux CI runner only — Windows is the
    existing compile-check leg with no test binary, macOS has no leg — so no
    test asserts a platform-specific outcome; the Windows store-path rule is
    exercised on Linux through the env seam. No test performs a non-loopback
    request.
16. Doc-ledger and boundary pins: `crates/core/tests/ledger_annotations.rs`
    asserts `024-C1`/`024-C2`/`024-L1`/`024-L2` carry `amended-by 025-C1`,
    `025-C2`, `025-L3`, `025-L3` and that `024-R1` names ADR-025 and the
    superseded half; that the `schemas/README.md` sibling bullet states
    `MAX_DISCOVERY_SLUG_OCTETS = 128` with the not-a-grammar-bound clause; and
    the corpus sweep for `steam/`, `epic/`, `local/` over `data/` and the valid
    schema fixtures, beside the new
    `schemas/manifests/v1/fixtures/invalid/id-source-key.toml` that the existing
    conformance walk refuses.

## Rollout

Merged to `main` as one branch; the runner owns the commits. Items 1 and 4 are
one commit each; item 5 is its own commit so the ledger annotation is
attributable; items 2 and 3 are one commit each because each lands a whole
seam. No data migration: no shipped corpus carries an id to rewrite (A20), and
an absent match store is a valid initial state (the file appears on the first
confirmation). The unit handles below are the ADR's own Rollout handles.

1. Row identity lands in `crates/discovery`: `Source` and `lookup_key`, the
   `slug` spelling, the `appid` read, the bound-only plugin gate, the friendly
   name on the row, the deletion of `slug.rs`, `Namespace`, `resolve_collisions`
   and their suites, the re-pin of `plugin_tests`, `game_tests`,
   `steam_linux_tests`, `steam_windows_tests` and `emission_tests` to the new
   surface, the module-doc reversals, and
   `MAX_DISCOVERY_SLUG_OCTETS` in `crates/core` `placement.rs`/`lib.rs` with the
   `schemas/README.md` sibling bullet and the twelve-to-thirteen sentence; both
   API probes re-pin. `025-C1 025-C2 025-B1 025-L3 025-R2`
2. The match store lands in `apps/discovery-cli`: the path rule, the JSON
   shape, load-time validation, and save-on-confirmation. `025-C3 025-L4`
3. The catalog client lands in `apps/discovery-cli`: the endpoint policy,
   `probe`, the bound-only CLI boundary, the per-slug query, the confirmation
   prompt, JSON candidate passthrough, and the re-pinned CLI output contract
   (`Namespace` column out, `Game Id` in) — including the migration that keeps
   the gate from breaking the pre-existing subprocess suites: `cli_tests`,
   `cli_output_polish_tests`, `cli_dual_plugin_tests` and `cli_slug_refusal_tests`
   run behind a loopback `TcpListener` fake; the unproducible fixture route of
   `cli_slug_refusal_tests` is deleted, its bound refusal pinned in-module in
   `main.rs` instead. `025-C4 025-L1 025-L2`
4. The corpus boundary is pinned: `boundary_tests.rs`, the new invalid manifest
   fixture, and the `data/` + valid-fixture sweep; the `024` identity tests stay
   retired while the slug-grammar tests for product ids keep their meaning.
   `025-L5`
5. The ledger annotations land in `docs/adrs/024-game-slug-identity.md` per
   `021-L4`, gate-pinned by `crates/core/tests/ledger_annotations.rs`.
   `025-C1 025-C2 025-L3`
6. The deferral is recorded in `03-build-notes.md`: this run ships no catalog
   server, and `025-C4` stands as the client-side provisional spelling — including
   its loopback-only transport and its percent-encoded query value — until
   ADR-026 states the service contract, the transport, and the intake. `025-R1`
7. Gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
   -- -D warnings`, `cargo test --workspace --locked`, and `npm ci && npm run
   check` in `tools/orchestration`, per `.github/workflows/check.yml`. No
   `Cargo.lock` change is expected; if one appears, it is a finding.

## Deferred / out of scope

- Catalog hosting, the canonical REST surface, auth, and the curation intake by
  which source-id↔game-id facts enter the catalog — ADR-026 (`025-R1`).
- The TLS-capable, authenticated transport, and any non-loopback endpoint —
  ADR-026 (`025-R1`), with the service it authenticates to.
- The server-side reading of the `?slug=` query value (the client sends the
  percent-encoded form) and any candidate count or pagination contract —
  ADR-026 (`025-R1`).
- Batching candidate queries, and any concurrency in the client — ADR-026
  (`025-R1`), which owns the surface a batch would be.
- The Epic backend and whether its external-id needs `epic/<namespace>/<itemId>`
  (`025-R2`, owner `unassigned`).
- A `local` discovery backend emitting `local/<kebab>` and its
  over-bound-name refusal or fallback — no follow-up yet (the ADR names no
  rollout step for it).
- Unattended matching of a lone candidate, and any confidence signal from the
  catalog — ADR-026 (`025-R1`), per ADR-025's Alternatives.
- Garbage collection of store records whose slug a source stopped reporting,
  and locking or merge rules for concurrent scans — no follow-up yet.
- Search, browsing, or any UI over `friendly_name`; it is written and printed by
  the JSON row only — ADR-026, which owns the catalog surface a search needs.
- Persistence of discovery results beyond the match store — ADR-022, unchanged.
- Codec taxonomy and per-game codec selection — ADR-017.

## Revision log

- Initial: transport — std-only client restricted to a loopback authority, the
  split spelled out (`parse_endpoint` grammar, then a post-resolution loopback
  address check), TLS and auth deferred to `025-R1`; keeps the Windows
  compile-check leg clean (A17).
- Initial: gate placement — the CLI scan driver, not `crates/discovery`, so
  enumeration stays network-free and the engine keeps its error-swallowing shape.
- Initial: `game_id` — composed at render time, not a `DiscoveredGame` field, so
  plugin rows carry only what a source can know.
- Initial: match store — one JSON file in the XDG/`%APPDATA%` data dir, created
  owner-only with a byte read budget, written only on confirmation, no locking;
  refusal (not start-empty) on unparseable content or an over-budget file.
- Initial: Steam appid — `AppState.appid` is the sole source, refused when
  absent or non-numeric; no filename fallback, no name-derived fallback.
- Initial: friendly name — whitespace-collapse then clamp at the bound; clamping
  is admissible only because the value joins nothing.
- Initial: `local` and `epic` tokens are declared without emitters, and the
  kebab rule and the namespace-pair question are deferred rather than designed.
- r1: CLI test-harness migration — the pre-existing `apps/discovery-cli`
  subprocess suites run behind a loopback fake (Rollout 3), and the
  emit-boundary bound refusal moves to an in-module `main.rs` pin.
- r1: table cells — the ACF-sourced `Display Name`/`Install Path` render through
  `escape_and_truncate`, so the renderer meets the escape invariant.

## As-built (appended 2026-11-18; the design text above stands as approved)

This section records where the shipped tree (`7b03661…c9983a81`) departed from
the approved design, with the reason for each. It changes no design decision.
The stage-side record, gate posture, and operator follow-ups live in
`03-build-notes.md`; items 1–4 below restate that file's Deviations 1–5 where
the code confirms them.

1. **Retirement is feature-gated, not deleted** (affects "Row identity" —
   "`crates/discovery/src/slug.rs` is deleted" — and Rollout 1; Test strategy
   #14). Stage toolchains deny `rm`/`mv`, so `src/slug.rs` ships emptied to a
   retirement comment (its `pub mod slug;` declaration is gone from `lib.rs`,
   so nothing compiles it), and `tests/slug_tests.rs` and
   `tests/collision_tests.rs` stay in the tree but are gated off every build by
   a never-enabled `required-features = ["retired-adr-024-suites"]` in
   `crates/discovery/Cargo.toml`; a `tests/retired/README.md` records the
   retirement. The retirement gate was re-pinned to accept gated-but-present
   suites. Deletion of the three files and the gate is the operator's hygiene
   follow-up; behavior is otherwise as designed.
2. **Rollout 5 landed through the operator** (affects Rollout 5, "Corpus
   boundary and ledger"). ADRs are runner-owned and read/write-denied to every
   stage, so the five `amended-by`/`filled by` lines in
   `docs/adrs/024-game-slug-identity.md` could not land in a stage commit; the
   operator landed them and the runner committed them with the green-posture
   record (`c9983a81`). `ledger_annotations` runs 7/7 green against them, so
   the gate-pinned half of Rollout 5 is fully satisfied inside this diff.
3. **Refusal-line spellings beyond the design's two store names** (affects
   "Match store" and Scan-flow steps 2/6). `match store unreadable` echoes the
   path as `escape_and_truncate(parent) + escape_and_truncate(file_name)` plus
   a `: <reason>` tail — a whole-path escape would cut a long XDG path before
   the file name the operator must act on. A save failure — the one failure the
   design leaves unnamed — got a fourth named line,
   `error: match store unwritable: <path>: <reason>`.
4. **Table cells escape conditionally, and all five columns use it** (affects
   Scan-flow step 8). The renderer passes a cell through `escape_and_truncate`
   only when it carries a control character; clean values render untruncated
   (the suites pin long temp install paths), and the same `cell()` also covers
   `Slug`, `Game Id`, and `Platform`. The escape invariant — no hostile ACF
   byte emits a raw control sequence — holds as designed.
5. **CLI dev-dependencies** (affects A10/Rollout 7's no-lock-change claim,
   harmlessly): `apps/discovery-cli` gained `[dev-dependencies]` on
   `bindforge-core`, `discovery`, and `serde_json` because the committed suites
   import them; every edge already existed, and `Cargo.lock` is unchanged, so
   the dependency finding Rollout 7 watches for did not fire.
6. **Client seam shapes differ cosmetically** (affects "Catalog client and
   gate"; behavior and refusal names unchanged): the `CatalogClient` trait
   carries only `candidates()` while `probe()` is an inherent `HttpCatalog`
   method; `parse_endpoint` returns the normalized base as a `String` rather
   than an `Endpoint` struct; and `Candidate` parses manually from
   `serde_json::Value` instead of deriving `Deserialize` (unknown fields are
   still ignored, response order preserved).
7. **No-data-directory is a designed-in degradation the design did not state**
   (affects "Match store" and Scan-flow steps 2/6): when no absolute
   environment value yields a data directory, `store_path` returns `None`, the
   scan runs with an empty, non-writable store, non-writable stores earn no
   confirmation prompt, and `save()` is a no-op — a scan with no memory rather
   than a refusal.
8. **Additive public surface beyond the named files**: `Source::ALL`,
   `Source::from_token`, and `Source::spells_lookup_key` (the closed-set check
   the match-store load and `confirm` run); `Display` and `std::error::Error`
   impls for `DiscoveryError` in `crates/discovery/src/registry.rs`, a file the
   design never names (the CLI's plugin-failure line still quotes the `Debug`
   shape). Neither adds behavior the design rejects.
9. **Records and strata** (no behavior): `crates/discovery/tests/.rm-probe`, a
   0-byte toolchain probe, shipped by accident and awaits the same operator
   deletion pass as item 1; several code/test doc comments cite the unit
   handles `025-C5`/`025-C6`, which exist in no ledger (the shipped contract
   they mean is `025-C4` and `025-L2`); and the deferral record of Rollout 6
   was additionally written into the source ADR as a Rollout item 6, landed by
   the run's first (research) commit `8e005c31` alongside `03-build-notes.md`.
