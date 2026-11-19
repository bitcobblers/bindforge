# correctness review r1 (pass c0, full panel)

verdict: request-changes

Scope: whole document (`docs/designs/026-registry-service/02-proposal.md`), against
`docs/adrs/026-registry-service.md` and the Revision log.

## Evidence gathered

Checklist 1 (assumption register) — every `verified` row I could check against the
tree holds: A1 (`apps/discovery-cli/src/catalog.rs:23,26,299,344,353,364` — header
*and* body capped at 65_536, one decimal `Content-Length`, no `Transfer-Encoding`,
`read_exact`), A2 (same file, `the catalog answered \`{status}\`` at the status-line
branch), A4 (`matches.rs::game_id_is_valid` = `is_slug_name` && `MAX_SLUG_OCTETS`),
A6 (`matches.rs::data_dir`/`store_path`, XDG→HOME/APPDATA, absolute-only), A7
(`save` → fixed `matches.json.tmp`, `create_new(true)` at `0o600`, parent `0o700`,
rename, re-`chmod`), A9 (`conformance.rs` `fixtures()` returns *every* file and
`every_valid_fixture_…`/`every_invalid_fixture_…`/`fixture_file_names_…` sweep them
through core's TOML `parse`; `common/mod.rs::is_fixture_name` demands `.toml`),
A10 (`version_gate.rs::core_declares_exactly_the_major_versions_present_under_schemas`
compares the *set* of major names; `v1` is already in `supported_majors()`), A11
(`LOOPBACK_HOSTS` at `catalog.rs:34`, the host check at `:141`, `probe()`'s
`is_loopback()` refusal at `:248`, unit test `:498`, subprocess test
`cli_catalog_gate_tests.rs:112`), A12 (`CatalogError::line` at `:71`), A13
(`Cargo.toml:12 members = ["crates/*", "apps/*"]`), A14 (`data/manifests/README.md:10`),
A15 (`tests/common/mod.rs:12` "no … non-loopback request"; `FakeCatalog`,
`closed_loopback_port`), A16 (`crates/discovery/src/game.rs:39-89`), and the
`thirteen constants` claim (`schemas/README.md:156`).

Checklist 2 (anti-hallucination) — both cited standards sources are live and say
what the register says they say: RFC 6750 ("TLS is mandatory to implement and use
with this specification"; "The authorization server MUST implement TLS") for A20;
RFC 9110 §9.2.2 (PUT/DELETE idempotent), §15.5.6 ("MUST generate an `Allow` header
field in a 405 response"), §15.5.1 (400 = malformed request syntax), §15.5.21 (422
= content syntax correct but unprocessable), §15.6.4 (503 temporary, MAY send
`Retry-After`) for A21. Both rows are labelled `unverified` and the body defers the
binding force to the ADR's own units — the honest posture, not a finding.

Checklist 6 — the design decides every open contract question the ADR leaves
(Q1–Q14 by their own labels) and discharges `026-C1`–`026-C7`, `026-L1`–`026-L6`,
`026-B1`–`026-B3`, `026-S1` except where noted below.

Checklist 7 — n/a: no previous round. Revision-log entries r0: each was checked
against the text it cites; none is a reworded non-fix. The two dangling anchors are
filed as a nit (below).

## Findings

- [major][new] Chosen design → The serving face — "`upstream` is `none` (root) or
  `ok`/`stale`/`down` (caching)" — `026-C2` closes `upstream` to those four values,
  and the design makes the verdict a **cached** gauge that health never probes live
  ("the verdict is **cached and refreshed on the same TTL**"), so a caching
  instance that has not yet contacted its upstream has no legal value to answer:
  `none` is claimed by root mode, `ok`/`stale` assert a fetch that never happened,
  and `down` asserts an unreachable upstream the instance never tried.
  `last_upstream_fetch` has no defined value before the first fetch either, and the
  design's own `fetched_at` rules refuse "not a non-negative integer", so `null` is
  not obviously admissible. Test strategy #12 ("reports `mode`/`upstream`/counts")
  cannot pin the startup body, and `T-005` and `T-007` will pick differently.
  This state is created by this design (the ADR's gauge was implied-live); the fix:
  name the value for the never-contacted state — `upstream: down` with
  `last_upstream_fetch: 0` (or the `ok`-equivalent the design prefers) — in the
  serving-face paragraph, and extend #12 to pin a freshly started caching instance.

- [major][new] Chosen design → The admin face and auth — "`DELETE
  /v1/admin/games/<game_id>` percent-decodes the path segment, validates the grammar
  (A4), removes the record." — `026-C4` says "**every** accepted mutation bumps
  `revision` and persists atomically through the tmp-and-rename discipline"; the
  design states the bump only for `POST` ("upserts one whole … record, validated
  against `026-C3` … bumps `revision`") and test strategy #10 pins it only for
  upsert ("upsert validates + bumps `revision` + atomic rename; delete
  decodes+validates the id"). The Concurrency paragraph's generic "mutations … then
  write the atomic tmp-and-rename" covers DELETE's persist but not its bump. Failure
  scenario: `T-006` ships a DELETE that leaves `revision` unchanged and every pinned
  test stays green, against an explicit ADR clause — and the Goals promise that
  mutations are "stated to the level a test engineer can pin" is then false. Fix:
  add "bumps `revision` and persists through the same tmp-and-rename" to the DELETE
  bullet and "delete bumps `revision` and persists" to #10.

- [major][new] Chosen design → Catalog document and its spec — "The registry's
  loader (`corpus.rs`) refuses by name: `version != 1`, `revision` negative, a
  `game_id` failing the grammar+bound (A4), a `name` over 128 octets (A5), a
  `source_key` failing `spells_lookup_key` or `025-B1` (A3, A16), a non-object/missing
  field." — that refusal list is presented as complete and never makes a duplicate
  `game_id` a refusal, though `POST /v1/admin/games` is an **upsert** keyed by
  `game_id` and `DELETE /v1/admin/games/<game_id>` names one record: a corpus
  carrying two records with the same `game_id` loads, then makes delete ambiguous,
  makes upsert's "one whole record" replacement non-deterministic about which row
  survives the next atomic write, and makes `stats`' `games` count disagree between
  the loaded model and the saved file. The same hole is in the cache document:
  `026-C7` sorts `entries` by `slug` byte order (one entry per slug implied) and the
  design's cache refusals ("refusals for bad version/shape/`game_id`/`name`", test
  #4) never name a duplicate `slug`, so a hand-edited cache with two entries for one
  slug has an undefined TTL winner. Fix: state that `corpus.rs` refuses a repeated
  `game_id` (and a repeated `source_key` inside one record) and `cache.rs` refuses a
  repeated `slug`, by name; add both to the named refusal lists in #1 and #4 and to
  `schemas/catalog/v1/fixtures/invalid/` so the corpus, not only the Rust, carries it.

- [major][new] Chosen design → CLI amendment — "The two pinned suites (unit +
  subprocess, A11) re-pin: remote hosts are now admitted" versus Test strategy #15
  "**no test dials non-loopback** — the in-process loopback fake stays the only
  server (A15)". The subprocess suite being re-pinned is
  `endpoint_policy_refuses_everything_beyond_the_loopback_spelling`
  (`cli_catalog_gate_tests.rs:112`), whose remote cases are `http://catalog.example`
  and `http://catalog.example:8080/v1` and which asserts today that each fails
  *before enumeration* with `error: catalog transport unavailable:`. Once
  `parse_endpoint` admits any host, those two inputs are admitted, so the same
  subprocess run proceeds into `probe()` — `to_socket_addrs` of a public domain plus
  a `TcpStream::connect_timeout` — and its stderr becomes `catalog unreachable`,
  network-dependent, and a direct violation of #15 and of the suite's own stated
  invariant (`tests/common/mod.rs:12`). `probe()` has no resolver or connector seam
  to inject (the amendment names only `parse_endpoint`, `probe()`, the refusal line
  and the two suites), so as written the amendment is either unpinnable or
  network-dialing. Fix: name the split — remote-host admission is pinned at
  `parse_endpoint` (pure, in the unit suite, and the `026-L2` disclosure line moves
  there), the subprocess suite keeps only the still-refused spellings
  (`https`/`ftp`/`@`/empty/non-numeric port) plus the re-pinned refusal text, and
  `probe()`'s admission is pinned only against an in-process loopback fake (or state
  that a `192.0.2.1` literal is the one sanctioned non-loopback input and correct #15
  to say so).

- [major][new] Chosen design → Budgets — "The three registry budgets are **private
  to `apps/registry`** (`constants.rs`)" — every budget the design names bounds the
  *cache* (`026-B1`/`026-B2`) or inbound *requests*; nothing bounds the catalog
  document, the one document the ADR expects to grow ("tens of thousands of games,
  each with several store keys"). `026-C3` makes the catalog loader's refusals "the
  schema's, by name, as every contract document's are (`schemas/README.md`
  discipline)", and that discipline is explicit: "A document larger than 1 MiB …
  cap binds *before* the read" and "`schemas/README.md` is this bound's one stated
  home" — which the shipped `data/manifests/README.md:14-18` restates for any
  contract document arriving from disk. As written, `corpus.rs` reads a 200 MB
  `--corpus` or local-entries file into memory with no named refusal, no `games`
  count cap and no cap on `source_keys` per record, and #1's refusal list has no
  over-cap case (unlike the cache, where "load over `026-B2` is refused by name" is
  pinned). Fix: add one private `MAX_CATALOG_DOCUMENT_OCTETS` (1_048_576, mirroring
  `026-B2`) checked on file length before the read in `corpus.rs`, name it in
  `constants.rs` and in #1's refusal list, and state in `spec.md` that it is the
  catalog format's bound (so the number has one stated home).

- [major][new] Chosen design → `apps/registry` — one crate, two modes — "It depends
  on `bindforge-core` for the grammar and name bounds (`is_slug_name`,
  `MAX_SLUG_OCTETS`, `MAX_NATIVE_NAME_OCTETS`, `escape_and_truncate`)" — the one
  predicate the design leans on hardest is not in core: `spells_lookup_key` and the
  closed source-token set are `Source`'s in `crates/discovery` (A16's own basis;
  `apps/discovery-cli/src/matches.rs:229,282` reaches them through its `discovery`
  dependency). The dependency list the design gives (`std` + `serde_json` +
  `bindforge-core`) cannot serve "a `source_key` failing `spells_lookup_key`" in
  `corpus.rs` or "`spells_lookup_key` to it" at `/v1/candidates`, so `T-004`/`T-005`
  must either add a dependency the design never lists — pulling the store scanners
  into the server binary — or re-spell `{steam, epic, local}` in `constants.rs`,
  giving the closed set two definitions. Fix: decide it in the module plan — either
  "`apps/registry` depends on `discovery` for `Source` (as `discovery-cli` already
  does)" in the crate's dependency sentence, or "the closed source-token set and
  `spells_lookup_key` are lifted into `bindforge-core` and `Source` re-exports them",
  naming the shipped text each replaces.

- [minor][new] Chosen design → The serving face — "The `026-L4` cold case is the
  named `503` with a legible reason phrase" appears inside the paragraph that both
  modes share ("The two modes share every handler"), and the design never states the
  root's answer for a well-formed, spellable slug that joins to nothing. `026-L4`
  confines the `503` to "a caching registry that holds neither a local entry nor a
  cache entry", and its own reasoning makes `[]` the *true* statement for a root
  that has never heard of the key, so `200 []` is the right answer — but `T-005`
  reading this section alone can reach `503`/`404` and no test in #2/#16 pins it.
  Fix: one sentence in serving-face step 2 — "an empty join in root mode is a `200`
  with `[]`; the `503` is reachable only from caching mode" — and one case in #16.

- [minor][new] Chosen design → The admin face and auth — "`GET /v1/admin/stats`
  answers `{games, source_keys, revision}` from the instance's own catalog document" —
  the field types are never stated. `026-L3`'s no-bulk rule turns on this: an
  implementer who answers `source_keys` as the list of keys answers with the
  catalog's whole key set over an admin route, which is exactly what `026-L3`
  forbids and what the rejected `GET /v1/admin/catalog` was rejected for (test #18
  pins only that `/v1/admin/catalog` is `404`). Fix: say `games` and `source_keys`
  are integer counts (with `revision`), and pin the three-field count shape in #10.

- [minor][new] Chosen design → The admin face and auth — "Enablement is judged from
  the bind address literal and the presence of a configured token." — the design
  never says what makes a literal loopback. `--host` takes the same spelling space
  the CLI's endpoint policy admits (`parse_endpoint` normalizes hostnames, and the
  amendment broadens hosts further), and the shipped precedent judges loopbackness
  twice and differently: the `LOOPBACK_HOSTS` literal set for configuration and
  `addr.ip().is_loopback()` on the *resolved* address in `probe()`. Under a
  literal-only rule, `--host localhost` with no token is a coin flip that decides
  whether the admin face exists at all — and it is the ADR's own stated use case ("a
  user's own daemon answers its own machine with no token friction"). Fix: judge
  enablement from the address actually bound (`local_addr()`'s `ip().is_loopback()`,
  the shipped `probe()` predicate), or name the closed loopback-literal set beside
  the quoted sentence.

