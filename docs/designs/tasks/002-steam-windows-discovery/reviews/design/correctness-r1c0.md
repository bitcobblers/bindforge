# correctness review r1c0

verdict: request-changes

Scope: whole document (`docs/designs/tasks/002-steam-windows-discovery/02-proposal.md`).
Inputs used: the proposal, `docs/tasks/T-002-steam-windows-discovery.md`, the
in-document Revision log (all entries are r0 self-decisions, no prior review
round to re-test), the cited repository files, and webfetch spot-checks of the
register's URLs.

## Findings

- [major][new] Assumption register A1 (and §2's reuse claim) — "A1 | Everything
  after root selection is the same on Windows as on Linux: … and library `path`
  values written with doubled backslashes. | verified | …https://github.com/Playmoir/knowledge-base/blob/main/steam-library-integration.md"
  — that citation is dead: both `github.com/Playmoir/knowledge-base` and the
  GitHub API report 404 (no such repository), and the `raw.githubusercontent.com`
  form 404s too. It is the only live-checked support in A1 for the doubled-backslash
  spelling and the second of two bases for the row's `verified` status (the other
  is a StackOverflow question that fetches 403 to a non-browser client, so it is
  inconclusive rather than confirming). A1 is the row that licenses the document's
  central move — "everything after root selection moves into a shared module" —
  so a `verified` row resting on a nonexistent source is the anti-hallucination
  failure the checklist names. A19 and §2's second-location paragraph also cite
  it, but A19 survives on its other basis, which I did verify live
  (`lutris/lutris/master/lutris/util/steam/config.py` reads `config/libraryfolders.vdf`).
  Fix: in A1, replace the Playmoir URL with a live source that actually shows a
  Windows `libraryfolders.vdf` `path` value with escaped backslashes, or split the
  `path`-spelling clause out of A1 and mark it `unverified` with the note that A2's
  `\\`/`\"` unescape (crates/discovery/src/keyvalues.rs:186-207, which I verified)
  is what makes either spelling arrive usable.

