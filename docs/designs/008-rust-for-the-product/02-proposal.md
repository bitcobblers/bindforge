# Design: ADR-008 — Rust product workspace, formats contract, and gates

## Context

ADR-008 ([docs/adrs/008-rust-for-the-product.md](../../adrs/008-rust-for-the-product.md)) fixes Rust as the only product language (crates under `crates/`, apps under `apps/`), keeps TypeScript in `tools/orchestration`, makes `schemas/` the public contract, and adds a cargo gate mirroring `npm run check`. This design turns those four clauses into an actual workspace: files, package names, toolchain floor, CI gates, the conformance-fixture layout, and the TypeScript boundary the research flags as the sharpest open question ([01-research.md](./01-research.md)). The repo is greenfield for product code: no `crates/`, no `Cargo.*`, no `schemas/`, no CI config exist yet (A23). BindForge reads and writes binding *files* only (ADR-008: "files remain the product"); it never reads or injects input events, so no X11/Wayland/Raw Input/HID API is targeted — a future component that captures input must name its API per platform in its own compatibility matrix.

## Goals / Non-goals

- Goals:
  - A cargo workspace whose members match ADR-008's layout, compiling green behind one cargo gate.
  - A toolchain floor and pin so every contributor and CI job builds identically.
  - A `schemas/` contract tree plus a conformance harness that runs every golden fixture against `crates/core`.
  - The TypeScript-boundary rule stated once, precisely (see Chosen design, "TypeScript boundary").
  - CI running both toolchains' gates, and a Rust section in `repo-practices`.
  - Chain resolution hardened against adversarial profile files — the precondition for the hub validating uploads with the same compiled types.
- Non-goals:
  - The model's semantics (taxonomy, reach tables, chains, policies, manifest contents) — owned by ADR-009.
  - Codec, scanner, and CLI implementations — owned by ADR-012 (planned).
  - Hub schema, migrations, upload policy, auth — owned by ADR-014 (planned).
  - Desktop webview UI framework, its npm toolchain, packaging pipelines — owned by the desktop design.
  - Any change to `tools/orchestration`.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | clap 4.6.7 current, MSRV 1.74, maintained under WG-CLI | verified | https://docs.rs/clap/latest/clap/ |
