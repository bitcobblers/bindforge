# Build notes: Registry service and manifest ingestion (026)

## Progress

Fix round for the code review's round-1 open findings (correctness, security, testing).
The tree already carried this fix's edits from the attempt that timed out; this attempt
re-verified each against the four gate legs and closed the one finding that attempt left
half-done: the `Content-Type` half of the browser finding, now landed (*Deviations →
Admin face and auth*).

**Completed — correctness seat.**

- Admin gate ran after the body: `admin::admits` takes the `RequestHead`, so a refused
  request never has its declared body read.
- The answer had no budget: `corpus::candidates_body` applies `MAX_CANDIDATES` and
  `MAX_RESPONSE_OCTETS` at the answer seam, trimming to the stable prefix the pinned
  client accepts; `cache::load` refuses a stored entry over the cap, `stage` truncates.
- Closed entry set at both levels: `DOCUMENT_ENTRIES`/`RECORD_ENTRIES` refuse any
  non-spec entry for loads and upserts; spec §Shape says "at **both** levels".
- `Catalog::load_required` refuses to start on an absent `--corpus` naming the path;
  `persist::save` `sync_all`s before the rename; keys delegate to
  `Source::spells_lookup_key`. The 404-vs-405 shapes are documented, not changed.

**Completed — security seat.**

- The credential was checked after the body was buffered: `read_request_head` answers a
  `RequestHead`, `admin_connection` gates on that, and only a passing request reaches
  `into_request`, where `read_body` buffers under the 16 MiB cap.
- `serve_one_connection` sets `RESPONSE_WRITE_TIMEOUT` before any write and shuts both
  halves down after; `corpus::response_bounded` keeps the `200` inside the cap;
  `http::drain_unread_body` discards a refused request's declared body inside
  `UNREAD_BODY_DRAIN_OCTETS`/`_TIMEOUT`; the header terminator is one `ends_with`.
- A disabled face is one flat 404 for every dispatch outcome, so no `Allow` list; an
  admin request carrying `Origin` is the named `400`; an upsert declared as any media
  type but `application/json` is the named `415` — both halves of the browser finding,
  both decided on the head.
- The unknown-flag refusal names the flag, never the value, and `--flag=value` parses so
  `--admin-token=…` is one argument; `data/catalog/README.md` no longer claims the ledger
  annotations exist and names §Rollout step 6 as where they land.

**Completed — testing seat.**

- Finding 1 closed on both sides: the answer is bounded in octets, the arithmetic is in
  the spec's §The answer's budget, and `wire.rs:606` is the leg that was missing — a
  corpus at every field bound whose join is over the cap before the trim, asserting a
  `200`, one `Content-Length`, a body at or under `MAX_RESPONSE_OCTETS`, and stable bytes.
- Four more wire legs: no `Allow` from a disabled face; an `Origin` request refused and
  inert where the same request without `Origin` mutates; an unauthenticated admin `401`
  whose declared body is never read; a root with an absent `--corpus` refusing to start.
  Plus unit legs for the closed entry set, the trimmed answer, the bounded discard, the
  head-only read, `--flag=value`, and the refusal that no longer echoes a value.
