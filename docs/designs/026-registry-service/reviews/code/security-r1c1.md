# Code review — security seat, round 1 dissent pass (c1)

- **Run**: 026-registry-service
- **Seat**: security (checklist `.opencode/skills/code-review/references/security.md`)
- **Scope as given**: `git diff 06d528f..HEAD -- ':(exclude)docs/designs'` (20 files,
  +1777/−155), read against `docs/designs/026-registry-service/02-proposal.md` and
  `docs/adrs/026-registry-service.md`.
- **Pass type**: dissent. Scope of this pass is first my own round-1 findings; new
  findings only where the changed code makes them true.
- **Inputs used**: that diff, the proposal, the ADR, the runner's gate record (green at
  2026-11-19T23:01:03Z: `npm run check`, `cargo fmt`, `cargo clippy -D warnings`,
  `cargo test --workspace --locked` all exit 0), and my round-1 findings as quoted by
  the caller. No review report or transcript was opened.
- **Verdict**: **approve** — 0 blocker, 0 major, 1 minor, 2 nit.

## 1. Round-1 findings, checked one by one

### 1.1 major — no outbound response budget (was: `http.rs` write path, `state.rs`, `cache.rs`) — **RESOLVED**

The octet cap now binds at the answer seam, and it binds for *every* answer path, not
just the corpus one:

- `corpus.rs:68-90` adds `response_bounded`, which walks the postings in `game_id`
  order and keeps the largest prefix whose serialized cost
  (`2` brackets + each object + `,` separators, measured by `Candidate::response_octets`,
  `corpus.rs:59-65`) fits `MAX_RESPONSE_OCTETS`. The accounting matches the body the
  seam emits (`corpus.rs:92-100`, `serde_json::to_vec` of the array), so the length
  test is exact, not an estimate. A candidate whose cost cannot be measured yields
  `usize::MAX` and stops the window — fail-small, never fail-wide.
- `serve.rs:294-301` (`candidates_answer`) now answers
  `Sent::Json(crate::corpus::candidates_body(&candidates))`. Because every answer —
  corpus join, cache hit, stale-cache answer, local entry, upstream fetch — arrives
  through `Answer::Candidates` at that one seam, this is the single choke point the
  round-1 fix asked for: the cache-hit and stale paths can no longer emit an entry list
  whole, and `write_response` no longer needs to measure anything the seam already
  bounded.
- The count cap is enforced at all three outbound-feeding seams, as
  `constants.rs:34-39` now states: the index truncation (`Catalog::candidates`), the
  cache **write** seam (`cache.rs:244-250`, `candidates.truncate(MAX_CANDIDATES)` in
  `stage`), and the cache **load** seam (`cache.rs:141-152`, an entry over the cap is a
  named load refusal alongside the other violations). `upstream.rs`'s existing inbound
  refusal is unchanged, so an over-cap list can no longer enter the document at all.
- Pinned three times over: `corpus.rs` unit
  (`a_wide_answer_is_trimmed_to_the_stable_prefix_the_pinned_client_accepts`, asserting
  the unbounded body really is `> MAX_RESPONSE_OCTETS` first, so the leg cannot pass
  vacuously), `tests/candidates_join.rs:184-273` (subprocess seam: `200`, body
  `≤ 65_536`, exactly one decimal `Content-Length` equal to the bytes written — i.e.
  trimmed before serialization, not truncated at the socket — a `game_id`-ordered stable
  prefix, byte-identical on a repeat request), and `src/wire.rs:606-660`.
- `schemas/catalog/v1/spec.md` gains §The answer's budget, which states the both-ways
  cap and the arithmetic the count cap misses (≈283 octets/candidate, ≈72,000 at 256,
  crossover near 231), and records the cache seams. The spec's "a non-empty join always
  keeps at least one" claim holds: the worst case is a 128-octet control-byte `name`
  escaped to ≈768 octets plus a 128-octet `game_id`, ≈920 octets — far below 65,536 —
  and the trim can never turn a join into the `026-L4` `503`, which stays caching-only.

