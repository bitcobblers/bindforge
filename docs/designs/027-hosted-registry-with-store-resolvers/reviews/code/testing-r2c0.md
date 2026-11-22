# Code review — testing seat, round 2, dissent pass c0

- **Change under review:** `git diff f6d24bca3d755f79e8f26ac938d9ffee9372c983..HEAD -- ':(exclude)docs/designs'`
  (7 files: `apps/registry/src/steam.rs`, `apps/registry/tests/steam.rs`, `apps/registry/tests/fixtures/store/{README.md,partner.steam-api.com.crt.der,partner.steam-api.com.key.pkcs8.der,some-other-name.example.crt.der,some-other-name.example.key.pkcs8.der}`)
- **Design:** `docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md` (Test strategy #6's
  transport-assurance pair at :621-629; #7 at :630-631), `docs/adrs/027-hosted-registry-with-store-resolvers.md`.
- **Inputs used:** this diff, the proposal, the ADR, the runner's gate record, my round-2 findings. No other
  review report or transcript opened.
- **Specialty applied:** `references/testing.md` only.

## Gate evidence (runner-supplied) and checks I executed

Runner gate at `2026-11-22T11:13:49.574Z` — green: `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` all exit 0.
The single captured `test result: ok. 0 passed; 0 failed; …` line is one zero-test target's summary, not the
steam suite's, so it says nothing about the new legs. I therefore ran the affected suite myself, to answer
the one coverage question this pass turns on (do the new transport legs compile, run, and stay green):

- `cargo test -p bindforge-registry --test steam --locked -- --exact the_production_transport_refuses_self_signed_store_certificates --nocapture`
  → `test result: ok. 1 passed; 0 failed; 0 ignored; … finished in 0.40s`
- `cargo test -p bindforge-registry --test steam --locked`
  → `test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.40s`

The new leg is `#[test]`, not `#[ignore]`d, and ran. No failing leg anywhere. `rustls` is a regular
(production) dependency of `apps/registry` (`Cargo.toml:30-34`, unchanged in this range), so the fixture
server compiles inside the integration target with no manifest or lock change — and it is the *same* rustls
build the production client uses.

## Resolution of my round-2 major (finding 1) — **closed**

The revision took the testing branch of my r2 fix, not the documentation branch, and took it fully:

1. **Fixtures landed.** `tests/fixtures/store/` carries two self-signed ECDSA leaf/key pairs, one per name,
   plus a README with the exact generation command (`-addext subjectAltName=DNS:$NAME`, `CA:FALSE`) and a
   stated refresh horizon. The key/cert pairing is validated by rustls itself at leg runtime
   (`with_single_cert(...).expect(...)`, `steam.rs:1092-1101`), so a corrupt or mismatched fixture panics
   loudly rather than passing.
2. **Both #6 cases are now driven over real TLS** (`steam.rs:1083-1131`): a loopback **rustls server**
   presenting each fixture, the client being `production_client_config()` — webpki roots only — installed by
   `with_production_transport(verify_name)` at the *test address* (`src/steam.rs:580-584`, `776-786`). Both
   handshakes must refuse (`expect_err`, `:1131`).
3. **The chain-vs-hostname discrimination is now an assertion, not a stub.** `assert!(!text.is_empty())` is
   gone; the verified-name case asserts `text.contains("UnknownIssuer")` (`:1141-1150`). That assertion is
   load-bearing on new production code: `io_reason` (`src/steam.rs:835-848`) is what puts the carried
   `rustls::Error` Display into the refusal at the write site (`:697-702`), so the leg pins both the transport
   posture and the reason-carrying refusal at once. My run proves the text really carries the condition.
4. **The mutation catcher is real.** The fake now watches its *decrypted* stream (`serve_tls_fixture`,
   `:1167-1193`) and the leg asserts emptiness (`:1153-1163`) — the exact shape #6's rationale
   (proposal :627-628: "a verifier relaxed to accept the fake passes the whole suite with the key on an
   unauthenticated link") describes. I traced the mutations that matter: relax the chain check → handshake
   completes → decrypted bytes → red; drop TLS for the verified arm or ignore `verify_name` → the refusal
   stops saying `UnknownIssuer` → red. The old cleartext-listener legs could answer neither.
5. **The prose contradiction I reported is resolved in the true direction.** Header `:3` ("the production
   transport's assurance half") and `:23` ("and its transport half") are now claims the suite keeps; the
   gap-note section (`:58-69`) records what actually runs; the leg doc (`:1066-1074`) no longer claims a
   listener "cannot chain to the pinned authority" while presenting nothing.

No test dependency, no port, and no outbound dial was added: both cases bind `127.0.0.1:0` and dial that, so
#17's no-outbound rule holds (proposal :628-629).

**Missing evidence, recorded explicitly.** I could not read the checked-in DERs to confirm each fixture's
`subjectAltName` carries its filename's name: my seat's shell allowlist covers `git diff*`/`cargo`, not
`openssl`, `ls` or `sed`; the `read` tool refuses binary; and ripgrep skips the binary files (it finds
`partner` only in the README). The claim is therefore backed by the README's documented command, not by the
bytes. This is *not* blocking: rustls builds the chain before comparing the name, so a chain-only relaxation
is caught either way (with a name-less fixture it lands as `NotValidForName`, which still fails the
`UnknownIssuer` assertion). Only the comment's stated rationale depends on it — see N2.

## Open findings from this diff

### N1 (nit) — the second iteration of the pair has no teeth; a fake that never answered could pass it
`apps/registry/tests/steam.rs:1083-1164`, with `:1175-1177` and `:1155`.

`the_production_transport_refuses_self_signed_store_certificates` asserts a refusal condition only in the
`verify_name == STORE_AUTHORITY` branch. For `some-other-name.example` the assertions are "the pull refused"
and "the fake decrypted nothing" — a shape the leg also produces if the fixture server never speaks TLS at
all: `serve_tls_fixture` returns silently on `ServerConnection::new` failure (`let Ok(connection) = … else {
return }`), and a client that dials a bound-but-unanswered socket gets `ConnectionRefused`-shaped retries
and an empty `seen` too. Iteration 1's liveness is pinned only indirectly, by its `UnknownIssuer` assertion.
The emptiness check additionally depends on a fixed `sleep(150 ms)` (`:1155`), so on a loaded machine the
failure direction is a false green, not a false red.

*Fix:* count accepted connections with an `AtomicUsize` bumped in the accept loop (`:1115`) and assert it per
case — the pull is `STEAM_PAGE_ATTEMPTS` (3) dials, so `assert_eq!(accepted, STEAM_PAGE_ATTEMPTS)` makes a
silent fake red in both iterations — and name a refusal condition for the mismatched-name case (its
`InvalidPeerCertificate`/`UnknownIssuer` text is available from the same `io_reason` enrichment) so neither
iteration can pass on absence alone.

### N2 (nit) — the chain-vs-name rationale rests on a fixture property nothing pins
`apps/registry/tests/steam.rs:1141-1150` ("the hostname fits") against
`apps/registry/tests/fixtures/store/*.crt.der`.

The leg's discriminating power comes from the verified-name fixture actually presenting
`partner.steam-api.com` in its SAN. No leg asserts that, and no reviewer can check it with the run's
read-only tools (see the missing-evidence note above). If a future regeneration drops the `-addext` (the
README's command is prose, not executed by any gate), the case silently degenerates into a second copy of
the other-name case while the comment still claims the opposite.

*Fix:* pin the fixture's identity in the leg — a few lines that parse the DER's SAN (or, cheaper and stronger,
a companion leg that builds a client with the fixture as an added anchor and asserts that handshake
*completes*) — so "chain, not hostname" is an observed property of the fixture rather than a comment.

### N3 (nit) — the new enrichment is pinned at one of its three call sites
`apps/registry/src/steam.rs:835-848` reached from `:700` (write), `:913` (header read), `:1040` (body fill).

The `InvalidData` arm is exercised only through the write site, where the handshake fails. The two read
sites' enriched arm covers a post-handshake TLS record failure, and no fixture in the suite can ever complete
a handshake to produce one — production-visible behaviour (a torn or forged record mid-page) asserted only by
the docstring's parenthetical. Not worth blocking; worth a line in `03-build-notes.md` as an unpinned arm, or
a fixture the client trusts if the run ever grows one.

### N4 (nit) — the carried marker-numbering ambiguity grew one more leg
`apps/registry/tests/steam.rs:1066` (and `:1195`): both transport legs are marked with the bare
`/// proves: Test strategy #6 …`, so the re-pinned 026 suites' markers and 027's still share one numbering
space (a grep for `Test strategy #16` mixes a 026 health pin with a 027 leg); only the file headers
disambiguate. Prefix with `design 027's` as the headers do.

## Carried findings — status (all in files this range does not touch; byte-identical to my round-2 read)

Re-verified this pass by grep over `apps/registry`; all still open. None blocks: they are five minors and
four nits, unchanged in kind and count from the previous round.

1. **minor** — `src/state.rs:461-466`: the `MAX_ISSUED_DOCUMENT_OCTETS` check on the mint/persist-before-serve
   path (now inside `Catalog::to_bytes_with`) has no leg; the only exercise is the load-side metadata gate
   (`tests/issued_entries.rs:510-555`, the suite's only `MAX_ISSUED_DOCUMENT_OCTETS` mentions), while the
   header at `:7-11` and Test strategy #7 (proposal :630-631) both promise the cap "refuses the same way".
2. **minor** — `src/steam.rs:426-429, :437-469`: zero `startup_snapshot` call sites under `apps/registry/tests`
   (grep: none), yet the docstring claims a test can watch which arm the `027-C3` startup decision takes, and
   three suites relocated their stale legs on that citation.
3. **minor** — `src/state.rs:369-387`: the canonical external-id filter is unpinned; the only non-decimal
   spelling queried anywhere is `steam/not-an-appid` (`tests/hosted_lookup_policy.rs:323`), which
   `u64::from_str` already rejected. No leg queries `steam/+…` or `steam/0…`.
4. **minor** — `src/steam.rs:620-625, :631-637, :340-345, :707-714`: still no leg for a 204, for an
   empty-`apps` full pull, for an empty continuation page that then claims finished, or for an `"apps": []`
   snapshot file (grep finds no `204` leg and no `"apps": []` plant); `attempt_page`'s `status != 200` rule at
   `:710-714` is asserted only by its comment.
5. **minor** — the retirement half of ADR-027 is still mass `#[ignore]` rather than deletion: 29 ignored legs
   across `cache_document.rs` (7), `caching_policy.rs` (11), `upstream_client.rs` (6),
   `upstream_endpoint_gate.rs` (3) plus `admin_mutations.rs:598` and `candidates_join.rs:331`, reasons still
   naming "pending the runner's deletion", with `src/{cache.rs,upstream.rs,endpoint.rs}` declared by neither
   `main.rs` nor `lib.rs` — a green `cargo test --workspace --locked` says nothing about them. Whether
   `03-build-notes.md` records the deletion as an outstanding runner action remains outside this seat's
   allowed inputs.
6. **nit** — `tests/mint_rule.rs:15-23` still says the `mint identity over bound` rung is "not pinned here or
   anywhere else in this package" and "needs a mint seam of its own … to be tested at all", while
   `src/mint.rs:316-330` pins exactly it in an unmarked unit.
7. **nit** — `tests/steam.rs:81-98`, `tests/issued_entries.rs:37-38`, `tests/retired_ids_and_curation.rs:35-36`
   re-spell budget constants as literals that `src/constants.rs` now owns and re-exports.
8. **nit** — see N4 (bare `proves:` markers), which this diff extended.
9. **nit** — `tests/retired_ids_and_curation.rs:349-436`: the issued-vs-corpus delete leg still asserts status
   equality with no named expected code per row, so one shared handler edit turns every row into `404 == 404`
   while A10's grammar refusal stops being enforced anywhere; and the two slash spellings' rows credit the 404
   to the grammar rule rather than to `admin::dispatch`'s `!segment.contains('/')` guard.

## Checklist walk

1. **Numbered Test-strategy items cited** — #6's transport half now has its pin (major closed); #1's last rung
   is pinned only by an unmarked unit and its suite header contradicts it; #7's octet-cap clause is half-pinned
   (load side only). Recorded above.
2. **Unit and integration layers where promised** — present: the library-face in-process suites (`steam.rs`,
   `state`, `issued_entries`) plus `src/mint.rs`'s unit. Nothing in this diff changed the layering.
3. **Previously rejected blocker/major now has a catching test** — yes, my r2 major; the TLS fixtures plus the
   decrypted-stream assertion are the test that would have caught what I reported, and it fails in the right
   directions (traced above, green by execution).
4. **Determinism** — loopback only, port `0`, injected timing, real TLS handshake (no external network, no live
   server); fixture reads are checked-in bytes. The only wall-clock is the fixed 50/150 ms settle waits, whose
   failure direction is a false green — folded into N1.
5. **Names describe behaviour** — `the_production_transport_refuses_self_signed_store_certificates` names the
   rule; the helper `serve_tls_fixture` names its role. Fine.
6. **Gate evidence** — supplied gate green; my two executed runs green; the zero-test summary line the runner
   captured does not stand in for the new legs, hence my own run.
7. **Untestable additions** — `io_reason`'s `InvalidData` arm at the two read sites is the only addition with no
   reachable leg (N3); the fixtures have consumers; no dead config knob added.

## Verdict

**approve.** No blocker or major finding open: my r2 major is closed by executable evidence, and what remains
is four nits introduced or extended by this range (N1-N4) plus the unchanged carried set (five minors, four
nits) that this revision did not address and this pass's scope did not touch.

Out-of-specialty FYI, no severity: the two `.key.pkcs8.der` fixtures are committed private keys — correct for
self-signed test material the README says "certify nothing", and the security seat may want to confirm the
README's "never use anywhere else" is enough for them.
