# Design: Serve the catalog from a public root registry, with caching user-hosted registries

## Context

ADR-025 shipped the discovery catalog *client* and deferred the server (`025-R1`).
ADR-026 discharges that half: one `apps/registry` binary with two modes — a
**root** mode serving a canonical catalog corpus, and a **caching** mode holding a
user's own registrations plus a per-slug cache of upstream lookups — pinned to
the per-slug `GET /v1/candidates?slug=…` question the shipped client already
speaks (`025-C4`), plus a health route and a token-or-loopback-gated admin face
(`026-C1`/`026-C2`/`026-C4`/`026-C5`). It also amends the CLI's loopback-only
endpoint policy to admit any literal-`http://` host (`026-L1`). This run ships
the contract and the ledger annotations and **no product code**: the binary,
schema, and CLI amendment land as tasks `T-004`…`T-008` per ADR-023. This
proposal decides the open contract questions in the research so those tasks can
be written faithfully. Link: [`docs/adrs/026-registry-service.md`](../../adrs/026-registry-service.md).

## Goals / Non-goals

- Goals:
  - One byte-identical serving face across both modes (`026-C1`/`026-C2`):
    exactly the response shape and budgets the shipped client pins (A1).
  - A `026-C3` catalog document with one spec at `schemas/catalog/v1/`, whose
    conformance lives cleanly alongside the existing TOML contract tree.
  - A caching lookup policy with provenance and a `026-C7` cache document whose
    budgets close against each other (`026-C6`/`026-C7`/`026-B1`/`026-B2`).
  - The admin enablement/auth rule, mutations, and atomic persist stated to the
    level a test engineer can pin (`026-C4`/`026-C5`).
  - `026-L1`'s retirement named at all three shipped sites, gate kept.
- Non-goals (scope guards for code review):
  - No product code in this run — the registry binary, loader, and CLI
    amendment ship as `T-004`…`T-008` (`026` Rollout 1, 7).
  - No TLS or transport assurance (`026-R1`); no bulk catalog surface
    (`026-L3`); no curation intake beyond the admin face and no auth beyond the
    single bearer token (`026-R2`, ADR-027 reserved).
  - No upstream prefetch or parallel hydration (`026-R3`); no non-bearer or
    per-token identities; no corpus governance policy for the public root.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | The response the serving face must be byte-identical to accepts only status token `200`, rejects `Transfer-Encoding`, rejects a duplicated or non-decimal `Content-Length`, caps the header block *and* body at `MAX_RESPONSE_OCTETS = 65_536`, refuses more than `MAX_CANDIDATES = 256`, and `read_exact`s exactly `Content-Length` octets. | verified | apps/discovery-cli/src/catalog.rs (`read_response`, `parse_candidates`) |
| A2 | On a non-`200` the client surfaces only the status line (`the catalog answered \`<status line>\``); any reason phrase beyond the code is the only operator-visible text. | verified | apps/discovery-cli/src/catalog.rs (`read_response`) |
| A3 | `025-B1` is `MAX_DISCOVERY_SLUG_OCTETS = 128`, capping the `<source>/<external-id>` lookup key (not a slug-grammar bound). | verified | schemas/README.md ("ADR-025 adds one budget"); docs/adrs/025-discovery-identity-and-catalog-matching.md `025-B1` |
| A4 | The `game_id` grammar is `is_slug_name(id) && len ≤ MAX_SLUG_OCTETS` (128), applied at candidate parse, write, and load. | verified | apps/discovery-cli/src/matches.rs (`game_id_is_valid`); schemas/README.md (`024-B1`) |
| A5 | `name` is bounded by `MAX_NATIVE_NAME_OCTETS = 128` (`013-B4`). | verified | schemas/README.md; docs/adrs/013-game-manifests.md (`013-B4`) |
| A6 | The data-directory rule (`<data>`) is absolute `$XDG_DATA_HOME`, else absolute `$HOME/.local/share`; on Windows absolute `%APPDATA%`; a relative value is unset. `store_path` joins `bindforge/matches.json`. | verified | apps/discovery-cli/src/matches.rs (`data_dir`, `store_path`) |
| A7 | Atomic persist is a *fixed* tmp name opened `create_new(true)` at `0600`, parent `0700`, rename into place, re-`chmod`; a stale tmp entry is a hard failure, not a write-through. | verified | apps/discovery-cli/src/matches.rs (`save`, `owner_only_new_file`, `the_temp_name_is_opened_exclusively_only`) |
| A8 | Owner-only modes are `cfg(unix)`-only with a documented no-op on Windows. | verified | apps/discovery-cli/src/matches.rs (`owner_only`, `owner_only_dir`) |
| A9 | `crates/core/tests/conformance.rs` parses **every** file under every `schemas/*/v*/fixtures/{valid,invalid}` through core's TOML `parse`/`rejection`, and `crates/core/tests/common/mod.rs::is_fixture_name` requires a `.toml` suffix — so a `schemas/catalog/v1/` directory holding JSON fixtures would fail that TOML sweep unless the harness is told it is not a core TOML format. | verified | crates/core/tests/conformance.rs; crates/core/tests/common/mod.rs (`is_fixture_name`, `zero_fixture_violation`) |
| A10 | The `version_gate.rs` majors-equality invariant compares the **set of major-name strings**; adding `schemas/catalog/v1/` contributes `v1`, already inside `supported_majors() = {v1, v2}`, so that invariant stays green without a core change. | verified | crates/core/tests/version_gate.rs (`core_declares_exactly_the_major_versions_present_under_schemas`); crates/core/tests/common/mod.rs (`format_dirs`) |
| A11 | `026-L1`'s loopback-only admission is asserted in three named shipped places: `LOOPBACK_HOSTS` + the `parse_endpoint` host check + `probe()`'s non-loopback refusal; the unit test `the_policy_refuses_everything_beyond_the_loopback_spelling`; and the subprocess suite `endpoint_policy_refuses_everything_beyond_the_loopback_spelling`. | verified | apps/discovery-cli/src/catalog.rs; apps/discovery-cli/tests/cli_catalog_gate_tests.rs |
| A12 | The refusal line hard-codes "with a loopback http:// endpoint — ADR-026 owns the real transport"; it must be re-pinned to the admitted spelling. | verified | apps/discovery-cli/src/catalog.rs (`CatalogError::line`) |
| A13 | `[workspace] members = ["crates/*", "apps/*"]` already matches an `apps/` crate, so `apps/registry` needs no workspace edit; a glob matching nothing aborts every gate. | verified | Cargo.toml |
| A14 | No shipped manifest exists — `data/manifests/` holds only `README.md`, whose text says "The first shipped manifest lands as its own PR." | verified | data/manifests/README.md; tree glob of `data/**` |
| A15 | The only server in CI is the in-process loopback `TcpListener` fake (`FakeCatalog`, `respond_to`, `candidates_body`, `ok`, `closed_loopback_port`); the gate suite asserts "no test performs a non-loopback request". | verified | apps/discovery-cli/tests/common/mod.rs; apps/discovery-cli/tests/cli_catalog_gate_tests.rs (header) |
| A16 | The source token set is `steam` · `epic` · `local`, and `spells_lookup_key` validates the `<source>/<external-id>` spelling. | verified | crates/discovery/src/game.rs (`Source`, `spells_lookup_key`) |
| A17 | Amendments never renumber: the target unit gains `amended-by` per `021-L4`; `021-L2` makes the join mechanical; `021-L5` requires every reservation to name an owner or `unassigned`. | verified | docs/adrs/021-decision-unit-ledger.md (`021-L2`, `021-L4`, `021-L5`) |
| A18 | Task sources live at `docs/tasks/T-NNN-slug.md` with frontmatter `doc_type: task`, `status: pending`, `type`, `functional_area` and a non-empty body (`023-C2`/`023-C4`). | verified | docs/adrs/023-decoupling-intent-from-implementation.md; docs/tasks/T-003-polish-discovery-cli-output.md |
| A19 | The shipped client percent-encodes everything outside the RFC 3986 unreserved set, so `steam/900001` travels as `steam%2F900001` and control bytes as `%XX`. | verified | apps/discovery-cli/src/catalog.rs (`percent_encode`, `candidates_url`) |
| A20 | RFC 6750 requires TLS for bearer tokens; `026-R1` defers TLS, so the admin bearer over plain HTTP deviates from the scheme it names. | unverified | https://www.rfc-editor.org/rfc/rfc6750.html (via `01-research.md` Platform notes) |
| A21 | RFC 9110 makes `PUT` idempotent, requires `405` to carry `Allow`, treats request-syntax faults as `400` and semantically-bad-but-well-formed content as `422`, and defines `503` as temporary (permitting `Retry-After`). | unverified | https://www.rfc-editor.org/rfc/rfc9110.html (via `01-research.md` Platform notes) |
| A22 | The shipped load-discipline precedent: `matches.rs::load` checks `fs::metadata` against `STORE_READ_BUDGET_OCTETS` and refuses an over-budget store *before* reading, and the two shipped JSON documents (`matches.json`, the candidate body) live outside the TOML formats contract (which is a `.toml`-fixture sweep, not a stated placement rule). | verified | apps/discovery-cli/src/matches.rs (`load`, `STORE_READ_BUDGET_OCTETS`); crates/core/tests/conformance.rs |
| A23 | The shipped client sends one `GET` per connection carrying `Connection: close`, then reads a single response: nothing in the shipped pair uses keep-alive. | verified | apps/discovery-cli/src/catalog.rs (request built with `Connection: close`) |

