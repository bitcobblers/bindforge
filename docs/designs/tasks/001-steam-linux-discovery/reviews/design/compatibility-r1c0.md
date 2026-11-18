# Compatibility review r1 (pass c0, full panel)

Scope: whole document (`docs/designs/tasks/001-steam-linux-discovery/02-proposal.md`).
Inputs used: the proposal (incl. its Revision log), `docs/tasks/T-001-steam-linux-discovery.md`,
read-only inspection of the code the proposal cites (`crates/discovery/src/*`,
`apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_tests.rs`, `Cargo.lock`,
`rust-toolchain.toml`, `.github/workflows/check.yml`), and the pinned toolchain's own
`std::env` source at the 1.85.0 tag. No run record other than this report was opened.

verdict: request-changes

Checklist coverage:
- Item 1 (matrix rows all have a status): pass — every row carries Yes / Partial / No; no
  `unknown`, no empty cell.
- Item 2 (OS input APIs X11/Wayland/Windows/macOS): not applicable — the design touches no
  input surface; it reads files.
- Item 3 (Node version floor): not applicable — no `tools/orchestration` change; the npm leg
  of `.github/workflows/check.yml:35-56` is untouched by the design.
- Item 4 (native modules / packaging story): pass — §10 and A15 keep the closure at the
  current 32 `[[package]]` entries in `Cargo.lock` (verified), pure-`std` reader, no build
  script, no prebuilds or node-gyp equivalent.
- Item 5 (keyboard reality / non-ASCII names): pass — `normalize_slug`
  (crates/discovery/src/slug.rs:6-12) keeps Unicode alphanumerics, so non-ASCII Steam names
  survive into the slug; the design does not transliterate or drop them. Slug *collisions*
  are the deferred item, not non-ASCII handling.
- Item 6 (paths by a library, per-OS config-dir conventions): findings 1 and 2.
- Item 7 (a platform test per supported matrix row): findings 3 and 4.

## Findings

