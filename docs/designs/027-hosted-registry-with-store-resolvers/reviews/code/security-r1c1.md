# Code review — security seat, round 1, dissent pass c1

Scope: `git diff 75cbc433ed8a546734c41d4eb928395373ab73e6..HEAD -- ':(exclude)docs/designs'`,
read against `docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md` and
`docs/adrs/027-hosted-registry-with-store-resolvers.md`. Inputs: that diff, the proposal, the
ADR, the runner's gate record, and my own round-1 findings. Specialty: security
(`references/security.md`) only.

**Verdict: `request-changes`** — 1 major open (carried, unaddressed), 1 minor and 1 nit new.

## Round-1 findings: disposition

| r1 | Severity | Disposition |
|---|---|---|
| 1 | major | **Resolved** — canonical-spelling gate at `apps/registry/src/state.rs:378-400`; mint ids spelled from `app.appid` |
| 2 | major | **Open** — still no TLS listener anywhere; see F-1 |
| 3 | major | **Resolved** — dead-owner adoption deleted; design's presence-is-refusal restored |
| 4 | minor | **Resolved** — `SteamStore::production` (§ below); `Config::store` no longer builds through `for_test` |
| 5 | minor | **Resolved** — authority held as a name, resolved in `connect`; suites no longer resolve it |
| 6 | nit | **Resolved** — `AdminRefusal::Named` → `Answer::named`, named 503 on the wire |

### 1 — canonical appid spelling (was major, closed)

`apps/registry/src/state.rs:378-387` now refuses a non-canonical decimal spelling before
the snapshot probe (`parse::<u64>().ok().filter(|appid| appid.to_string() == external_id)`),
and `state.rs:400` spells the mint's appid spellings from `app.appid.to_string()`
(`SnapshotApp.appid` is `u64`, `apps/registry/src/steam.rs:94`), so every `Record.source_keys`
entry the mint path writes is `steam/<canonical appid>` (`state.rs:452`) and one appid has one
key spelling, one held-set probe and one identity. The ~100× cheaper consumption of
`027-B3`'s issued cap that `steam/042`-class spellings bought is gone, and
§Issued entries' "appid-unique" property now holds at the gate that decides whether to mint.
The answer chosen for the rejected spellings is the fresh snapshot's `200 []`, which is the
branch the proposal's policy step 5 already names for "a non-decimal external id"; I asked
for a named refusal instead, but the design does not name a refusal for a token that is not
an appid spelling, so the implementation's reading is within the contract and no abuse
remains. Closed. (Regression assurance for the property is untested — new finding F-2.)

### 2, 3 — transport construction and the `<data>` lock (closed)

`apps/registry/src/steam.rs:538-547` is the production constructor (`Endpoint::Pinned`
+ `StoreTransport::Verified(production_client_config())`), and `apps/registry/src/main.rs`
`Config::store` is now one line calling it (`main.rs:345-347`); `Cleartext` is reachable only
through `SteamStore::for_test` (`steam.rs:556-565`), which no product path calls. The verified
arm is `ClientConfig::builder().with_root_certificates(webpki_roots)` with
`ServerName::try_from(self.verify_name)` at `steam.rs:777-785`, and the resolve moved into
`connect`/`dial_addresses` (`steam.rs:742-814`), so building a store dials and resolves
nothing — the `BINDFORGE_STEAM_WEB_API_KEY` integration legs plant a *fresh* snapshot
(`hosted_end_to_end.rs:82`, `mint_rule.rs:56`, `retired_ids_and_curation.rs:55`,
`cli_end_to_end.rs` header), so no test resolves `partner.steam-api.com`, which is Test
strategy #17's rule. `WriteLock::take` now refuses on presence alone
(`apps/registry/src/store_lock.rs:78-80,117-127`); `process_is_live` and `refuse_or_adopt`
are deleted, so the pid is a diagnostic and the double-write window between the pid read and
the `remove_file` is gone; the suites re-pin the crashed-run case and name the operator's
delete as the only recovery (`tests/common/mod.rs:432-455`, `tests/hosted_startup.rs:376-401`,
`tests/data_layout.rs:193-198`, `tests/issued_entries.rs:136-146`) — exactly the design's
§Concurrency posture plus my requested dead-owner coverage. Closed.

### 6 — the `retired id list full` rendering (closed)

`state.rs:702-710` returns `AdminRefusal::Named(Named::RetiredIdListFull)`;
`admin.rs:231-240,329` renders it as `503` + `named.reason()` +
`refusal_body(named.code(), …)`, so the admin face answers `503 retired id list full` with
`error: "retired_id_list_full"` instead of `503 Bad Request` / `bad_request`. Closed.

## Open and new findings

### F-1 — major — the store-hop assurance property is still not tested; the leg that claims it passes for an I/O reason

**Location:** `apps/registry/tests/steam.rs:1053-1131` (loopback fake at 1068-1090, chain
clause at 1122-1129) against `apps/registry/src/steam.rs:522-530,774-787`. **This file is
byte-identical across the reviewed range** (`git diff 75cbc43..HEAD --
apps/registry/tests/steam.rs` is empty), so this is my round-1 finding 2 unchanged.

