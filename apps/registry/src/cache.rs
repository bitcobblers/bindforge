//! The `026-C7` cache document and the `026-B1`/`026-B2` write-time caps.
//!
//! A cache is a `BTreeMap` keyed by `slug`, so the persisted entry array is in
//! `slug` byte order as `026-C7` states it, whatever order the lookups arrived in.
//! The load treats the file as untrusted input and refuses by name: a wrong
//! `version`, a missing or non-array `entries`, an entry that is not an object or
//! lacks `candidates`, a `game_id` outside the grammar or a `name` over `013-B4`
//! (A4, A5), a repeated `slug`, a `slug` failing the key rule, a cached entry
//! carrying more than `MAX_CANDIDATES` candidates, and a `fetched_at` outside the
//! load window — the clock guard §Budgets puts on a stamp a jump could have
//! written. Both caps are **write-time** invariants (§Budgets, open question 5):
//! after an insert the document is serialized, and the oldest `fetched_at` entries
//! are dropped and re-serialized until the exact buffer is inside `026-B2` octets
//! and the entry count inside `026-B1`. Nothing is rewritten by a load refusal (A7).

use crate::constants::{
    DOCUMENT_VERSION, FETCHED_AT_FUTURE_TOLERANCE_SECONDS, FETCHED_AT_PAST_TOLERANCE_SECONDS,
    MAX_CACHE_DOCUMENT_OCTETS, MAX_CACHE_ENTRIES, MAX_CANDIDATES,
};
use crate::corpus::{Candidate, LoadError, game_id_is_valid};
use crate::keys::key_is_admissible;
use crate::persist::{self, PersistError};
use bindforge_core::{MAX_NATIVE_NAME_OCTETS, escape_and_truncate};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One cached lookup: the projected answer and the wall-clock stamp the TTL and the
/// eviction key both read.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub candidates: Vec<Candidate>,
    pub fetched_at: u64,
}

impl CacheEntry {
    /// Whether this entry is still inside `026-B3`'s TTL at `now`. A stamp ahead of
    /// the clock is a backwards clock jump as far as TTL arithmetic can see, and the
    /// guard reads it as **expired**, never immortal (§Budgets).
    #[must_use]
    pub fn is_fresh(&self, now: u64, ttl_seconds: u64) -> bool {
        if self.fetched_at > now {
            return false;
        }
        now - self.fetched_at < ttl_seconds
    }
}

/// The cache: entries in `slug` byte order, plus the file they persist to.
#[derive(Debug, Clone)]
pub struct Cache {
    path: PathBuf,
    entries: BTreeMap<String, CacheEntry>,
}

impl Cache {
    /// An empty cache over `path`, which is also what an absent file means
    /// (`026-C7`: "an absent cache file is an empty cache, not an error").
    #[must_use]
    pub fn empty(path: PathBuf) -> Self {
        Self {
            path,
            entries: BTreeMap::new(),
        }
    }

    /// Whether the instance has ever had an answer to cache: the reachability probe
    /// is fired by a lookup that is about to fetch *and* already has a cached entry
    /// (§Caching mode's reachability rule).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `cached_slugs`, the summary count `026-C2` gives health.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The cached answer for one slug, fresh or stale.
    #[must_use]
    pub fn entry(&self, slug: &str) -> Option<&CacheEntry> {
        self.entries.get(slug)
    }