The round-1 consequence (a contract-valid 128/128-bound corpus answering 71,937 octets,
which `discovery-cli`'s `read_response` refuses, turning a legal root answer into
`catalog unreachable` for the CLI and an upstream error → cold `503` for a caching
instance) is closed: the same corpus now answers a trimmed `200`.

### 1.2 minor — the unknown-flag refusal echoed the configured token (`main.rs`) — **RESOLVED** for the named spelling

`main.rs:138-141` splits `--flag=value` (only when the name before `=` starts with `--`),
so `--admin-token=S3CRET` is parsed as the flag it spells instead of falling into the
unknown-flag arm; `main.rs:183-190` quotes only `name`, so whatever followed an
unrecognized `--flag=` is never repeated back. `--root=` is refused without echoing its
value (`main.rs:153-158`), and a slug holding `=` stays a positional
(`main.rs:191-196`). `an_equals_form_takes_its_value_and_a_refusal_never_quotes_one`
asserts the refusal line contains `--nope` and **not** `s3cret`/`also-secret`, which is
the pin I asked for. `Admin::new` filters an empty token (`admin.rs:141`) and the empty
inline form degrades to the design's own loopback-open posture, not to a bypass. Residual
one spelling over: finding **F-2**. No other site prints the token
(`admin_token` is read at `main.rs:172`, consumed at `main.rs:271`; `Config`'s
`#[derive(Debug)]` is never printed).

### 1.3 minor — the body was read before the credential was checked — **RESOLVED on the admin face**

`http.rs` now splits the pass: `read_request_head` (`http.rs:180-236`) parses the request
line, the headers (with the `Transfer-Encoding`, duplicate-field and length-decimal
refusals in place) and the target, and nothing else; `read_body` moved behind
`RequestHead::into_request` (`http.rs:76-81`). `serve_one_connection` (`serve.rs:118-138`)
routes `/v1/admin/**` into `admin_connection` **from the head alone**, and `admin_gate`
(`serve.rs:212-268`) decides dispatch → enablement → `admits` → the two browser-shape
refusals with no octet of body buffered; `admin::admits` takes the `RequestHead`
(`admin.rs:161-175`). A refused admin request gets `http::drain_unread_body`
(`http.rs:379-399`), which throws away at most `UNREAD_BODY_DRAIN_OCTETS` = 65,536
octets into a 4 KiB scratch (never buffers), clamps the discard to the declared length so
it cannot swallow a pipelined follow-on, and is bounded by
`UNREAD_BODY_DRAIN_TIMEOUT` = 2 s. Pinned at `tests/admin_auth.rs`
(`an_unauthenticated_admin_request_is_refused_before_its_declared_body_arrives`: declares
1 MiB, sends nothing, must get the `401` + `WWW-Authenticate` in under 15 s with the
corpus untouched) and `src/wire.rs:762-793`. The flat-`404` hardening is a bonus for the
`026-C5` never-an-existence-oracle rule: a disabled face now answers the same `404` for
every dispatch outcome and publishes no `Allow` list (`serve.rs:213-223`,
`admin.rs:13-25` docs, pinned in both suites).

The same budget on the **serving** face is still un-gated — finding **F-1**.

### 1.4 minor — browser path into the admin face (DNS rebinding / form forgery) — **RESOLVED**

Both halves of the fix are in, decided on the header block:

- **`Origin`**: a carried admin request that carries `Origin` is the named `400`
  (`serve.rs:232-242`), checked after `admits`, so it is never an oracle about the
  credential. `is_admin_path` covers the whole face, so it guards `POST` upsert,
  `DELETE` and `GET stats`.
- **`Content-Type`**: `POST /v1/admin/games` declared as anything but `application/json`
  (parameters after `;` ignored, case-insensitive — `media_type_spells_json`,
  `serve.rs:273-277`) is the named `415` (`serve.rs:243-258`); the header stays optional
  for a tool that sends none.