- [minor][new] Chosen design → Budgets — "the registry evicts the oldest `fetched_at`
  until **both** … hold" together with "TTL arithmetic uses the wall clock
  (`SystemTime`) with a guard that treats a backwards-jumped stamp as expired rather
  than immortal". Eviction order is the raw wall-clock stamp, so after a backwards
  clock jump the entries just fetched are the "oldest" and the dual-cap eviction
  discards exactly the freshest data first, repeatedly, while the TTL guard still
  serves stale entries; the answer then re-fetches and is evicted again. Fix: order
  eviction by a monotonic insert sequence held beside `fetched_at` (documented as
  insertion order, ties broken by `fetched_at`) and say so in #5 rather than "oldest
  `fetched_at` evicted first".

- [minor][new] Risks (row "Two `registry serve` on one corpus fail loudly") —
  "State it as a refused, unsupported configuration in `spec.md` and the CLI's
  startup check." — the mechanism the design relies on is the fixed-name
  `create_new(true)` tmp, which fails only at *save* time; a second instance starts,
  loads the corpus, and serves reads indefinitely, and a tmp left by a crash is
  indistinguishable from a live writer's. None of the nine modules in the file plan
  owns an instance marker, so the promised startup check has no mechanism and no
  test (#11 pins only "a second save over a stale tmp fails loudly"). Fix: drop "and
  the CLI's startup check" — the contract answer is the loud write-time failure,
  which is already stated — or name the marker (`registry.lock` under the corpus
  directory, exclusive-create at startup) as an `apps/registry` behavior with its
  own case in #11.

- [minor][new] Chosen design → Budgets — "`MAX_REQUEST_BODY_OCTETS` (a POST body
  cap)" — of the two new private request budgets only one gets a value
  (`MAX_REQUEST_HEADER_OCTETS` (65,536 …)); the body cap is unnamed, so the #11
  "refusals" it pins have no boundary to sit on, and it interacts with the missing
  `source_keys` count cap above: a record that is legal under `026-C3` can outgrow
  whatever number the implementer picks. The inbound framing rule is unstated with
  it: the response writer's `Transfer-Encoding` ban is pinned, but nothing says how
  a chunked or `Content-Length`-less admin `POST` is answered. Fix: state the number
  (with the record bound it is sized against) and one sentence in `http.rs` —
  a request body without a decimal `Content-Length`, or over the cap, is a `400`/
  `413` before any mutation.

