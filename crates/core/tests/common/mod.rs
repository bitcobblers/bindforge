//! Shared helpers for the `bindforge-core` conformance suites.
//!
//! This is not a test target of its own: cargo compiles `tests/*.rs` and
//! `tests/*/main.rs`, and each suite below pulls this module in with
//! `mod common;`.
//!
//! # The `bindforge-core` surface these suites assume
//!
//! The proposal names the guards it pins — `resolve_chain`, `MAX_CHAIN_DEPTH`,
//! `MAX_DOCUMENT_BYTES`, `MAX_NESTING_DEPTH` and the `Cycle` / `ChainTooDeep`
//! variants (Chosen design → Adversarial guard, and the budget `schemas/README.md`
//! publishes for third-party readers) — plus the harness behavior
//! (Chosen design → Formats contract). The remaining names here (`parse`,
//! `parse_file`, `parse_reader`, `serialize`, `supported_majors`,
//! `load_reach_tables`, `hand_matches_file_name`, `Hand`, `ReachClass`,
//! `CoreError::UnsupportedVersion`) are the smallest surface that can exercise a
//! promised behavior: `schemas/README.md` names `parse_file` and `parse_reader`
//! as the readers that apply the size cap themselves, and
//! `schemas/reach-classes/v1/spec.md` requires a table's `hand` entry to match
//! its file name — which is what `hand_matches_file_name` names at the code
//! level. A loaded table's classes are compared by their `id`, because the
//! contract's `classes` entry lists reach-class ids. Changing one of these names
//! means changing the proposal first, not the implementation.
//!
//! # Why the module-level `allow(dead_code)` stays
//!
//! Each test target compiles this module on its own, and every target uses a
//! different subset of these helpers, so every helper is dead code in some
//! target. Scoping the allow to individual items would mean annotating all of
//! them; a helper that stops being used anywhere is caught by the suites'
//! `proves:` comments, not by this attribute.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bindforge_core::CoreError;

/// The workspace root — the directory holding `Cargo.toml`, `schemas/` and
/// `.github/`, derived from this crate's package root.
///
/// proves: Formats contract → Harness — paths resolve through
/// `env!("CARGO_MANIFEST_DIR")`, so every suite finds the contract tree and the
/// CI definitions whatever the test binary's cwd is (A8, A26). That is the
/// code-level half of the one-gate promise in Test strategy #5: the cargo gate
/// needs no cwd convention, no network and no database.
pub fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .expect("crates/core sits directly under the workspace root")
        .to_path_buf()
}

/// The `schemas/` contract tree at the workspace root.
pub fn schemas_root() -> PathBuf {
    repo_root().join("schemas")
}

/// One `schemas/<format>/v<N>/` directory found by the walk.
pub struct FormatDir {
    pub format: String,
    pub major: String,
    pub path: PathBuf,
}

/// Every `schemas/<format>/v<N>/` directory under the workspace's contract
/// tree, sorted by path.
pub fn format_dirs() -> Vec<FormatDir> {
    format_dirs_under(&schemas_root())
}

/// Every `schemas/<format>/v<N>/` directory under an arbitrary root. The walk
/// takes the root as a parameter so a rule about the walk — the zero-fixture
/// rule below — can be exercised against a scratch tree as well as against the
/// real contract tree.
pub fn format_dirs_under(root: &Path) -> Vec<FormatDir> {
    let mut dirs = Vec::new();
    for format in read_dir_sorted(root) {
        if !format.is_dir() {
            continue;
        }
        for version in read_dir_sorted(&format) {
            let major = file_name(&version);
            if !version.is_dir() || !is_major_dir(&major) {
                continue;
            }
            dirs.push(FormatDir {
                format: file_name(&format),
                major,
                path: version,
            });
        }
    }
    dirs.sort_by(|a, b| a.path.cmp(&b.path));
    dirs
}

/// The majors the contract tree ships for one format — the majors *that format's*
/// reader supports. The version gate is per-format, because a major exists per
/// format: `reach-classes` carries `v1` and `v2` (design 012's Decision One),
/// every other shipped format carries `v1` alone. The corpus is what makes a
/// major supported (`schemas/README.md`, design 012's A2), and the crate-wide
/// `supported_majors()` is the union across formats, so a suite that pins a
/// refusal "naming the supported majors" names this list, not the union.
pub fn format_majors(format: &str) -> Vec<String> {
    format_dirs()
        .into_iter()
        .filter(|dir| dir.format == format)
        .map(|dir| dir.major)
        .collect()
}

