//! The running instance: the one mode, the documents it owns, and the hosted lookup
//! policy (ADR-027, §Hosted mode, §The hosted lookup policy, §Concurrency).
//!
//! One process, one accept loop, four locks and one mutation hold. Serving reads
//! take **shared** halves and never touch a file, so a reader cannot observe a
//! half-written document; a mutation — a corpus upsert, a steam mint, an issued or
//! corpus delete — holds `mutations` for its whole life, so persist order cannot
//! diverge from mutation order, and takes a document's write half only to swap the
//! model after its bytes are already on disk (§Concurrency's persist-before-serve).
//! Lock order is `mutations → corpus read → retired read/write → issued read/write`
//! and never the reverse. The steam snapshot has its own lock and its own writer —
//! the refresh thread — which is why no client request ever waits on a store pull.
//!
//! The policy itself is five steps and no branch of it dials anything: the corpus,
//! then the issued document, then the key's authority (`local` answers the empty
//! list, `epic` names the inactive resolver), then the snapshot for `steam`, which
//! either certifies a miss or pays the mint.

use std::sync::{Arc, Mutex, RwLock};

use bindforge_core::escape_and_truncate;
use serde_json::Value;

use crate::constants;
use crate::corpus::{Candidate, Catalog, Record};
use crate::mint::{self, MintRefusal};
use crate::persist;
use crate::retired::RetiredIds;
use crate::steam::SteamSnapshot;

/// What the instance answers one lookup key with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The join, from whichever source the policy names first. An empty list here is
    /// a *claim* — the two branches §The hosted lookup policy answers with `[]` —
    /// never a refusal that degraded.
    Candidates {
        candidates: Vec<Candidate>,
        /// Which source answered, which `registry query` prints and the serving face
        /// ignores.
        provenance: Provenance,
    },
    /// A named `503`: this instance has an answer about the *key*, and it is that it
    /// cannot answer about the game. Never `[]` (§The hosted lookup policy).
    Refusal(Named),
}

/// Which of the instance's sources answered one lookup (`027-C2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// The curated corpus, which outranks the mint and answers alone (`027-L5`).
    Corpus,
    /// The issued document: a record this instance minted.
    Issued,
    /// Nothing answered, and the instance is entitled to say so: `local/…`, or a
    /// fresh snapshot that does not hold the appid.
    None,
}

impl Provenance {
    /// The word `registry query` prints.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Corpus => "corpus",
            Self::Issued => "issued",
            Self::None => "none",
        }
    }
}

/// The one mode ADR-027 leaves the registry with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Corpus, issued document, steam snapshot.
    Hosted,
}

impl Mode {
    /// The startup line's and `health.mode`'s spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hosted => "hosted",
        }
    }
}

/// Whether the steam resolver was asked to run (`--no-steam` is the other arm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamPosture {
    /// The snapshot is the instance's own, refreshed by the run's thread.
    Active,
    /// `--no-steam`: the snapshot is never read and never pulled, and every steam
    /// miss is the named `503`.
    Inactive,
}

/// The named `503`s the hosted policy and the write paths answer with. Each one is a
/// distinct reason phrase because the status line is all the shipped CLI shows a
/// user (A2), and each one is a fact about the *instance* — which is why none of
/// them is ever spelled as an empty candidate list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Named {
    /// The loaded snapshot is past `027-B2`'s age bound: too old to certify a miss.
    SteamSnapshotStale,
    /// The resolver is active and there is no snapshot to consult.
    NoSteamSnapshot,
    /// `--no-steam`: this instance has no steam knowledge at all.
    SteamResolverInactive,
    /// The epic slot is reserved and empty (`027-C2`): `[]` would be a claim about
    /// Epic this registry cannot make.
    EpicResolverInactive,
    /// `027-B3`: at the issued cap a novel key has no persistable identity.
    MintCapReached,
    /// `027-C5`'s step 6: every spelling the mint may use is held or retired.
    MintIdentityHeld,
    /// `027-C5`'s step 5 last rung: the only available spelling is over the id bound.
    MintIdentityOverBound,
    /// `027-L2`: the retired list cannot grow, so the delete that would grow it does
    /// not happen.
    RetiredIdListFull,
    /// A persist refused the write; the model and the file are untouched.
    Unwritable(&'static str),
}

