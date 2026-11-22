# Code review — correctness seat, round 1 dissent pass (c1)

- **Change under review:** `git diff 75cbc433ed8a546734c41d4eb928395373ab73e6..HEAD -- ':(exclude)docs/designs'`
  (21 files: `.github/workflows/check.yml`, `Cargo.lock`, `README.md`, `apps/registry/{Cargo.toml,src/*,tests/*}`,
  `crates/core/tests/gate_contract.rs`).
- **Design:** `docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md`; `docs/adrs/027-hosted-registry-with-store-resolvers.md`.
- **Inputs used:** the diff, the proposal, ADR-027, the runner's gate record (green at 2026-11-22T09:51:31Z), my round-1 findings.
  `03-build-notes.md` and every other `reviews/` artifact were outside the permitted inputs, so every
  "declare it" finding below is a confirmation request, not an assertion that the entry is missing.
- **Pass type:** dissent. Scope discipline honored: the findings below lead with whether the r1 fixes landed,
  and add only what the changed code makes true.
- **Verdict:** approve (no open blocker or major; four minors, five nits).

## Checklist walk

1. **Named components exist and match.** `StoreTransport` (renamed from `Transport`, with `Cleartext`/`Verified`)
   and `SteamStore::production` now exist as §The store pull names them (steam.rs:157-203, 476-500) — r1 nit #15 closed.
   §Budgets' family is declared in `constants.rs` (constants.rs:106-165) and re-exported by `steam.rs:59-68` — r1 #12 closed.
   One structural addition the proposal does not name: `lib.rs` now publishes `corpus`, `keys`, `mint`, `retired`,
   `state` alongside `steam`/`constants`/`persist` (see F2).
2. **Assumption register.** Nothing in the diff silently reverses an ADR clause any more. The two reversals I
   reported at r1 are gone: `027-C3`'s load-or-pull is literal (F1 verifies the reading), and §Concurrency's
   "finding it present is the named startup refusal" is now the whole rule — `refuse_or_adopt`, `process_is_live`
   and the `/proc` heuristic are deleted (store_lock.rs:78-80, 116-125), the module doc names the earlier
   mechanism as not-the-design, and the pid is diagnostic only. r1 blocker #1 and majors #3/#4/#5/#6/#7 all closed
   (evidence below).
3. **Deviations declared.** Cannot be verified from my inputs; five additions and one cross-design re-pin need
   §Deviations lines (F2).
