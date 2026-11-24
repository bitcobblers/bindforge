//! The `unreal-ini` family — the locate walk and the merge-operator fold
//! (§The first family).
//!
//! proves: Test strategy #6 (locate walk), #7 (fold), #4 (`MAX_CANDIDATE_FILES`
//! and `MAX_SNAPSHOT_ENTRIES` seams), and #5 (the UTF-16 fixture twin).

mod common;

use bindforge_codecs::codec::{ExtractionRefusal, UserCandidates};
use bindforge_codecs::context::GameContext;
use bindforge_codecs::families::unreal_ini::UnrealIni;
use bindforge_codecs::{
    MAX_CANDIDATE_FILES, MAX_LOCATE_DEPTH, MAX_LOCATE_ENTRIES_PER_TREE, MAX_SNAPSHOT_ENTRIES,
};
use bindforge_core::MAX_NATIVE_NAME_OCTETS;
use common::{
    TempRoot, all_entries, context, docs_json, expect_refusal, extract_json, fixture_tree,
    located_files, player_input, unreal_claim,
};
use std::path::PathBuf;

/// One temp root carrying a single user tree at `Saved/Config/LinuxNoEditor/Game.ini`.
fn single_store(label: &str, appid: &str, ini: &str) -> (TempRoot, GameContext) {
    let root = TempRoot::new(label);
    let relative =
        format!("userdata/76561197960287930/{appid}/Saved/Config/LinuxNoEditor/Game.ini");
    root.write(&relative, ini);
    let canonical = root.canonical();
    let ctx = context(appid, vec![canonical.clone()], canonical);
    (root, ctx)
}

// ---------------------------------------------------------------------------
// Locate walk (§The first family, Test strategy #6)
// ---------------------------------------------------------------------------

// proves: Test strategy #6 (§The first family) — a candidate is an `*.ini`
// whose components end in `Saved/Config/<one directory>/<name>.ini`; the
// grammar is located at ≤ `MAX_LOCATE_DEPTH` tree-relative components.
#[test]
fn the_saved_config_grammar_is_located_within_the_depth_bound() {
    let root = TempRoot::new("locate-grammar");
    let appid = "900011";
    let ini = &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]);
    // 4 tree-relative components — the ordinary shape.
    root.write(
        &format!("userdata/76561197960287930/{appid}/Saved/Config/LinuxNoEditor/Engine.ini"),
        ini,
    );
    // Exactly `MAX_LOCATE_DEPTH` components: the last four are the grammar,
    // the first four are padding inside the tree.
    let padded =
        format!("userdata/76561197960287930/{appid}/a/b/c/d/Saved/Config/Android/Game.ini");
    assert_eq!(
        PathBuf::from(&padded)
            .strip_prefix(format!("userdata/76561197960287930/{appid}"))
            .expect("inside the tree")
            .components()
            .count(),
        MAX_LOCATE_DEPTH,
        "the fixture row must sit exactly at the depth bound"
    );
    root.write(&padded, ini);
    let canonical = root.canonical();
    let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical));
    let located = located_files(&claim);
    assert_eq!(
        located.len(),
        2,
        "both the 4-component and the 8-component file locate"
    );
    assert!(
        located
            .iter()
            .any(|file| file.ends_with("LinuxNoEditor/Engine.ini"))
    );
    assert!(
        located
            .iter()
            .any(|file| file.ends_with("Android/Game.ini"))
    );
}

// proves: Test strategy #6 (§Read discipline's `MAX_LOCATE_DEPTH`: "components
// in a located file's tree-relative path") — the grammar is not located beyond
// the depth bound.
#[test]
fn the_grammar_is_not_located_beyond_the_depth_bound() {
    let root = TempRoot::new("locate-deep");
    let appid = "900012";
    let ini = &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]);
    // 9 tree-relative components — one past `MAX_LOCATE_DEPTH`.
    root.write(
        &format!("userdata/76561197960287930/{appid}/a/b/c/d/e/Saved/Config/Android/Game.ini"),
        ini,
    );
    let canonical = root.canonical();
    let claim = UnrealIni.detect(&context(appid, vec![canonical.clone()], canonical));
    assert!(
        claim.is_none(),
        "a walk prunes a directory whose tree-relative path is past the depth bound"
    );
}

