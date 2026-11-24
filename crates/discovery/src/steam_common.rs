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

use crate::game::{DiscoveredGame, Platform, ScanRecord, Source, friendly_name};
use crate::keyvalues::{self, Value};
use crate::plugin::DiscoveryPlugin;
use crate::registry::DiscoveryError;
use bindforge_core::MAX_DISCOVERY_SLUG_OCTETS;
use std::path::{Path, PathBuf};

/// The widest appid the emission seam accepts: Steam's own ids are decimal, and
/// 20 digits is the largest value a 64-bit id can hold, so anything longer is
/// not an appid at all (`025-C1`).
const MAX_APPID_DIGITS: usize = 20;

/// A parsed `appmanifest_*.acf`, reduced to the three fields discovery emits.
/// ADR-025 adds `appid`: it is the row's identity source, the value the lookup
/// key is composed from, and a manifest without a usable one is refused here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppManifest {
    pub appid: String,
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

    /// The selected root set exactly as the core holds it — canonicalized,
    /// existence-filtered, byte order — which is what `roots()` hands a
    /// `GameContext` (design 017, §Discovery plumbing).
    pub(crate) fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// The scan behind `discover()` (design 017, §Discovery plumbing): the
    /// same enumeration as before the refactor — the per-root library
    /// enumeration and the shared dedupe are kept — with each row now carrying
    /// the root that *owns* the library it was found in.
    ///
    /// A library two roots share is scanned once, owned by the first root in
    /// `roots()` (canonical byte) order that named it; the `(slug,
    /// install_path)` sort stays the only ordering, the no-collision decision
    /// is unchanged, and §7's one error path lives here.
    pub(crate) fn scan(&self) -> Result<Vec<ScanRecord>, DiscoveryError> {
        let mut records: Vec<ScanRecord> = Vec::new();
        let mut skipped: usize = 0;
        if !self.roots.is_empty() {
            // The libraries of every selected root are one set, so a library two
            // roots share is scanned once (Chosen design §2) — now with its
            // owner kept: the first naming root claims it (design 017).
            let mut seen: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
            for root in &self.roots {
                for library in libraries_in_root_with_skips(root, &mut skipped) {
                    if !seen.insert(library.clone()) {
                        continue;
                    }
                    scan_library(&library, root, &self.target, &mut records, &mut skipped);
                }
            }
            // No collision pass runs: distinct installs of one Steam source carry
            // distinct AppIDs, so the keys leave this seam unique by construction,
            // and friendly-name duplicates are by design (`025-L3`). The
            // `(slug, install_path)` sort below is all the ordering there is.
        }

        // §7's one error path, its three clauses spelled out: at least one root
        // was selected, zero games were emitted, and at least one file was
        // skipped as unreadable or malformed. The message names the selected
        // roots and the count of skipped files and nothing else — no contents.
        if !self.roots.is_empty() && records.is_empty() && skipped > 0 {
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

        records.sort_by(|a, b| {
            (&a.game.slug, &a.game.install_path).cmp(&(&b.game.slug, &b.game.install_path))
        });
        Ok(records)
    }
}

impl<T: SteamTarget> DiscoveryPlugin for SteamCore<T> {
    fn name(&self) -> &'static str {
        T::NAME
    }

    /// `discover()` is exactly the row projection of [`Self::scan`], order
    /// included (design 017, §Discovery plumbing): the trait, the row and the
    /// pinned renderings are untouched (`017-L5`).
    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
        Ok(self.scan()?.into_iter().map(|record| record.game).collect())
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
/// adding one record per surviving manifest — the row beside the `library`
/// scanned and the `root` that owns it (design 017, §Discovery plumbing) —
/// and counting every skipped file.
fn scan_library<T: SteamTarget>(
    library: &Path,
    root: &Path,
    target: &T,
    records: &mut Vec<ScanRecord>,
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
            // A row whose identity cannot be built is a refusal of this file, so
            // it counts here too (ADR-025): count-only, never echoed per row.
            Some(manifest) => match discovered_game(library, &manifest, target) {
                Some(game) => records.push(ScanRecord {
                    game,
                    root: root.to_path_buf(),
                    library: library.to_path_buf(),
                }),
                None => *skipped += 1,
            },
            None => *skipped += 1,
        }
    }
}

