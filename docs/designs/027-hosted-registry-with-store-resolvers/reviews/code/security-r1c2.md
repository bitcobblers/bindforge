# Code review — security seat, round 1 dissent pass c2

- **Seat:** security (checklist `.opencode/skills/code-review/references/security.md`)
- **Pass:** c2 — dissent. Scope: `git diff f52686fecdc794062dc72f6d71c58262d2029fb9..HEAD -- ':(exclude)docs/designs'`
- **Change under review:** 3 files, +146/−7 — `apps/registry/tests/admin_auth.rs`,
  `apps/registry/tests/retired_ids_and_curation.rs`, `apps/registry/tests/steam.rs`
  (module doc only). No product source, no manifest, no lockfile in this increment.
- **Inputs used:** that diff; `docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md`
  (full current document); `docs/adrs/027-hosted-registry-with-store-resolvers.md`; the
  runner's gate record (green: `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`);
  my own round-1 findings. No review report or transcript was opened.
- **Prior findings re-checked first**, per the pass instruction. Their state, then the new code.

## Verdict

`request-changes` — one **major** carried open from round 1, addressed by documentation
rather than by the code the design asks for.

| Round-1 finding | State in this increment |
|---|---|
| 1 (major) — #6's transport-assurance pair has no TLS listener | **Open.** The legs are byte-for-byte unchanged; the only change is the new module-doc section `apps/registry/tests/steam.rs:58-70` recording the gap |
| 2 (minor) — canonical-`appid` gate has no test | **Open.** No leg anywhere in the tree pins a padded or `+`-prefixed appid spelling (`grep -rn "steam/0" apps/registry/tests` empty; `grep -rn "not-an-appid"` still hits only `tests/hosted_lookup_policy.rs:323`) |
| 3 (nit) — `rustls-webpki = ">=0.103.16, <0.104"` invented ceiling | **Open, unchanged.** `apps/registry/Cargo.toml:41`; resolved still `rustls 0.23.46` / `rustls-webpki 0.103.17` / `webpki-roots 1.0.12` (`Cargo.lock:254,277,384`) |

## Checklist

1. *External input validated at the boundary* — **pass, improved.** The new
   `retired_ids_and_curation.rs:349-436` leg drives the issued delete with decoded segments
   that carry `../`, an encoded `/`, a control byte, `.`, `_`, uppercase, the empty and the
   129-octet spelling, through the harness's RFC-3986 encoder
   (`tests/common/mod.rs:1138-1149`, `1157-1159`, `tests/retired_ids_and_curation.rs:142-144`),
   so the registry must decide *after* decoding — which is what §The curation face's
   segment clause for `DELETE /v1/admin/issued/<game_id>` requires. `admin.rs:288-321`
   runs both deletes through the same `game_id_is_valid` gate.
2. *Path handling* — **pass, newly pinned.** Dot-dot and encoded-separator segments are
   refused (`admin.rs:109-134` rejects a segment containing `/` at dispatch) and the leg
   asserts the redirected `<data>` tree is identical before/after
   (`retired_ids_and_curation.rs:374, 423-430`) and no `retired-ids-v1.json` appeared
   (431-435). Scope note: the tree witness covers only the redirected `<data>`; a hostile
   segment that deleted *outside* it would not be caught here. Not raised as a finding —
   a `game_id` reaches no filesystem path (deletes rewrite documents under the fixed-tmp
   discipline), so no reachable traversal exists to witness.
3. *Injection* — not applicable to the increment: no shell, no regex over hostile input;
   the refusal echo path is the shipped `bindforge_core::escape_and_truncate`
   (`admin.rs:294, 313`), unchanged here.
4. *Secrets and PII* — **pass.** Every added row is a refusal leg; the auth matrix asserts
   no token echo (`admin_auth.rs:188-197`, and the issued route is now in it at 326-331),
   and the transport legs keep `!text.contains(KEY)` (`tests/steam.rs:1115-1118`).
5. *New dependencies* — **none added in this increment**; resolutions unchanged from
   round 1. Open nit 3 (the invented `<0.104` ceiling) stands.
6. *Filesystem writes* — **pass.** The new legs assert refused requests write nothing, and
   the `Origin` matrix (`admin_auth.rs:311-379`) now carries a mutating call on the new
   route with the corpus byte-identity and `stats`-equality witnesses.
