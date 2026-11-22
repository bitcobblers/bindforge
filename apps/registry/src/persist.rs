//! Where the two documents live and how either one is written (design 026,
//! Chosen design: §`apps/registry`'s copied list; `026-C7`, A6, A7, A8).
//!
//! The data-directory rule and the tmp-and-rename persist are the copied shipped
//! discipline: `<data>` is absolute `$XDG_DATA_HOME`, else absolute
//! `$HOME/.local/share`, else absolute `%APPDATA%` on Windows, and a *relative*
//! value is treated as unset. A save stands up an owner-only `0700` directory,
//! writes the fixed `<document>.tmp` name under `create_new(true)` so a second
//! writer fails loudly rather than adopting a stale entry, `sync_all`s it, and
//! renames it into place with the mode re-asserted to `0600` past the `umask` — so
//! an acknowledged mutation is on the medium before it can become the document.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Which side of the data-directory rule applies — the same enum seam
/// `apps/discovery-cli/src/matches.rs` uses so both branches are testable on one
/// host (design 026 Test strategy #13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayout {
    /// `$XDG_DATA_HOME`, else `$HOME/.local/share`.
    Unix,
    /// `%APPDATA%`.
    Windows,
}

impl DataLayout {
    /// The layout of this build's target platform.
    #[must_use]
    pub fn host() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}

/// An environment value that is set and absolute for `layout`; a relative value is
/// invalid and treated as unset (A6).
fn absolute_env(
    layout: DataLayout,
    env: &dyn Fn(&str) -> Option<String>,
    key: &str,
) -> Option<PathBuf> {
    let value = env(key)?;
    let absolute = match layout {
        DataLayout::Unix => Path::new(&value).is_absolute(),
        DataLayout::Windows => has_drive_prefix(&value) || Path::new(&value).is_absolute(),
    };
    absolute.then(|| PathBuf::from(value))
}

/// A `C:`-style drive prefix, tested as bytes so the rule holds on any host.
fn has_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// `<data>` for `layout` under the environment `env` answers, or `None` when the
/// environment names no usable directory.
#[must_use]
pub fn data_dir(layout: DataLayout, env: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    match layout {
        DataLayout::Unix => match absolute_env(layout, env, "XDG_DATA_HOME") {
            Some(data_home) => Some(data_home),
            None => absolute_env(layout, env, "HOME").map(|home| home.join(".local/share")),
        },
        DataLayout::Windows => absolute_env(layout, env, "APPDATA"),
    }
}

/// `026-C7`: the one directory an instance owns inside `<data>`.
#[must_use]
pub fn registry_dir(data: &Path) -> PathBuf {
    data.join("bindforge").join("registry")
}

/// The two `026-C7` documents by name, for a data directory.
#[must_use]
pub fn document_path(data: &Path, name: &str) -> PathBuf {
    registry_dir(data).join(name)
}

/// The cache document's name (`026-C7`). Retired as a *name* by ADR-027: a hosted
/// instance refuses to start on it rather than carrying inert state, which is the
/// only thing it still reads it for.
pub const CACHE_DOCUMENT: &str = "cache-v1.json";

/// The local-entries document's name (`026-C7`). Retired with
/// [`CACHE_DOCUMENT`], with the same exception: `--corpus` may name it.
pub const CATALOG_DOCUMENT: &str = "catalog-v1.json";

/// `027-C4`: the issued document — a `026-C3` catalog document the instance mints
/// into, with its own `revision`.
pub const ISSUED_DOCUMENT: &str = "issued-v1.json";

/// `027-L2`: the retired-ids document — `{version, ids}` in byte order, the memory
/// that makes "deleting a record never frees its id" true.
pub const RETIRED_IDS_DOCUMENT: &str = "retired-ids-v1.json";

/// `027-C3`: the steam snapshot, the resolver's whole knowledge of the store.
pub const STEAM_SNAPSHOT_DOCUMENT: &str = "steam-snapshot.json";

/// §Concurrency's cross-process guard: `<data>/bindforge/registry/write.lock`, taken
/// under `create_new` by `serve` and `query` and removed on exit.
pub const WRITE_LOCK: &str = "write.lock";

/// Echoes a path inside the shared renderer's bounds without cutting off the file
/// name an operator has to act on: the parent is escaped and truncated, the last
/// component is escaped whole. Copied from the shipped client, because it is what
/// keeps a refusal naming a document in a hostile `$XDG_DATA_HOME` to one line.
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
    let echoed_parent = bindforge_core::escape_and_truncate(&parent);
    if parent.is_empty() {
        return bindforge_core::escape_and_truncate(&name);
    }
    format!(
        "{echoed_parent}{}{}",
        std::path::MAIN_SEPARATOR,
        bindforge_core::escape_and_truncate(&name)
    )
}

