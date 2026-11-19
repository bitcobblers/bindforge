# Code review — security seat, round 1 pass c0

- Run: `026-registry-service`
- Seat: security (full panel, c0)
- Scope: `git diff 217c46fda9eb95230c8df62233eca8ef7b6ffb71...HEAD -- ':(exclude)docs/designs'`
- Inputs used: that diff, `docs/designs/026-registry-service/02-proposal.md`,
  `docs/adrs/026-registry-service.md`, the runner's gate record, my own previous
  findings (none — first pass). No other `reviews/` content was opened.
- Verdict: **request-changes** — 1 major, 4 minor, 1 nit.

## Gate evidence

The runner's `cargo test --workspace --locked` leg is green; its captured tail
(`test result: ok. 0 passed; 0 failed; …`) is the last (doc-test) leg and by
itself cannot establish that the new suites ran. I re-ran the project's read-only
check `cargo test -p bindforge-registry`: every suite passes with real counts
(`response_shape` 7 passed, `admin_auth`, `persist_and_concurrency` 5 passed,
`upstream_client` 6, `upstream_endpoint_gate` 3, …), so the coverage exists and
the "0 passed" tail is not a missing-test signal. No behavior finding below
rests on the gate record alone; each cites source.

## Checklist walk (`references/security.md`)

1. **External input validated at the boundary.** Largely met, one gap. The two
   JSON documents are load-validated and size-gated before read
   (`persist.rs:236-252` metadata gate; `corpus.rs:104-175`; `cache.rs:86-206`
   refuses wrong version, bad `slug`/`game_id`/`name`, repeated `slug`, and an
   out-of-window `fetched_at`); `record_from_value` is shared by load and admin
   upsert (`corpus.rs:292-346`); upstream ingest validates `game_id`/`name`
   before the cache write and drops refusals with one escaped echo
   (`upstream.rs:215-272`); nothing evaluates code. **Gap: the cache document's
   per-entry candidate list is not count-bounded at load, and no answer path
   bounds the outbound body — Finding 1.**
2. **Path handling.** Met. Keys are opaque in-memory strings, never joined onto
   paths; `key_is_admissible` (`keys.rs:72-94`) refuses a further `/` or `\`,
   `.`/`..` components, control bytes and DEL, over the 128-octet cap on the
   *decoded* bytes; the `DELETE` segment is re-checked through
   `game_id_is_valid` (`admin.rs:224-233`) and `dispatch` rejects any segment
   carrying a separator (`admin.rs:74-86`), so `%2e%2e%2f` reaches neither a
   document nor a path. Written paths are the fixed `<data>` joins plus the
   operator's `--corpus` (persist.rs:73-89, 183-218). No shell is spawned
   anywhere in product code; the tests use `Command::new` with argv
   (`tests/common/mod.rs:483,752,827`).
3. **Injection.** Met, with an availability caveat (Finding 5). No regex, no
   string-interpolated commands. The outbound request target is built only from
   the decoded slug through the copied `percent_encode`
   (`upstream.rs:52-56`, `keys.rs:52-63`), so no CR/LF can reach the request
   line; the `Host:` value comes from `parse_endpoint`, which refuses
   credentials and non-decimal ports (`endpoint.rs:63-118`); absolute-form
   targets are refused (`http.rs:175-185`), so the server cannot be aimed at a
   third host. Refusal bodies serialize through `serde_json` and echo only
   escaped, truncated values (`escape_and_truncate` at `admin.rs:230`,
   `corpus.rs:304/331`, `state.rs:497`); status-line reasons are all
   `&'static str` (`http.rs:310-329`, `serve.rs:236-243`).
4. **Secrets and PII.** Two leaks. The configured admin token never reaches a
   refusal body, the health surface, or the upstream request
   (`admin.rs:179-194`, `serve.rs:173-181`, `upstream.rs:14-15`), and the test
   harness strips the ambient token/data variables from every child
   (`tests/common/mod.rs:478-490`, `:673`). **Findings 2 and 4 are the two
   places it does move.**