Body claims below that depend on A20/A21 are worded as design choices made for
standards-conformance; the binding obligations are the ADR's own unit texts, so
those choices are reversible without contract impact.

## Options considered

**Catalog document as a core TOML format** (a `catalog` arm in `bindforge-core`,
`format = "catalog"` / `version = "v1"` strings, `.toml` fixtures, `.toml`
on-disk seed and cache). Rejected: the ADR fixes the on-disk spellings as JSON —
`data/catalog/v1/catalog.json` (`026-S1`), `<data>/bindforge/registry/cache-v1.json`
(`026-C7`), and `026-C3`'s `version: 1` bare-integer form, which no shipped TOML
format shares — and the two shipped JSON documents (`matches.json`, the candidate
body) already live *outside* the TOML formats contract (A5, A6, A22). Forcing
TOML contradicts the ADR's fixed names for no contract gain.

**Catalog document as a JSON format, carved out of the TOML conformance walk**
(chosen). `schemas/catalog/v1/` ships a `spec.md` and `.json` fixtures, but
`crates/core/tests/conformance.rs` must not sweep it through the TOML parse
path (A9). Rejected the cheaper "just don't put it in `schemas/`": `026-C3`
mandates that path anyway, and the rejected location dodges only the
conformance walk — the seed at `data/catalog/v1/` still trips the
`data/`-walking source-key sweep (both amended in §Catalog document and its
spec) — so the harness carve-out is the honest cost, not the format choice.

**Budget reconciliation by refusing the whole cache over `026-B2`** (only).
Rejected alone: the match-store precedent refuses the *file* (A7), which for a
cache would turn a user's whole lookup memory into one error line, and it does
not stop a legal-but-large corpus (5,000 entries × up to `MAX_CANDIDATES`
candidates) from exceeding 1 MiB between writes.

**Concurrency by treating disk as source-of-truth per request.** Rejected: a
request that reads the corpus file while an admin rename is in flight can see a
torn view; the in-memory model guarded by a lock, with the file kept whole by
the tmp-and-rename discipline, is the smaller surface that cannot tear a
reader (A7).

## Chosen design

### `apps/registry` — one crate, two modes (traces `026-C1`)