// proves: Test strategy #6 (§The first family: "No platform-directory constant
// is assumed (A15)") — a platform directory name no fixture uses is still found.
#[test]
fn an_unusual_platform_directory_name_is_still_located() {
    let root = TempRoot::new("locate-platform");
    let appid = "900013";
    let ini = &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]);
    root.write(
        &format!("userdata/76561197960287930/{appid}/Saved/Config/WeirdPlatform-7/Engine.ini"),
        ini,
    );
    let canonical = root.canonical();
    let claim = UnrealIni.detect(&context(appid, vec![canonical.clone()], canonical));
    assert!(
        claim.is_some(),
        "the platform directory name is a wildcard, not a constant"
    );
}

// proves: Test strategy #6 (§The first family: "A decodable file without the
// section is not located") — out-of-grammar `.ini` files and non-`.ini` files
// contribute nothing.
#[test]
fn out_of_grammar_files_contribute_nothing() {
    let root = TempRoot::new("locate-grammar-miss");
    let appid = "900014";
    let tree = format!("userdata/76561197960287930/{appid}");
    let ini = &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]);
    root.write(&format!("{tree}/Saved/Config/Game.ini"), ini); // no platform dir
    root.write(&format!("{tree}/Saved/Engine.ini"), ini); // no Config dir
    root.write(&format!("{tree}/Config/LinuxNoEditor/Engine.ini"), ini); // no Saved
    root.write(
        &format!("{tree}/Saved/Config/LinuxNoEditor/Notes.txt"),
        ini, // right place, wrong extension
    );
    root.write(
        &format!("{tree}/Saved/Config/LinuxNoEditor/PlainSettings.ini"),
        "[/Script/Engine.GameUserSettings]\nbUseVSync=False\n", // no section+Bindings
    );
    let canonical = root.canonical();
    let claim = UnrealIni.detect(&context(appid, vec![canonical.clone()], canonical));
    assert!(
        claim.is_none(),
        "nothing matching the path grammar plus section is located"
    );
}

// proves: Test strategy #6 and #8 (§The first family: "The install tree is
// never a candidate tree") — an install tree holding a `DefaultInput.ini` with
// the full grammar and section contributes no candidate: defaults-only yields
// its user tree's entries only, never the defaults'.
#[test]
fn the_install_tree_contributes_no_candidates() {
    let fixture = fixture_tree("defaults-only");
    let claim = unreal_claim(&context("900004", vec![fixture.clone()], fixture.clone()));
    let located = located_files(&claim);
    assert!(
        located
            .iter()
            .all(|file| !file.contains("steamapps/common")),
        "the install tree was located: {located:?}"
    );
    let docs = docs_json(
        &UnrealIni
            .extract(&claim)
            .expect("the user tree must extract"),
    );
    let entries = all_entries(&docs);
    assert_eq!(
        entries,
        vec![("Interact".to_string(), vec!["E".to_string()])],
        "only the user tree's pair is stated; the install DefaultInput.ini contributes nothing"
    );
}

// proves: Test strategy #6 (§Config-path resolvers / §Read discipline) — a
// symlinked directory and a symlinked `.ini` are neither descended nor read.
#[cfg(unix)]
#[test]
fn symlinked_directories_and_files_are_never_followed() {
    use std::os::unix::fs::symlink;
    let outside = TempRoot::new("locate-link-outside");
    outside.write(
        "LinuxNoEditor/Engine.ini",
        &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]),
    );
    let root = TempRoot::new("locate-link");
    let appid = "900015";
    let tree = format!("userdata/76561197960287930/{appid}");
    root.mkdir(&format!("{tree}/Saved/Config"));
    // (a) the platform directory is a link out of the tree — not descended.
    symlink(
        outside.path(),
        root.join(&format!("{tree}/Saved/Config/LinuxNoEditor")),
    )
    .expect("symlink a config directory");
    let canonical = root.canonical();
    let claim = UnrealIni.detect(&context(appid, vec![canonical.clone()], canonical));
    assert!(claim.is_none(), "a symlinked directory is never descended");

    // (b) the `.ini` itself is a link out of the tree — never read.
    let root_b = TempRoot::new("locate-link-file");
    let tree_b = format!("userdata/76561197960287930/{appid}");
    root_b.mkdir(&format!("{tree_b}/Saved/Config/LinuxNoEditor"));
    symlink(
        outside.join("LinuxNoEditor/Engine.ini"),
        root_b.join(&format!("{tree_b}/Saved/Config/LinuxNoEditor/Engine.ini")),
    )
    .expect("symlink an ini");
    let claim_b = UnrealIni.detect(&context(
        appid,
        vec![root_b.canonical()],
        root_b.canonical(),
    ));
    assert!(
        claim_b.is_none(),
        "a symlinked .ini is never read as a candidate"
    );
}

