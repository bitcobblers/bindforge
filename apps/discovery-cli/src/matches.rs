//! The per-user match store (ADR-025 `025-C3`, `025-L4`): the one place a
//! cross-source `game_id` is kept.
//!
//! One JSON document at `<data>/bindforge/matches.json`, where `<data>` is
//! `$XDG_DATA_HOME` when set and absolute (else `$HOME/.local/share`) on
//! Linux/other and `%APPDATA%` on Windows — a relative value being unset
//! (`025-C3`). A Flatpak or Snap install needs no extra rule: its
//! `XDG_DATA_HOME` is already the sandbox redirect. The directory is resolved
//! from an environment *lookup closure*, mirroring
//! `steam_common::absolute_env` and `steam_linux::host_candidates`, so the rule
//! is unit-tested — both platform branches — without touching the process
//! environment.
//!
//! The file is `{ "version": 1, "matches": [ { slug, game_id, friendly_name } ] }`,
//! records sorted by `slug` byte order so a re-save is byte-stable. There is no
//! dedup logic: the store is keyed by `slug` alone, so two installs of one game
//! are two records sharing one `game_id`. Records never expire and are re-trusted
//! on every scan (`025-L4`).
//!
//! A `game_id` is the one identifier here that becomes a path component
//! downstream, so it is validated at all three places it is seen — candidate
//! parse, confirmation write, store load — because both the store file and the
//! server response are untrusted inputs (`025-L5`).

use bindforge_core::{
    MAX_DISCOVERY_SLUG_OCTETS, MAX_SLUG_OCTETS, escape_and_truncate, is_slug_name,
};
use discovery::game::Source;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The read budget of the whole store document: the same ceiling the catalog
/// response gets, because a two-field record set has no business being larger.
const STORE_READ_BUDGET_OCTETS: u64 = 65_536;

/// The document version this build writes and the only one it reads.
const STORE_VERSION: u32 = 1;

/// Which side of the data-directory rule applies. Chosen from the compile
/// target at the call site, and passed in as data so both branches are tested
/// on one host (design 025 Test strategy #5, Compatibility matrix: the Windows
/// half is unit-tested through this seam, not run on Windows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreLayout {
    /// `$XDG_DATA_HOME`, else `$HOME/.local/share`.
    Unix,
    /// `%APPDATA%`.
    Windows,
}

/// The layout of this build's target platform.
#[must_use]
pub fn host_layout() -> StoreLayout {
    if cfg!(windows) {
        StoreLayout::Windows
    } else {
        StoreLayout::Unix
    }
}

/// An environment value that is set and absolute for `layout`; a relative value
/// is invalid and treated as unset (the rule `steam_common::absolute_env`
/// applies to the Steam roots, applied here to the data directory). The
/// Windows branch tests its drive-prefix spelling as text, because a Linux
/// host's `Path::is_absolute` knows nothing about `C:` — the same
/// unit-test-the-seam-not-the-OS stance the layout enum exists for.
fn absolute_env(
    layout: StoreLayout,
    env: &dyn Fn(&str) -> Option<String>,
    key: &str,
) -> Option<PathBuf> {
    let value = env(key)?;
    let absolute = match layout {
        StoreLayout::Unix => Path::new(&value).is_absolute(),
        StoreLayout::Windows => has_drive_prefix(&value) || Path::new(&value).is_absolute(),
    };
    absolute.then(|| PathBuf::from(value))
}

/// A `C:`-style drive prefix, tested as bytes so the rule holds on any host.
fn has_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// The data directory for `layout` under the environment `env` answers, or
/// `None` when the environment names no usable directory (no path, no store).
#[must_use]
pub fn data_dir(layout: StoreLayout, env: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    match layout {
        StoreLayout::Unix => match absolute_env(layout, env, "XDG_DATA_HOME") {
            Some(data_home) => Some(data_home),
            None => absolute_env(layout, env, "HOME").map(|home| home.join(".local/share")),
        },
        StoreLayout::Windows => absolute_env(layout, env, "APPDATA"),
    }
}

