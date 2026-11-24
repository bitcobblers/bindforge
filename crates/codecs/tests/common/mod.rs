//! Shared helpers for the `crates/codecs` suites (ADR-017, design 017).
//!
//! This is not a test target of its own: cargo compiles `tests/*.rs`, and each
//! suite pulls this module in with `mod common;`.
//!
//! # Why the module-level `allow(dead_code)` stays
//!
//! Each test target compiles this module on its own and uses a different subset
//! of these helpers, so every helper is dead code in some target.
//!
//! # API surface this harness pins
//!
//! The suites read the crate through the names the proposal names:
//! `codec::{CodecFamily, Detection, Evidence, ExtractionRefusal,
//! UserCandidates}`, `context::GameContext`, `registry::{REGISTRY, claim,
//! claim_with, family_ids, is_known_family}`, `paths::{TreeKind, UserTree,
//! user_trees}`, `read::{open_capped, decode}`, `snapshot::{BindingSnapshot,
//! load_snapshot}` and `families::unreal_ini::UnrealIni`, with the budget
//! constants exported from the crate root and the crate depending on
//! `bindforge-core`, `discovery`, `serde` (dev: `serde_json`).
//! `registry::claim_with(families, ctx)` is the seam that runs the two claim
//! passes over a caller-provided family slice (`claim(ctx)` being the shipped
//! `REGISTRY` instance of it) — it is what lets the synthetic families pin
//! `017-C5`'s pass ordering without inventing appids (§The registry).
//! `GameContext`, `Detection` and `UserCandidates` are field-public so a suite
//! can build a fixture machine without a Steam install (§GameContext,
//! §Config-path resolvers).

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bindforge_codecs::codec::{Detection, ExtractionRefusal};
use bindforge_codecs::context::GameContext;
use bindforge_codecs::families::unreal_ini::UnrealIni;
use bindforge_codecs::registry::claim;
use bindforge_codecs::snapshot::{BindingSnapshot, load_snapshot};
use discovery::game::Platform;

/// A unique temp directory, removed when it goes out of scope (repo practice:
/// integration tests use temp directories and clean up after themselves).
pub struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bindforge-codecs-{label}-{nonce}-{nanos}"));
        fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("creating {}: {err}", path.display()));
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// This root canonicalized — the resolvers take canonical anchors
    /// (§Config-path resolvers), so every fixture comparison goes through this.
    pub fn canonical(&self) -> PathBuf {
        fs::canonicalize(&self.path).unwrap_or_else(|err| panic!("canonicalizing root: {err}"))
    }

    pub fn join(&self, relative: &str) -> PathBuf {
        self.path.join(relative)
    }

    /// Creates one directory (parents included), relative to the root.
    pub fn mkdir(&self, relative: &str) {
        let path = self.path.join(relative);
        fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("creating {}: {err}", path.display()));
    }

    /// Writes one file (parents created) with the given bytes, returning its path.
    pub fn write_bytes(&self, relative: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("creating {}: {err}", parent.display()));
        }
        fs::write(&path, bytes).unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
        path
    }

    pub fn write(&self, relative: &str, text: &str) -> PathBuf {
        self.write_bytes(relative, text.as_bytes())
    }

    /// Every regular file below `relative` (recursively), sorted — the byte
    /// snapshot the never-write posture is compared with.
    pub fn files_below(&self, relative: &str) -> Vec<(String, Vec<u8>)> {
        fn walk(dir: &Path, base: &Path, files: &mut Vec<(String, Vec<u8>)>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
                let path = entry.expect("readable entry").path();
                if path.is_dir() {
                    walk(&path, base, files);
                } else {
                    let name = path
                        .strip_prefix(base)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .into_owned();
                    files.push((name, fs::read(&path).expect("fixture file readable")));
                }
            }
        }
        let mut files = Vec::new();
        let dir = self.path.join(relative);
        if dir.is_dir() {
            walk(&dir, &dir, &mut files);
        }
        files.sort();
        files
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Canonicalize a checked-in fixture path (the resolvers take canonical anchors).
pub fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|err| panic!("canonicalizing {}: {err}", path.display()))
}

