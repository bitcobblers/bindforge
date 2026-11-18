//! The Linux Steam discovery plugin and its host policy (Chosen design §2-§7).
//!
//! The plugin reads library folders from `<root>/steamapps/libraryfolders.vdf`,
//! always includes the root itself as a library, enumerates the
//! `appmanifest_*.acf` files directly inside each library's `steamapps/`, and
//! emits one [`DiscoveredGame`] per surviving manifest. AppIDs are never read
//! and never stored (ADR-022).

use crate::game::{DiscoveredGame, Platform};
use crate::keyvalues::{self, Value};
use crate::plugin::DiscoveryPlugin;
use crate::registry::DiscoveryError;
use crate::slug::normalize_slug;
use std::path::{Path, PathBuf};

/// A parsed `appmanifest_*.acf`, reduced to the two fields discovery emits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppManifest {
    pub display_name: String,
    pub install_dir: String,
}

/// A Steam Linux discovery plugin over a fixed set of Steam roots.
///
/// The roots are held on the plugin because `DiscoveryPlugin::discover` takes
/// no arguments (A17); `from_roots` is the seam tests and `--steam-root` use.
#[derive(Debug, Clone)]
pub struct SteamLinuxPlugin {
    roots: Vec<PathBuf>,
}

impl SteamLinuxPlugin {
    /// Probes this host for Steam installations (Chosen design §2).
    ///
    /// `home` is read from `HOME`: unset, empty or relative means no roots.
    /// `std::env::home_dir` is deprecated on the pinned toolchain, so it is not
    /// called (A9).
    pub fn from_host() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        Self::from_roots(host_candidates(&home, &|key| std::env::var(key).ok()))
    }

    /// Scans caller-supplied roots, each taken through the canonical path rule.
    pub fn from_roots(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            roots: canonical_paths(roots),
        }
    }
}

impl DiscoveryPlugin for SteamLinuxPlugin {
    fn name(&self) -> &'static str {
        "steam-linux"
    }

    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
        let mut games: Vec<DiscoveredGame> = Vec::new();
        let mut skipped: usize = 0;
        if !self.roots.is_empty() {
            // The libraries of every selected root are one set, so a library two
            // roots share is scanned once (Chosen design §2).
            let mut libraries: Vec<PathBuf> = Vec::new();
            for root in &self.roots {
                libraries.extend(libraries_in_root_with_skips(root, &mut skipped));
            }
            for library in canonical_paths(libraries) {
                scan_library(&library, &mut games, &mut skipped);
            }
        }

        // §7's one error path, its three clauses spelled out: at least one root
        // was selected, zero games were emitted, and at least one file was
        // skipped as unreadable or malformed. The message names the selected
        // roots and the count of skipped files and nothing else — no contents.
        if !self.roots.is_empty() && games.is_empty() && skipped > 0 {
            let roots: Vec<String> = self
                .roots
                .iter()
                .map(|root| root.display().to_string())
                .collect();
            return Err(DiscoveryError::PluginError(format!(
                "steam-linux: no games discovered from {} selected root(s) [{}]; {} file(s) skipped as unreadable or malformed",
                self.roots.len(),
                roots.join(", "),
                skipped
            )));
        }

        games.sort_by(|a, b| (&a.slug, &a.install_path).cmp(&(&b.slug, &b.install_path)));
        Ok(games)
    }
}

