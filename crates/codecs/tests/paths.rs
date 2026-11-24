//! `user_trees` — the two launcher-context templates and their budgets
//! (§Config-path resolvers).
//!
//! proves: Test strategy #3.

mod common;

use bindforge_codecs::paths::{TreeKind, user_trees};
use common::{TempRoot, context};
use std::path::PathBuf;

fn userdata(root: &TempRoot, user: &str, appid: &str) {
    root.mkdir(&format!("userdata/{user}/{appid}"));
}

fn proton_prefix(root: &TempRoot, appid: &str) {
    root.mkdir(&format!(
        "steamapps/compatdata/{appid}/pfx/drive_c/users/steamuser"
    ));
}

fn kinds_and_users(ctx: &bindforge_codecs::context::GameContext) -> Vec<(TreeKind, String)> {
    user_trees(ctx)
        .into_iter()
        .map(|tree| (tree.kind, tree.user))
        .collect()
}

// proves: Test strategy #3 (§Config-path resolvers) — both templates resolve
// from a fixture machine: `SteamUserdata` from each resolved root and
// `ProtonPrefix` from the owning library, returned in `TreeKind` declaration
// order with `user` spelled per template (the SteamID directory name; the
// constant `steamuser` — A17, never the host username).
#[test]
fn both_templates_resolve_in_declaration_order() {
    let root = TempRoot::new("paths-both");
    userdata(&root, "76561197960287930", "900001");
    proton_prefix(&root, "900001");
    let canonical = root.canonical();
    let trees = user_trees(&context("900001", vec![canonical.clone()], canonical));
    let kinds: Vec<TreeKind> = trees.iter().map(|tree| tree.kind).collect();
    assert_eq!(
        kinds,
        vec![TreeKind::SteamUserdata, TreeKind::ProtonPrefix],
        "trees come back in TreeKind declaration order"
    );
    assert_eq!(
        trees[0].user, "76561197960287930",
        "the userdata user is the SteamID dir name"
    );
    assert_eq!(
        trees[1].user, "steamuser",
        "the proton user is the constant `steamuser`"
    );
    assert_eq!(
        trees[1].root,
        root.join("steamapps/compatdata/900001/pfx/drive_c/users/steamuser"),
        "the proton template hangs off the owning library, not the host home"
    );
}

// proves: Test strategy #3 (§Config-path resolvers) — `SteamUserdata`
// enumerates `ctx.roots` in `roots()` order and `<dir>` in byte order, and a
// root with no `<dir>/<appid>` under it contributes no tree.
#[test]
fn userdata_trees_follow_roots_order_then_directory_byte_order() {
    let root_a = TempRoot::new("paths-ord-a");
    let root_b = TempRoot::new("paths-ord-b");
    let root_c = TempRoot::new("paths-ord-c"); // contributes nothing
    for user in ["76561198000000002", "76561197960287930"] {
        userdata(&root_a, user, "900001");
    }
    userdata(&root_b, "76561199000000003", "900001");
    let (can_a, can_b, can_c) = (root_a.canonical(), root_b.canonical(), root_c.canonical());
    // The enumeration follows `roots()` order — the core's canonicalized byte
    // order — so the expected sequence is derived from the byte order of the
    // canonical roots themselves.
    let mut roots = vec![(can_a.clone(), "A"), (can_b.clone(), "B")];
    roots.sort();
    let mut expected: Vec<PathBuf> = Vec::new();
    let mut expected_users: Vec<String> = Vec::new();
    for (which_root, which) in &roots {
        let users: &[&str] = match *which {
            "A" => &["76561197960287930", "76561198000000002"],
            _ => &["76561199000000003"],
        };
        for user in users {
            expected.push(which_root.join("userdata").join(user).join("900001"));
            expected_users.push((*user).to_string());
        }
    }
    let mut all_roots = vec![can_a, can_b, can_c];
    all_roots.sort();
    let trees = user_trees(&context("900001", all_roots, root_a.canonical()));
    let actual: Vec<PathBuf> = trees.iter().map(|tree| tree.root.clone()).collect();
    let users: Vec<String> = trees.iter().map(|tree| tree.user.clone()).collect();
    assert_eq!(
        actual, expected,
        "three trees: two from the first root in `<dir>` byte order, one from the second; root C contributes none"
    );
    assert_eq!(users, expected_users);
}

// proves: Test strategy #3 (§Config-path resolvers: "each returned only when it
// exists") — absent trees are not returned.
#[test]
fn absent_trees_are_not_returned() {
    let root = TempRoot::new("paths-absent");
    root.mkdir("userdata"); // userdata exists, but no <dir>/<appid> under it
    let canonical = root.canonical();
    let trees = user_trees(&context("900001", vec![canonical.clone()], canonical));
    assert!(
        trees.is_empty(),
        "no located <steamid>/<appid> tree and no prefix means no trees"
    );
}