impl Named {
    /// The reason phrase, spelled *as the design names the condition* — §The hosted
    /// lookup policy's `steam snapshot stale` / `no steam snapshot` / `steam resolver
    /// inactive` and §Issued entries' `mint cap reached` ("literally as the ADR spells
    /// it"). The status line is all the shipped CLI shows a user, so the name the
    /// design, the ADR, and the runbook use is the name the line carries; the body's
    /// [`Self::code`] keeps the machine-readable spelling.
    #[must_use]
    pub const fn reason(&self) -> &'static str {
        match self {
            Self::SteamSnapshotStale => "steam snapshot stale",
            Self::NoSteamSnapshot => "no steam snapshot",
            Self::SteamResolverInactive => "steam resolver inactive",
            Self::EpicResolverInactive => "epic resolver inactive",
            Self::MintCapReached => "mint cap reached",
            Self::MintIdentityHeld => "mint identity held",
            Self::MintIdentityOverBound => "mint identity over bound",
            Self::RetiredIdListFull => "retired id list full",
            Self::Unwritable(_) => "instance document unwritable",
        }
    }

    /// The body's `error` token.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::SteamSnapshotStale => "steam_snapshot_stale",
            Self::NoSteamSnapshot => "no_steam_snapshot",
            Self::SteamResolverInactive => "steam_resolver_inactive",
            Self::EpicResolverInactive => "epic_resolver_inactive",
            Self::MintCapReached => "mint_cap_reached",
            Self::MintIdentityHeld => "mint_identity_held",
            Self::MintIdentityOverBound => "mint_identity_over_bound",
            Self::RetiredIdListFull => "retired_id_list_full",
            Self::Unwritable(_) => "instance_unwritable",
        }
    }

    /// The body's `detail`.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::SteamSnapshotStale => "the steam snapshot this instance loaded is past \
                 `027-B2`'s age bound, so it cannot certify that an appid is not in the \
                 store; the snapshot is being refreshed"
                .to_string(),
            Self::NoSteamSnapshot => "this instance has no steam snapshot to answer from: a \
                 cold store resolver never answers the empty list"
                .to_string(),
            Self::SteamResolverInactive => "this instance was started `--no-steam`, so it has \
                 no steam knowledge to answer with"
                .to_string(),
            Self::EpicResolverInactive => "the epic resolver is reserved and inactive: this \
                 registry has no epic knowledge, and the empty list would be a claim about \
                 Epic it cannot make"
                .to_string(),
            Self::MintCapReached => format!(
                "the issued document holds its {} records, so a new identity cannot be \
                 persisted; promotion frees a slot",
                constants::MAX_ISSUED_ENTRIES
            ),
            Self::MintIdentityHeld => "every identity the mint rule may derive for this \
                 appid is already held or retired by a different record: the id is \
                 curation's to solve"
                .to_string(),
            Self::MintIdentityOverBound => format!(
                "the only identity the mint rule may use for this key exceeds the \
                 {}-octet product-id bound",
                bindforge_core::MAX_SLUG_OCTETS
            ),
            Self::RetiredIdListFull => format!(
                "the retired-id list is at its {}-octet cap, so this delete cannot record \
                 the id it is retiring; the delete mutates nothing",
                constants::MAX_RETIRED_IDS_OCTETS
            ),
            Self::Unwritable(document) => format!(
                "the instance could not persist `{document}`: the model and the file are \
                 unchanged"
            ),
        }
    }

    /// Maps a mint's refusal onto the named `503` it is.
    #[must_use]
    const fn from_mint(refusal: MintRefusal) -> Self {
        match refusal {
            MintRefusal::IdentityHeld => Self::MintIdentityHeld,
            MintRefusal::IdentityOverBound => Self::MintIdentityOverBound,
        }
    }
}

