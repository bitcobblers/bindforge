# Code review — testing seat — round 1, pass c0

- **Scope reviewed:** `git diff a22cfd2694fde8d84be132c8d78faf09fda1adf4...HEAD -- ':(exclude)docs/designs'`
  (39 files: `Cargo.lock`, `apps/registry/**`, `docs/adrs/027-…md`). Only the source and
  test paths in that diff are under review here.
- **Design under test:** `docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md`
  (§Test strategy #1–#17), with `docs/adrs/027-hosted-registry-with-store-resolvers.md`.
- **Inputs:** the diff, the proposal, the ADR, the runner's gate record, my own prior
  findings (none — first pass). No other review report or transcript opened.
- **Checklist applied:** `references/testing.md` (code testing checklist) only.
- **Verdict:** `request-changes` — 2 major, 3 minor, 2 nit. No blockers.

## 1. Gate evidence (checklist item 6)

**Supplied by the runner** (gate at 2026-11-22T07:10:57Z, all green, exit 0):
`npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked`.

The one quoted tail line — `test result: ok. 0 passed; 0 failed; 0 ignored; …` — is
**not** a red flag and **not** evidence about this change either way: it is the summary of
the new `bindforge_registry` **library** target (`apps/registry/src/lib.rs:19-21`), which
ships no `#[test]`s. My own run reproduces that exact `running 0 tests` line as its last
summary. So the supplied record shows the gate passed but does not show that the 22 new/
re-pinned registry suites ran.

**Executed by me** to close that gap:

- `cargo test -p bindforge-registry --locked` → **green**. 223 passed, **29 ignored**,
  0 failed, across 25 targets; the heavy legs finished in ≤10.5 s each
  (`an_over_bound_page_fails_the_pull`, `at_the_issued_cap_…`,
  `an_at_cap_retired_list_refuses_a_growing_delete_…` included). This is the evidence that
  Test strategy #1–#15's suites actually run and pass on the code under review.
- `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` → **could not
  execute**: `the 'x86_64-pc-windows-msvc' target may not be installed` in my environment,
  and it is absent from the runner's gate record. **Missing evidence** for the platform
  leg Test strategy #16/#17 name — see finding M2; I do not claim it fails.

## 2. Test-strategy coverage map (checklist items 1 and 2)

| # | Suite(s) that cite it | Status |
|---|---|---|
| 1 Mint table | `tests/mint_rule.rs:132,190,229,265,321,381,407,466` | Covered except the `mint identity over bound` rung — see m1 |
| 2 Snapshot loader | `tests/steam.rs:185-466` (all eight refusal families, metadata gate proven shape-free at `:395`) | Covered |
| 3 Miss-posture table | `tests/hosted_lookup_policy.rs:132-560` | Covered except `503 no steam snapshot` — see m1 |
| 4 Mint write path | `tests/issued_entries.rs:145,220,263,335`, `:105,555` | Covered except the issued **octet** cap rung — see M3 |
| 5 Retired ids | `tests/retired_ids_and_curation.rs:146-587` | Covered, incl. the at-cap rename-buffer refusal and the no-op re-delete |
| 6 Pull codec + transport assurance | `tests/steam.rs:473-1187` | Codec, budgets, normalization, retries, deadline, key scrubbing all covered; **transport assurance is not** — see M1 |
| 7 Issued document | `tests/issued_entries.rs:398,495` | Covered |
| 8 Flags and startup | `tests/hosted_startup.rs:60-398` | Covered (flag retirements, leftover-document pair + `--corpus` exception, key, `<data>`, `write.lock` both halves) |
| 9 Health and stats | `tests/hosted_health_stats.rs:91-335`, `tests/admin_auth.rs:135` re-pin | Covered |
| 10 Admin issued route | `tests/retired_ids_and_curation.rs:308,345,686` | Covered, incl. the `404`-when-disabled and `401`-names-bearer posture |
| 11 Refresh scheduler | `tests/steam.rs:1193-1271` (pure decision, no sleeps) | Covered as designed; the `main.rs` thread body is unobserved — FYI only |
| 12 Hosted end-to-end | `tests/hosted_end_to_end.rs:133-303`, `tests/cli_end_to_end.rs:169` | Covered, incl. the byte-exact A8 wire shape and the second-`serve` refusal |
| 13 Stale-snapshot posture | `tests/hosted_lookup_policy.rs:392,456`, `tests/health_route.rs:109` | Covered |
| 14 Curation flow | `tests/retired_ids_and_curation.rs:612` | Covered |
| 15 Query one-shot | `tests/hosted_end_to_end.rs:356,435,500`, `tests/hosted_startup.rs:335` | Covered |
| 16 Platform | `tests/data_layout.rs:170-309` (env/`<data>` seams) | **Partial — the cargo-leg half is missing: M2** |
| 17 No-outbound rule / cargo job changes | `tests/hosted_end_to_end.rs:305`; harness scrub at `tests/common/mod.rs:703-722` | No-outbound half holds (my green run confirms no leg dialed out; every keyed run plants a fresh snapshot or passes `--no-steam`); **the "cargo job does change" half is missing: M2** |

Both promised layers exist: unit-level over the new library face
(`lib.rs:19-21` → `tests/steam.rs`) and subprocess integration over ephemeral loopback
ports. `tests/common/mod.rs` is sound: `127.0.0.1:0` binds only, ambient
`BINDFORGE_STEAM_WEB_API_KEY`/token/data-home vars scrubbed before every child
(`:715-722`), temp dirs per label, `--no-steam` default so no leg can reach the store.

**Determinism (item 4):** no wall-clock assertions (clocks are parameters:
`load_snapshot(path, now)`, `full(now)`, `next_refresh_seconds(now, last, outcome)`); no
real network, no live store; no fixed ports; retry/deadline gaps injected via
`with_timing` (`tests/steam.rs:146-149`) so no leg sleeps 30 s or 3,600 s. Longest
injected sleep is 1.5 s (`tests/steam.rs:969`), which is load-bearing for the
`fetched_at`-before-first-page pin. **Names (item 5):** behavior-phrased throughout.

## 3. Findings

### M1 — major · Test strategy #6's transport-assurance clause is not pinned; the test claims it is

`apps/registry/tests/steam.rs:1053-1131`
(`the_production_transport_refuses_an_unauthenticated_loopback_listener`), with
`:1136-1187`.

The design names the fixture exactly: "driven against a **loopback TLS listener presenting
a self-signed cert *for the name being verified* and one for another name**; it refuses
both handshakes, so the first case fails on the chain, not the hostname. Without it, a
verifier relaxed to accept the fake passes the whole suite with the key on an
unauthenticated link" (§Test strategy #6, and §Revision log r1→r2 — this is the property
that revision exists to buy).