    /// Loads the document at `path`, refusing an out-of-shape one by name.
    pub fn load(path: &Path, now: u64) -> Result<Self, LoadError> {
        let bytes = persist::read_capped(path, MAX_CACHE_DOCUMENT_OCTETS)
            .map_err(|error| LoadError::at(path, error.reason))?;
        let Some(bytes) = bytes else {
            return Ok(Self::empty(path.to_path_buf()));
        };
        let refused = |detail: String| LoadError::at(path, detail);
        let document: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
            refused(format!(
                "the cache document is not readable JSON, and `version` must be the bare \
                 integer {DOCUMENT_VERSION}: {error}"
            ))
        })?;
        let fields = document
            .as_object()
            .ok_or_else(|| refused("the cache document is not a JSON object".to_string()))?;
        let version = fields
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                refused(format!(
                    "`version` is missing or is not the bare integer {DOCUMENT_VERSION}"
                ))
            })?;
        if version != DOCUMENT_VERSION {
            return Err(refused(format!(
                "`version` {version} is not the bare integer {DOCUMENT_VERSION} `026-C7` spells"
            )));
        }
        let entries = fields
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| refused("`entries` is missing or is not an array".to_string()))?;
        let mut loaded: BTreeMap<String, CacheEntry> = BTreeMap::new();
        for entry in entries {
            let fields = entry
                .as_object()
                .ok_or_else(|| refused("a `entries` entry is not a JSON object".to_string()))?;
            let slug = fields
                .get("slug")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    refused("an `entries` entry carries no `slug` string".to_string())
                })?;
            if !key_is_admissible(slug) {
                return Err(refused(format!(
                    "an `entries` `slug` `{}` is not an admitted lookup key",
                    escape_and_truncate(slug)
                )));
            }
            let candidates = fields
                .get("candidates")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| refused("`candidates` is missing or is not an array".to_string()))?;
            if candidates.len() > MAX_CANDIDATES {
                // The count cap A1 pins is a property of the document as well as of
                // the answer: an entry that stores more than one answer may carry is
                // refused like every other load violation, rather than being served
                // whole on every lookup until the octet budget is reached.
                return Err(refused(format!(
                    "the cached entry for `{}` carries {} candidates, over the {} one answer \
                     may carry",
                    escape_and_truncate(slug),
                    candidates.len(),
                    MAX_CANDIDATES
                )));
            }
            let mut kept: Vec<Candidate> = Vec::with_capacity(candidates.len());
            for candidate in candidates {
                let game_id = candidate
                    .get("game_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        refused("a cached candidate carries no `game_id` string".to_string())
                    })?;
                if !game_id_is_valid(game_id) {
                    return Err(refused(format!(
                        "a cached `game_id` `{}` fails the slug grammar or its bound (A4)",
                        bindforge_core::escape_and_truncate(game_id)
                    )));
                }
                let name = candidate
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        refused("a cached candidate carries no `name` string".to_string())
                    })?;
                if name.len() > MAX_NATIVE_NAME_OCTETS {
                    return Err(refused(format!(
                        "a cached `name` is {} octets, over the {MAX_NATIVE_NAME_OCTETS}-octet \
                         bound (A5)",
                        name.len()
                    )));
                }
                kept.push(Candidate {
                    game_id: game_id.to_string(),
                    name: name.to_string(),
                });
            }
            let fetched_at = fields
                .get("fetched_at")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| {
                    refused("`fetched_at` is missing or is not a non-negative integer".to_string())
                })?;
            let oldest_allowed = now.saturating_sub(FETCHED_AT_PAST_TOLERANCE_SECONDS);
            let newest_allowed = now.saturating_add(FETCHED_AT_FUTURE_TOLERANCE_SECONDS);
            if fetched_at < oldest_allowed || fetched_at > newest_allowed {
                return Err(refused(format!(
                    "`fetched_at` {fetched_at} is outside the load window \
                     [{oldest_allowed}, {newest_allowed}] — the stamp is untrusted input"
                )));
            }
            if loaded
                .insert(
                    slug.to_string(),
                    CacheEntry {
                        candidates: kept,
                        fetched_at,
                    },
                )
                .is_some()
            {
                return Err(refused(format!(
                    "the `slug` `{}` is repeated: the cache would load ambiguously",
                    escape_and_truncate(slug)
                )));
            }
        }
        Ok(Self {
            path: path.to_path_buf(),
            entries: loaded,
        })
    }

    /// The document's bytes, for the unit tests that read the shape `stage`
    /// produces without staging an entry (`026-C7`'s shape, asserted below).
    #[cfg(test)]
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        serialize(&self.entries)
    }

    /// Stages one lookup's answer, applies both write-time caps, and returns the
    /// buffer to rename into place together with the staged entry set
    /// (§Budgets: the octet test is the document's exact length).
    ///
    /// The staged set is only adopted by [`Cache::commit`] after the write succeeds,
    /// so a persist failure leaves the model equal to the file the next read sees.
    #[must_use]
    pub fn stage(
        &self,
        slug: &str,
        candidates: Vec<Candidate>,
        now: u64,
    ) -> (BTreeMap<String, CacheEntry>, Vec<u8>) {
        let mut staged = self.entries.clone();
        // The per-entry candidate list is bounded here too: A1's count cap is a
        // property of the document, so nothing that reaches this seam may store a
        // longer list than a later answer would have to emit whole. The upstream
        // client already refuses a longer answer on the way in (`upstream.rs`); this
        // is the invariant, not the check.
        let mut candidates = candidates;
        candidates.truncate(MAX_CANDIDATES);
        staged.insert(
            slug.to_string(),
            CacheEntry {
                candidates,
                fetched_at: now,
            },
        );
        while staged.len() > MAX_CACHE_ENTRIES {
            drop_oldest(&mut staged);
        }
        let mut bytes = serialize(&staged);
        while bytes.len() > MAX_CACHE_DOCUMENT_OCTETS && !staged.is_empty() {
            drop_oldest(&mut staged);
            bytes = serialize(&staged);
        }
        (staged, bytes)
    }

    /// Adopts a staged entry set once its bytes are on disk.
    pub fn commit(&mut self, staged: BTreeMap<String, CacheEntry>) {
        self.entries = staged;
    }

    /// Writes the cache through the tmp-and-rename discipline.
    pub fn save(&self, bytes: &[u8]) -> Result<(), PersistError> {
        persist::save(&self.path, bytes)
    }
}

