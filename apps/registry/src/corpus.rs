//! The `026-C3` catalog document: load, validation, the slug→candidates index, and
//! the whole-buffer serialization an admin write renames into place.
//!
//! The loader refuses **by name** (`026-C3`, A7's load discipline): a wrong
//! `version`, a negative `revision`, a `game_id` failing the grammar or bound (A4),
//! a repeated `game_id` (records are keyed by it, so a duplicate loads ambiguously),
//! a `name` over `013-B4`'s bound (A5), a `source_key` failing the key rule (A16,
//! A3), a `source_key` repeated inside one record (the index would post the same
//! game twice and crowd the `MAX_CANDIDATES` window), a non-object or missing field,
//! and a document over `MAX_CATALOG_DOCUMENT_OCTETS`. Nothing on disk is rewritten by
//! a refusal, and the same `record_from_value` gate serves the admin upsert, so no
//! accepted mutation can persist a corpus its own loader refuses.
//!
//! The entry set is **closed** at both levels (`schemas/catalog/v1/spec.md`): an entry
//! outside `version`/`revision`/`games`, or outside a record's three, is a named
//! refusal — an entry the contract does not spell is never silently ignored.
//!
//! The projection side carries the response budget too: [`response_bounded`] trims a
//! candidate list to the largest stable prefix that fits `MAX_RESPONSE_OCTETS`,
//! because the shipped client's reader (A1) refuses a larger body and a root that
//! emitted one would stop serving the very face `026-C1` pins.

use crate::constants::{DOCUMENT_VERSION, MAX_CANDIDATES, MAX_RESPONSE_OCTETS};
use crate::keys::key_is_admissible;
use crate::persist;
use bindforge_core::{MAX_NATIVE_NAME_OCTETS, MAX_SLUG_OCTETS, escape_and_truncate, is_slug_name};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The entries `026-C3` spells at the top of a document, and no others: an entry
/// outside this set is a named refusal, not an ignored key.
const DOCUMENT_ENTRIES: [&str; 3] = ["version", "revision", "games"];

/// The entries `026-C3` spells on one record, and no others.
const RECORD_ENTRIES: [&str; 3] = ["game_id", "name", "source_keys"];

/// One curated record, as `026-C3` spells it.
#[derive(Debug, Clone)]
pub struct Record {
    pub game_id: String,
    pub name: String,
    pub source_keys: Vec<String>,
}

/// One candidate as the serving face projects it: `{game_id, name}` and nothing else
/// (`026-C2`'s projection, `026-L6`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub game_id: String,
    pub name: String,
}

impl Candidate {
    #[must_use]
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"game_id": self.game_id, "name": self.name})
    }

    /// The octets this candidate costs inside a response array (its own serialized
    /// object, without the separator — [`response_bounded`] counts that).
    #[must_use]
    fn response_octets(&self) -> usize {
        serde_json::to_vec(&self.json())
            .map(|bytes| bytes.len())
            .unwrap_or(usize::MAX)
    }
}

/// The largest prefix of `candidates` — which arrive in `game_id` byte order — that
/// fits **both** caps A1 pins on one answer: [`MAX_CANDIDATES`] candidates and
/// [`MAX_RESPONSE_OCTETS`] of body.
///
/// The count cap alone is not enough: every field a projection carries is itself
/// bounded at 128 octets (`024-B1`, `013-B4`), a bound the shipped
/// `fixtures/valid/on-every-length-bound.json` proves is inclusive, so
/// `MAX_CANDIDATES` max-bound records serialize to ≈72,000 octets — a body the
/// shipped client's reader refuses as over budget, and which would therefore turn a
/// legal root answer into `catalog unreachable` for the CLI and into an upstream
/// error for a caching instance. Trimming the *same stable window* the count cap
/// uses keeps the answer inside the cap and stable across requests, and it is the
/// answer the client's own budgets describe. Because a candidate costs at most ~283
/// octets, a non-empty list always keeps at least one candidate.
#[must_use]
pub fn response_bounded(candidates: &[Candidate]) -> &[Candidate] {
    let mut total = 2usize; // the brackets of `[]`
    let mut kept = 0usize;
    for candidate in candidates.iter().take(MAX_CANDIDATES) {
        let separator = usize::from(kept > 0);
        let cost = candidate.response_octets() + separator;
        if total + cost > MAX_RESPONSE_OCTETS {
            break;
        }
        total += cost;
        kept += 1;
    }
    &candidates[..kept]
}

