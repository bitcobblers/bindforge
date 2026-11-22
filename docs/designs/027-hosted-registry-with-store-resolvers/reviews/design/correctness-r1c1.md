# correctness review r1 (dissent pass c1)

verdict: request-changes

Scope as instructed: `git diff 7b5cec4… fcb66a8…` on `02-proposal.md`, then the
full current document, the ADR, and the Revision log. Verification run inside
this pass (read-only): `apps/registry/src/keys.rs` (`key_is_admissible`),
`apps/registry/src/constants.rs` (`MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216`,
`FETCHED_AT_PAST/FUTURE_TOLERANCE_SECONDS`), `crates/core/src/placement.rs`
(`MAX_DISCOVERY_SLUG_OCTETS = MAX_SLUG_OCTETS = 128`),
`crates/core/src/manifest.rs` (`MAX_NATIVE_NAME_OCTETS = 128`),
`.github/workflows/check.yml` (whole-workspace
`cargo check --target x86_64-pc-windows-msvc`, host-only tests, no macOS leg —
A16 holds), `data/catalog/v1/catalog.json` (the r1 convergence correction is
right: five of six ids are exactly what the rule mints; `portal-two` /
`Portal 2` → `portal-2` is the exception), `apps/discovery-cli/src/catalog.rs`
(`code != "200"` → `CatalogError::unreachable`) and
`apps/discovery-cli/src/main.rs:197` ("A mid-scan failure fails closed: no
partial table (`025-C4`)") — the r1 Compatibility-matrix claim is true, and
finding 1 rests on it.

## r1 closure check (my nine r1 findings)

- **[closed]** mint length bound — step 5 replaces the "by construction" claim
  with a bound step and a `steam-<appid>` fallback; Test strategy #1 adds the
  three 128-octet cases. Residual arithmetic defect re-filed as finding 5.
- **[closed]** cross-process posture — `write.lock` under `create_new` at
  startup, refusal `registry data directory is locked`, `query` included,
  pinned by #8/#12/#15. Residual consequence filed as finding 9 (nit).
- **[closed]** `fetched_at` — now "the wall clock read *before the pull's first
  page request*", with the reason stated; the delta-hole is gone.
- **[closed]** delete's two documents — "two persists under one mutation-lock
  hold, retired first", named `retired id list full`, retired in the lock
  order. Residual ordering contradiction filed as finding 4.
- **[closed]** A3/A17 — A17 added and marked *unverified*, Valve's page now
  says "documents the request side only", and the malformed-envelope/no-`apps`
  refusal exists.
- **[closed]** one retry constant for two tempos — `STEAM_PAGE_RETRY_SECONDS`
  vs `STEAM_RETRY_SECONDS`, "no startup path sleeps it". Deadline arithmetic
  filed as finding 7 (nit).
- **[closed]** games-only pin — no `include_*` flag ever sent, parenthetical
  narrowed to game appids.
- **[closed]** `query`'s empty answer — `(no candidates)` with provenance
  `none`, extended into #15.
- **[closed]** the `--corpus` carve-out to the leftover-document refusal, in
  §Hosted mode and #8.

## Findings

- [major][new] §The steam snapshot resolver (with §The hosted lookup policy
  step 5 and §Risks' `013-B4` row) — "`incomplete` is how many records the
  pull dropped for the `013-B4` name bound; `0` marks a worldlist, and the
  only kind allowed to certify a miss", "every steam key that is not an appid
  hit takes the named `503` `steam snapshot incomplete`". One over-long store
  name anywhere in the ~200k catalog turns *every* steam miss — every DLC,
  tool, video, hardware and delisted appid the games-only pull never asks for
  (finding closed in r1: the pull is games-only, so those appids are exactly
  the misses) — into a named `503`, and §Compatibility matrix now states the
  client consequence in its own words: the `503`s "fail the whole scan closed
  (`025-C4`'s fail-closed rule), so a snapshot past `027-B2`'s age bound turns
  a user's `discovery list` into a refused run" (verified in
  `apps/discovery-cli/src/catalog.rs` and `main.rs:197`). The state is
  self-perpetuating: nothing in the delta rules clears the mark, and a forced
  full refresh re-drops the same record, so `027-L3`'s promise — "inside it,
  'never heard of it' is a statement the snapshot is entitled to make" —
  becomes unavailable service-wide and indefinitely, with a budget amendment as
  the only remedy. The design rates this row Medium/High, so the mitigation it
  chose replaces the r0 cliff with an equivalent one rather than removing it:
  the pull survives, but the answer the ADR bought does not. Fix: bound the
  non-certification to the records actually discarded instead of to the whole
  document — the pull knows the dropped records' `appid`s, so keep them in a
  small capped `dropped_appids` list in the snapshot, refuse `200 []` only for
  those `appid`s, and let every other appid keep certifying its miss honestly
  (every page was examined, none was skipped); a breach of *that* list's cap
  is the pull failure `incomplete` was meant to avoid. That also makes the mark
  self-clearing (a full pull that sees the renamed title drops the entry) and
  deletes the global flag §Options' "no partial list is ever certified either
  way" sentence leans on.

- [minor][new] §The steam snapshot resolver / §The store pull — "`incomplete`
  is how many records the pull dropped for the `013-B4` name bound" against "a
  delta merges by `appid` (replace, or insert keeping ascending order) and
  re-persists; a drop for the `013-B4` bound bumps `incomplete` rather than
  failing the page". The field is defined per-pull but merged cumulatively, and
  nothing says which the implementation carries: a delta that *sets* the count
  from its own drops clears a record an earlier pull discarded, and the
  snapshot is then entitled to certify `200 []` against a worldlist it knows is
  short — the `027-L3` breach the mark exists to prevent, arriving by
  arithmetic rather than by attack. Fix: state that `incomplete` (or finding 1's
  `dropped_appids`) is cumulative across deltas and resets only on a full-pull
  replace, and say that the mark is what tells the operator to pursue the
  §Deferred resize.

- [minor][new] §The steam snapshot resolver / §Test strategy #9 — "exactly
  `{version: 1, fetched_at, incomplete, apps: [{appid, name}]}`" and health's
  "`{"games": <count>, "fetched_at": <unix>, "incomplete": <count>}`". `027-C3`
  pins both shapes verbatim — "`{version, fetched_at, apps: [{appid, name}]}`
  … refused by name outside its shape" and "a `steam_snapshot` fact carries
  `{games, fetched_at}` or `none`" — so the design ships a snapshot document
  the ADR's own shape sentence puts outside the contract, and #9 pins the
  health document the ADR does not allow. Rollout 1's annotations cover only
  "the `026` units this ADR amends", and the runner accepts ADR-027 as written,
  so nothing in the run reconciles the deviation. Fix: one line in Rollout 1
  (mirrored at the section) recording that `027-C3`'s snapshot shape and
  `steam_snapshot` health fact gain the `incomplete` fact under this run; drop
  the word "exactly" or make it the amended spelling.

- [minor][new] §Concurrency — "Lock order is `mutations → corpus read →
  retired read → issued read/write` and never the reverse: … deletes write the
  retired list then their catalog document". The order line puts corpus ahead
  of retired; the retire-first posture §Issued entries and §The curation face
  pin puts the retired write ahead of the corpus write for a corpus delete, and
  the section's "there is no cycle" rests on the line as written. All writers
  are serialized by the `mutations` mutex, so the exposure is a writer holding
  one document's `RwLock` write half while taking another's, against a reader
  that holds the later one and waits for the earlier — the exact shape the lock
  order exists to rule out. Fix: state that a delete's two model swaps are
  sequential (take the retired write half, swap, release, then the corpus
  write half) and never held together, and restate the order line to cover the
  write direction (`corpus read → retired read/write → issued read/write`)
  with the corpus *write* named as the one place a delete departs from it.

- [minor][new] §Issued entries step 5 — "A mint whose chosen id exceeds
  `MAX_SLUG_OCTETS` mints `steam-<appid>`: appid-unique, ≤ 26 octets under the
  closed key grammar's appid bound". No such bound exists: `keys.rs`'s
  `key_is_admissible` caps the *whole key* at `MAX_DISCOVERY_SLUG_OCTETS = 128`
  (`placement.rs:95`) and bounds the external id only by separators and control
  bytes, so a `steam/…` key may carry 122 decimal digits and the fallback lands
  at exactly `6 + 122 = 128` octets — in budget, but only just, and not by the
  arithmetic quoted. The step also never says which `appid` string the fallback
  is spelled from: the snapshot record's, or the admissible key's. Fix: cite
  `MAX_DISCOVERY_SLUG_OCTETS` (whole-key cap, external id ≤ 122) instead of the
  invented appid bound, and name the snapshot record's `appid` as the spelling
  source — the loader already refuses a non-decimal `appid`, and a digit-count
  bound on the snapshot's `appid` would make the claim true outright.

- [minor][new] §Test strategy #6 "Transport assurance" — "the *production*
  `StoreTransport` — the real `rustls` client config, webpki-roots anchors,
  server-name verification — driven against a loopback TLS listener serving a
  self-signed cert and one serving a cert for another name; it refuses the
  handshake both times". §The store pull pins the authority at compile time
  with "no operator-configurable store URL", and no seam that points the
  production client at `127.0.0.1` — with a name override or without one — is
  named anywhere. As written, both loopback cases can refuse for
  server-name-mismatch against `partner.steam-api.com` while chain validation
  is broken, so the case the entry says is the one that catches "a verifier
  relaxed to accept the fake" passes vacuously. Fix: name the test seam (a
  `StoreTransport` constructor taking connect address *and* verification name,
  production value still compile-time-pinned) and make the self-signed case
  present a cert whose subject name matches the connection's, so what the case
  fails on is the chain, not the hostname.

- [nit][new] §The store pull / §Budgets — "a pull of either kind that has not
  published inside `STEAM_PULL_DEADLINE_SECONDS` fails by name" with
  `STEAM_PAGE_ATTEMPTS = 3` separated by `STEAM_PAGE_RETRY_SECONDS = 30`,
  `STEAM_CONNECT_TIMEOUT = 2 s` / `STEAM_READ_TIMEOUT = 30 s` inside
  `STEAM_REFRESH_MAX_PAGES = 12`. The stated worst case is 12 × (3 × 32 + 2 ×
  30) ≈ 1.9 ks, above the 900 s deadline, so a store that is merely slow — not
  stalled — is a guaranteed named failure (startup refusal, or a delta that
  never lands), which is a posture the section presents only as a stall guard.
  Fix: one clause saying the deadline is meant to cut off a pull that would
  otherwise run past ~15 minutes *including* retries, or size it from the page
  budget.

- [nit][new] §Budgets — "`MAX_STEAM_PAGE_OCTETS = 8_388_608` (A17's page fields
  at ~150 octets for 50k records, ~12% over the mid-estimate)" beside
  `STEAM_PAGE_MAX_RESULTS = 50_000`. The cap covers a mid-estimate page, not a
  legal one: a 50k page whose names ride the `013-B4` bound the loader itself
  admits (128 octets, A10's surveyed maximum) is ~10 MiB of JSON, so the
  endpoint can answer a page the design must refuse, and every page after it
  with it. Fix: derive the pinned `max_results` from the cap
  (⌊8 MiB / worst-legal-record⌋), or size the cap for 50k worst-legal records,
  and say which the number is chosen for.

- [nit][new] §Hosted mode / §Concurrency / §Rollout 6 — "A `query` can mint and
  persist, exactly like a served query — which is why it takes the same `<data>`
  write lock and refuses to run against one a live `serve` holds". `027-C1`
  makes the hosted instance the one instance everyone points at, so the
  operator's one-shot run of the lookup policy can never run against a live
  hosted instance; it can only run against a stopped one or a copy of `<data>`
  (which it would then mint into). Fix: name the supported invocation in
  §Hosted mode and the runbook, or permit a read-only `query` against a locked
  `<data>` that answers from corpus/issued/snapshot and refuses to mint, saying
  which.

## FYI (no severity)

- Finding 1's per-`appid` fix would remove the global `incomplete` fact from the
  snapshot document, the health fact, and #2/#3/#9/#13's pins at once; the
  complexity and compatibility seats may prefer to weigh that before the
  reviser enlarges the document shape (finding 3) to carry a fact that need not
  be global.
- §Concurrency's move of the serialize-and-rename work inside the `mutations`
  mutex while readers take only `RwLock` read halves is the right shape for the
  write-amplification row; the measurement Rollout 4 promises is the number the
  complexity seat will want beside the 200k cap.
