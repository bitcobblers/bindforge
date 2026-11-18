//! Unit and fixture-integration tests for the Windows Steam discovery plugin
//! (`crates/discovery/src/steam_windows.rs`, T-002).
//!
//! Every test is hermetic: root selection is driven through the fake registry
//! and fake environment lookup behind the §4 seam, and scans run through
//! `from_roots` / `from_registry_and_roots` at the same read-only fixture
//! trees the Linux tests use — no new fixture tree is introduced
//! (T-002 Chosen design §1–§6, Test strategy #1–#8). No test touches a real
//! registry or the network; the `#[cfg(windows)]` registry reader is
//! compile-verified by the cross-target `cargo check` gate (Test strategy
//! #12), never executed here.

use discovery::game::{DiscoveredGame, Platform};
use discovery::plugin::DiscoveryPlugin;
use discovery::registry::DiscoveryError;
use discovery::slug::normalize_slug;
use discovery::steam_linux;
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::{
    RegistryKey, SteamWindowsPlugin, host_candidates, parse_manifest, usable_install_dir,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A checked-in fixture root under `tests/fixtures/steam/` (shared with the
/// Linux tests; Test strategy #5 forbids adding new ones).
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/steam")
        .join(name)
}

/// The plugin's canonical path rule emits canonical paths; tests compare
/// against the same canonicalization.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).expect("fixture path must exist")
}

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

/// RegistryKey / value-name constants as §3–§5 name them.
const STEAM_PATH: &str = "SteamPath";
const INSTALL_PATH: &str = "InstallPath";

/// The two environment defaults with their single pinned spelling (A21,
/// docs/tasks/T-002-steam-windows-discovery.md:44). Values in these tests are
/// absolute on the Linux gate host, which is where `host_candidates` runs.
const PROGRAM_FILES_X86: &str = "ProgramFiles(x86)";
const PROGRAM_FILES: &str = "ProgramFiles";

/// The `<value>\Steam` default root candidate, spelled with the host's join.
fn default_candidate(value: &str) -> PathBuf {
    PathBuf::from(value).join("Steam")
}

/// KeyValues text for an `AppState` section carrying the given
/// `name` / `installdir` pair (same shape as the Linux suite's helper).
fn manifest_text(name: Option<&str>, installdir: Option<&str>) -> String {
    let mut text = String::from("\"AppState\"\n{\n\t\"appid\"\t\t\"900001\"\n");
    if let Some(n) = name {
        text.push_str(&format!("\t\"name\"\t\t\"{n}\"\n"));
    }
    if let Some(i) = installdir {
        text.push_str(&format!("\t\"installdir\"\t\t\"{i}\"\n"));
    }
    text.push_str("}\n");
    text
}

/// `steam_windows::parse_manifest` reduced to a comparable shape.
fn windows_manifest(text: &str) -> Option<(String, String)> {
    parse_manifest(text).map(|m| (m.display_name, m.install_dir))
}

/// `steam_linux::parse_manifest` reduced to the same comparable shape.
fn linux_manifest(text: &str) -> Option<(String, String)> {
    steam_linux::parse_manifest(text).map(|m| (m.display_name, m.install_dir))
}

fn no_env(_key: &str) -> Option<String> {
    None
}

/// Fake registry behind the §4 seam: answers only the `(key, value)` pairs it
/// was given — every other query (including a refused value type) is `None` —
/// and records every query so the probing policy is observable.
struct FakeRegistry {
    values: HashMap<(RegistryKey, String), String>,
    calls: RefCell<Vec<(RegistryKey, String)>>,
}

impl FakeRegistry {
    fn new(entries: &[(RegistryKey, &str, &str)]) -> Self {
        Self {
            values: entries
                .iter()
                .map(|(key, name, value)| ((*key, (*name).to_string()), (*value).to_string()))
                .collect(),
            calls: RefCell::new(Vec::new()),
        }
    }