/// Serializes an entry set; pretty, so the document a reader opens is legible and
/// its length is the figure `026-B2` gates.
fn serialize(entries: &BTreeMap<String, CacheEntry>) -> Vec<u8> {
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|(slug, entry)| {
            let candidates: Vec<serde_json::Value> =
                entry.candidates.iter().map(Candidate::json).collect();
            serde_json::json!({
                "slug": slug,
                "candidates": candidates,
                "fetched_at": entry.fetched_at,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "version": DOCUMENT_VERSION,
        "entries": items,
    }))
    .expect("a cache document of plain entries serializes")
    .into_bytes()
}

/// Drops the oldest `fetched_at` entry — `026-B1`'s own eviction key, with the
/// `slug` as the tie-break so the order is total.
fn drop_oldest(entries: &mut BTreeMap<String, CacheEntry>) {
    let oldest = entries
        .iter()
        .min_by(|(left_slug, left), (right_slug, right)| {
            left.fetched_at
                .cmp(&right.fetched_at)
                .then_with(|| left_slug.cmp(right_slug))
        })
        .map(|(slug, _)| slug.clone());
    if let Some(slug) = oldest {
        entries.remove(&slug);
    }
}

#[cfg(test)]
mod tests {
    //! The load refusals, the write-time caps, the clock guard, and the byte order —
    //! in-process, over the same shapes the suites seed.
    use super::*;

    fn entry(slug: &str, fetched_at: u64) -> (String, CacheEntry) {
        (
            slug.to_string(),
            CacheEntry {
                candidates: vec![Candidate {
                    game_id: "cached-game".to_string(),
                    name: "Cached Game".to_string(),
                }],
                fetched_at,
            },
        )
    }

