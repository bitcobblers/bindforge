# Code review — testing seat, round 1 dissent pass (c1)

Run: `027-hosted-registry-with-store-resolvers`
Scope reviewed: `git diff 75cbc433ed8a546734c41d4eb928395373ab73e6..HEAD -- ':(exclude)docs/designs'`
(21 files, +1,333/−533), read against the current `02-proposal.md` and
`docs/adrs/027-hosted-registry-with-store-resolvers.md`. Inputs: that diff, the
proposal, the ADR, the runner's gate record, and my round-1 findings. No other
review report or transcript was opened.

**Verdict: request-changes** — one round-1 major (Test strategy #6's
transport-assurance clause) is still open; nothing in the diff touches the
file that clause is pinned in. The other round-1 major is fully resolved, two
round-1 minors resolved, two carried, and the changed code adds three new
unpinned behaviors (minors).

## Gate evidence (checklist item 6)

The runner's gate (2026-11-22T09:51:31.853Z, ~40 s after HEAD `f52686fe`
committed): `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace --locked` all green, with the captured
`test result: ok. 0 passed; 0 failed; …` tail showing no failing leg. The gate
therefore covers exactly this revision and shows no failure. It is the fresh
re-pin's green baseline, and I read every suite claim below against it.

Checks I executed myself: `cargo check -p bindforge-core --target
x86_64-pc-windows-msvc` → "the `x86_64-pc-windows-msvc` target may not be
installed". The cross-compile leg remains **unavailable locally, as at round
1** — recorded as missing evidence, not a failure. CI-only jobs
(`advisory-gate`, the Windows leg) are likewise outside the gate record by
design; their content is pinned by `gate_contract.rs` instead.

## Round-1 findings — disposition

1. **#6 transport assurance (major) — STILL OPEN.** See finding T1.
   `apps/registry/tests/steam.rs` is not in the diff range; the plain-TCP
   listener that "answers nothing" is the shipped state, unchanged.
2. **Rollout 3 gate half (major) — RESOLVED.** `.github/workflows/check.yml`
   now narrows the cross leg (`--exclude bindforge-registry`), adds the host
   `cargo check -p bindforge-registry --all-targets` step, and lands the
   RUSTSEC advisory screen as its own job; `README.md` gained the C-toolchain
   prerequisite line; `apps/registry/Cargo.toml` declares the §Options floors
   verbatim (`rustls >=0.23.46,<0.24`, `webpki-roots >=1.0.10,<2`,
   `rustls-webpki >=0.103.16,<0.104` — proposal lines 94–99), and `Cargo.lock`
   resolves rustls 0.23.46 / rustls-webpki 0.103.17 inside them. I verified in
   `Cargo.lock` that `ring` is depended on only by `rustls` and
   `rustls-webpki`, and those only by `bindforge-registry` — so the narrowed
   leg no longer asks the cross runner for `ring`'s build script at all.
   `crates/core/tests/gate_contract.rs` re-pinned the job set
   (`rust`, `orchestration`, `advisory-gate`) and added
   `the_windows_compile_leg_skips_the_registry_and_the_host_leg_covers_it`,
   which cites #16/#17 — the split is now pinned, not just promised. The
   leg's own green run stays CI-side evidence (see the missing-evidence note
   above).
3. **Octet cap on the mint write path (minor) — OPEN** (carried, T2).
4. **Unreachable named refusals (minor) — RESOLVED.** `503 no steam snapshot`
   is now pinned on the policy's own face: `hosted_lookup_policy.rs` builds
   `State::hosted` with an empty snapshot slot and asserts
   `refusal_reason(absent.candidates(...)) == "no steam snapshot"`, and
   `the_steam_refusals_name_their_own_reason` keeps the three cold names
   distinct with the `--no-steam` word still read off the wire. `pub mod
   mint;` published, and the over-bound rung is in fact pinned by
   `src/mint.rs`'s unit `an_appid_spelling_over_bound_is_named_not_truncated`
   (my round-1 "pinned nowhere" was wrong about the crate's own unit module —
   see nit T7 for the header that still says otherwise).