/// The store file for `layout` under the environment `env` answers.
#[must_use]
pub fn store_path(layout: StoreLayout, env: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    data_dir(layout, env).map(|dir| dir.join("bindforge").join("matches.json"))
}

/// The store's refusal to read a file it cannot trust. `<path>` is echoed by
/// [`echo_path`], and nothing on disk is rewritten: the user's remedy is to
/// remove the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError {
    pub path: PathBuf,
    /// What was wrong with the file, for the operator's own reading.
    pub reason: String,
}

/// The one failure the design leaves unnamed: a refusal to *write*. The load
/// refusals are `error: match store unreadable:`; a write failure is the same
/// shape with its own verb so a half-succeeded run is never read as a broken
/// store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteError {
    pub path: PathBuf,
    pub reason: String,
}

/// Echoes a path inside the shared renderer's bounds without cutting off the
/// file name the operator has to act on: the parent is escaped and truncated,
/// the last component is escaped whole. A newline in `$XDG_DATA_HOME` therefore
/// still yields exactly one line (`025-L5`).
#[must_use]
pub fn echo_path(path: &Path) -> String {
    let parent = path
        .parent()
        .map(|parent| parent.display().to_string())
        .unwrap_or_default();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    format!(
        "{}{}",
        escape_and_truncate(&parent),
        escape_and_truncate(&name)
    )
}

/// One confirmed record: the lookup key, the curated id the user chose for it,
/// and the display metadata that made the choice legible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchRecord {
    pub slug: String,
    pub game_id: String,
    pub friendly_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StoreDocument {
    version: u32,
    matches: Vec<MatchRecord>,
}

/// The in-memory store: records in `slug` byte order, one per distinct slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchStore {
    path: Option<PathBuf>,
    records: Vec<MatchRecord>,
}

impl MatchStore {
    /// Loads `path`, or an empty store when it is absent. Every other shape is
    /// the named refusal: unparseable content, an unknown `version`, a file past
    /// the read budget, a slug past `025-B1` or spelled outside
    /// `<token>/<external-id>`, or a `game_id` failing the shared grammar and
    /// bound.
    pub fn load(path: Option<PathBuf>) -> Result<MatchStore, StoreError> {
        let Some(file) = path.clone() else {
            // No usable data directory: the scan runs, it just has no memory.
            return Ok(MatchStore {
                path: None,
                records: Vec::new(),
            });
        };
        if !file.exists() {
            return Ok(MatchStore {
                path: path.clone(),
                records: Vec::new(),
            });
        }
        let refuse = |reason: String| StoreError {
            path: file.clone(),
            reason,
        };
        let oversized = fs::metadata(&file).is_ok_and(|meta| meta.len() > STORE_READ_BUDGET_OCTETS);
        if oversized {
            return Err(refuse(format!(
                "the file is past the {STORE_READ_BUDGET_OCTETS}-octet read budget"
            )));
        }
        let bytes = match fs::read(&file) {
            Ok(bytes) => bytes,
            Err(error) => return Err(refuse(format!("the file could not be read: {error}"))),
        };
        if u64::try_from(bytes.len())
            .unwrap_or(u64::MAX)
            .gt(&STORE_READ_BUDGET_OCTETS)
        {
            return Err(refuse(format!(
                "the file is past the {STORE_READ_BUDGET_OCTETS}-octet read budget"
            )));
        }
        let document: StoreDocument = match serde_json::from_slice(&bytes) {
            Ok(document) => document,
            Err(error) => return Err(refuse(format!("the file is not a match store: {error}"))),
        };
        if document.version != STORE_VERSION {
            return Err(refuse(format!(
                "version {} is not version {STORE_VERSION}",
                document.version
            )));
        }
        for record in &document.matches {
            if record.slug.len() > MAX_DISCOVERY_SLUG_OCTETS {
                return Err(refuse(format!(
                    "a record's lookup key is past `025-B1`'s cap: {}",
                    escape_and_truncate(&record.slug)
                )));
            }
            if !Source::spells_lookup_key(&record.slug) {
                return Err(refuse(format!(
                    "a record's lookup key is not spelled `<source>/<external-id>`: {}",
                    escape_and_truncate(&record.slug)
                )));
            }
            if !game_id_is_valid(&record.game_id) {
                return Err(refuse(format!(
                    "a record's game id is not a grammar-valid id: {}",
                    escape_and_truncate(&record.game_id)
                )));
            }
        }
        let mut records = document.matches;
        sort_records(&mut records);
        Ok(MatchStore { path, records })
    }