/// A refusal an admin request gets, with the status to answer it with.
pub enum AdminRefusal {
    /// The record, path segment, or document cap the catalog contract refused.
    Reject { status: u16, detail: String },
    /// A `DELETE` for a `game_id` the document does not hold.
    Absent { detail: String },
    /// The document could not be persisted; the model and file are unchanged.
    Persist { detail: String },
    /// One of the instance's own capacity laws stopped the write — the retired list
    /// cannot grow (`027-L2`), or a mutation would push a document past its cap.
    ///
    /// This is the write path's share of the same `Named` vocabulary the lookup path
    /// answers with, and it is carried as a name rather than flattened into
    /// [`Self::Reject`] because the name is the answer: §The hosted lookup policy
    /// names these refusals in the status line, and a `503` whose reason phrase came
    /// out `"Bad Request"` (review r1c0, S6) is the contract's own name lost on the
    /// way out.
    Named(Named),
}

/// The shared state one accept loop hands to its connection threads.
#[derive(Debug)]
pub struct State {
    corpus: Arc<RwLock<Catalog>>,
    issued: Arc<RwLock<Catalog>>,
    retired: Arc<RwLock<RetiredIds>>,
    snapshot: Arc<RwLock<Option<Arc<SteamSnapshot>>>>,
    steam: SteamPosture,
    /// §Concurrency's one hold: corpus mutations, mints, and both deletes.
    mutations: Mutex<()>,
}

/// The mint path's decision, made under `mutations` and acted on there too: an
/// answer, or the named reason the mint did not happen.
enum Minted {
    Answered(Vec<Candidate>),
    Refused(Named),
}

impl State {
    /// Builds the hosted instance over the three documents and the snapshot the
    /// startup seam loaded (an `Option` because `--no-steam` has none, and a cold
    /// start with no snapshot file is the startup pull's job, not this type's).
    #[must_use]
    pub fn hosted(
        corpus: Catalog,
        issued: Catalog,
        retired: RetiredIds,
        snapshot: Option<SteamSnapshot>,
        steam: SteamPosture,
    ) -> Self {
        Self {
            corpus: Arc::new(RwLock::new(corpus)),
            issued: Arc::new(RwLock::new(issued)),
            retired: Arc::new(RwLock::new(retired)),
            snapshot: Arc::new(RwLock::new(snapshot.map(Arc::new))),
            steam,
            mutations: Mutex::new(()),
        }
    }

    /// The mode, for the startup line.
    #[must_use]
    pub const fn mode(&self) -> Mode {
        Mode::Hosted
    }

    /// The snapshot's lock and model, for the refresh thread — the snapshot's only
    /// writer, which takes this lock and nothing else (§Concurrency's no-cycle rule).
    #[must_use]
    pub fn snapshot_slot(&self) -> Arc<RwLock<Option<Arc<SteamSnapshot>>>> {
        Arc::clone(&self.snapshot)
    }

    /// Publishes a pulled snapshot: the refresh thread's one act on the model.
    pub fn publish_snapshot(&self, snapshot: SteamSnapshot) {
        if let Ok(mut slot) = self.snapshot.write() {
            *slot = Some(Arc::new(snapshot));
        }
    }

