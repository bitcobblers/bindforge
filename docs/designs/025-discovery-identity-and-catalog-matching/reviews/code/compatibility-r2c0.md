# Code review — compatibility seat — round 2, pass c0

- Scope: `git diff 7b036617dfa2d7895888b73c9b9412387b1d7e08...HEAD -- ':(exclude)docs/designs'` (39 files, +5797/−1435).
- Inputs used: that diff, `02-proposal.md`, `docs/adrs/025-discovery-identity-and-catalog-matching.md`, the runner's gate record, this seat's own (empty) previous findings. No review reports or transcripts opened.
- Gate record (2026-11-19T03:30:34Z, green): `npm run check` exit 0; `cargo fmt --all -- --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo test --workspace --locked` exit 0.

## Checklist walk

1. **Platform code vs. the proposal's Compatibility matrix — pass.**
   - Linux: the XDG data-dir rule, the store round-trip, the loopback client and the Steam scan all run on the Linux test leg (`cli_match_store_tests.rs`, `cli_catalog_gate_tests.rs`, `cli_confirmation_flow_tests.rs`).
   - Windows: the `%APPDATA%` branch is data-selected through `StoreLayout` / the env-lookup closure (`apps/discovery-cli/src/matches.rs:43-58,70-86`) and unit-tested on Linux (`the_windows_branch_resolves_appdata_through_the_same_seam`), exactly the "unit-tested through the env seam, not run on Windows" the matrix states; the drive-prefix rule is tested as bytes so a Linux `Path::is_absolute` never decides it.
   - macOS: no macOS-specific code; the non-Windows `StoreLayout::Unix` branch is the same XDG/HOME rule the matrix claims.
   - Flatpak/Snap: store follows `XDG_DATA_HOME`; no extra rule, as stated.
   - `steam` is the only token with a shipped emitter; `epic`/`local` are declared (`Source` enum, `game.rs`) with no emitter and no emission test asserting they ship — matching the two "declared, unpopulated" matrix rows.
2. **Node floor — not applicable.** The diff touches no `tools/orchestration` path; `npm run check` is unchanged and green.
3. **Paths — pass.** Store location is built with `PathBuf::join` (`matches.rs:88-96`); the Steam install path construction is unchanged from the shipped code; the URL is composed by the pure `candidates_url` with per-byte percent-encoding rather than raw concatenation of untrusted bytes (`catalog.rs:170-185`); store records key on `slug` byte order (a documented, case-sensitive byte ordering); echoes of untrusted paths go through `echo_path`/`escape_and_truncate`.
4. **Guarded platform imports — pass.** Every `std::os::unix` use in `matches.rs` (`owner_only_dir`, `owner_only_new_file`, `owner_only`, and the mode-asserting test) sits behind `#[cfg(unix)]` with a plain-std non-unix fallback; `winreg` handling is untouched; `catalog.rs` is std-only (`TcpStream`, `to_socket_addrs`, `connect_timeout`), so nothing platform-locked reaches load on another platform.
5. **Locale/layout reality — pass.** `friendly_name` clamps at a `char` boundary (`game.rs:135-146`) and is pinned against a split `é` (`game_tests.rs`: `friendly_name_clamps_a_multibyte_name_to_a_char_boundary_inside_the_bound`); non-ASCII display names/paths ride through unchanged; a slug with arbitrary bytes travels per-byte percent-encoded (`catalog.rs` unit test with `\0`); a newline in `$XDG_DATA_HOME` yields one escaped refusal line, pinned both in-module (`echo_path` test) and through the real subprocess (`cli_match_store_tests.rs:310` `a_newline_in_the_data_home_path_still_yields_one_escaped_line`).
6. **Native dependencies — pass (none).** No new external crate: the client is std-only, the two new `[dev-dependencies]` blocks are workspace-internal plus the already-present `serde_json` (`apps/discovery-cli/Cargo.toml`), and `Cargo.lock` is not in the diff — `cargo test --locked` green confirms the lock is untouched. The A17 worry (no C-toolchain build script entering the Windows compile-check leg) holds by inspection.
7. **Tests run on the platforms they claim — pass.** The only platform-gated test is the unix 0700/0600 pin (`#[cfg(unix)]`, runs on the Linux leg where it matters — not skipped everywhere); the Windows store-path rule is exercised on Linux through the seam, per Test strategy #15; every network test binds `127.0.0.1:0` (never a fixed port, never non-loopback); `run_cli` strips `BINDFORGE_CATALOG_URL`/`XDG_DATA_HOME`/`HOME`/`APPDATA` from the child so no test touches the developer's real store or network.

## Evidence gaps (stated, non-blocking)

- The CI Windows leg (`cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`, `.github/workflows/check.yml:29-32`) is not in the runner's captured gate record and this seat's shell allow would not run it. By inspection the new code has no unguarded platform API and its non-unix fallbacks are plain std, so no compile hazard is identifiable; the "compile-verified" matrix row stands or falls in CI, not in this record.

## Findings

1. **minor — `apps/discovery-cli/src/catalog.rs:438` (with `authority()`, `catalog.rs:224-228`).** The endpoint policy admits `[::1]` as one of exactly three loopback host spellings, but `authority()` strips the brackets and the request is formatted `Host: {host}:{port}` from the stripped host, so an IPv6 endpoint sends `Host: ::1:8080`. RFC 9110 requires a bracketed form for an IPv6 `uri-host` in `Host:`; this build only ever talks to its own fake (which ignores the header), so no test or gate catches it, but the pinned provisional wire spelling interops with permissive peers only — a strict loopback server fails the `[::1]` endpoint the policy advertises. Fix: keep the bracketed host for the `Host` header (the normalized base already spells `http://[::1]:port`) and strip brackets only for `to_socket_addrs`/`connect_timeout`; add the request-line spelling for the `[::1]` endpoint to the pinned unit surface.

## FYI (out of specialty, no severity)

- `crates/discovery/tests/.rm-probe` (zero bytes, committed in `0f25a167`) is a delete-probe residue referenced nowhere; the runner/operator should remove it.
- `crates/discovery/tests/retired/README.md` says the two retired suites "are kept here," but the README is that directory's only file; the suites actually stand at `tests/slug_tests.rs` / `tests/collision_tests.rs` behind the `retired-adr-024-suites` `required-features` gate.
- Comments in `catalog.rs:36` and `main.rs` (json-row doc) cite unit id `025-C6`; the ADR-025 contract set is `025-C1`–`025-C4`.

## Verdict

approve — zero blocker/major findings; one minor.