#[cfg(not(unix))]
#[test]
fn symlinked_directories_and_files_are_never_followed() {
    // The link fixtures are made at test time behind `#[cfg(unix)]`; the
    // Windows-visible half of this face is the compile-check leg's (§Platform).
}

// proves: Test strategy #6 (§The first family: "a candidate is an `*.ini`
// whose `Path::components()` end component-wise in `Saved/Config/<one
// directory>/<name>.ini`") — pinned on raw OS-named bytes: a non-UTF-8 file
// name and a non-UTF-8 platform-directory name locate at their real paths and
// extract, per the accepted compatibility finding A (round 1: the walk queues
// and pushes `DirEntry::path()`, keeping the lossy string as the sort key
// only). Test-update item "Non-UTF-8 file and directory names (compat-A)" in
// `03-build-notes.md`.
#[cfg(unix)]
#[test]
fn byte_named_directories_and_files_locate_at_their_real_paths() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = TempRoot::new("locate-bytename");
    let appid = "900032";
    let config = root
        .path()
        .join(format!("userdata/76561197960287930/{appid}/Saved/Config"));
    let platform = config.join("LinuxNoEditor");
    std::fs::create_dir_all(&platform).expect("platform dir");
    // `weird\xffname.ini` — a file name the host filesystem names in raw bytes.
    let odd_file = OsString::from_vec(b"weird\xffname.ini".to_vec());
    let stating = platform.join(&odd_file);
    std::fs::write(
        &stating,
        player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]).as_bytes(),
    )
    .expect("byte-named ini");
    // `d\xffr` — a platform directory named in raw bytes, holding an ordinary file.
    let odd_dir = OsString::from_vec(b"d\xffr".to_vec());
    let byte_dir = config.join(&odd_dir);
    std::fs::create_dir_all(&byte_dir).expect("byte-named platform dir");
    let in_byte_dir = byte_dir.join("Engine.ini");
    std::fs::write(
        &in_byte_dir,
        player_input(&["+Bindings=(Name=\"MouseX\",Command=\"TurnRight\")"]).as_bytes(),
    )
    .expect("ini under a byte-named dir");

    let canonical = root.canonical();
    let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical.clone()));
    let located: Vec<PathBuf> = claim
        .per_user
        .iter()
        .flat_map(|user| user.files.clone())
        .collect();
    // The candidates are the real paths — under the canonicalized anchor the
    // context carries — not lossy stand-ins and not skipped subtrees.
    let expect = |relative: &std::path::Path| -> PathBuf {
        let stripped = relative.strip_prefix(root.path()).expect("under the root");
        canonical.join(stripped)
    };
    assert!(
        located.contains(&expect(&stating)),
        "the byte-named file must locate at its real path: {located:?}"
    );
    assert!(
        located.contains(&expect(&in_byte_dir)),
        "the file under the byte-named directory must locate: {located:?}"
    );

    let docs = UnrealIni
        .extract(&claim)
        .expect("both byte-named candidates extract");
    assert_eq!(
        all_entries(&docs_json(&docs)),
        vec![
            ("MoveForward".to_string(), vec!["W".to_string()]),
            ("TurnRight".to_string(), vec!["MouseX".to_string()]),
        ],
        "the byte-named subtree's pairs fold exactly like any other"
    );
}

#[cfg(not(unix))]
#[test]
fn byte_named_directories_and_files_locate_at_their_real_paths() {
    // Raw byte names are the `#[cfg(unix)]` half of this pin (§Platform).
}