/// One named `schemas/<format>/<major>/` directory; fails with the tree it
/// actually found when the version directory is missing.
pub fn version_dir(format: &str, major: &str) -> FormatDir {
    let path = schemas_root().join(format).join(major);
    assert!(
        path.is_dir(),
        "the contract tree must ship `{format}/{major}` at {}; the walk saw {:?}",
        schemas_root().display(),
        read_dir_sorted(&schemas_root())
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
    );
    FormatDir {
        format: format.to_string(),
        major: major.to_string(),
        path,
    }
}

/// The `fixtures/valid` or `fixtures/invalid` files of one version directory,
/// sorted. A missing directory yields an empty list, which is what the
/// zero-fixture assertion in `conformance.rs` is there to catch.
pub fn fixtures(dir: &FormatDir, kind: &str) -> Vec<PathBuf> {
    let fixtures = dir.path.join("fixtures").join(kind);
    if !fixtures.is_dir() {
        return Vec::new();
    }
    read_dir_sorted(&fixtures)
        .into_iter()
        .filter(|path| path.is_file())
        .collect()
}

/// The zero-fixture rule of the harness (Chosen design → Formats contract,
/// Test strategy #1): a `schemas/*/v*/` directory yielding zero valid or zero
/// invalid fixtures fails the run, because a file's presence in
/// `fixtures/valid/` is the definition of "supported". Returns a description of
/// the first directory that breaks the rule.
pub fn zero_fixture_violation(dirs: &[FormatDir]) -> Option<String> {
    for dir in dirs {
        let version = format!("{}/{}", dir.format, dir.major);
        if fixtures(dir, "valid").is_empty() {
            return Some(format!("{version} has no fixture in fixtures/valid/"));
        }
        if fixtures(dir, "invalid").is_empty() {
            return Some(format!("{version} has no fixture in fixtures/invalid/"));
        }
    }
    None
}

/// Reads one fixture file from the contract tree.
pub fn fixture_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()))
}

/// A scratch directory the caller can point the contract readers at, removed
/// when it goes out of scope — so a suite can exercise the rules that the
/// checked-in contract tree cannot carry (an empty fixture directory, a table
/// file whose name contradicts its `hand` entry, a document over the input
/// budget). Integration-style: temp directory, cleaned up afterwards.
pub struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "bindforge-core-{label}-{}-{nonce}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("creating scratch dir {}: {err}", path.display()));
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Creates a subdirectory of this scratch tree.
    pub fn make_dir(&self, relative: &str) -> PathBuf {
        let path = self.path.join(relative);
        fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("creating directory {}: {err}", path.display()));
        path
    }

    /// Writes one file into this scratch tree and returns its path. The parent
    /// is created first, so a suite can lay out a nested fixture corpus in any
    /// order; a bare `fs::write` fails at ENOENT on a directory the suite has
    /// not stood up yet, which is how the zero-fixture rule's trigger test
    /// panicked before it could assert anything.
    pub fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.path.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("creating {}: {err}", parent.display()));
        }
        fs::write(&path, text).unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
        path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// "Rejected" as the harness means it: the reference parser refuses the
/// document, or chain resolution does for a profile — the guards live in
/// `resolve_chain` (Adversarial guard), and the invalid-fixture corpus is the
/// contract for that behavior.
pub enum Rejection {
    Parse(CoreError),
    Chain(CoreError),
}

impl Rejection {
    pub fn detail(&self) -> String {
        match self {
            Rejection::Parse(err) | Rejection::Chain(err) => err.to_string(),
        }
    }
}

pub fn rejection(text: &str) -> Result<(), Rejection> {
    let document = bindforge_core::parse(text).map_err(Rejection::Parse)?;
    if let Some(profile) = document.profile() {
        bindforge_core::resolve_chain(profile).map_err(Rejection::Chain)?;
    }
    Ok(())
}

/// `ancestor-1 .. ancestor-<links>` — the chain the boundary tests write
/// against `bindforge_core::MAX_CHAIN_DEPTH` rather than a literal (Test
/// strategy #3, revision log: "#3 pinned to the constant, not the literal 32").
pub fn ancestor_chain(links: usize) -> Vec<String> {
    (1..=links)
        .map(|index| format!("ancestor-{index}"))
        .collect()
}