    /// The seam shape: `&dyn Fn(RegistryKey, &str) -> Option<String>`.
    fn lookup(&self) -> impl Fn(RegistryKey, &str) -> Option<String> + '_ {
        |key, name| {
            self.calls.borrow_mut().push((key, name.to_string()));
            self.values.get(&(key, name.to_string())).cloned()
        }
    }

    fn calls(&self) -> Vec<(RegistryKey, String)> {
        self.calls.borrow().clone()
    }
}

/// Fake environment lookup: answers only the variables it was given and
/// records every queried name, pinning the A21 spelling.
struct FakeEnv {
    values: HashMap<String, String>,
    calls: RefCell<Vec<String>>,
}

impl FakeEnv {
    fn new(vars: &[(&str, &str)]) -> Self {
        Self {
            values: vars
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
            calls: RefCell::new(Vec::new()),
        }
    }

    fn lookup(&self) -> impl Fn(&str) -> Option<String> + '_ {
        |name| {
            self.calls.borrow_mut().push(name.to_string());
            self.values.get(name).cloned()
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

/// The ordered three-`(key, value)`-pair query list §5 prescribes.
fn expected_registry_calls() -> Vec<(RegistryKey, String)> {
    vec![
        (RegistryKey::CurrentUserValveSteam, STEAM_PATH.to_string()),
        (
            RegistryKey::LocalMachineValveSteam32,
            INSTALL_PATH.to_string(),
        ),
        (
            RegistryKey::LocalMachineValveSteam64,
            INSTALL_PATH.to_string(),
        ),
    ]
}

/// The plugin's rows reduced to everything except the platform tag.
fn rows_without_platform(games: Vec<DiscoveredGame>) -> Vec<(String, String, PathBuf, bool)> {
    games
        .into_iter()
        .map(|g| (g.slug, g.display_name, g.install_path, g.codec_id.is_none()))
        .collect()
}

// ---------------------------------------------------------------------------
// parse_manifest parity with the Linux suite (Chosen design §2)
// ---------------------------------------------------------------------------

// proves: Test strategy #1 (Chosen design §2) — for every case in the current
// Linux `parse_manifest` suite (name+installdir, the name-absent and
// empty-name fallbacks, no `AppState`, unparseable text, mixed key case, and
// the `""`/`.`/`..`/`/abs/path`/`sub/dir` reject set)
// `steam_windows::parse_manifest` agrees with `steam_linux::parse_manifest`:
// the sharing claim is a test, not a comment.
#[test]
fn parse_manifest_agrees_with_linux_on_every_shared_case() {
    let cases = [
        (
            "name and installdir",
            Some("Example Game"),
            Some("Example Game"),
        ),
        ("name absent", None, Some("Nameless Install")),
        ("name empty", Some(""), Some("Nameless Install")),
        ("neither name nor installdir", None, None),
    ];
    for (label, name, installdir) in cases {
        let text = manifest_text(name, installdir);
        assert_eq!(
            windows_manifest(&text),
            linux_manifest(&text),
            "Windows must agree with Linux on the `{label}` case"
        );
    }
    let good = manifest_text(Some("Example Game"), Some("Example Game"));
    assert_eq!(
        windows_manifest(&good),
        Some(("Example Game".to_string(), "Example Game".to_string())),
        "the parity assertions must not pass on vacuous None-equality"
    );

    let absent = manifest_text(Some("Some Game"), None);
    assert_eq!(
        windows_manifest(&absent),
        linux_manifest(&absent),
        "an absent installdir must be refused on both platforms"
    );
    for bad in ["", ".", "..", "/abs/path", "sub/dir"] {
        let text = manifest_text(Some("Some Game"), Some(bad));
        assert_eq!(
            windows_manifest(&text),
            linux_manifest(&text),
            "Windows must agree with Linux on rejecting installdir `{bad}`"
        );
        assert!(
            windows_manifest(&text).is_none(),
            "`{bad}` must be refused on Windows too"
        );
    }

    let no_appstate =
        "\"NotAppState\"\n{\n\t\"name\" \"Example Game\"\n\t\"installdir\" \"Example Game\"\n}\n";
    assert_eq!(
        windows_manifest(no_appstate),
        linux_manifest(no_appstate),
        "a document without an AppState section must be refused identically"
    );

    let unparseable = "\"AppState\"\n{\n\t\"name\" \"Example Game\"\n"; // unbalanced braces
    assert_eq!(
        windows_manifest(unparseable),
        linux_manifest(unparseable),
        "unparseable text must be refused identically"
    );

    let mixed_case = "\"appstate\"\n{\n\t\"NAME\"\t\t\"Mixed Case Game\"\n\t\"stateflags\"\t\t\"4\"\n\t\"INSTALLDIR\"\t\t\"Mixed Case Game\"\n}\n";
    assert_eq!(
        windows_manifest(mixed_case),
        linux_manifest(mixed_case),
        "mixed key case must resolve identically"
    );
    assert_eq!(
        windows_manifest(mixed_case)
            .as_ref()
            .map(|(display, _)| display.as_str()),
        Some("Mixed Case Game"),
        "the mixed-case case must actually parse"
    );
}

// ---------------------------------------------------------------------------
// The one rule that differs: the `\` refusal (Chosen design §6)
// ---------------------------------------------------------------------------

// proves: Test strategy #2 (Chosen design §6, A7) — an `installdir` containing
// a backslash (`foo\bar`, the doubled form `foo\\bar` a real Windows ACF writes
// per A2, `C:\games`, `\\server\share`) yields None from
// `steam_windows::parse_manifest` and still yields Some from
// `steam_linux::parse_manifest`.
#[test]
fn parse_manifest_refuses_installdir_containing_backslash() {
    // Raw spellings as they appear inside the quoted ACF token. The reader
    // unescapes the doubled form to a single `\` (A2); the `\` clause then
    // refuses it on the Windows side only (§6).
    for raw in ["foo\\bar", "foo\\\\bar", "C:\\games", "\\\\server\\share"] {
        let text = manifest_text(Some("Some Game"), Some(raw));
        assert!(
            parse_manifest(&text).is_none(),
            "Windows must refuse installdir `{raw}`: `\\` is the path separator (A7)"
        );
        assert!(
            linux_manifest(&text).is_some(),
            "Linux must still accept `{raw}`: a backslash is an ordinary filename byte (A7)"
        );
    }
}

// proves: Test strategy #2 (Chosen design §6) — the Windows rule set is the
// Linux reject set plus the `\` clause, and nothing wider (reserved names,
// trailing dots and spaces stay Deferred).
#[test]
fn usable_install_dir_refuses_backslash_beyond_the_shared_rule_set() {
    for bad in [
        "",
        ".",
        "..",
        "/abs/path",
        "sub/dir",
        "foo\\bar",
        "C:\\games",
        "\\\\server\\share",
    ] {
        assert!(!usable_install_dir(bad), "Windows must refuse `{bad}`");
    }
    assert!(usable_install_dir("Example Game"));
    assert!(usable_install_dir("Nameless Install"));
}

// ---------------------------------------------------------------------------
// host_candidates — pure root selection (Chosen design §5)
// ---------------------------------------------------------------------------

// proves: Test strategy #3 (Chosen design §5, A3–A6) — with both keys present
// the list is exactly HKCU `SteamPath`, then the two HKLM `InstallPath` views
// in 32-then-64 order, then the `%ProgramFiles(x86)%\Steam` and
// `%ProgramFiles%\Steam` defaults — appended even though registry candidates
// were found — and the env lookup queries each pinned spelling exactly once,
// `(x86)` first (A21).
#[test]
fn host_candidates_orders_registry_candidates_then_env_defaults() {
    let reg = FakeRegistry::new(&[
        (
            RegistryKey::CurrentUserValveSteam,
            STEAM_PATH,
            "/steam/hkcu",
        ),
        (
            RegistryKey::LocalMachineValveSteam32,
            INSTALL_PATH,
            "/steam/hklm32",
        ),
        (
            RegistryKey::LocalMachineValveSteam64,
            INSTALL_PATH,
            "/steam/hklm64",
        ),
    ]);
    let env = FakeEnv::new(&[(PROGRAM_FILES_X86, "/pf-x86"), (PROGRAM_FILES, "/pf")]);
    let mut expected = paths(&["/steam/hkcu", "/steam/hklm32", "/steam/hklm64"]);
    expected.push(default_candidate("/pf-x86"));
    expected.push(default_candidate("/pf"));
    assert_eq!(
        host_candidates(&reg.lookup(), &env.lookup()),
        expected,
        "the candidate list must be HKCU, HKLM-32, HKLM-64, then the env defaults in order"
    );
    assert_eq!(
        env.calls(),
        [PROGRAM_FILES_X86.to_string(), PROGRAM_FILES.to_string()],
        "the probe must query the two pinned env spellings exactly once, `(x86)` first (§5, A21)"
    );
}

// proves: Test strategy #3 (Chosen design §5, A5) — HKCU alone yields exactly
// that one candidate.
#[test]
fn host_candidates_with_only_hkcu_lists_only_that_candidate() {
    let reg = FakeRegistry::new(&[(
        RegistryKey::CurrentUserValveSteam,
        STEAM_PATH,
        "/steam/hkcu",
    )]);
    assert_eq!(
        host_candidates(&reg.lookup(), &no_env),
        paths(&["/steam/hkcu"]),
        "a per-user install with no machine key and no env vars yields the HKCU candidate alone"
    );
}

// proves: Test strategy #3 (Chosen design §5, A4) — the HKLM 32-bit view alone
// yields exactly that one candidate.
#[test]
fn host_candidates_with_only_the_32_bit_hklm_view_lists_only_that_candidate() {
    let reg = FakeRegistry::new(&[(
        RegistryKey::LocalMachineValveSteam32,
        INSTALL_PATH,
        "/steam/hklm32",
    )]);
    assert_eq!(
        host_candidates(&reg.lookup(), &no_env),
        paths(&["/steam/hklm32"]),
        "the 32-bit-view InstallPath alone is a complete candidate list"
    );
}

// proves: Test strategy #3 (Chosen design §5, A4) — the HKLM 64-bit view alone
// yields exactly that one candidate.
#[test]
fn host_candidates_with_only_the_64_bit_hklm_view_lists_only_that_candidate() {
    let reg = FakeRegistry::new(&[(
        RegistryKey::LocalMachineValveSteam64,
        INSTALL_PATH,
        "/steam/hklm64",
    )]);
    assert_eq!(
        host_candidates(&reg.lookup(), &no_env),
        paths(&["/steam/hklm64"]),
        "the 64-bit-view InstallPath alone is a complete candidate list"
    );
}

// proves: Test strategy #3 (Chosen design §5) — when both HKLM views answer
// with distinct paths, both values are appended in 32-then-64 order.
#[test]
fn host_candidates_lists_both_hklm_view_values_in_32_then_64_order() {
    let reg = FakeRegistry::new(&[
        (
            RegistryKey::LocalMachineValveSteam32,
            INSTALL_PATH,
            "/steam/hklm32",
        ),
        (
            RegistryKey::LocalMachineValveSteam64,
            INSTALL_PATH,
            "/steam/hklm64",
        ),
    ]);
    assert_eq!(
        host_candidates(&reg.lookup(), &no_env),
        paths(&["/steam/hklm32", "/steam/hklm64"]),
        "both views answer: the values append in §5's 32-bit-then-64-bit order"
    );
}

// proves: Test strategy #3 (Chosen design §5) — both views reporting one path
// collapse to one root through the canonical-path rule: the install is scanned
// once, not twice.
#[test]
fn from_registry_and_roots_collapses_two_views_of_one_root() {
    let root_str = fixture("minimal").display().to_string();
    let reg = FakeRegistry::new(&[
        (
            RegistryKey::LocalMachineValveSteam32,
            INSTALL_PATH,
            &root_str,
        ),
        (
            RegistryKey::LocalMachineValveSteam64,
            INSTALL_PATH,
            &root_str,
        ),
    ]);
    let games = SteamWindowsPlugin::from_registry_and_roots(&reg.lookup(), &no_env)
        .discover()
        .expect("a root reported by both views must discover");
    assert_eq!(
        games.len(),
        4,
        "the two identical view candidates must collapse to one scanned root, not double the rows"
    );
    assert!(
        games.iter().all(|g| g.platform == Platform::SteamWindows),
        "every row must be tagged SteamWindows"
    );
}

// proves: Test strategy #3 (Chosen design §5) — the two views pointing at
// distinct installs yield two roots, and the shared canonical-path dedup keeps
// the overlap (the secondary install is also the primary's secondary library)
// at one copy per library.
#[test]
fn from_registry_and_roots_scans_two_view_roots_deduped() {
    let primary = fixture("minimal").display().to_string();
    let secondary = fixture("minimal").join("secondary").display().to_string();
    let reg = FakeRegistry::new(&[
        (
            RegistryKey::LocalMachineValveSteam32,
            INSTALL_PATH,
            &primary,
        ),
        (
            RegistryKey::LocalMachineValveSteam64,
            INSTALL_PATH,
            &secondary,
        ),
    ]);
    let games = SteamWindowsPlugin::from_registry_and_roots(&reg.lookup(), &no_env)
        .discover()
        .expect("two distinct roots must discover");
    assert_eq!(
        games.len(),
        4,
        "two roots sharing one library must yield exactly the fixture's four rows, deduped"
    );
    let slugs: Vec<&str> = games.iter().map(|g| g.slug.as_str()).collect();
    assert_eq!(
        slugs,
        [
            "unknown/example-game",
            "unknown/mixed-case-game",
            "unknown/nameless-install",
            "unknown/secondary-title",
        ],
        "both installs contribute their rows, ordered by (slug, install_path)"
    );
    assert!(
        games.iter().all(|g| g.platform == Platform::SteamWindows),
        "every row must be tagged SteamWindows"
    );
}

// proves: Test strategy #3 (Chosen design §5) — with neither key set, the list
// is exactly the two env candidates in order.
#[test]
fn host_candidates_without_registry_keys_yields_exactly_the_env_defaults() {
    let reg = FakeRegistry::new(&[]);
    let env = FakeEnv::new(&[(PROGRAM_FILES_X86, "/pf-x86"), (PROGRAM_FILES, "/pf")]);
    assert_eq!(
        host_candidates(&reg.lookup(), &env.lookup()),
        vec![default_candidate("/pf-x86"), default_candidate("/pf")],
        "no registry candidates: exactly `(x86)\\Steam` then `ProgramFiles\\Steam`"
    );
}

// proves: Test strategy #3 (Chosen design §4, §5) — a key the seam reports as
// None (a refused value type: `REG_EXPAND_SZ`, `REG_MULTI_SZ`, other `vtype`)
// contributes no candidate, and the env defaults carry the coverage.
#[test]
fn refused_registry_values_contribute_no_candidate() {
    // An empty value table stands in for §4's typing refusal: the probe reads
    // every key and gets None from each, exactly as the Windows reader does
    // for a non-`REG_SZ` value.
    let reg = FakeRegistry::new(&[]);
    let env = FakeEnv::new(&[(PROGRAM_FILES_X86, "/pf-x86"), (PROGRAM_FILES, "/pf")]);
    let candidates = host_candidates(&reg.lookup(), &env.lookup());
    assert_eq!(
        reg.calls().len(),
        3,
        "every key is still probed; the refusal is per-value, not a skip"
    );
    assert_eq!(
        candidates,
        vec![default_candidate("/pf-x86"), default_candidate("/pf")],
        "a refused value contributes no candidate (§4)"
    );
}

// proves: Test strategy #3 (Chosen design §5, A6) — `%ProgramFiles(x86)%`
// unset (the 32-bit-host case) leaves only the `%ProgramFiles%\Steam` default.
#[test]
fn host_candidates_without_program_files_x86_yields_only_the_program_files_default() {
    let reg = FakeRegistry::new(&[]);
    let env = FakeEnv::new(&[(PROGRAM_FILES, "/pf")]);
    assert_eq!(
        host_candidates(&reg.lookup(), &env.lookup()),
        vec![default_candidate("/pf")],
        "an unset `(x86)` variable is no candidate — the whole story on 32-bit hosts (A6)"
    );
}

// proves: Test strategy #3 (Chosen design §5) — neither key nor any env
// variable yields the empty list.
#[test]
fn host_candidates_without_keys_or_env_is_empty() {
    let reg = FakeRegistry::new(&[]);
    assert_eq!(
        host_candidates(&reg.lookup(), &no_env),
        Vec::<PathBuf>::new(),
        "a host with no Steam keys and no ProgramFiles variables has no candidates"
    );
}

// proves: Test strategy #3 (Chosen design §5, §1 `absolute_env`) — a
// set-but-relative env value is ignored; the absolute one still stands.
#[test]
fn host_candidates_ignores_a_relative_env_value() {
    let reg = FakeRegistry::new(&[]);
    let env = FakeEnv::new(&[
        (PROGRAM_FILES_X86, "relative/steam-files"),
        (PROGRAM_FILES, "/pf"),
    ]);
    assert_eq!(
        host_candidates(&reg.lookup(), &env.lookup()),
        vec![default_candidate("/pf")],
        "a relative `(x86)` value contributes no candidate"
    );
}

// proves: Test strategy #3 (Chosen design §5) — a registry value equal to an
// env candidate is NOT de-duplicated by `host_candidates`; the canonical-path
// rule is what collapses it (the double-listed root costs nothing).
#[test]
fn host_candidates_does_not_dedup_a_registry_value_equal_to_an_env_default() {
    let reg = FakeRegistry::new(&[(
        RegistryKey::CurrentUserValveSteam,
        STEAM_PATH,
        "/pf-x86/Steam",
    )]);
    let env = FakeEnv::new(&[(PROGRAM_FILES_X86, "/pf-x86"), (PROGRAM_FILES, "/pf")]);
    let mut expected = paths(&["/pf-x86/Steam"]);
    expected.push(default_candidate("/pf-x86"));
    expected.push(default_candidate("/pf"));
    let candidates = host_candidates(&reg.lookup(), &env.lookup());
    assert_eq!(
        candidates, expected,
        "the probe appends both spellings verbatim"
    );
    assert_eq!(
        candidates.len(),
        3,
        "`host_candidates` must not de-dup — that is the canonical-path rule's job (§5)"
    );
}

// ---------------------------------------------------------------------------
// SteamWindowsPlugin through the registry seam and the shared core
// ---------------------------------------------------------------------------

// proves: Test strategy #4 (Chosen design §5, §7) — a registry-sourced root
// reaches the shared scan end to end: the four minimal-fixture games come back
// tagged `Platform::SteamWindows` with `codec_id == None`,
// `slug == normalize_slug(display_name)`, `install_path` composed from the
// canonicalized library's `steamapps/common/`, ordered by (slug, install_path).
#[test]
fn from_registry_and_roots_scans_the_registry_sourced_root_end_to_end() {
    let root_arg = fixture("minimal");
    let root = canonical(&root_arg);
    let root_str = root_arg.display().to_string();
    let reg = FakeRegistry::new(&[(RegistryKey::CurrentUserValveSteam, STEAM_PATH, &root_str)]);
    let games = SteamWindowsPlugin::from_registry_and_roots(&reg.lookup(), &no_env)
        .discover()
        .expect("a registry-sourced root must discover without error");

    assert_eq!(games.len(), 4, "the four fixture games must come back");
    for game in &games {
        assert_eq!(game.platform, Platform::SteamWindows);
        assert!(game.codec_id.is_none(), "codec_id must stay None");
        assert_eq!(game.slug, normalize_slug(&game.display_name));
    }

    let expected = [
        ("unknown/example-game", "", "Example Game"),
        ("unknown/mixed-case-game", "", "Mixed Case Game"),
        ("unknown/nameless-install", "", "Nameless Install"),
        ("unknown/secondary-title", "secondary", "Secondary Title"),
    ];
    for (game, (slug, library, install_dir)) in games.iter().zip(expected) {
        assert_eq!(
            game.slug, slug,
            "rows must arrive in (slug, install_path) order"
        );
        let library_root = if library.is_empty() {
            root.clone()
        } else {
            canonical(&root.join(library))
        };
        let want = library_root
            .join("steamapps")
            .join("common")
            .join(install_dir);
        assert_eq!(
            game.install_path, want,
            "install_path must be the canonical library's steamapps/common path (emitted even though it does not exist)"
        );
    }
}

// proves: Test strategy #5 (Chosen design §1, §2) — the Windows plugin on the
// shared minimal fixture returns the same four rows as the Linux plugin except
// for the `platform` field; no new fixture tree is introduced.
#[test]
fn windows_rows_match_linux_rows_except_the_platform_tag() {
    let root = fixture("minimal");
    let linux = SteamLinuxPlugin::from_roots([root.clone()])
        .discover()
        .expect("the Linux scan of the shared fixture must succeed");
    let windows = SteamWindowsPlugin::from_roots([root])
        .discover()
        .expect("the Windows scan of the shared fixture must succeed");

    assert_eq!(windows.len(), 4, "the same four fixture rows");
    assert!(
        windows.iter().all(|g| g.platform == Platform::SteamWindows),
        "Windows rows must be tagged SteamWindows"
    );
    assert!(
        linux.iter().all(|g| g.platform == Platform::SteamLinux),
        "Linux rows must stay tagged SteamLinux"
    );
    assert_eq!(
        rows_without_platform(windows),
        rows_without_platform(linux),
        "slug, display_name, install_path and codec_id must be identical across platforms"
    );
}

// proves: Test strategy #5 (Chosen design §2 — the shared core's decoy rules)
// — no Windows row carries a `compatdata/` path or the prefix-decoy slug.
#[test]
fn windows_rows_never_carry_compatdata_paths_or_the_prefix_decoy() {
    let games = SteamWindowsPlugin::from_roots([fixture("minimal")])
        .discover()
        .expect("the minimal fixture root must discover without error");
    assert_eq!(
        games.len(),
        4,
        "only the four fixture games may be discovered"
    );
    for game in &games {
        assert!(
            !game.install_path.to_string_lossy().contains("compatdata"),
            "a Proton prefix path must never appear in output: {:?}",
            game.install_path
        );
        assert!(
            !game.slug.contains("prefix-decoy"),
            "the compatdata decoy game must never be discovered: {:?}",
            game.slug
        );
    }
}

// ---------------------------------------------------------------------------
// Failure policy and identity (Chosen design §2, §1)
// ---------------------------------------------------------------------------

// proves: Test strategy #6/#10 (Chosen design §2, §3) — the
// `Ok(vec![])`-when-no-roots rule: a Windows plugin with zero roots (what
// `from_host()` yields on a Linux host, §3) is "not installed here", not an
// error the engine would print.
#[test]
fn discover_with_no_roots_is_empty_ok() {
    let games = SteamWindowsPlugin::from_roots(Vec::new())
        .discover()
        .expect("zero roots must yield Ok, not Err");
    assert!(games.is_empty(), "no roots must yield zero games");
}

// proves: Test strategy #6 (Chosen design §2) — a nonexistent root is "not
// installed here": `Ok(vec![])`, same rule as Linux.
#[test]
fn discover_with_nonexistent_root_returns_empty_ok() {
    let missing = fixture("minimal").join("no-such-steam-root");
    let games = SteamWindowsPlugin::from_roots([missing])
        .discover()
        .expect("an unresolvable root must yield Ok, not Err");
    assert!(games.is_empty(), "no roots must yield zero games");
}

// proves: Test strategy #6 (Chosen design §2) — a selected root whose every
// manifest is malformed yields exactly one `DiscoveryError::PluginError` whose
// message starts `steam-windows:`, equals the plugin's own `name()` as the
// prefix, names the selected root, and leaks no file contents.
#[test]
fn all_broken_root_yields_plugin_error_prefixed_with_steam_windows() {
    let plugin = SteamWindowsPlugin::from_roots([fixture("all-broken")]);
    match plugin.discover() {
        Ok(games) => panic!(
            "a root that emitted zero games with files skipped must be Err, got {} games",
            games.len()
        ),
        Err(DiscoveryError::PluginError(msg)) => {
            assert!(
                msg.starts_with("steam-windows:"),
                "the message must start with the `steam-windows:` prefix, got: {msg}"
            );
            assert!(
                msg.starts_with(&format!("{}:", plugin.name())),
                "the message prefix must equal the plugin's registered name(), got: {msg}"
            );
            assert!(
                msg.contains("all-broken"),
                "the message must name the selected root, got: {msg}"
            );
            for leak in [
                "AppState",
                "appid",
                "installdir",
                "BrokenMarker900201",
                "BrokenMarker900202",
                "Broken Install",
            ] {
                assert!(
                    !msg.contains(leak),
                    "the message must not leak file contents (`{leak}` found): {msg}"
                );
            }
        }
        Err(other) => panic!("expected DiscoveryError::PluginError, got {other:?}"),
    }
}

// proves: Test strategy #6 (Chosen design §2) — the same prefix-equals-`name()`
// rule for the Linux plugin (the shared core keeps its message prefix stable).
#[test]
fn linux_all_broken_error_prefix_equals_the_linux_plugin_name() {
    let plugin = SteamLinuxPlugin::from_roots([fixture("all-broken")]);
    match plugin.discover() {
        Ok(games) => panic!("the all-broken root must be Err, got {} games", games.len()),
        Err(DiscoveryError::PluginError(msg)) => {
            assert!(
                msg.starts_with("steam-linux:"),
                "the message must keep the `steam-linux:` prefix, got: {msg}"
            );
            assert!(
                msg.starts_with(&format!("{}:", plugin.name())),
                "the message prefix must equal the plugin's registered name(), got: {msg}"
            );
        }
        Err(other) => panic!("expected DiscoveryError::PluginError, got {other:?}"),
    }
}

// proves: Test strategy #7 (Chosen design §1, §2 `T::NAME`) — plugin identity:
// the Windows plugin registers under "steam-windows", the Linux one under
// "steam-linux".
#[test]
fn plugin_names_are_steam_windows_and_steam_linux() {
    assert_eq!(
        SteamWindowsPlugin::from_roots(Vec::new()).name(),
        "steam-windows"
    );
    assert_eq!(
        SteamLinuxPlugin::from_roots(Vec::new()).name(),
        "steam-linux"
    );
}

// ---------------------------------------------------------------------------
// Registry seam shape (Chosen design §4, §5)
// ---------------------------------------------------------------------------

// proves: Test strategy #8 (Chosen design §4, §5) — the probe queries exactly
// the three (key, value-name) pairs §5 names, in §5's order, so a fourth
// (key, view) pair cannot be added silently. The `#[cfg(windows)]` reader
// behind the seam (`registry_string_value`, the crate's only `winreg`
// consumer) is compile-verified by Test strategy #12's cross-target check.
#[test]
fn host_candidates_queries_exactly_the_three_registry_pairs_in_order() {
    let reg = FakeRegistry::new(&[]);
    let candidates = host_candidates(&reg.lookup(), &no_env);
    assert!(
        candidates.is_empty(),
        "a fully absent registry yields no candidates"
    );
    assert_eq!(
        reg.calls(),
        expected_registry_calls(),
        "the probe must query HKCU `SteamPath`, then HKLM-32 `InstallPath`, then HKLM-64 `InstallPath` — exactly, in order, nothing else"
    );
}

// proves: Test strategy #8 (Chosen design §5) — every key is probed even when
// an earlier one answered: the candidate set is collected, there is no
// early-stop on the first hit.
#[test]
fn host_candidates_probes_every_key_even_after_earlier_hits() {
    let reg = FakeRegistry::new(&[
        (
            RegistryKey::CurrentUserValveSteam,
            STEAM_PATH,
            "/steam/hkcu",
        ),
        (
            RegistryKey::LocalMachineValveSteam32,
            INSTALL_PATH,
            "/steam/hklm32",
        ),
        (
            RegistryKey::LocalMachineValveSteam64,
            INSTALL_PATH,
            "/steam/hklm64",
        ),
    ]);
    let _ = host_candidates(&reg.lookup(), &no_env);
    assert_eq!(
        reg.calls(),
        expected_registry_calls(),
        "all three pairs are queried in order regardless of earlier answers"
    );
}