// proves: Test strategy #7 and #13 (§Read discipline: the `UnparsableEntry`
// echo "escaped and truncated through `bindforge_core::escape_and_truncate`")
// — a control byte inside a located directory name renders escaped: the
// refusal's `Display` stays single-line and control-free, per the accepted
// security finding 2 (round 1). Test-update item "Escaped refusal echoes
// (security-2)" in `03-build-notes.md`.
#[cfg(unix)]
#[test]
fn a_control_byte_in_an_echoed_path_is_escaped_and_stays_one_line() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = TempRoot::new("echo-control-byte");
    let appid = "900033";
    let odd_dir = OsString::from_vec(b"Weird\x1bDir".to_vec());
    let dir = root
        .path()
        .join(format!("userdata/76561197960287930/{appid}/Saved/Config"))
        .join(&odd_dir);
    std::fs::create_dir_all(&dir).expect("control-byte dir");
    std::fs::write(
        dir.join("Engine.ini"),
        player_input(&["Bindings=(Name=W,Command=NoQuotes)"]).as_bytes(),
    )
    .expect("refusing ini");

    let canonical = root.canonical();
    let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical));
    let refusal = expect_refusal(
        &claim,
        |refusal| matches!(refusal, ExtractionRefusal::UnparsableEntry { .. }),
        "a refusing tuple located under a control-byte directory name",
    );
    let message = format!("{refusal}");
    assert_eq!(
        message.lines().count(),
        1,
        "the echo must be one line: {message:?}"
    );
    assert!(
        !message.chars().any(char::is_control),
        "the echoed path must be control-free, got {message:?}"
    );
    assert!(
        message.contains("Engine.ini") && message.contains("Weird"),
        "the echo still names the refusing file: {message:?}"
    );
}

#[cfg(not(unix))]
#[test]
fn a_control_byte_in_an_echoed_path_is_escaped_and_stays_one_line() {
    // Control-byte directory names are the `#[cfg(unix)]` half of this pin
    // (§Platform).
}

// proves: Test strategy #6 (§The first family: "examining at most
// `MAX_LOCATE_ENTRIES_PER_TREE` entries per tree … a walk that reaches the
// budget stops and counts the rest") — a walk at the entry budget ends in a
// named outcome, never a loop: the candidate hidden behind `MAX_LOCATE_ENTRIES_PER_TREE`
// earlier-sorting entries is not located.
#[test]
fn the_locate_walk_stops_at_the_entry_budget() {
    let root = TempRoot::new("locate-budget");
    let appid = "900016";
    let tree = format!("userdata/76561197960287930/{appid}");
    // `MAX_LOCATE_ENTRIES_PER_TREE` shallow entries sort ahead of `Saved`.
    for index in 0..MAX_LOCATE_ENTRIES_PER_TREE {
        root.write_bytes(&format!("{tree}/a{index:06}"), b"x");
    }
    root.write(
        &format!("{tree}/Saved/Config/LinuxNoEditor/Engine.ini"),
        &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]),
    );
    let canonical = root.canonical();
    let claim = UnrealIni.detect(&context(appid, vec![canonical.clone()], canonical));
    match claim {
        None => {}
        Some(claim) => {
            assert!(
                located_files(&claim)
                    .iter()
                    .all(|file| !file.ends_with("Saved/Config/LinuxNoEditor/Engine.ini")),
                "the budget must cap what the walk examines; the hidden candidate may not be located"
            );
        }
    }
}

// proves: Test strategy #4 (§The first family: "At most `MAX_CANDIDATE_FILES`
// path-grammar matches per tree") — a tree holding more grammar matches than
// `MAX_CANDIDATE_FILES` locates only the cap, in the walk's byte order.
#[test]
fn a_tree_locates_at_most_the_candidate_cap() {
    let root = TempRoot::new("locate-cap");
    let appid = "900017";
    let tree = format!("userdata/76561197960287930/{appid}");
    let ini = &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]);
    for index in 0..MAX_CANDIDATE_FILES + 1 {
        root.write(
            &format!("{tree}/Saved/Config/LinuxNoEditor/f{index:02}.ini"),
            ini,
        );
    }
    let canonical = root.canonical();
    let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical));
    assert_eq!(
        claim.per_user[0].files.len(),
        MAX_CANDIDATE_FILES,
        "a further path-grammar match is skipped and counted, not located"
    );
}

