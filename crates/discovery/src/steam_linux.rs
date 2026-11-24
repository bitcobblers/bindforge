//! The Linux Steam discovery plugin and its host policy (Chosen design §2-§7).
//!
//! The plugin reads library folders from `<root>/steamapps/libraryfolders.vdf`,
//! always includes the root itself as a library, enumerates the
//! `appmanifest_*.acf` files directly inside each library's `steamapps/`, and
//! emits one [`DiscoveredGame`] per surviving manifest. Each manifest's
//! `AppState.appid` is read and composed into the row's lookup key
//! (`025-C1`): a manifest without a usable appid is refused like one with an
//! unusable `installdir`.
//!
//! The scan itself lives once in [`crate::steam_common`]; this module is the
//! Linux `pub` surface over it plus the Linux-only host probe (T-002 Chosen
//! design §1).

use crate::game::{DiscoveredGame, Platform};
use crate::plugin::DiscoveryPlugin;
use crate::registry::DiscoveryError;
use crate::steam_common::{self, SteamCore, SteamTarget};
use std::path::{Path, PathBuf};

pub use crate::steam_common::AppManifest;

/// The Linux platform parameterization of the shared scan core (T-002 Chosen
/// design §1): the tag it emits, and the rule set that stays bit-for-bit as it
/// was before the refactor (§6).
#[derive(Debug, Clone, Copy)]
pub struct LinuxTarget;

impl SteamTarget for LinuxTarget {
    const NAME: &'static str = "steam-linux";

    fn platform(&self) -> Platform {
        Platform::SteamLinux
    }

    fn install_dir_rejects_backslash(&self) -> bool {
        // On Linux `\` is an ordinary filename byte (T-002 A7), so the value
        // is not a nested path and the clause does not apply.
        false
    }
}

impl steam_common::sealed::Sealed for LinuxTarget {}

/// A Steam Linux discovery plugin over a fixed set of Steam roots.
///
/// The roots are held on the plugin because `DiscoveryPlugin::discover` takes
/// no arguments (A17); `from_roots` is the seam tests and `--steam-root` use.
/// After T-002 the roots live on the shared [`SteamCore`]; the `pub` surface is
/// unchanged.
#[derive(Debug, Clone)]
pub struct SteamLinuxPlugin {
    core: SteamCore<LinuxTarget>,
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
            core: SteamCore::with_roots(roots, LinuxTarget),
        }
    }

    /// The scan with each row's owning root and library beside it (design
    /// 017, §Discovery plumbing): a thin inherent wrapper over the shared
    /// core. The `DiscoveryPlugin` trait is not touched — third-party plugins
    /// keep `discover()` and simply have no bindings face this run.
    pub fn scan_records(&self) -> Result<Vec<crate::game::ScanRecord>, DiscoveryError> {
        self.core.scan()
    }

    /// The selected root set as the core holds it — canonicalized,
    /// existence-filtered, byte order (§Discovery plumbing).
    pub fn roots(&self) -> &[PathBuf] {
        self.core.roots()
    }
}

impl DiscoveryPlugin for SteamLinuxPlugin {
    fn name(&self) -> &'static str {
        self.core.name()
    }

    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
        self.core.discover()
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
    candidates.push(match steam_common::absolute_env(env, "XDG_DATA_HOME") {
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
    let snap = match steam_common::absolute_env(env, "SNAP_USER_DATA") {
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
    steam_common::libraries_in_root_with_skips(root, &mut 0)
}

/// Extracts the display name and install directory of one `appmanifest_*.acf`
/// under the Linux rule set (Chosen design §5; T-002 §2 — a thin wrapper over
/// the shared core).
///
/// `None` means "skip this manifest": no `AppState` section, a parse error, an
/// unusable `installdir`, or no name and no install directory to fall back on.
pub fn parse_manifest(text: &str) -> Option<AppManifest> {
    steam_common::parse_manifest(text, &LinuxTarget)
}
