# security review r1

verdict: request-changes

Scope: whole document (full panel, c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/026-registry-service.md`, the Revision log, and read-only
inspection of the code the proposal's assumptions cite
(`apps/discovery-cli/src/catalog.rs`, `apps/discovery-cli/src/matches.rs`,
`crates/discovery/src/game.rs`, `crates/core/src/placement.rs`, `Cargo.toml`,
`Cargo.lock`, `apps/discovery-cli/Cargo.toml`).

## Checklist walk

1. **Trust boundaries** — every source classified: corpus file (operator
   input, read at start, never rewritten on refusal), cache and local-entries
   documents (declared untrusted at load, validated again at the upstream
   write seam), upstream `/v1/candidates` response (untrusted: the design
   re-runs the shipped client's framing discipline with its own constants —
   verified against `catalog.rs::read_response`, which really does cap the
   header block and body at 65,536, reject `Transfer-Encoding`, duplicated /
   non-decimal `Content-Length`, over-count, and `read_exact` the promised
   length — A1 holds), inbound HTTP request (untrusted, budgeted), CLI args
   (user). One gap: the admitted lookup-key spelling is too weak for a
   network-fed seam (finding 1).
2. **Parsing untrusted data** — size limits on both request and response
   sides; schema validation by named refusals; no code evaluation; no
   compression anywhere, so no decompression bombs. The corpus load has no
   read budget (finding 4, nit). Cache-load `slug` validation is missing
   from the named refusal list; subsumed by finding 1's validator.
3. **Path handling** — every `game_id` surface (DELETE segment, upsert,
   cache-load re-check) is gated by `game_id_is_valid` =
   `is_slug_name` (`[a-z][a-z0-9-]*`, `crates/core/src/placement.rs`, whose
   doc states it exists to keep `..`/separators from ever asking the
   filesystem for a path) plus the length bound — traversal is impossible
   there. The lookup-key surfaces are *not* gated this way (finding 1). No
   path is ever interpolated into a shell string anywhere in the design.
4. **Secrets and PII** — the admin token is configured by flag or env; the
   bearer-over-plaintext deviation from RFC 6750 is recorded honestly (A20
   unverified, §Risks, §Deferred to `026-R1`) and is the ADR's own posture
   (`026-R1` states it) — consistent, not re-opened. What may be logged of
   the token is never stated (finding 3).
5. **Dependencies** — the design adds no new external dependency:
   `serde_json` is already shipped (`apps/discovery-cli/Cargo.toml`,
   workspace-pinned in `Cargo.lock`); RustSec publishes no advisory page for
   `serde_json` (package lookup 404s; the 2026 Fedora touchpoints are
   feature/other-package updates, none an unfixed `serde_json` advisory).
   std-only serving face adds nothing. Pass.
6. **Privileges** — TCP bind (default `127.0.0.1`, ephemeral port) and
   owner-only `0700`/`0600` files under `<data>`; least privilege holds.
   Loopback-default posture preserved. Pass.
7. **Fail closed** — named refusals everywhere, atomic tmp-and-rename with
   exclusive fixed-name tmp (verified in `matches.rs`), cold `503` never
   `[]` (`026-L4`), redirects refused upstream (protects `026-L2`), two
   writers on one corpus refused loudly. Pass; the adjacent availability
   gap is finding 2.

Deferred items: each maps to `026-R1`/`026-R2`/`026-R3` or a data edit and
is consistent with the ADR's Decision; none contradicts a cited clause —
none re-opened. The Revision log's r0 entries cover format carve-out, budget
closure, private budgets, seed ids, upstream integrity, the C5 ambiguity,
two-writer refusal, request budgets, and health posture; none of my
findings below is among them.

## Findings

- [major][new] Chosen design → Serving face — "Percent-decode `slug`
  **first**, then apply `025-B1`'s 128-octet cap to the *decoded* bytes and
  `spells_lookup_key` to it; the corpus/index is keyed by the decoded string"
  — the decode-first order is right, but the two admitted checks are
  provably too weak for the alphabet they admit: `spells_lookup_key`
  (`crates/discovery/src/game.rs`) is *shape-only* — closed source token,
  one `/`, non-empty external-id, no character rule, no dot rule — and
  `025-B1` is a length cap, so `slug=local/%2e%2e%2f%2e%2e%2fetc%2fpasswd`
  decodes to `local/../../etc/passwd` and passes the design's own admission
  gate for `/v1/candidates`, passes the `POST /v1/admin/games` upsert ("a
  `source_key` failing `spells_lookup_key` or `025-B1`" is the same
  predicate), and is persistable into the corpus and cache documents; the
  cache-load refusal list ("refusals for bad version/shape/`game_id`/`name`")
  never names `slug`. The design nowhere pins which lookup-key spellings the
  registry accepts nor states the key's data plane, so the property
  "separator- and dot-bearing keys can never ask the filesystem for a path"
  cannot be established from the text — while the codebase's own precedent
  (`is_slug_name`'s doc: the grammar "keeps a `..` or a separator from ever
  asking the filesystem for a path") shows a T-005/T-007 implementer
  indexing by joined path is the house pattern the next contributor reaches
  for. Fix: pin the admitted spelling — after the first `/`, exactly one
  non-empty `.`-free component with no embedded `/` — as one named validator
  applied at candidate query, admin upsert, and cache load, and state that
  lookup keys are opaque in-memory strings, never joined onto paths.

- [minor][new] Chosen design → module plan — "`serve.rs` — accept loop (one
  thread per connection, request-read timeout to bound low-and-slow)" —
  for the design's purpose, a publicly reachable root with an anonymous
  serving face, the pinned defenses bound inbound bytes only: no cap on
  concurrently open connections (one OS thread each, held to the timeout or
  longer), and the named timeout is a *read* timeout — a peer that never
  reads the response parks a writer thread indefinitely. The §Risks row
  "Low-and-slow / one-inbound-per-outbound exposure on a routable bind"
  lists only the read timeout and the octet budgets as its mitigations.
  Fix: name `MAX_CONCURRENT_CONNECTIONS` in `constants.rs` with an immediate
  close when over the cap, and a write-side timeout or overall per-connection
  deadline, in the `serve.rs` line and that risk row's mitigation.

- [minor][new] Chosen design → Admin face and auth — "`--admin-token` or
  `BINDFORGE_REGISTRY_ADMIN_TOKEN`" / "dropped with one escaped, single-line
  echo" — the design states the escaping posture for echoed untrusted ids
  but never states the logging posture for the token: nothing says the
  configured token never enters refusal lines, logs, health/stats output, or
  the `401` path's text. `026-R1` carries transport assurance, not
  local/accidental disclosure, so this gap is this design's to close. Fix:
  one sentence in the admin section — the configured token is never echoed
  by any refusal, log line, or status surface; `401` carries only
  `WWW-Authenticate: Bearer` and the status line.

- [nit][new] Chosen design → Catalog document and its spec — "a
  `source_key` failing `spells_lookup_key` or `025-B1` (A3, A16), a
  non-object/missing field. Nothing on disk is rewritten on refusal" — the
  corpus file is the one document the design loads with no read budget: the
  cache gets `026-B2` re-checked at load, the match store precedent caps the
  read at `STORE_READ_BUDGET_OCTETS` before parsing (`matches.rs`), but
  `data/catalog/v1/catalog.json` is checked only by per-record bounds known
  after the parse, so an oversized or corrupt corpus parses unbounded.
  Fix: mirror the precedent — a private corpus-read byte budget checked on
  file metadata before the parse, stated in `corpus.rs`'s line.

- [nit][new] Chosen design → Caching mode — "It **refuses redirects** (a
  `3xx` is an upstream error → `stale`/`503`), because following a redirect
  away from the configured endpoint would break `026-L2`'s disclosure model"
  — the redirect refusal is exactly the right trust-boundary call; the
  mirror sentence, however, pins framing rules only for responses the
  registry *reads*. The inbound request reader's framing is left open, and
  the deployment the ADR points at for assurance (an edge terminating TLS
  in front of the root, `026-R1`) makes a `Content-Length`/`Transfer-
  Encoding` framing mismatch between that edge and the hand-written reader a
  classic request-smuggling seam. Fix: extend the mirror sentence to the
  request reader in `http.rs` — inbound bodies are `Content-Length`-framed
  only; a request `Transfer-Encoding` is a named refusal.

## FYI (no severity)

- (correctness seat) Test strategy #3's closure "encoded length stays within
  the request budget" relies on the client's encoder (verified: expands
  ≤3×, within `MAX_REQUEST_HEADER_OCTETS`); an adversarial encoder is
  bounded by the same request budget, so the closure looks right — worth
  the correctness seat's second read.
- (complexity seat) `MAX_REQUEST_BODY_OCTETS` is named but never valued;
  only `MAX_REQUEST_HEADER_OCTETS` gets its 65,536.
- (compatibility seat) "Enablement is judged from the bind address literal"
  fails closed for hostname binds (treated as non-loopback ⇒ token or
  `404`), which is the safe direction but may surprise an operator binding
  `--host localhost` without a token.