/// The `GET /v1/candidates` body for one answer: the octet-bounded prefix of the
/// join, projected to `{game_id, name}` and framed as a JSON array.
#[must_use]
pub fn candidates_body(candidates: &[Candidate]) -> Vec<u8> {
    let items: Vec<serde_json::Value> = response_bounded(candidates)
        .iter()
        .map(Candidate::json)
        .collect();
    serde_json::to_vec(&serde_json::Value::Array(items)).expect("a list of candidates serializes")
}

/// A document this loader will not run: the file it refused, and the named
/// condition. One refusal is one line (`026-C3`).
#[derive(Debug)]
pub struct LoadError {
    pub path: PathBuf,
    pub detail: String,
}

impl LoadError {
    pub(crate) fn at(path: &Path, detail: impl Into<String>) -> Self {
        Self {
            path: path.to_path_buf(),
            detail: detail.into(),
        }
    }

    /// The single stderr line this refusal is reported as.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "error: {}: {}",
            persist::echo_path(&self.path),
            escape_and_truncate(&self.detail)
        )
    }
}

/// The in-memory catalog: records keyed by `game_id` (byte order), plus the
/// slug→candidates index built once at load with postings already in `game_id`
/// byte order, so a request is one probe plus a bounded clone (Chosen design:
/// §The serving face step 2).
#[derive(Debug, Clone)]
pub struct Catalog {
    path: PathBuf,
    revision: u64,
    records: BTreeMap<String, Record>,
    postings: BTreeMap<String, Vec<String>>,
}

impl Catalog {
    /// An empty document over `path`, with `revision` 0.
    #[must_use]
    pub fn empty(path: PathBuf) -> Self {
        Self {
            path,
            revision: 0,
            records: BTreeMap::new(),
            postings: BTreeMap::new(),
        }
    }

    /// The file this catalog persists to.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads and validates the document at `path`, treating an absent file as the
    /// empty document — the rule `026-C7` states for a caching instance's
    /// local-entries document ("created empty if absent").
    pub fn load(path: &Path, cap: usize) -> Result<Self, LoadError> {
        Self::load_at(path, cap, true)
    }

    /// The same read for a document the operator named on the command line: an
    /// absent file is the named startup refusal rather than an empty catalog.
    /// `--corpus` is that document, and only it: the absent-is-empty rule the
    /// contract states belongs to the local-entries document and to the cache
    /// (`026-C7`), and a root instance whose one content argument does not exist
    /// must be told — otherwise an operator's typo in the only path a root has
    /// becomes a silently empty public catalog instead of the named refusal every
    /// other corpus fault gets.
    pub fn load_required(path: &Path, cap: usize) -> Result<Self, LoadError> {
        Self::load_at(path, cap, false)
    }

