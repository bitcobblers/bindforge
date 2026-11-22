# Code review — compatibility seat — round 1, pass c0

- Change under review: `git diff a22cfd2694fde8d84be132c8d78faf09fda1adf4...HEAD -- ':(exclude)docs/designs'`
  (39 files, +8991/−1435; `apps/registry` source + suites, `Cargo.toml`, `Cargo.lock`).
- Inputs used: that diff, `02-proposal.md`, `docs/adrs/027-hosted-registry-with-store-resolvers.md`,
  the runner's gate record (green: `npm run check`, `cargo fmt`, `cargo clippy`, `cargo test --workspace --locked`),
  this seat's own previous findings (none — first pass). No other review artifact was opened.
- Specialty applied: `references/compatibility.md` only.

## Checklist walk

1. **Platform code matches the §Compatibility matrix.** Partially. OS-specific code is correctly guarded
   (`persist::owner_only_dir`/`owner_only_new_file`/`owner_only` under `#[cfg(unix)]` with a `not(unix)`
   arm, `persist::DataLayout::host()`'s `cfg!(windows)` branch, `store_lock::process_is_live`'s
   `cfg(target_os = "linux")` `/proc` probe with a fail-closed `not(linux)` arm). Paths are all `PathBuf::join`
   (`persist::registry_dir`, `document_path`, `store_lock::take`), and `echo_path`'s `MAIN_SEPARATOR` use is
   display-only — checklist item 3 clean. **But** the Windows row's stated support ("the shipped leg compiles
   the workspace for `x86_64-pc-windows-msvc` … Rollout 3 keeps `apps/registry` off that leg") and the Linux
   row's stated prerequisite ("needs a C toolchain for `ring`'s build script … `README.md` prerequisite per
   Rollout 3") are both unimplemented: see findings 1 and 2.
2. **Node floor.** Not applicable — no `tools/orchestration` or other Node path is in the review scope, and
   the gate's `npm run check` leg is green and unchanged.
3. **Paths.** Clean (see item 1). One nit on the `--corpus`-spare comparison, finding 6.
4. **Guarded platform imports.** Clean — no platform-only module is loaded unconditionally; `rustls` /
   `webpki-roots` are cross-platform and confined to `steam.rs`'s pull seam, as §Options requires.
5. **Locale / layout reality.** Clean. Non-ASCII store names survive every format that holds them:
   `steam::normalize_store_name` strips `char::is_control` then clamps on a `char` boundary to
   `MAX_NATIVE_NAME_OCTETS`, `load_snapshot` re-checks both bounds so a published file always reloads, and an
   all-control/all-whitespace title stores empty — which `corpus::record_from_value` admits (length-only).
   The mint's finite Latin-1/Latin-Extended-A fold (`mint::fold_latin`) plus `steam-<appid>` for a name with
   no Latin base letter keeps minted ids inside the ASCII `[a-z0-9-]` grammar, so `retired::load`'s
   `str::cmp` byte order and `is_slug_name` hold on every platform. Documents are written as UTF-8 JSON with
   no locale dependency; inbound keys are percent-decoded before the cap rides the decoded bytes.
6. **Native dependencies introduced in the diff.** **Finding 1 and finding 2** — `ring` (via `rustls`'s
   `ring` provider) with `cc`/`find-msvc-tools`/`shlex` in `Cargo.lock`, landed with no CI-matrix change and
   no packaging/prerequisite story. This is the checklist's "major if silent" case, and it is silent.
   License-wise there is no new problem: every added crate's licence is already in `deny.toml`'s allow list,
   and `[bans] multiple-versions = "warn"` absorbs the new `windows-sys 0.52`/`0.61` split, so the scheduled
   `advisories.yml` job degrades to a warning rather than a red.