// proves: Test strategy #4 and #6 (§The first family: "At most
// `MAX_CANDIDATE_FILES` path-grammar matches per tree, in the walk's byte
// order, are opened through `read::open_capped` and `read::decode` for that
// content test; a further match is skipped and counted") — the cap bites on
// the opens, independent of what locates, per the accepted performance
// finding 1 (round 1): grammar matches that never state the section still
// consume the open budget. Test-update item "Open-cap independent of
// locating (perf-1)" in `03-build-notes.md`.
#[test]
fn the_open_cap_bites_on_matches_that_never_locate() {
    let quiet = "[/Script/Engine.GameUserSettings]\nbUseVSync=False\n";
    let stating = &player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]);

    // The one stating file sorts past `MAX_CANDIDATE_FILES` quiet matches:
    // it is never opened, so it is never located, so the tree claims nothing.
    let root = TempRoot::new("locate-open-cap-past");
    let appid = "900019";
    let tree = format!("userdata/76561197960287930/{appid}/Saved/Config/LinuxNoEditor");
    for index in 0..MAX_CANDIDATE_FILES {
        root.write(&format!("{tree}/f{index:02}.ini"), quiet);
    }
    root.write(&format!("{tree}/f{MAX_CANDIDATE_FILES:02}.ini"), stating);
    let canonical = root.canonical();
    let claim = UnrealIni.detect(&context(appid, vec![canonical.clone()], canonical));
    assert!(
        claim.is_none(),
        "the stating file sits past the open cap and is skipped and counted, never opened"
    );

    // Inside the cap the same stating file opens and locates: the refusal
    // above is the cap, not a broken content test.
    let root = TempRoot::new("locate-open-cap-inside");
    let appid = "900020";
    let tree = format!("userdata/76561197960287930/{appid}/Saved/Config/LinuxNoEditor");
    for index in 0..MAX_CANDIDATE_FILES - 1 {
        root.write(&format!("{tree}/f{index:02}.ini"), quiet);
    }
    root.write(
        &format!("{tree}/f{:02}.ini", MAX_CANDIDATE_FILES - 1),
        stating,
    );
    let canonical = root.canonical();
    let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical));
    assert!(
        located_files(&claim)
            .iter()
            .any(|file| file.ends_with(&format!("f{:02}.ini", MAX_CANDIDATE_FILES - 1))),
        "the stating file inside the cap must be opened and located"
    );
}

// proves: Test strategy #4 (`017-B3`'s type invariant) — `UserCandidates::push`
// refuses the 33rd file behind the walk's skip-and-count.
#[test]
fn user_candidates_refuse_a_33rd_file() {
    let mut candidates = UserCandidates {
        user: "76561197960287930".to_string(),
        files: Vec::new(),
        tree_root: PathBuf::from("/tree"),
    };
    for index in 0..MAX_CANDIDATE_FILES {
        assert!(
            candidates
                .push(PathBuf::from(format!("/tree/f{index:02}.ini")))
                .is_ok(),
            "file {} of MAX_CANDIDATE_FILES must be admitted",
            index + 1
        );
    }
    assert!(
        candidates.push(PathBuf::from("/tree/over.ini")).is_err(),
        "the 33rd file breaches MAX_CANDIDATE_FILES and is refused"
    );
    assert_eq!(candidates.files.len(), MAX_CANDIDATE_FILES);
}

// proves: Test strategy #6 (§Config-path resolvers and §Read discipline: "A
// match the reader refuses cannot pass the content test either, so it is
// located on path grammar alone and `extract` refuses it by name") — an
// over-budget file is located and refused as `FileOverBudget`, never unseen.
#[test]
fn a_reader_refused_match_is_located_and_refused_by_name() {
    let root = TempRoot::new("locate-over-reader");
    let appid = "900018";
    let relative =
        format!("userdata/76561197960287930/{appid}/Saved/Config/LinuxNoEditor/Engine.ini");
    let mut bytes = player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]).into_bytes();
    bytes.resize(bindforge_codecs::MAX_BINDING_FILE_OCTETS + 1, b' ');
    root.write_bytes(&relative, &bytes);
    let canonical = root.canonical();
    let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical));
    expect_refusal(
        &claim,
        |refusal| matches!(refusal, ExtractionRefusal::FileOverBudget { .. }),
        "a located file over MAX_BINDING_FILE_OCTETS",
    );
}

// ---------------------------------------------------------------------------
// The fold (§The first family, Test strategy #7)
// ---------------------------------------------------------------------------

// proves: Test strategy #7 (§`bindings/v1` Input spelling, A20) — the shipped
// fixture's `+Bindings` pairs group by the `Command` string verbatim and the
// `Name` tokens stay verbatim: `MouseX`, `Left`, `Gamepad_RightX` and `TAB` are
// the file's own words and are *not* refused.
#[test]
fn native_tokens_survive_verbatim_and_group_by_command() {
    let fixture = fixture_tree("steam-userdata");
    let docs = extract_json(&context("900001", vec![fixture.clone()], fixture.clone()));
    assert_eq!(
        all_entries(&docs),
        vec![
            ("AltDrop".to_string(), vec!["Q".to_string()]),
            ("GenericButton0".to_string(), vec!["F".to_string()]),
            ("Jump".to_string(), vec!["SpaceBar".to_string()]),
            ("MoveForward".to_string(), vec!["W".to_string()]),
            ("ShowScores".to_string(), vec!["TAB".to_string()]),
            ("TurnLeft".to_string(), vec!["Left".to_string()]),
            (
                "TurnRight".to_string(),
                vec!["Gamepad_RightX".to_string(), "MouseX".to_string()]
            ),
        ],
        "every pair the user files state, verbatim, grouped and sorted"
    );
}