`the_production_transport_refuses_an_unauthenticated_loopback_listener` drives the verified
transport against a plain `TcpListener` that reads bytes and holds the socket
(`steam.rs:1068-1090`). Nothing in `apps/registry` constructs a TLS server: grepping the crate
for `ServerConfig`, `Acceptor`, `rustls::server`, `CertificateDer`, `rcgen`, `.pem`/`.der`
returns nothing, and `Cargo.toml`'s only dev-dependency is `serde_json`
(`apps/registry/Cargo.toml:45-46`). The client therefore fails because **the peer never
answers the handshake**, an I/O condition — which is precisely what a `ClientConfig` mutated
to `dangerous().insecure_skip_verify()`, or built on an empty `RootCertStore`, still produces.
The assertion intended to pin the chain case is `assert!(!text.is_empty(), …)`
(`steam.rs:1125-1128`): satisfied by any failure. The leg's own header
(`steam.rs:1053-1058`) claims it "refuses a loopback listener that cannot chain to the pinned
authority"; the harness never presents a chain, so the claim is false as written and the
suite is green with the verifier removed.

Why it is security and not cosmetics: the design names this the *one* test of the property
that keeps the Web API key off an unauthenticated link — Test strategy #6: "the *production*
`StoreTransport` … driven against a loopback TLS listener presenting a self-signed cert *for
the name being verified* and one for another name; it refuses both handshakes, so the first
case fails on the chain, not the hostname. Without it, a verifier relaxed to accept the fake
passes the whole suite with the key on an unauthenticated link" — and §The store pull promises
"one TLS listener in the same family for the transport-assurance case". §Options' chosen
option is built on exactly this class of weakness (RUSTSEC-2026-0381/-0382 named as "the
tooling to impersonate the authority carrying our key"; A14's CWE-319/CWE-1428), and the key
rides the query string of that request (`steam.rs:650-663`). The shipped product is correct
today; what is unpinned is that it *stays* correct, on the one hop this ADR decides to
authenticate. The `the_verify_name_is_independent_of_the_connect_address` leg
(`steam.rs:1136-1187`) does add real coverage — the SNI/`Host` carries the authority, not the
connect address — but it too fails for the no-TLS reason.

**Fix:** make the fake speak TLS, as the design already specifies: drive
`rustls::server::Acceptor` on the loopback listener with a clearly-labelled self-signed
end-entity cert + key whose SAN is `partner.steam-api.com`, and a second listener whose SAN is
another name (either a committed `0600`-sized DER fixture pair in `apps/registry/tests/data/`,
or a `rcgen` dev-dependency — the crate is test-only, so it costs the product graph nothing).
Assert, for both listeners, that (a) the client's pull fails, (b) the failure names a
certificate/chain condition (`UnknownIssuer`/`InvalidCertificate`/bad-signature wording) and
not `timed out`/`connection reset`/`steam pull deadline`, and (c) no `GET `, `GetAppList` or
key bytes reach the socket — the last two assertions already exist at
`steam.rs:1109-1121`. Replace `!text.is_empty()` with (b), and delete the "self-signed for the
very name being verified" comment until the listener really presents one.

### F-2 — minor — nothing in the suite pins the new canonical-appid gate, so the identity-uniqueness property can regress silently

**Location:** `apps/registry/src/state.rs:378-387` (new gate);
`apps/registry/tests/hosted_lookup_policy.rs:318-330` (the only spelling case pinned).

The gate is new product behavior added to answer F-1's r1 sibling, and the suite has exactly
one external-id spelling case, `steam/not-an-appid`. A revert to
`external_id.parse::<u64>().ok()` — the shape the round-1 code carried, and the shape
`str::parse` invites — keeps every current leg green: `steam/042` would again mint a second
immortal identity (`state.rs:452` spells `source_keys` from the queried key, `mint.rs:97,106,109`
spell `steam-<external_id>` and the collision suffix from the same string), and `027-L2` ids
are unrecoverable once served. The design's Test strategy #3 pins only the "non-decimal"
half, so the missing case is this change's own addition and it needs its own pin.

**Fix:** one small leg, on either face: against a fresh snapshot carrying appid `N`, query
`steam/0<N>`, `steam/+N`, `steam/0…0N` and `steam/N`; assert the first three answer `200 []`
and the fourth answers the minted id, and assert the issued document holds exactly **one**
record for appid `N` (one `steam/N` source key, one `game_id`, `revision` moved once). Add the
mirror assertion at the seam in `apps/registry/src/mint.rs`'s tests only if the pin is put
there rather than in `hosted_lookup_policy.rs`.

### F-3 — nit — the `rustls-webpki` floor dependency carries a ceiling the design does not give it

**Location:** `apps/registry/Cargo.toml:36-41`; resolved versions at `Cargo.lock:254`
(`rustls 0.23.46`), `:277` (`rustls-webpki 0.103.17`), `:384` (`webpki-roots 1.0.12`).

Declaring rustls' verifier as a direct floor-only requirement is the right mechanism, and
today's lock sits at or above every floor §Options names. But the declared
`rustls-webpki = ">=0.103.16, <0.104"` forecloses the other patched line the same sentence
names ("`rustls-webpki ≥ 0.103.16` / `≥ 0.104.0-alpha.8`"): the first time a `rustls 0.23.x`
patch — including one issued against the very `Debug`-disclosure class RUSTSEC-2026-0346 is
about — requires `rustls-webpki 0.104`, resolution fails and the only way to take the security
fix is to edit this line. The ceiling also does no floor work: the floor is the `>=` half.

**Fix:** state the floor without the invented ceiling — `rustls-webpki = ">=0.103.16"` (cargo
still intersects it with rustls' own requirement, so no incompatible verifier can be selected),
or keep the range but name the ceiling's owner in the comment and cross-reference
§Options' 0.104 clause so the next bump knows it is a decision, not an accident.

## Checklist walk (security)

1. **External input validated at the boundary** — store page and snapshot document:
   `decode_page` refuses a non-envelope/no-`apps`/non-`appid` page by name
   (`steam.rs:1034-1072`), names are control-stripped then clamped
   (`steam.rs:1084-1090`), declared page length refused before the body is drained
   (`steam.rs:945-985`), header block bounded by `MAX_RESPONSE_OCTETS`
   (`steam.rs:885-888`); queried keys reach `state.rs` through the unchanged `keys.rs`
   admissibility rule and the new canonical-appid filter. OK.
2. **Path handling** — no new path inputs; document paths are the fixed names under `<data>`,
   `--corpus` is operator-chosen, and the new `same_document` (`main.rs:365-390`) only ever
   *spares* a file from a refusal (dev/inode on unix, case-insensitive spelling elsewhere).
   No traversal, no interpolated shell. OK.
3. **Injection** — `form_encode` (`steam.rs:1095-1106`) percent-encodes everything outside
   RFC 3986's unreserved set, so the environment key cannot smuggle a query delimiter or a
   CRLF into the request line; the rest of the request is constants and `u64`s
   (`steam.rs:650-663`); `Host:` is the compile-time authority. No regex. OK.
4. **Secrets and PII** — key from `BINDFORGE_STEAM_WEB_API_KEY` only; no pull-failure line
   carries the key, URI or path (`steam.rs:462-467`, pinned by
   `tests/steam.rs:1020-1051`); fixtures are throwaway keys; the handshake legs assert no key
   bytes leave the process (`steam.rs:1109-1115`), which holds genuinely because rustls buffers
   application data until the handshake completes; `Answer::named` renders name+detail only.
   The transport-assurance *test* gap is F-1, not a leak. OK.
5. **New dependencies** — no new dependency versions enter in this range beyond the declared
   floors and the floor-only `rustls-webpki` declaration; resolved `rustls 0.23.46`,
   `rustls-webpki 0.103.17`, `webpki-roots 1.0.12`, `ring 0.17.14` sit at/above §Options'
   floors, and the new `advisory-gate` job (`.github/workflows/check.yml:83-126`) runs
   `cargo deny check advisories` (pinned `cargo-deny 0.20.2`, `contents: read`, 30-minute
   timeout, own toolchain) so a re-resolution onto an advised version turns the PR red — that
   is Rollout 3's screen and it is the right shape. Ceiling nit at F-3. OK.
6. **Filesystem writes** — snapshot publish and issued/retired writes keep the fixed-tmp
   `create_new` + `0600`/`0700` owner-only, rename-into-place discipline
   (`persist.rs:178-212`, `steam.rs:205-221`); the mint's new fast path serializes the exact
   buffer it renames and its bytes are pinned equal to the whole-document spelling
   (`corpus.rs:341-424`, unit test at `corpus.rs:548-594`), so the cheaper write cannot drift
   from what the loader and the next mint read; `write.lock` presence is now the whole
   cross-process rule. OK.
7. **Clean rejection** — the new `AdminRefusal::Named` arm renders a named 503 rather than
   falling through to `"Bad Request"`; mint and cap refusals answer named 503s with model,
   `revision` and file untouched (`state.rs:438-471`); no new panic/unwrap path on hostile
   input in the diffed lines (`to_bytes_with` and `insert` are total functions on already
   validated records). OK.

## Not applicable / out of specialty

- Items on the serving face's auth posture, candidate budgets and health-shape re-pins:
  unchanged in this range beyond the renders noted above.
- FYI (no severity): `apps/registry` is now two targets over one module set
  (`src/lib.rs:23-32` adds `corpus`, `keys`, `mint`, `retired`, `state`), so the library copy
  the re-pinned stale/cold legs drive and the binary copy that serves are the same source
  compiled twice — behaviorally identical, but the stale-snapshot rows of Test strategy #3/#13
  (`health_route.rs:84-124`, `hosted_lookup_policy.rs:14-36`) now pin the library's `State`,
  not the process's. Also FYI: `SteamStore::for_test` is `pub` on the library face, so the
  plaintext arm stays reachable by name from anything that links the crate; the integration
  suites' loopback fake is what forces it to be non-`#[cfg(test)]`.
