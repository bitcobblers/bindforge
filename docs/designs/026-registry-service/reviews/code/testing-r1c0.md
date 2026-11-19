# Code review — testing seat — round 1, pass c0

- Design: `docs/designs/026-registry-service/02-proposal.md` (§Test strategy #1–#19)
- Scope: `git diff 217c46fda9eb95230c8df62233eca8ef7b6ffb71...HEAD -- ':(exclude)docs/designs'`
- Checklist: `references/testing.md` (code testing checklist), this seat only
- Verdict: **request-changes** (1 major, 2 minor, 3 nit)

## Inputs used

The diff (63 files, +12,163/−92), `02-proposal.md`, `docs/adrs/026-registry-service.md`,
the runner's gate record, and `.github/workflows/check.yml` (read only to establish
which legs CI itself runs). No review report or transcript was opened.

## Gate evidence (checklist item 6)

Supplied, by the runner at 2026-11-19T21:53:05Z — all four legs green:
`npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked`.

The only `test result:` line the record quotes is
`ok. 0 passed; 0 failed; 0 ignored` — an empty leg (a `src/lib.rs` unit-test or
doc-test target), so the supplied record does not by itself show the new suites
ran. That is a renderer truncation, not a red gate. To resolve the coverage
question I re-ran the required command myself:

- Executed: `cargo test --workspace --locked` → exit 0, every leg `test result: ok`.
  `apps/registry` contributes **60 unit tests** (`src/*.rs`) plus **101 integration
  tests** across 15 `tests/*.rs` targets (`admin_auth` 9, `admin_mutations` 9,
  `cache_document` 7, `caching_policy` 11, `candidates_join` 7, `catalog_document` 13,
  `cli_end_to_end` 7, `data_layout` 5, `health_route` 4, `persist_and_concurrency` 5,
  `response_shape` 7, `slug_decoding` 7, `upstream_client` 6,
  `upstream_endpoint_gate` 3, `zz_exec_probe` 1); the amended
  `cli_catalog_gate_tests` leg is 7 green; `crates/core/tests/conformance.rs` is 9
  green with `catalog` in the walk, and `version_gate.rs` is 11 green (A10 holds).
- No leg failed; no test is `#[ignore]`d anywhere in the diff.
- Recorded for the #19 platform leg: `.github/workflows/check.yml:31-32` runs
  `rustup target add x86_64-pc-windows-msvc` + `cargo check --workspace
  --all-targets --target x86_64-pc-windows-msvc`, so the "Windows no-op path
  compiled" claim has a real CI leg — but it is **not** one of the runner's gate
  legs, so the gate record alone cannot establish it. Evidence noted, not a finding.

## Checklist walk

1. **Every numbered Test-strategy item cited** — pass, with one partial. Unit and
   integration citations are present and specific: #1 `catalog_document.rs:5`,
   #2 `candidates_join.rs:5`, #3 `slug_decoding.rs:5`, #4 `cache_document.rs:4`,
   #5 `cache_document.rs:334/357/422`, #6 `caching_policy.rs:4`, #7
   `upstream_client.rs:6`, #8 `upstream_client.rs:232`, #9 `admin_auth.rs:5`, #10
   `admin_mutations.rs:6`, #11 `persist_and_concurrency.rs:9`, #12
   `health_route.rs:4`, #13 `data_layout.rs:8` + `src/persist.rs:18`, #14
   `upstream_endpoint_gate.rs:9`, #15 `apps/discovery-cli/tests/cli_catalog_gate_tests.rs:15/112`
   + the new unit pin `the_amended_policy_admits_every_parseable_http_host`
   (`src/catalog.rs`), #16/#17 `cli_end_to_end.rs:6` + `response_shape.rs:5`, #18
   `response_shape.rs:136` + `admin_auth.rs:351`, #19 by the `cfg(unix)`/`cfg(windows)`
   split in `data_layout.rs` plus the CI cross-target check. #6's last clause
   ("re-times the once-per-TTL upstream probe") is named but not asserted —
   finding 3.
