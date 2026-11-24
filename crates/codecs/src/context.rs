//! `GameContext` (`017-C2`, §GameContext) — everything a codec is told about
//! a discovered game, and the only gate through which one arrives.
//!
//! `roots` is `017-C2`'s whole resolved set, read differently than its "main
//! root and library roots separately" wording: A18 puts `userdata/` under a
//! root no scan output labels, so every root is searched and a tree is used
//! only if it exists, in the core's canonicalized byte order.

use std::path::PathBuf;

use discovery::game::{ScanRecord, Source};

/// The lookup key's source token this gate admits: `steam` is the only
/// source with a shipped emitter and the only launcher whose trees the
/// resolvers build (`017-C2`).
const STEAM_TOKEN: &str = "steam";

/// The appid bound mirrored from the scan seam (`steam_common.rs`'s
/// `usable_appid`): 1–20 ASCII digits.
const MAX_APPID_DIGITS: usize = 20;

/// Everything a codec family is handed about one discovered game: the lookup
/// key, the digit appid, the install path, the platform, every root the scan
/// resolved, and the *owning* library (§Discovery plumbing is where the
/// owner comes from).
#[derive(Debug, Clone)]
pub struct GameContext {
    /// `<source>/<external-id>`, byte-for-byte the row's slug.
    pub lookup_key: String,
    /// The external id: 1..=20 ASCII digits.
    pub appid: String,
    /// The install path as the row spells it. Never a candidate tree (§The
    /// first family: defaults are not absorbed).
    pub install_path: PathBuf,
    /// The row's platform tag.
    pub platform: discovery::game::Platform,
    /// Every root the scan resolved, in the core's canonicalized byte order.
    pub roots: Vec<PathBuf>,
    /// The owning library: the anchor `steamapps/compatdata/` hangs off.
    pub library: PathBuf,
}

impl GameContext {
    /// The whole gate: `Some` only for a `steam` lookup key with a 1–20-digit
    /// external id — mirroring the appid rule the scan seam already enforces.
    /// A row that is not a Steam install yields no context, and the face
    /// answers `claim-refused`: no codec ever sees a half-parsed identity.
    #[must_use]
    pub fn from_record(record: &ScanRecord, roots: &[PathBuf]) -> Option<GameContext> {
        let (token, external_id) = record.game.slug.split_once('/')?;
        if token != STEAM_TOKEN || !Source::spells_lookup_key(&record.game.slug) {
            return None;
        }
        if external_id.is_empty()
            || external_id.len() > MAX_APPID_DIGITS
            || !external_id.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        Some(GameContext {
            lookup_key: record.game.slug.clone(),
            appid: external_id.to_string(),
            install_path: record.game.install_path.clone(),
            platform: record.game.platform,
            roots: roots.to_vec(),
            library: record.library.clone(),
        })
    }
}