    /// §The hosted lookup policy, for one admitted key.
    #[must_use]
    pub fn candidates(&self, key: &str, now: u64) -> Answer {
        // Step 1: the curated join answers alone, and the issued document is never
        // consulted for that key (`027-L5`).
        let curated = self
            .corpus
            .read()
            .map(|corpus| corpus.candidates(key))
            .unwrap_or_default();
        if !curated.is_empty() {
            return Answer::Candidates {
                candidates: curated,
                provenance: Provenance::Corpus,
            };
        }
        // Step 2: a key that already has a record never pays the mint path again, at
        // the cap or anywhere.
        let minted_already = self
            .issued
            .read()
            .map(|issued| issued.candidates(key))
            .unwrap_or_default();
        if !minted_already.is_empty() {
            return Answer::Candidates {
                candidates: minted_already,
                provenance: Provenance::Issued,
            };
        }
        // Steps 3 and 4: the key's authority decides what a miss means. The set is
        // closed (`keys.rs` admits nothing outside `steam`/`epic`/`local`).
        let (authority, external_id) = key.split_once('/').unwrap_or((key, ""));
        match authority {
            "local" => Answer::Candidates {
                candidates: Vec::new(),
                provenance: Provenance::None,
            },
            "epic" => Answer::Refusal(Named::EpicResolverInactive),
            "steam" => self.steam_lookup(key, external_id, now),
            // `keys.rs` admits exactly the three tokens above, so there is no fourth
            // arm to name; an admissible key of any other authority is the cold
            // spelling, which is a refusal and never the empty list.
            _ => Answer::Refusal(Named::SteamResolverInactive),
        }
    }

    /// Step 5: the snapshot's answer for one `steam/<external-id>` key, and the mint
    /// path when the snapshot certifies that the appid is there.
    fn steam_lookup(&self, key: &str, external_id: &str, now: u64) -> Answer {
        if self.steam == SteamPosture::Inactive {
            return Answer::Refusal(Named::SteamResolverInactive);
        }
        // The snapshot's read half is taken and released here: the mint path takes
        // `mutations` below, and the refresh thread is the snapshot's only writer.
        let loaded = self
            .snapshot
            .read()
            .ok()
            .and_then(|slot| slot.as_ref().map(Arc::clone));
        let Some(snapshot) = loaded else {
            return Answer::Refusal(Named::NoSteamSnapshot);
        };
        if snapshot.is_past_age_bound(now) {
            return Answer::Refusal(Named::SteamSnapshotStale);
        }
        // From here the snapshot is fresh, so it is entitled to certify a miss —
        // including over an external id that is not an appid *spelling*, which is not a
        // game appid and never can be (`027-L3`).
        //
        // Canonical spelling only. `u64::from_str` also takes `+7` and `007`, and one
        // appid must have exactly one key spelling: that is the property §Issued entries
        // steps 3 and 5 call "appid-unique", the one the mint's held-set probes stand on,
        // and the one `027-B3`'s cap arithmetic assumes when it counts one mint per key
        // ever. A zero-padded or `+`-prefixed spelling would mint a distinct immortal
        // identity for the same snapshot record, and would spell `steam-<external_id>`
        // from padding the store never issued. So a non-canonical spelling is the same
        // case the design already names — a fresh snapshot's `200 []` for a token that is
        // not an appid — never a second identity for one game.
        let Some(appid) = external_id
            .parse::<u64>()
            .ok()
            .filter(|appid| appid.to_string() == external_id)
        else {
            return Answer::Candidates {
                candidates: Vec::new(),
                provenance: Provenance::None,
            };
        };
        let Some(app) = snapshot.app(appid) else {
            return Answer::Candidates {
                candidates: Vec::new(),
                provenance: Provenance::None,
            };
        };
        let name = app.name.clone();
        // `external_id` is canonical from here, so the mint's appid spellings — which
        // §Issued entries step 5 spells from the queried key's external id — are the
        // snapshot record's own appid, and `steam/<appid>` is one key for one record.
        // Spelled from `app.appid` rather than from the text of the key so the identity
        // cannot drift from the record even if a caller loosens the gate above.
        match self.mint(key, &app.appid.to_string(), &name) {
            Minted::Answered(candidates) => Answer::Candidates {
                candidates,
                provenance: Provenance::Issued,
            },
            Minted::Refused(named) => Answer::Refusal(named),
        }
    }

