# Code review — security seat — round 2, dissent pass c0

- **Run:** 027-hosted-registry-with-store-resolvers
- **Seat:** security
- **Pass:** c0 (dissent) — checks the last revision against this seat's round-1 findings
- **Scope:** `git diff f6d24bca3d755f79e8f26ac938d9ffee9372c983..HEAD -- ':(exclude)docs/designs'`
  (7 paths: `apps/registry/src/steam.rs`, `apps/registry/tests/steam.rs`,
  `apps/registry/tests/fixtures/store/{README.md, two .crt.der, two .key.pkcs8.der}`)
- **Inputs used:** that diff, `02-proposal.md`, `docs/adrs/027-hosted-registry-with-store-resolvers.md`,
  the runner's gate record, my own round-1 findings. No other `reviews/` content opened.
- **Verdict:** approve — no blocker or major open; two carried findings (one minor, one nit)
  unaddressed by this increment, one new nit on the changed code.

## Increment under review

The diff is the answer to round-1 finding 1 and nothing else:

- `apps/registry/tests/fixtures/store/` — two checked-in self-signed end-entity
  certificates plus their PKCS#8 keys, and a README with the exact
  `openssl req -x509` recipe that produced them.
- `apps/registry/tests/steam.rs` — `the_production_transport_refuses_an_unauthenticated_loopback_listener`
  becomes `the_production_transport_refuses_self_signed_store_certificates`
  (steam.rs:1076), now backed by a real `rustls::ServerConnection` fake
  (`serve_tls_fixture`, steam.rs:1170) instead of a raw `TcpListener`; the
  chain case is pinned by `text.contains("UnknownIssuer")` (1146-1150) instead
  of `!text.is_empty()`, and the key/request assertions now watch the
  **decrypted** stream (`decrypted.is_empty()`, 1153-1163). The module-doc gap
  note (former 58-70) is replaced by a description of the real pair.
- `apps/registry/src/steam.rs` — a new `io_reason` (842-847) is substituted for
  three bare `error.kind()` interpolations (697-701, 910-914, 1037-1041), so an
  `InvalidData` leg — rustls' code for its own errors on this seam — carries the
  carried `rustls::Error` Display into the refusal. That Display is what makes
  the `UnknownIssuer` pin observable from the outside.

## Disposition of this seat's round-1 findings

1. **major — TLS listener absent, so §Test strategy #6's transport-assurance clause
   was reduced to prose.** **CLOSED.** The fixture pair exists and is what the design
   named. I verified the fixtures directly (`cat -v` on the DER): both are
   self-signed ECDSA P-256 leaves with `subject == issuer`, `subjectAltName` DNS equal to
   the file's own name (`partner.steam-api.com`, `some-other-name.example`),
   `basicConstraints critical CA:FALSE`, validity 2026-11-22 → 2046-11-17 — matching
   the README's recipe and its "refresh, not incident" note. They cannot chain to
   `webpki-roots` (steam.rs:522-530), which is the point. The pin is now behavioral, not
   vacuous, and the leg is a genuine mutation catcher: relax the verifier and the
   handshake for the *verified-name* cert completes, the fake's `StreamOwned::read`
   decrypts the real `GET`, and both 1146 and 1157 fire (the pull would then sit on the
   5 s deadline and answer `steam pull deadline`, so `UnknownIssuer` fires first). The old
   `!text.is_empty()` and the "writes the request … before the handshake is assured"
   claim are gone. Ran it 8× plus a full `cargo test -p bindforge-registry`: green every
   time, 0.40 s per leg, no flake from the 150 ms window (any delivered app data reaches
   `seen` before the client returns, well inside the sleep).
2. **minor — no leg pins the canonical-appid gate** (state.rs:378-387). **OPEN, unchanged.**
   This diff touches no `src/state.rs` and no lookup-policy test; grep for `steam/0` and
   `steam/+` across `apps/registry/tests` is still empty, and `hosted_lookup_policy.rs:309-335`
   remains the only external-id spelling shape pinned. See finding A.
3. **nit — invented `<0.104` ceiling on the verifier floor** (Cargo.toml:41). **OPEN,
   unchanged.** §Options (02-proposal.md:96-97) names `rustls-webpki ≥ 0.103.16` /
   `≥ 0.104.0-alpha.8` as two equally-patched lines. See finding B.

## Findings

### A. minor — the canonical-appid gate still has no regression guard