2. **Both layers** — pass. In-module `#[cfg(test)]` units (grammar, key rule, fold,
   framing, TTL arithmetic, data-dir rule through the `DataLayout` env closure, which
   runs the Windows branch on a Linux host: `src/persist.rs:316`) plus subprocess
   integration over raw loopback sockets and the real `discovery` binary.
3. **Revision-round regressions** — not applicable: round 1, pass c0, no previously
   rejected blocker/major to re-pin. The proposal's r2–r4 self-citations (#5, #7,
   #10, #11, #13, #15, #19) are each landed as real assertions: eviction reads the
   renamed buffer's length (`cache_document.rs:427`), the copied `percent_encode`
   builder is pinned outbound (`upstream_client.rs:190`, `slug_decoding.rs:257`), the
   422 is checked on the post-`revision`-bump buffer with "no tmp written"
   (`admin_mutations.rs:197` + `data.files_below`), persist-under-one-lock is raced
   with 12 upserts (`persist_and_concurrency.rs:269`), the CLI suite keeps only the
   refused spellings and re-pins the refusal line off `loopback`
   (`cli_catalog_gate_tests.rs:112`).
4. **Determinism** — pass except finding 2. Every server is a `registry` subprocess
   on `127.0.0.1:0` with the port read from its own printed line
   (`tests/common/mod.rs:504`); every upstream is an in-process `TcpListener` fake or
   a bound-and-closed loopback port; no test resolves or dials a non-loopback host
   (the remote spellings in the amended CLI unit test are string-only, and
   `upstream_endpoint_gate.rs:120` deliberately pins "any host" with `127.0.0.5` and
   `localhost` for exactly that reason); temp dirs are pid+nonce+nanos; ambient
   `XDG_DATA_HOME`/`HOME`/`APPDATA`/token vars are removed from every child
   (`AMBIENT_ENV_VARS`); no fixed port, no sleep longer than 1.5 s anywhere except
   finding 2's pair.
5. **Behavior-describing names** — pass. Names state the rule
   (`a_local_entry_answers_without_a_single_upstream_lookup`,
   `connections_past_the_admit_ceiling_are_closed_with_no_status_line`,
   `a_future_dated_entry_is_expired_rather_than_immortal`), not steps. The one
   name that overstates its body is finding 3. `tests/zz_exec_probe.rs` is an
   ordering-style file name but a real precondition test with a behavior name.
6. **Gate evidence** — assessed above; all required legs green, both as supplied and
   as re-executed. No finding on the gates.
7. **Untestable additions** — findings 1, 5, 6. Every other budget has a leg:
   `MAX_CACHE_ENTRIES`/`MAX_CACHE_DOCUMENT_OCTETS` (`cache_document.rs:361/427`),
   `MAX_CATALOG_DOCUMENT_OCTETS` at both ends (`catalog_document.rs:337`,
   `admin_mutations.rs:197`), `MAX_REQUEST_HEADER_OCTETS`/`MAX_REQUEST_BODY_OCTETS`
   before any mutation (`slug_decoding.rs:181`, `persist_and_concurrency.rs:345`),
   `MAX_CONCURRENT_CONNECTIONS` (`persist_and_concurrency.rs:430`), `MAX_CANDIDATES`
   both directions (`candidates_join.rs:135`, `upstream_client.rs:46`).

## Findings

### 1. major — nothing pins the answer against A1's 65,536-octet cap, and the serving face does not apply it

- Location: `apps/registry/tests/response_shape.rs:236` (the whole "caps" leg of Test
  strategy #16); `apps/registry/src/corpus.rs:218` (`candidates()` bounds the join by
  count only); `apps/registry/src/constants.rs:15` (the constant, used only by
  `src/upstream.rs:134/193`).