/// A checked-in fixture tree under `tests/fixtures/unreal/`, canonicalized.
pub fn fixture_tree(name: &str) -> PathBuf {
    canonical(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unreal")
            .join(name),
    )
}

/// A context anchored on the given roots/library for one appid — the shape
/// `GameContext::from_record` produces, built directly so the resolver and
/// family suites need no Steam install (§GameContext).
pub fn context(appid: &str, roots: Vec<PathBuf>, library: PathBuf) -> GameContext {
    GameContext {
        lookup_key: format!("steam/{appid}"),
        appid: appid.to_string(),
        install_path: library
            .join("steamapps")
            .join("common")
            .join("Example Game"),
        platform: Platform::SteamLinux,
        roots,
        library,
    }
}

/// KeyValues text naming the given libraries the way a `libraryfolders.vdf`
/// spells them (first entry the root itself, then each `path`).
pub fn library_folders(paths: &[PathBuf]) -> String {
    let mut text = String::from("\"libraryfolders\"\n{\n");
    for (index, path) in paths.iter().enumerate() {
        text.push_str(&format!(
            "\t\"{index}\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n",
            path.display()
        ));
    }
    text.push_str("}\n");
    text
}

/// A `[/Script/Engine.PlayerInput]` section carrying the given `Bindings=` lines.
pub fn player_input(bindings: &[&str]) -> String {
    let mut text = String::from("[/Script/Engine.PlayerInput]\n");
    for line in bindings {
        text.push_str(line);
        text.push('\n');
    }
    text
}

/// The shipped registry's claim for `ctx` (panics without one).
pub fn claim_ok(ctx: &GameContext) -> Detection {
    claim(ctx).unwrap_or_else(|| panic!("the shipped registry must claim this context"))
}

/// The `unreal-ini` claim for `ctx` (panics without one).
pub fn unreal_claim(ctx: &GameContext) -> Detection {
    UnrealIni
        .detect(ctx)
        .unwrap_or_else(|| panic!("unreal-ini must claim this context"))
}

/// Serializes snapshot documents, for byte-identity comparisons.
pub fn docs_json(docs: &[BindingSnapshot]) -> Vec<String> {
    docs.iter()
        .map(|doc| serde_json::to_string(doc).expect("a snapshot document serializes"))
        .collect()
}

/// Detects with `unreal-ini`, extracts, and serializes the documents.
pub fn extract_json(ctx: &GameContext) -> Vec<String> {
    let claim = unreal_claim(ctx);
    match UnrealIni.extract(&claim) {
        Ok(docs) => docs_json(&docs),
        Err(refusal) => panic!("extraction must not refuse here: {refusal:?}"),
    }
}

/// Runs `load_snapshot` over every `*.json` under a schema fixtures directory.
pub fn schema_fixture_dir(format: &str, version: &str, kind: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|crates| crates.parent())
        .expect("the workspace root above crates/")
        .join("schemas")
        .join(format)
        .join(version)
        .join("fixtures")
        .join(kind)
}

/// The JSON value of a serialized document.
pub fn doc_value(doc_json: &str) -> serde_json::Value {
    serde_json::from_str(doc_json).expect("a snapshot document is JSON")
}

/// The `[action, inputs…]` rows of one serialized snapshot document, verbatim.
pub fn entries_of(doc_json: &str) -> Vec<(String, Vec<String>)> {
    doc_value(doc_json)["entries"]
        .as_array()
        .expect("entries is an array")
        .iter()
        .map(|entry| {
            (
                entry["action"]
                    .as_str()
                    .expect("action is a string")
                    .to_string(),
                entry["inputs"]
                    .as_array()
                    .expect("inputs is an array")
                    .iter()
                    .map(|input| input.as_str().expect("input is a string").to_string())
                    .collect(),
            )
        })
        .collect()
}

