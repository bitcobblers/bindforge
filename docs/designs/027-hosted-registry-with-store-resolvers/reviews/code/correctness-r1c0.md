# Code review — correctness seat, round 1 pass c0

- **Design**: `docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md`
  (with `docs/adrs/027-hosted-registry-with-store-resolvers.md`)
- **Scope**: `git diff a22cfd2694fde8d84be132c8d78faf09fda1adf4...HEAD -- ':(exclude)docs/designs'`
  (39 paths, +8991/−1435)
- **Inputs used**: that diff, `02-proposal.md`, ADR-027, the runner's gate record
  (2026-11-22T07:10:57Z, green), the source files the diff touches, read to establish
  findings. No review report or transcript was opened. `03-build-notes.md` was **not**
  read: the caller's input list does not include it (see *Evidence limitations*).
- **Checklist**: `.opencode/skills/code-review/references/correctness.md`, all seven items.
- **Verdict**: **request-changes** — 1 blocker, 6 major, 5 minor, 3 nit.

## Checklist walk

1. **Named components exist and are wired** — *partly*. `Mode::Hosted`, the hosted
   policy (`state.rs::candidates`), `mint.rs::mint_id`, `retired.rs`, `steam.rs`
   (document, loader, pull, transport, `next_refresh_seconds`), `store_lock.rs`, the
   issued-delete route, the four-field health and four-count `stats` all exist, match
   the proposal's shapes, and are wired into `main.rs`/`serve.rs`. Missing: Rollout 3's
   `check.yml` Windows-leg split, Rollout 1's ledger annotations, Rollout 6's runbook
   (findings 5, 8, 9).
2. **Assumption register** — honored on A1 (health list), A2 (fixed-tmp `0600`,
   metadata-gated loads), A3/A17 (request parameters, envelope refusals), A6 (100k/day
   arithmetic), A8 (unchanged candidate caps), A10 (`MAX_SLUG_OCTETS`), A12 (plausibility
   pair reused), A15. Two drifts: the `027-C3` startup load-or-pull clause (finding 1)
   and `owner_only_dir`'s parent-mode discipline (finding 11).
3. **Deviations declared in `03-build-notes.md`** — *not assessable* (out of my inputs).
   Two code sites assert a declaration there in their own comments
   (`main.rs:357-367`, `steam.rs:58-67`); I state the deviation, not the declaration.
4. **Scope creep** — `store_lock.rs`'s stale-lock adoption (finding 4),
   `owner_only_dir`'s mode rule (finding 11).
5. **Build-notes claims** — not assessable (not read).
6. **Error paths exist and fail closed** — the named `503` family, the cap refusals,
   the `retired id list full` fail-closed path, and the persist-before-serve failure all
   exist and mutate nothing (verified in `state.rs:382-449`, `649-675`, and
   `tests/retired_ids_and_curation.rs:427-522`). Two holes: the empty-worldlist publish
   (finding 2) and the wire naming of one refusal (finding 10).
7. **Previous round's findings** — none: this seat's last verdict held no findings.

## Gate evidence

The runner's record is green on all four legs (`npm run check`, `cargo fmt`,
`cargo clippy -D warnings`, `cargo test --workspace --locked`). Nothing on those legs
covers the Windows cross-compile leg, live DNS, or the empty-worldlist path, so green
does not reach findings 1, 2, 5 or 7.

## Findings

### Blocker

