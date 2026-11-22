# Code review — security — round 1, pass c0

- Seat: security (full panel, round 1, pass c0)
- Scope: `git diff a22cfd2694fde8d84be132c8d78faf09fda1adf4...HEAD -- ':(exclude)docs/designs'`
- Inputs used: that diff; `02-proposal.md`; `docs/adrs/027-hosted-registry-with-store-resolvers.md`;
  the runner's gate record (green at 2026-11-22T07:10:57Z); the repo's own
  `.github/workflows/{check,advisories}.yml` and the resolved `Cargo.lock`
  (read to apply checklist item 5). No review report or transcript was opened.
- Verdict: **request-changes** — 3 major, 2 minor, 1 nit; no blocker.

## Checklist walk

1. **External input validated at the boundary** — mostly held. The snapshot
   loader (`steam.rs:320-415`) re-validates shape, order, `fetched_at`
   plausibility, the `013-B4` name bound and control bytes; the store envelope
   is refused by name when it is not the A17 shape (`steam.rs:862-893`); mint
   output is grammar-checked (`mint.rs:111-125`). One gap: the steam key's
   external id is validated by *parse* rather than canonically
   (finding **S1**).
2. **Path handling** — held. No untrusted value reaches a path: documents are
   `document_path(&data, <constant>)`, the admin path segment is grammar-checked
   before use and never joined to a filesystem path, `keys.rs` keeps `../`,
   `\`, `.`/`..` and control bytes out of keys.
3. **Injection** — held. No shell, no regex. The one interpolated outbound
   request target is built from a compile-time constant path plus
   `form_encode`d/decimal parameters (`steam.rs:549-562`, `941-552`), and the
   `Host` line is the pinned constant.
4. **Secrets and PII** — held in the shipped paths. The key is environment-only
   (`main.rs:63`, `269`), `SteamStore` derives `Clone` and not `Debug`
   (`steam.rs:451-462`), no `Refusal` carries the target or key, and
   `persist::echo_safe` scrubs every warning line. Tests: the harness scrubs
   `BINDFORGE_STEAM_WEB_API_KEY` from the ambient environment
   (`tests/common/mod.rs`). Two weaknesses in the *assurance* of this property:
   findings **S2** and **S5**.
5. **New dependencies** — held. `Cargo.toml` adds `rustls` (no default
   features, `ring` provider) and `webpki-roots`; `Cargo.lock` resolves
   `rustls 0.23.46` (≥ the `RUSTSEC-2026-0346` floor), `rustls-webpki 0.103.17`
   (≥ 0.103.16), `webpki-roots 1.0.12` (≥ 1.0.10), `ring 0.17.14`,
   `getrandom 0.2.17` — every floor §Options names is met, and
   `RUSTSEC-2026-0346` lists `>=0.23.46` as patched, so no unfixed advisory is
   in the tree. Advisory coverage for the new edges runs through the repo's
   scheduled `cargo deny` scan of the whole lockfile
   (`.github/workflows/advisories.yml:66-83`), which the new packages enter
   automatically. Not applicable as a defect: the design's Rollout-3 "screen
   beside the gate" wording — see FYI.
6. **Filesystem writes** — held for documents: every new document goes through
   `persist::save`'s fixed-tmp `create_new` `0600` + rename over a `0700`
   parent, is measured against its cap on the exact rename buffer, and
   `owner_only_dir` only ever *tightens* (`persist.rs:171-201`). One gap in the
   cross-process guard: finding **S3**.
7. **Clean rejection** — held at every new seam I probed: a malformed
   snapshot/envelope/document refuses the file or page, a failed persist leaves
   model, `revision` and file untouched (`state.rs:426-433`), a cap breach
   mutates nothing. The refusal *names* itself everywhere except one admin path
   — finding **S6**. No stack traces reach a client; the `503` reason phrases
   are static strings, so nothing user-controlled is echoed into a status line.

## Findings

### S1 — major — a non-canonical appid spelling mints extra immortal identities for one appid

- Location: `apps/registry/src/state.rs:358` (`external_id.parse::<u64>()` as
  the whole gate) with `apps/registry/src/mint.rs:97` (`format!("steam-{external_id}")`)
  and `apps/registry/src/state.rs:421` (`source_keys: vec![key.to_string()]`).
- Problem: `keys.rs:60-85` admits any non-empty external id, and
  `state.rs:358` accepts whatever `parse::<u64>()` accepts, so `steam/042` and
  `steam/42` both reach the mint path for the same snapshot record — while the
  issued join (`corpus.rs:321-334`) and the mint's appid spellings are both
  spelled from the *raw* text. Concretely, for one appid a single unauthenticated
  client can hold `portal-3`, then `portal-3-steam-042`, then
  `portal-3-steam-0042`, … — the key cap (`MAX_DISCOVERY_SLUG_OCTETS = 128`)
  admits about 113 distinct zero-padded spellings, and a CJK-only title yields
  `steam-042`, `steam-0042`, … instead. That contradicts the property the
  design states for exactly these spellings — §Issued entries step 5 calls the
  appid spelling "appid-unique" — and it converts `027-B3`'s cap, which §Risks
  calls "an ordinary operating state, not an abuse edge", into something one
  client can approach ~100× cheaper per game, each mint also costing a full
  issued-document rewrite under the mutation lock.
- Fix: canonicalize once at the policy gate and use that spelling everywhere:
  in `steam_lookup`, after parsing, require `external_id == appid.to_string()`
  (a non-canonical spelling is then refused at the boundary the same way an
  inadmissible key is — not answered `200 []`, which would be a false "never
  heard of it" for an appid the fresh snapshot does know), and mint/spell and
  store the record's source key from `appid`. One appid must have one key
  spelling, one held-set probe, and one identity.

### S2 — major — no test runs TLS, so the assurance property the key-custody rule rests on is unpinned

- Location: `apps/registry/tests/steam.rs:1060-1131` (and its `1122-1129`
  "chain, not the hostname" clause).
- Problem: Test strategy #6 names this the one property no earlier test had —
  "a loopback TLS listener presenting a self-signed cert *for the name being
  verified* and one for another name; it refuses both handshakes, so the first
  case fails on the chain, not the hostname … Without it, a verifier relaxed to
  accept the fake passes the whole suite with the key on an unauthenticated
  link", and §The store pull promises "one TLS listener in the same family for
  the transport-assurance case". There is no TLS listener: the fake is a plain
  `TcpListener` that reads bytes (`tests/steam.rs:1068-1090`), and nothing in
  `apps/registry/tests` or `apps/registry/src` constructs a server config,
  certificate or acceptor (grep for `rustls|Acceptor|Certificate|ServerConfig|DER`
  across the suite returns only comments). The handshake therefore fails because
  the peer speaks no TLS at all, which is also exactly how a *mutated,
  verifying-nothing* client would fail — so the suite stays green with a
  relaxed verifier, with the key in a cleartext query string. The one assertion
  meant to pin the chain case is `!text.is_empty()` (`:1126-1128`), satisfied by
  any handshake failure; the comment at `1123-1124` claims a case the harness
  never runs. The gate record's green legs establish nothing here — no leg in
  it exercises verification.
- Fix: make the fake speak TLS. `rustls::server::Acceptor` is already reachable
  from the test target (rustls is a normal dependency), so check in a
  clearly-labelled self-signed cert+key for `partner.steam-api.com` and one for
  another name, and assert (a) both handshakes are refused before any `GET `/
  `GetAppList`/key bytes leave the client, and (b) the failure names a
  certificate/chain condition rather than an I/O one. A relaxed verifier must
  then turn the suite red.

### S3 — major — stale-`write.lock` adoption is check-then-act, so two writers can hold it at once

- Location: `apps/registry/src/store_lock.rs:79-81`, `117-134` (`refuse_or_adopt`),
  `177-194` (`process_is_live`).
- Problem: §Concurrency adds `write.lock` precisely because "two
  persist-before-serve writers rename models read at their own startups over
  each other's `027-L2` records", and prescribes one recovery: "A crashed run
  leaves the file; the runbook names deleting it." The implementation instead
  auto-adopts: it reads the stamped pid, and if `/proc/<pid>` is gone it calls
  `std::fs::remove_file` and creates its own. The read and the remove are
  separate acts, so a second concurrent startup that read the corpse's pid
  *before* the first writer stamped its own can delete the first writer's live
  lock and take the file — after which both processes run, both mint, and both
  rename issued/retired documents over each other: the exact state the lock
  exists to refuse, with `027-L2` the casualty. Nothing in the suite covers
  this: no leg plants a dead-pid lock (`tests/hosted_startup.rs:302-330` plants
  an *empty* file, which the code treats as live; `:335-390` covers the honest
  lifecycle only). PID reuse pushes the same feature the other way — a
  recycled pid makes a genuinely dead lock look live forever, an availability
  cost with no runbook line, since the runbook this diff does not add still
  tells the operator to delete a file the code claims to handle.
- Fix: either keep the design's posture (no adoption: a present lock is the
  named refusal, and the Rollout-6 runbook names deleting it), or make
  adoption atomic and self-verifying — rename the candidate aside to a unique
  name (a `create_new` winner takes it), stamp, then re-read the live path and
  refuse if the stamp is not this run's pid — and pin the dead-owner case in a
  suite that plants a lock carrying a provably dead pid.

### S4 — minor — production can hold a `SteamStore` whose transport is cleartext

- Location: `apps/registry/src/steam.rs:464-479` (`for_test` installs
  `Transport::Loopback`), `490-503` (`with_production_transport` opts *in*),
  `apps/registry/src/main.rs:354` (production's one call, which must remember
  the chain).
- Problem: the safe transport is the opt-in arm of a builder whose default is
  plaintext, and the production path reaches it through a constructor named
  `for_test`. Any future call site that builds a store without
  `with_production_transport` — or a refactor that reorders the chain — sends
  `key=` in a query string over a plain socket (`steam.rs:549-562`, `591-601`),
  which is the CWE-319/CWE-1428 posture A14 and §Options' rejected alternative
  name. The type system does not prevent it; today only the one call site does.
- Fix: make the plaintext arm unrepresentable outside tests (`#[cfg(test)]`
  variant or a test-only builder) and let the ordinary constructor build the
  verified transport, so a `SteamStore` value in scope always carries a
  `ClientConfig`.