/// The ordered Steam root candidates for `home` (Chosen design §2, A8/A9).
///
/// Pure: the environment is a lookup function, so the list is testable without
/// touching the process environment. An empty or relative `home` has no
/// candidates.
pub fn host_candidates(home: &Path, env: &dyn Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    // An unset, empty or relative home has no candidates (A9).
    if !home.is_absolute() {
        return Vec::new();
    }

    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. `$XDG_DATA_HOME/Steam`, default `$HOME/.local/share/Steam`; a
    //    set-but-relative value is invalid and ignored (A9).
    candidates.push(match absolute_env(env, "XDG_DATA_HOME") {
        Some(data_home) => data_home.join("Steam"),
        None => home.join(".local/share/Steam"),
    });

    // 2. The native install directories.
    for suffix in [".steam/steam", ".steam/root", ".steam/debian-installation"] {
        candidates.push(home.join(suffix));
    }

    // 3. Flatpak.
    let flatpak = home.join(".var/app/com.valvesoftware.Steam");
    for suffix in [".local/share/Steam", ".steam/steam", ".steam/root"] {
        candidates.push(flatpak.join(suffix));
    }

    // 4. Snap: `$SNAP_USER_DATA/steam/common`, default `$HOME/snap/steam/common`.
    let snap = match absolute_env(env, "SNAP_USER_DATA") {
        Some(snap_user_data) => snap_user_data.join("steam/common"),
        None => home.join("snap/steam/common"),
    };
    for suffix in [".local/share/Steam", ".steam/steam", ".steam/root"] {
        candidates.push(snap.join(suffix));
    }

    candidates
}

/// The libraries of one Steam root (Chosen design §3): the root itself always,
/// plus every `path` in `<root>/steamapps/libraryfolders.vdf`, all taken through
/// the canonical path rule.
pub fn libraries_in_root(root: &Path) -> Vec<PathBuf> {
    libraries_in_root_with_skips(root, &mut 0)
}

/// `libraries_in_root`, counting a malformed or unreadable
/// `libraryfolders.vdf` into `skipped` for §7's error path.
fn libraries_in_root_with_skips(root: &Path, skipped: &mut usize) -> Vec<PathBuf> {
    // A library is the *parent* of its `steamapps/` (A2), so the root is a
    // library: a `libraryfolders.vdf` that yields nothing degrades to this.
    let mut libraries: Vec<PathBuf> = vec![root.to_path_buf()];
    let vdf = root.join("steamapps").join("libraryfolders.vdf");
    for path in library_paths(&vdf, skipped) {
        libraries.push(if path.is_absolute() {
            path
        } else {
            // A relative `path` is relative to the directory holding the VDF;
            // checked-in fixtures cannot carry this machine's absolute paths.
            vdf.parent().unwrap_or(Path::new(".")).join(path)
        });
    }
    canonical_paths(libraries)
}

/// The `path` values of the numeric-keyed blocks of a `libraryfolders.vdf`
/// (A3). `vdf` is read at this one fixed path and never searched for, so the
/// Proton-prefix look-alikes (A4) are never opened.
fn library_paths(vdf: &Path, skipped: &mut usize) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let Some(text) = read_text_file(vdf, skipped) else {
        return paths; // absent or unreadable: a degraded scan, not an error
    };
    let Ok(document) = keyvalues::parse(&text) else {
        *skipped += 1;
        return paths;
    };
    let Some(folders) = document.get_section("libraryfolders") else {
        return paths;
    };
    for (key, value) in &folders.children {
        // Modern shape: libraries are keyed by decimal index (A3); anything
        // else, such as the pre-2021 keys, is not a library block.
        if key.is_empty() || !key.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let Value::Section(block) = value else {
            continue;
        };
        match block.get_str("path") {
            // Empty values are dropped.
            Some(path) if !path.is_empty() => paths.push(PathBuf::from(path)),
            _ => continue,
        }
    }
    paths
}

/// Reads the `appmanifest_*.acf` files directly inside `<library>/steamapps/`,
/// adding one game per surviving manifest and counting every skipped file.
fn scan_library(library: &Path, games: &mut Vec<DiscoveredGame>, skipped: &mut usize) {
    let entries = match std::fs::read_dir(library.join("steamapps")) {
        Ok(entries) => entries,
        Err(_) => {
            *skipped += 1;
            return;
        }
    };
    for entry in entries {
        let Ok(entry) = entry else {
            *skipped += 1;
            continue;
        };
        let path = entry.path();
        let is_manifest = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"));
        if !is_manifest || !path.is_file() {
            continue;
        }
        // `read_text_file` counts its own refusals, so only a manifest that
        // parses but carries nothing usable is counted here.
        let Some(text) = read_text_file(&path, skipped) else {
            continue;
        };
        match parse_manifest(&text) {
            Some(manifest) => games.push(discovered_game(library, &manifest)),
            None => *skipped += 1,
        }
    }
}