What shipped is a **plain TCP** listener (`std::net::TcpListener::bind("127.0.0.1:0")` at
`:1068`) that accepts, reads bytes, and answers nothing. No certificate is ever presented,
so no chain is ever built, and the client's failure is an I/O/handshake failure that is
indistinguishable from a verification failure. Consequences, both concrete:

1. The named mutation is invisible. Replace the config at `src/steam.rs:495-499` with one
   that accepts any peer certificate; the handshake still fails (the fake speaks no TLS),
   `store.full()` is still `Err`, still no key bytes leave, and every assertion in the leg
   still passes — while production now ships the Web API key to any terminable link. The
   suite cannot tell the difference, which is the one thing #6 says it must be able to.
2. The "chain, not hostname" half is asserted vacuously: at `:1122-1129` the
   verified-name case asserts only `!text.is_empty()`, i.e. anything at all.

The two legs *do* pin something real — the transport is a TLS client that never writes
`GET`/the key before the handshake (`:1109-1121`), and the SNI/`Host` name is the pinned
authority rather than the connect address (`:1174-1186`, with `STORE_AUTHORITY` pinned at
`:1061`). Keep those. What is missing is the discrimination between "verified" and
"would-accept-anything".

**Fix:** add `rustls` (no default features) as a dev-dependency of `apps/registry` and a
loopback TLS fixture driven by a checked-in self-signed cert/key (or a small generator),
and run the two cases the design names — cert for the verified name, cert for another
name — asserting both are refused **and** that the refusal names a certificate/chain
condition (`UnknownIssuer`-class vs `NotValidForName`-class), both dialed at `127.0.0.1` so
#17's no-outbound rule holds. If a TLS fixture is deliberately out of budget for this run,
then correct the doc-comments at `tests/steam.rs:3-4`, `:53-56` and `:1053-1058` to claim
only what the legs pin (TLS client + name independence), and record #6's assurance clause
as unmet rather than proven — a claimed-but-absent pin is worse than an acknowledged gap.