### S5 — minor — key-bearing test legs resolve the real store authority from the gate

- Location: `apps/registry/src/main.rs:338-355` (`Config::store` resolves
  `(STORE_AUTHORITY, 443)` inside `build`, before load-or-pull), reached from
  every leg that sets `BINDFORGE_STEAM_WEB_API_KEY`
  (`tests/hosted_end_to_end.rs:33`, `tests/data_layout.rs:48`,
  `tests/health_route.rs:38`, `tests/hosted_lookup_policy.rs:41`,
  `tests/issued_entries.rs:32`, `tests/mint_rule.rs:32`,
  `tests/retired_ids_and_curation.rs:27`, `tests/cli_end_to_end.rs:56`).
- Problem: Test strategy #17 pins "nothing resolves `partner.steam-api.com`
  from a test". Resolving the address in `build` — not in the pull — makes that
  resolution unconditional for a resolver-active run, so the gate's own test
  processes ask a real third party's name (the green gate, whose
  `serve`-shaped legs need the resolve to succeed, is the evidence that they
  do). It also leaves no seam to stop the next step: a leg that sets a key
  without a loadable snapshot goes on to `store.full(now)`, which would dial
  the real host and put the query string, key included, on it. The harness
  scrubs ambient keys, so today's exposure is DNS only.