5. **Retire-by-ignore (minor) — OPEN** (carried, T6).
6. **Budget-constant literals in suites (nit) — OPEN** (carried, T8); the fix
   actually widened the seam (the family moved to `constants.rs` and is
   re-exported by `steam.rs`), and the suites still re-spell it.
7. **Bare `proves:` markers (nit) — OPEN** (carried, T9); none of the named
   legacy files changed.

## Findings

### T1 — major — `apps/registry/tests/steam.rs:1053-1131` (unchanged in c1); design #6's transport-assurance clause is still unpinned

The clause (proposal lines 621–629) names the test: the production transport
driven "against a loopback TLS listener presenting a self-signed cert *for the
name being verified* and one for another name; it refuses both handshakes, so
the first case fails on the chain, not the hostname. Without it, a verifier
relaxed to accept the fake passes the whole suite with the key on an
unauthenticated link." `the_production_transport_refuses_an_unauthenticated_loopback_listener`
still binds a plain `TcpListener` that presents no certificate and answers no
bytes; the handshake dies on I/O for both name cases, so relaxing the
verifier changes no assertion; and the chain-not-hostname case is asserted as
the vacuous `!text.is_empty()` (:1125-1128). `Cargo.toml`'s `[dev-dependencies]`
holds only `serde_json` — no rustls dev-dep, no cert fixture — and no suite
file imports rustls, so the c1 revision took neither of the two branches my
round-1 fix named (land the TLS legs, or correct the claims at
tests/steam.rs:3-4 "the production transport's assurance half", :52-56, and
:1053-1058 and record the clause as unmet). The design's own revision log
(r1→r2, line 802) calls this "the property no test had" and re-promises it in
the literal fixture form; checklist item 3 ("each previously rejected
blocker/major now has a test that would have caught it") is unmet. The legs do
pin real properties (no `GET`/key before handshake; SNI is the authority, not
the connect address), which is why this is one clause missing rather than the
whole transport unpinned.

Fix: as before — add `rustls` (no default features) plus a checked-in
self-signed cert/key fixture as dev-dependencies, drive both named cases
against a real loopback TLS acceptor dialing `127.0.0.1`, and assert the
refusal names a certificate/chain condition; or correct the three header/leg
claims and record #6's assurance clause as unmet.

### T2 — minor — `apps/registry/src/state.rs:462-470` (mint write path); the octet cap beside the count cap is still untested

Carried unchanged. The suite (`tests/issued_entries.rs:7-8`) still promises
"the octet cap refuses the same way", the count cap is pinned by
`at_the_issued_cap…` (:350-396) and the load-side gate by :510-555, but no leg
plants a document under `MAX_ISSUED_ENTRIES` whose `to_bytes_with` serialization
crosses `MAX_ISSUED_DOCUMENT_OCTETS` and asserts the named `503` with file,
model, and revision untouched. The c1 refactor (`Catalog::to_bytes_with`) moved
the measurement to the spliced buffer, which makes a wrong `revision + 1` /
splice bug in exactly that function land on this unexercised branch. Fix as
before: one leg beside the count-cap leg, fat 128-octet ids/names one record
below the boundary.

### T3 — minor (new in c1) — `apps/registry/src/steam.rs:413-467`; `startup_snapshot`'s three arms have no test, and its doc claims they do

Commit `b2dd33e7` moved the literal `027-C3` startup decision (load only a
present, loadable **and** fresh snapshot; stale/absent/unloadable → full pull;
failed pull → the named refusal line, proposal lines 269–272) out of `main.rs`
into the library face, and its doc says the move exists so "a test can point a
[`SteamStore`] at a loopback fake and watch which arm the decision takes,
which is the only way `027-C3`'s clause and Test strategy #17's no-outbound
rule can both be pinned." No test does: `startup_snapshot` has zero call
sites under `apps/registry/tests` (grep: 0). The load arm is covered only
indirectly (suites boot on fresh plants); the *stale → pulls in full* arm —
the very posture change this revision made, replacing the recorded deviation —
and the *absent/unloadable + failed pull → named line* arm are pinned nowhere,
while `hosted_lookup_policy.rs`, `health_route.rs` and `cli_end_to_end.rs` all
justified relocating their stale legs by citing this startup rule. The seam is
trivially drivable: `startup_snapshot(&SteamStore::for_test(fake_addr, KEY),
path, now)` over the existing `loopback` fake answers every arm in-process.
Fix: three short legs (fresh → returns the loaded snapshot, fake unqueried;
stale plant → the fake is asked for a full pull and the fresh snapshot is
published; absent plant + failing fake → the refusal line, key-free per
`027-L4`), or delete the doc's claim and record the arms as unpinned.

### T4 — minor (new in c1) — `apps/registry/src/state.rs:366-384`; the canonical-external-id gate ships with no test

The new filter makes `steam/+730` and `steam/00730` "not an appid" (`200 []`
on a fresh snapshot) and spells the mint from `app.appid.to_string()` so one
appid keeps one identity. No suite queries a non-canonical spelling (grep for
`+`/zero-padded appid spellings across `tests/`: none); #3's "non-decimal
external id" cell pins only the alphabetic case (`hosted_lookup_policy.rs:309`).
This is exactly the double-identity hazard the code comment says the mint's
held-set probes and `027-B3`'s cap arithmetic stand on, introduced by the
changed code with zero citations. Fix: one leg on the already-built in-process
`policy()` helper — `state.candidates("steam/+9100801", now)` and
`state.candidates("steam/009100801", now)` answer `200 []` with provenance
`none` while the canonical spelling answers the record.

### T5 — minor (new in c1) — `apps/registry/src/steam.rs:340-344, 615-640, 707-713`; four new named refusals in the pull path have no leg

The c1 revision added: (a) full pull of an empty worldlist → refused
("answered with no records at all", :622); (b) a continuation page carrying no
records that then claims finished (:633); (c) `load_snapshot` refusing an empty
`apps` list (:342); (d) any non-`200` status refused, not just non-`2xx`
(:710, "204/206 are answers this client has no reading for"). `tests/steam.rs`
is unchanged: the response-rule leg (:629-678) sends `500/429/302`/bad bodies
but no `2xx`-non-`200` and never an `ok(page_body(&[], false))`; no leg sends
an empty envelope or plants a `"apps": []` file. Each is a production-visible
rule (an ungranted key, a lying pagination, a hand-edited snapshot) that can
now fail a whole pull or a boot, asserted only by its own comment. Fix: three
rows in the existing `every_unhealthy_store_answer_fails_the_pull_by_name`
case list (`204`, empty full-pull body, empty continuation after
`have_more_results` true → also asserting the delta no-change case still
publishes) and one row in the #2 loader-refusal set for the empty `apps` file.

### T6 — minor (carried) — `apps/registry/tests/{cache_document,caching_policy,upstream_client,upstream_endpoint_gate}.rs` (29 `#[ignore]` legs) + `src/{cache,upstream,endpoint}.rs`

Unchanged in c1: the same 29 ignored legs (counted in-tree: 7+11+6+3+2 singles
in `admin_mutations.rs:598`, `candidates_join.rs:331`), the same three retired
modules sitting in `src/` declared by neither `main.rs` nor `lib.rs`, and the
same reason string naming the pending actor ("pending the runner's deletion").
A green `cargo test --workspace` still says nothing about those suites while
they keep compiling as test targets, and §Hosted mode/A15 says *retired*. I
cannot from this pass's allowed inputs see whether `03-build-notes.md` records
the deletion as an outstanding runner action (my round-1 acceptance branch);
if it does, this closes at the next boundary, but the code side should not
outlive the record. Fix: the runner deletes the three modules and four suites
(Rollout 2/3's completion), keeping the two single-leg retires ignored with
reason.

### T7 — nit — `apps/registry/tests/mint_rule.rs:15-23` vs `apps/registry/src/mint.rs:315-331`

The header still asserts the `mint identity over bound` clause is "not pinned
here or anywhere else in this package" and that the rung "needs a mint seam of
its own (`mint.rs::mint_id`) to be tested at all" — but `src/mint.rs`'s unit
`an_appid_spelling_over_bound_is_named_not_truncated` pins exactly that rung
(200-digit external id → `IdentityOverBound`, held-plain + over-bound suffix →
same), and c1 widened the seam anyway (`pub mod mint;`, `lib.rs:24`). The
header now under-reports coverage and misdirects the next reader; the unit
carries no `proves:` marker, so #1's traceability (checklist item 1) for the
last rung rests on an unmarked in-crate test contradicted by the suite header.
Fix: correct the header to point at the unit (and add a
`// proves: design 027's Test strategy #1, last rung` to it), or move the two
assertions into `tests/mint_rule.rs` over the published face.

### T8 — nit (carried) — `apps/registry/tests/steam.rs:69-84`, `tests/issued_entries.rs:36-38`, `tests/retired_ids_and_curation.rs:32`

The c1 revision centralized the store family in `constants.rs` and re-exported
it through `steam.rs` — the import surface my round-1 fix asked for — but the
suites still re-spell the twelve budget numbers as literals. A budget bump now
moves one side only. Import the published constants; keep a literal only where
a suite deliberately pins the design's number, marked as such.

### T9 — nit (carried) — inline `/// proves: Test strategy #N` in the legacy suites (e.g. `response_shape.rs:221`, `upstream_client.rs:59/165/204/252`, `upstream_endpoint_gate.rs:48/79/138`, `admin_auth.rs`, `candidates_join.rs`, `catalog_document.rs:199`, `slug_decoding.rs:39`)

Unchanged: bare markers in design-026 suites share the numbering space with
design-027 suites, so grepping "Test strategy #16" mixes a 026 health pin with
027's platform legs; only file headers disambiguate. Prefix the inline markers
in the legacy suites with their design.

## Checklist walk

1. **Citations per strategy item:** #1 (last rung — T7), #3/#13 stale+absent
   rows (now cited on the policy/health faces — accepted, see note below),
   #4 octet cap (T2), #6 assurance half (T1), #16/#17 (pinned by the re-pinned
   gate_contract legs). New unpinned behaviors: T3/T4/T5.
2. **Unit vs integration layers:** the c1 relocation of the unreachable cold
   rows to the library face is the right layer (the design's startup clause
   makes a stale-boot subprocess impossible without dialing the pinned
   authority; the suites say so on the spot); the integration legs for
   reachable postures stand.
3. **Revision rounds:** r1 major #2 → pinned by new gate_contract legs (met);
   r1 major #1 → not met (T1).
4. **Determinism:** the changed legs use ephemeral loopback listeners, injected
   `now`, and `with_timing` budgets; no wall-clock waits, no outbound dial
   (`startup_snapshot` resolving only at dial time keeps fresh-boot runs off
   the resolver, and the green suite attests it). The 50 ms sleeps in the
   existing assurance legs are unchanged.
5. **Names describe behavior:** the renamed legs
   (`health_stays_a_200_that_reports_the_supply_it_is_refusing_on`,
   `a_named_503_from_the_store_half_fails_the_cli_scan_closed`) read as
   behavior; accepted.
6. **Gate evidence:** all-green at HEAD, cited above; cross and CI-only legs
   unavailable here — recorded, not invented failures.
7. **Untestable additions:** none new beyond T3-T5 (all testable, untested);
   the unused-but-deliberate `rustls-webpki` floor dependency is a declared
   version floor with the RUSTSEC screen as its consumer, not a knob without
   one.

## Notes (no severity)

- Strategy #13's stale half can no longer exist as a *subprocess* leg once the
  startup order is literal (proposal 269-272 vs #13's Integration heading) —
  a design-internal tension the suites document at each relocation; the pinned
  content (policy row + health body) is faithful, so I record the tension, not
  a finding. Whether the build notes flag the #13 wording for the next design
  touch is outside this seat's inputs.
- `data_layout.rs`/`issued_entries.rs` now model the crashed-run lock
  lifecycle explicitly (`clear_crashed_run_write_lock` as the runbook act);
  this matches `store_lock.rs`'s honest rewrite and I found no gap.

**Verdict: request-changes** (T1 open; T2-T6 minor; T7-T9 nit).