- **Location:** `apps/registry/src/state.rs:378-387` (the
  `.filter(|appid| appid.to_string() == external_id)` gate) and 395-400 (mint spellings
  taken from `app.appid`); the only external-id shape pinned anywhere is
  `apps/registry/tests/hosted_lookup_policy.rs:309-335` (`steam/not-an-appid`).
- **Problem:** the code now guards the property twice — the gate rejects a non-canonical
  decimal spelling, and `mint` is spelled from `app.appid.to_string()` rather than the
  queried text, so even a loosened gate cannot re-spell `steam-<padded>` from padding the
  store never issued. That is the right shape; it is also unprotected. Deleting the
  `.filter(...)` (i.e. reverting to a bare `parse::<u64>().ok()`) leaves every current leg
  green: `steam/042` passes the dedup probe (`state.rs:434-437`, keyed on the *queried*
  text), finds the plain slug held, and takes the `<slug>-steam-<appid>` collision spelling
  as a second immortal `027-L2` identity for one snapshot record, which is what breaks
  `027-B3`'s one-mint-per-key arithmetic and §Issued entries steps 3/5's appid-unique
  premise. Test strategy #3 pins only the non-decimal half, so this case belongs to this
  diff's code and nothing in the suite would notice its removal.
- **Fix:** one leg, either face. Against a fresh snapshot carrying appid `N`, query
  `steam/0N`, `steam/+N`, `steam/0…0N` and `steam/N`; assert the first three answer `200 []`
  and mint nothing (no issued document), the fourth answers the minted id; then assert the
  issued document holds exactly one record for `N` — one `steam/N` source key, one `game_id`,
  revision moved once.

### B. nit — the verifier floor carries a ceiling the design does not name

- **Location:** `apps/registry/Cargo.toml:41` — `rustls-webpki = ">=0.103.16, <0.104"`
  (resolved: `Cargo.lock` rustls 0.23.46, rustls-webpki 0.103.17, webpki-roots 1.0.12).
- **Problem:** unchanged from round 1, and unchanged by an increment that otherwise
  engaged with this seat. §Options names `rustls-webpki ≥ 0.103.16` **and**
  `≥ 0.104.0-alpha.8` as two equally-patched lines (02-proposal.md:96-97); the `<0.104` half
  appears nowhere in the design. The ceiling does no floor work — the floor is the `>=` half,
  and cargo intersects this declaration with rustls' own requirement — so what it buys is
  the risk that the first `rustls 0.23.x` security patch requiring `rustls-webpki 0.104`
  cannot resolve, on the one hop that carries the Web API key, and that taking the fix then
  means editing this line. This crate's own comment (Cargo.toml:22-29) says the floors are
  load-bearing and the RUSTSEC screen is what makes them safe; a ceiling is the opposite of
  that posture.
- **Fix:** state the floor without the invented ceiling (`rustls-webpki = ">=0.103.16"`), or
  keep the range and name in the comment who owns the `<0.104` decision, cross-referencing
  §Options' `0.104.0-alpha.8` clause so the next bump reads it as a decision, not an accident.

### C. nit — the new detail-carrying failure sink is outside the URI-leghold

- **Location:** `apps/registry/src/steam.rs:842-847` with its three call sites
  (697-701, 910-914, 1037-1041); the sweep at `apps/registry/tests/steam.rs:1033-1064`
  and the new leg's leak assertion at `apps/registry/tests/steam.rs:1136-1140`.
- **Problem:** §Test strategy #6's last clause is "no failure line ever contains the key
  **or the request URI**", and this diff makes the failure lines richer for the first time:
  an `InvalidData` leg now interpolates a carried `rustls::Error` Display into the refusal
  that reaches `persist::echo_safe` and the operator's line (steam.rs:462-467, main.rs:503).
  Two guards cover that new surface, and between them they cover only half the clause:
  `no_pull_failure_line_carries_the_key_or_the_request_uri` sweeps only the two plaintext-fake
  cases (`http-500`, `bad-envelope`) — no leg ever routes a TLS-Display refusal through its
  `http://`/`https://`/`GetAppList` assertions — while the TLS leg asserts only
  `!text.contains(KEY)`. Nothing is wrong today (rustls' `Error` is a bounded enum whose
  Display never carries a URI, a path, or peer bytes), so this is coverage, not a defect;
  it matters because the assertion pair is the only thing that keeps it true after the next
  error-mapping edit lands on the same three call sites.
