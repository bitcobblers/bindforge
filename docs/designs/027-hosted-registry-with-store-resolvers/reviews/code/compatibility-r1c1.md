# Code review — compatibility seat, round 1 dissent pass (c1)

- Range under review: `git diff 75cbc433ed8a546734c41d4eb928395373ab73e6..HEAD -- ':(exclude)docs/designs'`
  (21 files: `.github/workflows/check.yml`, `Cargo.lock`, `README.md`, `apps/registry/{Cargo.toml,src/*,tests/*}`,
  `crates/core/tests/gate_contract.rs`).
- Inputs: that diff, `02-proposal.md`, `docs/adrs/027-hosted-registry-with-store-resolvers.md`,
  the runner's gate record (2026-11-22T09:51:31Z, green), my round-1 findings.
- Specialty: compatibility checklist only (`.opencode/skills/code-review/references/compatibility.md`).
- Scope of this pass: my six round-1 findings, checked first; anything else only where the
  changed code makes it true.

## Verdict

**approve.** All four findings that carried blocking or major weight are resolved in code; the
remaining items are one minor coverage claim and two nits. No open blocker, no open major.

## Round-1 findings — disposition

### 1. Blocker — native C dependency landed with no CI-matrix change (Rollout 3's `check.yml` half missing)

**Resolved.** `.github/workflows/check.yml:42-46` narrows the cross-target leg to
`cargo check --workspace --exclude bindforge-registry --all-targets --target x86_64-pc-windows-msvc`
and `:47-53` adds the host leg `cargo check -p bindforge-registry --all-targets` — the two halves
§Options ("the shipped whole-workspace `x86_64-pc-windows-msvc` compile leg (A16) drops
`apps/registry`, which is compile-checked on the host instead") and Rollout 3 prescribe. The package
name is right (`apps/registry/Cargo.toml:2` = `bindforge-registry`), so `--exclude`/`-p` resolve.
The concrete failure I named — the whole cargo gate going red because `ring`'s build script is asked
of a runner with no MSVC C toolchain — is closed: `Cargo.lock:239-250` shows `cc` is a dependency of
`ring` alone, `ring` (`Cargo.lock:253-264`, `:276-284`) is reachable only through `rustls` /
`rustls-webpki`, and those three are dependencies of `bindforge-registry` alone
(`Cargo.lock:60-69`). Excluding that package therefore removes the only C build script from the
Windows leg, and the rest of the workspace is what that leg already built green before this branch.
Rollout 3's other half, the RUSTSEC screen, landed as the `advisory-gate` job
(`check.yml:83-126`), and `crates/core/tests/gate_contract.rs` re-pins both the job set
(`:193-200`) and the leg split (the new `the_windows_compile_leg_skips_the_registry_and_the_host_leg_covers_it`),
so the shape is now a pinned contract rather than prose.

*Evidence still missing, recorded:* the gate record contains no cross-target leg and no
`cargo check -p bindforge-registry` leg (its four legs are `npm run check`, `cargo fmt`,
`cargo clippy`, `cargo test --workspace --locked`), so neither leg's green result is established by
the evidence supplied; the gate contract pins the workflow *text*, not the outcome. I established
the narrow leg's viability by the lockfile-graph read above instead of a run (this seat's shell is
limited to `git diff*`). That does not prevent proceeding: the leg's input set is derived, not
guessed, and Rollout 7's dated platform runs are the repo's existing place for the outcome
(`crates/core/tests/gate_contract.rs:10-12`). Not blocking. The residual coverage the split gives
up is finding C1-1 below.

### 2. Major — no packaging/prerequisite story for the C-toolchain requirement

**Resolved.** `README.md:21-27` is the prerequisite line Rollout 3 names: it says `apps/registry`'s
store pull rides `rustls` with the `ring` provider, that `ring` compiles C and assembly through a
build script, that a C toolchain is therefore a prerequisite of `cargo build`/`cargo test` for the
workspace, and it points at the CI leg that exists for exactly that reason. A contributor or a
Rollout-7 operator reading the README now learns the requirement before the build-script failure.
The stale parenthetical two lines above is finding C1-2.