- Problem: #16 promises "response shape pinned byte-exact to A1 (single decimal
  `Content-Length`, no `Transfer-Encoding`, **caps**)", and §The serving face states
  the requirement plainly: "wire traffic is per-slug candidate lists **within `025`
  budgets**". A1's reader caps header block *and* body at 65,536. The outbound face is
  bounded only by `MAX_CANDIDATES = 256`, and every field in the projection is itself
  bounded at 128 octets (`024-B1`, `013-B4`) — a bound the shipped fixture
  `schemas/catalog/v1/fixtures/valid/on-every-length-bound.json` proves is inclusive.
  256 records at both bounds serialize to ≈256 × 280 + separators ≈ 72,000 octets,
  over the cap; the crossover is around 234 candidates. The only assertion that even
  reads the octet bound (`response_shape.rs:236`) runs against five tiny answers, so
  the property is unestablished — and `MAX_RESPONSE_OCTETS` is declared with a
  doc-comment claiming the header block and body "are each capped at it" while no
  outbound code applies it. Concrete failure: a legal corpus whose key is claimed by
  ~250 games with max-length names makes `discovery list --catalog-url` against this
  root fail as `catalog unreachable`, and makes a caching instance refuse its own
  root's legal answer as an upstream error (`src/upstream.rs:193`), i.e. `026-C1`'s
  byte-identical face stops serving the client that pins it.
- Fix: add the missing leg — a root-mode corpus of `MAX_CANDIDATES` records at the
  `game_id`/`name` bounds sharing one source key, asserting the answer's framing
  satisfies the client (body ≤ `MAX_RESPONSE_OCTETS`), which will fail until the
  outbound projection is bounded in octets (truncate the stable window further, or
  answer a named refusal) and `MAX_CANDIDATES` truncation alone stops being treated as
  sufficient. Either way the arithmetic must be stated in `schemas/catalog/v1/spec.md`
  next to the cap, and the same bound re-used by the upstream client's read.

### 2. minor — two #6 legs rest on a one-second wall-clock window that spans subprocess startup

- Location: `apps/registry/tests/caching_policy.rs:151-178`
  (`a_fresh_cache_entry_answers_without_an_upstream_lookup`) and
  `:326-364` (`the_refresh_interval_override_ages_a_fresh_entry_and_retimes_the_probe`).
- Problem: both seed `fetched_at = now_unix()` *before* the server subprocess is
  started, run with `--refresh-interval 1`, then assert the *fresh* behaviour — that
  the cache is served and that zero candidate requests reach the fake. The freshness
  budget is therefore 1 s of wall clock covering spawn, the printed-port poll
  (`tests/common/mod.rs:507`) and the socket round trip. On a loaded runner those
  steps can exceed 1 s, at which point the entry is legitimately stale, a refresh
  fires, the fake answers its `[]` default, and the assertions fail for a reason that
  has nothing to do with the code under test. Checklist item 4 asks for tests with no
  wall-clock dependence; the stale legs (seeded `now − 5`, `now − 100_000`) get this
  right, only the fresh legs do not.
- Fix: capture the stamp after `Registry::caching(...)` returns (seed the document
  before start, or re-seed with a stamp taken once the listener answers), and let the
  fresh leg use the default `026-B3` TTL so "inside the TTL" is not a race; keep
  `--refresh-interval 1` for the legs that need expiry. If the sleep at `:349` stays,
  give it margin (the 1.5 s wait itself is safe — it can only overshoot).

### 3. minor — #6's "re-times the once-per-TTL upstream probe" clause is named in a test name and doc but never asserted

- Location: `apps/registry/tests/caching_policy.rs:326` (name and doc-comment claim
  re-timing; the only count assertion, `:358`, is on `GET /v1/candidates`);
  `apps/registry/tests/upstream_client.rs:343` counts probes only under the default
  TTL, proving "at most one per TTL", never that a second one is paid after expiry.