// proves: Test strategy #7 (§The first family) — `-Bindings` removes an exact pair.
#[test]
fn the_minus_operator_removes_an_exact_pair() {
    let (_root, ctx) = single_store(
        "fold-minus",
        "900021",
        &player_input(&[
            "+Bindings=(Name=\"E\",Command=\"Interact\")",
            "+Bindings=(Name=\"F\",Command=\"Interact\")",
            "-Bindings=(Name=\"E\",Command=\"Interact\")",
        ]),
    );
    let docs = extract_json(&ctx);
    assert_eq!(
        all_entries(&docs),
        vec![("Interact".to_string(), vec!["F".to_string()])]
    );
}

// proves: Test strategy #7 (§The first family: "`-` removes on an exact
// match") — a removal naming no pair the working set holds is a no-op, pinned
// at scale per the accepted performance finding 3 (round 1: `Fold::remove`
// returns when the pair was never admitted). Test-update item
// "Fold::remove no-op path" in `03-build-notes.md`.
#[test]
fn removals_of_absent_pairs_are_no_ops() {
    let mut lines: Vec<String> = (0..2000)
        .map(|index| format!("-Bindings=(Name=\"X{index}\",Command=\"NeverAdded\")"))
        .collect();
    lines.push("+Bindings=(Name=\"W\",Command=\"MoveForward\")".to_string());
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    let (_root, ctx) = single_store("fold-removal-noop", "900036", &player_input(&lines));
    let docs = extract_json(&ctx);
    assert_eq!(
        all_entries(&docs),
        vec![("MoveForward".to_string(), vec!["W".to_string()])],
        "2000 removals of absent pairs change nothing and refuse nothing"
    );
}

// proves: Test strategy #7 (§The first family: "`!Key=<tuple>` removes only the
// pairs it names") — a `!Bindings=(…)` tuple removes just the named pair.
#[test]
fn the_bang_tuple_removes_only_named_pairs() {
    let (_root, ctx) = single_store(
        "fold-bang-tuple",
        "900022",
        &player_input(&[
            "+Bindings=(Name=\"Q\",Command=\"DropItem\")",
            "+Bindings=(Name=\"Q\",Command=\"AltDrop\")",
            "!Bindings=(Name=\"Q\",Command=\"DropItem\")",
        ]),
    );
    let docs = extract_json(&ctx);
    assert_eq!(
        all_entries(&docs),
        vec![("AltDrop".to_string(), vec!["Q".to_string()])]
    );
}

// proves: Test strategy #7 (§The first family: "a bare `!Key=` (no value)
// clears every pair accumulated for that key") — a bare `!Bindings=` wipes the
// key's whole working set.
#[test]
fn a_bang_with_no_value_clears_the_key() {
    let (_root, ctx) = single_store(
        "fold-bang-clear",
        "900023",
        &player_input(&[
            "+Bindings=(Name=\"Q\",Command=\"DropItem\")",
            "+Bindings=(Name=\"E\",Command=\"Interact\")",
            "!Bindings=",
        ]),
    );
    let docs = extract_json(&ctx);
    assert!(
        all_entries(&docs).is_empty(),
        "a cleared key leaves no entry"
    );
}

// proves: Test strategy #7 (§The first family: "a bare `+`, `-` or `.` operand
// is out-of-grammar and skipped") — bare operands skip without refusing.
#[test]
fn a_bare_operand_is_skipped_not_refused() {
    let (_root, ctx) = single_store(
        "fold-bare",
        "900024",
        &player_input(&[
            "Bindings=",
            "+Bindings=",
            "-Bindings=",
            ".Bindings=",
            "+Bindings=(Name=\"W\",Command=\"MoveForward\")",
        ]),
    );
    let docs = extract_json(&ctx);
    assert_eq!(
        all_entries(&docs),
        vec![("MoveForward".to_string(), vec!["W".to_string()])],
        "the bare lines are out-of-grammar skips, not drift"
    );
}