    /// `027-C4`'s persist-before-serve for one store record, under `mutations`:
    /// dedup, cap, held set, mint, persist the whole record, then swap the model.
    fn mint(&self, key: &str, external_id: &str, name: &str) -> Minted {
        let _hold = self
            .mutations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (rebuilt, candidates) = {
            // Acquired in §Concurrency's order — `corpus read → retired read/write →
            // issued read/write` — so the one place that holds three document locks at
            // once cannot read as a fourth order to whoever adds the next writer.
            let corpus = match self.corpus.read() {
                Ok(corpus) => corpus,
                Err(_) => return Minted::Refused(Named::Unwritable(persist::ISSUED_DOCUMENT)),
            };
            let retired = match self.retired.read() {
                Ok(retired) => retired,
                Err(_) => return Minted::Refused(Named::Unwritable(persist::RETIRED_IDS_DOCUMENT)),
            };
            let issued = match self.issued.read() {
                Ok(issued) => issued,
                Err(_) => return Minted::Refused(Named::Unwritable(persist::ISSUED_DOCUMENT)),
            };
            // The dedup check is inside the hold, which is what stops two concurrent
            // mints of one key from both passing it.
            let already = issued.candidates(key);
            if !already.is_empty() {
                return Minted::Answered(already);
            }
            // `027-B3`: an identity that cannot be persisted is not an identity.
            if issued.game_count() >= constants::MAX_ISSUED_ENTRIES {
                return Minted::Refused(Named::MintCapReached);
            }
            let held = |candidate: &str| {
                corpus.holds(candidate) || issued.holds(candidate) || retired.contains(candidate)
            };
            let game_id = match mint::mint_id(name, external_id, &held) {
                Ok(game_id) => game_id,
                Err(refusal) => return Minted::Refused(Named::from_mint(refusal)),
            };
            let record = Record {
                game_id: game_id.clone(),
                name: name.to_string(),
                source_keys: vec![key.to_string()],
            };
            // §Concurrency's mint shape: a mint "computes and serializes the new
            // document straight into the write buffer" and "inserts the new key's
            // posting rather than rebuilding the index". `Catalog::to_bytes_with` does
            // exactly that against the live model — the old shape cloned the whole
            // catalog, rebuilt every posting, and re-serialized the document per novel
            // key, all under the one mutation lock, which at `027-B3`'s cap is the cost
            // the section priced out.
            let bytes = issued.to_bytes_with(&record);
            // The octet budget refuses a novel key the same way the record count
            // does (§Budgets), measured against the exact buffer a rename would run.
            if bytes.len() > constants::MAX_ISSUED_DOCUMENT_OCTETS {
                return Minted::Refused(Named::MintCapReached);
            }
            // The named `503` on a failed persist, with the model, the `revision`,
            // and the file all untouched.
            if persist::save(issued.path(), &bytes).is_err() {
                return Minted::Refused(Named::Unwritable(persist::ISSUED_DOCUMENT));
            }
            drop(corpus);
            drop(retired);
            drop(issued);
            (
                record,
                vec![Candidate {
                    game_id,
                    name: name.to_string(),
                }],
            )
        };
        // The in-memory document takes the insert its serialized bytes already carry:
        // same `game_id` order, same posting insert, same `revision` bump, under the
        // write lock with `mutations` still held, so no other writer moved the document
        // between the persist and the swap.
        if let Ok(mut issued) = self.issued.write() {
            issued.insert(rebuilt);
        }
        Minted::Answered(candidates)
    }

