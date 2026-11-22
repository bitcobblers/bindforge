# Build notes — ADR-027 hosted registry with store resolvers

## Progress

**Fix round r1 (code-review round 1). Everything below is in the tree this stage leaves behind
— and every posture claimed here was run; see *Gate posture*.**

**r1 continuation (the stage-timeout retry).** This stage made **no source change**: the tree at
the reviewed tip (`8d3e8edc`, over `b2dd33e7` fix + `f52686fe` test re-pin) is left as it stands,
and `test-update` (f52686fe) has since landed, so the eight legs the earlier "Test updates needed"
list named are re-pinned and the runner's captured gate at 2026-11-22T09:51:31Z is green on this
tree (see *Gate posture*). The two open seats — `security-r1c1`, `testing-r1c1` — are recorded
under `docs/designs/.../reviews/`, which this stage may not read (stage access restriction), so
the audit below is **independent**, not a reading of the findings.

An independent read of the security-relevant and contract-faithful surfaces found the reviewed
code already faithful to the proposal on every concrete point a security seat names:
`steam.rs` verifies the pinned authority through rustls' root-certificate client (cleartext is
reachable only via `for_test`), the key is `form_encode`d into one fixed request line and never a
diagnostic (every failure line names a `kind()`/condition, never the URI or key), `read_body`
bounds a page before draining it, and the pull deadline is paid at every read; `admin.rs` is a
constant-time bearer fold with loopback-or-bearer enablement and the `Origin`/`Content-Type`
gates actually enforced in `serve::admin_gate`; the issued-delete route re-validates the
`game_id` grammar; health is exactly `{status, mode, corpus_entries, steam_snapshot}` and stats is
`{games, source_keys, revision, issued}`; the `--upstream`/`--refresh-interval` retirement and the
retired-document-pair presence refusals are in the parser (`main.rs`), not the dead `endpoint.rs`.
The only residue is `src/{cache,upstream,endpoint}.rs`, which declare constants that no longer
exist and are declared by no target — inert, and their removal is a runner delete, not a `moves`
rename. The testing seat's findings are test-side; they are routed to `test-update` and this stage
does not touch test files.

1. **§Budgets in the one file it names.** The `MAX_STEAM_*` / `STEAM_*` store-resolver family
   (snapshot, page, retry, deadline, refresh, age, timeout) lives in
   `apps/registry/src/constants.rs`, re-exported by `steam.rs`.
2. **Startup is `027-C3`'s literal load-or-pull.** `steam::startup_snapshot` (library face)
   loads a snapshot only when present, loadable **and** inside
   `STEAM_SNAPSHOT_MAX_AGE_SECONDS`; anything else pulls in full and publishes before bind.
3. **The cross-process lock is presence, not liveness.** `store_lock.rs` dropped the
   `/proc` stale-owner adoption: a present `write.lock` is the named refusal and the stamped
   pid is operator-facing text — §Concurrency's sentence plus Rollout 6's runbook act.
4. **The mint pays §Concurrency's shape.** `Catalog::to_bytes_with` serializes the new
   document against the live model and `Catalog::insert` splices the record and inserts the
   posting instead of rebuilding the index; the per-mint model clone is gone, and
   `corpus.rs`'s unit test pins the spliced bytes equal to the whole-document spelling, and
   the mint's three document reads now follow §Concurrency's stated lock order.
5. **Named refusals keep their names on the write path.** `AdminRefusal::Named` +
   `admin.rs::Answer::named`, so `retired id list full` leaves as a `503` that says so
   rather than `"Bad Request"`.
