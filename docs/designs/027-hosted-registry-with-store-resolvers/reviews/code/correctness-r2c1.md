# Code review — correctness seat, round 2 ratification (c1)

- **Pass:** ratification. Scope as given: `git diff f52686fecdc794062dc72f6d71c58262d2029fb9..HEAD -- ':(exclude)docs/designs'`.
  Question asked: did the post-approval fixes break what this seat approved at r2?
- **Inputs used:** that diff; `02-proposal.md` (full); `docs/adrs/027-hosted-registry-with-store-resolvers.md`;
  the runner's gate record; my r2 findings. No review report or transcript was opened.
- **Checklist:** `.opencode/skills/code-review/references/correctness.md`, correctness only.
- **Increment under review (9 paths, +278/−53):** `apps/registry/src/steam.rs` (a new `io_reason`
  helper at the three `Wire` error seams), `apps/registry/tests/steam.rs` (Test strategy #6's
  transport-assurance pair rebuilt against real loopback rustls servers), four checked-in
  self-signed fixture pairs under `apps/registry/tests/fixtures/store/` with a README,
  `apps/registry/tests/admin_auth.rs` (`027-C6`'s issued `DELETE` added to the auth matrices),
  `apps/registry/tests/retired_ids_and_curation.rs` (a new issued-vs-corpus segment-parity leg).

## Verdict

**approve.** Nothing in this increment breaks a behavior or a pin this seat approved at r2; the
production change is three call sites and one helper; the suite is green and stable under repetition.
No blocker or major is open. Findings below are one carried minor for the still-unlanded Rollout 1/6
records, one carried minor for the still-unpinned `startup_snapshot` arms, and nits on the new tests.

## Checklist walk

1. **Components/interfaces named.** The increment adds no interface. It uses the two the design's
   #6 names: the test constructor taking connect address and verification name separately
   (`steam.rs:556` `for_test`, `steam.rs:580` `with_production_transport`, which installs
   `production_client_config()` — the same webpki-roots `ClientConfig` as `production()` at
   `steam.rs:522`). Address-vs-name separation is real, and `Endpoint::Address` means the pair never
   resolves `partner.steam-api.com`, so #17's no-outbound rule holds for both legs.
2. **Assumption register.** No row is silently changed. A3/A17 untouched; A7's std-only serving face
   untouched (the change is inside the `rustls` seam plus a test); A10 is what the new parity leg
   pins, not what it changes. A8/A1/A15 not touched by these paths.
3. **Deviations.** One new production behavior: pull-failure lines for a `Wire` leg now carry the
   `rustls::Error` Display when the kind is `InvalidData` (`steam.rs:835-848`, used at `697`, `913`,
   `1040`). This is the minimum needed for #6's chain-vs-name assertion to read anything, so it is
   design-implied rather than creep; `03-build-notes.md` was outside this pass's inputs, so its
   declaration there is unverified — recorded as a nit, not a finding of concealment.
4. **Scope creep.** None. The fixtures exist because #6 demands a self-signed cert *for the verified
   name*; the two test-file additions are matrices for routes §The curation face and #10 already
   name. No behavior outside the named seams moved: the three converted sites are exactly the
   `Wire` read/write sites (checked against `steam.rs:865-883`, `904`, `1031`); the remaining
   `error.kind()` uses (`steam.rs:759`, `765`, `803`) are the plain-socket and resolver seams, which
   can never carry a rustls error, so leaving them bare is consistent, not an oversight.
5. **Build-notes claims.** Not readable in this pass; nothing in the diff contradicts the proposal.
6. **Error paths fail closed.** `io_reason`'s added text is bounded (rustls error enums' Display; no
   request bytes, no URI, no key) and the leak discipline is re-pinned on the new path:
   `tests/steam.rs:1137-1140` asserts the refusal carries no key in *both* name cases, beside the
   cleartext fake's `no_pull_failure_line_carries_the_key_or_the_request_uri`. The new parity leg
   fails closed on writes: the issued document is byte-compared, the file tree is set-compared, and
   `retired-ids-v1.json` must still not exist.
7. **Prior rounds.** No r2 blocker or major existed. The r1 rendering fix for
   `retired id list full` still renders (named 503 at `state.rs:144`/`705`, `admin.rs:329`), and the
   behavior this seat approved in `admin.rs:304-321` (issued delete shares the corpus grammar check
   and the `Absent` shape) is unchanged and now has a leg of its own.

## What the increment establishes (positive findings)

- **#6's transport-assurance pair is now a real pair.** `tests/steam.rs:1076-1165` drives the
  production `ClientConfig` against a loopback rustls server presenting a self-signed cert whose
  name *is* the verified name (so the refusal must be chain-shaped) and a second for another name,
  and asserts `UnknownIssuer` in the first case. The old test's plaintext listener could only show
  "the peer speaks no TLS"; the README's own words say so. The empty-*decrypted*-stream assertion is
  the mutation catcher #6 names: a verifier relaxed to accept an untrusted chain completes the
  handshake and the fake's decrypted read collects the real `GET` with the key.
  I ran the leg six times: `test result: ok. 1 passed` each run, 0.40 s, no flakes; the accept-loop
  change (one accept thread, one bounded child per connection, 5 s read timeouts) is what lets the
  3-attempt page retry reach a live fake instead of a backlog.