    /// The confirmed id of `slug`, if this store holds one.
    #[must_use]
    pub fn game_id(&self, slug: &str) -> Option<&str> {
        self.records
            .iter()
            .find(|record| record.slug == slug)
            .map(|record| record.game_id.as_str())
    }

    /// Whether this store resolves `slug` (a resolved slug is never queried,
    /// `025-L4`).
    #[must_use]
    pub fn resolves(&self, slug: &str) -> bool {
        self.game_id(slug).is_some()
    }

    /// The confirmed records, in `slug` byte order. The CLI's own flow speaks
    /// through `game_id`/`resolves`; this is the store's read surface for its
    /// unit tests and any later consumer that lists the inventory.
    #[must_use]
    #[allow(dead_code)]
    pub fn records(&self) -> &[MatchRecord] {
        &self.records
    }

    /// Whether a save is possible at all (an unresolvable data directory is not).
    #[must_use]
    pub fn writable(&self) -> bool {
        self.path.is_some()
    }

    /// Records the user's choice for one slug. A `game_id` that fails the shared
    /// grammar and bound is refused here as well as at the candidate parse and
    /// at load: this is the write seam that feeds a path component (`025-L5`).
    pub fn confirm(&mut self, slug: &str, game_id: &str, friendly_name: &str) -> bool {
        if !Source::spells_lookup_key(slug)
            || slug.len() > MAX_DISCOVERY_SLUG_OCTETS
            || !game_id_is_valid(game_id)
        {
            return false;
        }
        if let Some(record) = self.records.iter_mut().find(|record| record.slug == slug) {
            record.game_id = game_id.to_string();
            record.friendly_name = friendly_name.to_string();
            return true;
        }
        self.records.push(MatchRecord {
            slug: slug.to_string(),
            game_id: game_id.to_string(),
            friendly_name: friendly_name.to_string(),
        });
        sort_records(&mut self.records);
        true
    }

    /// The canonical bytes of this store: `version` and the records in `slug`
    /// byte order, so re-saving the same record set is byte-identical whatever
    /// the confirmation order was.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let document = StoreDocument {
            version: STORE_VERSION,
            matches: self.records.clone(),
        };
        // A `Vec` of plain records cannot fail to serialize; an empty fallback
        // would only hide a bug, so the writer's own error path is the panic.
        serde_json::to_vec(&document).expect("the store document is serializable")
    }

    /// Writes the store through `matches.json.tmp` — opened exclusively, never
    /// through a pre-existing entry — and renames it into place, after standing
    /// up the parent directory. Where unix modes apply, the directory and both
    /// files are owner-only, so the confirmed inventory is never a `umask`
    /// accident.
    pub fn save(&self) -> Result<(), WriteError> {
        let Some(file) = self.path.clone() else {
            return Ok(());
        };
        let parent = file
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let failed = |reason: String| WriteError {
            path: file.clone(),
            reason,
        };
        owner_only_dir(parent)
            .map_err(|error| failed(format!("the directory is unwritable: {error}")))?;
        let tmp = file.with_file_name("matches.json.tmp");
        let bytes = self.to_bytes();
        {
            let mut handle = owner_only_new_file(&tmp)
                .map_err(|error| failed(format!("the temporary file is unwritable: {error}")))?;
            handle
                .write_all(&bytes)
                .and_then(|()| handle.flush())
                .map_err(|error| failed(format!("the temporary file is unwritable: {error}")))?;
        }
        owner_only(&tmp).map_err(|error| failed(format!("the temporary file's mode: {error}")))?;
        fs::rename(&tmp, &file).map_err(|error| failed(format!("the rename failed: {error}")))?;
        owner_only(&file).map_err(|error| failed(format!("the store file's mode: {error}")))?;
        Ok(())
    }
}