- The posture is sound against what a browser can actually forge: the only mutation here
  that is a CORS *simple* request is `POST` with a safelisted type, and the three types an
  HTML form produces (`text/plain`, `application/x-www-form-urlencoded`,
  `multipart/form-data`) are refused; `application/json` is not safelisted, so it needs a
  preflight this server never answers (no `OPTIONS` route, no
  `Access-Control-Allow-Origin` anywhere). Everything else the browser can send
  (`no-cors` `fetch` with an `ArrayBuffer`/`ReadableStream` body, which carries **no**
  `Content-Type`) still carries `Origin` for a non-`GET`, and a rebound page's same-origin
  `POST` does too, so the two rules together close the path rather than overlapping on one
  vector. `DELETE`/`PUT` are non-simple methods → preflight → refused.
- Pinned: `serve.rs` unit cases (each refused type, `""` refused, admitted spellings and
  the no-header caller accepted, `stats` with `text/plain` unbothered),
  `tests/admin_auth.rs:281-360` (three origins × three routes with a *valid* bearer, corpus
  byte-identical, model untouched, same write without `Origin` lands),
  `tests/admin_mutations.rs:191-243` + `246-...` (415 writes nothing, no tmp; JSON with
  `charset` lands), `src/wire.rs:695-760`.

No new `Access-Control-*` header, no `OPTIONS` handler, and no relaxation of the loopback
enablement (`026-C5`'s posture is kept, only the browser path closed).

### 1.5 minor — quadratic header-block scan — **RESOLVED**

`find_header_end` is gone; `read_header_block` (`http.rs:355-366`) tests
`buffer.ends_with(b"\r\n\r\n")` once per octet. Reading one octet at a time means the only
terminator that can become true at step *n* is the one ending at *n*, so the linear test
misses nothing and admits no partial terminator; the `MAX_REQUEST_HEADER_OCTETS` cap check
is untouched. That removes the ~2×10⁹ 4-byte comparisons per 64 KiB block, i.e. the
cheapest network-to-CPU amplification on the box, and matches the crate's own upstream
reader. Not a finding: the only thing my fix text asked for beyond the code — a
bounded-time assertion in `tests/persist_and_concurrency.rs` — was not added (the
`oversized_header` leg at `:396-402` still only pins the refusal), and quadratic-behavior
guarding is the performance seat's to press, so I record it here rather than as a finding.

### 1.6 nit — no `sync_all` before the rename — **RESOLVED**

`persist.rs:209-225`: `write_all` → `flush` → `sync_all` → `owner_only` → rename, with the
reason stated and the parent-directory omission reasoned (std has no portable directory
handle; a lost rename leaves the *previous* document whole, the safe direction).
`schemas/catalog/v1/spec.md`'s persistence paragraph now carries the same sentence, which
is the alternative my fix offered; both halves landed. `every_accepted_mutation_is_answered_only_after_the_document_holds_it`
(`tests/persist_and_concurrency.rs:338-392`) pins the observable half: after each `2xx`,
the renamed document holds the answered record at the bumped revision with no tmp beside
it. Mode discipline unchanged (`0600` under `0700`, fixed `create_new(true)` tmp), so
checklist 6 is satisfied.

## 2. Findings open after this revision

### F-1 (minor) — `MAX_REQUEST_BODY_OCTETS` is still an unauthenticated peer's memory allowance on the serving face

- **Location**: `apps/registry/src/serve.rs:124-131` (the non-admin arm calls
  `head.into_request(&mut stream)` before `route`), with
  `apps/registry/src/http.rs:300-326` (`read_body` allocates `vec![0u8; octets]` for any
  method that declares a length) and `apps/registry/src/constants.rs:29-32, 41-49`.
- **Problem**: the credential-before-body rule was applied to `/v1/admin/**` only. Any
  other path still has its declared body buffered *before* the route table is consulted,
  and no route on the serving face ever reads a body: `route` (`serve.rs:143-173`)
  answers `/v1/candidates` and `/v1/health` from `query_param` and ignores
  `request.body` entirely. So an unauthenticated peer needs no token and no admin path to
  buy `MAX_REQUEST_BODY_OCTETS` = `MAX_CATALOG_DOCUMENT_OCTETS` = 16,777,216 octets of
  resident buffer: `GET /v1/health HTTP/1.1\r\nContent-Length: 16777216\r\n\r\n` on each
  of `MAX_CONCURRENT_CONNECTIONS` = 64 admitted slots is ~1.07 GiB of buffered bytes for
  data the server throws away — the arithmetic my round-1 finding named, unchanged. The
  new `constants.rs:41-49` comment states the general property ("so
  `MAX_REQUEST_BODY_OCTETS` is never an unauthenticated peer's memory allowance") that the
  code holds only for the admin face, and `MAX_CATALOG_DOCUMENT_OCTETS` is sized for the
  upsert's whole-document body (§Budgets, `02-proposal.md:400-409`), which is now the only
  route that parses a body at all.
- **Fix**: read a body only where a route parses one — keep `read_body` behind the route
  table by giving `into_request` a "this route parses a body" flag (or move the call into
  `admin_connection`'s carried path), and for a method/route that parses none either
  refuse the `Content-Length` or discard it through `drain_unread_body`. Then the
  `constants.rs` sentence is true as written. Pin at
  `tests/persist_and_concurrency.rs` or `tests/admin_auth.rs`: an unauthenticated
  `GET /v1/health` declaring a large length is answered without the octets being consumed.

### F-2 (nit) — a single-dash `=` spelling still echoes the value in the unknown-flag refusal

- **Location**: `apps/registry/src/main.rs:138-141` (the split requires
  `name.starts_with("--")`) with the refusal arm at `:183-190`.
- **Problem**: the split is deliberately limited to `--`, so `-admin-token=S3CRET` (one
  dropped dash, the likeliest typo of the flag this rule exists to protect) is not split:
  `name` is the whole token, it matches the `other if other.starts_with('-')` arm, and
  `escape_and_truncate(other)` quotes `-admin-token=S3CRET` — the configured bearer token
  into terminal scrollback, journald or a CI log, which is checklist 4's rule and the
  comment at `:184-185` ("whatever followed an unrecognized `--flag=` is not repeated
  back") reads as the general claim. Narrower than the round-1 case (which the fix closed
  outright), hence nit.
- **Fix**: split on the first `=` whenever the part before it starts with `-` (positional
  slugs are unaffected — they start with a source token, not a dash), or clamp the quoted
  text in the unknown-flag refusal to the run up to the first `=` in every case; extend
  `an_equals_form_takes_its_value_and_a_refusal_never_quotes_one` with a `-admin-token=…`
  row asserting the line omits the value.

### F-3 (nit) — per-read timeouts bound a read, not a connection's life; the new drain comment overclaims

- **Location**: `apps/registry/src/constants.rs:51-53` and
  `apps/registry/src/http.rs:386-399` (`drain_unread_body`).
- **Problem**: the comment states "A peer that stops sending a body it declared holds the
  admitted slot for this long and no longer." The 2 s figure is a *socket* read timeout, so
  it expires only on a gap between octets: a peer that declared `Content-Length: 65536` and
  dribbles one octet every ~1.99 s keeps `drain_unread_body`'s loop alive for ~65,536
  iterations ≈ 36 h, holding one of the 64 slots (and, pre-existing and worse,
  `read_body`'s `read_exact` at `http.rs:321-324` has the same shape with
  `REQUEST_READ_TIMEOUT` = 60 s against a 16 MiB declaration). Nothing about the change
  makes the exposure larger — the per-connection ceiling for a refused request went from
  16 MiB to 64 KiB, and `MAX_CONCURRENT_CONNECTIONS` plus §Risks' accepted low-and-slow
  row bounds the blast radius — so this is a nit about a budget comment that promises a
  bound the mechanism does not provide.
- **Fix**: either give a drained (or body-read) connection an absolute deadline — track an
  `Instant` and stop after a stated total, e.g. `UNREAD_BODY_DRAIN_TIMEOUT` × 8 — or
  reword the comment to what the code does: a *gap* of that long ends the discard. Same
  wording check for `REQUEST_READ_TIMEOUT` ("bounds low-and-slow input, it does not police
  idle peers" is honest; the drain comment is not).

## 3. Checklist walk

1. **External input validated at the boundary** — pass. The cache document is now
   validated at the write seam *and* the load seam including the per-entry candidate count
   (`cache.rs:141-152, 244-250`); the catalog document's entry set is closed at both levels
   (`corpus.rs:215-227, 396-406`, fixture
   `schemas/catalog/v1/fixtures/invalid/record-entry-outside-closed-set.json`); upstream
   ingest validation is unchanged. Nothing here evaluates or executes document content.
2. **Path handling** — pass. No new path joins; lookup keys remain opaque in-memory
   strings. `--corpus` is operator-supplied and now refuses to start over an absent file
   (`corpus.rs:167-198`, `state.rs:170-180`) instead of serving an empty public catalog —
   an availability-clarity improvement, and the absent-is-empty rule stays where the
   contract puts it (`026-C7`). No traversal surface added.
3. **Injection** — pass. No shell, no regex on untrusted input. Every echo into a refusal
   line or load error runs through `escape_and_truncate` (`corpus.rs`, `main.rs`), so an
   arbitrary JSON member name cannot emit a control byte or an unbounded string.
   `drain_unread_body`'s clamp to the declared length keeps a refused connection from
   swallowing a pipelined follow-on, and the connection closes after one answer, so no
   request smuggling across the new gate.
4. **Secrets and PII** — pass, with F-2's residual. No token in source, fixtures or tests
   (the strings are self-evidently fake); the parse refusal no longer quotes a value for
   the `--` spelling; the `401` body never echoes the presented or configured token
   (`error_token` keyed on status alone, `http.rs:440-453`); the bearer compare is the
   constant-time fold (`admin.rs:182-193`); responses carry `Cache-Control: no-store`.
5. **New dependencies** — not applicable. The diff adds no `Cargo.toml` change; the crate
   is still `std` + `serde_json` + `bindforge-core`/`discovery`. Nothing to check against
   advisories.
6. **Filesystem writes** — pass (see §1.6): owner-only modes, fixed exclusive tmp,
   `sync_all` before the rename, all under the one model write lock; refusals write no tmp
   and leave model, revision and file untouched (pinned).
7. **Clean rejection of malformed/hostile input** — pass. The two new statuses are named,
   body-only refusals with a static `error` token (`415`/`400`), decided before any
   mutation; the disabled face is one flat `404`; no stack traces, no raw request octets in
   a refusal, and the 16 MiB/64 KiB caps hold.

## 4. Gate evidence

The runner's gate is green at 2026-11-19T23:01:03Z (`npm run check`, `cargo fmt --all --
--check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace
--locked`, exit 0), so every leg quoted above ran. The evidence I relied on for the
response-cap closure is behavioral (`candidates_join`, `wire.rs`) rather than a `test
result:` line: the legs assert the unbounded body is over budget *before* asserting the
bounded one is not, which is what stops them passing vacuously. The F-1 property has no
leg: `an_unauthenticated_admin_request_is_refused_without_its_declared_body` covers the
admin path only, so no green leg establishes anything about a serving-face GET that
declares a body — recorded as missing evidence for that finding, and not blocking because
the exposure is a resource bound the design already fixes by number, not a property whose
absence prevents proceeding.

## 5. FYI (not scored)

`415` and the `Origin` `400` are two named refusals the proposal's status list
(`026-C4`/A21: `400`/`401`/`404`/`405`/`422`/`503`) and `schemas/catalog/v1/spec.md` do not
spell; they harden the face and are pinned in code and tests, but the status surface a
client may see is now wider than the contract names — the compatibility seat may want a
spec line for it.