// proves: Test strategy #7 (§The first family: "a `Command` whose pairs the
// fold has all removed leaves no entry").
#[test]
fn a_command_fully_removed_leaves_no_entry() {
    let (_root, ctx) = single_store(
        "fold-empty",
        "900025",
        &player_input(&[
            "+Bindings=(Name=\"E\",Command=\"Interact\")",
            "+Bindings=(Name=\"F\",Command=\"Interact\")",
            "-Bindings=(Name=\"E\",Command=\"Interact\")",
            "-Bindings=(Name=\"F\",Command=\"Interact\")",
        ]),
    );
    let docs = extract_json(&ctx);
    assert!(
        all_entries(&docs).is_empty(),
        "every pair removed leaves no entry"
    );
}

// proves: Test strategy #7 (§The first family: "`.` adds and may duplicate") —
// a `.`-added pair collapses with the identical one already in the set.
#[test]
fn a_dot_add_collapses_with_the_pair_it_repeats() {
    let (_root, ctx) = single_store(
        "fold-dot",
        "900026",
        &player_input(&[
            "+Bindings=(Name=\"MouseX\",Command=\"TurnRight\")",
            ".Bindings=(Name=\"MouseX\",Command=\"TurnRight\")",
            "+Bindings=(Name=\"MouseY\",Command=\"TurnRight\")",
            ".Bindings=(Name=\"MouseY\",Command=\"TurnRight\")",
            ".Bindings=(Name=\"Left\",Command=\"TurnRight\")",
        ]),
    );
    let docs = extract_json(&ctx);
    assert_eq!(
        all_entries(&docs),
        vec![(
            "TurnRight".to_string(),
            vec![
                "Left".to_string(),
                "MouseX".to_string(),
                "MouseY".to_string()
            ]
        )],
        "duplicates collapse to one stated pair each"
    );
}

// proves: Test strategy #7 (§Chosen design: "dedupe byte-identical pairs only")
// — duplicate identical `+` pairs collapse.
#[test]
fn duplicate_identical_pairs_collapse() {
    let (_root, ctx) = single_store(
        "fold-dupe",
        "900027",
        &player_input(&[
            "+Bindings=(Name=\"W\",Command=\"MoveForward\")",
            "+Bindings=(Name=\"W\",Command=\"MoveForward\")",
        ]),
    );
    let docs = extract_json(&ctx);
    assert_eq!(
        all_entries(&docs),
        vec![("MoveForward".to_string(), vec!["W".to_string()])]
    );
}

// proves: Test strategy #7 (§Chosen design: entries are keyed by the `Command`
// string) — one key bound to two commands keeps both entries.
#[test]
fn one_key_bound_to_two_commands_keeps_both() {
    let (_root, ctx) = single_store(
        "fold-onekey",
        "900028",
        &player_input(&[
            "+Bindings=(Name=\"Q\",Command=\"DropItem\")",
            "+Bindings=(Name=\"Q\",Command=\"AltDrop\")",
        ]),
    );
    let docs = extract_json(&ctx);
    assert_eq!(
        all_entries(&docs),
        vec![
            ("AltDrop".to_string(), vec!["Q".to_string()]),
            ("DropItem".to_string(), vec!["Q".to_string()]),
        ]
    );
}

// proves: Test strategy #7 (§The first family: a `Bindings=(…)` tuple that
// lacks a quoted `Name=` or `Command=` … is `UnparsableEntry`) — the
// `broken-tuple` fixture refuses naming the tree-relative path and line.
#[test]
fn a_broken_tuple_is_an_unparsable_entry_naming_file_and_line() {
    let fixture = fixture_tree("broken-tuple");
    let claim = unreal_claim(&context("900006", vec![fixture.clone()], fixture.clone()));
    let refusal = expect_refusal(
        &claim,
        |refusal| matches!(refusal, ExtractionRefusal::UnparsableEntry { .. }),
        "an unquoted tuple",
    );
    let message = format!("{refusal:?} {refusal}");
    assert!(
        message.contains("Saved/Config/LinuxNoEditor/Engine.ini"),
        "the refusal names the file's tree-relative path: {message}"
    );
    assert!(
        message.contains("3"),
        "the refusal names the offending 1-based line (line 3): {message}"
    );
}