- Problem: every probe count in the suite is a single-TTL measurement, so a
  `registry` that cached the reachability verdict forever — never re-probing — passes
  every leg here. §Caching mode ("probed at most once per TTL, fired lazily by the
  first *lookup* after expiry") and Test strategy #6's last clause both name the
  re-timing, and this test is the one the suite says covers it.
- Fix: in the same test, count `upstream.lines_for("GET /v1/health")` before the wait
  and after the second lookup and assert it grew by exactly one probe (the fake
  already distinguishes routes, so this is one line); keep
  `upstream_client.rs:314` for the "at most one" half.

### 4. nit — #1's over-`MAX_CATALOG_DOCUMENT_OCTETS` condition ships no invalid fixture

- Location: `schemas/catalog/v1/fixtures/invalid/` (17 fixtures, none over-cap);
  Test strategy #1 says each listed condition "ships in `fixtures/invalid/*.json` as a
  named refusal".
- Problem: the refusal is genuinely tested (`tests/catalog_document.rs:337`, built in
  a temp dir), so this is a contract-artifact gap, not a behavior gap: an independent
  implementer running only `schemas/catalog/v1/fixtures/` never sees the document-cap
  refusal, which is one of the two ends §Budgets says the cap is enforced at. A 16 MiB
  golden fixture is not worth committing either.
- Fix: state the deviation where an implementer will read it — name
  `MAX_CATALOG_DOCUMENT_OCTETS` in `schemas/catalog/v1/spec.md`'s refusal roster with
  "no fixture: the cap is 16,777,216 octets; the refusal is pinned by the loader suite
  at `apps/registry/tests/catalog_document.rs`" — so the fixture roster and the
  contract roster are reconcilable.

### 5. nit — the two request/write timeouts have no leg

- Location: `apps/registry/src/constants.rs:71-75`
  (`REQUEST_READ_TIMEOUT` 60 s, `RESPONSE_WRITE_TIMEOUT` 15 s), used at
  `src/http.rs:386/392`.
- Problem: §Risks names "request-read **and write** timeouts" as the low-and-slow
  mitigation, and they are wired — but nothing observes them, because their values
  make any test wait a minute. Checklist item 7 treats an unobserved knob as a finding.
- Fix: expose the read timeout through the same private-knob seam as
  `--refresh-interval` (e.g. an undocumented `--request-read-timeout <seconds>` for
  tests, or a `constants` override in `serve.rs`'s unit tests) and add a leg to
  `persist_and_concurrency.rs`'s budget test: a connection that sends a partial
  request gets closed. Otherwise record the knob as deliberately unlegged in
  `03-build-notes.md`.

### 6. nit — an explicit `--port <N>` is parsed but never bound

- Location: `apps/registry/src/main.rs:144` (parse + the `not a port number` refusal,
  both unit-pinned at `main.rs:342/374`); no integration leg starts an instance on a
  non-zero port.
- Problem: §CLI surface makes port *0* the contract ("the printed value is the
  contract"), and that is exercised everywhere; the other value of the same flag is
  only exercised as a string. A bound-but-not-printed, or printed-but-not-bound,
  explicit port would pass the suite.
- Fix: one cheap leg in `health_route.rs` or `response_shape.rs`: start with
  `--port <a port learned from a throwaway bind>` and assert the printed port equals it
  and that health answers there.

## FYI (out of this seat's specialty, no severity)

The proposal's Context and §Non-goals state this run ships "the contract and the
ledger annotations and **no product code**", with the binary and the CLI amendment
deferred to `docs/tasks/T-004`…`T-008` — none of which exist in `docs/tasks/` on this
branch — while the diff under review is exactly that product code (~12,000 lines of
`apps/registry` plus the `026-L1` CLI amendment). The tests are written against the
shipped code rather than against the tasks, which is the better outcome for this
seat's checklist, but the scope statement and the change have diverged.