/// Builds the emitted row for `manifest` in `library` (Chosen design §5, A2).
fn discovered_game(library: &Path, manifest: &AppManifest) -> DiscoveredGame {
    DiscoveredGame {
        slug: normalize_slug(&manifest.display_name),
        display_name: manifest.display_name.clone(),
        // Emitted whether or not the directory exists; `library` is canonical,
        // and `install_dir` is validated by `parse_manifest`.
        install_path: library
            .join("steamapps")
            .join("common")
            .join(&manifest.install_dir),
        platform: Platform::SteamLinux,
        codec_id: None,
    }
}

/// Extracts the display name and install directory of one `appmanifest_*.acf`.
///
/// `None` means "skip this manifest": no `AppState` section, a parse error, an
/// unusable `installdir`, or no name and no install directory to fall back on.
pub fn parse_manifest(text: &str) -> Option<AppManifest> {
    let document = keyvalues::parse(text).ok()?;
    let app_state = document.get_section("appstate")?;
    let install_dir = app_state.get_str("installdir")?;
    if !usable_install_dir(install_dir) {
        return None;
    }
    // `display_name` needs to be non-empty because the slug derives from it
    // (ADR-022), so an absent or empty `name` falls back to `installdir`.
    let name = app_state.get_str("name").unwrap_or_default().trim();
    let display_name = if name.is_empty() { install_dir } else { name };
    Some(AppManifest {
        display_name: display_name.to_string(),
        install_dir: install_dir.to_string(),
    })
}

/// Whether `installdir` is safe to append to an emitted path: it is a single
/// directory name, never `.`, `..`, an absolute path, or a nested path
/// (Chosen design §5). An interior NUL byte is refused for the same reason §5
/// refuses `..`: the value is appended to a path a later consumer opens, and it
/// is not a name any platform can open.
fn usable_install_dir(install_dir: &str) -> bool {
    !install_dir.trim().is_empty()
        && !install_dir.contains('\0')
        && install_dir != "."
        && install_dir != ".."
        && !Path::new(install_dir).is_absolute()
        && !install_dir.contains('/')
}

/// Reads a small, valid-UTF-8 file. `None` means the file could not be read as
/// text: an absent file is a normal degraded scan, while an unreadable,
/// oversize or non-UTF-8 file is counted into `skipped` (Chosen design §4, §7).
fn read_text_file(path: &Path, skipped: &mut usize) -> Option<String> {
    // The size cap is applied before the contents are allocated (A16); a
    // metadata failure is left to the read below to report.
    const MAX_FILE_BYTES: u64 = keyvalues::MAX_DOCUMENT_BYTES as u64;
    let oversized = match std::fs::metadata(path) {
        Ok(meta) => meta.len() > MAX_FILE_BYTES,
        Err(_) => false,
    };
    if oversized {
        *skipped += 1;
        return None;
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                *skipped += 1;
            }
            return None;
        }
    };
    match String::from_utf8(bytes) {
        // A caller skips a file that is not valid UTF-8 rather than abort a scan.
        Ok(text) => Some(text),
        Err(_) => {
            *skipped += 1;
            None
        }
    }
}

/// The canonical path rule (Chosen design §2): canonicalize each entry, drop
/// entries that fail to canonicalize or lack a `steamapps` directory, then sort
/// and de-duplicate. Canonicalization resolves symlink aliases such as
/// `~/.steam/root`, so nothing is scanned twice.
fn canonical_paths(entries: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut unique = std::collections::BTreeSet::new();
    for entry in entries {
        let Ok(canonical) = std::fs::canonicalize(entry) else {
            continue;
        };
        if !canonical.join("steamapps").is_dir() {
            continue;
        }
        unique.insert(canonical);
    }
    unique.into_iter().collect()
}

/// An environment value that is set and absolute; a relative value is invalid
/// and treated as unset (A9).
fn absolute_env(env: &dyn Fn(&str) -> Option<String>, key: &str) -> Option<PathBuf> {
    let value = env(key)?;
    let path = PathBuf::from(value);
    path.is_absolute().then_some(path)
}