/// A profile document, its version line spelled out by the caller so the
/// version gate's inputs are readable in the test. `version: None` is the
/// document that declares no version at all.
///
/// The chain is written inline as one profile's ancestor list so a depth
/// boundary is a single document; ADR-014 owns how chains are otherwise
/// expressed.
pub fn profile_doc_declaring(id: &str, version: Option<&str>, derives: &[String]) -> String {
    let list = derives
        .iter()
        .map(|ancestor| format!("\"{ancestor}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let version_line = match version {
        Some(version) => format!("version = \"{version}\"\n"),
        None => String::new(),
    };
    format!("format = \"profile\"\n{version_line}id = \"{id}\"\nderives = [{list}]\n")
}

/// A `v1` profile document — the shape the guards in this design read.
pub fn profile_doc(id: &str, derives: &[String]) -> String {
    profile_doc_declaring(id, Some("v1"), derives)
}

/// Whether a document text declares a version at all — used to identify the
/// missing-version case inside the invalid-fixture corpus without a TOML
/// parser.
pub fn declares_version(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        !line.starts_with('#') && line.starts_with("version")
    })
}

/// The value of a one-line `key = "value"` entry, read textually. This is not a
/// TOML reader — it is how a suite names the entry a contract rule is about
/// (the reach-classes `hand` entry) without duplicating the grammar.
pub fn string_entry(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.starts_with('#') && line.starts_with(&prefix))?;
    let open = line.find('"')?;
    let close = line.rfind('"')?;
    if close <= open {
        return None;
    }
    Some(line[open + 1..close].to_string())
}

/// The values of a one-line `key = ["a", "b"]` entry, read textually — same
/// caveat as `string_entry`: it names what the contract file lists, it is not a
/// second parser.
pub fn string_array(text: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = [");
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.starts_with('#') && line.starts_with(&prefix))?;
    let open = line.find('[')?;
    let close = line.rfind(']')?;
    let inner = line[open + 1..close].trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    Some(
        inner
            .split(',')
            .map(|value| value.trim().trim_matches('"').to_string())
            .collect(),
    )
}

/// The quoted elements of a one-line `key = ["a", "b"]` entry, split only on the
/// commas that sit *outside* quotes. ADR-012's `keys` arrays need this and
/// `string_array` cannot give it: a `reach-classes/v2` element is one
/// `"<input-id> <class-id>"` pair, and the closed keyboard member set carries the
/// `,` glyph, so a naive split cuts `keyboard:, far` into two elements. Same
/// caveat as `string_entry`: this names what the contract file lists, it is not a
/// second TOML parser. Escapes stay exactly as the file writes them — `\\` for the
/// backslash glyph (A3) — and the caller unescapes whatever it compares.
pub fn quoted_array(text: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = [");
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.starts_with('#') && line.starts_with(&prefix))?;
    let open = line.find('[')?;
    let mut elements: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut inside_quotes = false;
    let mut escaped = false;
    let mut element_open = false;
    for ch in line[open + 1..].chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if inside_quotes && ch == '\\' {
            current.push(ch);
            escaped = true;
            continue;
        }
        if ch == '"' {
            inside_quotes = !inside_quotes;
            element_open = true;
            continue;
        }
        if inside_quotes {
            current.push(ch);
            continue;
        }
        if ch == ']' {
            break;
        }
        if ch == ',' && element_open {
            elements.push(std::mem::take(&mut current));
            element_open = false;
        }
    }
    if element_open {
        elements.push(current);
    }
    Some(elements)
}

fn read_dir_sorted(path: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(path)
        .unwrap_or_else(|err| panic!("reading directory {}: {err}", path.display()));
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| {
            let entry =
                entry.unwrap_or_else(|err| panic!("reading directory {}: {err}", path.display()));
            entry.path()
        })
        .collect();
    paths.sort();
    paths
}

fn is_major_dir(name: &str) -> bool {
    match name.strip_prefix('v') {
        Some(digits) => !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()),
        None => false,
    }
}

/// proves: Formats contract → versioning rule — fixture file names are
/// lowercase ASCII, `-`-separated.
pub fn is_fixture_name(name: &str) -> bool {
    let stem = match name.strip_suffix(".toml") {
        Some(stem) => stem,
        None => return false,
    };
    if stem.is_empty() {
        return false;
    }
    stem.split('-').all(|segment| {
        !segment.is_empty()
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| panic!("{} has no usable file name", path.display()))
        .to_string()
}