6. **The verified transport is the default.** `SteamStore::production` is the only
   constructor the binary calls (pinned authority held as a *name*, rustls config built
   there); cleartext is reachable only through `for_test`. `Endpoint::Pinned | Address` moved
   resolution to the dial, walking every address the name yields, so a run with a fresh
   snapshot never consults the resolver (Test strategy #17).
7. **Pull discipline closed out.** Non-`200` (not "non-2xx") fails a page; the deadline is
   paid inside the header and body reads, not only between attempts; an empty worldlist fails
   a full pull and a continuation page carrying nothing fails the pull; a record whose `name`
   is missing or not a string takes the empty-label posture rather than failing the supply.
   Publishes are compact and an empty-`apps` document is a loader refusal: an empty snapshot
   may never certify `200 []` for every appid.
8. **One appid, one key spelling**: only the canonical decimal appid is a `steam/…` lookup
   (`007`, `+7` are not) and the mint's appid spellings come from the record's own `appid`.
   **The `--corpus` exception compares files, not spellings** (`same_document`: dev+inode on
   unix, case-insensitive elsewhere).
9. **Refresh thread** anchored on the loaded snapshot's `fetched_at`; a pull whose *publish*
   failed is `PullOutcome::Failed`, so the retry is on `STEAM_RETRY_SECONDS`.
   **`persist`'s owner-only directory mode is a ceiling** (`narrow_to_owner_only`): bits are
   cleared, never widened, so a `0500` `<data>` keeps failing writes closed.
10. **`check.yml`'s `advisory-gate` job can actually turn something red**: it ran in
    `tools/orchestration`, which has no `Cargo.lock` and no `deny.toml`; it runs at the repo
    root now. Its `actions/checkout` SHA pin is corrected to the digest `v4.2.2` names, and
    its rationale moved inside the job — the shipped gate contract reads the `jobs:` mapping
    line by line, and a comment at that indent reads as a job. The file's header now names
    the third job too.
11. **Library face** (`src/lib.rs`): `constants`, `corpus`, `keys`, `mint`, `persist`,
    `retired`, `state`, `steam` — widened because the age-bound postures have no route
    through a booting binary any more (see *Test updates needed*).

**Last validation (this continuation stage).** The only gate leg this stage ran itself is
`cargo fmt --all -- --check` → exit 0, no output (fast/parse-only; the build tree is cold here,
so the workspace clippy/test legs were not re-run and no Rust source was changed to require them).
The authoritative posture for the current tree is the runner's captured gate quoted under
*Gate posture*: green on all four legs at 2026-11-22T09:51:31Z.

**Next concrete action.** The two open seats are `security-r1c1` and `testing-r1c1`. This stage
could not open them (review access restriction), so it changed nothing it could not justify from
the proposal alone. If a named finding survives the independent audit above, it needs its text
routed to a stage that may read `reviews/`; the `testing-r1c1` findings are test-side and belong to
`test-update`. No source change is owed that this tree does not already carry.

## Deviations from the proposal

- **Library face breadth.** The proposal implies the crate splits into a thin CLI over a
  full library face. `main.rs` now shares `constants`, `steam`, `state`, `corpus`,
  `keys`, `mint`, `persist`, `retired` with the library target, but `serve`, `admin`,
  `http` and `wire` still have binary-only module declarations until the CLI-mode suites
  move. Nothing in ADR-027 depends on finishing that; the face grew because the age-bound
  postures otherwise have no reachable route.
- **Provider choice inside the pinned dependency.** `rustls` is built with
  `default-features = false, features = ["ring", "std", "tls12"]`. The design pins rustls
  + webpki-roots, not a crypto provider; the default `aws-lc-rs` would add a second
  language's build graph to a crate this framework ships as one binary.
- **Startup order, made explicit.** `<data>` → `write.lock` → steam-key requirement →
  retired-document refusal → corpus load → snapshot load-or-pull → issued → retired. The
  proposal fixes the first two and leaves the rest to the named-refusal family; this
  order is what makes a `query` beside a live `serve` refuse for the lock.
- **Owner-only directory is a ceiling.** A2 states the mode a write *creates*
  (`parent 0700`); this tree never widens a directory that is already there, because the
  alternative silently overrides an operator's read-only data directory and answers a
  write that must fail (Test strategy #4).
- **The mint's appid spelling is the record's, not the key text's.** §Issued entries
  step 5 spells `steam-<appid>` "from the queried key's external id"; `state.rs` accepts
  only the canonical decimal spelling and passes the snapshot record's own `appid`, which
  that gate makes the same string. The spelling therefore cannot drift from the record it
  certifies.
- **Retired modules are still on disk.** `apps/registry/src/{cache,upstream,endpoint}.rs`
  are named for retirement by §Hosted mode and are declared by no target any more, but a
  stage may not `rm`: file removal belongs to the runner, and `moves` names renames, not
  deletions. They compile nowhere and are referenced nowhere; the deletion is the
  runner's act.

## Test updates needed (routed to `test-update`)

Eight legs in seven targets, all of them suite-side postures the source change makes
unreachable-by-that-route. No change to source I own turns any of them green.

- **The three restart-a-killed-run legs**, which all die on the named refusal
  `registry data directory is locked … (stamped owner <pid>)`:
  `hosted_startup.rs::a_run_takes_the_write_lock_and_a_clean_exit_removes_it`,
  `data_layout.rs::a_mint_lands_under_the_xdg_data_home_spelling_and_nothing_else_lands_beside_it`,
  `issued_entries.rs::an_issued_record_is_on_disk_before_the_answer_and_survives_a_restart`.
  Each starts a second instance after `common::Registry::drop` `kill()`ed the first, and a
  killed run is exactly the crashed run §Concurrency says leaves `write.lock` behind — the
  proposal's recovery is Rollout 6's runbook act, not an act a run performs. Stale-lock
  adoption by pid liveness is deliberately gone (it decided between processes who holds
  the catalog on evidence a lock file cannot carry). Fix: the restart seam either deletes
  `<data>/bindforge/registry/write.lock` between the killed run and the next start — one
  `common` helper, three call sites — or the legs use a first run that exits by itself.
- **The four stale-boot legs**:
  `hosted_lookup_policy.rs::a_snapshot_past_its_age_bound_turns_steam_misses_into_the_named_503`,
  `hosted_lookup_policy.rs::the_steam_refusals_name_their_own_reason`,
  `health_route.rs::a_stale_snapshot_refuses_lookups_and_shows_in_the_health_body_never_in_its_code`,
  `cli_end_to_end.rs::a_stale_snapshot_fails_the_cli_scan_closed_on_the_named_refusal`.
  All four plant a snapshot whose `fetched_at` is past `027-B2`'s age bound and expect the
  instance to boot on it. §The steam snapshot resolver's startup clause is "load the
  snapshot if present, loadable, **and** fresh, else pull in full", so startup now pulls
  the compile-time-pinned authority — a dial no test may make (Test strategy #17) — and each
  leg dies at the harness's 20 s bound with an empty stderr. The
  postures are still pinnable, just not through a booting binary: the library face grew
  for exactly this (`src/lib.rs`'s header records it), e.g.
  `State::hosted(Catalog::empty(…), Catalog::empty(…), RetiredIds::empty(…),
  Some(SteamSnapshot::for_test(now - 604_800 - 1, &[(appid, "Name")])),
  SteamPosture::Active).candidates("steam/<other>", now)` answers
  `Answer::Refusal(Named::SteamSnapshotStale)`, with `reason()` `"steam snapshot stale"`
  and `code()` `steam_snapshot_stale`; `--no-steam` and the corpus/issued half keep their
  socket routes, and the four legs' socket-only assertions (health body, CLI scan-closed)
  need to be re-expressed against that face plus the reachable `--no-steam` socket case.
- **`crates/core/tests/gate_contract.rs::the_rust_gate_runs_exactly_the_three_gate_commands_with_no_database`**
  pins `check.yml`'s job set to `["rust", "orchestration"]` and now sees
  `["rust", "orchestration", "advisory-gate"]`. The third job is Rollout 3's advisory
  screen, which design 027's Test strategy #17 names as one of the two ways the cargo job
  changes ("the Windows leg's scope, and the advisory screen"), so the assertion is the
  thing that is stale. Its other `gate_contract` legs already pass against the new job
  (`contents: read` permissions, `timeout-minutes: 30`, full SHA pins).

## Gate posture

The runner's captured gate for the code under review (`gate at 2026-11-22T09:51:31.853Z`) is
green on all four legs — `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace --locked`
each exit 0. That record is output the runner captured, not an instruction, and it is the posture
of this tree: `test-update` (commit `f52686fe`) landed after the earlier fix round and re-pinned
the eight legs the previous *Test updates needed* list named (three `write.lock` restart seams,
four stale-boot postures moved to the library face, one `gate_contract` job-set re-pin), so the
seven-target red that earlier fix round reported against the 07:10:57Z gate is resolved on this
tree. The earlier fix round's own per-leg measurements (registry in-source, `steam`/`mint_rule`/
`retired_ids_and_curation`/`hosted_health_stats`/`hosted_end_to_end`/`persist_and_concurrency`/
`response_shape`/`catalog_document`, and the orchestration `692 passed | 2 skipped`) stand as the
detail behind that green.

What this continuation stage measured itself: `cargo fmt --all -- --check` → exit 0, no output.
This stage changed no Rust source, so the clippy/test legs were not re-run here (the build tree
is cold in this shell) and their posture is the runner's captured green above, unchanged.

- This stage does not commit; the runner takes the tree as it stands.

## As-built close-out (2026-11-22, doc pass over `a22cfd26…15eaa58e`)

This pass compared the whole run range against the approved proposal and wrote
`02-proposal.md`'s **§As-built (2026-11-22)** — the drift list, with each item's
reason and the section it touches. That section is the record; it is not repeated
here except where a *build* fact belongs. This pass changed no code and no test.

**Rollout state of the tree** (the labels are the proposal's §Rollout items):

- **Item 2 — landed.** `Mode::Hosted` (`state.rs`, `as_str() = "hosted"`, and the
  startup line reads it rather than hard-coding the word); the `--upstream` /
  `--refresh-interval` retirement refusals in `Config::parse` (parser, not the
  dead `endpoint.rs`); the `cache-v1.json` / `catalog-v1.json` presence refusal
  with the `--corpus` exception; the missing-key and missing-`<data>` refusals;
  the four-field health re-pin; and `027-S1` — `data/catalog/v1/catalog.json` is
  untouched by the range, all six seed records still serve, and the seed's `epic/`
  key still answers from curation alone (an `epic/` key the corpus does *not* hold
  is the named `epic resolver inactive`).
- **Item 3 — landed.** `steam.rs` (the `027-C3` document and its loader, the
  `char::is_control` strip into `discovery::game::friendly_name`, the paged pull
  with `max_results=50000` pinned every page, the page backoff, the
  `steam pull deadline` refusal, and the pure `next_refresh_seconds` cadence);
  the `constants.rs` §Budgets family at the proposal's values; startup
  load-or-pull; `--no-steam`; the dependency floors and the `check.yml`
  Windows-leg split plus the `advisory-gate` screen and the `README.md`
  C-toolchain line; and the structural `027-L1` / `027-L4` seams (the snapshot is
  written only by the pull seam; `no_pull_failure_line_carries_the_key_or_the_request_uri`
  pins the other).
- **Item 4 — landed except its measurement.** `issued-v1.json`,
  `retired-ids-v1.json`, `mint.rs`, the persist-before-serve path under the one
  `mutations` hold, `store_lock.rs`'s `write.lock`, the record-count and octet
  cap postures, `027-L5` suppression, and the amended miss posture. The 100k /
  200k issued-document rewrite was **never measured**, so the figure §Budgets
  promises is still owed.
- **Item 5 — landed.** `DELETE /v1/admin/issued/<game_id>` behind the identical
  enablement/auth rule, the retired-first two-persist delete, and `stats.issued`.
- **Items 1 and 6 — not landed.** No `docs/adrs/` annotation of `026-*` or
  `024-C2` (the only ADR edit in the range is ADR-027's own `025-R2` code-span
  fix), and no `docs/registry-hosting.md`. Both are file-creation/ownership work
  no stage of this run held; the runbook gap is the one with consequences, because
  deleting a crashed run's `write.lock`, forcing a full refresh, and reading the
  retired file's size are code's only recovery paths for those states.

**Where each Test strategy item's legs live**, for whoever re-pins next: #1
`mint_rule.rs`; #2 and #6 `steam.rs` (with the TLS pair over
`tests/fixtures/store/`); #3 `hosted_lookup_policy.rs`; #4 `issued_entries.rs`;
#5 `retired_ids_and_curation.rs`; #7 `issued_entries.rs` (its `026-C3` refusals
stay pinned in `catalog_document.rs`); #8 `hosted_startup.rs`; #9
`hosted_health_stats.rs` and `wire.rs`; #10 and #14
`retired_ids_and_curation.rs` with `admin_auth.rs` and `admin_mutations.rs`; #11
`steam.rs`'s three cadence legs; #12
`hosted_end_to_end.rs`; #13 `hosted_lookup_policy.rs` and `health_route.rs` (both
on the library face for the stale half); #15 `hosted_end_to_end.rs`'s three
`query` legs, with the bound-CLI rows of §Compatibility matrix in
`cli_end_to_end.rs`; #16 `data_layout.rs`'s `DataLayout`/env legs and the
`check.yml` split; #17 `steam.rs` plus the
`crates/core/tests/gate_contract.rs` job-set re-pin (three jobs now). The four
A15 caching suites are `#[ignore]`d in place, each leg with the reason and the
suite that replaced it.

**Still owed to the runner, not to a stage** (file removals are the runner's
`moves`/deletes, so no stage could do them): `apps/registry/src/{cache,
upstream,endpoint}.rs` and the four ignored caching suites. They compile nowhere
and are declared by no target; until they go, the crate's file inventory is ahead
of §Hosted mode's retired list.

**Gate posture for this pass.** This stage's rights are read-only inspection plus
its two artifact files; `cargo` is not in its allowlist and it ran **no** gate leg.
The newest runner-captured record is the one the round-2 code seats cite — green
at `2026-11-22T11:13:49.574Z` on all four legs — and those seats additionally
record their own `cargo test -p bindforge-registry --test steam --locked` runs
green on the `f875d407` + `887e39de` code (the `io_reason` chain-refusal fix and
the real-TLS fixture pair). For the current tip that supersedes the
`09:51:31.853Z` record quoted under *Gate posture*; nothing else in this file's
earlier sections is contradicted by the tip.

This pass does not commit; the runner takes the tree as it stands.

## Doc pass r2 (2026-11-22) — verification of the same range, nothing new owed

Second documentation pass over `a22cfd26…HEAD`. **The range is unchanged**: the
branch tip is still `15eaa58e`, the same commit the *As-built close-out* above
compared, so this pass added no new drift entries to `02-proposal.md`'s §As-built
and appended to it only a **Verification addendum** recording that the drift list
was re-checked against the files rather than carried forward. Nothing in this
file's earlier sections is contradicted by that re-check, and the *As-built
close-out*'s Rollout state, Test-strategy index, and "Still owed to the runner"
list stand as written.

**What was re-checked, mechanically** (read-only inspection; file and symbol
names are the tree's):

- §Budgets' figures: every constant in `apps/registry/src/constants.rs` matches
  the value §Budgets prints, including the reused `026-C7` tolerance pair and the
  `STEAM_PULL_DEADLINE_SECONDS = 1_920` grid figure; `026-B1`–`B3` and the
  upstream timeouts are gone from the file, replaced by retirement comments.
- `apps/registry/Cargo.toml`: `rustls = ">=0.23.46, <0.24"` with
  `default-features = false` and `["ring", "std", "tls12"]`,
  `webpki-roots = ">=1.0.10, <2"`, and the never-imported `rustls-webpki` floor
  declaration — the *Dependency floors* entry, spelled as Cargo says it.
- `.github/workflows/check.yml`: three jobs, `cargo check --workspace --exclude
  bindforge-registry --all-targets --target x86_64-pc-windows-msvc` plus the host
  `cargo check -p bindforge-registry --all-targets`, and
  `cargo deny check advisories` in `advisory-gate`; `README.md` carries the
  C-toolchain prerequisite line.
- Retirement in place: `src/{cache,upstream,endpoint}.rs` exist and are declared
  by no target (no `mod cache|upstream|endpoint` in `main.rs`); the four A15
  suites are `#[ignore]`d leg by leg with the reason on each leg;
  `src/lib.rs` publishes exactly the eight modules the *library face* entry names.
- Named refusals: the policy/mint/cap/retired set (`steam snapshot stale`,
  `no steam snapshot`, `steam resolver inactive`, `epic resolver inactive`,
  `mint cap reached`, `mint identity held`, `mint identity over bound`,
  `retired id list full`) is `state.rs`'s `reason()` table;
  `registry data directory is locked` is `store_lock.rs`, with no `/proc` liveness
  path anywhere in it; the two flag-retirement lines are `main.rs`, not the dead
  `endpoint.rs`; `steam pull deadline` is `steam.rs::deadline_refusal`.
- Pull shape: `STORE_PATH = /IStoreService/GetAppList/v1` (the missing trailing
  slash the As-built records), `max_results` on every page, `last_appid` and
  `if_modified_since` continuation, the empty-worldlist and record-less-continuation
  refusals, `SteamSnapshot::published(now, …)` taking the caller's pre-pull clock,
  `merge` as the one linear two-way merge, and the thread's wait computed by
  `steam::next_refresh_seconds(now, last_pull_at, outcome)` seeded from the loaded
  snapshot's `fetched_at` and advanced to the pull's own start.
- Pins the As-built cites exist where it says:
  `mint_rule.rs::mints_steam_appid_when_the_bound_would_be_breached`,
  `steam.rs`'s `("redirect", 302 …)` case row,
  `steam.rs::no_pull_failure_line_carries_the_key_or_the_request_uri`, the four
  `tests/fixtures/store/*.der` files plus their README, and the stale-boot legs on
  `State::hosted(..)` with `SteamSnapshot::for_test` in `hosted_lookup_policy.rs`
  and `health_route.rs`, with `cli_end_to_end.rs`'s leg over `--no-steam`.
- Rollout gaps: `docs/registry-hosting.md` absent; `docs/adrs/026-registry-service.md`
  carries no `027-*` `amended-by`/retirement line and
  `docs/adrs/024-game-slug-identity.md` carries no `027-C5` cross-reference.

**Two precisions folded into the As-built (wording, not posture).** (i) The
owner-only ceiling is `persist::owner_only_dir` delegating to the private
`narrow_to_owner_only` — this file item 9 and the As-built each named one half.
(ii) Since `f875d407`, a pull-failure line also renders the rustls/io error chain
through `steam.rs::io_reason`, so the diagnostics rule §The store pull states
("no diagnostics carry the request URI or key") is now satisfied alongside an
error identity rather than in place of one; the no-key-no-URI pin above is what
keeps that honest.

**Reading notes for the earlier sections.** The *Progress* header and its
"Next concrete action" paragraph describe the tree at the r1 continuation
(`9041f47e`), not the tip; the *As-built close-out* and this section supersede
them on Rollout state and owed work. The *Test updates needed* list is closed —
`f52686fe` re-pinned those eight legs, as the close-out records.

**Gate posture for this pass.** Read-only inspection plus this file and
`02-proposal.md`; `cargo` and `npm` are not in this stage's rights and it **ran
no gate leg**. The newest runner-captured record quoted in this file remains the
`2026-11-22T11:13:49.574Z` green on all four legs cited under *Gate posture for
this pass* above, and nothing in this pass changed a tracked file outside the two
artifacts, so no leg needed re-running here.

**Still owed** — unchanged from the close-out and unchanged by this pass: the
100k/200k issued-document rewrite (Rollout 4's number), Rollout 1's ledger
annotations, `docs/registry-hosting.md` (Rollout 6), and the runner's deletion of
`apps/registry/src/{cache,upstream,endpoint}.rs` and the four `#[ignore]`d caching
suites.

This pass does not commit; the runner takes the tree as it stands.

## Operator remediation and the round r2 (2026-11-22)

**What the mutating loop did not reach.** Code round r1 ended in escalation: r1c1 came back with
all five seats dissenting and fourteen blocking findings, and `fix` hit the 30-minute
`stage-timeout` twice (`stageFailures.fix` reached 5), so the runner stopped the run at the
fix-round backstop rather than spend another half hour of coder on the same diff. Triaged by hand,
the fourteen split into two clusters the operator then landed **as commits on the run branch** —
outside any stage, with no stage credit:

1. **Rollout 3's gate work, which the coder never reached** — `3814c48c` splits the
   `x86_64-pc-windows-msvc` leg off `apps/registry` (its `ring` provider compiles C for the target
   and that runner has no MSVC C toolchain) and adds the `advisory-gate` job running
   `cargo deny check advisories` as a PR check; `97f90358` names the C-toolchain prerequisite in
   `README.md`; `f2ffa26d` declares the `rustls` / `rustls-webpki` / `webpki-roots` version floors
   in `apps/registry/Cargo.toml` with the lock edge.
2. **The one major `security` and `testing` both carried** — `f875d407` gives a wire-leg refusal
   its `rustls` identity (`io_reason`: `InvalidData` on this seam is only ever `rustls`, so the
   line now reads `invalid data: invalid peer certificate: UnknownIssuer`), and `887e39de` replaces
   the plaintext-socket stand-in with Test strategy #6's real pair: two checked-in self-signed
   end-entity certificates under `apps/registry/tests/fixtures/store/` (README carries the
   `openssl req` recipe and the November-2046 `notAfter`) answered by loopback
   `rustls::ServerConnection` fakes, the verified-name case pinned on `UnknownIssuer`, and the
   mutation catcher watching the **decrypted** stream.

The runner's ownership sweep preserved these as foreign commits, which is the designed behaviour —
stage edits outside a stage's ownership are reverted, operator commits are not.

**How r2 was opened.** A plain `resume` re-escalates from this checkpoint: `dissentingSeats` reads
`request-changes` regardless of stamp, and the transition out of a dissenting `code-review` needs an
unspent fix round (`codeBudgetUsed` was 5/5). The operator therefore used the human-directed retry
— `pipeline resume … --stage code-review --extend-budget 2` — which opens round r2 under a recorded
`operatorExtension` grant, refreshes `codeStamp` to HEAD, and runs the dissent pair. `r2c0`
(`security`, `testing`) approved; `r2c1` ratified the three seats whose approvals predated the
increment (`correctness`, `compatibility`, `performance`); the panel converged and `doc-sync` closed
the run at `codeBudgetUsed` 6 and 17/20 code review sessions.

**Gate posture for this section.** The operator re-ran all four repo gates on the final tree before
merging: `cargo fmt --all -- --check` clean, `cargo clippy --workspace --all-targets -- -D warnings`
clean, `cargo test --workspace --locked` green (72 result lines, no failures), `npm run check` green
(61 files, 692 passed, 2 skipped). One earlier `cargo test --workspace --locked` invocation reported
a `bindforge-core (lib test)` compile failure that three subsequent full invocations, including the
last two on the committed tree, did not reproduce; it is recorded here rather than smoothed over.

**Newly owed by this section**: nothing. The two r2c0 reports' non-blocking items (one minor, two
nits) join the owed list above; the blockers they were written against are closed in code.
