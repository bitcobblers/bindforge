//! The running instance: mode, the two locked documents, and the lookup policy
//! (design 026, Chosen design: §The serving face, §Caching mode, §Concurrency).
//!
//! One process, one accept loop, two locks. Serving reads take the **shared**
//! lock on the in-memory model and never touch a file, so a reader cannot observe
//! a half-written document; a mutation serializes the new model, writes the fixed
//! tmp name, renames it into place and swaps the model in, all under the one write
//! lock, so persist order can never diverge from mutation order.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use bindforge_core::escape_and_truncate;
use serde_json::Value;

use crate::cache::Cache;
use crate::constants;
use crate::corpus::{Candidate, Catalog, Record};
use crate::persist;
use crate::upstream::Upstream;

/// What the instance answers a slug with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The join, from whichever source the priority order names first.
    Candidates {
        candidates: Vec<Candidate>,
        /// Which source answered, which `registry query` prints and the serving
        /// face ignores.
        provenance: Provenance,
    },
    /// `026-L4`: a caching instance with nothing to answer from. Never `[]`.
    NoAnswer,
}

/// Which of the mode's sources answered one lookup (`026-C6`'s provenance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Root mode's corpus join.
    Root,
    /// The local-entries document, which is never merged and never expires
    /// (`026-L5`).
    Local,
    /// This instance's own cache entry, inside the TTL.
    Cache,
    /// The upstream, fetched for this lookup.
    Upstream,
    /// A cache entry past the TTL, served because the refresh failed.
    Stale,
}

impl Provenance {
    /// The word `registry query` prints.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Local => "local",
            Self::Cache => "cache",
            Self::Upstream => "upstream",
            // A stale answer is still a cache answer; the qualifier says why it was
            // served without the refresh.
            Self::Stale => "cache-stale",
        }
    }
}

/// The two modes `026-C1` names, chosen by the presence of `--upstream`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Answers from `--corpus` only, and never dials anyone.
    Root,
    /// Answers from the local-entries document, then its own cache, then upstream.
    Caching,
}

impl Mode {
    /// The `health.mode` spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Caching => "caching",
        }
    }
}

/// The last-known reachability verdict for `health.upstream`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Root mode: there is no upstream to report on.
    None,
    /// The last probe or fetch answered.
    Ok,
    /// The last fetch failed and a stale entry was served instead.
    Stale,
    /// Nothing has answered, or the last contact failed.
    Down,
}

impl Verdict {
    /// The `health.upstream` spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ok => "ok",
            Self::Stale => "stale",
            Self::Down => "down",
        }
    }
}

/// The gauge `026-C6` describes: the verdict as it stands, when the reachability
/// probe was last paid for, and when a candidate fetch last succeeded.
#[derive(Debug)]
struct Gauge {
    verdict: Verdict,
    last_probe_at: Option<u64>,
    last_fetch_at: u64,
}

impl Gauge {
    fn starting(mode: Mode) -> Self {
        Self {
            verdict: match mode {
                Mode::Root => Verdict::None,
                Mode::Caching => Verdict::Down,
            },
            last_probe_at: None,
            last_fetch_at: 0,
        }
    }