| A2 | axum 0.8.x ships, MSRV 1.80, breaking 0.9 in dev, consumes `tower` | verified | https://github.com/tokio-rs/axum |
| A3 | sqlx at 0.9; canonical repo now `transact-rs/sqlx` | verified | https://github.com/launchbadge/sqlx |
| A4 | `sqlx::query!` needs `DATABASE_URL` at compile time to a same-schema DB | verified | https://github.com/transact-rs/sqlx/blob/main/README.md ("Compile-time verification") |
| A5 | `cargo sqlx prepare --workspace` caches query metadata in a root `.sqlx/`; committing it removes the build-time DB need; `--check` verifies cache freshness in CI; `SQLX_OFFLINE=true` forces offline builds | verified | https://github.com/transact-rs/sqlx/blob/main/sqlx-cli/README.md |
| A7 | Cargo workspaces: shared lock/target, `cargo check --workspace`, `members` globs, `[workspace.package]`/`[workspace.dependencies]`/`[workspace.lints]` inheritance | verified | https://doc.rust-lang.org/cargo/reference/workspaces.html |
| A8 | `cargo test --workspace` runs unit + integration + doc tests of every member; `--locked` is deterministic; test binaries run with the package root as cwd | verified | https://doc.rust-lang.org/cargo/commands/cargo-test.html |
| A9 | clippy is the standard lint gate (`correctness` denies by default); `cargo fmt` is the rustfmt front end | verified | https://doc.rust-lang.org/clippy/ |
| A10 | Edition 2024 shipped in Rust 1.85.0 | verified | https://doc.rust-lang.org/edition-guide/rust-2024/index.html |
| A11 | `rust-toolchain.toml` pins the rustup toolchain for the checkout; checkable into source control together with `Cargo.lock` | verified | https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file |
| A12 | A user package named `core` collides with Rust's implicit `core` name and causes confusing import errors; upstream has an open issue asking for a warning | verified | https://github.com/rust-lang/cargo/issues/7760 |
| A13 | Tauri 2 is stable and current (2.12 line); renders with the OS webview, no bundled browser engine | verified | https://v2.tauri.app/start/ |
| A14 | Tauri capabilities grant per-window webview permissions but do not protect against malicious or insecure Rust core code | verified | https://v2.tauri.app/security/capabilities/ |
| A15 | Node is a prerequisite only if a JavaScript frontend framework is used; Rust via rustup is always required | verified | https://v2.tauri.app/start/prerequisites/ |
| A16 | Linux dev needs `libwebkit2gtk-4.1-dev` and friends; WebView2 is preinstalled on Windows 10 (1803+) and 11 | verified | https://v2.tauri.app/start/prerequisites/ |
| A19 | Proton prefixes live at `steamapps/compatdata/<appid>/pfx` per Steam library | verified | https://github.com/ValveSoftware/Proton/wiki/Proton-FAQ |
| A20 | TOML v1.1.0 is current, with implementations in 40+ languages | verified | https://toml.io/en/ |
| A21 | `toml_edit` 0.25.x (spec 1.1.0) preserves comments, spacing, and order; does not preserve dotted-key order or trailing newlines | verified | https://docs.rs/toml_edit/latest/toml_edit/ |
| A22 | The JSON Schema Test Suite model — language-neutral golden fixtures run by every implementer — is a working conformance pattern | verified | https://github.com/json-schema-org/JSON-Schema-Test-Suite |
| A23 | Repo state at merge: no `crates/`, no cargo files, no `schemas/`, no CI config; the only gate is `npm run check` in `tools/orchestration` | verified | repo listing 2026-11-11; `tools/orchestration/package.json:23` (its `"license": "MIT"` vs the Apache-2.0 `LICENSE` file is a repo fix outside this design) |
| A24 | Tauri v2's numeric MSRV is not published on its prerequisites page; assumed ≤ 1.85 | unverified | https://v2.tauri.app/start/prerequisites/ |
| A25 | No direct or transitive dependency at merge needs rustc newer than 1.85 | unverified | Cargo.lock review at merge; drift handled in Risks |
| A26 | `env!("CARGO_MANIFEST_DIR")` gives a crate's package root, letting tests reach repo-root `schemas/` regardless of cwd | verified | https://doc.rust-lang.org/cargo/reference/environment-variables.html |
| A27 | Tauri's own repository advisory list carries v2 advisories that the crates.io/RustSec OSV data does not: GHSA-w28w-mhc8-qvjv (high, CVSS 7.6, no CVE, published 2026-09-26) covers `tauri >= 2.0.0, <= 2.11.5`, patched only at 2.11.6, and the crates.io OSV entry for `tauri` fixes at 2.11.1 (GHSA-7gmj-67g7-phm9) with no entry at 2.11.6 | verified | https://github.com/tauri-apps/tauri/security/advisories/GHSA-w28w-mhc8-qvjv; OSV API query for `crates.io`/`tauri`, 2026-11-11 |
| A28 | `tools/orchestration` already declares a Node floor of `>=22.23.4`; the repo builds and checks under it today | verified | `tools/orchestration/package.json` `engines.node`, repo listing 2026-11-11 |

## Options considered

**Crate package names: bare (`core`, `transforms`) vs `bindforge-` prefixed.** ADR-008 fixes the directories, not the package names. Bare `core` is a known footgun: `core` is an implicit name in Rust name resolution, so a user package of that name shadows it and produces baffling import errors (A12). Rejected: bare names. Chosen: directories stay `crates/core` etc.; package names get the `bindforge-` prefix.

**Workspace floor: edition 2021 at Rust 1.80 vs edition 2024 at Rust 1.85.** The declared dependency floors are clap 1.74 and axum 1.80 (A1, A2), so both options build today. Rejected: 2021/1.80 — rustup auto-installs the pinned toolchain (A11), making the floor nearly free for contributors, so the only durable cost is locking out edition 2024 (which requires 1.85, A10) forever. Chosen: 2024/1.85, pending A24/A25.

