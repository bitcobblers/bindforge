//! Loading a `policy/v1` document (ADR-015, design 015).
//!
//! A policy is untrusted input exactly like a profile, patch, or manifest: it
//! arrives from disk, from another user, or from the hub, and its `id` is the
//! name that becomes a file name — the `<id>.toml` this loader joins, and the
//! name a produced base carries in its `generated-by` label. So the name is
//! matched against the slug grammar *before* any filesystem question is asked,
//! the same discipline `load_geometry` pins: `..`, a separator, an absolute
//! path, or an uppercase spelling refuses naming nothing but the grammar, and
//! no file is opened at all (A15).
//!
//! The rest is the shipped loader posture: the file is `<id>.toml` sitting
//! directly in the version directory, its `id` entry must agree with its stem,
//! and its declared major must equal the directory segment
//! (`check_dir_version`). The grammar itself — the closed entry set, the row
//! field counts, the tag shape — is `document`'s, and every rule that needs
//! loaded data in hand (which ids a tag matches, whether a move respects a
//! declared bound, which manifest inputs are reserved) belongs to the stack in
//! `crates/transforms`, not here.
//!
//! No new `CoreError` variant joins the family: every refusal below is
//! `InvalidValue`, `MissingEntry`, `Io`, or the version gate, in the one-line,
//! escaped, truncating, path-free message `schemas/README.md` promises (A9,
//! A14). A document that is not a policy document refuses on the format arm
//! rather than being reinterpreted.

use std::path::Path;

use crate::document::{Document, Policy, check_dir_version, parse_file};
use crate::error::CoreError;
use crate::placement::MAX_SLUG_OCTETS;
use crate::placement::is_slug_name;

/// The slug grammar, spelled the way the other loaders spell it in a refusal.
const SLUG_RULE: &str = "expected `[a-z][a-z0-9-]*`";

/// Load one policy from the TOML file `<id>.toml` sitting directly in one
/// `schemas/policy/v<N>/` version directory.
///
/// `id` is checked against the `[a-z][a-z0-9-]*` grammar and the octet bound
/// *before* it is joined onto `dir`, so a name that arrived from somewhere
/// else — a `..`, a separator, an absolute path, an uppercase spelling — is
/// refused without asking the filesystem anything and without naming a path:
/// `Path::join` would let an absolute name replace the directory outright,
/// which is precisely why the name never reaches it. The loaded document's `id`
/// entry must agree with the file stem, as a geometry's `geometry` entry and a
/// reach table's `hand` entry must agree with theirs, and the declared major
/// must equal the directory's segment.
pub fn load_policy(dir: &Path, id: &str) -> Result<Policy, CoreError> {
    if !is_slug_name(id) {
        let message = format!("a policy id is a slug: {SLUG_RULE}");
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message,
        });
    }
    if id.len() > MAX_SLUG_OCTETS {
        // The bound leads and the offender follows, as in every other
        // slug-bound refusal: the echoed id is by definition past the bound,
        // so an echo-first message would truncate the number away.
        let message = format!("a policy id is at most {MAX_SLUG_OCTETS} octets: `{id}`");
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message,
        });
    }
    let file_name = format!("{id}.toml");
    let document = parse_file(&dir.join(&file_name))?;
    let Document::Policy(policy) = document else {
        let message = format!("`{file_name}` must hold a policy document");
        return Err(CoreError::InvalidValue {
            key: "format".to_string(),
            message,
        });
    };
    // A policy's identity is its contents, and the file name is how a chain or
    // data directory looks it up: a stem that contradicts the `id` entry would
    // answer two different questions depending on which half a consumer
    // believed. The refusal names the stem and the entry, never a path.
    if policy.id != id {
        let declared = policy.id.as_str();
        let message = format!("`{file_name}` declares id `{declared}`");
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message,
        });
    }
    check_dir_version(dir, &policy.version, &file_name)?;
    Ok(policy)
}