    fn load_at(path: &Path, cap: usize, absent_is_empty: bool) -> Result<Self, LoadError> {
        let bytes =
            persist::read_capped(path, cap).map_err(|error| LoadError::at(path, error.reason))?;
        let Some(bytes) = bytes else {
            if absent_is_empty {
                return Ok(Self::empty(path.to_path_buf()));
            }
            return Err(LoadError::at(
                path,
                "the catalog document is not there: `--corpus` names a file this instance \
                 answers from, and it is not created when absent"
                    .to_string(),
            ));
        };
        let document: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
            LoadError::at(
                path,
                format!(
                    "the document is not readable JSON, and `version` must be the bare \
                     integer {DOCUMENT_VERSION}: {error}"
                ),
            )
        })?;
        Self::from_document(&document, path)
    }

    /// Validates a parsed document and builds the index over it.
    pub fn from_document(document: &serde_json::Value, path: &Path) -> Result<Self, LoadError> {
        let refused = |detail: String| LoadError::at(path, detail);
        let fields = document
            .as_object()
            .ok_or_else(|| refused("the document is not a JSON object".to_string()))?;
        if let Some(extra) = fields
            .keys()
            .find(|name| !DOCUMENT_ENTRIES.contains(&name.as_str()))
        {
            return Err(refused(format!(
                "`{}` is not an entry `026-C3` spells: a document's entry set is closed to \
                 `version`, `revision`, and `games`",
                escape_and_truncate(extra)
            )));
        }
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
                "`version` {version} is not the bare integer {DOCUMENT_VERSION} `026-C3` spells"
            )));
        }
        let revision = fields
            .get("revision")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                refused("`revision` is missing or is not a non-negative integer".to_string())
            })?;
        let games = fields
            .get("games")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                refused("`games` is missing or is not an array of records".to_string())
            })?;
        let mut records: BTreeMap<String, Record> = BTreeMap::new();
        for game in games {
            let record = record_from_value(game).map_err(refused)?;
            if records
                .insert(record.game_id.clone(), record.clone())
                .is_some()
            {
                return Err(refused(format!(
                    "`game_id` `{}` is repeated: records are keyed by it, so the document \
                     loads ambiguously",
                    escape_and_truncate(&record.game_id)
                )));
            }
        }
        let mut catalog = Self {
            path: path.to_path_buf(),
            revision,
            records,
            postings: BTreeMap::new(),
        };
        catalog.rebuild_postings();
        Ok(catalog)
    }

    /// Rebuilds the slug→candidates index from the records, in `game_id` byte order:
    /// `records` is keyed by `game_id`, so walking it in key order is what orders the
    /// postings (Chosen design: §The serving face step 2).
    fn rebuild_postings(&mut self) {
        let mut postings: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (game_id, record) in &self.records {
            for key in &record.source_keys {
                postings
                    .entry(key.clone())
                    .or_default()
                    .push(game_id.clone());
            }
        }
        self.postings = postings;
    }

    /// `026-C4`'s `revision`, the value `stats` reports and every mutation bumps.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// `stats`' `games`.
    #[must_use]
    pub fn game_count(&self) -> usize {
        self.records.len()
    }

    /// `stats`' `source_keys`: the sum of the records' key counts, never a list.
    #[must_use]
    pub fn source_key_count(&self) -> usize {
        self.records
            .values()
            .map(|record| record.source_keys.len())
            .sum()
    }

    /// The join for one admitted key: the postings, already in `game_id` byte order,
    /// truncated to the stable first `MAX_CANDIDATES` and projected to
    /// `{game_id, name}`. The count cap is the *index* bound; the answer seam trims
    /// the same stable window again to fit `MAX_RESPONSE_OCTETS`
    /// ([`response_bounded`]), which is what makes the body one the shipped client
    /// accepts.
    #[must_use]
    pub fn candidates(&self, key: &str) -> Vec<Candidate> {
        let Some(postings) = self.postings.get(key) else {
            return Vec::new();
        };
        postings
            .iter()
            .take(MAX_CANDIDATES)
            .filter_map(|game_id| self.records.get(game_id))
            .map(|record| Candidate {
                game_id: record.game_id.clone(),
                name: record.name.clone(),
            })
            .collect()
    }

    /// The whole document as the bytes a save renames into place: `version`, the
    /// `revision` the model carries, and the records in `game_id` byte order.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        Self::write_document(self.revision, self.records.values())
    }

    /// The document as it would read with `record` in it, computed **against the live
    /// model rather than a clone of it**: the same `game_id` byte order
    /// [`Catalog::to_bytes`] writes, with this one record spliced into its place, under
    /// the `revision` the matching [`Catalog::insert`] will carry.
    ///
    /// This is the seam §Concurrency's mint shape needs — a writer "computes and
    /// serializes the new document straight into the write buffer" — because at
    /// `027-B3`'s cap a mint that cloned the model first paid a full records-and-postings
    /// copy, an index rebuild, and the whole serialization per novel key, all inside the
    /// one mutation lock. A `game_id` the document already holds is replaced, the way
    /// [`Catalog::upsert`] replaces it.
    #[must_use]
    pub fn to_bytes_with(&self, record: &Record) -> Vec<u8> {
        let mut games: Vec<&Record> = Vec::with_capacity(self.records.len() + 1);
        let mut spliced = false;
        for existing in self.records.values() {
            match existing.game_id.cmp(&record.game_id) {
                std::cmp::Ordering::Less => games.push(existing),
                std::cmp::Ordering::Equal => {
                    games.push(record);
                    spliced = true;
                }
                std::cmp::Ordering::Greater => {
                    if !spliced {
                        games.push(record);
                        spliced = true;
                    }
                    games.push(existing);
                }
            }
        }
        if !spliced {
            games.push(record);
        }
        Self::write_document(self.revision + 1, games.iter().copied())
    }

    /// The one document spelling: `version`, the given `revision`, and `games` in the
    /// order handed over.
    fn write_document<'a>(revision: u64, games: impl Iterator<Item = &'a Record>) -> Vec<u8> {
        let games: Vec<serde_json::Value> = games
            .map(|record| {
                serde_json::json!({
                    "game_id": record.game_id,
                    "name": record.name,
                    "source_keys": record.source_keys,
                })
            })
            .collect();
        serde_json::to_vec(&serde_json::json!({
            "version": DOCUMENT_VERSION,
            "revision": revision,
            "games": games,
        }))
        .expect("a catalog document of plain records serializes")
    }

    /// Adds a record the document does not hold yet: `revision` bumps, and the
    /// postings are **inserted** — one map entry per source key, the new `game_id` put
    /// into the existing posting at its ordered position. That is §Concurrency's mint
    /// shape ("a mint inserts the new key's posting rather than rebuilding the index"):
    /// rebuilding at the issued cap would walk every record and re-clone every posted id
    /// per mint, under the mutation lock, for one new key.
    ///
    /// A `game_id` the document already holds is not an insert, so it falls back to
    /// [`Catalog::upsert`] — the operation that can retire the postings the replaced
    /// record used to hold.
    pub fn insert(&mut self, record: Record) {
        if self.records.contains_key(&record.game_id) {
            self.upsert(record);
            return;
        }
        let game_id = record.game_id.clone();
        for key in &record.source_keys {
            let posting = self.postings.entry(key.clone()).or_default();
            if let Err(at) = posting.binary_search(&game_id) {
                posting.insert(at, game_id.clone());
            }
        }
        self.records.insert(game_id, record);
        self.revision += 1;
    }

    /// `026-C4`'s upsert: replaces or adds the whole record, bumps `revision`, and
    /// rebuilds the index so the new key answers from the in-memory model at once.
    pub fn upsert(&mut self, record: Record) {
        self.records.insert(record.game_id.clone(), record);
        self.revision += 1;
        self.rebuild_postings();
    }

    /// `026-C4`'s delete: removes the record, bumps `revision`, and rebuilds the
    /// index. `false` when the record was not there, which bumps nothing.
    pub fn remove(&mut self, game_id: &str) -> bool {
        if self.records.remove(game_id).is_none() {
            return false;
        }
        self.revision += 1;
        self.rebuild_postings();
        true
    }

    /// Whether `game_id` names a record this catalog holds — the delete route's
    /// "is it there" question.
    #[must_use]
    pub fn holds(&self, game_id: &str) -> bool {
        self.records.contains_key(game_id)
    }
}