### 3. Major — startup resolved `partner.steam-api.com` before looking at the snapshot

**Resolved, structurally.** `apps/registry/src/main.rs`'s `store()` is now
`Ok(SteamStore::production(key))` — no `to_socket_addrs` on any startup path. `SteamStore` holds an
`Endpoint` (`steam.rs`: `Pinned(&'static str, u16)` | `Address(SocketAddr)`), and
`SteamStore::dial_addresses` is the only caller of `to_socket_addrs`, reached only from `connect`,
which is reached only from a page attempt. So a run that loads a present, loadable, fresh snapshot
never consults the resolver, and the compile-time pin stays the production value
(`steam.rs:540-542`); `tests/steam.rs`'s separate-verify-name seam
(`for_test` + `with_production_transport`) is untouched.
I checked Test strategy #17's rule from the other side rather than trusting the comment: every
subprocess leg that carries `BINDFORGE_STEAM_WEB_API_KEY` without `--no-steam` plants a snapshot
stamped `fetched_at: now_unix()` — `data_layout.rs:103-115` (planted under the branch each leg
resolves, which is why `data_of` exists), `hosted_end_to_end.rs:82`, `issued_entries.rs:61`,
`mint_rule.rs:56`, `retired_ids_and_curation.rs:55`, `hosted_lookup_policy.rs` (all legs pass
`now_unix()`), `hosted_health_stats.rs:72,152`, `cli_end_to_end.rs:190,262,394` — and the legs that
do not plant run `--no-steam`. No test resolves or dials `partner.steam-api.com`; the nine instances
I named in round 1 are gone. The two postures that genuinely cannot boot a process any more (a
past-bound snapshot, no snapshot) moved to the library face (`bindforge_registry::state`'s
`State::hosted` + `steam::startup_snapshot`) instead of being deleted, which is what keeps #3's cold
cells and #13's stale half pinned without a dial. The remaining `to_socket_addrs` in the crate is
`main.rs:578-587`, the operator-named `--host` bind, unchanged.

### 4. Minor — dependency floors not spelled in the manifest

**Resolved.** `apps/registry/Cargo.toml:22-41` declares `rustls = ">=0.23.46, <0.24"`,
`webpki-roots = ">=1.0.10, <2"` and an explicit `rustls-webpki = ">=0.103.16, <0.104"`, each named
against the advisory it closes (RUSTSEC-2026-0346, -0381/-0382), and the `advisory-gate` job makes
a re-resolution onto an advised version fail the PR — the thing that makes a floor mean something,
given `advisories.yml` is triage input (`advisories.yml:67-70` forces its exit to `|| true`).
`Cargo.lock:254,277,384` satisfies the three floors (0.23.46 / 0.103.17 / 1.0.12), and the new
`rustls-webpki` entry at `Cargo.lock:66` is the only lock change. Pin drift between the two
workflows' scanner pins is not present: `check.yml:106-107` and `advisories.yml:45-46` agree on
`1.88.0` / `0.20.2`, both read the root `deny.toml` (which exists), and both declare
`permissions: contents: read` and a 30-minute budget. The ceiling on the verifier floor is C1-3.

### 5. Minor — the store's address is the resolver's first answer, no fallback

**Resolved.** `SteamStore::connect` now takes `dial_addresses()`' whole list and walks it in order,
keeping the last error kind, exactly as `TcpStream::connect((host, port))` does; the doc comment
names the IPv6-first case. An unreachable first AAAA is one failed dial, not a refused startup.
`Endpoint::Address` short-circuits to one address, so the loopback suites are unaffected.

### 6. Nit — the leftover-document refusal compared canonical spellings as strings

**Resolved.** `main.rs`'s new `same_document` asks for file identity where the platform can answer
it (`(dev(), ino())` under `#[cfg(unix)]`) and falls back to a case-insensitive spelling comparison
elsewhere; the caller is now
`if path.exists() && !same_document(&self.spared_document(&path), &corpus_path)`. Symlinked
`--corpus` spellings spare the file, since `spared_document` canonicalizes first. The fallback
branch's own coverage is C1-1.

