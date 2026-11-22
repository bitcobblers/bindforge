# Performance review — ADR-027 hosted registry, round 2 ratification (c1)

Seat: performance (specialty checklist `references/performance.md`).
Pass: **ratification c1** — later revisions landed after the round-2 approval;
this pass checks only that they did not break what this seat approved.
Scope (caller's): `git diff f52686fecdc794062dc72f6d71c58262d2029fb9..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and
`docs/adrs/027-hosted-registry-with-store-resolvers.md`.

Inputs used: the scoped diff, `02-proposal.md` (unchanged across this range —
`git diff f52686fe..HEAD -- 02-proposal.md` is empty), the ADR, the runner's
gate record, my own round-2 findings, and my own read-only `cargo test` runs.
No other review report or transcript was opened.

## What the increment actually is

    apps/registry/src/steam.rs                         |  20 ++-
    apps/registry/tests/admin_auth.rs                  |  44 +++++-
    apps/registry/tests/fixtures/store/README.md       |  24 ++++
    apps/registry/tests/fixtures/store/*.der           |  Bin (4 files, ~1.1 KiB)
    apps/registry/tests/retired_ids_and_curation.rs    |  95 ++++++++++++-
    apps/registry/tests/steam.rs                       | 148 +++++++++++++------

One production line-group (`io_reason`, `steam.rs:835-847`, called at
`:700`, `:913`, `:1040`) and the rest test-only: the Test-strategy #6 pair is
re-driven against loopback rustls servers on checked-in self-signed fixtures,
`admin_auth.rs` gains the issued route in the existing auth matrices, and
`retired_ids_and_curation.rs` gains one segment-rule leg. No change touches
`state.rs`, `corpus.rs`, `persist.rs`, `mint.rs`, the load path, the serving
path, or the concurrency/lock shape this seat approved.

## Checklist walk

1. **Event/request path — no sync fs or network added.** Verified the only
   production change is on an already-failing error path: `io_reason` formats a
   refusal string where the code previously formatted `error.kind()`. Cost is
   one `format!` on a leg that is already returning a transient/permanent
   refusal; the happy path (`attempt_page`, `read_header_block`, `fill`) is
   byte-identical apart from the reason argument. No new allocation, lock, fs
   or network act on any request-serving path. Neutral.
2. **Complexity.** No new loops over unbounded input. The new test leg iterates
   10 fixed segments × 2 routes; the fixture server accepts connections per
   dial, and the client's dial count is bounded by `STEAM_PAGE_ATTEMPTS` (3)
   for the pull's first page, since the page fails all attempts and the pull
   aborts.
3. **Unbounded growth.** Checked the new fixture machinery: `serve_tls_fixture`
   (`tests/steam.rs:1165-1188`) appends only bytes the *client* delivered, which
   is one bounded `GET` per dial; every fixture read carries a 5 s
   `set_read_timeout` (`tests/steam.rs:1119`), so no fixture thread can block
   forever and none busy-spins (`Ok(0) | Err(_) => break`). The per-connection
   threads are bounded by the client's dial budget. The outer accept-loop thread
   owning its listener is the same shape as the pre-change leg, so no new leak.
   Fixtures are checked-in DER (≈1.1 KiB), not generated at build time, so the
   gate pays no keygen.
4. **Startup.** Nothing in the increment runs at boot; the boot/resolve
   separation this seat approved is untouched — resolution is still asked only
   in `connect` (`steam.rs:742-750`, `dial_addresses` at `:793-814`).
5. **Cache invalidation.** Not applicable to this increment (no cache
   introduced or touched).
6. **Benchmark-like legs: run them and quote numbers.** Run this pass, same
   profile as the runner's green gate:
   - new transport pair, alone: `test result: ok. 1 passed; 0 failed; 0 ignored;
     0 measured; 26 filtered out; finished in 0.40s` — two real ECDSA P-256
     rustls handshakes plus two 150 ms settle sleeps (`tests/steam.rs:1157`), so
     the leg's floor is the sleeps, not the cryptography. It replaces a leg that
     cost one plaintext read + 50 ms; net suite cost is a few hundred ms.
   - full `tests/steam.rs`: `test result: ok. 27 passed; … finished in 5.45s`.
   - full `tests/retired_ids_and_curation.rs` (with the new segment leg):
     `test result: ok. 12 passed; … finished in 3.38s`.
   No suite-runtime regression of any consequence.

## Ratification of what this seat approved

- **Round-2 finding 1 (Rollout 4 / §Budgets rewrite figure).** Not addressed and
  not broken: this diff adds no timing leg, and `corpus.rs:355-378`
  (`to_bytes_with`), `state.rs:461`/`:469` (`to_bytes_with` + `persist::save`
  under `mutations`) are untouched, so the mint cost I approved is exactly the
  cost I approved. §Budgets:535-537 and Rollout 4:730-731 still point at a
  measurement nothing in the reviewed paths yields; the suite's only 200k-record
  leg (`tests/issued_entries.rs:355-411`) is still refused at the cap check
  (`state.rs:439`) before it serializes. Carried as missing evidence, unchanged.
- **Round-2 finding 2 (per-attempt resolver leg outside the deadline).**
  Unchanged: `connect` still bounds the dial with
  `timeout.min(STEAM_CONNECT_TIMEOUT)` at `steam.rs:751` and checks the deadline
  at `:743-746`, then resolves at `:750` → `:797-805` with no timeout, once per
  page attempt. The diff did not move it; still a boundary note, not a defect.
- **Round-2 finding 3 (no dribbling-peer deadline leg).** Unchanged and still
  green: both mid-read guards are intact (`within_pull_deadline` at
  `steam.rs:898` in `read_header_block` and `:1029` in `fill`, defined `:1006`),
  and `tests/steam.rs:942` (`a_stalled_pull_is_cut_off_by_its_own_deadline`,
  measured inside the 5.45 s suite above) remains the only deadline leg, still
  the total-silence peer. The new TLS pair exercises the refusal-rendering seam,
  not the deadline.
- **New production seam.** `io_reason` (`steam.rs:835-847`) widens a refusal
  string to rustls' `Error` Display only for `ErrorKind::InvalidData`, which the
  doc comment argues only rustls yields through `StreamOwned`. Performance-wise
  it is one small `format!` on a refusal path; the refusals are emitted once per
  failed page attempt, not per byte or per record. The claim that the Display is
  bounded and carries no key/URI is the security seat's to hold; nothing about it
  costs the serving face or the pull loop anything measurable.

Nothing in the increment makes a previously-false performance claim true, and
nothing regresses the boot, mint, cap, or pull-budget behaviour approved in
round 2.

## Findings

Three findings, all carried from round 2 with locations re-verified against the
current tree; **none is caused or worsened by this increment**, and none is a
blocker or major.

1. **minor** — `apps/registry/src/corpus.rs:355-378`,
   `apps/registry/src/state.rs:461,469`, against `02-proposal.md` Rollout 4
   (:730-731) and §Budgets (:535-537). The measured issued-document rewrite at
   100k and 200k records that §Budgets promises "beside that figure", and that
   §Deferred's batched-write deferral is supposed to defer against, is still
   produced by nothing in the reviewed paths; §Budgets still states it
   prospectively. Missing evidence, not a defect. *Fix:* one `#[ignore]`d timing
   leg booting against a 100k-record and a 200k-record issued document, asking a
   single novel key, printing the `to_bytes_with` and `persist::save` elapsed
   times, with the two figures carried into §Budgets.
2. **nit** — `apps/registry/src/steam.rs:750` and `:793-814`. `to_socket_addrs`
   is asked once per page attempt (up to `STEAM_REFRESH_MAX_PAGES ×
   STEAM_PAGE_ATTEMPTS` = 36 per pull) and is the one blocking network act on
   the pull path with no bound: `:743-746` checks the deadline, then resolves
   without a timeout, so a black-holed resolver can spend the resolver's own
   retry grid 36 times against the named 1,920 s
   `STEAM_PULL_DEADLINE_SECONDS` guarantee. Noise with a cached resolver. *Fix:*
   resolve once per pull and reuse across the page loop's attempts, or say in
   `dial_addresses` that the resolver's own timeouts bound this leg.
3. **nit** — `apps/registry/src/steam.rs:898` and `:1029`;
   `apps/registry/tests/steam.rs:942` is the only deadline leg. No leg dials a
   fake that dribbles a header block or a body under the 30 s socket read
   timeout, which is exactly the peer the mid-read deadline guard exists for, so
   dropping either call site later would stay green. *Fix:* a loopback leg with
   a short pull deadline whose fake writes a few bytes every 50 ms after the
   headers, asserting the refusal names `steam pull deadline` and the elapsed
   time is the deadline rather than a socket-timeout grid.

## FYI (out of specialty, no severity)

`tests/fixtures/store/README.md` documents the fixture certificates' `notAfter`
as November 2046 (`-days 7300`) as a "scheduled test-fixture refresh, not an
incident" — a dated maintenance clock on a green gate, flagged for whichever
seat cares about clocks.

## Verdict

**approve.** Zero open blockers and zero open majors. The increment is one
error-rendering helper plus test and fixture work; it does not touch the boot,
mint/rewrite, cap, pull-budget, or lock-scope behaviour this seat approved in
round 2, and the measured suite costs above show no regression.