- [major][new] §5 root candidates vs §1's seam shape vs Test strategy #3 —
  "2. `HKLM\SOFTWARE\Valve\Steam` → `InstallPath`: the 32-bit-view result, then the
  64-bit-view result if the first was `None` (A4)" against §1's
  "`pub enum RegistryKey { CurrentUserValveSteam, LocalMachineValveSteam }`" and
  Test strategy #3's "HKLM-32-bit-view only; HKLM-64-bit-view only".
  §3 puts the view fallback inside the concrete reader ("for the HKLM key, open
  with `KEY_READ | KEY_WOW64_32KEY` first and, if that yields nothing,
  `KEY_READ | KEY_WOW64_64KEY`"), which is a `#[cfg(windows)]`-only function
  (§3, §1). So `host_candidates` receives exactly one value per key — it cannot
  order "the 32-bit-view result, then the 64-bit-view result" as §5 says — and a
  fake keyed by `(RegistryKey, &str)` (§4: "a closure over a
  `HashMap<(RegistryKey, &str), String>`") cannot express "32-bit view only" versus
  "64-bit view only". The two named #3 cases are therefore unwritable, and the
  view-ordering policy — the design's whole answer to A4 and its reason for
  rejecting the hardcoded `Wow6432Node` spelling — ends up covered only by
  Test strategy #12's type-check and #13's manual run. Fix: put the view in the
  seam so `host_candidates` orders it — e.g. `RegistryKey::LocalMachineValveSteam32`
  and `…ValveSteam64` variants (four orderable candidates: HKCU, HKLM-32, HKLM-64,
  envs) with `registry_string_value` reduced to "open this key with these flags,
  read one `REG_SZ`" — keeping Test strategy #3's two cases as written; or delete
  those two cases from #3 and say plainly in §5 and Risks that view ordering is
  compile-verified and manually verified only.

- [major][new] §8 CLI wiring (and the Goals bullet "`steam-windows` wired into
  `apps/discovery-cli`'s `discovery list`, with `--steam-root` still working") —
  §8's `else` branch is "// `--steam-root` is the Linux seam from T-001 …
  engine.register_plugin(Box::new(SteamLinuxPlugin::from_roots(steam_root.iter().cloned())));",
  i.e. when `--steam-root` is given the Windows plugin is not registered at all and
  no `steam-windows` row can ever come from an explicit root. The task's Done-when
  says the opposite at docs/tasks/T-002-steam-windows-discovery.md:65-67: "The
  `steam-windows` plugin is wired into `apps/discovery-cli` (`discovery list`)
  alongside `steam-linux`, and `--steam-root` keeps working **for it**." §8
  acknowledges the gap ("A user who wants `steam-windows`-tagged rows from an
  explicit root is a one-line CLI change with its own argument, and that question
  is *Deferred*") and Deferred hands it away, so this is a deferral that
  contradicts a stated requirement of the source document, not an out-of-scope
  refinement. Fix (smallest, keeps the three CLI tests undoubled): in the
  `--steam-root` branch register the Windows plugin instead of the Linux one when
  compiled for Windows (`#[cfg(windows)]`), so the flag feeds the plugin whose rows
  the host can actually produce, and state that §8's "no `#[cfg]` in `main.rs`" is
  dropped for that branch; otherwise register
  `SteamWindowsPlugin::from_roots(steam_root.iter().cloned())` alongside the Linux
  plugin and say in §8 and Test strategy #9 that the fixture-row count doubles to
  eight and `cli_tests.rs:68-72` / `:124` are re-pinned across both platform tags —
  and in either case record the reading of :66-67 in the Revision log rather than
  in Deferred.

- [minor][new] §5 root candidates — "A relative value is ignored by the same
  `absolute_env` helper Linux uses (crates/discovery/src/steam_linux.rs:338-342)" —
  `absolute_env` is a private `fn` in `steam_linux.rs`, and §1's inventory of what
  moves ("moved verbatim from steam_linux.rs: library_paths,
  libraries_in_root_with_skips, scan_library, read_text_file, canonical_paths,
  discovered_game, and parse_manifest/usable_install_dir") does not include it, while
  §1 also keeps `host_candidates` in `steam_linux`. As specified, `steam_windows`'s
  `host_candidates` can neither call it nor share it, and copying it is the
  duplication the Done-when clause forbids. Fix: add `absolute_env` to §1's moved
  list (`steam_common`, `pub(crate)`), with `steam_linux`'s call sites rewired to it.

- [minor][new] §5 + Test strategy #3 — "each only when the variable is set and
  absolute" is decided by `absolute_env`, whose predicate is `PathBuf::is_absolute`,
  which §6 itself flags as host-dependent ("`Path::is_absolute` (which reads a
  Windows path as relative on a Linux host)"). `host_candidates` is compiled and
  tested on Linux (§3, Test strategy #3), so the env gate can only pass its own
  tests there if the fake supplies POSIX-absolute values, leaving the Windows-shaped
  value the design is actually about (`C:\Program Files (x86)`) either untested or
  dropped. Fix: in §5, state which values the fake supplies and add one #3 case that
  pins what a Windows-absolute value does on each host — ideally by generalizing the
  absoluteness check over `SteamTarget` (alongside `install_dir_rejects_backslash`)
  so the rule under test on Linux is the rule that runs on Windows.

- [nit][new] §1 surface sketch vs §2's no-drift claim — §2 says the prefix is
  "`T::NAME`, which is the same string the plugin's `name()` returns, so the prefix
  can no longer drift from the registered plugin name", but §1's sketch writes
  "impl DiscoveryPlugin for SteamWindowsPlugin { /* delegates to `core`,
  name() = "steam-windows" */ }", i.e. a second spelling of the tag next to
  `WindowsTarget::NAME`. Fix: write `name()` as `self.core.name()` in the §1 sketch
  for both plugins, so the single-source claim the §2 sentence makes is actually the
  shape on the page.

## Checklist disposition

1. Assumption register — all code-cited bases check out exactly: A2
   (keyvalues.rs:186-207 unescapes `\\` and `\"`), A13 (rust-toolchain.toml:7
   `1.85.0`, Cargo.toml:14-18 edition 2024 / `rust-version = "1.85"`,
   crates/discovery/Cargo.toml:6-7 `serde` alone), A14 (steam_linux.rs:38-43, 46-50,
   58-93, 101, 322-334), A15 (steam_linux_tests.rs:12-14 imports exactly those four
   items; main.rs:33-38 registers one plugin; cli_tests.rs:68-72 "exactly the four
   fixture rows", :124 `"SteamLinux"`), A16 (game.rs:5-18), A10 (Cargo.lock:250
   `windows-link 0.2.1`, :256 `windows-sys 0.61.2`, no `cfg-if`, no `winreg`),
   §9's `check.yml:28-33` three commands, and A9's live API surface (docs.rs
   `winreg::reg_key::RegKey`: `open_subkey_with_flags(path, REG_SAM_FLAGS)`,
   `get_raw_value -> RegValue { vtype, bytes }`), plus winreg 0.56.0 existing with
   `license: MIT`, `rust_version: 1.60`, deps `cfg-if ^1.0` and
   `windows-sys >=0.59, <=0.61`, ≈2.0K SLoC. A1 is the failure: finding 1. A17-A21
   are honestly marked `unverified` and the body never states them as fact; §5's
   use of A20's expected-failure model is confined to the `unknown` 32-bit row.
2. URL spot-checks — 6 checked: crates.io winreg (supports), docs.rs RegKey
   (supports), Microsoft alternate-registry-view (supports A4/A5, including
   "These flags have no effect on shared registry keys" and "applications should not
   use these keys directly"), Microsoft USMT env vars (supports A6's
   64-bit-systems-only `(x86)`), lutris `config.py` (supports A19),
   steam-game-path `windows-registry.md` (supports A18 and the Options paragraph on
   the `reg.exe` encoding hazard and per-key process spawn). Playmoir: dead →
   finding 1. The StackOverflow links 403 to this client; recorded as
   not-verifiable, not as evidence against the design.
3. Internal contradictions — findings 2 (seam shape vs candidate order vs test
   cases) and 5 (tag spelled twice).
4. Missing edge cases — empty root set, absent keys, unset env vars, unmounted
   libraries and name-collapsing are covered, and `canonical_paths`
   (steam_linux.rs:322-334) really does sort and de-dup, so §5's "a double-listed
   root costs nothing" holds. One edge the design does not argue: on a Windows host
   §8 registers `SteamLinuxPlugin::from_host()` too, whose `HOME` lookup
   (steam_linux.rs:39-41) can succeed under MSYS/Git-Bash; the design's only
   justification is the comment "each plugin probes its own platform". Noted as FYI.
5. Interfaces — one shape, one name, except finding 5.
6. Does it decide what the source asked? Yes for the registry crate and seam, the
   decomposition, VDF location, `\\?\` namespace, `installdir` hardening,
   `REG_EXPAND_SZ`, candidate order and gating shape (all eight Revision-log
   entries answer a real open question with a reason). One task clause is not met
   as designed: finding 3.
7. Previous-round findings — not applicable: this is the first review round; the
   Revision log's entries are the author's own r0 decisions, and none of the
   findings above is a re-report of a rejected one.

## FYI (no severity)

- §9/Test strategy #11 pin the committed lock diff to "`winreg` and `cfg-if`" on
  the reading that the already-locked `windows-sys 0.61.2` satisfies winreg 0.56's
  published bound `windows-sys >=0.59, <=0.61`. That depends on how Cargo treats a
  partial version in a `<=` comparator; if it resolves as `<=0.61.0`, a second
  `windows-sys` entry lands in the lock and #11's pinned assertion fails. I could
  not establish it with the allowed check commands. (compatibility/complexity)
- §7 ships `\\?\`-prefixed `install_path` into the table and `--json`; the design
  documents it in Risks and Deferred, but it is a consumer-visible shape choice.
  (compatibility)
- §8 registers `SteamLinuxPlugin::from_host()` on a Windows host; its `HOME`-based
  candidate list can be non-empty there (MSYS/Git-Bash sets `HOME`), and the
  canonicalize-and-require-`steamapps` rule is what silently makes it inert. One
  sentence in §8 would make that deliberate rather than incidental. (correctness
  adjacency, below finding threshold)
- Deferred's "`crates/discovery/Cargo.toml` → `version.workspace = true`" is real:
  that file still spells `version = "0.1.0"` while the workspace declares it.
  (complexity)