### M2 — major · Test strategy #16/#17's promised cargo-job change did not land, and the leg is uncovered by any evidence

`.github/workflows/check.yml:29-32` (unchanged by this diff), against
`apps/registry/Cargo.toml:21-26` (new `rustls` with the `ring` provider) and
`Cargo.lock` (`bindforge-registry → rustls, webpki-roots`; `ring 0.17.14`, whose
`build.rs` compiles C/asm for the target).

#16 and #17 both state the gate changes with this branch: "the shipped leg …
**whose scope Rollout 3 narrows** so `ring`'s target build script is never asked of a
runner without a target C toolchain, with `apps/registry` compile-checked on the host
instead" (#16) and "The cargo job **does change**, per Rollout 3: the Windows leg's scope,
and the advisory screen" (#17). Rollout 3 (proposal lines 719-727) names the same
`check.yml` split plus the `README.md` C-toolchain prerequisite line. None of it is in the
diff: the workflow still runs `cargo check --workspace --all-targets
--target x86_64-pc-windows-msvc`, which now includes `apps/registry`, and `README.md`
gains no C-toolchain prerequisite. The design's own §Options/§Compatibility-matrix
reasoning says this leg is exactly where `ring`'s target build script bites.

The runner's gate record contains only host-target legs, and I could not execute the
cross leg here (target not installed), so **I cannot establish that the leg still passes,
and no supplied evidence does either**. That is the required property, and it is why this
is not a nit: the branch changes the crate's dependency graph in the one way the design
said mandates a gate change, and ships without it. Either CI breaks on the next push, or
it passes and the platform posture the matrix records (Windows compile-only via a host
compile of `apps/registry`) is not the posture actually gated.

**Fix:** land the Rollout 3 gate half with the code — narrow the
`x86_64-pc-windows-msvc` leg to the members that stay C-free and add a host-target
`cargo check -p bindforge-registry` step — plus the `README.md` prerequisite line. If the
split is intentionally deferred out of this run, the branch needs a green run of the
*unmodified* whole-workspace cross leg attached to the gate record before this claim can
stand, and #16/#17 marked unmet. (`deny.toml`/`advisories.yml` already give the repo a
scheduled RUSTSEC screen, so the advisory-screen half of Rollout 3 needs at most a note
that the new `rustls`/`rustls-webpki`/`webpki-roots` floors at
`Cargo.toml:21-26` + `Cargo.lock` (`0.23.46` / `0.103.17` / `1.0.12`) are inside it.)

### M3 — minor · The issued document's octet cap has no test on the write side the design puts it on

