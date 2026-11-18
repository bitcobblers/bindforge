# compatibility review r3c0 (full panel, whole document)

verdict: request-changes

Inputs used: `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md` (whole document, incl. its
Revision log) and `docs/tasks/T-001-steam-linux-discovery.md`. No review report or transcript opened.
Code claims checked with read-only inspection of `crates/discovery/src/{engine.rs,plugin.rs,slug.rs}`,
`apps/discovery-cli/Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.github/workflows/check.yml`, and
the pinned toolchain's own `library/std/src/env.rs` at tag `1.85.0`.

## Checklist walk-through

1. **Matrix status per row** — pass. All 11 rows carry a status (`Yes` / `Partial` / `No`); no `unknown`,
   no empty cell. Excluded platforms are each traced to an owner (Windows/macOS/Deck → ADR-022).
2. **OS input APIs** — not applicable. The plugin touches no input subsystem; its OS surface is the
   filesystem plus `$HOME`/`$XDG_DATA_HOME`/`$SNAP_USER_DATA`. The design names its target platform
   (Linux) and justifies the one abstraction it introduces (`keyvalues.rs` platform-neutral for a later
   Windows plugin, r0: q18).
3. **Node version floor** — not applicable to the Node leg: no TypeScript or `tools/orchestration` file is
   in scope (the package's only "discovery" hit is an unrelated comment,
   tools/orchestration/src/pipeline/runner.ts:1403). The *analogous* floor — the pinned Rust toolchain — is
   where Finding 1 lives.