- [blocker][new] Chosen design §3 *Library selection within a root* + §5 *Manifest to
  `DiscoveredGame`* — "`<root>/steamapps` itself, always." /
  "`install_path = l.join("steamapps").join("common").join(m.install_dir)`" — the document
  uses "library" in two mutually exclusive senses and never picks one. §3's closing sentence
  ("`appmanifest_*.acf` files are read only as entries directly inside a selected library's
  `steamapps/`") and §5's composition both treat a returned entry as a library *root*
  (something with `steamapps/` under it), and §2's canonical path rule says the same for
  "library paths within a root" — they are dropped when "they lack the required `steamapps`
  directory". The always-included entry, though, is `<root>/steamapps`, which is the
  `steamapps` directory itself. Consequences, all from the text as written: (a) that entry
  fails §2's own gate unless `<root>/steamapps/steamapps` exists, so the always-scan that
  Risks row 2 and A12 rely on to defeat the silent-zero bug instead silently drops the root's
  own library; (b) if the gate is read as not applying, the root's manifests are looked for
  at `<root>/steamapps/steamapps/appmanifest_*.acf` — a path no real Steam tree has — and the
  emitted `install_path` is one level too deep; (c) the §9 fixture, whose manifests sit
  directly in `steamapps/`, then yields one game (the secondary library's), not the four
  *Test strategy* #8 asserts, while #6's "ends with `steamapps/common/<installdir>`" passes
  vacuously and hides it; (d) the Compatibility matrix row "Linux, native Steam under `$HOME`
  … | Yes" and the source task's Done-when clause "Running `discovery list` on a Linux
  machine with Steam correctly identifies installed games"
  (docs/tasks/T-001-steam-linux-discovery.md:24) both fail for the most common single-library
  install. Fix: state one convention in §3 — `libraries_in_root` returns library *roots*,
  each containing a `steamapps/` directory; the always-included entry is `root` itself, gated
  on `root/steamapps` being a directory; §2's gate and §5's join stay as written. Then §9's
  fixture VDF entries become `"0" → ".."` and `"1" → "../../secondary"` (both joined to the
  VDF's own directory, `<fixture>/steamapps`), which keeps the "0" entry exercising
  root/library de-duping and the "1" entry exercising the relative join.

- [major][new] Chosen design §2 *Root selection* — "`home` comes from
  `std::env::home_dir()` (A9)" — that API is still `#[deprecated(since = "1.29.0", note =
  "This function's behavior may be unexpected on Windows. Consider using a crate from
  crates.io instead.")]` on the toolchain this repo pins (`rust-toolchain.toml` channel
  `1.85.0`; verified in `library/std/src/env.rs` at the 1.85.0 tag). The un-deprecation the
  1.85.0 release notes promised ("It will be un-deprecated in a subsequent release") landed
  after 1.85.0, so on the pinned floor a call from `crates/discovery` emits the warn-by-default
  `deprecated` lint, and `.github/workflows/check.yml:31` runs
  `cargo clippy --workspace --all-targets -- -D warnings` — Test strategy #10's gate, Rollout
  step 3 and the Compatibility matrix's "CI `rust` job … | Yes" row cannot pass as designed.
  §10 and A15 (no new dependency, `Cargo.lock` byte-identical) rule out the crates.io
  replacement the deprecation note asks for, so the design has no legal way to keep this line.
  A second, smaller mismatch in the same clause: A9 cites the *current* std docs, whose Unix
  rule is `$HOME` "if it is set (and not an empty string)", whereas the 1.85.0 text is
  "including to an empty string" — so on the pinned floor an empty `HOME` yields `Some("")`
  and every §2 candidate turns into a CWD-relative path that can pass the `steamapps` gate
  against an unrelated directory. Fix: replace that clause with `home` =
  `std::env::var_os("HOME")`, treating unset-or-empty as "no home" (empty candidate list, so
  §7's `Ok(vec![])`), leaving `host_candidates(home, env)` and its test unchanged; a bare
  `#[allow(deprecated)]` clears the lint but leaves the empty-`$HOME` behaviour.

- [minor][new] Chosen design §2 *Canonical path rule* + Compatibility matrix row "Linux,
  native Steam under `$HOME` (incl. `~/.steam/*`) | Yes" — "`std::fs::canonicalize` resolves
  the symlink aliases (`~/.steam/root` → the real root) … so no library is ever scanned
  twice" — the native row's Yes rests on de-duping the `~/.steam/steam` / `~/.steam/root` /
  `~/.steam/debian-installation` aliases of one install (§2 candidates 2), but *Test
  strategy* #3 pins only the VDF `"0" → "."` case, and §9's "No test writes to the
  filesystem" bars a symlink fixture, so the alias property this row claims is never asserted
  (checklist item 7). Fix: name a test that feeds the same directory to `from_roots` in two
  spellings that canonicalize identically (the fixture root and `<fixture>/steamapps/..`) and
  asserts the game count is unchanged — no symlink and no write needed.

- [minor][new] Compatibility matrix row "Linux, libraries on arbitrary mounts (`/run/media/…`,
  `/mnt/…`, ZFS) | Yes" — "Absolute `path` values are accepted as-is" — no test in *Test
  strategy* exercises the absolute branch of §3's rule: #3 covers the relative `../secondary`
  entry only, and §9 states outright that a checked-in fixture "cannot contain this machine's
  absolute paths", so the one code path that makes this row Yes is the only branch left
  untested (checklist item 7). Fix: add to §1's surface a pure
  `pub fn library_path(vdf_dir: &Path, value: &str) -> Option<PathBuf>` (the "absolute
  as-is, else join to the VDF's directory, drop empty" rule that §3 already states), have
  `libraries_in_root` use it, and assert both branches in Test strategy #3 — consistent with
  §1's "Every `pub` item above is exercised by a test", and reachable from the
  `crates/discovery/tests/` integration tests the design keeps. Otherwise downgrade this row
  to Partial.

## FYI (no severity)

- Correctness seat: the finding-1 path-model contradiction also makes §9's four-game fixture
  expectation and Test #8's exact slug list unattainable as written; I framed it as path
  semantics because the two senses of "library" are a path-composition defect.
- Correctness/complexity seat: §5's "`install_dir` … rejected when … absolute, or contains
  `/`" is Linux-complete (`/` is the only separator there, `\` is a legal filename byte). It
  is not the guard a Windows caller would want, but `parse_manifest` sits in `steam_linux.rs`
  and q18 claims reuse only of `keyvalues.rs`, so the claim holds.
- Not verified either way (no evidence available to this seat): A6, A7, A16, A20 remain
  `unverified` in the register with the stated rollback in Rollout step 5; that is the right
  handling, not a finding.
- Verified as claimed by the proposal and used above: A14 (crates/discovery/src/engine.rs:20-33
  swallows a plugin `Err` into one `eprintln!` on stderr and still returns `Ok`), A15 (32
  `[[package]]` entries in `Cargo.lock`; `crates/discovery/Cargo.toml` has one dependency),
  A17 (trait `discover(&self)` no-arg, five-field `DiscoveredGame`, `Platform::SteamLinux`,
  `DummyPlugin` used only at apps/discovery-cli/src/main.rs:41), A18 (rust job: fmt, clippy
  `-D warnings`, `cargo test --workspace --locked`, `ubuntu-latest` only), and the two existing
  CLI tests that Test #9 replaces (`apps/discovery-cli/tests/cli_tests.rs`, which today assert
  the header and `[]` by spawning `cargo run`).