7. *Clean rejection of hostile input* — **pass.** 401/404/400/422 posture for the new route
   is pinned identically to the corpus routes (`admin_auth.rs:66, 271-273` and the
   equivalence leg), no `Allow` list on a disabled face, no 2xx on any malformed segment.

The increment's own content is clean; the open major is the one round 1 named and the
revision answered in prose.

## Findings

### 1. major — #6's transport-assurance pair still has no TLS listener; the revision documents the gap instead of closing it

**Location:** `apps/registry/tests/steam.rs:58-70` (new module-doc section) driving
`apps/registry/tests/steam.rs:1067-1145` (unchanged), against
`02-proposal.md` §The store pull ("in tests … with **one TLS listener in the same family**
for the transport-assurance case (§Test strategy #6)") and §Test strategy #6's
Transport-assurance clause ("driven against a loopback TLS listener presenting a
self-signed cert *for the name being verified* and one for another name; it refuses both
handshakes, so the first case fails on the chain, not the hostname. Without it, a verifier
relaxed to accept the fake passes the whole suite with the key on an unauthenticated
link"), plus §Revision log r1→r2, which records this as "the property no test had".

**Problem.** The fake is still a plain `TcpListener` that reads bytes and holds the socket
(`1082-1104`); the client's failure is an I/O timeout against a peer that never answers the
handshake. `apps/registry/src/steam.rs:522-530` builds the one `ClientConfig` the tests
drive, and nothing in this crate can answer it: `grep -rn "rcgen|rustls::server|ServerConfig|Acceptor|CertificateDer|PrivateCert"`
over `apps/registry` and both manifests and `Cargo.lock` is empty, and `find` over the
repository finds no `.der`/`.pem`/`.key`. So a `ClientConfig` mutated to
`dangerous().insecure_skip_verify()`, or built on an empty `RootCertStore`, produces the
identical failure and the identical byte record, and every added and existing assertion
stays green — including the clause that is supposed to pin the chain case, which is still
`assert!(!text.is_empty(), …)` (`1139-1142`), satisfied by any refusal text. The new
module-doc section is honest about this and is a genuine improvement over round 1 — but it
changes no assertion, and it does not amend the design. Two claims in the surrounding
comments are now actively inconsistent with it: the leg header still says the pair "refuses
a loopback listener that **cannot chain to the pinned authority**" (`1069-1070`) and the
inline comment still says "a self-signed listener for the very name being verified must
still be refused" (`1137-1138`) when no listener presents any certificate; and the new
section overstates the replacement in the other direction, claiming the fake "does show … a
transport that writes the request — and so the key — before the handshake is assured"
(`66-68`) — with `Wire::Verified` a rustls `StreamOwned` (`src/steam.rs:776-787, 851-871`),
application data written before the peer is trusted is buffered in-process and never
reaches the watched socket, so that case is not observable either; what the fake really
catches is a transport that dials or writes cleartext.

Why this is still major rather than a documented accepted risk: the design names this leg
as the *only* thing standing between a relaxed verifier and the Web API key travelling on an
unauthenticated link — the very weakness (§Options' rejected plaintext pull, A14's
CWE-319/CWE-1428) that the whole `rustls` dependency was taken to close, on the one hop
`027-R3`'s edge deferral explicitly does not cover. The product code is correct as written
(`production_client_config` with `webpki_roots` and `ServerName::try_from(self.verify_name)`);
that is precisely why no gate failure marks the gap, and why the only remaining defence
against a future loosening of those five lines is this leg. A deviation from a named Test
strategy clause is a design amendment (the proposal's own §Test strategy / §Revision log),
not a module comment in the suite that is missing the clause.

**Fix.** Either close it or route it through the document; the round-1 fix stands, with a
fixture-free fallback:

- Close it, cheapest first: add `rcgen` as a `[dev-dependencies]` (or commit a labelled
  DER cert/key pair under `apps/registry/tests/data/`) and drive
  `rustls::server::Acceptor` on the loopback listener — listener A presents a self-signed
  end-entity cert whose SAN is `partner.steam-api.com` (the name being verified), listener B
  another name. For both, assert the pull fails, that the refusal names a
  certificate/chain condition (`UnknownIssuer` / `InvalidCertificate` / bad signature)
  rather than `timed out` / connection reset / `steam pull deadline`, and keep the existing
  no-`GET`/no-`GetAppList`/no-key-byte assertions (`1123-1135`). Then replace
  `!text.is_empty()` with the chain-condition assertion and delete the `1137-1138` comment
  that describes a listener which does not exist.
- If a dev-dependency is judged out of budget: call the production config's verifier
  directly — `config.dangerous().verifier.verify_server_cert(anchor, &ServerName, &[CertificateDer
  for a self-signed `partner.steam-api.com` leaf], &[], now)` — and assert it returns
  `Err(InvalidCertificate(UnknownIssuer))`. That pins "the verifier this crate ships rejects
  an untrusted chain for the verified name" without a TLS listener. Weaker than the
  designed leg (it does not exercise the connect path), and it still needs a cert.
- Whichever: fix the two overclaiming comments (`1069-1070`, `1137-1138`), correct the
  "writes the request … before the handshake is assured" claim (`66-68`) to what the fake
  actually catches (a cleartext dial or write), and record the reduction against
  §Test strategy #6 in the proposal's Revision log so the panel — not a test file — owns it.

### 2. minor — the canonical-`appid` gate still has no test

**Location:** `apps/registry/src/state.rs:378-387` (`parse::<u64>().ok().filter(|appid|
appid.to_string() == external_id)`); the only external-id spelling shape the suite pins is
`apps/registry/tests/hosted_lookup_policy.rs:323` (`steam/not-an-appid`).

**Problem.** Unchanged from round 1 and unaddressed by this increment. Nothing pins that a
zero-padded or `+`-prefixed spelling of an appid the fresh snapshot knows mints nothing.
Reverting the gate to the bare `parse::<u64>().ok()` keeps every current leg green while
re-opening the property the gate's own comment (369-377) names: `source_keys` is spelled
from the queried key (`state.rs:452`) while the mint's appid spellings come from the
snapshot record (`state.rs:395-400`), so `steam/042` passes the dedup probe (434-437, keyed
on the queried text), finds the plain slug held, and takes the `<slug>-steam-<appid>`
collision spelling as a **second immortal `027-L2` identity** for one snapshot record —
`027-B3`'s one-mint-per-key cap arithmetic and §Issued entries steps 3/5's "appid-unique"
premise both included. §Test strategy #3 names only the non-decimal half, so this case
belongs to the new gate and needs its own pin.

**Fix.** One leg, either face: against a fresh snapshot carrying appid `N`, query
`steam/0N`, `steam/+N`, `steam/0…0N` and `steam/N`; assert the first three answer `200 []`
and the fourth answers the minted id; then assert the issued document holds exactly one
record for `N` — one `steam/N` source key, one `game_id`, `revision` moved once.

### 3. nit — the `rustls-webpki` floor declaration carries a ceiling §Options does not

**Location:** `apps/registry/Cargo.toml:41` — `rustls-webpki = ">=0.103.16, <0.104"`;
resolved `Cargo.lock:277` (`0.103.17`).

**Problem.** Unchanged from round 1. §Options names `rustls-webpki ≥ 0.103.16` *and*
`≥ 0.104.0-alpha.8` as two equally-patched lines; with the `<0.104` half in place, the first
`rustls` 0.23.x security patch that needs `rustls-webpki` 0.104 cannot resolve, and taking
that fix means editing this line — while the ceiling does no floor work, since the floor is
the `>=` half and cargo intersects the range with `rustls`'s own requirement anyway. The
`027-L4`/CWE-319 credential-transport hop is exactly the one this crate must be able to
re-resolve onto a patched verifier.

**Fix.** State the floor without the invented ceiling (`rustls-webpki = ">=0.103.16"`), or
keep the range and say in the comment who owns the `<0.104` decision, cross-referencing
§Options' `0.104.0-alpha.8` clause so the next bump reads it as a decision rather than an
accident.

## Evidence and limits

- Gate record (green, `2026-11-22T11:13:49Z`) is consistent with all three findings: none
  of them describes a failing leg — they describe assertions that cannot fail.
- Absence evidence for finding 1 was taken from the tree, not from a report: no TLS server
  type, acceptor, cert type or `rcgen` anywhere in `apps/registry`, either manifest, or
  `Cargo.lock`; no certificate fixture anywhere in the repository.
- Not re-reviewed here: the c1 increment's product code (`mint.rs`, `persist.rs`,
  `store_lock.rs`, `wire.rs`, the retired-ids cap arithmetic, the `serve.rs` request
  framing) — outside this pass's scope and not re-touched by it. Nothing in this
  increment's changed lines makes a new claim about that code that I could not verify.