**Hub compile gate: Postgres service in the main gate, ban `query!` macros, or committed `.sqlx` offline cache.** A Postgres service in the main job couples every workspace build to a database (A4) and slows CI; banning the macros surrenders the compile-time SQL checking that made sqlx the pick. Chosen: sqlx offline mode — `cargo sqlx prepare --workspace` committed at the workspace root; the main gate needs no DB, and a separate hub-era CI job keeps the cache honest with `--check` (A5); the mechanism itself is ADR-014's, listed in Deferred.

**TypeScript boundary: literal ban, frontend-scoped carve-out, or broad UI code in TS.** A literal ban forces a no-framework vanilla frontend, undercutting the "webview reuse and design velocity" rationale the ADR itself gives for Tauri. Broad TS lets product logic leak into the webview and re-creates the polyglot conformance tax the ADR exists to remove. Chosen: the carve-out in "TypeScript boundary" below.

## Chosen design

### Workspace shape

Root `Cargo.toml` (new):

```toml
[workspace]
resolver = "3"                    # a virtual workspace has no edition to infer it from (A7)
members = ["crates/*", "apps/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "Apache-2.0"    # matches the LICENSE file at the repo root

[workspace.dependencies]
bindforge-core = { path = "crates/core" }
serde = { version = "1", features = ["derive"] }
# clap, toml_edit (first consumer: ADR-012), axum, sqlx (A2, A3): enter when their consumer lands
```