A new binary crate `apps/registry` (workspace member via `apps/*`, A13).
`std`-only for its serving face: `std::net::TcpListener` accept loop,
hand-written HTTP/1.1 framing, `serde_json` for bodies (already a shipped-client
dependency, so "std-only" means no HTTP/TLS runtime, not "no deps"). It depends
on `bindforge-core` for the grammar and name bounds (`is_slug_name`,
`MAX_SLUG_OCTETS`, `MAX_NATIVE_NAME_OCTETS`, `escape_and_truncate`, A4, A5) and
on `discovery` for `Source` and `spells_lookup_key` alone — the same closed
source-token set `discovery-cli` reaches (A16), never a re-spelled one, with the
transitive Windows-only `winreg` edge taken for that predicate and retired by
the deferred lift below. The shipped client is bin-only, so six invariants are
**copied** into this crate (not shared), each re-pinned by the §Test strategy
item named for it: the response reader and the shipped request builder
`percent_encode`/`candidates_url` (#7), the endpoint gate (#14, #15), the
`game_id_is_valid` predicate (#8), the `<data>` rule (#13), the tmp-and-rename
persist (#10, #11); their lift to a shared crate is deferred (§Deferred).

Modules (concrete file plan):

- `main.rs` — argument parse, `serve` / `query` dispatch, ephemeral-port print.
- `serve.rs` — accept loop (one thread per admitted connection, up to
  `MAX_CONCURRENT_CONNECTIONS`, over-cap closed at accept; request-read *and*
  write timeouts to bound low-and-slow), router, mode selection.
- `http.rs` — response writer emitting exactly one decimal `Content-Length`,
  never `Transfer-Encoding` (A1), then closing the connection — one answer per
  connection, echoing the shipped client's own `Connection: close` (A23), so no
  idle client holds an admitted slot; the request reader admits
  `Content-Length`-framed bodies within the budgets below — a
  `Transfer-Encoding`, missing or non-decimal length, or an over-cap body is
  refused before any mutation.
- `corpus.rs` — `026-C3` catalog-document load + validation + the
  slug→candidates index.
- `admin.rs` — `026-C4` upsert/delete/stats + `026-C5` enablement/auth.
- `cache.rs` — `026-C7` cache document, `026-B1`/`026-B2` enforcement, eviction.
- `upstream.rs` — `026-C6` upstream client.
- `endpoint.rs` — the `--upstream` URL gate: the copied `parse_endpoint` rules
  (A11; copy list above).
- `constants.rs` — the private budgets (see §Budgets).

CLI surface:

- `registry serve --root --corpus <path>` → root mode over a `026-C3` corpus.
- `registry serve --upstream <url>` → caching mode; the local-entries document
  defaults to `<data>/bindforge/registry/catalog-v1.json` (a `026-C3` instance,
  created empty if absent, mutated by the admin face), relocated in tests
  through the same env seam as #13.
- `--host` (default `127.0.0.1`), `--port` (default `0`; bind-to-port-0 and
  print the assigned port — the *printed* value is the contract).
- `--admin-token` or `BINDFORGE_REGISTRY_ADMIN_TOKEN`.
- `--refresh-interval <seconds>` — a bare unsigned count of seconds — overrides
  `026-B3` (default `REGISTRY_UPSTREAM_TTL_SECONDS = 3_600`).
- `registry query <slug>` runs the mode's policy once and prints the answer with
  its provenance; in caching mode this is the real policy and **does** write the
  cache, and it exits non-zero on the `026-L4` cold case so scripts see failure
  (`026-C6`).

The two modes share every handler; mode is an enum field on the server state,
so `GET /v1/candidates` is byte-identical between them (`026-C1`) — health's
`mode`/`upstream` fields and the caching-only cold `503` are the ADR's own
mode-revealing exceptions.

### Catalog document and its spec (traces `026-C3`, resolves open Q1/Q2)

`026-C3` shape, on disk as JSON: `{ "version": 1, "revision": <non-negative>,
"games": [ { "game_id", "name", "source_keys": [<lookup keys>] } ] }`. The
registry's loader (`corpus.rs`) refuses by name: `version != 1`, `revision`
negative, a `game_id` failing the grammar+bound (A4), a repeated `game_id`
(records are keyed by it, so a duplicate loads ambiguously), a `name` over 128
octets (A5), a `source_key` failing §The serving face step 1's rule (A16, A3), a
`source_key` repeated within one record (the slug→candidates index would post
the same game twice, crowding the `MAX_CANDIDATES` window), a non-object/missing
field, and a document over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets) — on the
write side, §The admin face and auth applies that cap to every upsert. Nothing
on disk is rewritten on refusal (A7's load discipline).

`schemas/catalog/v1/` ships `spec.md` + `fixtures/valid/*.json` +
`fixtures/invalid/*.json` and is declared a **JSON format**, a documented
exception to "Formats are TOML". The load-bearing harness consequence (A9) is
resolved by naming the files that change:

- `crates/core/tests/common/mod.rs`: one `const` naming `catalog` as the JSON
  format, and `is_fixture_name` gains the directory (or a per-directory suffix
  set) so it accepts `.json` there — the one call site is in the fixture-name
  test below; the zero-fixture rule still requires catalog to ship ≥1 valid +
  ≥1 invalid fixture (it counts files, not parses).
- `crates/core/tests/conformance.rs`: guard the parse/rejection loops —
  `every_valid_fixture_…` and both `every_invalid_fixture_…` message tests —
  behind the core-TOML check so catalog fixtures are not parsed as TOML.
  `fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique` is
  **not** guarded: through the directory-aware `is_fixture_name` it keeps its
  lowercase, `-`-separated and case-uniqueness rule over catalog's `.json`
  fixtures, which is the only coverage behind #19's case-unique-fixture-name
  claim for a case-insensitive filesystem.
- `crates/discovery/tests/boundary_tests.rs`:
  `the_shipped_corpus_carries_no_source_key_spelling` fails on a `steam/`,
  `epic/`, or `local/` literal in any file under `data/` or any
  `schemas/*/v*/fixtures/valid/` — which is exactly what `026-S1` mandates —
  so keep both roots and skip only files under `data/catalog/` and
  `schemas/catalog/`, with the doc-comment naming `catalog` as the one tree
  where a lookup key is legal content; the `checked > 0` guard is unchanged.
- `version_gate.rs`: **no change** — the majors-equality invariant already holds
  (A10); the per-format gate cases are format-specific and never name `catalog`.
- `schemas/README.md`: amend the "Formats are TOML" paragraph to record that
  `catalog` is a JSON format whose conformance is exercised by its own loader in
  `apps/registry`, and that the JSON contract documents (`matches.json`, the
  candidate body, the cache) join it outside the TOML subset.

`version` stays the bare integer `1` (the ADR's spelling); the version-refusal
lives in `corpus.rs`, named, not in core's `UnsupportedVersion` path.

### The seed corpus (traces `026-S1`, resolves open Q3)

`data/catalog/v1/catalog.json` is a `026-C3` corpus whose `game_id`s are
grammar-valid kebab strings (e.g. `example-game`) each carrying its
`steam/<appid>` `source_key`. The research shows `data/manifests/` is empty
(A14), so "ids drawn from the shipped manifest ids" has nothing to draw from
today. This is safe because the registry validates `game_id` **only** against the
slug grammar (A4) and never joins it against `data/manifests/` — `013-L5`'s
name-agreement rule is a manifest-tree rule, not a registry rule. The seed
therefore mints provisional grammar-valid ids — `026-S1` is partially satisfied,
and step 6 of §Rollout records that on the unit — and adopting real
manifest ids is deferred to the first manifest PR (§Deferred).

### The serving face (traces `026-C2`, `026-L3`, `026-L6`; resolves Q6/Q7/Q13)

`GET /v1/candidates?slug=<percent-encoded>`:

1. Percent-decode `slug` **first**, then apply `025-B1`'s 128-octet cap and
   `spells_lookup_key` to the *decoded* bytes (A3, A16, A19): that shipped pair
   is the registry's whole key rule, so a dotted external id — `local/Steam.SE`,
   the ADR's `local/<dir>` case — is admitted (`is_slug_name`'s dot ban is
   `game_id`-only, A4). Path safety is the only extra refusal, at its own scope:
   after the first `/`, one non-empty component with no further `/` or `\`, none
   equal to `.` or `..`, and no control byte or invalid-UTF-8 sequence,
   so `%2e%2e%2f` and `%0a` never reach a corpus, cache document, or refusal line
   (keys are opaque in-memory strings, never joined onto paths). The rule
   governs candidate queries, admin upserts, and cache loads alike; an over-cap or
   failing decoded slug is the `404` `026-C2` names.
2. Answer = the join, served from the `corpus.rs` slug→candidates index: at
   load, postings are built already ordered by `game_id` byte order, so a
   request is one probe plus a clone of at most `MAX_CANDIDATES` records, each
   projected to `{game_id, name}` **only** — `source_keys`, friendly names, and
   install paths never leave (A4; `026-C2`'s projection; `026-L6` permits only
   the queried key outbound). Truncation is stable across requests (A1). An
   empty join in root mode is a `200` with `[]`; the `026-L4` `503` is
   reachable only from caching mode.
3. Response written with the byte shape A1 pins.

`GET /v1/health`: always `200` with `{status, mode, upstream, local_entries,
cached_slugs, last_upstream_fetch}`; `upstream` is `none` (root) or
`ok`/`stale`/`down` (caching); a caching instance that has not yet contacted
its upstream reports `down` with `last_upstream_fetch: 0` — the named value of
the never-fetched state. It is a **status document, not a readiness gate**:
it never answers non-`200`, so a `down` upstream is reported in the body, not
in the code. Answering it reads the **last-known** upstream verdict (see
§Caching mode) and never fires that probe. It is never gated (`026-L3` permits
the summary counts).

Any other route, or a malformed slug, is `404` (`026-C2`). The `026-L4` cold
case is the named `503` with a legible reason phrase (e.g. `HTTP/1.1 503 No
catalog answer for slug`), because the CLI shows the user nothing but the status
line (A2). `405` carries an `Allow` header naming the route's methods for
standards-conformance (A21, unverified); the binding obligation is `026-C4`'s
"A `405` names any method the route does not carry", so a test pins the method
list, not the header spelling.

No route answers a whole document in either mode: wire traffic is per-slug
candidate lists within `025` budgets, health/stats summaries and single-record
mutations, and the operator who wants the whole corpus reads their own file
(`026-L3`, #18).

### The admin face and auth (traces `026-C4`, `026-C5`; resolves Q10/Q11)

Present in both modes, mutating that instance's own catalog document (root: the
`--corpus` file; caching: the local-entries document):

- `POST /v1/admin/games` upserts one whole `{game_id, name, source_keys}` record,
  validated against `026-C3` and refused by name; the model with the upsert and
  the next `revision` value is serialized whole — the exact buffer renamed — and
  one over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets) is the named `422` for
  well-formed-but-unusable content (A21): model, `revision` and file untouched,
  no tmp written; only then does that buffer go to the tmp and rename (A7).
- `DELETE /v1/admin/games/<game_id>` percent-decodes the path segment, validates
  the grammar (A4), removes the record, bumps `revision`, and persists through
  the same tmp-and-rename (A7).
- `GET /v1/admin/stats` answers `{games, source_keys, revision}` — all three
  integer counts, never key lists (`026-L3`) — from the instance's own catalog
  document (not the cache).

Enablement and auth (`026-C5`):

- Enabled when the listener binds loopback-only **or** a token is configured.
- A token, once configured, is **required on every admin request regardless of
  bind** — the ADR's loopback-vs-token ambiguity (open Q11) resolves to
  "configured ⇒ required everywhere"; a non-loopback bind with **no** token
  leaves admin **disabled**.
- Enablement is judged from the address actually bound — `IpAddr::is_loopback` on
  the listener's `local_addr()`, the test `probe()` applies to each resolved
  address (`catalog.rs:248`) rather than a literal-spelling comparison, and the
  registry's own judgment, not one of the copied blocks — together with the
  presence of a configured token. The compared value is the token after `Bearer `
  (the scheme token, not the whole header), compared under a constant-time fold
  that does not early-return on a length mismatch. The accepted `Authorization`
  header is bounded by the request header budget below.
- Disabled admin routes answer `404`; enabled ones answer `401` for a missing or
  wrong token, carrying `WWW-Authenticate: Bearer` for conformance (A20,
  unverified). The configured token never appears in a refusal line, log, or
  status surface.

**Concurrency (open Q10):** one process, single accept loop. An
`Arc<RwLock<Corpus>>` guards the in-memory catalog document and
an `Arc<RwLock<Cache>>` the cache; serving reads take the **shared** lock on
the in-memory model (never the file), so a reader can never observe a
half-written document; a mutation serializes the new model, writes the fixed tmp
name, renames it into place, and swaps the model in, **all under the one write
lock**, so persist order can never diverge from mutation order and a restart
cannot lose an accepted mutation (#11). A reader may wait on one local write, and
that latency is accepted. **Running two `registry serve` instances against one
corpus is a refused, unsupported state**: the fixed-name `create_new(true)` tmp
makes the second writer fail loudly (A7), the documented answer to what the ADR
left unstated.

### Caching mode (traces `026-C6`, `026-L4`, `026-L5`; resolves Q8/Q9/Q14)

Per queried slug, in priority order:

1. **Local entry** (a game in the local-entries document whose `source_keys`
   contain the slug) answers with **zero** upstream requests, is never merged
   with upstream candidates, and never expires (`026-L5`).
2. **Cached entry within TTL** answers as-is.
3. **Stale cache + failed refresh** answers the stale entry and moves
   `health.upstream` to `stale` (serve-stale-on-error, the standard posture).
4. **Miss with reachable upstream** fetches `GET <upstream>/v1/candidates?slug=`,
   target built from the *decoded* slug via the copied `percent_encode` (A19),
   validates the answer, caches it with `fetched_at`.
5. **Cold miss with no reachable upstream** is the named `503` of `026-L4` —
   never `[]`.

The **upstream client** (`upstream.rs`) applies the copied response discipline
(§`apps/registry`) with its own private constants: reject `Transfer-Encoding`,
duplicated or non-decimal `Content-Length`, body over `MAX_RESPONSE_OCTETS`,
more than `MAX_CANDIDATES` candidates, and a non-`200`; connect/read timeouts
mirror `2 s`/`5 s` (A1). It **refuses redirects** (a `3xx` is an upstream error
→ `stale`/`503`), because following a redirect away from the configured
endpoint would break `026-L2`'s disclosure model. `--upstream` admits only a
literal `http://` URL with a parseable host+port (any host) per `endpoint.rs`'
copied rules (A11); `https://` upstream is **not** admitted in v1 (`026-C1`
std-only; the transport hop's assurance is `026-R1`, open Q14).

**Upstream reachability** (the `health.upstream` gauge) is gauged by the
upstream's own `/v1/health`, probed **at most once per TTL**, fired lazily by
the first *lookup* after expiry on that request's own thread — there is no timer
thread, and a health read only reports the verdict as it stands; it never fires
the probe. Lookups themselves reveal reachability too (a connect failure on a
cold fetch ⇒ `down`), and the lazy once-per-TTL probe is how the design keeps
`026-C6`'s "gauged by `/v1/health`" without a second round trip per cold lookup.

**Upstream ingest is validated before it is cached** (open Q9): each upstream
`{game_id, name}` runs through `game_id_is_valid` (A4) and `name ≤
MAX_NATIVE_NAME_OCTETS` (A5) before entering `cache-v1.json`; a failed id is
dropped with one escaped, single-line echo. This is the cache being untrusted
input at its *write* seam as well as its load seam (A7, `026-C7`). It narrows the
payload, not the identity: an on-path attacker on the plain-HTTP hop can still
choose which legitimate id a slug resolves to (§Risks); closing that is `026-R1`.

### Budgets (traces `026-B1`, `026-B2`, `026-B3`; resolves Q4/Q5)

The three registry budgets are **private to `apps/registry`** (`constants.rs`),
following the precedent `matches.rs` already set with its private
`STORE_READ_BUDGET_OCTETS` (A22). They are **not** added to core's
thirteen-constant family and get **no** `schemas/README.md` bullet, so that count is
unchanged (open Q4); `schemas/catalog/v1/spec.md` states the catalog format's own
bound, the named refusals that enforce it on both sides of a document's life, and
the per-record arithmetic behind `MAX_CATALOG_DOCUMENT_OCTETS` below.

The research's "budgets do not close" finding (5,000 entries × up to 256
candidates can exceed 1 MiB) is resolved by making both caps **write-time**
invariants (open Q5), with no bookkeeping of its own: after a write the registry
serializes the cache and re-serializes after each dropped entry — one extra pass
per eviction at most — dropping the oldest-`fetched_at` entries, `026-B1`'s own
key, until the buffer is ≤ `026-B2` octets and the entry count ≤ `026-B1`, then
renames that buffer into place, so the octet test is the document's exact length
and not a sum of per-entry totals. A realistic entry is ~150 octets, so
~7,000 slugs fit inside 1 MiB: `026-B1`'s 5,000 is the cap that binds ordinary
answers, `026-B2` only a corpus answering at `MAX_CANDIDATES`. The file is
rewritten write-through after each lookup that adds an entry; a crash loses that
entry only. `026-B2` is **re-checked at load**, which also refuses a repeated
`slug` and any `slug` failing §The serving face step 1's rule by name (A7).
`fetched_at` is untrusted input, so the load refuses one outside
`[now − 31_536_000 s, now + 86_400 s]`, and TTL arithmetic uses the wall clock
(`SystemTime`) with a guard that treats a backwards-jumped stamp as expired
rather than immortal — that guard is where a clock jump is handled, not an
eviction-order tie-break.

Because the ADR names no request-side budgets, the serving/admin face carries
private request budgets in the same `constants.rs`: `MAX_REQUEST_HEADER_OCTETS`
(65,536, mirroring the client's own header cap, A1); `MAX_REQUEST_BODY_OCTETS`
(= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally
hold also fits an upsert and no per-record `source_keys` bound is needed); and
`MAX_CONCURRENT_CONNECTIONS` (64 — one thread per admitted connection, which
answers exactly one request before `http.rs` closes the connection, so ≤64 live
threads and ≤64 in-flight
upstream hops at the mirrored `2 s`/`5 s` timeouts; over-cap is closed at accept
with no status line, which the shipped CLI reports as its named unreachable).
These bound inbound input as the client bounds outbound (`026-C2`/`026-C4`
elaboration, not new ADR units). The corpus is budgeted too:
`MAX_CATALOG_DOCUMENT_OCTETS` (16,777,216 — ≈120,000 records at the ~140-octet
`{game_id, name, source_keys[]}` size `026`'s "tens of thousands of games, each
with several store keys" implies) caps a document at **both** ends of its life:
`fs::metadata`-gated before the read of the `--corpus` or local-entries file and
refused by name (A22), and checked against the exact post-`revision`-bump buffer
an admin write renames into place (§The admin face and auth, #10), so no
accepted mutation can persist a corpus its own loader refuses.

### CLI amendment (traces `026-L1`, `026-L2`; resolves the divergence window)

In `apps/discovery-cli/src/catalog.rs`:

- `parse_endpoint` keeps the literal-`http://` requirement and the digits-only
  port but **drops the `LOOPBACK_HOSTS` host check and the constant with it** —
  any parseable host+port is admitted (A11).
- `probe()` drops the `is_loopback()` non-loopback-resolution refusal and keeps
  everything else: resolve, refuse an empty address set, prove one
  `TcpStream::connect_timeout` **before any plugin is constructed** — the
  fail-closed gate survives with its loopback half amended (`026-L1`).
- The refusal line drops "loopback" and re-pins to the admitted spelling — a
  configured `http://` endpoint, any host — naming `--catalog-url` /
  `BINDFORGE_CATALOG_URL` (A12).
- The suites split: remote-host admission is pinned in the **unit** suite at
  `parse_endpoint` alone, and the `026-L2` disclosure statement moves there;
  the subprocess suite keeps only the still-refused spellings (`https`/`ftp`/
  `@`/empty/non-numeric port) with the re-pinned refusal text, so it never
  resolves a remote name; `probe()`'s admission is pinned only against the
  in-process loopback fake (A11, A15).

The `026-L2` consent model is unchanged by the amendment: a scan sends every
queried source key to the configured endpoint; the choice of endpoint *is* the
consent; a caching registry's upstream traffic carries only the lookups its
clients asked for, never client identity and never the bulk catalog.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Plain-HTTP upstream hop lets an on-path attacker pick which legitimate `game_id` a slug caches (`026-R1` deferred). | Medium | High | Validate shape/grammar before caching (limits payload, not identity); report `mode`/`upstream` in health; TLS assurance is `026-R1`, not this run. |
| Bearer admin token is link-visible on a public root (`026-R1` deferred; A20). | Medium | High | Loopback admin needs no token; non-loopback admin requires a token and is the operator's edge-TLS deployment (`026-R1`); record the RFC 6750 deviation. |
| A cold caching registry's first scan is N serial upstream lookups with a second health round trip (`026-R3`). | High | Medium | Cache the upstream-health verdict and reuse lookup outcome; prefetch stays `026-R3`. |
| Two `registry serve` on one corpus fail loudly (by design, A7) — an operator may not expect it. | Low | Medium | State it as a refused, unsupported configuration in `spec.md`; the loud write-time tmp failure is the whole mechanism — there is no startup check or instance marker. |
| CLI behavior change (remote hosts admitted) is deferred to `T-008`, leaving a window where `026-L1` says "admit" but code refuses. | High | Medium | Accepted and documented: the ADR ships no product code (Rollout 7); see §Rollout. |
| Harness carve-out drifts — a future contributor adds `schemas/foo/v1/` JSON fixtures and re-trips the TOML walk or the source-key sweep (A9). | Medium | Low | Single `const` names `catalog` at the skip sites; the `boundary_tests` sweep keeps both roots and skips only `data/catalog/` + `schemas/catalog/` (#1's harness list), so every other tree stays under the hygiene gate. |
| Low-and-slow / one-inbound-per-outbound exposure on a routable bind. | Medium | Medium | Thread-per-connection capped by `MAX_CONCURRENT_CONNECTIONS` (64, over-cap closed at accept); request-read **and write** timeouts; `MAX_REQUEST_HEADER_OCTETS`/`MAX_REQUEST_BODY_OCTETS`. |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux | Yes | Primary CI target. `<data>` = `$XDG_DATA_HOME` else `$HOME/.local/share`; owner-only `0700`/`0600` modes apply (A8); `std::net` serving face. |
| macOS | Yes | Same `<data>` rule and unix modes as Linux (A6, A8); exercised only via the env-closure unit seam, not a macOS CI runner. |
| Windows | Yes | `<data>` = `%APPDATA%` (relative = unset); owner-only modes are a documented no-op (A8); cache and local-entries paths `<data>\bindforge\registry\{cache-v1,catalog-v1}.json`. Not run in CI — unit-tested through the `StoreLayout` env-closure seam. |
| Other / exotic Unix (BSD et al.) | No | Not a v1 target: the Unix data-dir branch and `std::net` may carry it; nothing exercises or promises it. |

No product code ships this run, so the matrix describes the target `apps/registry`
behavior the tasks must meet, not shipped artifacts; the three `Yes` rows (Linux
CI, macOS, Windows) are the rows the tasks owe behavior to, and no other Unix is.

## Test strategy

Unit (in `apps/registry` unless noted):

1. **Catalog-document loader (`026-C3`):** every `fixtures/valid/*.json` loads;
   each of the following — wrong `version`, negative `revision`, grammar-invalid
   `game_id` (A4), repeated `game_id`, `name` over 128 octets (A5), a `source_key`
   failing §The serving face step 1's rule (A3, A16), a `source_key` repeated within one
   record, malformed record, document over `MAX_CATALOG_DOCUMENT_OCTETS` —
   ships in `fixtures/invalid/*.json` as a named refusal; nothing on disk is
   rewritten.
2. **Corpus join:** `candidates(slug)` probes the load-built index and returns
   the games whose `source_keys` contain the decoded slug, postings already
   ordered by `game_id`, projected to `{game_id, name}` only, truncated to
   `MAX_CANDIDATES` (A1); a root-mode empty join answers `200` with `[]`.
3. **Decode-then-validate:** `025-B1`'s cap and §The serving face step 1's rule
   apply to **decoded** `slug` and decoded `DELETE` path segment; over-cap,
   unspellable, separator- or dot-dot-bearing (`local/%2e%2e%2f…`), or
   control-byte-bearing (`steam/%0a730`) ⇒ `404`; a dotted external id
   (`local/Steam.SE`) is admitted; encoded length stays within the request
   budget.
4. **Cache document (`026-C7`):** valid load sorted by `slug` byte order; refusals
   for bad version/shape/`game_id`/`name`, repeated `slug`, and a `slug` failing
   §The serving face step 1's rule; absent file ⇒ empty cache; a `fetched_at`
   outside the load bound is refused.
5. **Write-time dual eviction (`026-B1`+`026-B2`):** after writes, the cache
   holds `count ≤ 5_000` **and** a written document `≤ 1_048_576` octets, oldest
   `fetched_at` evicted first; load over `026-B2` is refused by name.
6. **Lookup policy (`026-C6`/`026-L4`/`026-L5`):** local entry answers with zero
   upstream requests; fresh cache answers; stale-cache + failed-refresh answers
   stale and sets `upstream: stale`; cold miss + no upstream ⇒ the named `503`
   (never `[]`), against an injected upstream fake — with the fresh/stale pair
   run under `--refresh-interval 1`, so the override is what turns a fresh entry
   stale on the next query and re-times the once-per-TTL upstream probe.
7. **Upstream client discipline:** rejects `Transfer-Encoding`, duplicated/
   non-decimal `Content-Length`, over-budget body, over-count candidates,
   non-`200`; refuses a `3xx` as an upstream error (A1); a space- or %-bearing
   `slug` reaches upstream only percent-encoded from the decoded key (A19).
8. **Upstream ingest validation:** a malformed upstream `game_id` or `name` is
   dropped before the cache write, with one escaped single-line echo (A4, A5).
9. **Admin auth (`026-C5`):** loopback+no-token enables without a credential; a
   configured token is required on every admin request including loopback;
   non-loopback+no-token disables (admin ⇒ `404`); constant-time token compare;
   wrong/missing ⇒ `401` with `WWW-Authenticate` and no echo of the configured
   token; `405` carries `Allow`.
10. **Admin mutations (`026-C4`):** upsert validates + bumps `revision` + atomic
    rename, and an upsert whose serialized corpus with its bumped `revision`
    would exceed `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422` with model,
    `revision` and file untouched and no tmp written; delete decodes + validates +
    bumps `revision` + atomic rename; `405` on a method a route lacks; `stats` is the
    three-field integer shape `{games, source_keys, revision}` from the instance
    document.
11. **Atomic persist + concurrency:** a second save over a stale tmp fails loudly
    (A7); a reader never sees a torn document (served from the locked in-memory
    model); N concurrent upserts leave the persisted document equal to the model with
    `revision` at the final value; `MAX_REQUEST_HEADER_OCTETS`/
    `MAX_REQUEST_BODY_OCTETS` refusals — a `Transfer-Encoding`,
    `Content-Length`-less, or over-cap body is refused before any mutation;
    over-`MAX_CONCURRENT_CONNECTIONS` (64) is closed at accept with no status
    line.
12. **Health (`026-C2`):** always `200`; reports `mode`/`upstream`/counts; the
    `upstream` verdict is read from the last-known cached value and a health read
    never fires the TTL probe; a freshly started caching instance answers
    `upstream: down`, `last_upstream_fetch: 0`.
13. **Data-dir / cache path (`026-C7`, A6):** both Unix and Windows branches
    through the `StoreLayout` env-closure seam (XDG/HOME vs `%APPDATA%`),
    relative = unset; this seam is how caching tests relocate the local-entries
    document — the subprocess cases (#17) set `XDG_DATA_HOME`/`APPDATA` in the
    child environment, since there is no path knob.
14. **`--upstream` endpoint gate:** literal `http://` + parseable host/port
    admitted (any host); `https`/`ftp`/`@`/empty/non-numeric refused (A11).
15. **CLI amendment (`apps/discovery-cli`, `026-L1`):** remote-host admission is
    pinned at `parse_endpoint` in the unit suite only; `probe()`'s admission
    is pinned against the loopback fake and still refuses before plugin
    construction; the subprocess suite re-pins only the refused spellings
    (`https`/`ftp`/`@`/empty/non-numeric port) and the new refusal line; **no
    test dials or resolves a non-loopback host** — the loopback fake stays
    the only server (A11, A15).

Integration (subprocess / over a loopback socket):

16. **Root end-to-end:** `registry serve --root --corpus <seed>` bound to an
    ephemeral `127.0.0.1` port; `discovery list --catalog-url http://127.0.0.1:<port>`
    resolves a `steam/<appid>` to a curated id, and an absent slug answers `200`
    with `[]`; response shape pinned byte-exact
    to A1 (single decimal `Content-Length`, no `Transfer-Encoding`, caps);
    `GET /v1/health` shape.
17. **Caching end-to-end:** a root and a caching registry chained via `--upstream`
    (both on `127.0.0.1` ephemeral ports); a cold slug fetches-then-caches; a
    warm slug makes zero upstream requests; an upstream-down cold slug surfaces
    the named `503` to the CLI as `catalog unreachable` and fails closed; `query`
    prints provenance.
18. **No-bulk (`026-L3`):** no route answers a whole document; `/v1/admin/catalog`
    is `404`; `stats` is the only corpus-size surface.

Platform:

19. **Linux** is exercised directly by the cargo gates; **Windows** and **macOS**
    behavior is proven by the env-closure seam (#13) and case-unique fixture
    names, not by a runner on those OSes; owner-only file/dir modes are asserted
    only where `cfg(unix)` (#11), with the Windows no-op path compiled.

## Rollout

This run ships the contract and the ledger annotations only; the service lands as
tasks. Steps from merged branch to shipped feature:

1. Merge the contract artifacts this proposal fixes: `schemas/catalog/v1/`
   (spec + JSON fixtures) with the harness carve-out files named in §Chosen
   design, and `data/catalog/v1/catalog.json` (minted grammar-valid seed ids,
   A14). This is task `T-004` (ADR Rollout item 1); step 6's ledger pass
   annotates `026-S1` accordingly.
2. `T-005` lands the root serving face in `apps/registry` (`026-C1`/`026-C2`,
   `026-L3`/`026-L6`): `serve --root`, the join, health, and the named `404`s.
3. `T-006` lands the admin face (`026-C4`/`026-C5`): enablement/auth, mutations
   with validating refusals, atomic persist, stats, and the concurrency posture
   (two instances on one corpus refused).
4. `T-007` lands caching mode (`026-C6`/`026-C7`, `026-L4`/`026-L5`,
   `026-B1`/`026-B2`/`026-B3`): lookup policy with provenance, the cache document
   with dual write-time eviction and load re-check, the TTL, the cold `503`, the
   `query` subcommand; prefetch stays `026-R3`.
5. `T-008` lands the CLI amendment (`026-L1`/`026-L2`): `parse_endpoint` admits
   any host, `probe()` keeps the fail-closed pre-plugin connect, the suites split
   and the refusal line re-pin per §CLI amendment. **Until `T-008` merges the CLI
   still refuses non-loopback endpoints while `026-L1` states it must not** — an
   accepted, documented contract-ahead-of-code divergence window that the ADR
   itself acknowledges by shipping no product code.
6. Ledger annotations (ADR Rollout item 6): `025-C4` and `025-L1` gain their
   `amended-by 026-L1` lines per `021-L4`; `025-R1` records this ADR as filling
   its catalog half with the profile half reserved forward to `026-R2` (ADR-027);
   `026-S1` gains an `amended-by`/partially-satisfied line recording that the
   seed mints grammar-valid ids while `data/manifests/` is empty (`021-L4`).
7. The deferral is recorded (ADR Rollout item 7): the binary, schema and CLI
   amendment ship as `T-004`…`T-008`; the transport assurance stays `026-R1`.

Task authoring: the five task documents `docs/tasks/T-004`…`T-008` are written
per `023-C2`/`023-C4` as the ADR's rollout (separate source docs, not this
artifact); this proposal specifies each task's scope and the Test-strategy items
it must satisfy so they can be authored faithfully (A18).

## Deferred / out of scope

- TLS and any transport assurance, including the registry→upstream hop's
  integrity (bearer-over-plaintext deviation from RFC 6750 included) — `026-R1`,
  owner `unassigned`.
- The profile half of `025-R1`, curation intake beyond the admin face, and auth
  beyond the single bearer token — `026-R2`, ADR-027 (reserved).
- Upstream prefetch and parallel hydration (the cold N-serial-lookup cliff) —
  `026-R3`, owner `unassigned`.
- The seed corpus adopting the real shipped manifest ids once the first manifest
  PR lands (`data/manifests/` is empty, A14) — no follow-up doc yet; a data edit,
  not a contract change.
- A non-bearer / per-token-identity admin auth scheme; `Allow`/`Retry-After`
  header precision beyond the ADR's "names any method" — `026-R2` or no follow-up
  yet.
- Public-root corpus governance and dispute resolution — the ADR names it a
  neighbor, not content; no follow-up yet.
- Lifting the six copied invariant blocks (§`apps/registry`) from
  `apps/discovery-cli` into a shared crate so both binaries call one copy, which
  also gives `Source`/`spells_lookup_key` that shared home: `apps/registry` ships
  the copies, each pinned by the tests above, and the lift itself is an
  implementation refinement, not a contract decision.

## Revision log

- r1: catalog JSON-vs-TOML harness clash, open Q1/Q2 (§Catalog document and its spec) — JSON format carved out of the conformance walk; r2 guards only the parse/rejection loops, keeps the fixture-name test ungoverned by that guard through a directory-aware `is_fixture_name` (#19's case-unique-name claim on a case-insensitive filesystem, its only coverage), and the `boundary_tests` sweep keeps both roots while skipping only `data/catalog/` + `schemas/catalog/`.
- r1: `026-B1 × 026-B2` jointly unsatisfiable, open Q5 (§Budgets) — write-time dual caps; r2 restores `026-B1`'s own oldest-`fetched_at` eviction key and takes the octet test from the write buffer's length — per-entry totals, running total and insertion sequence deleted (#5).
- r0: `026-B1`–`026-B3` export location (open Q4) — addressed by keeping them private to `apps/registry`, no README count change (§Chosen design → Budgets).
- r1: `026-S1` empty manifest tree, open Q3 (§The seed corpus) — minted grammar-valid ids; r1 records the partial satisfaction on the `026-S1` unit itself via the ledger pass (§Rollout step 6); real-id adoption deferred (§Deferred).
- r0: plaintext upstream integrity (§Risks, open Q9/Q14) — addressed by validating upstream `game_id`/`name` before caching; transport integrity deferred to `026-R1`.
- r1: `026-C5` loopback/token ambiguity, open Q11 (§Admin face and auth) — configured token required everywhere; loopback+no-token enabled; non-loopback+no-token disabled; enablement judged from the bound address via `IpAddr::is_loopback` on `local_addr()`, the registry's own judgment, not a copied `probe()` block.
- r1: two instances on one corpus / torn reads, open Q10 (§Admin face and auth → Concurrency) — two-writer state refused, and the write-time tmp failure is the whole mechanism (no startup check or instance marker); r3: persist runs inside the one model write lock and the persist-only lock idea is deleted outright, so persist order cannot diverge from mutation order, and the one-local-write wait is named as accepted rather than pointed at §Deferred (#11).
- r1: no request-side or corpus-document budgets anywhere (§Budgets) — all private to `apps/registry`: `MAX_REQUEST_HEADER_OCTETS` 65,536; `MAX_REQUEST_BODY_OCTETS` = the document cap (so no per-record key-count bound); `MAX_CONCURRENT_CONNECTIONS` 64, over-cap closed at accept; `MAX_CATALOG_DOCUMENT_OCTETS` 16,777,216 sized from `026`'s corpus scale and shown in `spec.md`; inbound framing refusal before mutation (`http.rs`). r4: that document cap is enforced at both ends — the check moved onto the post-`revision`-bump buffer that is renamed, an over-cap upsert the named `422` before any tmp, so a revision growing a digit cannot persist what the loader would refuse (#10); cache eviction re-serializes per dropped entry; a thread answers one request then closes (A23).
- r1: health as always-`200` status doc vs readiness probe, open Q7 (§Serving face) — reports `upstream` from the last-known verdict, never non-`200`, ungated; names the never-contacted caching body (`down`/`last_upstream_fetch: 0`) and `200 []` for a root empty join; r2: only lookups fire the once-per-TTL probe — a health read never does — and #6 pins the `--refresh-interval <seconds>` override (a bare count of seconds) that re-times it.
- r0: CLI behavior change deferred to `T-008` (§Risks last) — accepted: documented contract-ahead-of-code divergence window (§Rollout step 5).
- r1: unvalidated lookup-key alphabet (§Serving face step 1, §Caching mode step 4) — r2 deletes the registry-private `.`-ban and r3 the space refusal, which traverses nothing and would `404` a key the shipped pair admits, keeping the path-safety refusals at their own scope (a further separator, a `.`/`..` component, a control or invalid-UTF-8 byte), so a dotted `local/<dir>` key is admitted; r4 makes the widened alphabet safe on the outbound hop by copying the shipped `percent_encode`/`candidates_url` builder as the sixth invariant — the upstream request target is built from the decoded slug, so a space- or `%`-bearing key reaches upstream only percent-encoded (#1, #3, #4, #7); corpus refuses repeated `game_id` and a within-record repeated `source_key`, cache refuses repeated `slug`.
- r1: DELETE unpinned from the `026-C4` bump/persist rule (§Admin face and auth) — delete now states its `revision` bump and tmp-and-rename; `stats` fixed to three integer counts (#10).
- r1: CLI suite re-pin would dial remote hosts (§CLI amendment) — remote admission pinned at `parse_endpoint` in the unit suite only; subprocess suite keeps refused spellings; no test resolves or dials non-loopback (#15); #13 names the env seam (`XDG_DATA_HOME`/`APPDATA` in the child, #17) as the only document-relocation lever, since there is no path knob.
- r1: `apps/registry` dependency line (§`apps/registry`) — `bindforge-core` + `discovery` + `serde_json`; the bin-only client's copied invariants are named and re-pinned, each by the §Test strategy item cited for it, six since r4 added the `percent_encode`/`candidates_url` request builder; the `discovery` edge is taken for `spells_lookup_key` alone, its `winreg` transitive retired by the deferred lift (§Deferred).
- r1: exotic-Unix matrix row carried no decision (§Compatibility matrix) — decided `No` (not a v1 target); r2 relabels row 1 to `Linux` so no platform is both a `Yes` the tasks owe and a `No`, and names both caching documents on the Windows row.

## As-built (recorded 2026-11-19)

What the run's build stages actually shipped, read against this approved design
over `git diff 217c46fd…HEAD`. This section records drift; it changes no decision
above. Each entry states the drift, why it happened, and the sections of this
document it lands against. The build record — gate evidence, per-seat review
dispositions, and the reasoning behind each choice — is
[`03-build-notes.md`](03-build-notes.md).

**A. The service shipped, not only the contract.** *Drift*: §Rollout opens "this
run ships the contract and the ledger annotations only", yet the branch carries
the whole `apps/registry` crate (both modes, the admin face, the cache, `query`),
`schemas/catalog/v1/` (spec + 5 valid + 18 invalid `.json` fixtures),
`data/catalog/v1/catalog.json`, the three harness carve-out files named in
§Catalog document and its spec with the `schemas/README.md` amendment, and the
`026-L1` amendment in `apps/discovery-cli`. *Reason*: the run's implement stages
built the service and all five code-review seats reviewed that diff as the change
under review; a build stage does not delete a reviewed, green implementation to
match a rollout plan, and `docs/tasks/**` is runner-owned, so no `T-004`…`T-008`
source documents exist to reassign it to. *Sections*: Context, §Goals/Non-goals
(first non-goal), §Rollout 1–5 and "Task authoring". **Decision owed to the
operator**: re-scope the ADR's rollout to what the branch carries, or author
`T-004`…`T-008` and treat this branch as their implementation.

**B. §Rollout step 6 did not run.** *Drift*: no `amended-by 026-L1` lines on
`025-C4`/`025-L1`, no catalog-half line on `025-R1`, no partially-satisfied line
on `026-S1`. *Reason*: `docs/adrs/**` is outside every stage's file ownership in
this run; the ledger pass is a separate act. *What did land*: `data/catalog/README.md`
states the seed's ids are minted, says the unit's wording is ahead of the file,
and names step 6 as where the annotation is owed. *Sections*: §Rollout step 6,
§Deferred.

**C. Module plan: nine files became thirteen.** *Drift*: every named module
exists with its named duties, and four more were added — `keys.rs` (§The serving
face step 1's rule, plus the copied `percent_decode`/`percent_encode` the design
put with the request builder), `persist.rs` (the `<data>` rule, the
`cache-v1.json`/`catalog-v1.json` names, the tmp-and-rename), `state.rs` (the mode
enum, the lookup policy, the health/stats projections) and `wire.rs`
(`#[cfg(test)]` byte-shape harness). *Reason*: one module per rule rather than per
role — the key rule and the persist rule are each cited by three callers.
*Section*: §`apps/registry` (Modules).

**D. Four named refusals the contract does not carry.** *Drift*: (i) any admin
request carrying `Origin` is a `400`; (ii) an upsert whose declared
`Content-Type` is not `application/json` is a `415` (the header stays optional,
so no described caller is refused); both decided on the header block, before the
body is read. (iii) The catalog document's entry set is **closed** at both levels
— a member outside `version|revision|games` or outside
`game_id|name|source_keys` is a load and upsert refusal (one new fixture,
`record-entry-outside-closed-set.json`), where §Catalog document and its spec
names only "a non-object/missing field". (iv) A parameterized admin path asked
with a method it does not carry (`GET /v1/admin/games/<game_id>`) is a `404`, not
a `405` with `Allow: DELETE`, and a disabled face answers one flat `404` for every
dispatch outcome; the `405`/`Allow` rule survives on the collection and count
routes of an enabled face and on both public routes. *Reason*: (i)/(ii) close the
one write a browser can forge to a token-gated plain-HTTP face without a preflight
(`026-R1` defers TLS); (iii) keeps a document this instance would write back out of
a shape it never reads; (iv) is `026-C5`'s never-an-existence-oracle rule taken to
its conclusion. *Sections*: §The serving face (`405`/`Allow` paragraph), §The admin
face and auth, §Catalog document and its spec (loader roster), §Test strategy #1/#9/#10.

**E. `MAX_RESPONSE_OCTETS` is bidirectional.** *Drift*: A1's 65,536 bounds this
instance's **outbound** `GET /v1/candidates` body as well as the upstream read —
`corpus::response_bounded`/`candidates_body` answer the largest `game_id`-ordered
prefix that fits inside the `MAX_CANDIDATES` window (256 postings at the
`024-B1`/`013-B4` length bounds are ≈72,000 octets, over the cap a count-only trim
would emit), `cache::stage` truncates a staged candidate list and `cache::load`
refuses a stored entry over the cap. §The serving face step 3 said only "the byte
shape A1 pins". *Reason*: a face whose answer the pinned client refuses has stopped
being the byte-identical face `026-C1` requires. *Open nit carried into the next
round*: `main.rs::query_command` prints the whole up-to-`MAX_CANDIDATES` join, not
the trimmed one, so `serve.rs`'s "`query` prints the same projection it answers
with" comment is ahead of the code (raised as an FYI by both the compatibility and
testing seats). *Sections*: §The serving face steps 2–3, §Budgets, §Test strategy
#2/#5/#7.

**F. Budgets as built.** *Drift*: every figure this section names landed unchanged
(`MAX_REQUEST_HEADER_OCTETS` 65,536; `MAX_REQUEST_BODY_OCTETS` =
`MAX_CATALOG_DOCUMENT_OCTETS` = 16,777,216; `MAX_CONCURRENT_CONNECTIONS` 64 with
over-cap closed at accept; `026-B1` 5,000; `026-B2` 1,048,576; `026-B3` 3,600;
upstream `2 s`/`5 s`), and six constants the design did not name were added:
`UNREAD_BODY_DRAIN_OCTETS` (65,536) and `UNREAD_BODY_DRAIN_TIMEOUT` (2 s) for the
bounded discard of a refused admin request's declared body;
`REQUEST_READ_TIMEOUT` (60 s) and `RESPONSE_WRITE_TIMEOUT` (15 s) — the two
low-and-slow timeouts §`apps/registry` asks for without figures; and
`FETCHED_AT_PAST_TOLERANCE_SECONDS` (31,536,000) /
`FETCHED_AT_FUTURE_TOLERANCE_SECONDS` (86,400), which are §Budgets'
`[now − 31_536_000, now + 86_400]` interval given names. *Reason*: a refusal that
precedes the body read, and a wall-clock bound per direction, need their own
numbers. Open question 4's answer holds: all of them are private to
`apps/registry`, with no `schemas/README.md` bullet and no change to core's
thirteen-constant family. *Section*: §Budgets.

**G. The persist gained one fsync.** *Drift*: a save is `create_new(true)` at
`0600` → `write_all` → **`sync_all`** → rename → mode re-assert; the parent
directory is deliberately not synced (std has no portable directory handle). The
copied shipped precedent (`matches.rs`) renames a tmp it never fsyncs. *Reason*:
an admin `2xx` acknowledges a mutation, and the design's own promise that "a
restart cannot lose an accepted mutation" is not kept by a rename whose bytes may
still be in the page cache. *Sections*: §Admin face and auth (Concurrency, A7),
§`apps/registry`'s copied list.

**H. A placement refusal the design did not name.** *Drift*:
`Catalog::load_required` makes an absent `--corpus` a startup refusal naming the
path consulted (present-and-empty stays a valid zero-game root); the caching
pair (`cache-v1.json`, `catalog-v1.json`) keeps the designed absent-is-empty rule.
Caching mode also refuses to start when no `<data>` resolves, naming
`XDG_DATA_HOME`. *Reason*: a root serving an empty catalog while its operator
believes the corpus was loaded is the worse failure; the CLI has no path knob
(#13), so the missing-data case has to be said out loud. *Sections*:
§`apps/registry` (CLI surface), §Caching mode.

**I. The cache document's spelling moves §Budgets' arithmetic.** *Drift*:
`cache-v1.json` is serialized pretty, so a one-candidate entry is ≈250 octets
rather than the ≈150 assumed: `026-B2` starts evicting near 4,000 entries and
becomes the binding cap for an ordinary corpus, where the section's arithmetic
makes `026-B1`'s 5,000 bind first at ~7,000 slugs. Caps, the oldest-`fetched_at`
eviction key and the write-buffer-length octet test are as designed; the two
serving-side timeouts stay unlegged, deliberately. *Reason*: legibility of a
document an operator opens; switching to compact output changes the on-disk shape
two suites read, which is a test engineer's re-pin. *Section*: §Budgets.

**J. CLI elaborations.** *Drift*: `--flag=value` is accepted for every value
flag, and the unknown-flag refusal quotes only the flag name it did not recognize,
never the value that followed it; exactly one of `--root`/`--upstream` chooses the
mode or the run is refused; a `query` slug may contain `=` because only a `--`
token is ever split; connection threads are spawned with 256 KiB stacks. *Reason*:
the first two are credential rules — a mistyped `--admin-token=…` must not reach a
terminal, journal or CI log. *Section*: §`apps/registry` (CLI surface).

**K. The admin credential — built as approved, and one earlier record was wrong.**
Enablement is judged from the bound address together with the presence of a token:
loopback with no token is enabled with no credential; **a configured token
enables the face and is required on every admin request regardless of bind**; a
routable bind with no token has no admin face at all. The credential is the
`Authorization` header's value after `Bearer `, folded without an early return on a
length mismatch, and a `401` carries `WWW-Authenticate: Bearer` and never echoes
the token. No drift. Recorded because an earlier revision of
`03-build-notes.md` stated the opposite ("enabled exactly when the address bound is
loopback", and the header as `X-BindForge-Token`); both claims are corrected there.
*Section*: §The admin face and auth.

**L. Test strategy as built.** Fifteen integration suites under
`apps/registry/tests/` plus the in-process `wire.rs` harness and the per-rule unit
tests; fourteen of them map to #1–#19 and each header names the items it proves, and
the six legs `03-build-notes.md` routed to test-update landed in `8e40cf38`. The
fifteenth is a suite the design does not name: `zz_exec_probe.rs`, which proves the
process can be killed mid-write and runs last by name. #1's over-`MAX_CATALOG_DOCUMENT_OCTETS` condition
still ships no invalid fixture — the fixture would be 16,777,216 octets inside a
tree every sweep enumerates — and the spec's §Named refusals roster says so beside
the row. *Section*: §Test strategy.

**M. Not built, and not asked for.** No CORS preflight answer and no
`Access-Control-*` header (`OPTIONS` is a `404` like any undefined route); no
`max-age`/`stale-while-revalidate`, with `Cache-Control: no-store` on the responses
that carry it; no OpenAPI document; no `data/registries/` fixture; no CI change (the
crate is a workspace member, so the existing cargo trio covers it). The seed's six
records carry minted kebab `game_id`s against real `steam/<appid>` keys with
`revision: 1`, which is what §The seed corpus provides for. *Sections*:
§The serving face, §Compatibility matrix, §Deferred.