/// The stderr value filter every warning passes through: exactly one line of
/// printable ASCII. Control bytes become backslash-`u` escapes, and long values are
/// truncated the way the error discipline truncates them. Common typographic
/// punctuation — the prose in this crate's own refusal text — folds to its ASCII
/// spelling, and anything else outside ASCII is escaped rather than written. One
/// hostile store error, or one badly-worded document, therefore
/// cannot forge an extra stderr line or smuggle a non-ASCII byte into a log an
/// operator's tooling reads as ASCII (`027-L4`'s scrubbing half).
#[must_use]
pub fn echo_safe(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in bindforge_core::escape_and_truncate(value).chars() {
        match c {
            '—' | '–' | '−' => out.push('-'),
            '…' => out.push_str("..."),
            '‘' | '’' => out.push('\''),
            '“' | '”' => out.push('"'),
            c if c.is_ascii() => out.push(c),
            other => out.push_str(&format!("\\u{:04x}", other as u32)),
        }
    }
    out
}

/// A persist that did not happen: what stopped it, naming the document in the
/// reason itself (every call site names the path in its own words).
#[derive(Debug)]
pub struct PersistError {
    pub reason: String,
}

impl PersistError {
    fn at(_path: &Path, reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

/// Creates `dir` if it is missing, owner-only where unix modes apply (A8). Public
/// because ADR-027's hosted instance takes its `<data>` write lock in this
/// directory before it has anything to persist, so the directory has to be created
/// by a caller that is not saving a document.
pub fn owner_only_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
        narrow_to_owner_only(dir)
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(dir)
    }
}

/// Enforces [`owner_only_dir`]'s owner-only **ceiling**, which is not the same rule as
/// "its mode is `0700`": this clears the group and other bits and never sets one, so a
/// directory the operator holds stricter (`0500`) stays as they set it and a save into
/// it fails closed on its own permissions. Widening it back to `0700` would undo a
/// deliberate read-only posture on the way to a write the design says must fail — Test
/// strategy #4's injected persist failure is the named `503` with the file and the
/// model untouched, and a save that chmods the directory first answers that write `200`.
/// A2's "parent `0700`" is the mode [`owner_only_dir`] *creates* and the ceiling this
/// never exceeds.
#[cfg(unix)]
fn narrow_to_owner_only(dir: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(dir)?.permissions().mode();
    if mode & 0o077 != 0 {
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Opens a fresh file for writing, refusing to follow or reuse an existing entry,
/// owner-only where unix modes apply (A7). Public for the same reason
/// [`owner_only_dir`] is: §Concurrency's `write.lock` is taken with exactly this
/// exclusive-creation discipline.
pub fn owner_only_new_file(path: &Path) -> std::io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
    }
    #[cfg(not(unix))]
    {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
}

/// Re-asserts the owner-only file mode past the `umask`, where unix modes apply.
fn owner_only(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, PermissionsExt::from_mode(0o600))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// Writes `bytes` to `path` through the fixed `<file-name>.tmp` name and renames it
/// into place (A7). The caller holds the write lock that produced `bytes`, so
/// persist order cannot diverge from mutation order (§Concurrency).
pub fn save(path: &Path, bytes: &[u8]) -> Result<(), PersistError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    owner_only_dir(parent)
        .map_err(|error| PersistError::at(path, format!("the directory is unwritable: {error}")))?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| PersistError::at(path, "the document has no file name"))?;
    let tmp = path.with_file_name(format!("{name}.tmp"));
    {
        let mut handle = owner_only_new_file(&tmp).map_err(|error| {
            PersistError::at(
                path,
                format!(
                    "the temporary file `{}` is unwritable: {error}",
                    tmp.display()
                ),
            )
        })?;
        handle
            .write_all(bytes)
            .and_then(|()| handle.flush())
            .map_err(|error| {
                PersistError::at(path, format!("the temporary file is unwritable: {error}"))
            })?;
        // The octets are on the medium *before* the rename can make them the document.
        // Without this, a filesystem that does not order data ahead of the rename's
        // commit can leave a zero-length or stale file for a mutation the admin face
        // already answered `200` for, which is exactly what §Concurrency promises a
        // restart cannot do. The parent directory is deliberately not synced: std
        // offers no portable directory handle, and a rename lost that way leaves the
        // *previous* document whole — the failure mode that would lose an accepted
        // mutation is the one this call closes.
        handle.sync_all().map_err(|error| {
            PersistError::at(
                path,
                format!("the temporary file could not be synced: {error}"),
            )
        })?;
    }
    owner_only(&tmp)
        .map_err(|error| PersistError::at(path, format!("the temporary file's mode: {error}")))?;
    fs::rename(&tmp, path)
        .map_err(|error| PersistError::at(path, format!("the rename failed: {error}")))?;
    owner_only(path)
        .map_err(|error| PersistError::at(path, format!("the document's mode: {error}")))
}