4. **Scope creep.** Everything added traces to an r1 finding or to a more literal reading of a proposal sentence
   (`status != 200` for §The store pull's "non-`200`"; the deadline paid at every read for the named
   `steam pull deadline`; `same_document` for #8's "except at the path `--corpus` names"). The mint's
   `to_bytes_with`/`insert` fast path is §Concurrency's own mint shape (proposal 458-461), and the unit test
   corpus.rs:548-594 pins that the incremental spelling's bytes equal the whole-document spelling.
5. **Build-notes claims.** Not readable here; F2 asks that the listed behaviors be declared.
6. **Error paths fail closed.** New named refusals fail closed and refuse to publish: an empty full pull
   (steam.rs:620-625), a continuation page that carries nothing then claims to be finished (626-637), an empty
   `apps` list at load (336-343), a non-canonical appid spelling answered as the design's `200 []`
   (state.rs:376-383), a refused publish scored `PullOutcome::Failed` on the refresh thread (main.rs:480-498),
   and `retired id list full` now rendered as a named `503` on the admin face (admin.rs:231-241, state.rs:702-710).
7. **r1 blocker/majors actually addressed.** Yes, all seven; the residuals are evidence, not behavior (F1).

## r1 findings — disposition

| r1 | Was | Now |
| --- | --- | --- |
| blocker, startup load-or-pull | accepted any loadable snapshot | `steam::startup_snapshot` (steam.rs:437-471) loads only `Ok(snapshot) if !is_past_age_bound(now)`; past-age, absent and unloadable all take the full pull, publish, and the named refusal only on a failed pull. `main.rs:357-365` delegates with no added logic. The stale-posture comment that claimed a §Deviations entry is gone. r1 #1's `--no-steam` alternative is the documented operator exit (main.rs:353-356), matching proposal 272-274. The refresh anchor moved off "startup counted as the last pull" onto the snapshot's own `fetched_at` (main.rs:456-461). **Unpinned — F1.** |
| major, empty worldlist publishes | yes | Refused at the pull (steam.rs:620-637) and refused at load (336-343); the doc claim at steam.rs:140-146 now matches the code. **Unpinned — F1.** |
| major, non-canonical appid mints a second identity | yes | `parse::<u64>().ok().filter(|appid| appid.to_string() == external_id)` (state.rs:379-382) answers a `+7`/`007` spelling as the design's `200 []`, and the mint is spelled from `app.appid.to_string()` (state.rs:400). |
| major, stale-lock adoption | yes | Deleted; present ⇒ refuse (store_lock.rs:78-80, 116-125), pid is diagnostic, `Drop` and the runbook carry cleanup, and `hosted_startup.rs:329-403` pins both halves plus the operator's deletion act. |
| major, Rollout 3 CI legs | absent | `check.yml:37-53` splits the msvc leg (`--exclude bindforge-registry`; no workspace crate depends on it, so the exclusion really drops `ring`) and adds the host compile check; `check.yml:83-126` adds the advisories-only RUSTSEC screen at the repo root, where `deny.toml` and `Cargo.lock` are. `gate_contract.rs:192-286` re-pinned for both. |
| major, dependency floors | undeclared | `rustls ">=0.23.46, <0.24"`, `webpki-roots ">=1.0.10, <2"`, and a declared `rustls-webpki ">=0.103.16, <0.104"` floor pin (apps/registry/Cargo.toml:21-41), which are proposal 93-101's floors; the ceiling is tighter than the design's 0.104-alpha alternative, which is within the floor. |
| major, DNS before the load/pull decision | yes | `Config::store` builds `SteamStore::production(key)` — no resolver call (main.rs:339-347); resolution moved to `dial_addresses` at dial time (steam.rs:790-813) and every address the name yields is walked (steam.rs:735-765). No keyed binary leg reaches the pull path any more (every hosted suite leg is `--no-steam` or plants a `now_unix()` snapshot). |
| minor, Rollout 1 ledger annotations | absent | Still absent — **F4.** |
| minor, Rollout 6 runbook | absent | Still absent, and now load-bearing — **F3.** |
| minor, `retired id list full` rendered as `503 Bad Request` | yes | `AdminRefusal::Named` + `Answer::named` carry reason phrase and body token (admin.rs:231-241, 329; state.rs:227-237, 702-710). Test still accepts line-or-body — **N1.** |
| minor, `owner_only_dir` drift | — | Split into the shipped create-mode plus a named `narrow_to_owner_only` ceiling (persist.rs:178-211); the never-widen intent is now explicit and A2's `0700` is named as the create mode. Closed. |
| minor, §Budgets family location | yes | Moved to `constants.rs`, stale justification comment deleted, `steam.rs` re-exports. Closed. |
| nit, `normalize_store_name` re-implementation | yes | Calls `discovery::game::friendly_name` after the `char::is_control()` strip, exactly proposal 309-312; `clamp_to_name_bound` deleted. Closed. |
| nit, missing `name` posture | — | Documented as the empty-name posture (steam.rs:1051-1060), the same shape the design fixes for a title that normalizes to nothing. Accepted as-is. |
| nit, `Transport`/`Loopback` naming | yes | `StoreTransport` + `Cleartext`, plus `SteamStore::production` as the one production construction path. Closed. |

## Findings

### F1 (minor) — the corrected startup clause and the three new refusals have no leg
`apps/registry/src/steam.rs:437-471`, `620-637`, `336-343`; `apps/registry/src/state.rs:379-383`.
`startup_snapshot` has exactly one call site outside its own definition (`main.rs:362`; grep over `--include=*.rs`
returns main.rs:349/362, steam.rs:164/437 and nothing under `apps/registry/tests/`), so all three arms of `027-C3`'s
load-or-pull — the direction my r1 blocker was about — are unpinned: deleting the `!snapshot.is_past_age_bound(now)`
guard, or reverting the arm order, keeps the whole suite green. Same for the two new named refusals
(`the store's worldlist answered with no records at all`, `the steam snapshot's `apps` is empty`) and the canonical
spelling gate: `grep` over `apps/registry/tests` finds no `steam/007`, no `{"apps": []}`, and no empty-pull fixture
(`tests/steam.rs:730-806` covers the page budget and the lost-later-page partial case only). The design's Test
strategy #8 names no stale-boot leg and #13's planted-stale half moved in-process, so no listed leg asks this
question either. The property itself is establishable by inspection — `startup_snapshot` is 35 lines with no
other reader of the guard — which is why this is evidence-thinness, not a blocker.
**Fix:** three `tests/steam.rs` legs driving `startup_snapshot` against the existing loopback fake —
fresh file ⇒ returned as loaded and `fake.request_count() == 0`; past-bound file ⇒ full pull, file rewritten at
the new stamp, returned; unloadable file ⇒ full pull — plus one leg per new refusal (a `{"apps": [],
"have_more_results": false}` answer, a continuation page that carries no records, and a planted file with an
empty `apps` list refusing at load), and `steam/007` / `steam/+7` rows beside the `200 []` cells of
`tests/hosted_lookup_policy.rs`.

### F2 (minor) — six behaviors this revision added are not named in the proposal; one re-pins another design's contract
`apps/registry/src/steam.rs:620-637`, `336-343`; `apps/registry/src/main.rs:456-461`, `367-401`;
`apps/registry/src/state.rs:400`; `apps/registry/src/lib.rs:25-35`; `crates/core/tests/gate_contract.rs:192-201`.
Each of these is defensible and, per my own r1 fix text, wanted — but none is in the proposal's enumerated
behavior, so each needs a §Deviations entry (I may not read `03-build-notes.md`, so this asks that they exist):
the empty-worldlist pull refusal and the empty-`apps` load refusal (neither is in §The store pull's named
refusal list, proposal 300-305); the refresh thread's first sleep being anchored on the snapshot's `fetched_at`
rather than the proposal's "sleep `STEAM_SNAPSHOT_REFRESH_SECONDS`" (proposal 342-347) — a necessary consequence
of the corrected startup clause, since a legal 6.9-day-old snapshot would otherwise sit past `027-B2` for a day;
`steam-<appid>` spelled from the record's `appid` where mint step 5 says "spelled from the queried key's external
id" (proposal 399) — behaviorally identical behind the new canonical gate, but a wording departure;
`same_document`'s dev/inode-plus-case-insensitive identity for the `--corpus` exception, which also spares hard
links; and the lib face taking `corpus`, `keys`, `mint`, `retired` and `state`. Separately,
`gate_contract.rs:193-201` re-pins design 008's Test-strategy-#6 "two independent jobs per PR" to three jobs, so
the screen could be checked; the proposal (100-101, 697-698) licenses "a RUSTSEC screen beside them" but the
cross-design re-pin is the part worth recording.
**Fix:** one §Deviations line each, and one line naming the 008 gate-contract re-pin.

### F3 (minor) — Rollout 6's runbook still does not exist, and this revision made it load-bearing
`docs/` has no `registry-hosting.md` (`ls docs/` → `adrs designs prompts tasks workflows.md`). Proposal 736-756
names its content; none of it has an artifact. The lock posture I asked for at r1 makes the gap operational:
`store_lock.rs:78-80` refuses any present lock, so every unclean stop — including a SIGTERM, which runs no
`Drop` — needs the runbook's "delete a crashed run's leftover `write.lock`" act before the instance can start
again, and three suites now perform that act as Rollout 6's (`tests/common/mod.rs:432-455`,
`tests/hosted_startup.rs:376-403`, `tests/data_layout.rs:193-198`, `tests/issued_entries.rs:136-145`). The
proposal's other runbook-only controls for this branch's behavior — the restart policy that bounds daily startup
pulls (the startup clause now pulls on every boot over an absent/past-age snapshot), forcing a full refresh by
deleting the snapshot, and the retired-list-size read — are equally undocumented.
**Fix:** land `docs/registry-hosting.md` with the enumerated content per Rollout 6.