- [minor][new] Options considered — "the two shipped JSON documents (`matches.json`,
  the candidate body) already live *outside* the TOML formats contract (A5, A6)" and
  Chosen design → Budgets — "the precedent `matches.rs` already set with its private
  `STORE_READ_BUDGET_OCTETS` (A7)" — the anchors do not carry the facts. A5 is the
  `MAX_NATIVE_NAME_OCTETS` bound and A6 the data-directory rule; neither states that
  those documents sit outside the TOML subset (that is `crates/core/tests/
  conformance.rs`'s walk over `.toml` fixtures plus `matches.rs`'s `serde_json`
  store). A7 is the atomic-persist row; the over-budget *file* refusal is
  `STORE_READ_BUDGET_OCTETS` at `matches.rs:36,194`, which no register row covers —
  and it is also the load-bearing precedent for the catalog/cache load refusals
  above. Fix: add one register row for `matches.rs`'s private read-budget refusal
  (`load`, `STORE_READ_BUDGET_OCTETS`) and re-anchor those two passages to it.

- [minor][new] Deferred / out of scope — "The seed corpus adopting the real shipped
  manifest ids once the first manifest PR lands (`data/manifests/` is empty, A14)" —
  this is the one deferral that contradicts its source clause rather than merely
  deferring it: `026-S1` requires "a `026-C3`-valid corpus opened with a handful of
  real games (game ids **drawn from the shipped manifest ids**, each carrying its
  `steam/<appid>` source key)". The empty tree makes the clause unsatisfiable today
  (A14 verified: `data/manifests/README.md:10` "The first shipped manifest lands as
  its own PR"), so minting provisional ids is the only possible act — but Rollout
  step 6's ledger annotations name only `025-C4`, `025-L1` and `025-R1`, so the
  ledger will record `026-S1` as discharged by a corpus that does not satisfy it.
  Fix: add `026-S1` to Rollout step 6's annotations (an `amended-by`/`partially-
  satisfied-by` line per `021-L4`, recording that the seed mints grammar-valid ids
  while `data/manifests/` is empty), so the deferral is visible on the unit itself.

- [nit][new] Chosen design → The serving face — "mode is an enum field on the server
  state, so a client cannot tell which answered (`026-C1`)" — the same face's
  `GET /v1/health` answers `mode` and `upstream` by name, and a caching instance's
  cold `503` where root answers `200 []` is a mode-revealing difference the ADR
  itself mandates. Fix: scope the claim — "`GET /v1/candidates` is byte-identical
  between modes; health reports `mode` by design."

- [nit][new] Chosen design → CLI amendment — "**drops the `LOOPBACK_HOSTS` host
  check**" — retirement of the check leaves `const LOOPBACK_HOSTS` (`catalog.rs:34`)
  unused, and the repo gate is `cargo clippy --workspace --all-targets -- -D
  warnings`. The Goals line ("retirement named at all three shipped sites") covers it
  in spirit; the amendment's bullet list does not. Fix: name the constant's deletion
  in the first bullet.

- [nit][new] Revision log — "catalog JSON-vs-TOML harness clash (§\"Where the format
  contract and the ADR disagree\", open Q1/Q2)" and "health as always-`200` status
  doc vs readiness probe (§Platform notes, open Q7)" — both anchor to sections this
  document does not have (the first is now "Catalog document and its spec", the
  second is now "The serving face"). Fix: re-point both entries at the sections that
  now hold the decision.

## FYI (no severity)

- `read_response`, `percent_encode` and `parse_endpoint` are private items of a
  bin-only crate (`apps/discovery-cli/Cargo.toml` declares no `[lib]`), so the
  upstream client's "reuses the shipped client's response discipline" and
  `endpoint.rs`'s "mirroring `parse_endpoint`" are necessarily a second copy beside
  the shipped one; the Deferred section names only the `data_dir` lift. Another
  seat's business (complexity), but the copy set is worth enumerating before
  `T-007` grows it.
- Keeping `026-B1`–`026-B3` out of `schemas/README.md` is a coherent call (and it is
  the right one for `crates/discovery/tests/zz_api_probe.rs`, which pins core's
  thirteenth exported constant), but the README's own principle is that a bound's
  "one stated home" is that file; `schemas/catalog/v1/spec.md` will state numbers the
  published budget list does not carry. Compatibility seat's call.
- A20/A21 are marked `unverified`; both cited RFCs check out under direct fetch (the
  four A21 claims are RFC 9110 §9.2.2, §15.5.6, §15.5.1, §15.5.21, §15.6.4). The
  register could say so, which would stop a later round spending its own fetch on it.