- **Fix:** in the TLS leg, alongside the existing `!text.contains(KEY)`, assert
  `!text.contains("http://") && !text.contains("https://") && !text.contains("GetAppList")`
  over the same `text` — three lines that put the new sink inside #6's URI clause.
  (Related guard-rail, already half-respected: Cargo.toml:23 names RUSTSEC-2026-0346 as
  *`Debug` disclosing TLS secrets*. `io_reason` interpolates `Display`, and its doc-comment
  says so; keep it that way — a later `{error:?}` on these seams is the advisory's own door.)

## Checklist (references/security.md)

1. **Boundary validation of external input** — n/a to this diff: no request-handling or
   schema path changed.
2. **Path handling** — clean. The only path built is
   `env!("CARGO_MANIFEST_DIR")/tests/fixtures/store/<fixed-name>.<suffix>` from two loop
   constants (steam.rs:1084-1091); no `..`, no untrusted component, no traversal.
3. **Injection** — clean. No shell, no regex, no interpolated command; the README's
   `openssl` recipe is documentation with a shell variable, not executed code.
4. **Secrets and PII** — clean, and now explicitly considered: the diff commits two private
   keys. Both are self-signed fixture keys for certificates that chain to nothing
   (README:1-10 states they "certify nothing and must never be used anywhere else"); the
   `KEY` in the suite is the fake. No real credential, no PII, no key or URI in any
   production line — the new Display stays behind `persist::echo_safe`. Note for the record:
   git records the keys at mode `100644` while the working copy holds `0600`; a fresh
   checkout therefore lands them world-readable. Immaterial for throwaway material, worth
   knowing if anyone ever reuses this directory shape.
5. **New dependencies** — none. No `Cargo.toml`/`Cargo.lock` change in the range: the fixture
   route avoided `rcgen`/a dev-dependency entirely, and the TLS server code rides the
   already-pinned `rustls` (>=0.23.46, past RUSTSEC-2026-0346) at the same feature set. The
   only dependency observation is finding B, on a line this diff did not touch. I found no
   advisory against the resolved rustls 0.23.46 / rustls-webpki 0.103.17 /
   webpki-roots 1.0.12 trio, and `.github/workflows/check.yml`'s RUSTSEC screen is the
   standing guard.
6. **Filesystem writes** — none added; the new code only reads two fixture files.
7. **Clean rejection** — clean. The store refusal remains one named line for the operator
   (startup refusal, or the `warning:` line at main.rs:503), never an HTTP body —
   `wire.rs:386-391` keeps health to `{status, mode, corpus_entries, steam_snapshot}` — so
   the extra TLS detail is operator-facing, control-stripped by `echo_safe`, and cannot be
   driven by peer bytes: rustls' error Display is a bounded enum, so no log-injection or
   stack-trace-to-client path opens.

## Evidence

- Runner gate at 2026-11-22T11:13:49.574Z — green on all four legs (`npm run check`,
  `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked`). (The single captured `test result: ok. 0 passed` line
  belongs to a leg whose tests are ignored; several such legs exist independent of this diff.)
- My own runs, on this worktree:
  `cargo test -p bindforge-registry --test steam -- --exact
  the_production_transport_refuses_self_signed_store_certificates` — 8 consecutive runs, all
  `ok. 1 passed; 0 failed`, 0.39-0.40 s each; full `cargo test -p bindforge-registry` — all
  legs green, including the 27-leg `steam.rs` binary.
- Fixture DER inspected byte-wise for subject/issuer/SAN/basicConstraints/validity (see the
  disposition of finding 1).

## FYI (no severity)

The pair dials each certificate with *its own* name (`verify_name` and the fixture are the
same loop variable, steam.rs:1083-1089), so both legs fail on the chain and nothing ever
presents the pinned authority with a certificate for another name. That is no weaker than
§Test strategy #6's literal second case — and hostname rejection is not observable at this
seam anyway, since an untrusted chain answers `UnknownIssuer` before the name is ever
consulted, and `with_production_transport` deliberately refuses to install a non-production
`ClientConfig` that could carry a fixture root. The name plumbing itself is pinned by
`the_verify_name_is_independent_of_the_connect_address` (steam.rs:1199-1249), which reads the
raw ClientHello and asserts the authority is what the handshake carries.