### F4 (minor) — Rollout 1's ledger annotations still absent (r1 #8, unchanged)
The diff touches no file under `docs/adrs/` except ADR-027 itself; `grep 027` over `docs/adrs/026-registry-service.md`
returns only the pre-existing `026-R2` reservation lines, and `docs/adrs/024-game-slug-identity.md` returns none.
Proposal 702-710 requires `amended-by` lines on `026-C1`/`026-C2`, retirement lines on `026-C6`, `026-C7`,
`026-B1`–`026-B3`, `026-L4`'s `amended-by 027-L3`, `026-L5`'s caching half, `026-R2`'s intake half, and the
`024-C2` cross-reference — while this branch's earlier rounds deleted `cache.rs`, `upstream.rs`, `endpoint.rs`
and the `026-B1`–`B3` constants.
**Fix:** land the annotations per `021-L4`.

### N1 (nit) — `tests/retired_ids_and_curation.rs:496-507` still accepts the name in "line or body"
The rendering fix makes `HTTP/1.1 503 retired id list full` the answer, but the assertion would still pass a
`503 Bad Request` with the name only in prose — the exact looseness that let r1 #10 through.
**Fix:** assert the status line carries `retired id list full`.

### N2 (nit) — nothing structurally stops a future binary leg from dialing the real authority
Test strategy #17's no-outbound rule now holds by convention only: every keyed subprocess leg plants a
`now_unix()` snapshot or starts `--no-steam` (verified across `apps/registry/tests/*`), and the lazy
`dial_addresses` resolution (steam.rs:790-813) means a fresh planted snapshot never consults the resolver. But a
leg that plants no snapshot, or one older than `STEAM_SNAPSHOT_REFRESH_SECONDS`, would now reach
`partner.steam-api.com` for real — and with the new `fetched_at` anchor (main.rs:456-461) such a snapshot makes
the refresh thread dial within seconds of bind rather than a day later.
**Fix:** an injectable resolver seam on `Endpoint::Pinned`, or a suite-level guard that fails the run when the
production store is dialed.