**1. A stale snapshot never triggers the startup pull, reversing `027-C3`'s load-or-pull clause.**
`apps/registry/src/main.rs:368-401` (`load_or_pull_snapshot`): `load_snapshot` is
accepted on any *loadable* file, and the full pull runs only when nothing loadable is on
disk. ADR-027 `027-C3` (adr lines 105-108) spells the clause verbatim: "the snapshot is
loaded if present **and within `027-B2`'s freshness**, else pulled in full before the
process binds"; the proposal repeats it at §The steam snapshot resolver (proposal:268-270)
and states its consequence at proposal:271-273 ("A stale snapshot does not rescue a dead
key at startup: the operator's degraded option is `--no-steam`"). As shipped, an instance
booted over a snapshot older than 604,800 s answers `503 steam snapshot stale` for *every*
steam key until the refresh thread's first delta, up to 86,400 s later
(`main.rs:458-463` counts the startup as the last pull), and per §Compatibility matrix a
bound CLI fails a whole scan closed on those `503`s — while the operator has no signal
beyond `health.steam_snapshot.fetched_at`. Nothing tests the design's direction either:
`apps/registry/tests/hosted_startup.rs` has no stale-snapshot startup leg, and
`tests/steam.rs:210` only pins that a stale-but-plausible file *loads*.
*Fix*: in `load_or_pull_snapshot`, treat `is_past_age_bound(now)` as the pull case (load
only present-and-fresh, pull otherwise, keep the present-but-unloadable repair path and
the named startup refusal), and add the Test-strategy-#8/`027-C3` startup leg. If the
team instead intends to keep this posture, `027-C3` needs an amendment before the code
can ship — the comment at `main.rs:357-367` claims a §Deviations entry, which I could not
read; an undeclared change to an ADR clause is the blocker either way.

### Major

**2. A full pull that returns no records publishes an empty worldlist, and the instance
then certifies `200 []` for every game appid.**
`apps/registry/src/steam.rs:527-545` (`pull`) publishes whatever `collected` holds;
`SteamSnapshot::published` with `loaded: None` (`steam.rs:265-275`) yields an empty
snapshot and `main.rs:380-388`/`main.rs:479-487` rename it into place. Nothing in
`decode_page` (`steam.rs:862-893`) refuses `{"apps": [], "have_more_results": false}`,
and `load_snapshot` (`steam.rs:372-412`) loads an empty `apps` list. The module's own doc
asserts the opposite: "A published empty worldlist is a refusal of the pull that produced
it, never a fact about the store" (`steam.rs:190-197`) — no such refusal exists, and no
test in `tests/steam.rs` covers it. Concretely: a 200-with-empty-list answer (a key
without the `IStoreService` grant, a gateway's empty envelope) replaces a good snapshot at
startup, and for `STEAM_SNAPSHOT_MAX_AGE_SECONDS` every `steam/<appid>` lookup falls to
`state.rs:364-368` and answers a fabricated "never heard of it" — exactly the posture
§Options' "Partial snapshot serving" rejection refused to allow (proposal:133-138), and
the claim `027-L3` says the snapshot is entitled to make only when it holds the games-only
set.
*Fix*: fail the pull by name when a full pull collects no records (and when a page yields
zero records with `have_more_results` false after the first page), and refuse the empty
`apps` list at load as a malformed shape; add the leg to `tests/steam.rs`.

**3. Non-canonical `steam/<external-id>` spellings mint distinct identities for one appid.**
`state.rs:358` accepts any spelling `u64::from_str` takes (`steam/7`, `steam/007`,
`steam/+7` all parse to 7 — `keys.rs:60-85` admits all three, since
`Source::spells_lookup_key` only needs a non-empty external id), and `mint.rs:97` spells
the appid forms from `format!("steam-{external_id}")` — the *queried* spelling, a choice
`mint.rs:78-81` states as deliberate. So `steam/007` mints `my-game`, and `steam/7` — the
same appid, the same snapshot record — is pushed by the step-4 held check to
`my-game-steam-7`, an id the design reserves for a *different* record that happens to
share the name (proposal:387-390). One game ends up with two load-bearing ids, which is
the property `027-L2` exists to protect, and the design's repeated claim that the appid
spelling is "appid-unique" (proposal:385, 399-404) is false. The same hole inflates
`027-B3`: each zero-padded spelling is a novel key, so one appid can consume ~120 of the
200,000 slots, breaking the §Risks cap arithmetic that assumes one mint per key ever.
*Fix*: require the external id to be its canonical decimal spelling before the mint path
(`app.appid.to_string() == external_id`), answering `200 []` otherwise like any other
non-appid token, and spell `steam-<appid>` from the snapshot record's `appid`, not from
the key. Extend Test strategy #1's table with the `007`/`+7` rows.

**4. `write.lock` adopts a lock whose recorded pid is dead — behavior the proposal never
asked for, and unsafe across pid namespaces.**
`store_lock.rs:116-134` (`refuse_or_adopt`) and `store_lock.rs:171-194`
(`process_is_live`, `/proc/<pid>`), with the module doc at `store_lock.rs:9-18` openly
naming the narrowing ("one narrowing the proposal's runbook step … is softened at").
§Concurrency (proposal:471-479) chose the blunt rule precisely because the fixed-tmp
discipline cannot refuse a second writer: "`serve` and `query` take
`<data>/bindforge/registry/write.lock` under `create_new` at startup … and finding it
present is the named startup refusal", with "A crashed run leaves the file; the runbook
names deleting it". A pid is not a cross-namespace identity: two containers sharing a
`<data>` volume each see their own `/proc`, so a `query` in container B that cannot find
pid N — the live `serve`'s pid in container A — deletes the lock and runs as the second
persist-before-serve writer §Concurrency exists to refuse, overwriting the live instance's
`027-L2` records. That is the exact loss the proposal's r0→r2 revision log says the lock
was added to close.
*Fix*: implement the designed rule (present ⇒ refuse), and leave the corpse cleanup to the
runbook (Rollout 6). If a stale-lock path is genuinely wanted, it needs a liveness test
that cannot be satisfied across namespaces (a lock heartbeat the owner rewrites, or an
`flock` on the same inode), and a proposal amendment first.

**5. Rollout 3's gate work is absent: `check.yml` still cross-checks the whole workspace
for `x86_64-pc-windows-msvc`, and no RUSTSEC screen was added.**
`.github/workflows/check.yml:29-32` is untouched and still runs
`cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`, now including
`apps/registry` with `rustls`/`ring` (`apps/registry/Cargo.toml:21-26`). The proposal
names this leg's split as a Rollout 3 deliverable twice, for exactly this reason: "the
shipped whole-workspace `x86_64-pc-windows-msvc` compile leg (A16) drops `apps/registry`,
which is compile-checked on the host instead" (proposal:106-109) and "the `check.yml`
Windows-leg scope split that keeps `apps/registry` compile-checked on the host for `ring`'s
build script" (proposal:722-724); §Compatibility matrix promises the same
(proposal:559). The same Rollout item's "RUSTSEC advisory screen added to the gate beside
them" (proposal:721) is also absent from the diff; the pre-existing
`.github/workflows/advisories.yml` is a weekly scan whose findings deliberately "must not
fail the job", so nothing gates these advisories. The runner's green gate record does not
run either leg, so this is invisible to it.
*Fix*: land the `check.yml` split (`--workspace --exclude bindforge-registry` for the
msvc target, plus a host `cargo check -p bindforge-registry`) and the advisory screen, per
Rollout 3.

**6. The dependency floors the design names are not in the manifest.**
`apps/registry/Cargo.toml:21` declares `rustls = { version = "0.23", … }` and
`:26` `webpki-roots = "1.0"`. Those caret ranges admit everything below the floors
the design names as the mitigation — `rustls ≥ 0.23.46` (RUSTSEC-2026-0346, `Debug`
disclosing TLS secrets), `rustls-webpki ≥ 0.103.16`, `webpki-roots ≥ 1.0.10` with its
1.x-parent note (RUSTSEC-2026-0381/-0382) — proposal:93-101, Risks row at proposal:543
("named version *floors* … plus a RUSTSEC advisory screen"). Today's `Cargo.lock` happens
to satisfy them (rustls 0.23.46, rustls-webpki 0.103.17, webpki-roots 1.0.12), so no
present exposure; the loss is durability: any `cargo update` that re-resolves within
`^0.23`/`^1.0` lands on an affected version, and per finding 5 no gate screens it. The
design's assumption here is the §Options row, and the manifest silently widened it.
*Fix*: state the floors where the design states them —
`rustls = { version = "0.23.46", default-features = false, features = [...] }`,
`webpki-roots = "1.0.10"` — and pin `rustls-webpki` to the floor too, or add a `deny.toml`
`[bans]` minimum-version / `[advisories]` entry so the floor is enforced rather than
incidental.

**7. Startup resolves `partner.steam-api.com` even when no pull is needed, breaking the
suite's own no-outbound rule.**
`main.rs:338-355` (`Config::store`) resolves the pinned authority through
`to_socket_addrs` and is called at `main.rs:305`, *before*
`load_or_pull_snapshot` decides whether to dial anything. Every hosted suite leg that runs
the real binary with a key set and a planted snapshot — e.g.
`tests/hosted_end_to_end.rs:286`, `:369`, `:389`, `:414`, `:523`, `:543` — therefore
issues a live DNS lookup for the real store authority, which Test strategy #17 forbids in
terms: "nothing resolves `partner.steam-api.com` from a test (Test strategy #6's TLS cases
dial `127.0.0.1`)" (proposal:693-696). It also makes a hosted start hard-fail on a
networkless restart ("the steam store authority … could not be resolved") when the
snapshot on disk is fresh and no pull was ever needed — a self-inflicted availability
loss on the very posture §Risks calls the operator's dial.
*Fix*: resolve lazily — decide load-vs-pull first, and resolve only on the branch that
will dial (passing the resolved address into the store the caller then builds). Add a
suite guard that no test resolves the authority (bind the resolution behind an injectable
resolver, or assert the failure text is unreachable with a fresh snapshot and no DNS).

### Minor

**8. Rollout 1's ledger annotations did not land.** The diff touches exactly one file
under `docs/` besides the run record (ADR-027's own unit-handle span), and
`grep`ing `docs/adrs/026-registry-service.md` and `docs/adrs/024-game-slug-identity.md`
finds no `027` reference. Proposal Rollout 1 (proposal:702-710) requires `amended-by`
lines on `026-C1`/`026-C2`, retirement lines on `026-C6`, `026-C7`, `026-B1`–`B3`,
`026-L4`'s `amended-by 027-L3`, `026-L5`'s caching half, `026-R2`'s intake half, and the
`024-C2` cross-reference. The tree now ships code that deletes `cache.rs`,
`upstream.rs`, `endpoint.rs` and the `026-B1`–`B3` constants while those contracts still
read as live law (`021-L4`). *Fix*: land Rollout 1's annotations.

**9. The operator runbook (`docs/registry-hosting.md`, Rollout 6) is absent.** The whole
of proposal:736-756 — key custody and the CWE-526 stance, the no-URL-in-logs rule, file
custody for corpus/issued/retired-ids/snapshot, forcing a full refresh, deleting a
crashed run's `write.lock`, the retired-list size check, the `026` document-pair cleanup
with no migration, `query` as a stopped-instance tool, the restart policy bounding daily
startup pulls, snapshot age as user-visible availability, the cap arithmetic, and the
Steam ToS obligations "by name" — has no artifact in the diff. Several findings above
(1, 4, 7) lean on runbook steps that do not exist. *Fix*: land Rollout 6.

**10. `retired id list full` reaches the wire as `503 Bad Request` with the `bad_request`
token.** `state.rs:662-666` returns `AdminRefusal::Reject { status: 503, … }`, and
`admin.rs:231-249` (`Answer::refused`) has no 503 arm, so `serve.rs:203` +
`serve.rs:355-369` write `HTTP/1.1 503 Bad Request` with `{"error":"bad_request"}`; the
name survives only in the body's prose, which is why
`tests/retired_ids_and_curation.rs:496-507` has to accept "line **or** body contains
retired … and full/cap". That contradicts the discipline the same branch applies to the
candidate face (`serve.rs:304-313`: the reason phrase carries the name because the status
line is all the shipped CLI shows) and the design's "named refusal" wording
(proposal:440-446). *Fix*: give `Answer::refused` a 503 arm (reason + token), or carry
`Named` through `AdminRefusal` so the admin face renders the same names the candidate face
does; tighten the test to the status line.

**11. `owner_only_dir`'s parent-mode rule changed beyond the proposal.**
`persist.rs:178-203` replaced an unconditional `set_permissions(0o700)` with a conditional
(`if mode & 0o077 != 0`). A2 (proposal:55) records the discipline as "parent `0700`"; the
change is motivated by Test strategy #4's injected persist failure ("a save that chmods
the directory back to `0700` first answers that write `200`"), which is a test-harness
problem the proposal did not solve by widening a shipped invariant. The ceiling still
holds (group/other bits are cleared), and a stricter directory now stays stricter and
fails the write closed — so the risk is the silent drift, not a live hole.
*Fix*: keep the ceiling but make the intent explicit (a named
`assert_within_owner_only_ceiling` that never widens), or inject the read-only directory
in the test without touching the shipped rule; record whichever is chosen under
§Deviations.

**12. §Budgets' "one file" claim is not met, and the comment justifying the split
describes a future that already happened.** `MAX_STEAM_SNAPSHOT_OCTETS`,
`STEAM_SNAPSHOT_MAX_AGE_SECONDS`, `STEAM_REFRESH_MAX_PAGES`, `STEAM_PAGE_*`,
`STEAM_CONNECT_TIMEOUT`/`READ_TIMEOUT`/`RETRY_SECONDS` and
`STEAM_PULL_DEADLINE_SECONDS` live in `steam.rs:58-117`; `constants.rs:86-110` carries
only the three issued/retired budgets and a pointer note. The proposal says the family is
"new `constants.rs` family … kept in the one file" (proposal:504-526). The justification
at `steam.rs:61-66` — "They live here until the increment that wires this resolver into
the serving face, because the binary declares its own copy of `constants` and has no
reader for them yet" — is stale in this very diff: `main.rs:39` already uses
`bindforge_registry::constants`, the resolver is wired, and the whole reason the family
was landed is Rollout 3. *Fix*: move the family to `constants.rs` as §Budgets states, and
delete the stale comment.

### Nit

**13.** `steam.rs:901-920` re-implements the whitespace-collapse-and-clamp rule
(`normalize_store_name` + `clamp_to_name_bound`) instead of calling
`discovery::game::friendly_name` "through that existing `discovery` dependency" as A7 and
proposal:306-312 specify. I read both and the two are behaviorally identical (leading and
trailing whitespace dropped, interior runs to one space, longest char-boundary prefix
inside `MAX_NATIVE_NAME_OCTETS`), so nothing is wrong — but the design named the call, and
a future divergence in `crates/discovery/src/game.rs` will now be invisible here.
*Fix*: call `discovery::game::friendly_name` after the `char::is_control()` strip.

**14.** `steam.rs:883-887` turns a record with a missing or non-string `name` into an
empty name rather than refusing the page; A17's envelope carries `name` on every record,
and §The store pull's refusal list covers "a body that is not the envelope". Silently
blanking a malformed record is defensible under "nothing is dropped", but it is a third
posture the proposal does not name. *Fix*: either refuse the page with the malformed-shape
name, or say in the comment that a missing `name` is the empty-name posture.

**15.** Naming/structure: the design calls the seam `StoreTransport`; the code calls it
`steam.rs:439-449` `Transport` with a `Loopback` variant that is only ever production-
adjacent, and production builds it through `SteamStore::for_test(...)` +
`with_production_transport(...)` (`main.rs:354`). The `_test` name on the only production
construction path hides which transport a hosted instance runs. *Fix*: rename to
`StoreTransport` and give the production constructor its own name
(`SteamStore::production(address, key)`).

## Evidence limitations

- Checklist item 3 (undeclared deviation ⇒ blocker) and item 5 (build-notes claims true)
  could not be evaluated: the caller's input list does not include
  `03-build-notes.md`, so I did not read it. Finding 1 is written against the ADR clause
  and the proposal, and stands as a blocker on the contract either way: if the deviation
  is declared, it still needs an ADR amendment; if it is not, item 3 makes it one.
- I did not run CI, so finding 5 is stated against the design's own A16/§Options analysis
  of `ring`'s target build script plus the untouched workflow file, not against an observed
  CI failure. `cargo check --target x86_64-pc-windows-msvc` was not run locally.
- Rollout 4's "issued-document rewrite measured at 100k and 200k records, recorded in
  §Budgets" produces no code or test artifact in this diff and no code deliverable was
  expected from it; recorded as unverified rather than missing.

## FYI (out of specialty, no severity)

`state.rs:417` clones the whole issued `Catalog` (which rebuilds the postings index,
`corpus.rs:361-365`) for every mint, where §Concurrency specifies an incremental posting
insert and a write straight into the buffer — the performance seat's ground.
`apps/registry/src/persist.rs` is compiled into both the lib (`lib.rs`) and the bin
(`main.rs:24`), so the persist discipline exists twice in one binary; and the retired
suites (A15) are `#[ignore]`d in place with their `--upstream` harnesses still compiling,
rather than retired per §Hosted mode.

## Verdict

request-changes — findings 1 (blocker) and 2–7 (major) must close before this seat
approves; findings 8–15 should be fixed or explicitly deferred in the build notes.
