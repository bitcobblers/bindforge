//! The Windows Steam discovery plugin and its host policy (T-002 Chosen
//! design §1, §3–§7).
//!
//! The scan after root selection is the shared [`crate::steam_common`] core;
//! this module adds only the Windows platform tag, the extra `installdir`
//! `\`-refusal (§6), the pure registry-plus-environment root probe (§5) and the
//! `#[cfg(windows)]` registry reader (§3). The plugin type, its policy and the
//! seam are compiled and tested on every host; only the real registry read is
//! Windows-gated, so `from_host()` on a non-Windows host yields no roots —
//! there is no Windows Steam install to find there (§3).

use crate::game::{DiscoveredGame, Platform};
use crate::plugin::DiscoveryPlugin;
use crate::registry::DiscoveryError;
use crate::steam_common::{self, SteamCore, SteamTarget};
use std::path::PathBuf;

pub use crate::steam_common::AppManifest;

/// The value name under `HKCU\Software\Valve\Steam` (T-002 A3).
const STEAM_PATH_VALUE: &str = "SteamPath";

/// The value name under `HKLM\SOFTWARE\Valve\Steam` (T-002 A4).
const INSTALL_PATH_VALUE: &str = "InstallPath";

/// The pinned spelling of the 32-bit-program-files environment variable
/// (absent on 32-bit Windows hosts, A6, A21).
const PROGRAM_FILES_X86: &str = "ProgramFiles(x86)";

/// The pinned spelling of the program-files environment variable (A21).
const PROGRAM_FILES: &str = "ProgramFiles";

/// The `(key, view)` pairs the probe reads (T-002 Chosen design §1, §4).
///
/// Each variant fixes exactly one registry key opened with one registry view,
/// so no call site can invent a fourth pair: the fake in a test and the real
/// reader answer the same closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryKey {
    /// `HKCU\Software\Valve\Steam`, opened plainly: `HKCU\Software` is a
    /// shared, non-redirected key (A5).
    CurrentUserValveSteam,
    /// `HKLM\SOFTWARE\Valve\Steam` opened with `KEY_READ | KEY_WOW64_32KEY`
    /// — the view that reaches the 32-bit install on 64-bit Windows (A4).
    LocalMachineValveSteam32,
    /// `HKLM\SOFTWARE\Valve\Steam` opened with `KEY_READ | KEY_WOW64_64KEY`
    /// (A4).
    LocalMachineValveSteam64,
}

/// The Windows platform parameterization of the shared scan core (T-002
/// Chosen design §1, §6).
#[derive(Debug, Clone, Copy)]
pub struct WindowsTarget;

impl SteamTarget for WindowsTarget {
    const NAME: &'static str = "steam-windows";

    fn platform(&self) -> Platform {
        Platform::SteamWindows
    }

    fn install_dir_rejects_backslash(&self) -> bool {
        // On Windows `\` is the path separator (A7), so an `installdir`
        // containing it would silently become a nested path inside the
        // emitted `install_path` (§6).
        true
    }
}

impl steam_common::sealed::Sealed for WindowsTarget {}

/// A Steam Windows discovery plugin over a fixed set of Steam roots.
///
/// Like the Linux plugin, the roots are held on the plugin (here on its
/// [`SteamCore`]) because `DiscoveryPlugin::discover` takes no arguments;
/// `from_roots` and `from_registry_and_roots` are the seams tests and
/// `--steam-root` use, and `from_host()` is the only function that touches a
/// real registry (§3).
#[derive(Debug, Clone)]
pub struct SteamWindowsPlugin {
    core: SteamCore<WindowsTarget>,
}

impl SteamWindowsPlugin {
    /// Probes this host's registry and environment for Steam installations
    /// through the §4 seam (Chosen design §3).
    #[cfg(windows)]
    pub fn from_host() -> Self {
        Self::from_registry_and_roots(&registry_string_value, &|key| std::env::var(key).ok())
    }

    /// There is no Windows Steam install on a non-Windows host, so the probe
    /// yields no roots and the plugin's scan is the `Ok(vec![])` "not
    /// installed here" rule (Chosen design §3, §8).
    #[cfg(not(windows))]
    pub fn from_host() -> Self {
        Self::from_roots(Vec::new())
    }

    /// Scans caller-supplied roots, each taken through the canonical path rule.
    pub fn from_roots(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            core: SteamCore::with_roots(roots, WindowsTarget),
        }
    }

    /// Builds the plugin from the candidate list `host_candidates` selects
    /// through the registry and environment seams, then pipes it through the
    /// shared canonical-path rule exactly as the Linux `from_host` does with
    /// its candidate list (§3, §5). A root that cannot be canonicalized or
    /// lacks a `steamapps/` is already dropped there, so there is no separate
    /// "is Steam installed here" test.
    pub fn from_registry_and_roots(
        reg: &dyn Fn(RegistryKey, &str) -> Option<String>,
        env: &dyn Fn(&str) -> Option<String>,
    ) -> Self {
        Self::from_roots(host_candidates(reg, env))
    }
}