    /// Whether the once-per-`026-B3`-TTL probe is due. The never-probed state is
    /// an expired state, which is what makes the first fetching lookup the one
    /// that pays for it.
    fn probe_is_due(&self, now: u64, ttl_seconds: u64) -> bool {
        match self.last_probe_at {
            None => true,
            Some(at) => now.saturating_sub(at) >= ttl_seconds,
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
}

/// The shared state one accept loop hands to its connection threads.
#[derive(Debug)]
pub struct State {
    mode: Mode,
    catalog: Arc<RwLock<Catalog>>,
    cache: Arc<RwLock<Cache>>,
    upstream: Option<Upstream>,
    ttl_seconds: u64,
    gauge: Mutex<Gauge>,
}

impl State {
    /// Builds a root instance over `--corpus`.
    ///
    /// The corpus has to **exist**: a root's one content argument is named by the
    /// operator, and an absent file is the named startup refusal, not a silently
    /// empty public catalog (absent-is-empty is `026-C7`'s rule for the
    /// local-entries document and the cache).
    ///
    /// # Errors
    ///
    /// Returns the load refusal when the corpus document is outside `026-C3` or is
    /// not there.
    pub fn root(corpus: PathBuf) -> Result<Self, crate::corpus::LoadError> {
        let catalog = Catalog::load_required(&corpus, constants::MAX_CATALOG_DOCUMENT_OCTETS)?;
        // A root instance has no cache document (`026-C7`'s caching-mode rule), so
        // this is an inert empty cache that keeps the counts uniform: the name is
        // never joined onto a path it might be written to, because a root instance
        // never stages a lookup.
        let cache = Cache::empty(PathBuf::from(persist::CACHE_DOCUMENT));
        Ok(Self {
            mode: Mode::Root,
            catalog: Arc::new(RwLock::new(catalog)),
            cache: Arc::new(RwLock::new(cache)),
            upstream: None,
            ttl_seconds: constants::REGISTRY_UPSTREAM_TTL_SECONDS,
            gauge: Mutex::new(Gauge::starting(Mode::Root)),
        })
    }

    /// Builds a caching instance: local entries over `local_entries` (created
    /// empty when absent, `026-C7`), the cache over `cache`, and one gated
    /// upstream origin.
    ///
    /// # Errors
    ///
    /// Returns the load refusal when either document is present and out of shape.
    pub fn caching(
        local_entries: PathBuf,
        cache: PathBuf,
        upstream: Upstream,
        ttl_seconds: u64,
    ) -> Result<Self, crate::corpus::LoadError> {
        let now = crate::now_unix();
        let catalog = Catalog::load(&local_entries, constants::MAX_CATALOG_DOCUMENT_OCTETS)?;
        let cache = Cache::load(&cache, now)?;
        Ok(Self {
            mode: Mode::Caching,
            catalog: Arc::new(RwLock::new(catalog)),
            cache: Arc::new(RwLock::new(cache)),
            upstream: Some(upstream),
            ttl_seconds: ttl_seconds.max(1),
            gauge: Mutex::new(Gauge::starting(Mode::Caching)),
        })
    }

    /// The mode this instance runs in.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// `026-C2`'s join for one admissible slug, under the mode's priority order.
    #[must_use]
    pub fn candidates(&self, slug: &str, now: u64) -> Answer {
        if self.mode == Mode::Root {
            let answer = self
                .catalog
                .read()
                .map(|catalog| catalog.candidates(slug))
                .unwrap_or_default();
            return Answer::Candidates {
                candidates: answer,
                provenance: Provenance::Root,
            };
        }

        // 1. A local entry answers alone, is never merged, and never expires
        //    (`026-L5`), so no upstream request is made for it at all.
        let local = self
            .catalog
            .read()
            .map(|catalog| catalog.candidates(slug))
            .unwrap_or_default();
        if !local.is_empty() {
            return Answer::Candidates {
                candidates: local,
                provenance: Provenance::Local,
            };
        }

        // 2. A cached entry inside `026-B3`'s TTL answers as-is; nothing about the
        //    upstream's state is consulted, and no probe is paid for.
        let cached = self
            .cache
            .read()
            .ok()
            .and_then(|cache| cache.entry(slug).cloned())
            .filter(|entry| entry.is_fresh(now, self.ttl_seconds))
            .map(|entry| entry.candidates);
        if let Some(candidates) = cached {
            return Answer::Candidates {
                candidates,
                provenance: Provenance::Cache,
            };
        }

        // 3. Anything else goes upstream: a stale entry (step 3's serve-stale) or a
        //    cold miss (step 4/5). The reachability probe is at most one extra round
        //    trip per TTL and is fired here, by the lookup that is about to fetch,
        //    and only once the cache already holds something — a cache that cannot
        //    go stale has nothing for the gauge to decide about, and the cold fetch
        //    is its own reachability test.
        let stale = self
            .cache
            .read()
            .ok()
            .and_then(|cache| cache.entry(slug).cloned())
            .map(|entry| entry.candidates);
        self.maybe_probe(now);
        let fetched = self
            .upstream
            .as_ref()
            .map(|upstream| upstream.candidates(slug));
        // A refused answer and a transport failure are the same event to the
        // policy; "no upstream to ask" is the same thing with a different cause.
        let outcome: Result<Vec<Candidate>, String> = match fetched {
            Some(Ok(candidates)) => Ok(candidates),
            Some(Err(error)) => Err(error.0),
            None => Err("no upstream is configured to ask".to_string()),
        };
        match outcome {
            // The fetch answered: cache it, move the gauge, serve it.
            Ok(candidates) => {
                self.cache_lookup(slug, &candidates, now);
                self.set_verdict(Verdict::Ok, Some(now));
                Answer::Candidates {
                    candidates,
                    provenance: Provenance::Upstream,
                }
            }
            Err(detail) => {
                // `026-C6`'s serve-stale-on-error: a failed refresh is only a
                // verdict, never a lost answer the cache already holds.
                eprintln!(
                    "warning: the upstream answer was unusable: {}",
                    crate::upstream::echo_safe(&detail)
                );
                match stale {
                    Some(candidates) => {
                        self.set_verdict(Verdict::Stale, None);
                        Answer::Candidates {
                            candidates,
                            provenance: Provenance::Stale,
                        }
                    }
                    None => {
                        self.set_verdict(Verdict::Down, None);
                        Answer::NoAnswer
                    }
                }
            }
        }
    }

    /// Fires the reachability probe when it is due, on the calling lookup's own
    /// thread: there is no timer thread.
    fn maybe_probe(&self, now: u64) {
        let Some(upstream) = self.upstream.as_ref() else {
            return;
        };
        let has_something_to_go_stale = self.cache.read().map(|cache| !cache.is_empty());
        if !has_something_to_go_stale.unwrap_or(false) {
            return;
        }
        {
            let mut gauge = self
                .gauge
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !gauge.probe_is_due(now, self.ttl_seconds) {
                return;
            }
            // The slot is claimed before the round trip so a slow upstream cannot
            // make two concurrent lookups both pay for the same probe.
            gauge.last_probe_at = Some(now);
        }
        let verdict = if upstream.answers_health() {
            Verdict::Ok
        } else {
            Verdict::Down
        };
        let mut gauge = self
            .gauge
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gauge.verdict = verdict;
    }

    /// Records a verdict, and the fetch stamp when this verdict came from a
    /// candidate fetch that succeeded.
    fn set_verdict(&self, verdict: Verdict, fetched_at: Option<u64>) {
        let mut gauge = self
            .gauge
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gauge.verdict = verdict;
        if let Some(at) = fetched_at {
            gauge.last_fetch_at = at;
        }
    }

    /// Caches one fetched answer through the write-time caps and the tmp-and-rename
    /// discipline (§Budgets): stage, persist, then adopt.
    fn cache_lookup(&self, slug: &str, candidates: &[Candidate], now: u64) {
        let Ok(mut cache) = self.cache.write() else {
            return;
        };
        let (staged, bytes) = cache.stage(slug, candidates.to_vec(), now);
        if let Err(error) = cache.save(&bytes) {
            // The cache is a convenience, not a promise: a failed write leaves the
            // model equal to the file, and the answer is still served.
            eprintln!(
                "warning: the cache document was not written: {}",
                crate::upstream::echo_safe(&error.reason)
            );
            return;
        }
        cache.commit(staged);
    }

    /// The `026-C2` health document, read from the state as it stands: answering
    /// it never fires the probe.
    #[must_use]
    pub fn health(&self) -> Vec<(&'static str, Value)> {
        let gauge = self
            .gauge
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let local_entries = self
            .catalog
            .read()
            .map(|catalog| catalog.game_count())
            .unwrap_or_default();
        let cached_slugs = if self.mode == Mode::Root {
            0
        } else {
            self.cache
                .read()
                .map(|cache| cache.len())
                .unwrap_or_default()
        };
        vec![
            ("status", Value::String("serving".to_string())),
            ("mode", Value::String(self.mode.as_str().to_string())),
            (
                "upstream",
                Value::String(gauge.verdict.as_str().to_string()),
            ),
            ("local_entries", Value::from(local_entries as u64)),
            ("cached_slugs", Value::from(cached_slugs as u64)),
            ("last_upstream_fetch", Value::from(gauge.last_fetch_at)),
        ]
    }

    /// The three counts `026-C4` gives `GET /v1/admin/stats`, from the instance's
    /// own catalog document — never from the cache.
    #[must_use]
    pub fn stats(&self) -> Vec<(&'static str, Value)> {
        let catalog = self.catalog.read();
        match catalog {
            Ok(catalog) => vec![
                ("games", Value::from(catalog.game_count() as u64)),
                (
                    "source_keys",
                    Value::from(catalog.source_key_count() as u64),
                ),
                ("revision", Value::from(catalog.revision())),
            ],
            Err(_) => vec![
                ("games", Value::from(0)),
                ("source_keys", Value::from(0)),
                ("revision", Value::from(0)),
            ],
        }
    }

    /// Upserts one whole record: validated by the caller through
    /// [`crate::corpus::record_from_value`], then serialized whole, capped,
    /// persisted and swapped — all under the one write lock (§Concurrency).
    ///
    /// # Errors
    ///
    /// Returns the named `422` when the document with the record in it is over
    /// `MAX_CATALOG_DOCUMENT_OCTETS`, or a `500`-class refusal when the write
    /// itself failed.
    pub fn upsert(&self, record: Record) -> Result<(), AdminRefusal> {
        let mut catalog = self
            .catalog
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let rebuilt = apply_upsert(&catalog, &record);
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
        persist::save(catalog.path(), &bytes).map_err(|error| AdminRefusal::Persist {
            detail: error.reason,
        })?;
        *catalog = rebuilt;
        Ok(())
    }

    /// Removes one record by `game_id`, bumping `revision` through the same
    /// tmp-and-rename.
    ///
    /// # Errors
    ///
    /// Returns [`AdminRefusal::Absent`] for a `game_id` the document does not hold
    /// — which bumps nothing — or a persistence refusal.
    pub fn remove(&self, game_id: &str) -> Result<(), AdminRefusal> {
        let mut catalog = self
            .catalog
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !catalog.holds(game_id) {
            return Err(AdminRefusal::Absent {
                detail: format!(
                    "the document holds no record with game_id `{}`",
                    escape_and_truncate(game_id)
                ),
            });
        }
        let mut rebuilt = catalog.clone();
        rebuilt.remove(game_id);
        let bytes = rebuilt.to_bytes();
        if bytes.len() > constants::MAX_CATALOG_DOCUMENT_OCTETS {
            return Err(AdminRefusal::Reject {
                status: 422,
                detail: "the catalog document after the delete is over the cap".to_string(),
            });
        }
        persist::save(catalog.path(), &bytes).map_err(|error| AdminRefusal::Persist {
            detail: error.reason,
        })?;
        *catalog = rebuilt;
        Ok(())
    }
}

/// The candidate document with one upsert folded in and `revision` bumped. The
/// buffer this returns is the exact buffer renamed into place, which is why the
/// cap is measured on it and nothing is written before that measurement.
fn apply_upsert(catalog: &Catalog, record: &Record) -> Catalog {
    let mut rebuilt = catalog.clone();
    rebuilt.upsert(record.clone());
    rebuilt
}