- This attempt's pair for the `415`: `wire::an_upsert_declared_as_a_form_media_type_is_
  refused_and_writes_nothing`, and `serve::tests::an_upsert_body_declared_as_any_other_
  type_is_refused_on_the_head`. No committed test file was edited; findings 2–6's legs
  are in *Test updates owed*.

**Remaining.** No open finding from the three seats is undone in code. Out of reach by
ownership, reasoned in *Deviations*: §Rollout step 6's ledger annotations (`docs/adrs/**`
is not writable from a stage) and the performance seat's cache items — a fourth seat this
round was not asked to answer. Test-owned legs: *Test updates owed*.

**As-built pass (2026-11-19).** A read of the whole branch
(`git diff 217c46fda9eb95230c8df62233eca8ef7b6ffb71..HEAD`) against `02-proposal.md`,
recorded as a dated **As-built** section at the foot of that document: thirteen entries
(A–M) — scope, the unwritten ledger pass, the module plan, the four contract-unnameable
refusals, the bidirectional response cap, the constants that joined §Budgets', the
persist's `sync_all`, the corpus-placement refusal, the pretty cache's arithmetic, the
CLI elaborations, the admin credential, the test roster, and the not-built list. It
changes no design decision. Three consequences for this file:
`02-proposal.md` is now the drift record and the bullets below stay the working
reasoning; all six *Test updates owed* legs below **landed** in `8e40cf38` and are
annotated where they live; and two records here were factually wrong about the admin
face and are corrected in place (*Deviations → Admin face and auth*, and the two
*spec ↔ code* rows about enablement and the credential header) — the code implements
§The admin face and auth's rule as approved (loopback bind **or** a configured token
enables; a configured token is required on every admin request regardless of bind), over
the `Authorization: Bearer` header, and the earlier text had it as loopback-only bind
with an `X-BindForge-Token` header. Panel state after this branch's last passes: five
approvals — correctness, security and testing at r1c1, compatibility and performance at
r1c2 — with no blocker or major open. The open residue is one minor and four nits: the
two serving-side timeouts still have no test seam (testing r1c1's narrowed minor, and a
decision these notes already record in *Test strategy (#1–#19)*), the `query`-vs-trim
projection sentence (*spec ↔ code diffs*), the `--port` leg's borrowed
`closed_loopback_port()` helper (compatibility r1c2 C-5), the persist leg's doc comment
crediting itself with `sync_all` coverage it cannot observe, and the escaped-name half of
the trim's ≈283-octet figure (testing r1c1 nits 3–4).

**Last validation.** `cargo fmt --all -- --check` silent, exit 0; clippy under
`-D warnings` clean on the re-checked `bindforge-registry`; `cargo test --workspace
--locked` ok everywhere (registry unit 78/78, every integration suite ok, none
regressed); `npm run check` 61 files, 692 passed, 2 skipped. Tails in *Gate posture*.

## Gate posture

The four legs, as I ran them in this worktree after the final edit (the `415` rule and
its two legs included):

```
$ cargo fmt --all -- --check                     # no output, exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Checking bindforge-registry v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
$ cargo test --workspace --locked                # every leg ok, none failed
$ npm run check        (in tools/orchestration)
    Checking formatting...
    All matched files use Prettier code style!
     Test Files  61 passed (61)
          Tests  692 passed | 2 skipped (694)
```

`npm ci` is outside this stage's shell allowlist, so the Node leg ran against the
`node_modules` the runner had already installed from the lockfile; the runner's
own gate recorded `npm ci && npm run check` ok.

The registry suites this round touched, from the workspace run:

```
running 78 tests  apps/registry (lib+bins unit)  ... test result: ok. 78 passed; 0 failed
running  9 tests  admin_auth                     ... test result: ok.  9 passed; 0 failed
running  9 tests  admin_mutations                ... test result: ok.  9 passed; 0 failed
running  7 tests  cache_document                 ... test result: ok.  7 passed; 0 failed
running 11 tests  caching_policy                 ... test result: ok. 11 passed; 0 failed
running  7 tests  candidates_join                ... test result: ok.  7 passed; 0 failed
running 13 tests  catalog_document               ... test result: ok. 13 passed; 0 failed
running  7 tests  cli_end_to_end                 ... test result: ok.  7 passed; 0 failed
running  5 tests  data_layout                    ... test result: ok.  5 passed; 0 failed
running  4 tests  health_route                   ... test result: ok.  4 passed; 0 failed
running  5 tests  persist_and_concurrency        ... test result: ok.  5 passed; 0 failed
running  7 tests  response_shape                 ... test result: ok.  7 passed; 0 failed
running  7 tests  slug_decoding                  ... test result: ok.  7 passed; 0 failed
running  6 tests  upstream_client                ... test result: ok.  6 passed; 0 failed
running  3 tests  upstream_endpoint_gate         ... test result: ok.  3 passed; 0 failed
running  1 test   zz_exec_probe                  ... test result: ok.  1 passed; 0 failed
```

The runner's own gate for the code under review recorded all four legs green — `$ npm
run check`, `$ cargo fmt --all -- --check` and `$ cargo clippy --workspace --all-targets
-- -D warnings` each `(exit 0)`, and `$ cargo test --workspace --locked` `(exit 0)` whose
captured tail is the zero-test doc-test leg (`test result: ok. 0 passed; 0 failed; 0
ignored`), so the per-suite counts above are from my own run rather than that tail. No
leg failed, so there is no failing tail to quote.

**After this round.** The per-suite counts above are the fix round's, taken before the
test-update stage added legs. Every code and test byte on this branch is in `8e40cf38`
(everything after it is `docs/`), and the runner's gate for that state is green at
`2026-11-19T23:01:03Z`: `npm run check`, `cargo fmt --all -- --check`, `cargo clippy
--workspace --all-targets -- -D warnings` and `cargo test --workspace --locked`, each exit
0. That is the evidence the three dissent and ratification passes relied on (correctness,
security and testing at r1c1; compatibility and performance at r1c2). The as-built pass
that wrote *Progress*'s last paragraph re-ran nothing: `cargo` is outside its shell
allowlist, so it read the code the gate ran and reports the runner's result, not new
numbers.

No committed test was edited and none regressed. The reason the new gates pass over
them untouched: no committed request carries `Origin` or `Content-Type` at all — which
is also why the `415` rule is written narrow, refusing only a *declared* type so a leg
that declares none is untouched (the only `Content-Type` under `apps/registry/tests` is
on a fake upstream *response*, `tests/common/mod.rs:1002`), and the suites that want the
admin face absent get it by binding a non-loopback `--host`, which is the mechanism the
face itself uses. `zz_exec_probe` runs last by name so its SIGKILL self-test cannot
disturb the others.

## Test updates owed (routes to test-update) — all six landed in `8e40cf38`

These are integration legs that pin behaviour this round added or newly made
observable. They are the test engineer's to write; I did not touch `tests/**`.

**Landed — all six, by the test-update stage in `8e40cf38`.** Each row below keeps its
original wording (it is the route that was issued) with the leg that now pins it named at
the end; nothing in this list is still owed.

1. `apps/registry/tests/candidates_join.rs` — an octet-ceiling leg. Today
   `a_key_claimed_by_more_games_than_the_cap_is_truncated_to_256_in_order` (`:135`)
   pins the count cap, and `tests/response_shape.rs` never measures a body against
   `constants::MAX_RESPONSE_OCTETS`; but 256 postings at the `024-B1`/`013-B4`
   length bounds are ≈72,000 serialized octets, over the cap at a crossover near
   231. What this round's `corpus::candidates_body` needs is a corpus whose 256
   postings carry 128-octet `name`s, asserting the answer is the `game_id`-ordered
   stable prefix, its body is at most `constants::MAX_RESPONSE_OCTETS`, and it is
   still a `200` — not a refusal, and not a socket-level truncation. **Landed**:
    `candidates_join.rs::the_octet_cap_trims_a_wide_join_to_a_stable_prefix_under_the_response_budget`
    (`:194`) — 256 postings at both field bounds over the wire: `200`, one
    `Content-Length` equal to the body, the body within the cap, the prefix in `game_id`
    order. The `wire.rs:605` unit pair proves the unbounded body is over budget before it
    proves the bounded one is not, so neither leg can pass vacuously.
2. `apps/registry/tests/caching_policy.rs` — the seat's findings 2 and 3. The two
   fresh legs (`a_fresh_cache_entry_answers_without_an_upstream_lookup` at `:151`,
   `the_refresh_interval_override_ages_a_fresh_entry_and_retimes_the_probe` at
   `:326`) seed `fetched_at = now_unix()` before the subprocess starts and then run
   `--refresh-interval 1`, so the freshness budget is one second of wall clock that
   must also cover spawn and the printed-port poll. Either take the stamp after
   `Registry::caching(..)` answers, or let the fresh leg run on the default `026-B3`
   TTL and keep `--refresh-interval 1` only where expiry is the subject. And in
   `:326`, assert `upstream.lines_for("GET /v1/health")` grows by exactly one probe
   across the wait and the second lookup: the leg's name claims the re-timing and
   only the `/v1/candidates` count at `:358` is checked today. **Landed both ways**:
    `a_fresh_cache_entry_answers_without_an_upstream_lookup` (`:156`) runs on the default
    `026-B3` TTL, and
    `the_refresh_interval_override_ages_a_fresh_entry_and_retimes_the_probe` (`:333`)
    lets the instance cache the entry on its own first lookup, so `fetched_at` post-dates
    the spawn, and it polls `upstream.lines_for("GET /v1/health")` for exactly one more
    probe across the wait and the refreshed lookup.
3. `apps/registry/tests/` — a leg that passes an explicit `--port <N>` and asserts
   the server answers there. The flag parses, but no committed suite uses it:
   `tests/common/mod.rs:504` deliberately relies on the port-0 default and reads the
   printed address, which is right for every other leg and leaves the explicit-bind
   branch of `main.rs::bind` unpinned. **Landed**: `cli_end_to_end.rs:513-556` starts an
    instance with an explicit `--port <N>` and asserts it answers there. Carried nit
    (compatibility seat, r1c2 C-5): the value comes from `closed_loopback_port()`, whose
    stated contract is the `026-L4` unreachable — it drops the listener it learned the
    port from, so a parallel `bind("127.0.0.1:0")` in the same test binary can take it
    first; rare on Linux's rotating allocator, unproven on the matrix's Windows row.
4. `apps/registry/tests/persist_and_concurrency.rs` — a leg for the ordering
   `persist::save` now documents: `write_all` → `sync_all` → rename. The suite
   already reads the document back and checks no `.tmp` remains; an assertion that
   the renamed document always holds the record the mutation answered for is what
   stops a refactor from dropping the `sync_all` and keeping the suite green. **Landed**:
    `persist_and_concurrency.rs::every_accepted_mutation_is_answered_only_after_the_document_holds_it`
    (`:348`). One correction from the testing seat (r1c1 nit 3): what that leg observes is
    answer-after-rename, not the fsync — deleting `sync_all` would leave it green — so the
    durability step stays deliberately unlegged, since proving it needs a crash harness
    this suite does not have. The leg's own doc comment overstates this; restating it is
    test-owned.
5. `apps/registry/tests/admin_auth.rs` — three legs for behaviour this round added
   that no committed suite covers: (a) with the face disabled (the `--host 0.0.0.0`
   shape the suite already uses), every admin path answers the same `404` and sends
   **no `Allow` header**, for `GET`/`POST`/`PUT`/`DELETE`; (b) an admin request
   carrying any `Origin`, with a valid credential, is the named `400` and nothing is
   mutated afterwards — the seat's finding 4 asks for this one in particular; and
   (c) an unauthenticated admin `POST` that declares a body and then stops writing
   is refused before the body arrives, so the refusal is not separated from the
   request by `constants::REQUEST_READ_TIMEOUT`. **Landed**, all three:
    `a_disabled_admin_face_publishes_no_allow_list_for_any_method` (`:235`),
    `an_admin_request_carrying_an_origin_is_400_and_mutates_nothing` (`:285`), and
    `an_unauthenticated_admin_request_is_refused_before_its_declared_body_arrives`
    (`:357`). The testing seat notes (r1c1 nit 2) that the last of these bounds its own
    20 s socket timeout, so it is a leg about the credential gate and not about
    `REQUEST_READ_TIMEOUT`, which stays unlegged by decision (see *Test strategy (#1–#19)*).
6. `apps/registry/tests/admin_mutations.rs` — legs for the `415` that closes the
   `Content-Type` half of the browser finding: an upsert whose body is declared
   `text/plain`, `application/x-www-form-urlencoded`, `multipart/form-data` or
   `application/xml` is the `415` whose body carries
   `error: unsupported_media_type`, with the corpus byte-identical afterwards, and the
   same record declared `application/json; charset=utf-8` still lands. Nothing
   committed has to move for the rule to pass — the header is optional, and no committed
   admin request sends one (the only `Content-Type` under `apps/registry/tests` is one a
   fake upstream response sends, at `tests/common/mod.rs:1002`) — so these are additions
   and not a re-pin. A leg that sends `Content-Type: application/json` on the existing
   mutations would pin the admitted spelling at the subprocess seam as well. **Landed**:
    `an_upsert_declared_as_a_non_json_media_type_is_415_and_writes_nothing` (`:201`) and
    `an_upsert_declared_as_json_lands_and_other_routes_ignore_the_header` (`:254`), which
    pins `application/json; charset=utf-8` as admitted and shows `stats` and a `DELETE`
    are not asked about the header. See
   *Deviations → Admin face and auth*.

## Deviations

**Scope.**

- **This branch ships the product code the proposal's §Rollout assigns to
  `T-004`…`T-008`.** §Rollout opens "this run ships the contract and the ledger
  annotations only; the service lands as tasks", and §Goals/Non-goals repeats it.
  What the run's build stages actually landed — and what all five review seats
  reviewed as the change under review — is the whole `apps/registry` crate (both
  modes, the admin face, the cache, the CLI) plus its suites, alongside the
  `026-L1` admission in `apps/discovery-cli`. This is the run's largest deviation
  and it is recorded, not resolved: deleting a reviewed and green implementation to
  match a rollout plan is the operator's call, not a build stage's, and the five
  task sources are runner-owned (`docs/tasks/**` is read-only to stages). Decision
  owed: either the ADR's rollout is re-scoped to what the branch carries, or
  `T-004`…`T-008` are authored and this branch becomes their implementation.
- **§Rollout step 6's ledger annotations are not written.** `025-C4` and `025-L1`
  need their `amended-by 026-L1` lines, `025-R1` needs the catalog-half line,
  `026-S1` needs the partially-satisfied line for the minted seed ids.
  `docs/adrs/**` is not writable by any stage in this run, so step 6 lands with the
  ledger pass. What did land is the honest half: `data/catalog/README.md` — the
  file the `026-S1` line is about — no longer asserts the annotation exists, says
  the unit's wording is ahead of the seed, and names the annotation as owed here.
- **The `GET /v1/games` list route does not exist, which is what §The serving face
  requires.** "No route answers a whole document in either mode" (`026-L3`), and
  `tests/…` pin the no-bulk posture through `/v1/admin/catalog` (`026-L3`, #18). No
  deviation, recorded here because the earlier version of these notes had it the
  other way round.
- **No OpenAPI document and no `data/registries/` fixture were written, because the
  proposal asks for neither.** The earlier version of these notes claimed both as
  gaps; they are not in §Chosen design or §Rollout. The wire shape is pinned by the
  proposal's §The serving face and §The admin face and auth, by A1's byte shape, and
  by the committed suites.

**The serving face.**

- **The face answers no CORS preflight and sends no `Access-Control-*` header.**
  The proposal names no browser client: its §The serving face prescribes the
  candidates, health, `404` and `405` behaviours and nothing else, so an `OPTIONS`
  is a `404` like any other route it does not define. Rationale: a header set that
  invites browser traffic to a service with no browser auth story is a larger
  surface than the feature asks for. Consequence: a page in a browser cannot call
  this registry, which is the posture the admin face's `Origin` refusal assumes.
- **No `max-age`, no `stale-while-revalidate`, and `Cache-Control` is sent only as
  `no-store` on the responses that carry it.** Rationale: same clause — the
  proposal puts freshness in the service's own cache (`026-B1`–`026-B3`) and never
  in an HTTP cache, and no committed leg reads a freshness header. Decision: leave
  it; if an intermediary is ever put in front of a registry, the header rule is a
  design of its own.
- **`/v1/health` is the JSON status document the proposal names, and it never answers
  non-`200`** (`state.rs::health`, served at `serve.rs::route`, answered at the exact path
  `/v1/health` and nothing else): the six fields are `status`/`mode`/`upstream`/
  `local_entries`/`cached_slugs`/`last_upstream_fetch`, and the `down` state is
  a body value, not a status code.

**Admin face and auth.**

- **A disabled admin face answers the same `404` an unknown path gets,** for every
  dispatch outcome, so a `PUT` to `/v1/admin/games` on an instance whose face is disabled
  (a routable bind with no token, `026-C5`) is not a `405` with `Allow: POST`. Rationale:
  `026-C5` says the routes are absent when the face is off, and an `Allow` list is an
  existence oracle. The detail line says only
  that this instance's admin face is not enabled. On an *enabled* face the `405`/`Allow`
  rule is what the proposal asks for, except on a parameterized path asked with a method it
  does not carry, which is a `404` by the same never-an-oracle reasoning (proposal
  As-built D).
- **There is one listener, and the admin face is enabled when the address bound is
  loopback _or_ a token is configured** (`main.rs::serve_command` passes
  `bound.ip().is_loopback()` and the configured token to `admin::Admin::new`, whose rule
  is `Some(_) => true, None => bound_loopback`). Enabling is judged from the address
  actually bound, not from the `--host` spelling. **This is the proposal's §The admin face
  and auth rule as approved, not a deviation, and an earlier revision of these notes had
  it wrong in two ways** (as-built pass, 2026-11-19): it said "enabled exactly when the
  address bound is loopback", and its *spec ↔ code* row named the credential
  `X-BindForge-Token`. The code reads the `Authorization` header's value after `Bearer `
  (`admin.rs::admits`) and answers a refusal with `WWW-Authenticate: Bearer`; a configured
  token is required on every admin request regardless of bind
  (`admin_auth::a_configured_token_is_required_on_every_admin_request_including_loopback`),
  and it is what opens a routable bind. Consequence, correctly stated: `--host 0.0.0.0`
  **without** `--admin-token` has no admin face at all
  (`admin_auth::a_non_loopback_bind_without_a_token_disables_the_face`); the same bind
  **with** a token is the enabled-and-token-required case the design resolves open Q11 to.
  Rationale for the residual risk, which the design accepts: a token over plain HTTP is
  link-visible, and that deviation from the scheme it names is `026-R1`'s, deferred on
  purpose (A20).
- **Any admin request carrying an `Origin` is a named `400`**, whatever the value,
  once the route, method and credential pass (`serve.rs::admin_gate`). This face
  serves an operator and their tools; a browser is not a client it can authenticate,
  and the face answers no preflight (see *The serving face*), so there is no
  legitimate `Origin` to allow-list. The refusal is checked on the header block, so
  the request's body is never read.
- **A browser-shaped admin write is refused before its body is read: `Origin` is the
  named `400`, a non-JSON `Content-Type` is the named `415`.** The security seat's fix
  asked for both halves. The `Origin` half is the flat refusal this face can afford:
  every method it carries is non-`GET`, a cross-origin or DNS-rebounded request always
  names its origin, and no preflight is answered here, so there is no legitimate
  `Origin` to allow-list. The media-type half needed a shape decision, because the
  proposal names no `Content-Type` rule at all: `serve.rs::admin_gate` refuses an
  upsert whose declared type is anything but `application/json` (`415`, decided on the
  header block, parameters after the `;` ignored), and answers a request that declares
  no `Content-Type`. Rationale: the request the rule exists to stop is one a browser
  can forge without a preflight, and every such request *does* carry the header — an
  HTML form produces exactly `text/plain`, `application/x-www-form-urlencoded` or
  `multipart/form-data`, and a cross-origin `fetch` that sets `application/json` has to
  preflight, which this face never answers. Requiring the header would add a refusal no
  contract names for a tool that needs no media type to describe a JSON body, and it
  would move every committed admin-mutation leg, each of which posts with no
  `Content-Type` at all; narrowing it to *declared-but-wrong* closes the forged write
  without inventing an obligation. The rule guards the one route that parses a body
  (`POST /v1/admin/games`): a `Content-Type` on `GET /v1/admin/stats` or a `DELETE` is
  not a payload claim and is not asked about. Decision: keep the narrow rule; the
  refusal legs it needs are *Test updates owed* item 6.

**Upstream client.**

- **`--upstream` admits any host, and `upstream::fetch` resolves names with the OS
  resolver (`to_socket_addrs`).** That is `026-L1` and A11 as §Caching mode writes
  them — "a literal `http://` URL with a parseable host+port (any host)" — copied
  into `endpoint.rs` from `apps/discovery-cli/src/catalog.rs`, which this branch
  already amended. Consequence, stated plainly: nothing here stops a configured
  upstream from naming a host on the operator's private network, and a name whose
  answer changes between lookups (DNS rebinding) is not detected. The gate that
  does exist refuses a non-`http` scheme, credentials in the authority, an
  unparseable host, a non-decimal port, and drops any path from the normalized
  base. Decision: keep — the admission is the unit's rule, and the trust question
  is `026-R1`'s, which the proposal defers on purpose.
- **One `GET` per connection with `Connection: close`, no credential sent, no
  `Location` followed** (a `3xx` is an upstream error → `stale`/`503`), per
  §Caching mode's disclosure model; `UPSTREAM_CONNECT_TIMEOUT`/`UPSTREAM_READ_TIMEOUT`
  are the `2 s`/`5 s` the same section names.

**Persistence.**

- **A save is `owner_only_new_file` (`create_new`) → `write_all` → `sync_all` →
  rename, and a failed save leaves the previous document in place.** The proposal's
  failure-modes bullet asks that "a mid-write failure leaves the previous file
  intact", which this delivers. The `sync_all` before the rename is this round's
  addition, beyond the shipped precedent: `apps/discovery-cli/src/matches.rs` renames
  a tmp file it never fsyncs, so the copied discipline closes an accepted-mutation
  loss the registry's own admin answers would otherwise be liable for. The parent
  directory is deliberately not synced — std has no portable directory handle, and a
  rename lost that way leaves the previous document whole.

**Budgets.**

- **`MAX_RESPONSE_OCTETS` is now documented as the whole response budget and is
  enforced on the outbound answer, not only on the upstream read.** Rationale: the
  security seat's major finding is that the shipped client (A1) refuses a larger
  response than this instance was emitting, so the same figure has to bound what we
  write. `MAX_REQUEST_BODY_OCTETS` is unchanged in value (it *is* the catalog
  document cap) and its refusal predates this round — what moved is *when* a peer
  meets it, because the admin gate no longer runs behind the body read.
- **`UNREAD_BODY_DRAIN_OCTETS` and `UNREAD_BODY_DRAIN_TIMEOUT` are new constants**
  the proposal does not name. Rationale: the discard-after-refusal path needs a
  byte and a wall-clock bound of its own; borrowing either of the existing caps
  would tie an unrelated budget to it.
- **The cache document is serialized pretty, so `026-B2` binds an ordinary corpus
  before `026-B1` does.** The performance seat's minor finding 3 is correct: a
  pretty-printed one-candidate entry is ~250 octets, not the ~150 the proposal's
  §Budgets arithmetic assumes, so the 1 MiB octet cap starts evicting around 4,000
  entries instead of 5,000. Rationale for not changing the spelling in this round:
  the seat approved, and the two fixes it offers (compact `to_vec`, or restating
  the fit arithmetic) pull in opposite directions — compact output changes a
  file's on-disk shape that `tests/cache_document.rs` and `tests/data_layout.rs`
  read, and only the test engineer can re-pin that. Decision: leave the pretty
  spelling, record the arithmetic here and in *spec ↔ code diffs*, and let the next
  round choose. The seat's findings 2 (per-eviction re-serialize under the write
  lock) and 4 (duplicate clone and third read lock on a cold lookup) are the same
  seat's bounded nits and are likewise untouched.
- **The header reader still takes one octet per `read`, which is the seat's other
  half of finding 1.** The quadratic scan is gone — the terminator test is now one
  `ends_with` per octet — but a maximal 65,536-octet block still costs 65,536
  one-octet syscalls. Rationale for stopping there: the byte-at-a-time read is what
  makes the head/body boundary exact. Reading a 4 KiB chunk would pull body octets
  into the head reader, and `RequestHead::into_request` then `read_exact`s the body
  from the same stream, so correctness needs a buffered reader threaded through the
  head split this round introduced. That is a reader refactor with the framing
  contract in its blast radius, from a seat that approved; the amplification it
  removes is bounded by `MAX_REQUEST_HEADER_OCTETS` and by
  `MAX_CONCURRENT_CONNECTIONS`, and the low-and-slow wall-clock bound is
  `REQUEST_READ_TIMEOUT`.

**Test strategy (#1–#19).**

- **#6's fresh/stale legs are wall-clock legs and #6's probe clause is unasserted.**
  The proposal writes #6 as a lookup-policy leg; `tests/caching_policy.rs` currently
  spends 150 ms sleeps against a 100 ms TTL for the fresh/stale halves and never
  asserts that the `--refresh-interval` override re-times the probe its own name
  names. Both are the seat's findings 2 and 3, and both are test-owned files. That
  is their shape to change, not mine (see *Test updates owed* item 2).
- **§Budgets' two serving-side timeouts stay unlegged** (`REQUEST_READ_TIMEOUT`
  is 60 s, `RESPONSE_WRITE_TIMEOUT` is 15 s, applied to the socket in `http.rs`).
  Rationale for no leg: an honest leg waits out a real timeout, so the cheapest
  honest version costs 15 s of gate time, and the cheap version needs a
  `--request-timeout <tiny>` knob no contract describes — there is no such flag
  today, so writing that leg means adding CLI surface to satisfy a test. The
  behaviour the timeouts bound *is* covered: the head-only read, the bounded
  discard with its own 2 s bound, and the write-half shutdown all have legs.
  Decision: leave the two figures unlegged, recorded here and in the seat's nit 5.
- **#1's over-`MAX_CATALOG_DOCUMENT_OCTETS` refusal still ships no invalid
  fixture.** Rationale: the fixture would itself be 16,777,216 octets, and
  `crates/core/tests/common/mod.rs` enumerates everything under a `fixtures/` tree,
  so a file that size enters every sweep. The spec's §Named refusals roster now
  carries the condition with the no-fixture reason next to it, so the gap is stated
  where a reader meets it, and the refusal is pinned by `tests/catalog_document.rs`
  and `tests/admin_mutations.rs` instead.

**Rollout.**

- **Steps 3–5's task boundaries were not observed** — see the first *Scope* bullet,
  which is where the decision sits.
- **`Catalog::load_required` is what the corpus ordering asks of code.** §Placement
  says `--corpus` names a file that has to exist, so a root whose one content
  argument is missing refuses to start naming the path consulted, instead of
  serving an empty catalog while an operator believes the migration ran.
  Present-and-empty is still a valid corpus: that is a root with zero games.
- **No CI change, and none was asked for.** `.github/workflows/check.yml` runs the
  cargo trio over the workspace and the npm check, and `apps/registry` is a
  workspace member, so the new crate is already inside the gate.

## Checked risks

- **`corpus::load` and `corpus::load_required` are two names for one read.** Both
  are one-line wrappers over `load_at(path, cap, absent_is_empty)`, so the absent-file
  rule is the only thing that differs and there is no second parse path to keep in
  step. The risk they carry is the opposite one: a caller that picks the wrong
  wrapper silently adopts absent-is-empty for a document the operator named.
  `state.rs::State::root` is the only `load_required` caller, which is the whole
  blast radius.
- **`response_bounded` is a second budget check, downstream of `stage`'s
  truncation.** They check the same thing at different layers: `stage` bounds what
  can be stored, `response_bounded` bounds what is about to be written. Keeping
  both means a future cache change cannot silently reintroduce the socket-level
  truncation.
- **The discard after a refusal is best effort.** If a peer stops mid-body, the
  drain gives up at `UNREAD_BODY_DRAIN_TIMEOUT` and the connection is closed anyway;
  the refusal was already flushed. Nothing depends on the discard succeeding — the
  face answers one request per connection and sends `Connection: close`, so there is
  no next message to resynchronise with, and the unit leg pins only that the discard
  stops at its byte bound.
- **The admin head-gate reads the head with the same `REQUEST_READ_TIMEOUT` as
  the public face.** A client that sends a partial head holds a thread for that
  long. It was already the case for both faces before this round, and the
  improvement here is that a complete head is no longer forced to wait out a body
  it will never see.
- **The `Origin` gate reads a header a non-browser caller can forge, and the `415`
  reads one they can omit.** Neither is an authenticity check — together they are the
  documented refusal for the one shape a token gate cannot see: a request a *browser*
  produces, which cannot choose either header freely. The real defence stays the token
  and the loopback bind. Consequence to keep in mind: a hand-written client that sends
  `Content-Type: text/plain` with a JSON body is now refused, and the reason is posture,
  not parsing.

## spec ↔ code diffs

Two contract sources, and they cover different ground: `schemas/catalog/v1/spec.md`
is the **document** contract (shape, named refusals, the format's budget, the
answer's budget, placement); the **wire** behaviour of this service has no schema
document of its own and is fixed by the proposal's §The serving face and §The admin
face and auth, plus A1's byte shape and the committed suites. Rows are labelled
which of the two they come from. `02-proposal.md` carries its own dated **As-built**
section (entries A–M) for the drift against the approved design; where a row below is one
of those entries it is stated here in the working shorthand the seats used, and the
contract-level wording lives there.

| Statement | Code site | Status |
| --- | --- | --- |
| proposal §The serving face: `--host`/`--port`, port-0 bind, the printed address is the contract | `main.rs::Config::parse`, `main.rs::bind`, `serve_command` | implemented |
| proposal §The serving face: `GET /v1/candidates` joins from the load-built index, projected to `{game_id, name}`, stable truncation | `state.rs::State::candidates`, `corpus.rs::candidates_body` | implemented |
| proposal §The serving face: decode-then-validate the slug, inadmissible is `404` | `serve.rs::candidates_answer` with `keys::key_is_admissible` | implemented |
| proposal §The serving face: `GET /v1/health` always `200`, JSON status document, never fires the probe | `state.rs::State::health`, served in `serve.rs::route` | implemented |
| proposal §The serving face: anything else is `404`, a wrong method is `405` naming what it does carry | `serve.rs::route`, `http.rs::write_refused` | implemented |
| proposal §The serving face: no route answers a whole document (`026-L3`) | `serve.rs::route`, `admin.rs::dispatch` | implemented, and `026-L3`'s bulk paths are `404`s |
| proposal §The admin face and auth: enabled when the bound address is loopback **or** a token is configured; a configured token required on every admin request; routable bind with no token disables (`026-C5`) | `main.rs::serve_command`, `admin.rs::Admin::new` | implemented as designed — see *Deviations → Admin face and auth* for the correction this row carries |
| proposal §The admin face and auth: the `Authorization: Bearer <token>` credential, compared after the scheme token under a constant-time fold; `401` with `WWW-Authenticate: Bearer` and no token echo | `admin.rs::admits` (takes the `RequestHead`), `admin.rs::token_matches`, `serve.rs::admin_gate`, `http.rs::write_refused` | implemented, and the gate now runs before the body is read (an earlier row here named the header `X-BindForge-Token`; it was wrong) |
| proposal §The admin face and auth: a disabled face publishes nothing | `serve.rs::admin_gate` | implemented — one flat `404`, no `Allow` |
| browser-shape refusals this contract does not spell out, added for `026-C5`'s never-an-oracle posture and the plain-HTTP bearer risk (`026-R1`): an admin request carrying `Origin` is `400`, an upsert whose declared `Content-Type` is not `application/json` is `415`, both on the header block | `serve.rs::admin_gate`, `serve.rs::media_type_spells_json`, `http.rs::Refused::unsupported_media_type` | implemented — the `Content-Type` is optional, so no caller this contract describes is refused; see *Deviations → Admin face and auth* |
| proposal §The admin face and auth + §CLI amendment: upsert, delete, stats, `revision` bump, atomic persist | `admin.rs::handle_upsert`, `handle_delete`, `persist.rs::save` | implemented |
| proposal §Caching mode: TTL, dual write-time eviction, load re-check, cold `503`, once-per-TTL probe fired by lookups only | `cache.rs`, `state.rs`, `upstream.rs::answers_health` | implemented; the two timeouts named there (`2 s`/`5 s`) are `constants::UPSTREAM_CONNECT_TIMEOUT`/`UPSTREAM_READ_TIMEOUT` |
| proposal §Caching mode: `--upstream` admits a literal `http://` URL with any host+port (`026-L1`, A11) | `endpoint.rs::parse_endpoint`, copied into `apps/discovery-cli` | implemented — see *Deviations → Upstream client* for what "any host" costs |
| spec §Shape: closed entry set at document **and** record level | `corpus.rs::DOCUMENT_ENTRIES`, `RECORD_ENTRIES`, `from_document`, `record_from_value` | implemented this round, with `fixtures/invalid/record-entry-outside-closed-set.json` |
| spec §Named refusals: every row has a fixture except the over-cap document | `schemas/catalog/v1/fixtures/invalid/` | implemented — the roster says which row ships no fixture and why |
| spec §This format's budget: `fs::metadata`-gated load, measured post-bump write | `persist.rs::read_capped`, `state.rs`, `admin.rs` | implemented |
| spec §The answer's budget: `MAX_RESPONSE_OCTETS`/`MAX_CANDIDATES` inbound **and** outbound | `constants.rs`, `corpus.rs::response_bounded`/`candidates_body`, `cache.rs::stage`+`load`, `upstream.rs::read_response` | implemented this round |
| spec §Placement and the two documents: `--corpus` must exist; the caching pair is absent-is-empty | `corpus.rs::load_required` via `state.rs::State::root`, `persist.rs::read_capped` | implemented this round |
| spec §Placement and the two documents: tmp `create_new` → `sync_all` → rename; two writers on one corpus fail loudly | `persist.rs::save` | implemented, `sync_all` added this round |
| proposal §Budgets: request-side budgets (`MAX_REQUEST_HEADER_OCTETS`, the body cap) enforced before allocation | `http.rs::read_header_block`, `declared_body_octets`, `read_body` | implemented |
| proposal §The serving face step 2 + spec §The answer's budget, read across to the operator surface: `registry query` prints the projection the face answers with | `main.rs::query_command` prints `state.candidates(…)` whole (up to `MAX_CANDIDATES`); `corpus::response_bounded`'s octet trim is only on the `GET /v1/candidates` path | **open nit** — for a join the trim shortens, `query` lists more candidates than the face answers. Raised as an FYI by the testing (r1c1) and compatibility (r1c2) seats; the face's own cap is pinned, so this is a second-surface disagreement, not an unbounded answer |
| proposal: CORS headers, a browser client, an HTTP-cache freshness rule | — | not in either contract, and not implemented — see *Deviations → The serving face* |
| proposal §Rollout step 6: ledger annotations on `025-C4`, `025-L1`, `025-R1`, `026-S1` | `data/catalog/README.md` records the gap | not writable from a build stage — see *Deviations → Scope* |
| proposal §Rollout steps 3–5: the service lands as `T-004`…`T-008` | the whole `apps/registry` crate on this branch | diverged — see *Deviations → Scope*, decision owed to the operator |