/// Builds the emitted row for `manifest` in `library` (Chosen design §5, A2).
/// The `platform` field is the only one the target changes (§2).
///
/// ADR-025 makes this the plugin's identity gate, and the gate is bound-only: the
/// slug is `Source::Steam.lookup_key(&appid)` — the one spelling constructor — and
/// the only thing refused here is a key past
/// [`bindforge_core::MAX_DISCOVERY_SLUG_OCTETS`]. The shared grammar is *not*
/// checked, because `steam/123` fails it by design (`025-L3`); the appid read in
/// [`parse_manifest`] already caps the key far inside the bound, so this check is
/// defence in depth. `None` is the count-only refusal: the caller counts the file
/// into `skipped` and emits nothing, and no invented key (a file name's number, a
/// name-derived spelling) ever becomes an identity. Length is never truncated.
fn discovered_game<T: SteamTarget>(
    library: &Path,
    manifest: &AppManifest,
    target: &T,
) -> Option<DiscoveredGame> {
    let slug = Source::Steam.lookup_key(&manifest.appid);
    if slug.len() > MAX_DISCOVERY_SLUG_OCTETS {
        return None;
    }
    Some(DiscoveredGame {
        friendly_name: friendly_name(&manifest.display_name),
        slug,
        display_name: manifest.display_name.clone(),
        // Emitted whether or not the directory exists; `library` is canonical,
        // and `install_dir` is validated by `parse_manifest`.
        install_path: library
            .join("steamapps")
            .join("common")
            .join(&manifest.install_dir),
        platform: target.platform(),
        codec_id: None,
    })
}

/// Extracts the display name, install directory and appid of one
/// `appmanifest_*.acf`.
///
/// `None` means "skip this manifest": no `AppState` section, a parse error, no
/// usable `appid`, an unusable `installdir`, or no name and no install directory
/// to fall back on. The `installdir` rule set is `T`'s (§2, §6); everything else
/// is shared. A missing or non-numeric appid is the same skip-and-count outcome
/// as an unusable `installdir`: a row whose identity the ACF cannot name is
/// refused, never given a name-derived or file-name-derived spelling (`025-C1`).
pub fn parse_manifest(text: &str, target: &impl SteamTarget) -> Option<AppManifest> {
    let document = keyvalues::parse(text).ok()?;
    let app_state = document.get_section("appstate")?;
    // `get_str` is the reader's case-insensitive lookup, so `appid`, `AppId` and
    // `APPID` all resolve (A3): no exact-case read is needed.
    let appid = app_state.get_str("appid").unwrap_or_default().trim();
    if !usable_appid(appid) {
        return None;
    }
    let install_dir = app_state.get_str("installdir")?;
    if !usable_install_dir(install_dir, target.install_dir_rejects_backslash()) {
        return None;
    }
    // `display_name` needs to be non-empty because the friendly name comes from it
    // (ADR-022), so an absent or empty `name` falls back to `installdir`.
    let name = app_state.get_str("name").unwrap_or_default().trim();
    let display_name = if name.is_empty() { install_dir } else { name };
    Some(AppManifest {
        appid: appid.to_string(),
        display_name: display_name.to_string(),
        install_dir: install_dir.to_string(),
    })
}