    /// The `026-C2` health document, re-pinned by `027-C6`: four fields, with the
    /// snapshot's own `{games, fetched_at}` shape or the `"none"` spelling. The
    /// issued count is deliberately not public health — it is in `stats`.
    #[must_use]
    pub fn health(&self) -> Vec<(&'static str, Value)> {
        let corpus_entries = self
            .corpus
            .read()
            .map(|corpus| corpus.game_count() as u64)
            .unwrap_or_default();
        let steam_snapshot = match self.snapshot.read().ok().and_then(|slot| slot.clone()) {
            Some(snapshot) => serde_json::json!({
                "games": snapshot.len() as u64,
                "fetched_at": snapshot.fetched_at(),
            }),
            None => Value::String("none".to_string()),
        };
        vec![
            ("status", Value::String("serving".to_string())),
            ("mode", Value::String(self.mode().as_str().to_string())),
            ("corpus_entries", Value::from(corpus_entries)),
            ("steam_snapshot", steam_snapshot),
        ]
    }

    /// The four counts `027-C6` gives `GET /v1/admin/stats`: the shipped three from
    /// the corpus, plus the issued document's record count. Counts only, never a key
    /// or name list (`026-L3`).
    #[must_use]
    pub fn stats(&self) -> Vec<(&'static str, Value)> {
        let (games, source_keys, revision) = match self.corpus.read() {
            Ok(corpus) => (
                corpus.game_count() as u64,
                corpus.source_key_count() as u64,
                corpus.revision(),
            ),
            Err(_) => (0, 0, 0),
        };
        let issued = self
            .issued
            .read()
            .map(|issued| issued.game_count() as u64)
            .unwrap_or_default();
        vec![
            ("games", Value::from(games)),
            ("source_keys", Value::from(source_keys)),
            ("revision", Value::from(revision)),
            ("issued", Value::from(issued)),
        ]
    }

    /// Upserts one whole curated record: validated by the caller through
    /// [`crate::corpus::record_from_value`], then serialized whole, capped,
    /// persisted and swapped, under the one mutation hold.
    ///
    /// # Errors
    ///
    /// Returns the named `422` when the corpus document with the record in it is over
    /// `MAX_CATALOG_DOCUMENT_OCTETS`, or a persist refusal.
    pub fn upsert(&self, record: Record) -> Result<(), AdminRefusal> {
        let _hold = self
            .mutations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (path, rebuilt) = {
            let corpus = self.corpus.read().map_err(|_| AdminRefusal::Persist {
                detail: "the corpus model is unavailable".to_string(),
            })?;
            let mut rebuilt = corpus.clone();
            rebuilt.upsert(record);
            (corpus.path().to_path_buf(), rebuilt)
        };
        let bytes = rebuilt.to_bytes();
        if bytes.len() > constants::MAX_CATALOG_DOCUMENT_OCTETS {
            return Err(AdminRefusal::Reject {
                status: 422,
                detail: format!(
                    "the catalog document with this record in it is {} octets, over the \
                     {}-octet cap",
                    bytes.len(),
                    constants::MAX_CATALOG_DOCUMENT_OCTETS
                ),
            });
        }
        persist::save(&path, &bytes).map_err(|error| AdminRefusal::Persist {
            detail: error.reason,
        })?;
        if let Ok(mut corpus) = self.corpus.write() {
            *corpus = rebuilt;
        }
        Ok(())
    }

    /// Removes one curated record. Every delete — issued or corpus — is two persists
    /// under one hold, retired first (§Issued entries' `027-L2`).
    ///
    /// # Errors
    ///
    /// Returns [`AdminRefusal::Absent`] for a `game_id` the corpus does not hold, the
    /// named refusal when the retired list cannot grow, or a persist refusal.
    pub fn remove(&self, game_id: &str) -> Result<(), AdminRefusal> {
        let _hold = self
            .mutations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let held = self
            .corpus
            .read()
            .map(|corpus| corpus.holds(game_id))
            .unwrap_or(false);
        if !held {
            return Err(AdminRefusal::Absent {
                detail: format!(
                    "the document holds no record with game_id `{}`",
                    escape_and_truncate(game_id)
                ),
            });
        }
        self.append_retired(game_id)?;
        let (path, rebuilt) = {
            let corpus = self.corpus.read().map_err(|_| AdminRefusal::Persist {
                detail: "the corpus model is unavailable".to_string(),
            })?;
            let mut rebuilt = corpus.clone();
            rebuilt.remove(game_id);
            (corpus.path().to_path_buf(), rebuilt)
        };
        let bytes = rebuilt.to_bytes();
        if bytes.len() > constants::MAX_CATALOG_DOCUMENT_OCTETS {
            return Err(AdminRefusal::Reject {
                status: 422,
                detail: "the catalog document after the delete is over the cap".to_string(),
            });
        }
        persist::save(&path, &bytes).map_err(|error| AdminRefusal::Persist {
            detail: error.reason,
        })?;
        if let Ok(mut corpus) = self.corpus.write() {
            *corpus = rebuilt;
        }
        Ok(())
    }