// proves: Test strategy #3 (§Config-path resolvers) — a `userdata` directory
// name outside 1–20 digits is not a SteamID and the tree is skipped.
#[test]
fn a_non_digit_userdir_is_skipped() {
    let root = TempRoot::new("paths-nondigit");
    userdata(&root, "76561197960287930", "900001");
    userdata(&root, "notasteamid", "900001");
    userdata(&root, "76561197960287930x", "900001");
    let canonical = root.canonical();
    let found = kinds_and_users(&context("900001", vec![canonical.clone()], canonical));
    assert_eq!(
        found,
        vec![(TreeKind::SteamUserdata, "76561197960287930".to_string())],
        "only the digit-named directory names a user tree"
    );
}

// proves: Test strategy #3 and #6 (§Config-path resolvers: "a tree whose own
// final component is a link … is skipped and counted"; "a link is never a
// `UserTree` anchor") — a digit-named symlink is skipped and a symlinked tree
// root is not anchored. The link fixtures are the platform-scoped half of #17.
#[cfg(unix)]
#[test]
fn symlinked_userdirs_are_skipped_and_never_anchored() {
    use std::os::unix::fs::symlink;
    let root = TempRoot::new("paths-symlink");
    let outside = TempRoot::new("paths-symlink-target");
    outside.mkdir("real-user-tree");
    root.mkdir("userdata");
    symlink(
        outside.join("real-user-tree"),
        root.join("userdata").join("76561197960287930"),
    )
    .expect("symlink a digit-named userdir");
    let canonical = root.canonical();
    let trees = user_trees(&context("900001", vec![canonical.clone()], canonical));
    assert!(
        trees
            .iter()
            .all(|tree| tree.kind != TreeKind::SteamUserdata),
        "a symlinked userdir is skipped, never followed as a tree"
    );

    // A symlinked *tree root*: the proton template's own final component
    // (`steamuser`) is a link — `symlink_metadata` skips it.
    let lib = TempRoot::new("paths-symlink-pfx");
    lib.mkdir("steamapps/compatdata/900001/pfx/drive_c/users");
    symlink(
        outside.join("real-user-tree"),
        lib.join("steamapps/compatdata/900001/pfx/drive_c/users/steamuser"),
    )
    .expect("symlink the template root");
    let trees = user_trees(&context("900001", vec![root.canonical()], lib.canonical()));
    assert!(
        trees.iter().all(|tree| tree.kind != TreeKind::ProtonPrefix),
        "a template root that is itself a link is never anchored (the NTFS/exFAT broken-prefix case)"
    );
}

#[cfg(not(unix))]
#[test]
fn symlinked_userdirs_are_skipped_and_never_anchored() {
    // The link fixtures are made at test time behind `#[cfg(unix)]`; the
    // Windows-visible half of this face is the compile-check leg's (§Platform).
}

// proves: Test strategy #3 (§Config-path resolvers, the `MAX_USER_DIRS_PER_GAME`
// design-time bound) — two fixture roots holding 9- and 8-digit user
// directories keep the first 16 in enumeration order; the 17th is past the cap.
#[test]
fn the_user_dir_cap_keeps_the_first_sixteen_in_enumeration_order() {
    let root_a = TempRoot::new("paths-cap-a");
    let root_b = TempRoot::new("paths-cap-b");
    let nine_digit: Vec<String> = (100000001..=100000009).map(|n| n.to_string()).collect(); // 9 dirs, 9 digits
    let eight_digit: Vec<String> = (10000001..=10000008).map(|n| n.to_string()).collect(); // 8 dirs, 8 digits
    for user in &nine_digit {
        userdata(&root_a, user, "900001");
    }
    for user in &eight_digit {
        userdata(&root_b, user, "900001");
    }
    let (can_a, can_b) = (root_a.canonical(), root_b.canonical());
    let mut roots = vec![can_a.clone(), can_b.clone()];
    roots.sort();
    let (first, first_users, second, second_users) = if roots[0] == can_a {
        (&can_a, &nine_digit, &can_b, &eight_digit)
    } else {
        (&can_b, &eight_digit, &can_a, &nine_digit)
    };
    let mut expected: Vec<String> = Vec::new();
    let mut expected_users: Vec<String> = Vec::new();
    for user in first_users {
        expected.push(
            first
                .join("userdata")
                .join(user)
                .join("900001")
                .display()
                .to_string(),
        );
        expected_users.push(user.clone());
    }
    let room = bindforge_codecs::MAX_USER_DIRS_PER_GAME - expected_users.len();
    for user in second_users.iter().take(room) {
        expected.push(
            second
                .join("userdata")
                .join(user)
                .join("900001")
                .display()
                .to_string(),
        );
        expected_users.push(user.clone());
    }
    let trees = user_trees(&context("900001", roots, root_a.canonical()));
    let actual: Vec<String> = trees
        .iter()
        .map(|tree| tree.root.display().to_string())
        .collect();
    assert_eq!(
        actual.len(),
        bindforge_codecs::MAX_USER_DIRS_PER_GAME,
        "17 located user trees across two roots keep exactly MAX_USER_DIRS_PER_GAME (the one silent bound)"
    );
    assert_eq!(
        actual, expected,
        "the kept trees are the first ones in roots-then-byte order"
    );
}