## Compatibility checklist

1. **Platform code matches the Compatibility matrix** — mostly yes; one claim outruns the gate
   (C1-1). Platform code is behind explicit guards: `main.rs:380` / `:390`, `persist.rs:179` / `:188`
   and `:203`, `persist.rs:218/227/235/240/413`, and the suites' `cfg(unix)` / `cfg(windows)` legs.
   The increment also *removed* a platform divergence: `store_lock.rs` deleted the
   `#[cfg(target_os = "linux")]` `/proc/<pid>` liveness probe and the off-Linux "assume live"
   branch, so `registry data directory is locked` is now one answer on every platform, matching the
   matrix's Linux/macOS/Windows rows (`store_lock.rs:142-152`).
2. **Node floor** — not applicable: no `tools/orchestration` or JS file in the range.
3. **Paths** — fine. Documents are still built by `Path::join` (`registry_document`,
   `persist::save`); the only string-built URI is the store request target, percent-encoded; the one
   new path *comparison* moved from strings to identity on unix. Case-sensitivity assumption is now
   stated and handled (`same_document`'s doc comment names the Windows long-component case fold).
4. **Guarded platform imports** — not applicable in the JS sense; the Rust analogue holds: no
   platform-only module is reached unguarded.
5. **Locale/layout reality** — fine, and one improvement: `normalize_store_name` is now the
   `char::is_control()` strip followed by the call A7 and Rollout 3 actually name,
   `discovery::game::friendly_name`, replacing this crate's private re-spelling of the whitespace
   collapse and clamp, so a non-ASCII or non-Latin store title is normalized by the shipped rule
   the rest of the framework uses before it reaches the loader's `013-B4` gate.
   `state.rs`'s canonical-appid filter is a single-key-spelling rule, not a locale rule:
   `steam/007` and `steam/+7` are no longer identities, which protects the held-set probe and the
   cap's one-mint-per-key arithmetic.
6. **Native dependencies introduced** — story landed: README prerequisite (`README.md:21-27`), CI
   matrix split (`check.yml:37-53`), advisory screen (`check.yml:83-126`), floors in the manifest
   (`apps/registry/Cargo.toml:22-41`). No longer silent, so not major.
7. **Tests run on the platforms they claim** — #16's Windows half is now pinned as workflow text by
   `gate_contract.rs`'s new leg, and #17's no-outbound rule is enforced by construction (resolution
   lives in `connect` only) plus the fresh-snapshot legs above. #16's `%APPDATA%`/no-op-modes seam
   and the crate's non-unix product code are the coverage C1-1 records.

## Findings

### C1-1 — minor — `apps/registry`'s non-unix code, including a branch this diff added, is now compiled by no leg

- Location: `.github/workflows/check.yml:37-41` and `:47-53` (the leg split and its stated
  coverage), read against `apps/registry/src/main.rs:390-395`, `apps/registry/src/persist.rs:188`
  (and `:218/:227/:235/:240/:413`), `apps/registry/tests/data_layout.rs:320`.
- Problem: the increment that added the `ring` dependency also moved `apps/registry` out of the
  only leg that ever compiled it for a non-unix target, and the new host leg runs on
  `ubuntu-latest`, so it compiles `x86_64-unknown-linux-gnu` — the target the clippy and test legs
  already build. Net effect for this crate: `same_document`'s `#[cfg(not(unix))]` fallback (added by
  this diff, the branch the Windows row of the Compatibility matrix depends on for the
  `--corpus`-exception posture finding 6 fixed), `persist.rs`'s `#[cfg(not(unix))]` owner-only
  no-op branches, and `data_layout.rs`'s `#[cfg(windows)]` `%APPDATA%` leg are compiled by nothing:
  the cross-target leg excludes the package, and no gate ever runs a Windows host. The two comments
  that record the exchange overstate what it buys — `check.yml:40-41`
  ("The crate's Windows posture is the host compile check below plus the DataLayout seam its suites
  run") and `crates/core/tests/gate_contract.rs:262-264,280-284` ("the crate's Windows posture is paid for on the host
  target") — because a Linux host compile verifies no Windows code path, and the seam those lines
  point at is itself no longer compiled. Concrete failure: a type error in a `cfg(windows)` branch
  of `apps/registry` — including one a future contributor adds to `same_document` or
  `owner_only_dir` — passes all three PR jobs, while the same error anywhere else in the workspace
  is caught by the narrowed leg. The Rollout 3 split itself is the design's prescription
  (§Options, §Compatibility matrix's Windows row, Rollout 3), so the split is not the finding;
  the undocumented-and-mis-stated coverage hole beside it is.
- Fix: either pay for it or name it. Paid: add one `windows-latest` step (that runner carries the
  MSVC toolchain `ring`'s build script wants) running
  `cargo check -p bindforge-registry --all-targets`, which restores exactly the coverage the
  Linux cross-target leg gave up and compiles the suite's `%APPDATA%` leg. Named: correct the two
  comments and the matrix's Windows row to say the crate's Windows code has no compile leg in this
  gate and belongs to Rollout 7's dated Windows-contributor run. Cheapest partial payment: make the
  case-folded spelling comparison unconditional (`identity || case_folded_equal`) with the dev/ino
  check added under `#[cfg(unix)]`, so the fallback logic at least compiles and is unit-testable on
  the Linux host.

### C1-2 — nit — `README.md:13-14` still says the cargo gate needs none of the listed system packages

- Location: `README.md:13-14` against `README.md:21-27` (both after this diff).
- Problem: the sentence that introduces the apt list still reads "(the cargo gate itself needs none
  of them)" while the paragraph six lines below says a C toolchain "is a prerequisite of
  `cargo build`/`cargo test` for the workspace" and names `build-essential` from that very list. A16
  recorded the parenthetical as verified precisely so it would change, and my round-1 fix asked for
  it. Impact is now low — the correct claim is adjacent and names the package — so this is a stale
  sentence rather than a missing story.
- Fix: narrow it, e.g. "(the desktop-era Tauri set: the cargo gate needs `build-essential` from it,
  for `ring`; nothing else)".

### C1-3 — nit — the `rustls-webpki` floor's `<0.104` ceiling can stop constraining the verifier it exists to pin

- Location: `apps/registry/Cargo.toml:36-41`.
- Problem: the requirement is named as the load-bearing floor for a crate this one never imports
  ("it is named so no re-resolution can land the transitive verifier below the patched line"). With
  a `< 0.104` ceiling, the day rustls' own requirement moves to `^0.104` — the branch §Options
  explicitly allows ("`rustls-webpki ≥ 0.103.16` / `≥ 0.104.0-alpha.8`") — cargo satisfies the two
  requirements with two copies: rustls verifies against 0.104.x, this manifest's floor constrains an
  unused 0.103.x copy, and the comment's promise goes quietly false while the lockfile still
  resolves and the advisory screen still reads one version per name. No present-day effect
  (`Cargo.lock:277` is a single 0.103.17).
- Fix: drop the ceiling on the transitive floor (`rustls-webpki = ">=0.103.16"`), which keeps one
  unified copy whichever line rustls tracks and keeps the floor on the copy that is actually linked;
  the two named floors keep their `< 0.24` / `< 2` ceilings, since those are direct dependencies this
  crate imports.

## FYI (no severity, out of specialty)

`apps/registry` now compiles `steam.rs`, `state.rs`, `corpus.rs`, `mint.rs`, `retired.rs`, `keys.rs`
and `constants.rs` twice — once into the library face the suites drive, once into the binary through
`main.rs`'s own `mod` declarations — so the cold-posture and mint legs assert against a second
compiled copy of the same source rather than the code the binary runs. Whether that duplication is
acceptable is the correctness/testing seats' call; it is not a platform matter.
