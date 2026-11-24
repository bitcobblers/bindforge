//! The published surface of `bindforge-codecs` (§`crates/codecs` layout,
//! §The codec face, §Read discipline) and the `Cargo.lock` posture (§Chosen
//! design: "the only `Cargo.lock` delta is the `bindforge-codecs` stanza and
//! the `discovery-cli` dependency edge").
//!
//! proves: Test strategy #16.

mod common;

use bindforge_codecs::codec::{
    CodecFamily, Detection, Evidence, ExtractionRefusal, UserCandidates,
};
use bindforge_codecs::context::GameContext;
use bindforge_codecs::families::unreal_ini::UnrealIni;
use bindforge_codecs::paths::{TreeKind, UserTree, user_trees};
use bindforge_codecs::read::{decode, decode_owned, open_capped};
use bindforge_codecs::registry::{REGISTRY, claim, family_ids, is_known_family};
use bindforge_codecs::snapshot::{BindingSnapshot, load_snapshot};
use bindforge_codecs::{
    MAX_BINDING_FILE_OCTETS, MAX_CANDIDATE_FILES, MAX_LOCATE_DEPTH, MAX_LOCATE_ENTRIES_PER_TREE,
    MAX_SNAPSHOT_ENTRIES, MAX_SNAPSHOT_OCTETS, MAX_USER_DIRS_PER_GAME,
};
use discovery::game::{DiscoveredGame, Platform, ScanRecord};
use std::path::PathBuf;

// proves: Test strategy #16 (§`crates/codecs` layout) — the published surface
// resolves in value and call position: the trait, the claim payload, the
// refusal, the context, the resolvers, the reader, the loader, the registry.
#[test]
fn the_published_surface_resolves() {
    // Budgets, with the numbers the proposal's table states.
    let _: usize = MAX_BINDING_FILE_OCTETS;
    let _: usize = MAX_SNAPSHOT_ENTRIES;
    let _: usize = MAX_CANDIDATE_FILES;
    let _: usize = MAX_USER_DIRS_PER_GAME;
    let _: usize = MAX_LOCATE_DEPTH;
    let _: usize = MAX_LOCATE_ENTRIES_PER_TREE;
    let _: usize = MAX_SNAPSHOT_OCTETS;
    assert_eq!(MAX_BINDING_FILE_OCTETS, 1_048_576);
    assert_eq!(MAX_SNAPSHOT_ENTRIES, 4096);
    assert_eq!(MAX_CANDIDATE_FILES, 32);
    assert_eq!(MAX_USER_DIRS_PER_GAME, 16);
    assert_eq!(MAX_LOCATE_DEPTH, 8);
    assert_eq!(MAX_LOCATE_ENTRIES_PER_TREE, 16_384);
    assert_eq!(MAX_SNAPSHOT_OCTETS, 4_194_304);

    // The codec face and the shipped family.
    let family: &dyn CodecFamily = &UnrealIni;
    assert_eq!(family.id(), "unreal-ini");
    assert_eq!(
        family.appids(),
        &[] as &[&'static str],
        "empty this run (A21)"
    );

    // The registry seam.
    let _: &[&dyn CodecFamily] = REGISTRY;
    let _: Vec<&str> = family_ids();
    let _: bool = is_known_family("unreal-ini");

    // The context, resolvers and reader.
    let root = common::TempRoot::new("api-probe");
    let canonical = root.canonical();
    let ctx: GameContext = common::context("900001", vec![canonical.clone()], canonical.clone());
    let trees: Vec<UserTree> = user_trees(&ctx);
    let _ = trees
        .iter()
        .map(|tree| tree.kind)
        .collect::<Vec<TreeKind>>();
    let claim: Option<Detection> = claim(&ctx);
    let _ = claim.map(|claim| {
        let _: &str = claim.family;
        let _: &Evidence = &claim.evidence;
        let user: &UserCandidates = &claim.per_user[0];
        let _: &String = &user.user;
    });
    let _: Result<Vec<u8>, ExtractionRefusal> = open_capped(&root.write("x.ini", "x"));
    let _: Result<String, ExtractionRefusal> = decode(b"hello");
    // The owned-buffer sibling added by the round-1 fix pass (perf finding 7,
    // recorded in `03-build-notes.md`'s Ledger) is part of the surface too.
    let _: Result<String, ExtractionRefusal> = decode_owned(b"hello".to_vec());
    // The refusal prints by name (§Read discipline: "Named
    // `ExtractionRefusal` variants, each printed by name").
    let named = decode(&[0xFF, 0xFE, 0x00, 0xD8]);
    let named = named.expect_err("the unpaired surrogate refuses");
    let printed = format!("{named:?} {named}");
    assert!(
        printed.contains("UndecodableText"),
        "a refusal must print its variant name: {printed}"
    );

    // The record seam on the discovery side and its context constructor.
    let record = ScanRecord {
        game: DiscoveredGame {
            slug: "steam/900001".to_string(),
            friendly_name: "Example Game".to_string(),
            display_name: "Example Game".to_string(),
            install_path: PathBuf::from("/l/steamapps/common/Example Game"),
            platform: Platform::SteamLinux,
            codec_id: None,
        },
        root: canonical.clone(),
        library: canonical,
    };
    let _from_record: Option<GameContext> = GameContext::from_record(&record, &[]);

    // The loader and its model.
    let loaded: Result<BindingSnapshot, bindforge_codecs::snapshot::SnapshotRefusal> =
        load_snapshot(&root.path().join("absent.json"));
    assert!(loaded.is_err(), "a missing document is a load failure");
}

// proves: Test strategy #16 (§`crates/codecs`: "the only `Cargo.lock` delta is
// the `bindforge-codecs` stanza and the `discovery-cli` dependency edge") — the
// lockfile gains the package stanza, and `discovery-cli` names the crate.
// Any other new package or moved resolved version is a finding (§Rollout 9).
#[test]
fn the_lockfile_carries_the_stanza_and_the_cli_edge() {
    let lock = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|crates| crates.parent())
            .expect("workspace root")
            .join("Cargo.lock"),
    )
    .expect("Cargo.lock reads");
    assert!(
        lock.contains("name = \"bindforge-codecs\""),
        "the lockfile must carry the bindforge-codecs stanza"
    );
    let cli_stanza = lock
        .split("[[package]]")
        .find(|stanza| stanza.contains("name = \"discovery-cli\""))
        .expect("the discovery-cli stanza");
    assert!(
        cli_stanza.contains("bindforge-codecs"),
        "the only new edge the CLI gains is the dependency on bindforge-codecs"
    );
}
