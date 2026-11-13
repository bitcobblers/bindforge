//! Chain resolution — the one piece of chain semantics this design decides
//! (Chosen design -> Adversarial guard). All other chain semantics are
//! ADR-014's.
//!
//! `resolve_chain` is the guard that lets the hub validate uploads with the
//! same compiled types the CLI and desktop use: it walks the chain
//! iteratively and refuses adversarial shapes with typed errors instead of
//! recursing into them.
//!
//! Cross-file chain loading — reading a chain's documents and applying these
//! same two refusals to the *resolved* chain — is `chain_ops::load_chain`
//! (design 014). This module keeps its single-document meaning and its shipped
//! code untouched; its fixtures stay the contract for what it pins (A15).

use std::collections::HashSet;

use crate::document::Profile;
use crate::error::CoreError;

/// Maximum number of ancestor links a profile's chain may hold. 32 is the
/// initial value ADR-014 may tune but may not remove; the boundary tests and
/// the fixture corpus are written against this constant, never the literal.
pub const MAX_CHAIN_DEPTH: usize = 32;

/// Resolve one profile's ancestor chain, ordered from the profile itself to
/// its deepest ancestor.
///
/// Rejects a chain that derives from itself with [`CoreError::Cycle`] and a
/// chain longer than [`MAX_CHAIN_DEPTH`] links with [`CoreError::ChainTooDeep`],
/// before and instead of any deeper traversal.
pub fn resolve_chain(profile: &Profile) -> Result<Vec<String>, CoreError> {
    if profile.derives.len() > MAX_CHAIN_DEPTH {
        return Err(CoreError::ChainTooDeep);
    }

    let mut chain = Vec::with_capacity(profile.derives.len() + 1);
    let mut seen: HashSet<&str> = HashSet::new();
    chain.push(profile.id.clone());
    seen.insert(profile.id.as_str());
    for ancestor in &profile.derives {
        if !seen.insert(ancestor.as_str()) {
            return Err(CoreError::Cycle);
        }
        chain.push(ancestor.clone());
    }
    Ok(chain)
}