5. **New dependencies.** Met, no advisory surface. `Cargo.lock` adds exactly one
   entry, the new `bindforge-registry` package itself, depending on
   `bindforge-core`, `discovery` and `serde_json` (`apps/registry/Cargo.toml:9-20`)
   — all already in the lock via the shipped client. No new third-party crate,
   so nothing to check against advisories.
6. **Filesystem writes.** Met, one durability nit (Finding 6). Owner-only
   `0700` parent, fixed tmp name opened `create_new(true)` at `0600`, mode
   re-asserted after rename (`persist.rs:131-218`), refuse-before-write on the
   document cap (`state.rs:456-479`), refuse-and-leave-untouched on load
   refusal, two-writers-on-one-corpus fails loudly. Least privilege holds.
7. **Clean rejection of malformed/hostile input.** Met on framing:
   `Transfer-Encoding`, duplicated/absent/non-decimal `Content-Length`,
   over-cap declared length and over-cap header block are refused with a
   well-framed `400` and a JSON body before any mutation
   (`http.rs:110-298`, `serve.rs:117-123`); no stack traces reach a peer; a
   disabled admin face answers `404`, never a `401` oracle
   (`serve.rs:172-181`). Not met at the response seam — Finding 1.

Additional posture that checks out: the token compare is a constant-shape fold
that never early-returns and never matches across a length difference
(`admin.rs:151-162`); a configured token is required on every carried admin
request regardless of bind, and a routable bind without one disables the face
(`admin.rs:111-118`, pinned by `tests/admin_auth.rs:188-226`); enablement is
judged from `local_addr()`'s `IpAddr::is_loopback`, not a spelling
(`main.rs:249-251`); the upstream client refuses `3xx` and never repeats a
`Location` (`upstream.rs:145-153`); stderr warnings all pass `echo_safe`, so a
hostile upstream cannot forge a log line (`upstream.rs:266-296`); `026-L3`
holds — no route answers a whole document and `stats` is three integers
(`admin.rs:59-94`, `state.rs:428-445`, `tests/response_shape.rs:139-159`); the
`026-L1` CLI amendment keeps the fail-closed pre-plugin connect and moves no
test onto a remote host (`apps/discovery-cli/src/catalog.rs` `probe()`, the
unit-only remote-admission case, and the subprocess list trimmed to the still
-refused spellings), with the `026-L2` disclosure added at the decision site;
tests never resolve or dial off-host.

## Findings

### 1. major — no outbound response budget: `MAX_CANDIDATES` is not applied to the cache answer path, and nothing caps a response body

**Location:** `apps/registry/src/http.rs:310-329` (`write_response`),
`apps/registry/src/state.rs:253-279` (cache-hit and stale answers served
whole), `apps/registry/src/cache.rs:136-171` (load bounds no per-entry
candidate list).

