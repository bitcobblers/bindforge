//! The closed static registry and its two claim passes (`017-C5`, §The
//! registry).
//!
//! `claim` runs two passes over [`REGISTRY`] and the first claim wins: the
//! appid pass (families whose `appids()` contain the context's appid, in id
//! order; a claim carries `Evidence::AppidTable`) runs before any tree
//! probing, and the probe pass runs only if pass 1 claimed nothing (every
//! family in id order; a claim carries `Evidence::InstallTreeProbe`). A tree
//! probe claims only when it located at least one file in its own grammar;
//! an appid-table hit claims regardless of what the grammar's files turn out
//! to contain — that is `017-L2`'s family-confirmed-but-files-not-found case.

use crate::codec::{CodecFamily, Detection, Evidence};
use crate::context::GameContext;
use crate::families::unreal_ini;

/// The closed implementation set, declared in family-id byte order — a unit
/// test asserts the declaration *is* byte-sorted so the ordering law cannot
/// rot.
pub static REGISTRY: &[&dyn CodecFamily] = &[&unreal_ini::UnrealIni];

/// The shipped registry's first claim on `ctx` (§The registry).
#[must_use]
pub fn claim(ctx: &GameContext) -> Option<Detection> {
    claim_with(REGISTRY, ctx)
}

/// The two claim passes over a caller-provided family slice — the seam that
/// lets a synthetic family pin the pass ordering without inventing appids
/// (`claim` is the shipped `REGISTRY` instance of it). Both passes iterate in
/// family-id byte order whatever the slice's arrangement.
#[must_use]
pub fn claim_with(families: &[&dyn CodecFamily], ctx: &GameContext) -> Option<Detection> {
    let mut ordered: Vec<&&dyn CodecFamily> = families.iter().collect();
    ordered.sort_by_key(|family| family.id());

    // Pass 1 — the appid table. The table selects whose `detect` runs; it
    // does not rubber-stamp: a table family whose own detect declines does
    // not claim. Every family whose `detect` runs here is remembered so
    // pass 2 never re-walks a tree a declined table family already walked
    // once (`detect` is a pure function of `ctx` — the second answer is the
    // first).
    let mut probed: Vec<&'static str> = Vec::new();
    for family in &ordered {
        if !family.appids().iter().any(|appid| *appid == ctx.appid) {
            continue;
        }
        probed.push(family.id());
        if let Some(claim) = family.detect(ctx) {
            return Some(Detection {
                evidence: Evidence::AppidTable,
                ..claim
            });
        }
    }

    // Pass 2 — the tree probe, only when pass 1 claimed nothing. Families
    // pass 1 already ran are skipped: their decline is this pass's answer too.
    for family in &ordered {
        if probed.contains(&family.id()) {
            continue;
        }
        if let Some(claim) = family.detect(ctx) {
            return Some(Detection {
                evidence: Evidence::InstallTreeProbe,
                ..claim
            });
        }
    }
    None
}

/// The published family ids, in the registry's byte order — the closed set
/// `013-R3`'s narrowed half hands to consumers without a dependency edge in
/// core.
#[must_use]
pub fn family_ids() -> Vec<&'static str> {
    REGISTRY.iter().map(|family| family.id()).collect()
}

/// Whether `id` names a family in the closed set (case-sensitively — family
/// ids are their slug grammar).
#[must_use]
pub fn is_known_family(id: &str) -> bool {
    REGISTRY.iter().any(|family| family.id() == id)
}

/// The family `id` names, for a face that holds a `Detection` and must run
/// its claiming family's `extract`.
#[must_use]
pub fn family(id: &str) -> Option<&'static dyn CodecFamily> {
    REGISTRY.iter().find(|family| family.id() == id).copied()
}
