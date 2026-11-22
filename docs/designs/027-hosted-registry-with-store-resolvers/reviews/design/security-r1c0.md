# security review r1 (c0, full panel)

verdict: request-changes

Checklist applied: (1) trust boundaries — store snapshot and store pages are
network-fed, attacker-influenceable content; corpus operator-held; issued and
retired documents self-written; client keys untrusted. Design classifies all
of these; mint input is the snapshot name, never client input (good). (2)
parsing — named caps (027-B1, MAX_STEAM_PAGE_OCTETS), metadata-gated reads
(verified in `apps/registry/src/persist.rs` and `constants.rs`), no
decompression surface (hand-rolled HTTP/1.1, `Transfer-Encoding` refused —
verified in `upstream.rs`). (3) path handling — issued delete follows the
shipped decode-then-validate pattern (`corpus.rs` `is_slug_name` ∧
`MAX_SLUG_OCTETS`, verified); no path interpolation. (4) secrets — env-only
key, no URI/key in diagnostics, file custody deferral consistent with the
ADR. (5) dependencies — checked below. (6) privileges — outbound to one pinned
authority plus owner-only data dir; least privilege holds. (7) fail-closed —
named refusals throughout; one gap (F4).

## Findings

- [major][new] The store pull / Test strategy #6, #17 — "in tests, the same
  request/response codec run against a loopback fake (plain TCP)" — the one
  control that separates the chosen option from the rejected plaintext one is
  that the production hop authenticates `partner.steam-api.com`: without it
  the personal, rotation-foreclosed `BINDFORGE_STEAM_WEB_API_KEY` (the design
  itself says "rotation foreclosed" in the Risks table) rides a link any
  path-MITM can read and any snapshot consumer can be poisoned through. Not
  one planned test exercises that: the production rustls `ClientConfig`
  (webpki-roots trust anchor, server-name verification) is built by hand and
  never driven — every pull test runs the codec over plain loopback TCP, so a
  transport configured to accept self-signed certs, or a custom verifier
  added to keep the fake seam shareable, passes the entire suite green. The
  property "the store hop verifies the pinned authority" cannot be
  established from the test strategy as written, and the key's confidentiality
  rests on it. Fix: add one named test — drive the *production* transport
  against a loopback-only TLS listener serving a self-signed cert and assert
  the handshake is refused (and, secondarily, that a cert for a wrong
  server-name is refused); 127.0.0.1 dialing keeps the "no test resolves or
  dials a non-loopback name" rule intact.

- [major][new] Risks — "013-B4's 128-octet name bound has no margin (A10):
  one over-long store name fails every pull, and the resolver ages into 503
  within 7 days" rated Likelihood Low. The rebuttal in the Revision log
  ("fail the pull by name; silent record drops rejected as a hole under
  027-L3; re-sizing deferred") is unsound on likelihood: store display names
  are attacker-chosen content, and the design's own A11 records that "games
  have renamed existing appids to impersonate hits" — anyone cheap enough to
  ship or rename one Steam title can plant a >128-octet name, which fails
  every delta (the record recurs in the `if_modified_since` window) and every
  forced full refresh, with the only remedy (re-sizing) deferred "no
  follow-up yet". That is permanent, remote, unauthenticated denial of the
  ADR's central feature for every hosted user — the 7-day age bound converts
  all steam answers to `503` forever, not once. The framing is a false
  binary: loudly failing the whole pull and *silently* dropping the record
  are not the only postures. Fix: a third posture that preserves `027-L3` —
  drop the offending record, persist an "incomplete" fact in the snapshot
  document (and in `health.steam_snapshot`), and while that fact stands no
  steam miss earns `200 []` (all non-hit steam keys take the named `503`,
  corpus/issued/appid-hits keep answering); the pull survives, the honesty
  claim is never made about a list the registry knows is incomplete, and the
  partial-snapshot rule is untouched because no partial list is ever
  certified.

- [major][new] Options considered / Risks — "The store hop gets `rustls` (no
  default features, `ring` provider) plus `webpki-roots`" with the sole
  dependency mitigation "versions pinned". One month before this proposal,
  CVE-2026-111943 produced advisories *in the exact crates this design
  introduces*: RUSTSEC-2026-0381 (`webpki-roots`, patched ≥ 1.0.10; the
  advisory warns that the 0.26.x compatibility package stays affected unless
  its separately resolved 1.x parent is ≥ 1.0.10) and RUSTSEC-2026-0382
  (`rustls-webpki`, patched ≥ 0.103.16 / ≥ 0.104.0-alpha.8) — certificate-path
  acceptance of distrusted emSign intermediates whose private keys are now
  public, i.e. exactly the tooling to impersonate the pinned authority on the
  hop that carries the key. RUSTSEC-2026-0346 (`rustls` Debug impls disclose
  TLS secrets, patched ≥ 0.23.46) additionally collides with this design's
  "no diagnostics carry the request" posture. These are patched, so not
  checklist blockers — but "versions pinned" names no floor and the "cargo
  trio covers it" mitigation covers no advisories (fmt/clippy/test do not
  screen RUSTSEC). Fix: state the minimum floors in the dependency decision
  (rustls ≥ 0.23.46, rustls-webpki ≥ 0.103.16 or ≥ 0.104.0-alpha.8,
  webpki-roots ≥ 1.0.10 with the 1.x-parent resolution note) and make a
  RUSTSEC advisory screen (cargo audit or cargo deny) part of Rollout item 3
  alongside the dependency landing.

- [minor][new] Issued entries, retired ids — "`MAX_RETIRED_IDS_OCTETS =
  33_554_432`" names the budget but pins no posture at exhaustion, unlike the
  issued document whose enforcement is spelled out ("enforced at load
  (metadata-gated) and against the exact buffer a write renames"). Unpinned,
  an implementer facing a full retired list can either block every future
  delete (curation's correction and promotion paths die) or fail open by
  dropping the append — and a dropped append silently re-opens a retired
  `game_id`, which is `027-L2` ("deleting a record never frees its id")
  broken by budget arithmetic. Fix: one sentence pinning fail-closed — a
  delete whose retired-list write would exceed the cap is the named refusal
  and mutates nothing; the load-side metadata gate mirrors the issued
  document's.

## FYI (no severity)

- The delta's `if_modified_since` reuses a wall-clock-published `fetched_at`;
  a future clock step (bounded to 86,400 s by the plausibility pair) shifts
  the delta window and can skip newly added/changed apps for up to a day
  while `200 []` still certifies them — a correctness-seat freshness
  question (A12 neighborhood), not a security one.
- `MAX_STEAM_PAGE_OCTETS = 8_388_608` against a pinned 50,000-item page leaves
  ~167 octets per record; a pathological page of near-128-octet names grazes
  the cap. Performance/correctness arithmetic, same failure family as the
  finding above.
- Verified assumptions held under read-only grep: A2 persist discipline
  (`create_new` fixed tmp at 0600, parent 0700, rename, metadata-gated read),
  A8 constants (`MAX_RESPONSE_OCTETS = 65_536`, `MAX_CANDIDATES = 256`,
  `MAX_CONCURRENT_CONNECTIONS = 64`), A10 grammar/bounds, shipped upstream
  refusal of `Transfer-Encoding` and `3xx`, and the shipped decode-then-validate
  admin path pattern the new issued delete copies.
