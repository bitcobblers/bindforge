//! The retirement sweep of design 025 / ADR-025 (Test strategy #14): the `024`
//! identity machinery — `derive_slug`, the `Namespace` type, the `namespace` field,
//! and `resolve_collisions` — must resolve nowhere in `crates/discovery`'s or
//! `apps/discovery-cli`'s `src/`, and the suites that pinned them must take no part
//! in any build (deleted outright or `required-features`-gated off every command the
//! gates run; file removal is the manifest's named operator follow-up).
//!
//! proves: design 025's Test strategy #14 (Rollout 1: "the deletion of `slug.rs`,
//! `Namespace`, `resolve_collisions` and their suites, the module-doc reversals") —
//! this is the gate for that retirement: it reads the shipped source texts and the
//! package manifest, and it fails while any retired name still stands in a compiled
//! surface or any retired suite still builds. The positive half — the new surface
//! resolving — is `zz_api_probe.rs`; the behavioral re-pins live in `game_tests.rs`,
//! `emission_tests.rs`, `plugin_tests.rs`, `steam_linux_tests.rs`, and
//! `steam_windows_tests.rs`, whose assertions name `steam/<appid>` only.
//!
//! Unit-level: reads checked-in source texts, writes nothing, no network. (Reading
//! the shipped text here is the sweep's subject — the corpus-sweep pattern of
//! `crates/core/tests/ledger_annotations.rs`, pointed at the retired names.)

use std::path::{Path, PathBuf};