4. **Native modules** — pass, verified first-hand: `Cargo.lock` has exactly 32 `[[package]]` entries
   (A15's figure), `apps/discovery-cli/Cargo.toml` already depends on `clap`/`serde_json`, and
   `apps/discovery-cli/tests/` does not exist, so §10's "no new dependency, `Cargo.lock` byte-identical"
   and Test strategy #9's "both existing tests" are both consistent with the `--locked` gate.
5. **Keyboard reality / non-ASCII names** — see Finding 4.
6. **File paths** — pass with one hole. Everything composes through `PathBuf::join` +
   `fs::canonicalize` (no string concatenation), and the per-OS config-dir conventions are named
   individually (XDG default + set-but-relative rejection, `~/.steam/{steam,root,debian-installation}`,
   Flatpak `.var/app/com.valvesoftware.Steam`, Snap `snap/steam/common` + `$SNAP_USER_DATA`). The hole is
   the one path input taken from the environment without the design's own absolute-path discipline —
   Finding 2.
7. **Platform test per supported row** — Finding 3 (the arbitrary-mounts row's mechanism is exercised by
   no named test), and the "no home directory" row's mechanism is untestable as written, which
   Finding 2's fix also closes.

Verified first-hand and not in dispute: A14 (`crates/discovery/src/engine.rs:20-33` swallows a plugin
`Err` into one `eprintln!` and returns `Ok`), A17 (trait is two methods; `DummyPlugin`'s only code use is
`apps/discovery-cli/src/main.rs:41`, so deleting it breaks nothing outside that arm), A18 (`check.yml`
`rust` job: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked`, `ubuntu-latest` only).

## Findings

- [major][new] Assumption register A9 + Chosen design §2 Root selection (and §10's floor claim) —
  "`home` comes from `std::env::home_dir()` (A9)" / "`#[path]`-free, `unsafe`-free, no `std` feature flags
  required on the 1.85 floor (A15)" — on the toolchain this repo pins, that call is deprecated, so the
  design's own gate claim is false as written. `rust-toolchain.toml` pins channel `1.85.0`, and at tag
  `1.85.0` `library/std/src/env.rs` carries

  ```rust
  #[deprecated(since = "1.29.0", note = "This function's behavior may be unexpected on Windows. ...")]
  pub fn home_dir() -> Option<PathBuf>
  ```

  The undeprecation landed *after* the floor (Rust 1.86.0 release notes: "Undeprecate `env::home_dir`"),
  and A9's "verified" basis is the unversioned `doc.rust-lang.org/std/env/fn.home_dir.html`, i.e. current
  stable, which no longer shows the deprecation — a floor/version mismatch in the register itself.
  Consequence: the first `cargo clippy --workspace --all-targets -- -D warnings` (the `Clippy` step of the
  CI `rust` job, A18) fails on the `deprecated` lint at the one call site, and the design's
  *Dependency posture* forecloses the usual outs — §10 grants no new dependency ("`Cargo.toml` gain
  nothing", `Cargo.lock` byte-identical), so `dirs`/`home` are out and a toolchain bump is out of this
  task. Fix: state the call-site decision in §2 in one clause — call it under a scoped
  `#[allow(deprecated)]` on `from_host()` with a note that the lint is the 1.85.0 floor's, not a real
  defect on Unix (`home_dir()` there is exactly `$HOME`, else the passwd entry, which is all §2 needs) —
  or drop the std call and read `std::env::var_os("HOME")` with a documented empty-value rule, which is
  the same text Finding 2 needs. Then correct A9's basis to the versioned 1.85.1 page so the register
  records the floor it was verified against, and add the `deprecated` lint to the §10 posture sentence
  instead of leaving "no `std` feature flags required on the 1.85 floor" as the whole account of the floor.

- [minor][new] Chosen design §2 Root selection — "`home` comes from `std::env::home_dir()` (A9); when it
  returns `None`, the candidate list is empty and the plugin discovers nothing" — that is the only guard
  on the one path input the design takes from the environment, and on the pinned toolchain it is not the
  guard the wording implies. `home_dir()` is documented in 1.85.0 to return the value of `HOME` "if it is
  set (**including to an empty string**)", so a service, container or cron context with `HOME=` gets
  `Some("")`, not `None`, and §2's candidates become `.local/share/Steam`, `.steam/steam`, `.steam/root`,
  `.var/app/com.valvesoftware.Steam/…`, `snap/steam/common/…` — *relative* paths. The "kept only if
  `<candidate>/steamapps` is a directory" probe and the canonical path rule both resolve those against the
  process working directory, so `discovery list` run from a directory that happens to contain a
  `.steam/steam/steamapps` tree treats that tree as an installed Steam root — host discovery that silently
  depends on where the user happened to `cd`. This contradicts the Compatibility matrix row "Linux, Steam
  not installed / no home directory | Yes | `Ok(vec![])`, no error, no stderr output (§7)", and it is the
  one place where the design's own discipline for the neighbouring env input is missing: candidate 1 gets
  "a set-but-relative `XDG_DATA_HOME` is ignored per the base-directory spec (A9)". Fix: extend the §2
  rule by one clause to the input it is currently applied to the sibling of — `host_candidates` returns an
  empty candidate list when `home` is empty or relative — and name the case in *Test strategy* #5
  alongside the existing "a relative `XDG_DATA_HOME` does not (A9)" assertion. `host_candidates` is
  already pure and already asserted path-by-path, so this costs no new seam, no write and no dependency.

- [minor][new] Compatibility matrix + Test strategy #3 — the row "Linux, libraries on arbitrary mounts
  (`/run/media/…`, `/mnt/…`, ZFS) | Yes | Absolute `path` values are accepted as-is; an unmounted library
  is skipped, not fatal" is the only `Yes` row whose distinguishing mechanism no test in *Test strategy*
  exercises. §3's rule has two branches — "a `path` is used as-is when absolute and otherwise joined to
  the directory holding the VDF" — and the fixture reaches only the second one: §9's `libraryfolders.vdf`
  holds `"0" → "."` and `"1" → "../secondary"`, #3 asserts exactly those two, and §9 explains why the
  absolute branch cannot appear in a checked-in file ("a checked-in fixture cannot contain this machine's
  absolute paths"). A fabricated absolute value in the fixture cannot cover it either: the `steamapps`
  gate in the canonical path rule would drop it, so the assertion would pass without ever reaching
  "accepted as-is". The `--steam-root` seam does not close the gap either — it supplies *roots*, which
  skip `libraries_in_root`'s VDF branch. So the mechanism the row rests on is asserted nowhere, and the
  row's second clause (an unmounted library "is skipped, not fatal") is likewise unclaimed by #7, whose
  two cases are a nonexistent root and an all-unparseable `steamapps`. Fix: make the branch a pure
  function in §1's public surface — `pub fn resolve_library_path(vdf_dir: &Path, value: &str) ->
  Option<PathBuf>` — and have `libraries_in_root` call it, then extend Test strategy #3 with the two
  assertions the row needs on values it can name without touching the filesystem: an absolute value such
  as `/bindforge-abs/Media/SteamLibrary` resolves to itself, and `"1"` under
  `tests/fixtures/steam/minimal/steamapps` resolves inside the fixture. This fits §1's existing rule that
  "every `pub` item above is exercised by a test in *Test strategy*"; it replaces the status quo of the
  branch living inline in `libraries_in_root`, where it can only be reached through a machine-specific
  file.

- [nit][new] Test strategy #8 (and #6's "every returned game has … `slug ==
  normalize_slug(display_name)`") — "returns exactly four games — `unknown/example-game`,
  `unknown/mixed-case-game`, `unknown/nameless-install`, `unknown/secondary-title`" — every display name
  the suite pins is ASCII, so nothing in the run records what a Linux user with a localized or accented
  store name gets. `normalize_slug` (crates/discovery/src/slug.rs:1-35) tests each character with
  `char::is_alphanumeric`, so accented and non-Latin names survive into the slug rather than collapsing to
  `unknown/` with an empty tail — good behavior that is an accident of the existing helper rather than an
  expectation this design pins, on the one input (§5's `display_name`) that a real Steam install supplies
  in the user's own locale. Fix: give one fixture manifest an accented `name` (say `"Les Misérables"`),
  add its row to #8's expected list and its `unknown/les-misérables` slug to #6's contract. The
  slug format itself stays ADR-022's deferred problem — this pins only that non-ASCII names do not
  degrade to `unknown/`.

## FYI (no severity)

- §2's canonical path rule means every emitted `install_path` is the symlink-resolved real path, so a user
  whose library is a symlink (`~/.local/share/Steam` under a symlinked home, a
  `/mnt/games/SteamLibrary` link) sees a different path from the one Steam and their own shell show, and a
  binding that stores it will not survive a re-link or a second machine with a different mount point.
  Consistent with "nothing the plugin emits carries an unresolved `..`" and with *Deferred*'s "Validating
  `install_path` before a codec writes to it", but the stability property of the emitted prefix is nobody's
  stated decision.
- §9's `compatdata/900001/pfx/…/Steam/steamapps/` decoy tree adds the literal segment `Program Files
  (x86)` to a checked-in path, and `appmanifest_900005.acf`'s `"installdir" "../outside"` is a tracked
  file whose content is a traversal string. Neither is followed by the code, but some git archive and
  editor tooling reacts to both; a one-line note in §9 that these are inert decoys would head off the
  question.
- §5's rejects (`.`/`..`/absolute/contains `/`) bound traversal but not the character class of
  `installdir`, so a control character or a NUL in the parsed value reaches `PathBuf::join` and the
  printed/JSON-emitted `install_path`; the same applies to `display_name`. Left to the security seat.
