//! The row a discovery plugin emits, and the identity half ADR-025 gives it.
//!
//! A discovered game's `slug` is the catalog lookup key `<source>/<external-id>`,
//! composed by exactly one constructor ([`Source::lookup_key`]) — the only place
//! that spelling exists in the product (`025-C1`). It rides
//! [`bindforge_core::MAX_DISCOVERY_SLUG_OCTETS`] and is exempt from the shared
//! slug grammar by design: the key joins nothing, becomes no product id, and is
//! never a path component (`025-L3`, `025-L5`). The display name's normalized
//! form rides beside it as [`friendly_name`], which joins nothing either.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    SteamLinux,
    SteamWindows,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::SteamLinux => write!(f, "steam-linux"),
            Platform::SteamWindows => write!(f, "steam-windows"),
        }
    }
}

/// The closed set of discovery sources whose external ids a lookup key may carry
/// (`025-C1`). `as_str` spells the token; the tokens are the whole set, so a key
/// outside it was not built by this crate.
///
/// `Source` is construction-side only: it carries no serde, because the row
/// carries the composed key and never the token. Of the three, only [`Source::Steam`]
/// has a shipped emitter this run — `epic` waits on `025-R2` and `local` on a
/// backend that does not exist yet, so their tokens are declared and unpopulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Steam,
    Epic,
    Local,
}

impl Source {
    /// Every token of the closed set, in the order the set is stated.
    pub const ALL: [Source; 3] = [Source::Steam, Source::Epic, Source::Local];

    /// The token this source spells in a lookup key: `steam`, `epic`, `local`.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::Steam => "steam",
            Source::Epic => "epic",
            Source::Local => "local",
        }
    }

    /// The one place the `<source>/<external-id>` spelling exists: a source token
    /// and an external id joined by exactly one `/`. Every shipped row is built
    /// through here, which is why the emit boundary may check
    /// [`bindforge_core::MAX_DISCOVERY_SLUG_OCTETS`] and nothing else about the
    /// shape (`025-L3`): a plugin cannot spell a key differently without calling
    /// another function.
    #[must_use]
    pub fn lookup_key(&self, external_id: &str) -> String {
        format!("{}/{external_id}", self.as_str())
    }

    /// The source whose token `token` spells, or `None` for anything outside the
    /// closed set. This is how an untrusted key — a hand-edited match-store
    /// record — is checked against the same set the constructors use.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Source> {
        Source::ALL
            .into_iter()
            .find(|source| source.as_str() == token)
    }

    /// Whether `key` is spelled `<token>/<external-id>` with `<token>` in the
    /// closed set and a non-empty external id. Shape only: the length bound is
    /// the caller's check, as it is at the emit boundary.
    #[must_use]
    pub fn spells_lookup_key(key: &str) -> bool {
        let Some((token, external_id)) = key.split_once('/') else {
            return false;
        };
        !external_id.is_empty() && Source::from_token(token).is_some()
    }
}

/// The row's display metadata: `display_name` trimmed, interior whitespace runs
/// collapsed to one space, then clamped to the longest `char`-boundary prefix
/// inside [`bindforge_core::MAX_NATIVE_NAME_OCTETS`].
///
/// This is the only normalization left in this crate, and it is a whitespace
/// rule, not a derivation law: nothing parses the result, it is never joined to
/// anything, and it is never user-visible (the UI prints `display_name`). That
/// is what makes the clamp admissible here and only here — a truncated display
/// string costs a shorter label, never a different identity (`025-C2`).
#[must_use]
pub fn friendly_name(display_name: &str) -> String {
    let normalized = display_name
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    let budget = bindforge_core::MAX_NATIVE_NAME_OCTETS;
    let mut end = normalized.len().min(budget);
    while !normalized.is_char_boundary(end) {
        end -= 1;
    }
    normalized[..end].to_string()
}

/// One discovered row beside the Steam context that owns it (design 017,
/// §Discovery plumbing): the row itself, the Steam *root* that owns the
/// library it was found in (the root itself, or the root whose
/// `libraryfolders.vdf` named it), and the library that was scanned.
///
/// This is the additive seam behind `SteamCore::scan()`; `discover()` is the
/// plain projection of it (`r.game`), so the trait, the row's serde shape and
/// every pinned `list` rendering are untouched by its existence (`017-L5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRecord {
    /// The emitted row, unchanged in shape and spelling.
    pub game: DiscoveredGame,
    /// The Steam root that owns `library` — the root itself, or the root whose
    /// `libraryfolders.vdf` named it.
    pub root: PathBuf,
    /// The library directory this row's manifests were scanned from.
    pub library: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredGame {
    /// The `<source>/<external-id>` catalog lookup key (`025-C1`).
    pub slug: String,
    /// Normalized display metadata ([`friendly_name`]); never a user-visible
    /// string and never a join key (`025-C2`).
    pub friendly_name: String,
    pub display_name: String,
    pub install_path: PathBuf,
    pub platform: Platform,
    pub codec_id: Option<String>,
}