/// Whether `appid` is an external id a lookup key may be composed from: 1–20
/// ASCII digits, already trimmed. An empty, whitespace-only, non-numeric,
/// `/`-bearing or over-long value names no install, and the number in
/// `appmanifest_<n>.acf` is never a fallback (`025-C1`).
pub fn usable_appid(appid: &str) -> bool {
    !appid.is_empty()
        && appid.len() <= MAX_APPID_DIGITS
        && appid.bytes().all(|byte| byte.is_ascii_digit())
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

#[cfg(test)]
mod tests {
    //! Unit tests for the two private seams of the emission path, over
    //! hand-built [`AppManifest`]s: the appid rule the reduction applies, and the
    //! bound-only identity gate in `discovered_game`. The bound skip is reachable
    //! here and nowhere else — the appid rule caps a real key far inside
    //! `025-B1` — so this in-module construction is the only way to pin it
    //! (design 025 Test strategy #4: defence in depth). The end-to-end suites own
    //! the row shape and the skip counting.
    use super::*;
    use crate::steam_linux::LinuxTarget;

    /// One reduced manifest: the appid is the identity, the other two fields are
    /// constants the row carries unchanged.
    fn manifest(appid: &str) -> AppManifest {
        AppManifest {
            appid: appid.to_string(),
            display_name: "Example   Game".to_string(),
            install_dir: "Example Game".to_string(),
        }
    }

    fn row(appid: &str) -> Option<DiscoveredGame> {
        discovered_game(Path::new("/lib"), &manifest(appid), &LinuxTarget)
    }

    #[test]
    fn the_appid_rule_admits_one_to_twenty_ascii_digits_only() {
        assert!(usable_appid("1"), "one digit is a key");
        assert!(
            usable_appid(&"1".repeat(20)),
            "the twentieth digit is admitted"
        );
        assert!(!usable_appid(""), "an empty value names nothing");
        assert!(!usable_appid(&"1".repeat(21)), "21 digits is not an appid");
        assert!(!usable_appid("900o1"), "a non-digit is refused");
        assert!(!usable_appid("900/1"), "a separator is refused");
        assert!(
            !usable_appid(" 900001"),
            "an untrimmed space is refused here"
        );
    }

    #[test]
    fn the_row_composes_the_lookup_key_and_the_normalized_friendly_name() {
        let game = row("900001").expect("an ordinary appid builds a row");
        assert_eq!(
            game.slug, "steam/900001",
            "the identity is the composed key, never a spelling built beside the constructor"
        );
        assert_eq!(
            game.friendly_name, "Example Game",
            "the row's friendly name is the display name's whitespace normalization"
        );
        assert_eq!(
            game.display_name, "Example   Game",
            "the display name travels raw"
        );
        assert_eq!(
            game.install_path,
            PathBuf::from("/lib/steamapps/common/Example Game"),
            "the install path is built from the install directory, not the identity"
        );
    }

    #[test]
    fn a_grammar_invalid_but_in_bound_key_still_builds_its_row() {
        // `steam/1` fails the shared product-slug grammar and is emitted all the
        // same: the identity gate is bound-only (`025-L3`), where the retired
        // grammar check refused exactly this row.
        let game = row("1").expect("a grammar-invalid key is inside the bound");
        assert_eq!(game.slug, "steam/1");
        assert!(!bindforge_core::is_slug_name(&game.slug));
    }

    #[test]
    fn an_over_bound_lookup_key_is_refused_by_the_row_construction() {
        // One octet past `025-B1`, untruncated, and the row is refused outright.
        let over_bound = "9".repeat(MAX_DISCOVERY_SLUG_OCTETS - "steam/".len() + 1);
        assert_eq!(
            Source::Steam.lookup_key(&over_bound).len(),
            MAX_DISCOVERY_SLUG_OCTETS + 1
        );
        assert!(
            row(&over_bound).is_none(),
            "the bound-only gate refuses an over-long key rather than truncating it"
        );
    }

    #[test]
    fn a_key_exactly_on_the_bound_builds_its_row() {
        let at_bound = "8".repeat(MAX_DISCOVERY_SLUG_OCTETS - "steam/".len());
        assert_eq!(
            Source::Steam.lookup_key(&at_bound).len(),
            MAX_DISCOVERY_SLUG_OCTETS
        );
        assert!(
            row(&at_bound).is_some(),
            "the bound is inclusive: exactly `025-B1` octets survive"
        );
    }
}