/// The shared `game_id` rule (A4): the grammar a product id rides, plus its bound.
/// Copied from the shipped client, which is the sixth copied invariant.
#[must_use]
pub fn game_id_is_valid(game_id: &str) -> bool {
    is_slug_name(game_id) && game_id.len() <= MAX_SLUG_OCTETS
}

/// Validates one `{game_id, name, source_keys}` record, returning the named
/// condition as a `String` so the loader and the admin upsert refuse identically.
pub fn record_from_value(value: &serde_json::Value) -> Result<Record, String> {
    let fields = value
        .as_object()
        .ok_or_else(|| "a `games` record is not a JSON object".to_string())?;
    if let Some(extra) = fields
        .keys()
        .find(|name| !RECORD_ENTRIES.contains(&name.as_str()))
    {
        return Err(format!(
            "`{}` is not an entry `026-C3` spells on a record: a record is exactly `game_id`, \
             `name`, and `source_keys`",
            escape_and_truncate(extra)
        ));
    }
    let game_id = fields
        .get("game_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "`game_id` is missing or is not a string".to_string())?;
    if !game_id_is_valid(game_id) {
        return Err(format!(
            "`game_id` `{}` fails the slug grammar `[a-z][a-z0-9-]*` or the \
             {MAX_SLUG_OCTETS}-octet bound",
            escape_and_truncate(game_id)
        ));
    }
    let name = fields
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "`name` is missing or is not a string".to_string())?;
    if name.len() > MAX_NATIVE_NAME_OCTETS {
        return Err(format!(
            "`name` is {} octets, over the {MAX_NATIVE_NAME_OCTETS}-octet bound (`013-B4`)",
            name.len()
        ));
    }
    let keys = fields
        .get("source_keys")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "`source_keys` is missing or is not an array of lookup keys".to_string())?;
    let mut source_keys: Vec<String> = Vec::with_capacity(keys.len());
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for key in keys {
        let spelled = key
            .as_str()
            .ok_or_else(|| "`source_key` is not a string".to_string())?;
        if !key_is_admissible(spelled) {
            return Err(format!(
                "`source_key` `{}` is not a `<token>/<external-id>` lookup key",
                escape_and_truncate(spelled)
            ));
        }
        if !seen.insert(spelled) {
            return Err(format!(
                "`source_key` `{}` is repeated within one record",
                escape_and_truncate(spelled)
            ));
        }
        source_keys.push(spelled.to_string());
    }
    Ok(Record {
        game_id: game_id.to_string(),
        name: name.to_string(),
        source_keys,
    })
}