Members, package names, and when each is created (the `crates/*` and `apps/*` globs follow A7's flat-members recommendation; because the globs match one path segment, the desktop era adds the explicit member path `apps/desktop/src-tauri`):

| Directory (fixed by ADR-008) | Package name | Owns | Created in |
|---|---|---|---|
| `crates/core` | `bindforge-core` | model types, taxonomy, reach-table loader, chain resolution — reference implementation of `schemas/` | this design; contents per ADR-009 |
| `crates/transforms` | `bindforge-transforms` | policy stacks | ADR-009 |
| `crates/adapters` | `bindforge-adapters` | codec families, config path discovery, write policy | ADR-012 (planned) |
| `crates/scanner` | `bindforge-scanner` | installed-game detection, config fingerprinting (`compatdata` enumeration per A19) | ADR-012 (planned) |
| `apps/cli` | `bindforge-cli`, binary `bindforge` | clap front end (A1) | ADR-012 (planned) |
| `apps/desktop` | `src-tauri` package `bindforge-desktop` | Tauri v2 app (A13) | hub era, desktop design |
| `apps/hub` | `bindforge-hub` | axum + sqlx + Postgres service (A2, A3) | hub era, ADR-014 (planned) |

Dependency edges point one way: `bindforge-core` depends on nothing; `transforms`, `adapters`, `scanner` → `core` (and `scanner` → `adapters`); apps → the crates they drive; never app → app. Workspace members inherit edition, `rust-version`, and dependency versions (A7), so the floor is declared exactly once.

Also new at the root: `rust-toolchain.toml` with `[toolchain] channel = "1.85.0"` — rustup installs it automatically in any checkout (A11); the channel equals `rust-version` and is bumped only when a reviewed dependency upgrade or a published toolchain security advisory requires it. `.gitignore` gains `/target`; `Cargo.lock` is committed and CI uses `--locked` (A8, A11).

### Gates (ADR-008 clause 4)

The cargo equivalents of `npm run check` (A9, A23), documented in `repo-practices`:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
```

CI is new: the repo has no CI config yet (A23). `.github/workflows/check.yml` runs two independent jobs on every PR — `rust` (the three gate commands above, with nothing installed beyond the pinned toolchain (A11)) and `orchestration` (`npm ci && npm run check` in `tools/orchestration`, whose `setup-node` `node-version` is pinned to `22.23.4` — the Node floor, i.e. the version `tools/orchestration` already builds under, taken verbatim from its `engines.node` entry (A28); this is the only Node floor this design sets, and the `repo-practices` Rust section documents that same value) — satisfying the ADR's "CI must run both". Per-PR dependency-advisory scanning is deliberately excluded from the gate: it needs a tool outside the pinned toolchain and an advisory database that is live network state, so it can turn an unchanged PR red on any day the database moves. That obligation is not deferred — this design is the document that creates the repo's CI, so it commits to the only recurring look at the lockfile: a second workflow, `.github/workflows/advisories.yml` (Rollout 4), runs weekly and on manual dispatch, never as a PR check, installing `cargo-deny` at a pinned version and running `cargo deny check advisories` against the committed `Cargo.lock` and a root `deny.toml`; the repo maintainer triages what it reports within the week, and a `deny.toml` `skip` names the advisory id it waives. That same run is the trigger for the `rust-toolchain.toml` security-advisory bump (Workspace shape); its crates.io/RustSec data is not every dependency's only advisory source (A27, TypeScript boundary). Both workflows declare `permissions: contents: read` at the job level and pin every third-party action to a commit SHA. No job needs Postgres; the sqlx offline-cache mechanism decided in Options considered arrives with `apps/hub` under ADR-014 (see Deferred).

### Formats contract: `schemas/` and the conformance harness (clause 3)

```
schemas/
  README.md                 # versioning rules below (single home for them)
  profile/v1/spec.md        fixtures/valid/  fixtures/invalid/
  patch/v1/spec.md          fixtures/valid/  fixtures/invalid/
  manifest/v1/spec.md       fixtures/valid/  fixtures/invalid/
  policy/v1/spec.md         fixtures/valid/  fixtures/invalid/
  reach-classes/v1/         tables as TOML data + fixtures/valid/  fixtures/invalid/
```

Versioning rule (canonical, mirrored in `schemas/README.md`): one directory per major version (`v1`, `v2`, …); a document that declares no version, or a major `crates/core` does not support, is rejected with a structured error naming the supported majors; minor bumps are additive only; a file's presence in `vN/fixtures/valid/` is the definition of "supported", and the majors `bindforge-core` declares must equal the majors present under `schemas/*/v*/` (Test strategy #2); fixture file names are lowercase ASCII, `-`-separated, and case-unique within a fixture directory. Formats are TOML (ADR-009) over TOML v1.1.0 (A20); the fixtures are the conformance currency — no machine-checkable schema language is committed (see Deferred), following the JSON-Schema-Test-Suite precedent (A22).

Harness: `crates/core/tests/conformance.rs` resolves the fixture root via `env!("CARGO_MANIFEST_DIR")` (A8, A26), walks every `schemas/*/v*/fixtures/{valid,invalid}` directory, and asserts: every valid fixture parses and re-serializes to a semantically equal value; every invalid fixture is rejected; and a `schemas/*/v*/` directory yielding zero valid or zero invalid fixtures fails the run. Reach-class table fixtures are asserted by load + distinctness (Test strategy #4), not re-serialization. The scaffold ships fixtures for `profile/v1` and `reach-classes/v1` only; the `patch`, `manifest`, and `policy` directories arrive with ADR-009 and the same walk picks them up with no harness change.

Adversarial guard (enables the ADR's "hub validates uploads with the same compiled types" consequence): profile, patch, policy, and manifest documents arriving from disk, from another user, or from the network are untrusted input at every entry point (CLI, desktop, hub); every guard in this section applies before any of them can reach a writer, and the invalid-fixture corpus is the contract for that behavior. `bindforge-core`'s `resolve_chain` returns typed errors `Cycle` (a chain deriving from itself) and `ChainTooDeep` (chain longer than `MAX_CHAIN_DEPTH = 32`) instead of recursing, and parsing rejects documents larger than 1 MiB or nested deeper than 16 table/array levels, with typed errors; the constants and the budget are documented in `schemas/README.md` so third-party readers can conform. This design decides that input budget, the two error variant names, and the depth bound — constraints ADR-009 must adopt, with 32 an initial value it may tune but not remove; all other chain semantics are ADR-009's.

### TypeScript boundary (clause 1; canonical)

Product behavior is Rust only. Outside `tools/orchestration`, JS/TS source may live in exactly one place: `apps/desktop`'s webview frontend, which holds presentation only — everything it triggers is behavior in a workspace crate, reached through Tauri commands gated by capabilities under `apps/desktop/src-tauri/capabilities/` (A14); capability files enumerate only the app's own commands, and the `fs`, `shell`, and `dialog` plugins are not enabled — file access happens in workspace crates, never through plugin scopes. Capability-based isolation assumes a patched framework, and this design carries no patch number for it: the desktop design pins the newest `tauri` release available when it lands in `[workspace.dependencies]` (never below 2.11.1) and checks Tauri's own repository advisory list — the crates.io/RustSec data the scheduled `cargo deny` scan reads does not carry every v2 advisory (A27). Node is a prerequisite only if that frontend uses a JavaScript framework (A15); the framework, bundler, npm config, and any Node floor they bring are the desktop design's choice (deferred). The desktop-era `src-tauri` package's exact `members` path is likewise the desktop design's call.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Pre-1.0 churn (axum 0.9, sqlx org move — A2, A3) breaks dependency upgrades | Med | Low | Committed `Cargo.lock` and CI `--locked` (A8); upgrades are reviewed PRs, never `cargo update` drive-bys |
| Adapter file-write bugs are unmitigated by Tauri capabilities (A14) | Low | High | Codec conformance fixtures pin `toml_edit`'s known non-preservation gaps (A21) when `crates/adapters` lands (ADR-012 takes the obligation) |
| MSRV drift: a transitive dep demands rustc > 1.85 (A25) | Med | Low | Pinned lock makes drift explicit at upgrade PRs, not silent; floor bump is a deliberate workspace change updating `rust-version` and `rust-toolchain.toml` together |
| A contributor creates a workspace package colliding with a sysroot name (A12) | Low | Low | Naming rule in the `repo-practices` Rust section: workspace packages are `bindforge-`-prefixed |
| Contributor friction from second toolchain / missing Linux system deps (A16) | Med | Low | `rust-toolchain.toml` auto-pin (A11); README and `repo-practices` list rustup and the Tauri system package list |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Windows 10 (1803+) / 11 — desktop era | yes (target) | packaging pipeline details (installer, WebView2 bootstrapper) owned by the desktop design |
| Linux (glibc distros; artifacts deb + AppImage) | yes | dev deps per Tauri list (A16) |
| Steam Deck / SteamOS | no (v1) | no packaged artifact targets the read-only image; scanner work assumes standard prefixes (A19) |
| Flatpak, Android / iOS | no (v1) | sandbox-vs-scanner and mobile targets deferred, see Deferred / out of scope |
| macOS | no (v1) | no macOS support committed for v1; the desktop design may revisit |
| CI runner (Linux) | yes | both gates; cargo gate needs no Postgres or `DATABASE_URL` |
| Postgres (hub runtime, hub era) | yes (target) | compile-time path per the Options sqlx decision; server version pinned and tested by ADR-014's strategy, see Deferred |
| Contributor dev env | Linux + Windows yes; macOS no (v1) | rustup pin auto-installs (A11); Tauri prereqs cover Linux/Windows (A16) |

## Test strategy

1. Unit — conformance harness contract: for every directory under `schemas/*/v*/fixtures/valid/`, `bindforge-core` parses the fixture and re-serializes to a semantically equal value; for every `fixtures/invalid/` fixture, it rejects with a structured error; a `schemas/*/v*/` directory yielding zero valid or zero invalid fixtures fails.
2. Unit — version gate: a document declaring an unsupported major version, or declaring no version at all, is rejected with an error naming the supported majors (missing-version fixture under `schemas/profile/v1/fixtures/invalid/`; rule in `schemas/README.md`); the majors `bindforge-core` declares equal the majors present under `schemas/*/v*/`.
3. Unit — chain guard: `resolve_chain` rejects a self-deriving chain (`Cycle`), accepts a chain of exactly `MAX_CHAIN_DEPTH` links, and rejects one link deeper (`ChainTooDeep`) — boundary-tested with fixtures under `schemas/profile/v1/fixtures/invalid/`.
4. Unit — reach-table loader: tables load from `schemas/reach-classes/v1/` fixture data into the in-memory model with per-hand (left/right) tables kept distinct.
5. Integration — one-gate promise: from a fresh clone with no `DATABASE_URL` set and no Postgres reachable, every command listed in the Gates block succeeds on the Linux CI runner, with nothing installed beyond the pinned toolchain.
6. Integration — gate independence: a PR that breaks only TS fails `orchestration` while `rust` passes; a PR that breaks only Rust fails `rust` while `orchestration` passes.
7. Platform: the cargo gate must be green on the Linux CI runner; the gate commands and the `rust-toolchain.toml` auto-install are verified once on Linux and once on a Windows contributor machine (Rollout 7), which is where Windows gate greenness is established; the desktop packaging matrix is exercised only when the desktop design lands.

## Rollout

1. Add root `Cargo.toml`, `rust-toolchain.toml` (contents above); extend `.gitignore` with `/target`; commit `Cargo.lock`. No migrations or defaults change — the product tree does not exist yet (A23).
2. Create the `crates/core` package skeleton with the naming rule applied; its public API and model content follow ADR-009; unit tests live in each crate's `tests/`, mirroring `src/` per the ADR-004 convention.
3. Create the `schemas/` tree with `README.md` (versioning rule) plus `profile/v1` and `reach-classes/v1` with one valid+invalid fixture pair each; add `crates/core/tests/conformance.rs`.
4. Add `.github/workflows/check.yml` with the `rust` and `orchestration` jobs (action SHAs pinned, `permissions: contents: read`, `setup-node` `node-version` at the Node floor from Gates), plus `.github/workflows/advisories.yml` and the root `deny.toml` (Gates).
5. Add the Rust section to `.opencode/skills/repo-practices/SKILL.md`: workspace package shape and naming rule, the cargo gate commands, the Node floor (the `engines.node` value cited in Gates), the capability-file rule (own commands only; no `fs`/`shell`/`dialog` plugins), and tests-under-each-crate.
6. Update `README.md` with the contributor prerequisite: rustup one-liner and the Linux system-deps list (A16).
7. Verify Test strategy #5 on a fresh Linux clone and the Windows leg of Test strategy #7 on a Windows contributor machine before announcing to contributors.

Later, with the hub era (ADR-014): `apps/hub` follows the sqlx offline-cache decision (Options considered); no change to the other gates.

## Deferred / out of scope

- Taxonomy ids, reach-table content, chain semantics other than the depth guard and its boundary fixtures, patch/policy/manifest semantics and their fixture cases, and manifest- or scanner-derived fields that identify the user or host — owned by ADR-009 (`docs/adrs/009-canonical-profile-and-manifest-model.md`).
- Codec implementations, config path discovery, write policy, and conformance obligations for third-party game formats (INI/TOML/UTF-16 round-trips), including path canonicalization and rejection of `..`/absolute paths arriving from untrusted profile data, no interpolation of paths into shell strings, and path redaction when fingerprinting — owned by ADR-012 (planned).
- Seed preset policy set — owned by ADR-011 (planned).
- Hub DB schema, migrations, upload policy, auth, voting, plus the sqlx offline-cache mechanism and the hub-path `cargo sqlx prepare --check` job — owned by ADR-014 (planned).
- Webview UI framework, `apps/desktop` npm toolchain, its exact workspace-members path, and the desktop-era Node floor that toolchain may need — owned by the desktop design (no number yet).
- Packaging pipeline details: WebView2 bootstrapper detection, Edge-preview-channel CI, deb/AppImage builds — owned by the desktop design (no number yet).
- A machine-checkable schema language for the TOML formats (research leaves this unknown) — no follow-up yet; fixtures remain the contract.
- Flatpak viability, macOS support, Tauri mobile targets — no follow-up yet; excluded for v1 per Compatibility matrix.

## Revision log

- Chosen design → Workspace shape (r1, r2): license set to `Apache-2.0` per the `LICENSE` file (the `package.json` MIT mismatch recorded in A23 as a repo fix); `resolver = "3"` added; `serde` given a version floor; `clap`/`toml_edit` entries, empty lints tables, and the sqlx profile block deleted (first consumers land in ADR-012 / hub era); desktop era noted as needing an explicit `apps/desktop/src-tauri` member; `crates/transforms` creation moved to ADR-009; toolchain bump also allowed for a published toolchain security advisory; the dependency-edge sentence reworded so `bindforge-core` reads as the depends-on-nothing root (r2).
- Chosen design → Gates (r1, r1c3, r2): the per-PR cargo gate is exactly the three `cargo fmt`/`clippy`/`test` commands — advisory scanning stays out of it (unpinned tool outside the rustup toolchain, live advisory DB); the obligation itself stays owned here as the scheduled non-blocking `advisories.yml` job + root `deny.toml`, maintainer-triaged, replacing the unowned r1 deferral (r1c3); CI pins action SHAs, sets `permissions: contents: read`; the Node floor is `22.23.4` — `tools/orchestration`'s `engines.node` value (A28) — and `setup-node` and the `repo-practices` bullet both mirror that one value, while any desktop-era Node floor belongs to the desktop design (r2); the canonical sqlx-rule paragraph, its Risks row, and its test moved to Options considered + Deferred (ADR-014).
- Chosen design → Formats contract (r1): untrusted-input premise stated; 1 MiB / 16-level parse budget added beside the depth bound; versioning rule covers a missing version, majors↔fixture-directory equality (Test #2), and fixture file-name constraints; harness fails on a zero-fixture format directory; scaffold scope narrowed to `profile/v1` + `reach-classes/v1`; reach-classes fixtures given the `valid`/`invalid` layout, asserted by Test #4.
- Chosen design → Adversarial guard (r1): the budget, error variant names, and depth bound are decided here as constraints ADR-009 must adopt (32 an initial value); Deferred narrowed to match.
- Chosen design → TypeScript boundary (r1, r1c3, r2): capability files enumerate only the app's own commands, `fs`/`shell`/`dialog` plugins stay disabled; the desktop design pins the newest `tauri` release available when it lands, with 2.11.1 only as a floor and Tauri's own advisory list checked (A27) — no fixed patch number carried here; Node stays conditional on a JS framework being used (A15) and the desktop-era Node floor is deferred there, so the only Node floor this design states is Gates'.
- Compatibility matrix (r1): macOS and contributor-dev cells set to definite `no (v1)`; Linux row relabeled to glibc distros with deb + AppImage artifacts; Steam Deck/SteamOS row added; Postgres `yes (target)` row names ADR-014's strategy as where it is exercised; Rollout 7 schedules the Windows gate verification (Test #7).
- Options considered / register (r1): Flatpak option entry and the Windows packaging-note detail dropped to Deferred; orphaned assumptions A6, A17, A18 deleted; A2 stated as `0.8.x`; A16 narrowed to the dev-prerequisite claim (the misattributed quote removed).
- Context (r1): stated that BindForge handles binding files only and targets no input-event API.
- Test strategy (r1): renumbered after removing the hub-era `.sqlx` item; #2 adds the missing-version rejection and the majors-equality assertion; #3 pinned to the constant, not the literal 32; #5 cites the Gates commands instead of restating them, so it cannot drift from the gate.

## As-built (2026-11-11)

What the implementation at `HEAD` actually contains, recorded against this
approved design. This note changes no decision above; it records drift, its
reason, and the sections it touches. The implementation-side record, with how
each item walked through review, is [`03-build-notes.md`](./03-build-notes.md).

- **Affected: Chosen design → Workspace shape.**
  - `Cargo.toml` ships `members = ["crates/*"]`, not
    `["crates/*", "apps/*"]`. Reason: a glob matching no member aborts every
    gate command at manifest load, and git tracks no empty `apps/` directory;
    `apps/*` joins with the first app crate (`apps/cli`, ADR-012). The
    proposal's glob stands as the target state.
  - `Cargo.lock` is hand-authored (lock v3, the single path member, no
    external deps — the workspace has none): no stage in the pipeline that
    built this tree had a cargo toolchain available; the first
    dependency-adding PR regenerates/confirms it.
  - `rust-toolchain.toml` lists only `channel = "1.85.0"` — rustfmt and
    clippy ship in the default rustup profile, no `components` list.
- **Affected: Chosen design → Formats contract.**
  - Parsing is an in-tree minimal reader (`crates/core/src/toml.rs`), not an
    external TOML crate (the proposal names none here and reserves
    `toml_edit` for ADR-012), keeping the committed lock dependency-free and
    the pinned toolchain fully hermetic — Test strategy #5's "nothing
    installed beyond the pinned toolchain" taken literally. Consequence:
    the accepted grammar is a **documented subset** of TOML v1.1.0, not all
    of it. Every admitted document is legal TOML; table headers, dotted
    keys, multi-line arrays, floats, and hex integers are refused, each with
    an invalid fixture; `\uXXXX`/`\UXXXXXXXX` escapes are decoded (a
    widening, needed by a serializer that must escape control characters).
    The subset is stated in `schemas/README.md` and both `spec.md` grammar
    sections, so the contract tree — not only the reader — draws the line.
  - Reach tables ship as `left.toml`/`right.toml`, each a one-hand document
    whose `hand` entry must equal its file name (rule exposed as the pure
    function `hand_matches_file_name`; a corpus fixture cannot pin its own
    file name against the loader's expected hand).
  - The harness parses and grammar-checks every corpus fixture and, for a
    reach-classes table, also runs the load half (`load_corpus_table` stands
    the fixture in for its declared hand in a scratch directory); Test
    strategy #4's load + distinctness obligation holds, split between the
    harness and `reach_tables.rs`.
- **Affected: Chosen design → Formats contract, Adversarial guard
  paragraph.** The 1 MiB budget is enforced before the read as well as on
  the text (`parse_file` metadata-checks the length, `parse_reader` takes at
  most `MAX_DOCUMENT_BYTES + 1` bytes), so no reader materializes an
  over-budget document. The 16-level nesting guard is realized as 16 array
  levels plus blanket rejection of everything table-shaped, because the v1
  grammars admit no inline tables or table headers. `Cycle`, `ChainTooDeep`,
  and `MAX_CHAIN_DEPTH` ship exactly as fixed here; the remaining
  `CoreError` variant names (`Malformed`, `NotUtf8`, `MissingEntry`,
  `UnknownFormat`, `InvalidValue`, `DocumentTooLarge`, `NestingTooDeep`,
  `Io`), `resolve_chain`'s `Ok` payload (`Vec<String>`, profile-first), and
  the rejection-text sanitization (one truncating, control-escaping helper;
  IO paths reachable only via `source_path`) are implementation spelling in
  the space this design leaves open.
- **Affected: Chosen design → Gates; Rollout 4.**
  - The scheduled scan runs the whole `cargo deny check` (advisories, bans,
    licenses, sources — every section `deny.toml` configures has a
    consumer), not only `check advisories`, with `--format sarif` in the
    root position the pinned 0.20.2 accepts, a stdout redirect, a `"runs"`
    content guard, and the report uploaded as a `cargo-deny-report`
    artifact (SARIF is the only format 0.20.2 writes to stdout; the JSON
    stream is hard-wired to stderr).
  - cargo-deny 0.20.2 declares MSRV 1.88.0, above the product pin, so the
    job installs its own `1.88.0` `SCAN_TOOLCHAIN` and runs the tool under
    `RUSTUP_TOOLCHAIN`; the product pin stays 1.85.0 — A25's MSRV check
    applied to the scanner itself.
  - The scan job declares `actions: read` beside `contents: read` — one
    line beyond this section's "Both workflows declare `permissions:
    contents: read`". (The round-1c5 correctness review notes the stated
    justification is wrong — the upload works on `contents: read` and the
    scope is inert — flagged for the next touch.)
  - The waiver key is `[advisories] ignore`, not `skip` (`skip` is
    cargo-deny's licenses key); `deny.toml` says so in a comment.
  - All three jobs carry `timeout-minutes`; `check.yml` triggers on
    `pull_request` only — the design's "two independent jobs on every PR".
- **Rollout status.** Rollouts 1–6 landed as scripted (with the shapes
  above). Rollout 7 is **outstanding**: no pipeline stage or review seat has
  been able to run a toolchain, so the three gate commands have never been
  executed against this tree; Test #5's and #7's machine evidence belongs to
  the merge PR's `check.yml` `rust` job, a Windows contributor machine, and
  one manual `workflow_dispatch` — the dated checklist in
  `03-build-notes.md`.