    fn seeded(entries: Vec<(String, CacheEntry)>) -> Cache {
        Cache {
            path: PathBuf::from("cache-v1.json"),
            entries: entries.into_iter().collect(),
        }
    }

    #[test]
    fn entries_persist_sorted_by_slug_and_carry_their_stamp() {
        let cache = seeded(vec![entry("steam/900003", 10), entry("epic/5", 11)]);
        let text = String::from_utf8(cache.to_bytes()).expect("utf-8");
        let document: serde_json::Value = serde_json::from_str(&text).expect("JSON");
        let slugs: Vec<&str> = document["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .map(|entry| entry["slug"].as_str().expect("a slug"))
            .collect();
        assert_eq!(slugs, vec!["epic/5", "steam/900003"]);
        assert_eq!(document["version"].as_u64(), Some(1));
    }

    #[test]
    fn a_write_past_the_entry_cap_drops_the_oldest_fetched_at() {
        let many: Vec<(String, CacheEntry)> = (0..MAX_CACHE_ENTRIES)
            .map(|index| entry(&format!("steam/{}", 700_000 + index), 1_000 + index as u64))
            .collect();
        let cache = seeded(many);
        let (staged, bytes) = cache.stage(
            "steam/900999",
            vec![Candidate {
                game_id: "fresh".to_string(),
                name: "Fresh Game".to_string(),
            }],
            10_000,
        );
        assert_eq!(staged.len(), MAX_CACHE_ENTRIES, "never cap + 1");
        assert!(
            !staged.contains_key("steam/700000"),
            "the oldest went first"
        );
        assert!(staged.contains_key("steam/900999"));
        assert!(bytes.len() <= MAX_CACHE_DOCUMENT_OCTETS);
    }

    #[test]
    fn a_write_past_the_octet_cap_drops_oldest_until_the_buffer_fits() {
        let fat = |slug: &str, fetched_at: u64| {
            (
                slug.to_string(),
                CacheEntry {
                    candidates: (0..200)
                        .map(|index| Candidate {
                            game_id: format!("cand-{index:04}"),
                            name: "A cached candidate name".to_string(),
                        })
                        .collect(),
                    fetched_at,
                },
            )
        };
        let mut entries: Vec<(String, CacheEntry)> = Vec::new();
        for index in 0..120 {
            entries.push(fat(&format!("steam/{:06}", 400_000 + index), 1_000 + index));
        }
        let cache = seeded(entries);
        let (_staged, over) = cache.stage("steam/900888", vec![], 10_000);
        assert!(
            over.len() <= MAX_CACHE_DOCUMENT_OCTETS,
            "the renamed buffer is inside `026-B2`: {}",
            over.len()
        );
    }

    #[test]
    fn a_stamp_ahead_of_the_clock_is_expired_not_immortal() {
        let future = CacheEntry {
            candidates: Vec::new(),
            fetched_at: 1_000_000,
        };
        assert!(future.is_fresh(1_000_000, 3_600), "its own second is fresh");
        assert!(
            !future.is_fresh(999_999, 3_600),
            "a stamp ahead of the clock is expired, never immortal"
        );
        assert!(future.is_fresh(1_003_599, 3_600));
        assert!(!future.is_fresh(1_003_600, 3_600), "at the TTL it is stale");
    }

    #[test]
    fn a_load_refuses_every_shape_out_of_contract_by_name() {
        let now = 1_800_000_000;
        let poisoned: Vec<(serde_json::Value, &str)> = vec![
            (serde_json::json!({"version": 2, "entries": []}), "version"),
            (serde_json::json!({"version": 1}), "entries"),
            (
                serde_json::json!({"version": 1, "entries": {"steam/9": []}}),
                "entries",
            ),
            (
                serde_json::json!({"version": 1, "entries": ["steam/900001"]}),
                "entries",
            ),
            (
                serde_json::json!({"version": 1, "entries": [
                    {"slug": "steam/900001", "fetched_at": now}]}),
                "candidates",
            ),
            (
                serde_json::json!({"version": 1, "entries": [{"slug": "steam/900001",
                    "candidates": [{"game_id": "../x", "name": "X"}], "fetched_at": now}]}),
                "game_id",
            ),
            (
                serde_json::json!({"version": 1, "entries": [{"slug": "steam/900001",
                    "candidates": [{"game_id": "ok-game", "name": "n"}], "fetched_at": -1}]}),
                "fetched_at",
            ),
            (
                serde_json::json!({"version": 1, "entries": [{"slug": "gog/900001",
                    "candidates": [], "fetched_at": now}]}),
                "gog/900001",
            ),
            (
                serde_json::json!({"version": 1, "entries": [
                    {"slug": "steam/900001", "candidates": [], "fetched_at": now},
                    {"slug": "steam/900001", "candidates": [], "fetched_at": now}]}),
                "steam/900001",
            ),
            (
                serde_json::json!({"version": 1, "entries": [{"slug": "steam/900001",
                    "candidates": [], "fetched_at": now - 40_000_000}]}),
                "fetched_at",
            ),
        ];
        let dir =
            std::env::temp_dir().join(format!("bindforge-cache-refusals-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        for (index, (document, needle)) in poisoned.iter().enumerate() {
            let path = dir.join(format!("poisoned-{index}-cache-v1.json"));
            std::fs::write(&path, document.to_string().as_bytes()).expect("the fixture lands");
            let error = Cache::load(&path, now)
                .err()
                .unwrap_or_else(|| panic!("{document} loads, and it must be refused"));
            let line = error.line();
            assert!(
                line.contains("cache"),
                "the refusal names the document it refused: {line}"
            );
            assert!(
                line.contains(needle),
                "the refusal for `{needle}` must name it: {line}"
            );
            assert!(!line.contains('\n'), "one refusal is one line: {line:?}");
            assert!(
                std::fs::read_to_string(&path).is_ok(),
                "the file is untouched"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_load_refuses_an_entry_carrying_more_candidates_than_one_answer_may() {
        // A1's count cap is a property of the document, so a stored entry that
        // could only be answered by truncation is a load violation, not a long
        // answer served over and over.
        let now = 1_800_000_000;
        let wide: Vec<serde_json::Value> = (0..=MAX_CANDIDATES)
            .map(|index| serde_json::json!({"game_id": format!("game-{index}"), "name": "N"}))
            .collect();
        let document = serde_json::json!({"version": 1, "entries": [
            {"slug": "steam/900001", "candidates": wide, "fetched_at": now}]});
        let dir = std::env::temp_dir().join(format!("bindforge-cache-wide-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("wide-cache-v1.json");
        std::fs::write(&path, document.to_string().as_bytes()).expect("the fixture lands");
        let error = Cache::load(&path, now)
            .expect_err("an entry over the candidate cap is refused at load");
        assert!(error.line().contains("candidates"), "{}", error.line());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_staged_entry_stores_no_more_candidates_than_one_answer_may() {
        let wide: Vec<Candidate> = (0..MAX_CANDIDATES * 2)
            .map(|index| Candidate {
                game_id: format!("game-{index}"),
                name: "N".to_string(),
            })
            .collect();
        let cache = Cache::empty(PathBuf::from("cache-v1.json"));
        let (staged, bytes) = cache.stage("steam/900001", wide, 10);
        assert_eq!(
            staged["steam/900001"].candidates.len(),
            MAX_CANDIDATES,
            "the write seam caps the list, in the stable order"
        );
        assert!(staged["steam/900001"].candidates[0].game_id == "game-0");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn an_absent_file_loads_as_the_empty_cache() {
        let path = std::env::temp_dir().join("bindforge-no-such-cache-v1.json");
        let cache = Cache::load(&path, 1_800_000_000).expect("absent is empty");
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }
}