// proves: Test strategy #7 (§Read discipline: the `UnparsableEntry` "names the
// bound's name when the verdict is a verbatim `Command=` or `Name=` string over
// `MAX_NATIVE_NAME_OCTETS`") — an over-bound verbatim token refuses.
#[test]
fn an_over_bound_verbatim_token_is_an_unparsable_entry() {
    let long_name = "n".repeat(MAX_NATIVE_NAME_OCTETS + 1);
    let (_root, ctx) = single_store(
        "fold-over-name",
        "900029",
        &player_input(&[&format!(
            "+Bindings=(Name=\"{long_name}\",Command=\"TooLong\")"
        )]),
    );
    let claim = unreal_claim(&ctx);
    expect_refusal(
        &claim,
        |refusal| matches!(refusal, ExtractionRefusal::UnparsableEntry { .. }),
        "a Name= string over MAX_NATIVE_NAME_OCTETS",
    );
}

// proves: Test strategy #4 (§Read discipline, `017-B2`: `MAX_SNAPSHOT_ENTRIES`
// counts distinct pairs *as admitted*) — a fold that admits
// `MAX_SNAPSHOT_ENTRIES + 1` pairs mid-fold names itself as
// `SnapshotEntryBudget`: one entry with 4097 inputs breaches it as surely as
// 4097 entries do.
#[test]
fn the_fold_names_itself_at_the_snapshot_entry_budget() {
    let many_inputs: Vec<String> = (0..MAX_SNAPSHOT_ENTRIES + 1)
        .map(|index| format!("+Bindings=(Name=\"K{index}\",Command=\"OneAction\")"))
        .collect();
    let lines: Vec<&str> = many_inputs.iter().map(String::as_str).collect();
    let (_root, ctx) = single_store("fold-budget-inputs", "900030", &player_input(&lines));
    let claim = unreal_claim(&ctx);
    expect_refusal(
        &claim,
        |refusal| matches!(refusal, ExtractionRefusal::SnapshotEntryBudget { .. }),
        "one entry with MAX_SNAPSHOT_ENTRIES + 1 inputs",
    );

    let many_entries: Vec<String> = (0..MAX_SNAPSHOT_ENTRIES + 1)
        .map(|index| format!("+Bindings=(Name=\"K\",Command=\"A{index}\")"))
        .collect();
    let lines: Vec<&str> = many_entries.iter().map(String::as_str).collect();
    let (_root, ctx) = single_store("fold-budget-entries", "900031", &player_input(&lines));
    let claim = unreal_claim(&ctx);
    expect_refusal(
        &claim,
        |refusal| matches!(refusal, ExtractionRefusal::SnapshotEntryBudget { .. }),
        "MAX_SNAPSHOT_ENTRIES + 1 entries",
    );
}

// proves: Test strategy #5 (§Read discipline / A19) — the `bom-utf16` fixture's
// entries survive the BOM sniffing: its UTF-16LE and UTF-16BE forms extract
// the same documents as its UTF-8 twin, and the UTF-8 BOM form too.
#[test]
fn the_utf16_fixture_extracts_the_same_entries_as_its_utf8_twin() {
    let text = std::fs::read_to_string(fixture_tree("bom-utf16").join("Engine.ini"))
        .expect("the fixture source reads");

    let mut variants: Vec<(&str, Vec<u8>)> = vec![
        ("utf8", text.as_bytes().to_vec()),
        ("utf8-bom", common::utf8_with_bom(&text)),
        ("utf16le", common::utf16le_with_bom(&text)),
        ("utf16be", common::utf16be_with_bom(&text)),
    ];
    let mut results: Vec<(String, Vec<String>)> = Vec::new();
    for (label, bytes) in variants.drain(..) {
        let root = TempRoot::new(&format!("bom-{label}"));
        root.write_bytes(
            "userdata/76561197960287930/900003/Saved/Config/LinuxNoEditor/Engine.ini",
            &bytes,
        );
        let canonical = root.canonical();
        let docs = extract_json(&context("900003", vec![canonical.clone()], canonical));
        results.push((label.to_string(), docs));
    }
    let baseline_label = results[0].0.clone();
    let baseline = &results[0].1;
    for (label, docs) in &results[1..] {
        assert_eq!(
            docs, baseline,
            "the {label} form extracted differently from the {baseline_label} twin"
        );
    }
    assert_eq!(
        all_entries(baseline),
        vec![
            ("ScrollUp".to_string(), vec!["PGUP".to_string()]),
            ("ShowInventory".to_string(), vec!["TAB".to_string()]),
            ("TurnRight".to_string(), vec!["MouseX".to_string()]),
        ]
    );
}
