//! Shared helpers for the `crates/discovery` suites that need a Steam root they can
//! point a plugin at (repo practice: integration tests use temp directories and clean
//! up after themselves).
//!
//! This is not a test target of its own: cargo compiles `tests/*.rs`, and each suite
//! pulls this module in with `mod common;`.
//!
//! # Why the module-level `allow(dead_code)` stays
//!
//! Each test target compiles this module on its own and uses a different subset of
//! these helpers, so every helper is dead code in some target — the same reason
//! `crates/core/tests/common/mod.rs` carries the attribute.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// One Steam root in the system temp directory, removed when it goes out of scope.
///
/// The layout is the one the checked-in fixtures under `tests/fixtures/steam/` use:
/// `<root>/steamapps/appmanifest_<appid>.acf`, with a `steamapps/libraryfolders.vdf`
/// only when the caller wants a secondary library beside the root. A root with no
/// `libraryfolders.vdf` still yields the root itself as a library.
pub struct SteamRoot {
    path: PathBuf,
}

impl SteamRoot {
    /// Creates `<label>`-tagged empty root; `with_secondary` also stands up a
    /// `secondary/` library that the root's `libraryfolders.vdf` names.
    pub fn new(label: &str, with_secondary: bool) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "bindforge-discovery-{label}-{}-{nonce}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("steamapps"))
            .unwrap_or_else(|err| panic!("creating {}: {err}", path.display()));
        let root = Self { path };
        if with_secondary {
            fs::create_dir_all(root.path.join("secondary").join("steamapps"))
                .unwrap_or_else(|err| panic!("creating secondary library: {err}"));
            fs::write(
                root.path.join("steamapps").join("libraryfolders.vdf"),
                "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"..\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"../secondary\"\n\t}\n}\n",
            )
            .expect("libraryfolders.vdf");
        }
        root
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes one `appmanifest_<appid>.acf` into the root library: `name` is written
    /// only when given (so `None` is the name-absent case and `Some("")` the
    /// name-empty case), and `installdir` is the single path component the plugin
    /// appends to `<library>/steamapps/common/`. The `appid` entry is spelled in its
    /// shipped lowercase form; [`SteamRoot::acf_keyed`] is the ADR-025 seam that
    /// controls the entry itself.
    pub fn acf(&self, appid: &str, library: &str, name: Option<&str>, installdir: &str) {
        self.acf_keyed(appid, library, name, installdir, Some(("appid", appid)));
    }

    /// Writes one `appmanifest_<file_id>.acf` with full control over the `appid`
    /// entry, which ADR-025 makes the row's identity source: `appid` is the
    /// `(key, value)` pair as spelled in the ACF when `Some` — so key casing
    /// (`AppId`, `APPID`) and refused values (empty, non-numeric, an embedded `/`,
    /// 21 digits) are all expressible — and the entry is omitted entirely when
    /// `None`, which is the no-`appid` refusal case the emission seam must count and
    /// drop. The file name is spelled from `file_id` independently of the entry, so
    /// a suite can pin that the number in `appmanifest_<n>.acf` is never read as an
    /// identity fallback (`025-C1`, the Options' refused filename fallback).
    pub fn acf_keyed(
        &self,
        file_id: &str,
        library: &str,
        name: Option<&str>,
        installdir: &str,
        appid: Option<(&str, &str)>,
    ) {
        let mut text = String::from("\"AppState\"\n{\n");
        if let Some((key, value)) = appid {
            text.push_str(&format!("\t\"{key}\"\t\t\"{value}\"\n"));
        }
        text.push_str("\t\"Universe\"\t\t\"1\"\n");
        if let Some(name) = name {
            text.push_str(&format!("\t\"name\"\t\t\"{name}\"\n"));
        }
        text.push_str(&format!(
            "\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"{installdir}\"\n\t\"SizeOnDisk\"\t\t\"1000000\"\n}}\n"
        ));

        let library_dir = if library.is_empty() {
            self.path.clone()
        } else {
            self.path.join(library)
        };
        let dir = library_dir.join("steamapps");
        fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("creating {}: {err}", dir.display()));
        let file = dir.join(format!("appmanifest_{file_id}.acf"));
        fs::write(&file, text).unwrap_or_else(|err| panic!("writing {}: {err}", file.display()));
    }

    /// This root canonicalized — the plugin's path rule emits canonical paths.
    pub fn canonical(&self) -> PathBuf {
        fs::canonicalize(&self.path).unwrap_or_else(|err| panic!("canonicalizing root: {err}"))
    }
}

impl Drop for SteamRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// A display name whose derived slug is `head + 1 + tail` octets: `head` letters, one
/// separator, `tail` letters.
pub fn letters(head: usize, tail: usize) -> String {
    format!("{} {}", "k".repeat(head), "k".repeat(tail))
}

/// The same name with `!` as its separator: it derives the identical slug and sorts
/// after the space-separated spelling in byte order.
pub fn letters_exclaim(head: usize, tail: usize) -> String {
    format!("{}!{}", "k".repeat(head), "k".repeat(tail))
}
