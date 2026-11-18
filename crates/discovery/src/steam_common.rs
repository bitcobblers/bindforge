//! The Steam scan core shared by the Linux and Windows plugins (T-002 Chosen
//! design §1, §2).
//!
//! Everything after root selection — the `libraryfolders.vdf` read, the
//! `appmanifest_*.acf` enumeration, the size-capped UTF-8 reader, the
//! canonical-path rule, skip counting, the single error path and the output
//! sort — lives here exactly once. What genuinely differs per platform (the
//! `Platform` tag and message prefix, and whether `installdir` may contain
//! `\`) arrives through the sealed [`SteamTarget`] parameter, so the two
//! plugins are thin wrappers over one [`SteamCore`].
//!
//! The module is crate-private; its items are `pub` so `steam_linux` and
//! `steam_windows` can use and re-export them, and nothing outside the crate
//! reaches them.

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

/// Seals [`SteamTarget`] to the platforms this crate ships (T-002 Chosen
/// design §1): `steam_linux` and `steam_windows` implement this, no crate
/// outside can, and the trait carries no downcast escape hatch.
pub(crate) mod sealed {
    /// The sealing supertrait of [`super::SteamTarget`].
    pub trait Sealed {}
}

/// The platform parameterization of the shared scan core (T-002 Chosen
/// design §1): the tag and message prefix the core emits, and the one scan
/// rule whose value differs per platform.
pub trait SteamTarget: Copy + sealed::Sealed {
    /// The registered plugin name; also the error message's prefix, so the
    /// prefix can no longer drift from the registered name (§2).
    const NAME: &'static str;

    /// The platform tag of the rows this target emits (§2, §7).
    fn platform(&self) -> Platform;

    /// Whether an `installdir` containing `\` is refused (§6). The Linux
    /// target answers `false`, keeping its rule set bit-for-bit as it is.
    fn install_dir_rejects_backslash(&self) -> bool;
}

/// A Steam scan over a fixed, canonicalized set of Steam roots for the
/// platform `T` (T-002 Chosen design §2).
///
/// The roots are held on the core because `DiscoveryPlugin::discover` takes
/// no arguments (T-001 A17); the plugin wrappers' root-injection seams all
/// run through [`SteamCore::with_roots`].
#[derive(Debug, Clone)]
pub struct SteamCore<T: SteamTarget> {
    roots: Vec<PathBuf>,
    target: T,
}

impl<T: SteamTarget> SteamCore<T> {
    /// Builds a core over caller-supplied roots, each taken through the
    /// canonical path rule.
    pub(crate) fn with_roots(roots: impl IntoIterator<Item = PathBuf>, target: T) -> Self {
        Self {
            roots: canonical_paths(roots),
            target,
        }
    }
}

impl<T: SteamTarget> DiscoveryPlugin for SteamCore<T> {
    fn name(&self) -> &'static str {
        T::NAME
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
                scan_library(&library, &self.target, &mut games, &mut skipped);
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
                "{}: no games discovered from {} selected root(s) [{}]; {} file(s) skipped as unreadable or malformed",
                T::NAME,
                self.roots.len(),
                roots.join(", "),
                skipped
            )));
        }

        games.sort_by(|a, b| (&a.slug, &a.install_path).cmp(&(&b.slug, &b.install_path)));
        Ok(games)
    }
}

/// The libraries of one Steam root (Chosen design §3): the root itself always,
/// plus every `path` in `<root>/steamapps/libraryfolders.vdf`, all taken through
/// the canonical path rule. `libraries_in_root` (per-platform) is a thin
/// wrapper over this.
pub fn libraries_in_root_with_skips(root: &Path, skipped: &mut usize) -> Vec<PathBuf> {
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
fn scan_library<T: SteamTarget>(
    library: &Path,
    target: &T,
    games: &mut Vec<DiscoveredGame>,
    skipped: &mut usize,
) {
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
        match parse_manifest(&text, target) {
            Some(manifest) => games.push(discovered_game(library, &manifest, target)),
            None => *skipped += 1,
        }
    }
}

/// Builds the emitted row for `manifest` in `library` (Chosen design §5, A2).
/// The `platform` field is the only one the target changes (§2).
fn discovered_game<T: SteamTarget>(
    library: &Path,
    manifest: &AppManifest,
    target: &T,
) -> DiscoveredGame {
    DiscoveredGame {
        slug: normalize_slug(&manifest.display_name),
        display_name: manifest.display_name.clone(),
        // Emitted whether or not the directory exists; `library` is canonical,
        // and `install_dir` is validated by `parse_manifest`.
        install_path: library
            .join("steamapps")
            .join("common")
            .join(&manifest.install_dir),
        platform: target.platform(),
        codec_id: None,
    }
}

/// Extracts the display name and install directory of one `appmanifest_*.acf`.
///
/// `None` means "skip this manifest": no `AppState` section, a parse error, an
/// unusable `installdir`, or no name and no install directory to fall back on.
/// The `installdir` rule set is `T`'s (§2, §6); everything else is shared.
pub fn parse_manifest(text: &str, target: &impl SteamTarget) -> Option<AppManifest> {
    let document = keyvalues::parse(text).ok()?;
    let app_state = document.get_section("appstate")?;
    let install_dir = app_state.get_str("installdir")?;
    if !usable_install_dir(install_dir, target.install_dir_rejects_backslash()) {
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
/// is not a name any platform can open. `reject_backslash` is the Windows-only
/// extra clause (T-002 §6): on Windows `\` is the path separator, so
/// `foo\bar` would silently become a nested path inside the emitted
/// `install_path`.
pub fn usable_install_dir(install_dir: &str, reject_backslash: bool) -> bool {
    !install_dir.trim().is_empty()
        && !install_dir.contains('\0')
        && install_dir != "."
        && install_dir != ".."
        && !Path::new(install_dir).is_absolute()
        && !install_dir.contains('/')
        && !(reject_backslash && install_dir.contains('\\'))
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
/// `~/.steam/root`, so nothing is scanned twice. On Windows it also collapses
/// two registry views of one install, and yields extended-length (`\\?\`)
/// paths (T-002 §5, §7, A8).
pub fn canonical_paths(entries: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
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
pub fn absolute_env(env: &dyn Fn(&str) -> Option<String>, key: &str) -> Option<PathBuf> {
    let value = env(key)?;
    let path = PathBuf::from(value);
    path.is_absolute().then_some(path)
}