### N3 (nit) — the new advisory screen's greenness is unevidenced
The runner's gate record contains `npm run check`, `cargo fmt`, `cargo clippy` and `cargo test`; it has no
`cargo deny check advisories` leg and no msvc leg, and cargo-deny is not installed here, so I cannot establish
that today's `Cargo.lock` clears the screen the way `check.yml:112-115` promises it will. A lock carrying an
unwaived advisory would turn the new job red on landing while the scheduled scan (`advisories.yml:79-83`,
`|| true`) stays quiet.
**Fix:** record a `cargo deny check advisories` run (and a `cargo check --workspace --exclude
bindforge-registry --all-targets --target x86_64-pc-windows-msvc` run) against this lock in the rollout checks.

### N4 (nit) — `AdminRefusal::Named`'s doc overstates its routing
state.rs:227-237 says the variant carries capacity refusals "or a mutation would push a document past its cap",
but all three document-cap refusals are still `AdminRefusal::Reject { status: 422 }` (state.rs:567, 622, 671);
`RetiredIdListFull` is its only producer (state.rs:709).
**Fix:** drop the cap clause or route those refusals by name too.

### N5 (nit) — the relocated stale-supply pins leave no on-the-wire proof of the in-service case
`tests/health_route.rs:150-234` and `tests/hosted_lookup_policy.rs:486-520` now read `state.rs`'s faces
in-process, and the socket legs were re-spelled onto `--no-steam` (`health_route.rs:85-125`,
`cli_end_to_end.rs:300-375`). That is the honest reading of an instance that ages a snapshot only through a
refresh thread with no loopback seam, and §Compatibility matrix's bound-CLI claim is still pinned on a real
named `503` (`cli_end_to_end.rs:332-375`, now also asserting the reason survives the CLI hop and the match store
stays unwritten). Recording it because the coverage moved rather than grew: no socket leg answers
`503 steam snapshot stale` any more.
**Fix:** with F1's seam, one socket leg over an instance whose snapshot was published stale at boot is
unnecessary — a `startup_snapshot` leg plus these in-process pins is the honest pair; say so in the build notes.

## FYI (out of specialty, no severity)

The `to_bytes_with`/`Catalog::insert` mint fast path is performance-seat territory; from here it reads as
§Concurrency's own mint shape and `corpus.rs:548-594` pins byte-equality with the whole-document spelling,
including the replace-in-place case.