#[cfg(test)]
mod tests {
    //! The loader's refusals and its arithmetic, in-process: the join's order and
    //! truncation, the projection, and the record gate the admin face shares.
    use super::{
        Catalog, Record, candidates_body, game_id_is_valid, record_from_value, response_bounded,
    };
    use crate::constants::{MAX_CANDIDATES, MAX_RESPONSE_OCTETS};
    use std::path::Path;

    const MAX_SLUG_OCTETS_TEST: usize = 128;

    fn document(games: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({"version": 1, "revision": 3, "games": games})
    }

    fn record(game_id: &str, keys: &[&str]) -> serde_json::Value {
        serde_json::json!({"game_id": game_id, "name": "A Game", "source_keys": keys})
    }

    /// The incremental document and the whole document are one spelling.
    ///
    /// §Concurrency's mint shape serializes the new document **against the live model**
    /// and inserts the new key's posting rather than rebuilding the index, which is only
    /// a cheaper spelling of the same contract if the bytes agree with the model-clone
    /// shape it replaced. This pins that, so the mint's fast path cannot drift from the
    /// document every other seam — loader, admin face, the next mint — reads.
    #[test]
    fn the_incremental_document_spelling_is_the_whole_document_spelling() {
        let catalog = Catalog::from_document(
            &document(vec![
                record("alpha", &["steam/900100"]),
                record("bravo", &["steam/900200"]),
                record("delta", &["steam/900400"]),
            ]),
            Path::new("issued-v1.json"),
        )
        .expect("the document loads");
        let novel = Record {
            game_id: "charlie".to_string(),
            name: "Charlie Game".to_string(),
            source_keys: vec!["steam/900300".to_string()],
        };
        let mut inserted = catalog.clone();
        inserted.insert(novel.clone());
        assert_eq!(
            catalog.to_bytes_with(&novel),
            inserted.to_bytes(),
            "the spliced bytes are the bytes the insert produces, in `game_id` order and \
             under the same `revision`"
        );

        // A `game_id` the document already holds is replaced by both spellings — the
        // splice puts it in the place the old record held, the upsert replaces it there.
        let repeat = Record {
            game_id: "bravo".to_string(),
            name: "Renamed Bravo".to_string(),
            source_keys: vec!["steam/900200".to_string(), "epic/7".to_string()],
        };
        let mut replaced = catalog.clone();
        replaced.upsert(repeat.clone());
        assert_eq!(
            catalog.to_bytes_with(&repeat),
            replaced.to_bytes(),
            "a replacement splices into the record's own place"
        );
    }