/// All `[action, inputs…]` rows across the serialized documents, sorted.
pub fn all_entries(docs: &[String]) -> Vec<(String, Vec<String>)> {
    let mut rows: Vec<(String, Vec<String>)> =
        docs.iter().flat_map(|doc| entries_of(doc)).collect();
    rows.sort();
    rows
}

/// Asserts the rows are the contract's canonical form: actions sorted, each
/// entry's inputs deduplicated and sorted, files sorted (§Determinism).
pub fn assert_sorted_canonical(docs: &[String]) {
    for doc in docs {
        let rows = entries_of(doc);
        let mut sorted = rows.clone();
        sorted.sort();
        assert_eq!(
            rows, sorted,
            "entries must be sorted by action byte order: {doc}"
        );
        for (action, inputs) in &rows {
            let mut deduped: Vec<String> = inputs.clone();
            deduped.dedup();
            assert_eq!(inputs, &deduped, "inputs of {action} must be deduplicated");
            let mut sorted_inputs = inputs.clone();
            sorted_inputs.sort();
            assert_eq!(
                inputs, &sorted_inputs,
                "inputs of {action} must be byte-sorted"
            );
        }
        let (_evidence, files) = provenance_of(doc);
        let mut sorted_files = files.clone();
        sorted_files.sort();
        assert_eq!(
            files, sorted_files,
            "provenance.files must be byte-sorted: {doc}"
        );
    }
}

/// The `user` label of one serialized document.
pub fn user_of(doc_json: &str) -> Option<String> {
    doc_value(doc_json)
        .get("user")
        .and_then(|user| user.as_str())
        .map(str::to_string)
}

/// The `provenance` block (`evidence`, `files`) of one serialized document.
pub fn provenance_of(doc_json: &str) -> (String, Vec<String>) {
    let provenance = &doc_value(doc_json)["provenance"];
    (
        provenance["evidence"]
            .as_str()
            .expect("evidence is a string")
            .to_string(),
        provenance["files"]
            .as_array()
            .expect("files is an array")
            .iter()
            .map(|file| file.as_str().expect("file is a string").to_string())
            .collect(),
    )
}

/// The tree-relative candidate paths a claim located, sorted for comparison.
pub fn located_files(claim: &Detection) -> Vec<String> {
    let mut files: Vec<String> = claim
        .per_user
        .iter()
        .flat_map(|user| user.files.iter())
        .map(|file| file.to_string_lossy().into_owned())
        .collect();
    files.sort();
    files
}

/// Transcodes UTF-8 text to little-endian UTF-16 with a BOM (A19).
pub fn utf16le_with_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

/// The same as big-endian.
pub fn utf16be_with_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFE, 0xFF];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    bytes
}

/// A UTF-8 BOM followed by the text.
pub fn utf8_with_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

/// The `Name`/`Command` token universe a fixture INI can legitimately state:
/// every quoted token of its `Bindings=(Name="…",Command="…")` tuples
/// (extraction honesty: no document may carry a token outside this set).
pub fn ini_tokens(ini_text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = ini_text;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        tokens.push(after[..close].to_string());
        rest = &after[close + 1..];
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

/// Extracts with `unreal-ini`, expecting the named refusal variant.
pub fn expect_refusal(
    claim: &Detection,
    matches: impl Fn(&ExtractionRefusal) -> bool,
    what: &str,
) -> ExtractionRefusal {
    let result = UnrealIni.extract(claim);
    let refusal = match result {
        Ok(docs) => panic!(
            "{what}: extraction was expected to refuse, got {} documents",
            docs.len()
        ),
        Err(refusal) => refusal,
    };
    assert!(matches(&refusal), "{what}: unexpected refusal {refusal:?}");
    refusal
}

/// Loads a snapshot document from disk (thin re-export so suites read one name).
pub fn load(path: &Path) -> Result<BindingSnapshot, bindforge_codecs::snapshot::SnapshotRefusal> {
    load_snapshot(path)
}