7. **Tests run where they claim.** **Finding 3** — the new suites assert a no-outbound invariant the shipped
   startup path now breaks. The `#[cfg(windows)]` `%APPDATA%` leg and the `#[cfg(unix)]` owner-mode legs in
   `tests/data_layout.rs:315`/`:345` are guarded rather than skipped-everywhere, matching Test strategy #16;
   the `#[ignore]`d caching suites (`caching_policy.rs`, `cache_document.rs`, `upstream_client.rs`,
   `upstream_endpoint_gate.rs`, and `candidates_join`'s mode-pair leg) carry per-leg reasons and are the
   A15 retirements, with file deletion left to the runner — accepted.

## Findings

### 1. Blocker — the new C/asm dependency has no CI-matrix change, and the shipped Windows compile leg now owes it a target C toolchain

- Location: `apps/registry/Cargo.toml:16-26` (added `rustls` with the `ring` provider + `webpki-roots`),
  `Cargo.lock` (adds `ring 0.17.14` with build-script deps `cc 1.11.3`, `find-msvc-tools`, `shlex`),
  against `.github/workflows/check.yml:29-32` (unchanged: `rustup target add x86_64-pc-windows-msvc` then
  `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`).
- Problem: `ring` compiles C and assembly for the *target* through a build script, so a `cargo check` for
  `x86_64-pc-windows-msvc` from `ubuntu-latest` has to run that build script with no MSVC C toolchain on the
  runner. The proposal says exactly this and prices it — §Options ("`ring` compiles C and assembly for the
  *target* through a build script — not the 'pure-Rust' stack this section claimed — so Rollout 3 names its
  price: the shipped whole-workspace `x86_64-pc-windows-msvc` compile leg (A16) drops `apps/registry`, which
  is compile-checked on the host instead"), §Risks row 1, and the §Compatibility matrix's Windows row
  ("Rollout 3 keeps `apps/registry` off that leg for `ring`'s target build script"). Rollout 3's dependency
  half landed in this branch; its `check.yml` half did not — `check.yml` is not in the diff at all. Concrete
  failure: the `rust` job's first step fails, so every PR in the repo goes red on a step that has nothing to
  do with it, and the Windows row's compile coverage silently stops existing. The gate record the runner
  captured covers `npm run check`, `cargo fmt`, `cargo clippy` and `cargo test --workspace --locked` on the
  host target only; no cross-target leg is in it, so the property "the Windows compile leg still runs" cannot
  be established from the evidence supplied, and the design itself predicts it cannot.
- Fix: land Rollout 3's leg split beside the dependency — narrow the cross leg to the crates that stay C-free
  (`cargo check -p bindforge-core -p discovery --all-targets --target x86_64-pc-windows-msvc`, plus whatever
  else is on that leg today) and add a host-target `cargo check -p bindforge-registry` step, or add
  `--exclude bindforge-registry` to the existing step and a host step for it. Re-run the leg in CI and put
  its result in the gate record so the Windows row's claim is evidenced.

### 2. Major — no packaging/prerequisite story for the C toolchain the workspace now needs

- Location: `README.md:13-25` (unchanged: "the desktop-era Tauri system dependencies (**the cargo gate itself
  needs none of them**)", then the three cargo gates), with `apps/registry/Cargo.toml:21-26` as the cause.
- Problem: the checklist's item-6 packaging story is the second half of Rollout 3 (`README.md`'s
  "prerequisite line that names a C toolchain (A16)"), and the §Compatibility matrix's Linux row states it as
  the hosting target's own requirement ("needs a C toolchain for `ring`'s build script (A16, `README.md`
  prerequisite per Rollout 3)"). `README.md` is not in the diff, so a contributor — or an operator building
  the hosted binary the ADR's Rollout 7 deploys — follows a document that explicitly promises the cargo gate
  needs no system packages, and `cargo test --workspace --locked` dies inside ring's build script. A16 was
  recorded as a *verified* assumption precisely so this line would have to change.
- Fix: add the prerequisite the matrix already promises (a C compiler for the host target, e.g. naming
  `build-essential`/`clang` as the one apt package the cargo gate *does* need now) and correct the "needs none
  of them" parenthetical.

### 3. Major — startup resolves the pinned store authority before it looks at the snapshot, so every resolver-active test run reaches a public hostname

- Location: `apps/registry/src/main.rs:304-311` (`let built = self.store(key)?;` before
  `load_or_pull_snapshot`) and `apps/registry/src/main.rs:338-354`
  (`(steam::STORE_AUTHORITY, 443).to_socket_addrs()`, i.e. a getaddrinfo of `partner.steam-api.com`).
- Problem: Test strategy #17 pins "nothing resolves `partner.steam-api.com` from a test", and three of the new
  suites state that as their own guarantee — `tests/data_layout.rs:33-36` ("no test dials or resolves a
  non-loopback host (Test strategy #17)"), `tests/cli_end_to_end.rs:41-43`, `tests/hosted_startup.rs:14-16`.
  The harness makes the violation automatic: `tests/common/mod.rs:503-510` adds `--no-steam` only for a leg
  that spells *neither* `--no-steam` *nor* the key, so every leg that configures the key (with a planted,
  loadable snapshot, precisely so that nothing is pulled) runs the resolver-active path and therefore the
  resolver — `tests/data_layout.rs:131`, `tests/hosted_end_to_end.rs:138`, `tests/cli_end_to_end.rs:138`,
  `tests/hosted_lookup_policy.rs:94`, `tests/hosted_health_stats.rs:76`, `tests/issued_entries.rs:82`,
  `tests/mint_rule.rs:128`, `tests/retired_ids_and_curation.rs:91`, `tests/health_route.rs:95`. Concrete
  failure: in any environment where DNS for that name fails or is firewalled (an offline developer box, a
  network-policy-sandboxed runner, the kind of sealed CI this repo's no-outbound rule exists for),
  `cargo test --workspace --locked` refuses those instances with
  `the steam store authority 'partner.steam-api.com' could not be resolved` at startup — nine suites red with
  nothing about the change under test. The same line is an operator-facing cost on the supported Linux row: a
  hosted instance with a perfectly good snapshot on disk cannot boot on a host with a broken resolver, even
  though it needs the resolver for nothing at that moment.
- Fix: keep the authority as a *name* in `SteamStore` and resolve it where it is used — inside
  `SteamStore::connect` (`apps/registry/src/steam.rs:626-659`) at pull time — so a run that loads a loadable
  snapshot never consults the resolver and `main.rs` never calls `to_socket_addrs`. The compile-time pin stays
  the production value, and `tests/steam.rs`'s two-name seam is unaffected because it already injects the
  connect address. Failing that, resolve lazily behind a "do I actually have to pull?" decision, but the
  suites' stated invariant is cleanest with the first.

### 4. Minor — the dependency floors the design names as the RUSTSEC mitigation are not the floors in the manifest

- Location: `apps/registry/Cargo.toml:21` (`rustls = { version = "0.23", … }`) and `:26`
  (`webpki-roots = "1.0"`).
- Problem: §Options requires these "at named floors rather than 'pinned versions': `rustls ≥ 0.23.46`
  (RUSTSEC-2026-0346 …), `rustls-webpki ≥ 0.103.16` / `≥ 0.104.0-alpha.8`, and `webpki-roots ≥ 1.0.10` …", and
  §Rollout 3 lands the deps "*at the floors §Options names*". Today's `Cargo.lock` happens to satisfy them
  (`rustls 0.23.46`, `rustls-webpki 0.103.17`, `webpki-roots 1.0.12`), but nothing in the manifest says so:
  the requirement strings admit `rustls 0.23.0` and `webpki-roots 1.0.0`, so any re-resolve
  (`cargo update`, a lockfile refresh after an unrelated bump, a downstream vendored build) can land the exact
  versions the two advisories describe. The repo's only advisory control is `advisories.yml`, which is
  scheduled weekly and "triage input, not a PR gate", so it cannot refuse such a resolve — which is why
  Rollout 3 asked for the screen "beside them".
- Fix: spell the floors in the requirement strings (`rustls = "0.23.46"`, `webpki-roots = "1.0.10"`, plus a
  `rustls-webpki = "0.103.16"` entry or an explicit comment naming why the transitive floor matters), and note
  in the same comment which advisory each floor closes so the next bump cannot silently drop below it.

### 5. Minor — the store address is the resolver's first answer only, with no fallback across the rest

- Location: `apps/registry/src/main.rs:339-353` (`.next()` on the `to_socket_addrs` iterator, then that one
  `SocketAddr` is the store's whole address, `apps/registry/src/steam.rs:455-479`).
- Problem: on the supported Linux hosting row, a host whose resolver returns an IPv6 address first and whose
  IPv6 route is broken gets a connect timeout on every page attempt (`STEAM_CONNECT_TIMEOUT`,
  `STEAM_PAGE_ATTEMPTS` × `STEAM_PAGE_RETRY_SECONDS`), so a *startup* full pull fails and the instance refuses
  to bind — even though the same resolver also returned a working v4 address. Nothing about the shape of the
  answer is used, so this is a platform assumption rather than a design decision anyone made.
- Fix: keep the whole resolved list on `SteamStore` and try each address in turn inside the page attempt (the
  way `TcpStream::connect((host, port))` walks it), or fall through the list before calling a page transient
  failure. Fix 3's lazy-resolution change is the natural place to carry this.

### 6. Nit — the `--corpus` spare is a canonical-spelling comparison, which a case-insensitive filesystem can split

- Location: `apps/registry/src/main.rs:284-294` with `:328-333` (`spared_document` =
  `std::fs::canonicalize`, compared with `!=`).
- Problem: the retirement refusal spares "the path `--corpus` names" by comparing canonical spellings as
  strings. On the §Compatibility matrix's Windows row, `--corpus %APPDATA%\BINDFORGE\REGISTRY\Catalog-V1.JSON`
  is the same file as the refused `<data>/bindforge/registry/catalog-v1.json`, and Rust's `canonicalize` does
  not fold the case of long path components — the operator gets "delete it or point `--corpus` at it`" for
  having already done what the line asks. Linux hosting is unaffected (case-sensitive, byte compare).
- Fix: compare by file identity where it can be had (`metadata` dev/ino on unix, `GetFileInformationByHandle`
  via a documented no-op elsewhere), or as the cheap version compare `as_os_str().to_ascii_lowercase()` on the
  canonical spellings when `cfg!(windows)`.

## Missing evidence, recorded explicitly

- The cross-target compile leg is not in the runner's gate record, so finding 1's failure is reasoned from the
  design's own A16/Rollout-3 text and the unchanged `check.yml`, not from a captured red. That is why it is
  blocking: the required property (the compatibility matrix's Windows row still has its leg) cannot be
  established from what was supplied, and the proposal states the condition under which it fails.
- The quoted gate tail (`test result: ok. 0 passed; 0 failed; 0 ignored;`) is one leg's summary from the
  renderer, not the run's total; the `cargo test --workspace --locked` exit 0 is what stands for the suites.
  The `#[ignore]`d retirement legs do not count toward it — noted, not a defect: their reasons are on the
  legs and A15 lists them.

## FYI (no severity, not this seat's call)

- `apps/registry/src/main.rs:357-399` deviates from §The steam snapshot resolver's "load the snapshot if
  present, loadable, **and fresh**, else pull in full" by loading a loadable-but-stale snapshot; combined with
  `apps/registry/src/main.rs:443-470` sleeping the full `STEAM_SNAPSHOT_REFRESH_SECONDS` first, a boot on a
  nearly-age-bound snapshot can leave steam misses on the named `503` for up to a day — which the matrix's
  client row reads as user-visible availability, since a bound CLI fails the whole scan closed. Flagging for
  the correctness seat; its own comment points at a build-notes §Deviations entry.
- `apps/registry/src/cache.rs`, `src/upstream.rs` and `src/endpoint.rs` are still in the tree but no longer
  declared by `main.rs`, so their `echo_safe`/`percent_encode` copies are dead; and `apps/registry/src/steam.rs:58-67`
  still explains the budget family as living there because "the binary declares its own copy of `constants`",
  which `main.rs:36` contradicts now.
- `store_lock::process_is_live` (`apps/registry/src/store_lock.rs:169-190`) answers "live" for every pid off
  Linux, so on the Windows/macOS rows a killed run's `write.lock` blocks startup until the file is deleted by
  hand. That is the fail-closed direction the runbook names, but it is worth one runbook sentence that the
  auto-adoption is Linux-only.