- **#10's "auth posture identical to the corpus routes" is now matrix coverage, not prose**
  (`admin_auth.rs:66`, `:268-271`, `:323-329`): the issued delete is asked without a credential, with
  a wrong one, with a valid one plus an `Origin`, on a face disabled by a routable bind, and the
  collection spelling is pinned flat-404 with no `Allow`.
- **The new segment-parity leg has teeth.** `retired_ids_and_curation.rs:349-437` compares the two
  routes' status codes for the same decoded segment. Because the implementation answers an
  out-of-grammar segment `422` and an absent one `404` (`admin.rs:290-296` vs `:308-316`, `:327`), a
  mutation that dropped the issued route's grammar check would break parity rather than pass it; the
  `Outer-Wilds` row additionally catches a case-fold leak into the delete path, since that fold
  would hit the live `outer-wilds` record and answer 2xx. Segments reach the wire percent-encoded by
  `common/mod.rs:1138-1149`, so `steam/9455001` and `../../etc/passwd` arrive as one segment and the
  registry must decide *after* decoding — the rule `slug_decoding.rs` pins for the query side.
- **Gate plus my own runs are green at HEAD**: the runner's four legs, and locally
  `cargo test -p bindforge-registry` → every leg `test result: ok`, 0 failed.

## Findings

1. **minor — `apps/registry/src/steam.rs:437-471`; no test reference.** Carried from r2 #1,
   unchanged by this increment: `grep startup_snapshot` over `apps/registry/{src,tests}` still shows
   only the definition and the one `main.rs:362` call site, so none of the three load-or-pull arms is
   exercised and the age-bound guard stays deletable with a green suite. The fix from r2 stands
   (three `tests/steam.rs` legs against the existing fake: fresh-file-loads, past-bound-repulls,
   unloadable-repairs). Not a regression and not newly load-bearing, so still minor.
2. **minor — `docs/registry-hosting.md` (absent), `docs/adrs/026-registry-service.md`,
   `docs/adrs/024-game-slug-identity.md`.** Carried from r2 #3 and #4: this diff touches no `docs/`
   path, so Rollout 6's runbook and Rollout 1's ledger annotations are still unlanded, against a
   branch whose `write.lock` posture makes "delete a crashed run's leftover `write.lock`" the only
   recovery from any unclean stop.
3. **nit — `apps/registry/tests/retired_ids_and_curation.rs:589-600`.** Carried from r2 #5 and still
   not applied: the assertion accepts the name in "line or body" (`line.contains("retired") ||
   body.contains("retired")` ∧ full/cap in either), so a `503 Bad Request` naming the refusal only in
   prose still passes. The rendering fix is in place at `state.rs:705`/`admin.rs:329`, so the status
   line can be asserted directly.
4. **nit — `apps/registry/tests/steam.rs:1083-1151`.** The two legs are now indistinguishable in
   outcome: each presents a certificate whose name equals the name being verified, so both fail
   purely on chain-building and no leg can show that `verify_name` is plumbed through
   `connect`/`ServerName::try_from` (`steam.rs:777-784`) — hardcoding the authority there passes the
   pair, as it passed the r2 version. With webpki roots only this is structurally unfixable (a
   name-mismatch case needs a trusted chain), and §Test strategy #6 asks only for the chain-vs-name
   discrimination the first leg now gives. Record the residual in the build notes rather than
   leaving the pair's second leg read as name coverage.
5. **nit — `apps/registry/tests/steam.rs:1153-1163`.** The increment replaced the old raw-socket
   assertions ("no `GET `/`GetAppList`/key bytes reached the listener") with a decrypted-stream
   assertion. The design's named mutation is better caught than before, but the strictly-different
   property — that no request byte leaves the process before the handshake is assured, whatever the
   socket carries — is now pinned by `StreamOwned` rather than by the suite. A comment saying the
   pre-assurance byte rule moved to the transport would keep the shift honest.
6. **nit — `apps/registry/Cargo.toml:43-44`, `apps/registry/tests/steam.rs:1092-1101`.** The comment
   "integration tests see only dev-dependencies" is now contradicted by the suite building
   `rustls::ServerConfig` from the *product* dependency (it compiles because Cargo hands test targets
   the normal dependencies too). Either name `rustls` in `[dev-dependencies]` for the fakes or fix the
   sentence, so the seam the suite stands on is declared.
7. **nit — `.github/workflows/check.yml:83-126`.** Carried from r2 #7: this pass's gate record again
   has no `cargo deny check advisories` leg and no `--target x86_64-pc-windows-msvc` leg, so the new
   advisory screen's current greenness against this `Cargo.lock` stays unevidenced.

## Evidence cited

- Runner's gate record (2026-11-22T11:13:49Z, green): `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace --locked`.
- My own runs at HEAD: `cargo test -p bindforge-registry` (all legs `ok`, 0 failed);
  `cargo test -p bindforge-registry --test steam --test retired_ids_and_curation --test admin_auth`
  (12 + 12 + 27 passed, 0 failed); six repeats of
  `--test steam the_production_transport` → `test result: ok. 1 passed`, 0.40 s each.
- Source read for the walk: `steam.rs:471-584`, `660-815`, `835-848`, `865-883`, `904`, `1031`;
  `admin.rs:290-331`; `state.rs:218-237`, `144`, `705`; `common/mod.rs:1138-1159`;
  `tests/fixtures/store/README.md`.

## FYI (out of specialty, no severity)

Two `.der` private keys are checked in under `tests/fixtures/store/`. They certify nothing and the
README scopes them to the fixtures, but if the repo ever adds a secret scanner, that directory is the
one that will trip it.