**Problem.** `constants.rs:31-33` states `MAX_CANDIDATES` is "the most
candidates one answer may carry, **in either direction**", and the proposal's
Goal ("exactly the response shape and budgets the shipped client pins (A1)",
02-proposal.md:21) plus §The serving face step 3 ("Response written with the
byte shape A1 pins") require the answer to stay inside A1's caps: body
`≤ MAX_RESPONSE_OCTETS = 65_536` and `≤ 256` candidates
(02-proposal.md:43, 252; Test strategy #16's "caps" clause at :556). Neither is
enforced on the outbound seam: `Catalog::candidates` truncates only the corpus
index (`corpus.rs:218-231`); the cache paths return `entry.candidates`
un-truncated (`state.rs:253-279`), `Cache::load` never bounds a per-entry
candidate list (`cache.rs:136-171`), and `write_response` measures nothing.
Two concrete failures: (a) a **contract-valid** corpus — `MAX_CANDIDATES`
records sharing one `source_key`, each riding the 128-octet `game_id` and
128-octet `name` bounds that `fixtures/valid/on-every-length-bound.json`
declares loadable — answers `256 × 280 + 257 = 71,937` octets, which
`apps/discovery-cli/src/catalog.rs::read_response` refuses as over-budget, so
the shipped CLI fails against a legal corpus and a caching instance's own
upstream reader turns a legal root answer into an upstream error and then a
cold `503`; (b) a cache document carrying thousands of candidates for one slug
is served whole, up to ~the 1 MiB `026-B2` file size, per request, repeatedly.
`spec.md:101-103` waves at exactly this ("the arithmetic is per record, not per
candidate list") without closing the response seam. `tests/response_shape.rs:236-239`
asserts the cap only against a three-record corpus, and
`tests/candidates_join.rs:135-178` truncates short-named records only, so no
test pins it.

**Fix.** Apply the bound on every answer path, not just the index: cap the
per-entry candidate list at the cache's write seam (`upstream.rs` already
refuses `> MAX_CANDIDATES`) *and* at `Cache::load`, and refuse a cached entry
over the cap like every other load violation; then measure the serialized
candidate body against `MAX_RESPONSE_OCTETS` in `candidates_answer`/
`write_response` and answer a named refusal (or truncate the stable
first-`MAX_CANDIDATES` window further) rather than emit a body the pinned
reader rejects. Add a wide corpus at the 128/128 bounds to
`tests/response_shape.rs`, plus a cache entry over `MAX_CANDIDATES` to
`tests/cache_document.rs`.

### 2. minor — the bearer token is echoed into a stderr line by the CLI's unknown-flag refusal (`--admin-token=…`)

**Location:** `apps/registry/src/main.rs:165-170` (with the flag arm at `:154`).

**Problem.** Only the space-separated spelling is recognized, so the common
`--flag=value` form falls into the unknown-flag arm and prints
``error: `--admin-token=S3CRET` is not a flag this registry has`` (main.rs:166-169,
via `main.rs:65-68`). The configured credential then lands in terminal
scrollback, journald, or CI logs from a single typo — checklist 4's "none in …
log lines". No test covers this spelling.

**Fix.** Split each argument on its first `=` and match the name (so
`--admin-token=` works or is refused by name), and echo only the flag name in
that refusal — never the value. Pin with a unit case asserting the refusal
line does not contain the value.

### 3. minor — the full 16 MiB request-body budget is consumed before the admin credential is checked

**Location:** `apps/registry/src/http.rs:234-260` (called at `:161`), with
`constants.rs:24-29` and `constants.rs:56`.

**Problem.** `read_request` reads the declared body before routing, and the
enablement/`admits` checks happen only afterwards (`serve.rs:162-191`). An
unauthenticated peer therefore does not need a valid token to make the server
declare-and-buffer `MAX_REQUEST_BODY_OCTETS` = 16 MiB per connection — one per
admitted slot, so ~1 GiB of buffered body per burst of
`MAX_CONCURRENT_CONNECTIONS` = 64 sockets — and then get a `401`. The proposal
chose the two budgets separately (§Budgets, 02-proposal.md:400-418) and never
took their product as a pre-credential allowance.

**Fix.** Read the header block first and, for `/v1/admin/**`, decide
`is_enabled`/`admits` from the already-complete headers and answer `401`/`404`
before reading the declared body (close the connection instead of draining it);
or give the admin upsert its own body cap well below the document cap and keep
16 MiB as a load-side figure only. Pin with a `tests/admin_auth.rs` case
asserting an unauthenticated oversized `POST` is refused without its body being
consumed.

### 4. minor — the token-less loopback admin face accepts preflight-free browser requests

**Location:** `apps/registry/src/admin.rs:111-118` and `:210-222`
(no `Content-Type` requirement), `apps/registry/src/serve.rs:162-194`.

**Problem.** With the defaults (`--host 127.0.0.1`, no token) the admin face is
enabled and `admits` returns `true` for a request carrying no credential
(`admin.rs:134-136`). `POST /v1/admin/games` parses its body as JSON whatever
its `Content-Type` (`admin.rs:210-217`) and nothing inspects `Origin` or
`Sec-Fetch-Site`, so a simple cross-origin request — which needs no CORS
preflight this server would have to answer — carries a full record. A page in a
browser on the operator's own machine, DNS-rebound to `127.0.0.1`, can upsert
into a caching instance's local-entries document; by `026-L5` such an entry
answers with zero upstream requests, is never merged and never expires, so a
single blind write pins an attacker's `game_id` for a lookup key the user's
`discovery list` will later trust. The only current mitigations are the
ephemeral default port and the absence of a readable response. This is
hardening at the implementation seam, not a challenge to `026-C5`'s
loopback/token rule.

**Fix.** Keep the posture, close the browser path: require
`Content-Type: application/json` on `POST /v1/admin/games` (a non-simple type,
so a cross-origin caller must preflight and this server never answers an
`OPTIONS`), and answer a carried admin request that carries an `Origin` header
with the named `400`. Pin both in `tests/admin_auth.rs`.

### 5. minor — the header-block cap is a CPU amplifier: the request reader rescans the whole buffer for every octet

**Location:** `apps/registry/src/http.rs:262-305` (`find_header_end`, `:300-305`).

**Problem.** `read_header_block` reads one octet at a time and calls
`find_header_end(&buffer)` — `bytes.windows(4).position(...)` from offset `0` —
after every octet. A maximal `MAX_REQUEST_HEADER_OCTETS` = 65,536 block costs
~2×10⁹ 4-byte comparisons, i.e. seconds of CPU for 64 KiB of input, per
connection, and the 60 s `REQUEST_READ_TIMEOUT`
(`constants.rs:67-71`) lets the peer hold the admitted slot for it. That turns
the design's own inbound budget into the cheapest available network-to-CPU
amplification and is exactly the low-and-slow/one-inbound-per-outbound exposure
the §Risks row ("Low-and-slow … `MAX_REQUEST_HEADER_OCTETS`",
02-proposal.md:456) claims those budgets bound. The crate's own reader already
does the linear thing (`upstream.rs:131` uses `head.ends_with(b"\r\n\r\n")`).

**Fix.** Test only the last four octets per iteration
(`buffer.ends_with(b"\r\n\r\n")`), or resume the scan from
`buffer.len() - 4`; keep the existing cap check and add a bounded-time
assertion to `tests/persist_and_concurrency.rs`' budget case.

### 6. nit — the atomic persist never fsyncs, so an acknowledged mutation can still be lost

**Location:** `apps/registry/src/persist.rs:194-218`.

**Problem.** `save` does `write_all` → `flush` → `owner_only` → `rename`, with
no `sync_all()` on the tmp handle (and no parent-directory sync). On a
filesystem that does not order data before the rename's commit, a power loss
can leave a zero-length or stale document for a mutation the admin face already
answered `200` for — weakening §Concurrency's "a restart cannot lose an
accepted mutation" (02-proposal.md:315-326) and checklist 6's "atomic where
partial writes would corrupt state". It matches the shipped `matches.rs`
precedent, so this is a nit rather than a regression.

**Fix.** `handle.sync_all()` before dropping the handle (no-op branch on
non-unix), optionally sync the parent after the rename, and pin the call in
`tests/persist_and_concurrency.rs`; or state the durability limit in
`schemas/catalog/v1/spec.md`'s persistence paragraph.

## FYI (out of specialty, no severity)

This branch ships the whole `apps/registry` binary, the `schemas/catalog/v1`
format and the `026-L1` CLI amendment, while 02-proposal.md's Non-goals
(:30-32) and Rollout (:575-604) state that this run ships "no product code" and
lands them as `T-004`…`T-008` — a scope/rollout divergence for the
compatibility/complexity seats to weigh, not a security one.