    /// `DELETE /v1/admin/issued/<game_id>`: the mint's record leaves the document and
    /// its id enters the retired set, in the same two persists under one hold.
    ///
    /// # Errors
    ///
    /// As [`Self::remove`].
    pub fn remove_issued(&self, game_id: &str) -> Result<(), AdminRefusal> {
        let _hold = self
            .mutations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let held = self
            .issued
            .read()
            .map(|issued| issued.holds(game_id))
            .unwrap_or(false);
        if !held {
            return Err(AdminRefusal::Absent {
                detail: format!(
                    "the issued document holds no record with game_id `{}`",
                    escape_and_truncate(game_id)
                ),
            });
        }
        self.append_retired(game_id)?;
        let (path, rebuilt) = {
            let issued = self.issued.read().map_err(|_| AdminRefusal::Persist {
                detail: "the issued model is unavailable".to_string(),
            })?;
            let mut rebuilt = issued.clone();
            rebuilt.remove(game_id);
            (issued.path().to_path_buf(), rebuilt)
        };
        let bytes = rebuilt.to_bytes();
        if bytes.len() > constants::MAX_ISSUED_DOCUMENT_OCTETS {
            return Err(AdminRefusal::Reject {
                status: 422,
                detail: "the issued document after the delete is over the cap".to_string(),
            });
        }
        persist::save(&path, &bytes).map_err(|error| AdminRefusal::Persist {
            detail: error.reason,
        })?;
        if let Ok(mut issued) = self.issued.write() {
            *issued = rebuilt;
        }
        Ok(())
    }

    /// The first of a delete's two persists: the id joins the retired set, and the
    /// list's own cap is paid against the exact buffer a rename would run. A no-op
    /// append — the id is already in the set — persists nothing, which is what lets a
    /// delete whose first half already landed be retried against an at-cap list.
    ///
    /// The caller holds `mutations`.
    fn append_retired(&self, game_id: &str) -> Result<(), AdminRefusal> {
        let (path, rebuilt) = {
            let retired = self.retired.read().map_err(|_| AdminRefusal::Persist {
                detail: "the retired-id model is unavailable".to_string(),
            })?;
            if retired.contains(game_id) {
                return Ok(());
            }
            let mut rebuilt = retired.clone();
            rebuilt.insert(game_id);
            (retired.path().to_path_buf(), rebuilt)
        };
        let bytes = rebuilt.to_bytes();
        if bytes.len() > constants::MAX_RETIRED_IDS_OCTETS {
            // Carried as the name, so the status line says `retired id list full` and
            // the body carries `retired_id_list_full` — the same named rendering the
            // lookup path uses for its `503`s, rather than a `503` whose reason phrase
            // fell through to `"Bad Request"`.
            return Err(AdminRefusal::Named(Named::RetiredIdListFull));
        }
        persist::save(&path, &bytes).map_err(|error| AdminRefusal::Persist {
            detail: error.reason,
        })?;
        if let Ok(mut retired) = self.retired.write() {
            *retired = rebuilt;
        }
        Ok(())
    }
}