impl DiscoveryPlugin for SteamWindowsPlugin {
    fn name(&self) -> &'static str {
        self.core.name()
    }

    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
        self.core.discover()
    }
}

/// The ordered Steam root candidates from the registry and environment
/// (T-002 Chosen design §5).
///
/// Pure: the registry and the environment are lookup functions, so the list is
/// testable without a registry, a filesystem or the process environment. The
/// values arrive in mixed case and either slash spelling; no normalization
/// happens here — the canonical-path rule turns both spellings into one
/// comparable form, so a value that duplicates an env candidate is appended
/// verbatim and collapsed there.
pub fn host_candidates(
    reg: &dyn Fn(RegistryKey, &str) -> Option<String>,
    env: &dyn Fn(&str) -> Option<String>,
) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. The per-user key first — it tracks the Steam client the user
    //    actually runs (A3, A5) — then the machine key's two views, 32-bit
    //    before 64-bit (A4). Every key is probed even after a hit; there is
    //    no early stop, and a refused value is simply no candidate (§4).
    for (key, value_name) in [
        (RegistryKey::CurrentUserValveSteam, STEAM_PATH_VALUE),
        (RegistryKey::LocalMachineValveSteam32, INSTALL_PATH_VALUE),
        (RegistryKey::LocalMachineValveSteam64, INSTALL_PATH_VALUE),
    ] {
        if let Some(value) = reg(key, value_name) {
            candidates.push(PathBuf::from(value));
        }
    }

    // 2. The default-install folders, appended even when a registry candidate
    //    was found (§5): `%ProgramFiles(x86)%\Steam` — absent on 32-bit
    //    Windows (A6) — then `%ProgramFiles%\Steam`. A set-but-relative value
    //    is ignored by `absolute_env`.
    for name in [PROGRAM_FILES_X86, PROGRAM_FILES] {
        if let Some(dir) = steam_common::absolute_env(env, name) {
            candidates.push(dir.join("Steam"));
        }
    }

    candidates
}

/// Extracts the display name and install directory of one `appmanifest_*.acf`
/// under the Windows rule set (T-002 Chosen design §2, §6): the Linux reject
/// set plus the `\` refusal, and nothing wider.
pub fn parse_manifest(text: &str) -> Option<AppManifest> {
    steam_common::parse_manifest(text, &WindowsTarget)
}

/// Whether `installdir` passes the Windows rule set (T-002 Chosen design §6).
pub fn usable_install_dir(install_dir: &str) -> bool {
    steam_common::usable_install_dir(install_dir, WindowsTarget.install_dir_rejects_backslash())
}

/// The concrete Windows registry reader behind the §4 seam: the crate's only
/// `winreg` consumer, so `winreg` never appears in a non-Windows compile
/// (T-002 Chosen design §3, A11).
///
/// Reads exactly one `(key, value)` pair per call, each [`RegistryKey`] variant
/// fixing one (key, view) pair (§3). A value is reported only when the
/// registry stored a plain `REG_SZ` path (§4); every failure — absent key,
/// absent value, access denied, other `vtype`, non-UTF-16 data — is `None`.
/// Reading is the only thing that can fail, and a failure is "no candidate",
/// never an error.
#[cfg(windows)]
fn registry_string_value(key: RegistryKey, name: &str) -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, RegType,
    };

    // The logical key is opened with a view flag rather than a hardcoded
    // `Wow6432Node` spelling, which is wrong on 32-bit Windows (A4).
    let (hive, path, flags) = match key {
        // `HKCU\Software` is a shared, non-redirected key (A5): opened plainly.
        RegistryKey::CurrentUserValveSteam => (HKEY_CURRENT_USER, r"Software\Valve\Steam", None),
        RegistryKey::LocalMachineValveSteam32 => (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Valve\Steam",
            Some(KEY_READ | KEY_WOW64_32KEY),
        ),
        RegistryKey::LocalMachineValveSteam64 => (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Valve\Steam",
            Some(KEY_READ | KEY_WOW64_64KEY),
        ),
    };

    let hive = RegKey::predef(hive);
    let subkey = match flags {
        Some(flags) => hive.open_subkey_with_flags(path, flags).ok()?,
        None => hive.open_subkey(path).ok()?,
    };

    // The raw read makes the value type observable (A9); only a plain `REG_SZ`
    // path is usable — an unexpanded `REG_EXPAND_SZ` would carry literal
    // `%VAR%` text, and nothing wider is attempted (§4, A18).
    let value = subkey.get_raw_value(name).ok()?;
    if value.vtype != RegType::REG_SZ {
        return None;
    }

    // `REG_SZ` is UTF-16LE: an odd length or unpaired surrogates are a broken
    // value, and trailing NUL units are the C-string terminator, not content.
    if value.bytes.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = value
        .bytes
        .chunks(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let end = units
        .iter()
        .rposition(|unit| *unit != 0)
        .map_or(0, |index| index + 1);
    String::from_utf16(&units[..end]).ok()
}