- Fix: keep the authority pinned for `Host`, SNI and verification, but move the
  resolve out of `build` into the transport's `connect` (or take the connect
  address through the address/name seam §Test strategy #6 already names), so a
  planted-snapshot leg never resolves and no leg can reach a non-loopback
  address.

### S6 — nit — `retired id list full` reaches the wire as `503 Bad Request` / `error: "bad_request"`

- Location: `apps/registry/src/state.rs:662-666` (the status-503 `Reject`) with
  `apps/registry/src/admin.rs:231-249` (`Answer::refused` maps every unmapped
  status to reason `"Bad Request"` and token `"bad_request"`).
- Problem: §Issued entries makes this a *named* refusal, and `Named::code()`
  already carries `retired_id_list_full` — which this path never uses — while
  `query`/`health` render the name properly. An operator's admin client sees a
  status line that contradicts itself and an error token that names nothing,
  leaving only the prose detail to identify which cap refused the delete.
- Fix: give `Answer::refused` a 503 arm (`"Service Unavailable"`) and pass the
  `Named` code through on this path, as the serving face's
  `refusal_body(named.code(), &named.detail())` already does.

## Missing evidence, recorded

- No leg in the captured gate exercises TLS verification, and no leg in the
  suite can (S2). I record that the property "the shipped `ClientConfig`
  refuses an untrusted chain, and the key only travels after it accepts one"
  is established here by construction and reading only
  (`steam.rs:494-503`: `ClientConfig::builder().with_root_certificates(webpki-roots)`
  with `ServerName::try_from` verification at `648-656`) — which is why I rated
  it major rather than blocker. If that reading is wrong, the finding is a
  blocker.
- No leg covers the dead-owner `write.lock` case (S3), so no evidence exists
  either way for the adoption path.

## FYI (no severity, not this seat's call)

- `steam.rs:606` accepts the whole `200..300` band where §The store pull names
  `non-200` the refusal; `main.rs:359-378` loads a stale-but-loadable snapshot
  where `027-C3` says "load if present and within `027-B2`'s freshness, else
  pull in full" (the deviation is self-documented at `:359-368` but the
  `03-build-notes.md` it cites was not among my inputs).
- Nothing in this diff lands Rollout 3's `check.yml` Windows-leg scope split or
  the `README.md` C-toolchain prerequisite — with `ring` now in the tree, the
  unchanged `cargo check --workspace --target x86_64-pc-windows-msvc` leg is the
  one the proposal says must drop `apps/registry` — nor Rollout 6's
  `docs/registry-hosting.md`, which is where this design parks key custody
  (CWE-526), the no-URL-in-logs rule, and the cap/restart arithmetic an operator
  needs.

## Verdict

request-changes — S1, S2, S3 open as majors.