    #[test]
    fn a_load_builds_byte_ordered_postings_and_projects_them() {
        let catalog = Catalog::from_document(
            &document(vec![
                record("alphabet", &["steam/900100"]),
                record("alpha", &["steam/900100", "epic/5"]),
                record("alpha-2", &["steam/900100"]),
                record("alpha-x", &["steam/900100"]),
            ]),
            Path::new("corpus.json"),
        )
        .expect("the document loads");
        let ids: Vec<String> = catalog
            .candidates("steam/900100")
            .into_iter()
            .map(|candidate| candidate.game_id)
            .collect();
        assert_eq!(ids, vec!["alpha", "alpha-2", "alpha-x", "alphabet"]);
        assert_eq!(catalog.candidates("epic/5").len(), 1);
        assert!(catalog.candidates("steam/999999").is_empty());
        assert_eq!(catalog.revision(), 3);
        assert_eq!(catalog.game_count(), 4);
        assert_eq!(catalog.source_key_count(), 5);
        assert_eq!(
            catalog.candidates("epic/5")[0].json(),
            serde_json::json!({"game_id": "alpha", "name": "A Game"}),
            "a candidate is exactly `{{game_id, name}}`"
        );
    }

    #[test]
    fn the_record_gate_names_the_condition_it_refuses() {
        let refused: Vec<String> = [
            record("Example-Game", &["steam/1"]),
            record("example.game", &["steam/1"]),
            record("ok-id", &["gog/1"]),
            record("ok-id", &["steam/1", "steam/1"]),
            record("ok-id", &["steam/"]),
            serde_json::json!({"game_id": "ok-id", "source_keys": []}),
            serde_json::json!({"game_id": "ok-id", "name": "N"}),
            serde_json::json!("just a string"),
        ]
        .iter()
        .filter_map(|value| record_from_value(value).err())
        .collect();
        assert_eq!(refused.len(), 8, "every shape is refused: {refused:?}");
        assert!(
            refused.iter().all(|line| !line.contains('\n')),
            "one refusal is one line: {refused:?}"
        );
        assert!(refused[0].contains("game_id"));
        assert!(refused[2].contains("source_key"));
        assert!(refused[5].contains("name"));
        assert!(refused[6].contains("source_keys"));
    }

    #[test]
    fn a_record_on_every_bound_loads_and_one_more_octet_does_not() {
        let on_bound = serde_json::json!({
            "game_id": "a".repeat(MAX_SLUG_OCTETS_TEST),
            "name": "n".repeat(128),
            "source_keys": [format!("steam/{}", "9".repeat(122))],
        });
        record_from_value(&on_bound).expect("on the bound loads");
        let over = serde_json::json!({
            "game_id": "a".repeat(MAX_SLUG_OCTETS_TEST + 1),
            "name": "n",
            "source_keys": [],
        });
        assert!(record_from_value(&over).is_err());
        assert!(game_id_is_valid("example-game"));
        assert!(!game_id_is_valid("Example"));
    }

    #[test]
    fn an_entry_outside_the_closed_set_is_a_named_refusal() {
        // The spec's sentence — "the entry set is closed" — is a refusal the loader
        // performs, at document level and at record level, not prose about a key the
        // reader happens not to look at.
        let mut padded = document(vec![record("example-game", &["steam/1"])]);
        padded
            .as_object_mut()
            .expect("a document object")
            .insert("games_extra".to_string(), serde_json::json!([]));
        let error = Catalog::from_document(&padded, Path::new("corpus.json"))
            .expect_err("an extra document entry is refused");
        assert!(error.line().contains("games_extra"), "{}", error.line());

        let mut padded_record = record("example-game", &["steam/1"]);
        padded_record
            .as_object_mut()
            .expect("a record object")
            .insert("gameId".to_string(), serde_json::json!("typo"));
        let detail =
            record_from_value(&padded_record).expect_err("an extra record entry is refused");
        assert!(detail.contains("gameId"), "{detail}");
        assert!(!detail.contains('\n'), "one refusal is one line: {detail}");
    }

