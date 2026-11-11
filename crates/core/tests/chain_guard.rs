//! The chain guard: the one piece of chain semantics this design decides
//! (Chosen design → Adversarial guard). Profile, patch, policy and manifest
//! documents arriving from disk, from another user, or from the network are
//! untrusted input at every entry point, and these guards run before any of
//! them can reach a writer.
//!
//! proves: Test strategy #3 — `resolve_chain` rejects a self-deriving chain
//! (`Cycle`), accepts a chain of exactly `MAX_CHAIN_DEPTH` links, and rejects
//! one link deeper (`ChainTooDeep`), boundary-tested with fixtures under
//! `schemas/profile/v1/fixtures/invalid/`.
//!
//! The boundary is written against `bindforge_core::MAX_CHAIN_DEPTH`, never
//! the literal 32: the proposal fixes 32 as an initial value ADR-009 may tune
//! (it may not remove the bound).

mod common;

use bindforge_core::CoreError;
use common::{ancestor_chain, fixture_text, fixtures, profile_doc, version_dir};

/// The guard's verdict for one document: the typed error from chain
/// resolution, or the parser's if it refuses the document first — the
/// proposal pins the variant names and the bound, not which layer applies
/// them. `None` means the document passed the guard.
fn guard_error(text: &str) -> Option<CoreError> {
    let document = match bindforge_core::parse(text) {
        Ok(document) => document,
        Err(err) => return Some(err),
    };
    match document.profile() {
        Some(profile) => bindforge_core::resolve_chain(profile).err(),
        None => None,
    }
}

/// proves: Test strategy #3 — a chain deriving from itself is rejected with
/// the typed `Cycle` error instead of recursing.
#[test]
fn rejects_a_chain_that_derives_from_itself() {
    let text = profile_doc("self-deriving", &["self-deriving".to_string()]);

    let err = guard_error(&text).expect("a profile that derives from itself must be rejected");
    assert!(
        matches!(err, CoreError::Cycle),
        "expected the `Cycle` variant, got {err:?}"
    );
}

/// proves: Test strategy #3 — a chain of exactly `MAX_CHAIN_DEPTH` links is
/// accepted: the bound is a limit, not an off-by-one wall.
#[test]
fn accepts_a_chain_of_exactly_max_chain_depth_links() {
    let text = profile_doc(
        "at-the-boundary",
        &ancestor_chain(bindforge_core::MAX_CHAIN_DEPTH),
    );

    assert!(
        guard_error(&text).is_none(),
        "a chain of exactly MAX_CHAIN_DEPTH links must resolve"
    );
}

/// proves: Test strategy #3 — one link past `MAX_CHAIN_DEPTH` is rejected
/// with the typed `ChainTooDeep` error instead of recursing further.
#[test]
fn rejects_a_chain_one_link_deeper_than_max_chain_depth() {
    let links = bindforge_core::MAX_CHAIN_DEPTH + 1;
    let text = profile_doc("past-the-boundary", &ancestor_chain(links));

    let err = guard_error(&text)
        .expect("a chain of MAX_CHAIN_DEPTH + 1 links must be rejected, not resolved");
    assert!(
        matches!(err, CoreError::ChainTooDeep),
        "expected the `ChainTooDeep` variant, got {err:?}"
    );
}

/// proves: Test strategy #3 — the boundary also ships as fixtures under
/// `schemas/profile/v1/fixtures/invalid/`, which is what makes it part of the
/// language-neutral contract instead of a Rust-only behavior: the corpus must
/// hold a chain that derives from itself and a chain past the bound, rejected
/// as `Cycle` and `ChainTooDeep` respectively.
#[test]
fn the_invalid_corpus_ships_both_chain_boundary_cases() {
    let profile_v1 = version_dir("profile", "v1");
    let rejections: Vec<(String, Option<CoreError>)> = fixtures(&profile_v1, "invalid")
        .into_iter()
        .map(|path| {
            let label = path.display().to_string();
            (label, guard_error(&fixture_text(&path)))
        })
        .collect();

    let rejected_cycle = rejections
        .iter()
        .any(|(_, err)| matches!(err, Some(CoreError::Cycle)));
    assert!(
        rejected_cycle,
        "{}'s fixtures/invalid/ must contain a self-deriving chain rejected as `Cycle`; saw {:?}",
        profile_v1.path.display(),
        rejections
            .iter()
            .map(|(label, err)| format!("{label} -> {err:?}"))
            .collect::<Vec<_>>()
    );

    let rejected_depth = rejections
        .iter()
        .any(|(_, err)| matches!(err, Some(CoreError::ChainTooDeep)));
    assert!(
        rejected_depth,
        "{}'s fixtures/invalid/ must contain a chain deeper than MAX_CHAIN_DEPTH rejected as \
         `ChainTooDeep`",
        profile_v1.path.display()
    );
}