/// Reads a document whose size the caller has already gated, distinguishing "no
/// file yet" — which `026-C7` and `026-C3` both read as an empty document — from a
/// read that failed.
pub fn read_if_present(path: &Path) -> Result<Option<Vec<u8>>, PersistError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PersistError::at(
            path,
            format!("the document is unreadable: {error}"),
        )),
    }
}

/// The metadata-gated read §Budgets fixes for both documents (A22): the size is
/// refused *before* the file is read.
pub fn read_capped(path: &Path, cap: usize) -> Result<Option<Vec<u8>>, PersistError> {
    match fs::metadata(path) {
        Ok(meta) if meta.len() > cap as u64 => Err(PersistError::at(
            path,
            format!(
                "the document is {} octets, over the {cap}-octet cap",
                meta.len()
            ),
        )),
        Ok(_) => read_if_present(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PersistError::at(
            path,
            format!("the document cannot be inspected: {error}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    //! The data-directory rule through the env-closure seam (both platform branches,
    //! and relative-is-unset), and the save: exclusive tmp, owner-only modes, no tmp
    //! left behind.
    use super::*;

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
    fn the_unix_branch_prefers_an_absolute_xdg_data_home() {
        let found = data_dir(
            DataLayout::Unix,
            &env(&[
                ("XDG_DATA_HOME", "/sandbox-data"),
                ("HOME", "/sandbox-home"),
            ]),
        );
        assert_eq!(found, Some(PathBuf::from("/sandbox-data")));
    }

    #[test]
    fn the_unix_branch_falls_back_to_home_local_share() {
        let found = data_dir(DataLayout::Unix, &env(&[("HOME", "/sandbox-home")]));
        assert_eq!(found, Some(PathBuf::from("/sandbox-home/.local/share")));
        assert_eq!(
            document_path(&found.expect("a data directory"), CACHE_DOCUMENT),
            PathBuf::from("/sandbox-home/.local/share/bindforge/registry/cache-v1.json")
        );
    }

    #[test]
    fn a_relative_data_directory_is_unset() {
        let found = data_dir(
            DataLayout::Unix,
            &env(&[
                ("XDG_DATA_HOME", "relative-data"),
                ("HOME", "/sandbox-home"),
            ]),
        );
        assert_eq!(found, Some(PathBuf::from("/sandbox-home/.local/share")));
        assert_eq!(
            data_dir(
                DataLayout::Unix,
                &env(&[("XDG_DATA_HOME", "relative-data")])
            ),
            None,
            "with no usable fallback there is no data directory at all"
        );
    }

    #[test]
    fn the_windows_branch_uses_appdata_including_a_drive_prefix() {
        assert_eq!(
            data_dir(
                DataLayout::Windows,
                &env(&[("APPDATA", r"C:\Users\op\AppData\Roaming")])
            ),
            Some(PathBuf::from(r"C:\Users\op\AppData\Roaming")),
            "a drive prefix counts as absolute even on a host that knows nothing about `C:`"
        );
        assert_eq!(
            data_dir(DataLayout::Windows, &env(&[("APPDATA", "relative")])),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_save_is_exclusive_owner_only_and_leaves_no_tmp() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!(
            "bindforge-registry-persist-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("a post-epoch clock")
                .as_nanos()
        ));
        let document = root.join("bindforge").join("registry").join(CACHE_DOCUMENT);
        save(&document, b"{\"version\":1,\"entries\":[]}").expect("the save lands");
        let mode = fs::metadata(&document)
            .expect("the document")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "the document is owner-only");
        let dir = registry_dir(&root);
        assert_eq!(
            fs::metadata(&dir)
                .expect("the directory")
                .permissions()
                .mode()
                & 0o777,
            0o700,
            "the directory the save stood up is owner-only"
        );
        assert!(
            !dir.join("cache-v1.json.tmp").exists(),
            "the tmp is renamed away"
        );
        let stale = dir.join("catalog-v1.json.tmp");
        fs::write(&stale, b"a save that never finished its rename").expect("a stale tmp entry");
        let catalog = document_path(&root, CATALOG_DOCUMENT);
        let error = save(&catalog, b"{\"version\":1,\"revision\":0,\"games\":[]}")
            .expect_err("a stale tmp is a loud failure, never an adopted one");
        assert!(
            error.reason.contains("unwritable"),
            "the refusal says which write failed: {}",
            error.reason
        );
        assert_eq!(
            fs::read(&stale).expect("the stale tmp"),
            b"a save that never finished its rename"
        );
        assert!(!catalog.exists(), "nothing was renamed into place");
        let _ = fs::remove_dir_all(&root);
    }
}
