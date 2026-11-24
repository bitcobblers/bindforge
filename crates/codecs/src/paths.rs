//! The config-path resolvers (§Config-path resolvers).
//!
//! Two template families, each returned only when it exists, in `TreeKind`
//! declaration order then byte order of the tree's `root` — a total order
//! over the trees, which §Determinism relies on — and each built from a
//! `GameContext` anchor. No host-config tree ships this run: `$HOME/.config`
//! is shared by every Unreal title and no `ScanRecord` field names the
//! project directory inside it (`017-L3`).
//!
//! Candidate paths are never canonicalized: the anchors (`roots`, `library`)
//! are already canonical from the scan and no link is followed at an anchor
//! or below one, so a walk never leaves the tree it started in.

use std::path::PathBuf;

use crate::MAX_USER_DIRS_PER_GAME;
use crate::context::GameContext;

/// The located launcher-context kinds, in declaration order — the first key
/// of the trees' total order (§Determinism).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeKind {
    /// `<root>/userdata/<steamid>/<appid>` — the local SteamID tree.
    SteamUserdata,
    /// `<library>/steamapps/compatdata/<appid>/pfx/drive_c/users/steamuser`
    /// — the Proton prefix (A17: `steamuser` is a template constant, never
    /// the host username).
    ProtonPrefix,
}

/// One located user tree.
#[derive(Debug, Clone)]
pub struct UserTree {
    /// Which template located it.
    pub kind: TreeKind,
    /// The context label: the SteamID directory name, or `steamuser`.
    pub user: String,
    /// The tree's root, uncanonicalized by design (§module docs).
    pub root: PathBuf,
}

/// The appid bound a `userdata` directory name must not be confused with;
/// the user directories themselves are 1–20 ASCII digits (a local SteamID),
/// mirroring the context's appid rule.
fn is_digit_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_APPID_DIGITS_BOUND
        && name.bytes().all(|byte| byte.is_ascii_digit())
}

/// The same bound `context::GameContext::from_record` enforces on appids —
/// a `userdata` directory outside it is not a SteamID.
const MAX_APPID_DIGITS_BOUND: usize = 20;

/// The located user trees for `ctx`: `SteamUserdata` from every root in
/// `roots()` order with `<dir>` in byte order (at most
/// `MAX_USER_DIRS_PER_GAME` trees kept, across all roots, in that
/// enumeration order — the one silent bound), then the `ProtonPrefix`
/// template off the owning library. A non-digit directory name, a directory
/// that is itself a link, and a tree whose own final component is a link are
/// each skipped, never anchored.
#[must_use]
pub fn user_trees(ctx: &GameContext) -> Vec<UserTree> {
    let mut trees: Vec<UserTree> = Vec::new();
    for root in &ctx.roots {
        let userdata = root.join("userdata");
        let Ok(entries) = std::fs::read_dir(&userdata) else {
            continue; // a root with no `userdata/` contributes no tree
        };
        // `<dir>` in byte order — `read_dir` return order never reaches the
        // trees' total order.
        let mut names: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            // `DirEntry::file_type()` does not follow symlinks: a
            // digit-named link is skipped here and never anchored.
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if is_digit_name(&name) {
                names.push(name);
            }
        }
        names.sort();
        for name in names {
            let tree_root = userdata.join(&name).join(&ctx.appid);
            // The tree exists only when its own final component is a real
            // directory: `symlink_metadata` so a link is never anchored.
            let Ok(meta) = std::fs::symlink_metadata(&tree_root) else {
                continue;
            };
            if !meta.is_dir() {
                continue;
            }
            trees.push(UserTree {
                kind: TreeKind::SteamUserdata,
                user: name,
                root: tree_root,
            });
            // The one silent bound: past the 16th tree the game's remaining
            // local users are simply absent.
            if trees.len() >= MAX_USER_DIRS_PER_GAME {
                return proton_trees(ctx, trees);
            }
        }
    }
    proton_trees(ctx, trees)
}

/// Appends the `ProtonPrefix` template after the `SteamUserdata` trees —
/// `TreeKind` declaration order — when it exists and is not itself a link.
fn proton_trees(ctx: &GameContext, mut trees: Vec<UserTree>) -> Vec<UserTree> {
    let root = ctx
        .library
        .join("steamapps")
        .join("compatdata")
        .join(&ctx.appid)
        .join("pfx")
        .join("drive_c")
        .join("users")
        .join("steamuser");
    match std::fs::symlink_metadata(&root) {
        Ok(meta) if meta.is_dir() => trees.push(UserTree {
            kind: TreeKind::ProtonPrefix,
            user: "steamuser".to_string(),
            root,
        }),
        _ => {} // absent, unreadable, or a link: never anchored. A broken
                // prefix (A17) locates no grammar, so the probe claims
                // nothing and the face answers `claim-refused` (exit 3) —
                // never a hang and never an out-of-tree read. The matrix's
                // "named degradation" for this row is that status: a
                // symlinked prefix and an absent one reach the same answer,
                // because `family-confirmed-files-absent` (exit 4) belongs
                // to appid-table claims, which `017-R4`'s routing data would
                // be the first to create (§The registry).)
    }
    trees
}