`apps/registry/src/state.rs:424-428` returns `MintCapReached` when the serialized issued
buffer exceeds `MAX_ISSUED_DOCUMENT_OCTETS`; §Test strategy #4 promises "**the octet cap
refuses the same way**" beside the record-count cap. `tests/issued_entries.rs` pins the
count cap at the real 200,000 (`:335-396`) and the **load-side** metadata gate (`:495-550`),
and its header (`:17-18`) explains that caps are pinned by planting real records because
"there is no budget knob" — but nothing plants the octet side of a *mint*. The rung is
reachable in production (≈150k records with 128-octet ids and names is one mint short of
64 MiB and still under `027-B3`'s count cap), and it is the check that keeps
`persist-before-serve` from renaming an over-budget document.

**Fix:** one leg beside `at_the_issued_cap_…`: plant a document under the record cap whose
serialization sits within one record of `MAX_ISSUED_DOCUMENT_OCTETS` (fat 128-octet ids and
names), assert the novel key is the named `503` and that the file, model, and `revision`
are untouched — the same three assertions the count-cap leg already makes. Stream the
fixture as `an_over_bound_issued_document_is_refused_on_its_size_alone` already does
(`:516-521`).

### m1 — minor · Two named refusals are unreachable through every tested face and have no seam pinning them

- `mint identity over bound` — recorded by the suite itself at
  `apps/registry/tests/mint_rule.rs:15-23`: no served query can put the mint on that rung,
  because `keys.rs` caps the joined key at 128 and `steam-<external id>` is spelled from
  it.
- `503 no steam snapshot` — recorded at `apps/registry/tests/hosted_lookup_policy.rs:16-20`
  and asserted nowhere: `state.rs:349-351` returns it only when the snapshot slot is empty,
  and startup is load-or-pull (`§The steam snapshot resolver`), so a live instance either
  has a snapshot or never bound. `tests/steam.rs:325`'s loader message is a different
  condition.

Test strategy #1 and #3 both list these as pins, so under checklist item 7 they read as
untestable additions (a `503` and a mint rung with no reachable caller and no consumer),
not as passes. `apps/registry/src/lib.rs:19-21` exposes `constants`/`persist`/`steam` — but
not `mint`, which is why the mint rung needs a seam the crate declined to publish while it
published the pull's.

**Fix:** mirror the `steam.rs` precedent: add `pub mod mint;` to the library face and pin
`mint_id`'s over-bound rung directly (`mint_id(&"k".repeat(128), &"9".repeat(123), …)` →
`MintIdentityOverBound`), and pin `no steam snapshot` as a unit over
`State`'s posture decision (or over `steam_lookup` with an empty slot). If the judgment is
that neither is reachable by construction, that judgment belongs in §The hosted lookup
policy and §Issued entries as a reachability note the next reviewer can check, not only in
two test-file headers.

### m2 — minor · A15's retirement is a mass `#[ignore]`, and the retired modules stay in the tree

`grep` count: **29 ignored legs** — `tests/cache_document.rs` (7),
`tests/caching_policy.rs` (11), `tests/upstream_client.rs` (6),
`tests/upstream_endpoint_gate.rs` (3), one each in `tests/admin_mutations.rs:598` and
`tests/candidates_join.rs:331`. The four heavy targets ran `0 passed; N ignored` in
0.00 s in my run. Each carries the reason "suite retired in place pending the runner's
deletion", and `tests/candidates_join.rs:312-330` explains the stage has no delete right —
a real constraint, but the outcome is that the branch as reviewed ships four test targets
whose every leg is dead, plus four source files that nothing compiles any more:
`apps/registry/src/cache.rs`, `upstream.rs`, `endpoint.rs` are no longer declared in
`apps/registry/src/main.rs:19-30`. §Hosted mode's "Retired code: `cache.rs`, `upstream.rs`,
`endpoint.rs` … Retired suites per A15" means gone, not disabled; a green
`cargo test --workspace` now says nothing about ~1,900 lines that are still in the tree and
still compiled as test targets.

**Fix:** have the runner delete the four modules and four suites as the recorded completion
of this stage (Rollout 2/3 line items), and keep the two single-leg retires
(`admin_mutations.rs:598`, `candidates_join.rs:331`) as ignored-with-reason if a whole-file
deletion is what needs the runner's hand. If deletion truly cannot happen in this run,
state that in `03-build-notes.md` as an outstanding runner action, so "retired" in the
ledger does not quietly mean "ignored".