    #[test]
    fn a_wide_answer_is_trimmed_to_the_stable_prefix_the_pinned_client_accepts() {
        // `MAX_CANDIDATES` records on every field bound: the count cap alone emits a
        // body A1's reader refuses, so the octet cap has to bind the answer.
        let games: Vec<serde_json::Value> = (0..MAX_CANDIDATES)
            .map(|index| {
                let letters = [
                    (b'a' + (index / 26) as u8) as char,
                    (b'a' + (index % 26) as u8) as char,
                ];
                serde_json::json!({
                    "game_id": format!("{}{}", "g".repeat(125), letters.iter().collect::<String>()),
                    "name": "n".repeat(128),
                    "source_keys": ["steam/900001"],
                })
            })
            .collect();
        let catalog = Catalog::from_document(&document(games), Path::new("corpus.json"))
            .expect("a max-bound corpus is contract-valid");
        let joined = catalog.candidates("steam/900001");
        assert_eq!(
            joined.len(),
            MAX_CANDIDATES,
            "the count cap is not binding here"
        );
        let unbounded = serde_json::to_vec(
            &joined
                .iter()
                .map(super::Candidate::json)
                .collect::<Vec<_>>(),
        )
        .expect("candidates serialise");
        assert!(
            unbounded.len() > MAX_RESPONSE_OCTETS,
            "the wide answer really is over A1's body cap: {} octets",
            unbounded.len()
        );

        let body = candidates_body(&joined);
        assert!(
            body.len() <= MAX_RESPONSE_OCTETS,
            "an answer stays inside the budget the shipped client pins: {} octets",
            body.len()
        );
        let kept = response_bounded(&joined);
        assert!(
            !kept.is_empty() && kept.len() < MAX_CANDIDATES,
            "the octet bound binds but never empties a non-empty join: {}",
            kept.len()
        );
        assert_eq!(
            kept,
            &joined[..kept.len()],
            "the window is the stable first candidates in `game_id` order"
        );
        assert_eq!(
            serde_json::from_slice::<Vec<serde_json::Value>>(&body)
                .expect("the body is a JSON array")
                .len(),
            kept.len()
        );
        assert_eq!(
            candidates_body(&catalog.candidates("steam/900001")),
            body,
            "the truncation is stable across requests"
        );
    }

    #[test]
    fn a_mutation_bumps_revision_and_the_index_follows_it() {
        let mut catalog = Catalog::from_document(
            &document(vec![record("example-game", &["steam/900001"])]),
            Path::new("corpus.json"),
        )
        .expect("the corpus loads");
        let added = record_from_value(&record("new-game", &["steam/900002", "epic/42"]))
            .expect("a valid record");
        catalog.upsert(added);
        assert_eq!(
            catalog.revision(),
            4,
            "every accepted mutation bumps `revision`"
        );
        assert_eq!(catalog.game_count(), 2);
        assert_eq!(catalog.source_key_count(), 3);
        assert_eq!(catalog.candidates("steam/900002").len(), 1);
        assert!(catalog.remove("new-game"));
        assert_eq!(catalog.revision(), 5);
        assert!(catalog.candidates("steam/900002").is_empty());
        assert!(!catalog.remove("new-game"), "a second delete is a no-op");
        assert_eq!(catalog.revision(), 5, "a delete of nothing bumps nothing");
        assert!(catalog.holds("example-game"));
        let bytes = catalog.to_bytes();
        let round: Catalog = Catalog::from_document(
            &serde_json::from_slice::<serde_json::Value>(&bytes).expect("the bytes are a document"),
            Path::new("corpus.json"),
        )
        .expect("the document it wrote loads");
        assert_eq!(round.revision(), catalog.revision());
        assert_eq!(round.game_count(), catalog.game_count());
    }
}