/// The shared `game_id` rule: the grammar a product id rides, plus its bound.
#[must_use]
pub fn game_id_is_valid(game_id: &str) -> bool {
    is_slug_name(game_id) && game_id.len() <= MAX_SLUG_OCTETS
}

fn sort_records(records: &mut [MatchRecord]) {
    records.sort_by(|a, b| a.slug.cmp(&b.slug));
}

/// Creates `dir` if it is missing, owner-only where unix modes apply.
fn owner_only_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(dir)
    }
}

/// Opens a fresh file for writing, refusing to follow or reuse an existing
/// entry, owner-only where unix modes apply.
fn owner_only_new_file(path: &Path) -> std::io::Result<fs::File> {
    #[cfg(unix)]
    {
        std::os::unix::fs::OpenOptionsExt::mode(
            std::fs::OpenOptions::new().write(true).create_new(true),
            0o600,
        )
        .open(path)
    }
    #[cfg(not(unix))]
    {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
    }
}

/// Reasserts the owner-only file mode past the `umask`, where unix modes apply.
fn owner_only(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the seams the subprocess suites cannot reach from one
    //! host: the data-directory rule (both platform branches, through the
    //! environment lookup), the load-time refusals, and the save — sorted,
    //! byte-stable, exclusive, owner-only.
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "bindforge-store-unit-{label}-{}-{nonce}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("scratch dir");
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// An environment from literal pairs, the seam the design names.
    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let owned: Vec<(String, String)> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        move |key: &str| {
            owned
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.clone())
        }
    }

    #[test]
    fn an_absolute_xdg_data_home_owns_the_store_on_the_unix_branch() {
        let path = store_path(
            StoreLayout::Unix,
            &env(&[("XDG_DATA_HOME", "/data"), ("HOME", "/home/u")]),
        );
        assert_eq!(path, Some(PathBuf::from("/data/bindforge/matches.json")));
    }

    #[test]
    fn a_relative_xdg_data_home_is_unset_and_home_wins_on_the_unix_branch() {
        let relative = store_path(
            StoreLayout::Unix,
            &env(&[("XDG_DATA_HOME", "relative/dir"), ("HOME", "/home/u")]),
        );
        assert_eq!(
            relative,
            Some(PathBuf::from("/home/u/.local/share/bindforge/matches.json")),
            "a relative value is invalid, which is the same as unset (A9's rule)"
        );
        let unset = store_path(StoreLayout::Unix, &env(&[("HOME", "/home/u")]));
        assert_eq!(
            unset,
            Some(PathBuf::from("/home/u/.local/share/bindforge/matches.json")),
            "with no XDG value the `$HOME` fallback places the store"
        );
    }

    #[test]
    fn the_windows_branch_resolves_appdata_through_the_same_seam() {
        let path = store_path(
            StoreLayout::Windows,
            &env(&[("APPDATA", "C:/Users/u/AppData/Roaming")]),
        );
        assert_eq!(
            path,
            Some(PathBuf::from(
                "C:/Users/u/AppData/Roaming/bindforge/matches.json"
            )),
            "the Windows branch is the same absolute-value rule on `%APPDATA%`"
        );
        assert_eq!(
            store_path(StoreLayout::Windows, &env(&[("APPDATA", "relative\\path")])),
            None,
            "a relative `%APPDATA%` is unset, and with no data directory there is no store"
        );
        assert_eq!(store_path(StoreLayout::Windows, &env(&[])), None);
    }

    #[test]
    fn no_data_directory_at_all_is_no_store_rather_than_an_error() {
        assert_eq!(store_path(StoreLayout::Unix, &env(&[])), None);
        let store = MatchStore::load(None).expect("no store is a loadable state");
        assert!(store.records().is_empty());
        assert!(!store.writable());
    }

    #[test]
    fn an_absent_file_is_an_empty_store() {
        let scratch = Scratch::new("absent");
        let file = scratch.0.join("bindforge").join("matches.json");
        let store = MatchStore::load(Some(file)).expect("absent is empty");
        assert!(store.records().is_empty());
    }

    #[test]
    fn a_valid_file_yields_its_records_in_slug_order() {
        let scratch = Scratch::new("valid");
        let dir = scratch.0.join("bindforge");
        fs::create_dir_all(&dir).expect("store dir");
        fs::write(
            dir.join("matches.json"),
            br#"{"version":1,"matches":[{"slug":"steam/9","game_id":"b-game","friendly_name":"B"},{"slug":"steam/10","game_id":"a-game","friendly_name":"A"}]}"#,
        )
        .expect("store file");
        let store = MatchStore::load(Some(dir.join("matches.json"))).expect("a valid store loads");
        assert_eq!(
            store
                .records()
                .iter()
                .map(|record| (record.slug.as_str(), record.game_id.as_str()))
                .collect::<Vec<(&str, &str)>>(),
            vec![("steam/10", "a-game"), ("steam/9", "b-game")],
            "records are read in `slug` byte order"
        );
        assert_eq!(store.game_id("steam/9"), Some("b-game"));
        assert!(store.resolves("steam/10"));
        assert!(!store.resolves("steam/404"));
    }

    #[test]
    fn every_untrusted_store_shape_is_the_named_refusal() {
        let padded = format!(
            "{{\"version\":1,\"matches\":[],\"pad\":\"{}\"}}",
            "p".repeat(70_000)
        );
        let broken: Vec<(&str, String)> = vec![
            ("truncated", "{\"version\":1,\"matches\":[{\"slug\":".to_string()),
            ("version-two", "{\"version\":2,\"matches\":[]}".to_string()),
            ("over-budget", padded),
            (
                "slug-past-bound",
                format!(
                    "{{\"version\":1,\"matches\":[{{\"slug\":\"steam/{}\",\"game_id\":\"g\",\"friendly_name\":\"F\"}}]}}",
                    "9".repeat(130)
                ),
            ),
            (
                "slug-unspelled",
                "{\"version\":1,\"matches\":[{\"slug\":\"example-game\",\"game_id\":\"g\",\"friendly_name\":\"F\"}]}".to_string(),
            ),
            (
                "slug-unknown-token",
                "{\"version\":1,\"matches\":[{\"slug\":\"origin/9\",\"game_id\":\"g\",\"friendly_name\":\"F\"}]}".to_string(),
            ),
            (
                "game-id-traversal",
                "{\"version\":1,\"matches\":[{\"slug\":\"steam/9\",\"game_id\":\"../x\",\"friendly_name\":\"F\"}]}".to_string(),
            ),
        ];
        for (label, body) in &broken {
            let scratch = Scratch::new(label);
            let dir = scratch.0.join("bindforge");
            fs::create_dir_all(&dir).expect("store dir");
            let file = dir.join("matches.json");
            fs::write(&file, body).expect("store file");
            let outcome = MatchStore::load(Some(file.clone()));
            let Err(error) = outcome else {
                panic!("the `{label}` store must be refused, got {outcome:?}");
            };
            assert_eq!(error.path, file, "the refusal names the file");
            assert!(
                !fs::read(&file).expect("still there").is_empty(),
                "the refusal rewrites nothing for `{label}`"
            );
        }
    }

    #[test]
    fn the_refusal_echo_of_a_path_is_one_escaped_line_keeping_the_file_name() {
        let odd = PathBuf::from("/data/xdg\nhome/bindforge/matches.json");
        let line = echo_path(&odd);
        assert_eq!(
            line.lines().count(),
            1,
            "a newline in the data directory is escaped, never raw: {line:?}"
        );
        assert!(
            line.contains("matches.json"),
            "the operator has to see which file to remove: {line:?}"
        );
        assert!(
            !line.contains('\n') && !line.contains('\u{7}'),
            "the echo carries no control byte: {line:?}"
        );
    }

    #[test]
    fn confirming_refuses_an_id_that_is_not_a_product_id() {
        let mut store = MatchStore::load(None).expect("empty store");
        assert!(store.confirm("steam/9", "good-game", "Game"));
        assert!(
            !store.confirm("steam/10", "../x", "Game"),
            "`../x` is no id (`025-L5`)"
        );
        assert!(!store.confirm("steam/10", "UPPER", "Game"));
        assert!(!store.confirm("example-game", "fine", "Game"));
        assert_eq!(
            store
                .records()
                .iter()
                .map(|record| record.slug.as_str())
                .collect::<Vec<&str>>(),
            vec!["steam/9"]
        );
    }

    #[test]
    fn a_saved_store_is_sorted_canonical_and_reuses_its_own_bytes() {
        let scratch = Scratch::new("save");
        let file = scratch.0.join("bindforge").join("matches.json");
        let mut store = MatchStore::load(Some(file.clone())).expect("empty store");
        store.confirm("steam/5002", "cand-one", "Second Game");
        store.confirm("steam/5001", "cand-one", "First Game");
        store.save().expect("the save lands");

        let first = fs::read(&file).expect("the store file");
        assert_eq!(first, store.to_bytes(), "the file is the canonical bytes");
        assert!(
            !file.with_file_name("matches.json.tmp").exists(),
            "the temp name is renamed away, never left behind"
        );
        // Re-saving a store whose confirmations arrived in the other order
        // produces the same bytes.
        let mut reshuffled = MatchStore::load(Some(file.clone())).expect("reload");
        reshuffled.confirm("steam/5001", "cand-one", "First Game");
        reshuffled.confirm("steam/5002", "cand-one", "Second Game");
        assert_eq!(
            reshuffled.to_bytes(),
            first,
            "one record set, two confirmation orders: identical bytes"
        );

        let names: Vec<String> = fs::read_dir(scratch.0.join("bindforge"))
            .expect("store dir")
            .map(|entry| {
                entry
                    .expect("entry")
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        assert_eq!(names, vec!["matches.json".to_string()]);
    }

    #[test]
    fn the_temp_name_is_opened_exclusively_only() {
        let scratch = Scratch::new("exclusive");
        let file = scratch.0.join("bindforge").join("matches.json");
        let store = MatchStore::load(Some(file.clone())).expect("empty store");
        let tmp = file.with_file_name("matches.json.tmp");
        fs::create_dir_all(file.parent().expect("parent")).expect("store dir");
        fs::write(&tmp, b"pre-existing").expect("a stale temp entry");
        let outcome = store.save();
        assert!(
            outcome.is_err(),
            "the save never writes through a pre-existing `matches.json.tmp`"
        );
        assert_eq!(fs::read(&tmp).expect("untouched"), b"pre-existing");
        assert!(!file.exists(), "nothing is renamed into place");
    }

    #[cfg(unix)]
    #[test]
    fn the_directory_and_the_store_file_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let scratch = Scratch::new("modes");
        let file = scratch.0.join("bindforge").join("matches.json");
        let mut store = MatchStore::load(Some(file.clone())).expect("empty store");
        store.confirm("steam/9", "cand-one", "Game");
        store.save().expect("the save lands");

        let dir_mode = fs::metadata(scratch.0.join("bindforge"))
            .expect("store dir")
            .permissions()
            .mode();
        assert_eq!(dir_mode & 0o777, 0o700, "`bindforge/` is owner-only");
        let file_mode = fs::metadata(&file)
            .expect("store file")
            .permissions()
            .mode();
        assert_eq!(file_mode & 0o777, 0o600, "matches.json is owner-only");
    }
}