### n1 — nit · Budget constants are re-spelled in the suites instead of read from the crate

`tests/steam.rs:69-84`, `tests/issued_entries.rs:36-38`,
`tests/retired_ids_and_curation.rs:32` re-declare `MAX_STEAM_SNAPSHOT_OCTETS`,
`MAX_STEAM_PAGE_OCTETS`, `STEAM_REFRESH_MAX_PAGES`, `STEAM_PAGE_ATTEMPTS`,
`STEAM_PAGE_MAX_RESULTS`, `MAX_NATIVE_NAME_OCTETS`, `REFRESH_SECONDS`, `MAX_AGE_SECONDS`,
`RETRY_SECONDS`, `MAX_ISSUED_ENTRIES`, `MAX_ISSUED_DOCUMENT_OCTETS`,
`MAX_RETIRED_IDS_OCTETS` as literals — while `src/steam.rs:71-117` publishes most of them
and `src/constants.rs:98` is already on the library face (`lib.rs:19`). A budget change
then moves one side only, and the failure is a confusing fixture assertion rather than a
type error. Import them where the crate publishes them; keep local literals only where a
suite genuinely wants to pin the design's number independently of the code (say so on
those).

### n2 — nit · `proves: Test strategy #N` markers now mean two different designs

Design 026's numbering and design 027's share the same bare marker.
`tests/response_shape.rs:221` ("#16's shape clause"), `tests/admin_auth.rs:135` and
`:438` ("#9"), `tests/candidates_join.rs:48`, `:79`, `:142`, `tests/catalog_document.rs:199`
and `tests/slug_decoding.rs:39` ("#1"–"#3") cite **026**; the file headers do say
"design 026's" / "design 027's" (`tests/candidates_join.rs:6`, `tests/mint_rule.rs:5`), but
the inline markers — the ones checklist item 1 is read off — do not, so a grep for
"Test strategy #9" returns design-027 health pins and design-026 admin-auth pins
indistinguishably. Prefix the inline markers with the design they cite in the re-pinned
legacy suites.

## 4. Checklist walk

1. **Numbered Test-strategy items cited** — all 17 have citing tests; clause-level gaps at
   M1 (#6), M2 (#16/#17), M3 (#4), m1 (#1, #3).
2. **Unit + integration layers** — both present (library-face suite over `steam.rs`;
   subprocess suites over ephemeral loopback).
3. **Revision-round regression tests** — not applicable: first pass, no prior
   blocker/major.
4. **Determinism** — pass: injected clocks and timing, `127.0.0.1:0` everywhere, no
   outbound name, no ambient-env inheritance.
5. **Behavior-named tests** — pass.
6. **Gate evidence** — supplied record green for all four legs; my executed
   `cargo test -p bindforge-registry --locked` is green (223 passed / 29 ignored) and
   supplies the coverage the record's single `0 passed` lib-target line cannot. The
   whole-workspace Windows cross-compile leg is **missing evidence** (M2), not a failure I
   observed.
7. **Untestable additions** — findings, not passes: m1 (two unreachable named refusals),
   m2 (29 ignored legs + 4 orphan modules).

**FYI (no severity, other seats' ground):** Rollout 1's ledger annotations
(`026-C1`/`C2` `amended-by`, `026-C6`/`C7`/`B1`–`B3` retirement lines, `024-C2`
cross-reference) and Rollout 6's `docs/registry-hosting.md` runbook are absent from this
diff; the only non-`apps/registry` change is a 2-line edit in `docs/adrs/027-…md`.

## 5. Verdict

`request-changes`. The suite architecture is strong — decision-table pins, real-cap
fixtures rather than knobs, an honest seam contract in `tests/steam.rs`, and a harness that
scrubs the Web API key before any child starts. Two things must move before approval:
Test strategy #6's transport-assurance clause needs the TLS fixture the design specified
(M1), and the gate/platform change that #16/#17 say travels with this dependency addition
needs to land or its evidence supplied (M2).