/// The workspace root: `crates/discovery` sits two levels below it.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/discovery sits directly under the workspace root")
        .to_path_buf()
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(dir).unwrap_or_else(|err| panic!("reading {dir:?}: {err}"));
    for entry in entries {
        let path = entry.expect("readable entry").path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// proofs: design 025's Test strategy #14 — `derive_slug`, the `discovery::slug`
/// module, `resolve_collisions`, and the `Namespace` type resolve nowhere in the two
/// shipped source trees: the derivation, the collision pass, and the namespace enum
/// retire with `025-C1`/`025-C2`, and the module-doc reversal removes their last
/// sentences from the tree.
#[test]
fn the_retired_identity_names_resolve_in_no_shipped_source_tree() {
    let mut sources = rust_files(&repo_root().join("crates").join("discovery").join("src"));
    sources.extend(rust_files(
        &repo_root().join("apps").join("discovery-cli").join("src"),
    ));
    assert!(
        !sources.is_empty(),
        "the sweep must read the shipped source trees"
    );

    let retired = [
        "derive_slug",
        "resolve_collisions",
        "Namespace",
        "slug::",
        "mod slug",
    ];
    let mut violations: Vec<String> = Vec::new();
    for path in &sources {
        let text = std::fs::read_to_string(path).expect("source text is readable");
        for name in retired {
            if text.contains(name) {
                violations.push(format!("{} still mentions `{name}`", path.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "ADR-025 retires the whole `024` identity machinery; its names must be gone \
         from every shipped source file, doc comments included (module-doc reversals):\n{}",
        violations.join("\n")
    );
}

/// proves: design 025's Test strategy #14 + Rollout 1 — `derive_slug`'s module and
/// the two suites that pinned the retired `024` machinery are gone from every
/// compiled surface: `crates/discovery/src/lib.rs` no longer declares the `slug`
/// module (so a still-checked-in `src/slug.rs` compiles into nothing), and
/// `tests/slug_tests.rs` and `tests/collision_tests.rs` take no part in any build —
/// each is either deleted from the tree outright or, in the shape this run's
/// toolchain records it (stage agents cannot delete files, so the file removals are
/// the named operator follow-up in `crates/discovery/Cargo.toml`), declared as a
/// `[[test]]` target behind a `required-features` gate that the package's default
/// features never enable and no gate command (`cargo test/clippy/fmt --workspace`,
/// never run with `--features`) ever selects. Either way the suite builds and runs
/// nowhere: that is what `025`'s "their suites are gone" means of a build tree, and
/// this gate holds for both the run's recorded shape and the operator's post-cleanup
/// tree (a deleted file must not leave a dangling target declaration behind).
#[test]
fn the_retired_module_and_suites_are_gone() {
    let discovery = repo_root().join("crates").join("discovery");

    // `derive_slug` retires with its `pub mod slug;` line: undeclared, a leftover
    // `src/slug.rs` is dead text no crate target compiles (the name sweep above
    // already pins that no shipped source spells `slug::` or `derive_slug`).
    let lib_text = std::fs::read_to_string(discovery.join("src").join("lib.rs"))
        .expect("the crate root text is readable");
    assert!(
        !lib_text.contains("mod slug"),
        "`crates/discovery/src/lib.rs` must no longer declare the retired `slug` \
         module: ADR-025 Rollout 1 deletes the module, and the declaration is the \
         only route by which its file reaches a compiled crate"
    );

    // The two retired suites: absent outright, or gated out of every build.
    let manifest = std::fs::read_to_string(discovery.join("Cargo.toml"))
        .expect("the package manifest text is readable");
    for suite in ["slug_tests", "collision_tests"] {
        let path = discovery.join("tests").join(format!("{suite}.rs"));
        let declared = manifest.contains(&format!("name = \"{suite}\""));
        if !path.exists() {
            assert!(
                !declared,
                "`tests/{suite}.rs` is deleted, so `{suite}` must also be gone from \
                 the package manifest's `[[test]]` declarations — a dangling target \
                 declaration is no way to leave a suite retired"
            );
            continue;
        }
        let gated = manifest
            .split("[[test]]")
            .skip(1)
            .filter(|chunk| chunk.contains(&format!("name = \"{suite}\"")))
            .any(|chunk| chunk.contains("required-features"));
        assert!(
            gated,
            "`tests/{suite}.rs` pins machinery ADR-025 retires (`024-C2` derivation \
             / the collision pass), so while it stands in the tree it must be \
             declared as a `[[test]]` target behind `required-features` — the run's \
             recorded retirement shape"
        );
    }

    // The gate feature itself must be never-default: the workspace gates run
    // without `--features`, so an off-by-default feature excludes the suites from
    // every command the CI gate (`check.yml`) runs.
    let features_block = features_block_of(&manifest);
    assert!(
        features_block.contains("retired-adr-024-suites"),
        "the manifest declares the `retired-adr-024-suites` gate feature that keeps \
         the retired suites out of every build"
    );
    assert!(
        !features_block.contains("default"),
        "the gate feature must stay off the package's default features — a default \
         build must never select the retired suites: {features_block}"
    );
}

/// The package manifest's `[features]` table body, up to the next table header.
fn features_block_of(manifest: &str) -> &str {
    let start = manifest
        .find("[features]")
        .expect("the manifest declares its `[features]` table");
    manifest[start + "[features]".len()..]
        .find('[')
        .map(|next| &manifest[start + "[features]".len()..start + "[features]".len() + next])
        .unwrap_or(&manifest[start + "[features]".len()..])
}

/// proves: design 025's Test strategy #14 + #13 (CLI half of the retirement) — the
/// `Namespace` spelling is gone from the CLI's source, its rendering, and its suite
/// texts: the `Namespace` table column retires and the shipped CLI suites assert
/// `Game Id` and the `steam/<appid>` slugs instead. The JSON row's retired
/// `"namespace"` key is likewise gone from the source (the suites' own absence
/// assertion in `cli_json_mode_tests.rs` is the pin that it no longer serialises).
#[test]
fn the_namespace_spelling_is_gone_from_the_cli_surface() {
    let mut violations: Vec<String> = Vec::new();
    for path in rust_files(&repo_root().join("apps").join("discovery-cli").join("src")) {
        let text = std::fs::read_to_string(&path).expect("text is readable");
        if text.contains("Namespace") || text.contains("\"namespace\"") {
            violations.push(path.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "`Namespace` and the `\"namespace\"` JSON key retire everywhere in the CLI's \
         source; these files still name them:\n{}",
        violations.join("\n")
    );

    let mut violations: Vec<String> = Vec::new();
    for path in rust_files(&repo_root().join("apps").join("discovery-cli").join("tests")) {
        let text = std::fs::read_to_string(&path).expect("text is readable");
        if text.contains("Namespace") {
            violations.push(path.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "the shipped CLI suites carry the `Namespace` column nowhere; these still \
         name it:\n{}",
        violations.join("\n")
    );
}

/// proves: design 025's Test strategy #14 + Chosen design (Emission — "`steam_linux.rs`'s
/// module doc loses the \"AppIDs are never read\" clause") — the ADR-022 holdover
/// sentence (A2's only statement) is gone from the Linux plugin's module docs.
#[test]
fn the_steam_linux_module_doc_no_longer_claims_appids_are_never_read() {
    let path = repo_root()
        .join("crates")
        .join("discovery")
        .join("src")
        .join("steam_linux.rs");
    let text = std::fs::read_to_string(&path).expect("the module text is readable");
    assert!(
        !text.contains("AppIDs are never read"),
        "`crates/discovery/src/steam_linux.rs` must lose the \"AppIDs are never read\" \
         clause: this run reads the appid — it is the row's identity (`025-C1`)"
    );
}

/// proves: design 025's Test strategy #14 (Chosen design: Bound — "the API probe is
/// re-pinned to the new surface") — the discovery API probe names the new surface, so
/// a re-pin that quietly keeps asserting the retired one cannot pass: `lookup_key`
/// and the new bound must appear in `zz_api_probe.rs`.
#[test]
fn the_discovery_api_probe_is_re_pinned_to_the_new_surface() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("zz_api_probe.rs");
    let text = std::fs::read_to_string(&path).expect("the probe text is readable");
    for name in ["lookup_key", "Source", "MAX_DISCOVERY_SLUG_OCTETS"] {
        assert!(
            text.contains(name),
            "the re-pinned probe must resolve `{name}` — the new surface ADR-025 ships"
        );
    }
}
