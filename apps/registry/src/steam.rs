//! The steam store resolver (design 027, §The steam snapshot resolver and §The
//! store pull): the `027-C3` snapshot document and its loader, the store's paged
//! pull codec, the transport that carries the pull, and the refresh thread's
//! decision.
//!
//! Three disciplines hold this module together.
//!
//! **The document is a contract, not a cache.** A snapshot this instance publishes
//! is the one its loader must be able to read back, so every rule the pull applies
//! on the way in — ascending `appid`, one record per `appid`, a `013-B4`-legal
//! `name` with no control byte, a stamp inside the shipped `026-C7` plausibility
//! pair — is re-checked at load rather than trusted because the pull wrote it. The
//! octet bound is the metadata gate [`persist::read_capped`] again: an oversized
//! file is refused on its `stat`, never parsed.
//!
//! **The pull owns the store's messiness.** A game name that breaks the name bound
//! is control-stripped and clamped to the shipped `friendly_name` form here, at the
//! only seam that can, and its record *stays* in the snapshot: dropping one appid
//! on a bad title would hand a remote server a permanent denial of this
//! instance's supply. Nothing about a name is ever asked of the store — the request
//! lines are fixed (§The store pull), so a queried key, user, or slug can never
//! reach the authority (`027-L4`).
//!
//! **The key never leaves the assured path.** The Web API key is placed in one
//! query string, on one path, over a transport that has finished verifying the
//! pinned authority; no failure line this module produces carries the key, the
//! request URI, or the path.
//!
//! The module is deliberately free of the serving face: [`SteamStore`] takes where it
//! dials (a pinned authority in production, an address in a test), its timing, and its
//! transport as data, which is what lets the suite run the *production* transport and
//! the real codec against a loopback fake instead of a parallel test-only copy of the
//! logic.

use std::fmt;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use serde_json::Value;

use super::constants;
use super::persist;

/// `027-C3` pins the store at this host, and there is no operator-configurable
/// store URL anywhere in this framework (§Options' plaintext rejection). It is a
/// compile-time constant for exactly that reason.
pub const STORE_AUTHORITY: &str = "partner.steam-api.com";

/// The one request path the pull ever uses, with the world-list operation A3
/// documents and nothing else.
const STORE_PATH: &str = "/IStoreService/GetAppList/v1";

// §Budgets' store-resolver family is declared in [`constants`] with the crate's
// other budgets — §Budgets names that file as the family's one home — and re-exported
// here so the code that can breach them, and the library face `tests/steam.rs` pins,
// spell them the way §The store pull does.
pub use constants::{
    MAX_STEAM_PAGE_OCTETS, MAX_STEAM_SNAPSHOT_OCTETS, STEAM_CONNECT_TIMEOUT, STEAM_PAGE_ATTEMPTS,
    STEAM_PAGE_MAX_RESULTS, STEAM_PAGE_RETRY_SECONDS, STEAM_PULL_DEADLINE_SECONDS,
    STEAM_READ_TIMEOUT, STEAM_REFRESH_MAX_PAGES, STEAM_RETRY_SECONDS,
    STEAM_SNAPSHOT_MAX_AGE_SECONDS, STEAM_SNAPSHOT_REFRESH_SECONDS,
};

/// A refusal whose text is the whole report: what condition was met, never a
/// credential, a URI, or a path fragment the caller did not already have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(String);

impl Refusal {
    fn say(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Refusal {}

/// One `{ appid, name }` record of the snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotApp {
    /// The store's decimal application id — the identity the resolver key
    /// `steam/<appid>` is spelled from.
    pub appid: u64,
    /// The store's title as the pull normalized it: empty when the title was
    /// nothing but control characters or whitespace.
    pub name: String,
}

/// The published worldlist: a stamp plus the records in ascending `appid` order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamSnapshot {
    fetched_at: u64,
    apps: Vec<SnapshotApp>,
}

impl SteamSnapshot {
    /// The seam the suites publish a snapshot through: the records are taken in the
    /// order given, which the fixtures already spell ascending.
    #[must_use]
    pub fn for_test(fetched_at: u64, apps: &[(u64, &str)]) -> Self {
        Self {
            fetched_at,
            apps: apps
                .iter()
                .map(|(appid, name)| SnapshotApp {
                    appid: *appid,
                    name: (*name).to_string(),
                })
                .collect(),
        }
    }

    /// The wall clock read before the publishing pull's first page request.
    #[must_use]
    pub fn fetched_at(&self) -> u64 {
        self.fetched_at
    }

    /// The records, ascending by `appid`.
    #[must_use]
    pub fn apps(&self) -> &[SnapshotApp] {
        &self.apps
    }

    /// How many appids this snapshot certifies misses for.
    #[must_use]
    pub fn len(&self) -> usize {
        self.apps.len()
    }

    /// Whether the snapshot was published empty. A published empty worldlist is a
    /// refusal of the pull that produced it ([`SteamStore::pull`] refuses it by
    /// name), never a fact about the store, so this is only ever true of a
    /// hand-built fixture.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }

    /// The record for one appid, which is what a `steam/<appid>` lookup reads.
    #[must_use]
    pub fn app(&self, appid: u64) -> Option<&SnapshotApp> {
        self.apps
            .binary_search_by(|app| app.appid.cmp(&appid))
            .ok()
            .map(|index| &self.apps[index])
    }

    /// Whether the stamp is past `027-B2`'s age bound.
    ///
    /// Two seams read this, and they read it differently. [`load_snapshot`] does *not*:
    /// a stale-but-plausible document is still a well-formed one, and the loader's
    /// refusals are shape and plausibility only. [`startup_snapshot`] does — §The steam
    /// snapshot resolver's startup clause loads a snapshot that is present, loadable
    /// **and** inside this bound, and pulls in full on anything else, because a
    /// past-bound snapshot cannot certify a miss. Inside a running instance this is the
    /// serving policy: the lookup turns a now-stale snapshot's steam misses into the
    /// named `503` while corpus and issued keep answering.
    #[must_use]
    pub fn is_past_age_bound(&self, now: u64) -> bool {
        now.saturating_sub(self.fetched_at) > STEAM_SNAPSHOT_MAX_AGE_SECONDS
    }

    /// The document as it goes to disk: `027-C3`'s shape, in the order the loader
    /// insists on.
    #[must_use]
    pub fn to_document(&self) -> Value {
        Value::Object(serde_json::Map::from_iter([
            (
                "version".to_string(),
                Value::from(constants::DOCUMENT_VERSION),
            ),
            ("fetched_at".to_string(), Value::from(self.fetched_at)),
            (
                "apps".to_string(),
                Value::Array(
                    self.apps
                        .iter()
                        .map(|app| {
                            Value::Object(serde_json::Map::from_iter([
                                ("appid".to_string(), Value::from(app.appid)),
                                ("name".to_string(), Value::from(app.name.clone())),
                            ]))
                        })
                        .collect(),
                ),
            ),
        ]))
    }

    /// Publishes the snapshot through the shipped durable-write seam, so a crash
    /// mid-publish leaves the previous snapshot in place (`026-C7`'s rule, re-read
    /// by §The steam snapshot resolver).
    pub fn publish(&self, path: &Path) -> Result<(), Refusal> {
        // The compact spelling of `027-C3`'s shape. Pretty-printing the same records
        // is the same document about half again as large against the same
        // `MAX_STEAM_SNAPSHOT_OCTETS`, on every daily publish, on every metadata gate
        // over the file, and in every later parse; the runbook reads the file with a
        // JSON tool, not with an eye.
        let text = serde_json::to_vec(&self.to_document())
            .map_err(|_| Refusal::say("the snapshot does not serialise"))?;
        if text.len() > MAX_STEAM_SNAPSHOT_OCTETS {
            return Err(Refusal::say(format!(
                "the snapshot would be {} octets, over the {}-octet bound",
                text.len(),
                MAX_STEAM_SNAPSHOT_OCTETS
            )));
        }
        persist::save(path, &text).map_err(|error| Refusal::say(error.reason))
    }

    /// Assembles a published snapshot from the records one pull collected.
    ///
    /// A full pull (`loaded: None`) *replaces*: the result is exactly what the store
    /// returned. A delta *merges*: one linear two-way merge against the loaded
    /// vector, so a record the delta did not mention stays — A3's `if_modified_since`
    /// semantics carry changes and additions only, never removals.
    fn published(now: u64, collected: Vec<SnapshotApp>, loaded: Option<&Self>) -> Self {
        let changes = sort_and_dedup(collected);
        let apps = match loaded {
            None => changes,
            Some(loaded) => merge(loaded.apps(), &changes),
        };
        Self {
            fetched_at: now,
            apps,
        }
    }
}

/// Sorts ascending by `appid` and keeps the last of a repeated appid — the order
/// and uniqueness `027-C3` fixes, enforced on the pull's own output so a published
/// document always survives its own loader.
fn sort_and_dedup(records: Vec<SnapshotApp>) -> Vec<SnapshotApp> {
    let mut sorted = records;
    sorted.sort_by_key(|app| app.appid);
    let mut kept: Vec<SnapshotApp> = Vec::with_capacity(sorted.len());
    for record in sorted {
        match kept.last_mut() {
            Some(last) if last.appid == record.appid => *last = record,
            _ => kept.push(record),
        }
    }
    kept
}

/// The one linear two-way merge §The store pull fixes: `appid` order preserved, a
/// changed record carried by the store's new name, an untouched loaded record
/// kept as it was.
fn merge(loaded: &[SnapshotApp], changes: &[SnapshotApp]) -> Vec<SnapshotApp> {
    let mut merged: Vec<SnapshotApp> = Vec::with_capacity(loaded.len() + changes.len());
    let mut here = loaded.iter().peekable();
    let mut there = changes.iter().peekable();
    loop {
        match (here.peek(), there.peek()) {
            (Some(kept), Some(change)) => match kept.appid.cmp(&change.appid) {
                std::cmp::Ordering::Less => merged.push(here.next().expect("peeked").clone()),
                std::cmp::Ordering::Greater => merged.push(there.next().expect("peeked").clone()),
                std::cmp::Ordering::Equal => {
                    here.next();
                    merged.push(there.next().expect("peeked").clone());
                }
            },
            (Some(_), None) => merged.push(here.next().expect("peeked").clone()),
            (None, Some(_)) => merged.push(there.next().expect("peeked").clone()),
            (None, None) => return merged,
        }
    }
}

/// Reads and validates a snapshot document. Every condition §The steam snapshot
/// resolver names is a refusal of the whole file, named by the condition itself.
pub fn load_snapshot(path: &Path, now: u64) -> Result<SteamSnapshot, Refusal> {
    let bytes = match persist::read_capped(path, MAX_STEAM_SNAPSHOT_OCTETS) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            return Err(Refusal::say(format!(
                "there is no steam snapshot at {}",
                persist::echo_path(path)
            )));
        }
        Err(error) => {
            return Err(Refusal::say(format!(
                "the steam snapshot at {} could not be read: {}",
                persist::echo_path(path),
                error.reason
            )));
        }
    };
    let document: Value = serde_json::from_slice(&bytes).map_err(|_| {
        Refusal::say(format!(
            "the steam snapshot at {} is not a JSON object",
            persist::echo_path(path)
        ))
    })?;
    let fields = document
        .as_object()
        .ok_or_else(|| Refusal::say("the steam snapshot is not a JSON object"))?;

    match fields.get("version").and_then(Value::as_u64) {
        Some(constants::DOCUMENT_VERSION) => {}
        other => {
            return Err(Refusal::say(format!(
                "the steam snapshot's `version` {other:?} is not the version this registry reads"
            )));
        }
    }

    let fetched_at = fields
        .get("fetched_at")
        .and_then(Value::as_u64)
        .ok_or_else(|| Refusal::say("`fetched_at` is missing or is not a decimal integer"))?;
    let oldest_allowed = now.saturating_sub(constants::FETCHED_AT_PAST_TOLERANCE_SECONDS);
    let newest_allowed = now.saturating_add(constants::FETCHED_AT_FUTURE_TOLERANCE_SECONDS);
    if fetched_at < oldest_allowed || fetched_at > newest_allowed {
        return Err(Refusal::say(format!(
            "`fetched_at` {fetched_at} is outside the plausibility window this registry loads"
        )));
    }

    let records = fields
        .get("apps")
        .and_then(Value::as_array)
        .ok_or_else(|| Refusal::say("the steam snapshot's `apps` is missing or is not a list"))?;
    // An empty worldlist is not a shape this instance ever publishes: the pull that
    // collected nothing is refused by name, so a file with no records is a hand-edited
    // or truncated one. Loading it would hand the policy a snapshot entitled to
    // certify `200 []` for *every* appid in the store.
    if records.is_empty() {
        return Err(Refusal::say(
            "the steam snapshot's `apps` is empty: no pull publishes an empty worldlist, so \
             this file is not a snapshot this registry can answer from",
        ));
    }
    let mut apps: Vec<SnapshotApp> = Vec::with_capacity(records.len());
    let mut previous: Option<u64> = None;
    for record in records {
        let fields = record
            .as_object()
            .ok_or_else(|| Refusal::say("a steam snapshot record is not an object"))?;
        let appid = fields.get("appid").and_then(Value::as_u64).ok_or_else(|| {
            Refusal::say("a steam snapshot record's `appid` is not a decimal integer")
        })?;
        if let Some(seen) = previous {
            if appid == seen {
                return Err(Refusal::say(format!("`appid` {appid} appears twice")));
            }
            if appid < seen {
                return Err(Refusal::say(format!(
                    "`appid` {appid} is out of order after {seen}"
                )));
            }
        }
        previous = Some(appid);
        let name = fields
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("`appid` {appid} carries no `name`"))
            .map_err(Refusal::say)?;
        if name.len() > bindforge_core::MAX_NATIVE_NAME_OCTETS {
            return Err(Refusal::say(format!(
                "`name` for `appid` {appid} is {} octets, over the name budget",
                name.len()
            )));
        }
        if name.chars().any(char::is_control) {
            return Err(Refusal::say(format!(
                "`name` for `appid` {appid} carries a control byte"
            )));
        }
        apps.push(SnapshotApp {
            appid,
            name: name.to_string(),
        });
    }

    Ok(SteamSnapshot { fetched_at, apps })
}

/// What the refresh thread decided to do next, from the pull it just finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullOutcome {
    /// A snapshot was published, so the next pull is the next scheduled delta.
    Published,
    /// The pull failed; the old snapshot stays published and the pull is retried
    /// on the shorter gap.
    Failed,
}

/// The refresh thread's whole decision, as a pure function of the clock (§The
/// store pull). The thread sleeps on this and nothing else, so no test of the
/// cadence has to wait for it.
#[must_use]
pub fn next_refresh_seconds(now: u64, last_pull_at: u64, outcome: PullOutcome) -> u64 {
    let gap = match outcome {
        PullOutcome::Published => STEAM_SNAPSHOT_REFRESH_SECONDS,
        PullOutcome::Failed => STEAM_RETRY_SECONDS,
    };
    last_pull_at.saturating_add(gap).saturating_sub(now)
}

/// Startup's load-or-pull decision (§The steam snapshot resolver), stated once where
/// the suite can drive it.
///
/// `027-C3`'s clause has three arms, and the middle one is the whole point: the
/// snapshot is loaded when it is present, loadable, **and** inside
/// [`STEAM_SNAPSHOT_MAX_AGE_SECONDS`]; anything else — absent, unloadable, or loadable
/// but past bound — pulls in full and publishes before the process binds. A past-bound
/// snapshot is not the load's case because it cannot certify a miss: an instance booted
/// over one answers every `steam/<appid>` lookup with the named `503` until something
/// refreshes it, which §Compatibility matrix's client row reads as a failed scan. The
/// operator's alternative to a startup pull is the posture that neither reads nor pulls
/// the file, `--no-steam`.
///
/// This lives on the library face rather than only in the binary's startup seam for the
/// same reason the pull does: a test can point a [`SteamStore`] at a loopback fake and
/// watch which arm the decision takes, which is the only way `027-C3`'s clause and Test
/// strategy #17's no-outbound rule can both be pinned.
///
/// # Errors
///
/// The refusal line, already display-safe (`027-L4` names the condition and never the
/// URL or the key), missing only the `error: ` the binary's own lines carry: either the
/// publish failed, or there is no usable snapshot and the pull that would have
/// replaced it says why not.
pub fn startup_snapshot(
    store: &SteamStore,
    path: &Path,
    now: u64,
) -> Result<SteamSnapshot, String> {
    // Why the pull is happening, for the line a failed pull becomes: the cold start's
    // loader refusal, the repair case's refusal, or the stale case's age.
    let reason = match load_snapshot(path, now) {
        Ok(snapshot) if !snapshot.is_past_age_bound(now) => return Ok(snapshot),
        Ok(_) => format!(
            "`{}` is past 027-B2's age bound, so it cannot certify that an appid is not in \
             the store; a full pull replaces it",
            persist::echo_path(path)
        ),
        // Absent is the cold start; present-and-refused is the repair case, and its
        // refusal line is what the pull's failure becomes.
        Err(error) => error.to_string(),
    };
    match store.full(now) {
        Ok(snapshot) => {
            snapshot.publish(path).map_err(|error| {
                format!("the steam pull: {}", persist::echo_safe(&error.to_string()))
            })?;
            Ok(snapshot)
        }
        // One line, naming the condition and never the URL or the key (`027-L4`).
        Err(pull) => Err(format!(
            "no usable steam snapshot: {}; the pull that would replace it: {}",
            persist::echo_safe(&reason),
            persist::echo_safe(&pull.to_string())
        )),
    }
}

/// §The store pull's transport seam: which pipe a page travels on.
///
/// The verified arm is the only one any ordinary construction path installs;
/// [`StoreTransport::Cleartext`] exists for the loopback fake the codec suite answers
/// with, and the only way to hold one is the test constructor that is named for it.
/// That is the CWE-319 posture §Options pins: the safe transport is the default, and
/// a `SteamStore` a production path builds always carries the `ClientConfig` that
/// verifies the pinned authority.
#[derive(Clone)]
enum StoreTransport {
    /// Plaintext, for the loopback fake the suite answers with. The *codec* is the
    /// production one; only the link is the test's.
    Cleartext,
    /// The production transport: rustls against the crate's own trust anchors,
    /// verifying the name it was handed — which is the pinned authority in
    /// production, and a separate parameter from the address in a test.
    Verified(Arc<ClientConfig>),
}

/// Where a page is dialed. Production holds only the *name*, and resolves it when it
/// is about to dial; the suite holds an address. Resolving in `connect` is what keeps
/// a run that loads a loadable-and-fresh snapshot from consulting the resolver at all
/// (Test strategy #17's no-outbound rule), and it keeps the whole answer: every
/// address the name yields is tried for a page, the way
/// `TcpStream::connect((host, port))` walks it, so a host whose resolver answers an
/// unreachable IPv6 address first still reaches the store over its IPv4 one.
#[derive(Clone)]
enum Endpoint {
    /// The compile-time-pinned authority and its port. There is no operator seam to
    /// another store, and `resolve` is only ever asked of this name.
    Pinned(&'static str, u16),
    /// A test's address, dialed as given and never resolved.
    Address(SocketAddr),
}

/// The store pull. One of these is cheap to build and holds no connection: every
/// page is its own connection, closed when the page is answered (A23's discipline
/// applied outward). Building one resolves nothing and dials nothing.
#[derive(Clone)]
pub struct SteamStore {
    endpoint: Endpoint,
    key: String,
    verify_name: String,
    transport: StoreTransport,
    page_backoff: Duration,
    pull_deadline: Duration,
}

/// The trust anchors §Options names: the crate's own roots, so a hosted instance
/// needs no system trust store.
#[must_use]
fn production_client_config() -> Arc<ClientConfig> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    )
}

impl SteamStore {
    /// The production store: the compile-time-pinned authority, the verified
    /// transport, and the key from `BINDFORGE_STEAM_WEB_API_KEY`. This is the
    /// constructor a hosted instance builds, and it is the one whose transport is not
    /// a choice — building a store cannot accidentally build a plaintext one.
    #[must_use]
    pub fn production(key: &str) -> Self {
        Self {
            endpoint: Endpoint::Pinned(STORE_AUTHORITY, 443),
            key: key.to_string(),
            verify_name: STORE_AUTHORITY.to_string(),
            transport: StoreTransport::Verified(production_client_config()),
            page_backoff: Duration::from_secs(STEAM_PAGE_RETRY_SECONDS),
            pull_deadline: Duration::from_secs(STEAM_PULL_DEADLINE_SECONDS),
        }
    }

    /// The test seam §The store pull's wording assumes: the production codec over a
    /// plaintext loopback link, dialed at `address` and never resolved. The name
    /// verified against it stays [`STORE_AUTHORITY`] until
    /// [`SteamStore::with_production_transport`] is called, which is how a test dials
    /// the fake with the assurance posture intact. Production uses
    /// [`SteamStore::production`].
    #[must_use]
    pub fn for_test(address: SocketAddr, key: &str) -> Self {
        Self {
            endpoint: Endpoint::Address(address),
            key: key.to_string(),
            verify_name: STORE_AUTHORITY.to_string(),
            transport: StoreTransport::Cleartext,
            page_backoff: Duration::from_secs(STEAM_PAGE_RETRY_SECONDS),
            pull_deadline: Duration::from_secs(STEAM_PULL_DEADLINE_SECONDS),
        }
    }

    /// Replaces the page backoff and the whole pull's deadline. The suite's injected
    /// timing is what keeps the retry and deadline clauses observable in seconds.
    #[must_use]
    pub fn with_timing(mut self, page_backoff: Duration, pull_deadline: Duration) -> Self {
        self.page_backoff = page_backoff;
        self.pull_deadline = pull_deadline;
        self
    }

    /// Installs the production transport, verifying `verify_name`. The address is
    /// untouched, which is the point: a test dials the fake while the client still
    /// verifies the authority.
    #[must_use]
    pub fn with_production_transport(mut self, verify_name: &str) -> Self {
        self.verify_name = verify_name.to_string();
        self.transport = StoreTransport::Verified(production_client_config());
        self
    }

    /// Pulls the whole worldlist: the result replaces the published snapshot.
    pub fn full(&self, now: u64) -> Result<SteamSnapshot, Refusal> {
        self.pull(now, None)
    }

    /// Pulls what changed since `loaded`'s own stamp and merges it in. The delta
    /// cursor is the loaded snapshot's `fetched_at` and nothing else (§The store
    /// pull).
    pub fn delta(&self, now: u64, loaded: &SteamSnapshot) -> Result<SteamSnapshot, Refusal> {
        self.pull(now, Some(loaded))
    }

    /// Walks the pages. `now` is the clock the caller read *before* the pull began,
    /// and it is what the published stamp becomes: a stamp taken at publish time
    /// would sit above appids the pull never saw, and the next delta would ask for
    /// changes it would then never receive.
    fn pull(&self, now: u64, loaded: Option<&SteamSnapshot>) -> Result<SteamSnapshot, Refusal> {
        let deadline = Instant::now() + self.pull_deadline;
        let since = loaded.map(SteamSnapshot::fetched_at);
        let mut cursor: Option<u64> = None;
        let mut collected: Vec<SnapshotApp> = Vec::new();
        for _ in 0..STEAM_REFRESH_MAX_PAGES {
            let target = self.request_target(cursor, since);
            let page = self.page(&target, deadline)?;
            let continued = cursor.is_some();
            let yielded = page.records.len();
            collected.extend(page.records);
            if !page.have_more_results {
                // An empty *full* pull is a refusal, never a publish (§The store pull's
                // "nothing is dropped", §Options' rejected partial serving): a store
                // that answers 200 with no records — a key without the
                // `IStoreService` grant, a gateway's empty envelope — would otherwise
                // replace a good snapshot and have the instance certify `200 []` for
                // every game appid in the store for `027-B2`'s whole age bound.
                if loaded.is_none() && collected.is_empty() {
                    return Err(Refusal::say(
                        "the store's worldlist answered with no records at all: an empty \
                         snapshot is a refusal of this pull, never a fact about the store",
                    ));
                }
                // Same reasoning one page in: a listing that ends on a page holding
                // nothing after a continuation said more than it delivered, so the
                // worldlist this pull holds is not the one it was promised. A *delta*'s
                // first page holding nothing is the ordinary no-change case and
                // publishes its merge.
                if continued && yielded == 0 {
                    return Err(Refusal::say(
                        "the store's listing continued onto a page that carried no records \
                         and then claimed to be finished: the worldlist is incomplete, so \
                         nothing publishes",
                    ));
                }
                return Ok(SteamSnapshot::published(now, collected, loaded));
            }
            cursor = collected.iter().map(|app| app.appid).max();
        }
        Err(Refusal::say(format!(
            "the store listing still had more pages after the {}-page budget",
            STEAM_REFRESH_MAX_PAGES
        )))
    }

    /// The fixed request lines §The store pull allows, and nothing else. The key is
    /// the only value here that came from anywhere but this function.
    fn request_target(&self, cursor: Option<u64>, since: Option<u64>) -> String {
        let mut target = format!(
            "{STORE_PATH}?key={}&max_results={}",
            form_encode(&self.key),
            STEAM_PAGE_MAX_RESULTS
        );
        if let Some(appid) = cursor {
            target.push_str(&format!("&last_appid={appid}"));
        }
        if let Some(since) = since {
            target.push_str(&format!("&if_modified_since={since}"));
        }
        target
    }

    /// One page, attempted [`STEAM_PAGE_ATTEMPTS`] times with the page
    /// backoff between attempts, and abandoned for the deadline when the clock says
    /// so. A permanent condition — a page this client cannot read at all — is not
    /// retried: the same request earns the same answer.
    fn page(&self, target: &str, deadline: Instant) -> Result<Page, Refusal> {
        let mut last: Option<Refusal> = None;
        for attempt in 0..STEAM_PAGE_ATTEMPTS {
            if attempt > 0 {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(deadline_refusal());
                }
                thread::sleep(self.page_backoff.min(remaining));
            }
            if Instant::now() >= deadline {
                return Err(deadline_refusal());
            }
            match self.attempt_page(target, deadline) {
                Ok(page) => return Ok(page),
                Err(PageFailure::Final(refusal)) => return Err(refusal),
                Err(PageFailure::Transient(refusal)) => last = Some(refusal),
            }
        }
        Err(last.unwrap_or_else(|| Refusal::say("the store page failed")))
    }

    /// One connection, one request, one page.
    fn attempt_page(&self, target: &str, deadline: Instant) -> Result<Page, PageFailure> {
        let mut wire = self.connect(deadline)?;
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {STORE_AUTHORITY}\r\nConnection: close\r\n\r\n"
        );
        wire.write_all(request.as_bytes()).map_err(|error| {
            transient(format!(
                "the store request could not be sent: {}",
                io_reason(&error)
            ))
        })?;

        let head_block = read_header_block(&mut wire, deadline)?;
        let head_end = header_end(&head_block).unwrap_or(head_block.len());
        let (status, headers, declared) = parse_head(&head_block[..head_end])?;
        // §The store pull names `non-200` the refusal, not `non-2xx`: a `204`, a `206`,
        // or any other success shape is an answer this client has no reading for, and a
        // page this client cannot read at all is not worth its retries.
        if status != 200 {
            return Err(transient(format!(
                "the store answered {status} where only 200 is an answer"
            )));
        }
        if headers.iter().any(|name| name == "transfer-encoding") {
            return Err(permanent(
                "the store answered with a `Transfer-Encoding` framing this client does not read",
            ));
        }
        // A read window that reached the blank line may have carried body octets
        // with it; they are the body's first bytes, not something to discard.
        let body = read_body(
            &mut wire,
            declared,
            head_block[head_end..].to_vec(),
            deadline,
        )?;
        let page = decode_page(&body)?;
        Ok(page)
    }

    /// Dials the store and hands back the pipe the page travels on: the plain socket,
    /// or the TLS session that has to finish verifying before it will carry a byte of
    /// the request.
    ///
    /// Production resolves the pinned authority *here*: the only moment a run needs
    /// the resolver is the moment it is about to dial, so a run that loaded a
    /// loadable-and-fresh snapshot never consults it (Test strategy #17). The whole
    /// answer is kept and worked through in order, the way
    /// `TcpStream::connect((host, port))` walks it: an unreachable first address is
    /// one failed dial, not a failed pull.
    fn connect(&self, deadline: Instant) -> Result<Wire, PageFailure> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(PageFailure::Final(deadline_refusal()));
        }
        let timeout = STEAM_READ_TIMEOUT.min(remaining);
        let mut dialed: Option<TcpStream> = None;
        let mut last: Option<String> = None;
        for address in self.dial_addresses()? {
            match TcpStream::connect_timeout(&address, timeout.min(STEAM_CONNECT_TIMEOUT)) {
                Ok(stream) => {
                    stream
                        .set_read_timeout(Some(timeout))
                        .and_then(|()| stream.set_write_timeout(Some(timeout)))
                        .map_err(|error| {
                            transient(format!(
                                "the store connection could not be bounded: {}",
                                error.kind()
                            ))
                        })?;
                    dialed = Some(stream);
                    break;
                }
                Err(error) => last = Some(error.kind().to_string()),
            }
        }
        let stream = dialed.ok_or_else(|| {
            transient(format!(
                "the store could not be reached: {}",
                last.unwrap_or_else(|| "the authority resolved to no address".to_string())
            ))
        })?;
        match &self.transport {
            StoreTransport::Cleartext => Ok(Wire::Plain(stream)),
            StoreTransport::Verified(config) => {
                let name = ServerName::try_from(self.verify_name.clone()).map_err(|_| {
                    permanent("the store authority is not a name this transport can verify")
                })?;
                let session = ClientConnection::new(Arc::clone(config), name).map_err(|error| {
                    transient(format!(
                        "the store's TLS session could not be opened: {error}"
                    ))
                })?;
                Ok(Wire::Verified(Box::new(StreamOwned::new(session, stream))))
            }
        }
    }

    /// The addresses one page may be dialed at, in resolver order. Asking the
    /// resolver is the pull's own act and never a startup one; the pinned authority is
    /// the only name it is ever asked for.
    fn dial_addresses(&self) -> Result<Vec<SocketAddr>, PageFailure> {
        match &self.endpoint {
            Endpoint::Address(address) => Ok(vec![*address]),
            Endpoint::Pinned(authority, port) => {
                let resolved = (*authority, *port)
                    .to_socket_addrs()
                    .map(|addresses| addresses.collect::<Vec<_>>())
                    .map_err(|error| {
                        transient(format!(
                            "the steam store authority could not be resolved: {}",
                            error.kind()
                        ))
                    })?;
                if resolved.is_empty() {
                    return Err(permanent(
                        "the steam store authority resolved to no address",
                    ));
                }
                Ok(resolved)
            }
        }
    }
}

/// Why one page failed, and whether trying again could mean anything.
enum PageFailure {
    /// Worth another attempt: a socket condition, a `5xx`, a rate limit.
    Transient(Refusal),
    /// Not worth another attempt: the answer itself is one this client cannot use.
    Final(Refusal),
}

impl From<Refusal> for PageFailure {
    fn from(refusal: Refusal) -> Self {
        Self::Final(refusal)
    }
}

fn transient(reason: impl Into<String>) -> PageFailure {
    PageFailure::Transient(Refusal::say(reason))
}

/// Why a wire leg failed, as one bounded reason. A plain OS failure keeps its
/// bare kind, exactly as before. `InvalidData` is rustls' code on this seam —
/// no OS error yields it through `StreamOwned` — and its carried `rustls::Error`
/// Display is the identity Test strategy #6's chain-vs-name pin needs (e.g.
/// `invalid peer certificate: UnknownIssuer`). The Display of these errors
/// carries no key, no URI, and no page content: §The store pull's refusal
/// discipline still holds, as the suite's leak assertions re-check.
fn io_reason(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::InvalidData => format!("{}: {error}", error.kind()),
        kind => kind.to_string(),
    }
}

fn permanent(reason: impl Into<String>) -> PageFailure {
    PageFailure::Final(Refusal::say(reason))
}

fn deadline_refusal() -> Refusal {
    Refusal::say("steam pull deadline")
}

/// One page as the store framed it: the records and A17's continuation flag.
struct Page {
    records: Vec<SnapshotApp>,
    have_more_results: bool,
}

/// The pipe a page travels on. An enum, not a `dyn` object, so the plain and the
/// verified branch share the reading code without a coercion puzzle.
enum Wire {
    Plain(TcpStream),
    /// Boxed: a finished TLS session is a kilobyte of buffers, and this pipe is
    /// created per page, so the plain branch should not pay for it.
    Verified(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Wire {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buffer),
            Self::Verified(stream) => stream.read(buffer),
        }
    }

    fn write_all(&mut self, buffer: &[u8]) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.write_all(buffer),
            Self::Verified(stream) => stream.write_all(buffer),
        }
    }
}

/// Reads up to and including the header block's blank line, inside the response
/// budget that bounds every answer this registry reads (A1), and inside the pull's own
/// deadline.
fn read_header_block(wire: &mut Wire, deadline: Instant) -> Result<Vec<u8>, PageFailure> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut window = [0_u8; 1024];
    loop {
        if header_end(&bytes).is_some() {
            return Ok(bytes);
        }
        within_pull_deadline(deadline)?;
        if bytes.len() > constants::MAX_RESPONSE_OCTETS {
            return Err(permanent(
                "the store's header block is over the response budget",
            ));
        }
        match wire.read(&mut window) {
            Ok(0) => {
                return Err(transient("the store closed the connection mid-response"));
            }
            Ok(read) => bytes.extend_from_slice(&window[..read]),
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => {
                return Err(transient(format!(
                    "the store's answer could not be read: {}",
                    io_reason(&error)
                )));
            }
        }
    }
}

fn header_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

/// The status code, the header names, and the declared body length — the three
/// things §The store pull's response rules read before touching a body octet.
fn parse_head(head: &[u8]) -> Result<(u16, Vec<String>, Option<u64>), PageFailure> {
    let text = String::from_utf8_lossy(head).into_owned();
    let mut lines = text.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| permanent("the store's status line is empty"))?;
    let code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|field| field.parse().ok())
        .ok_or_else(|| permanent("the store's status line is not one this client reads"))?;
    let mut names: Vec<String> = Vec::new();
    let mut declared: Option<u64> = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if name == "content-length" {
            declared = value.trim().parse::<u64>().ok();
        }
        names.push(name);
    }
    Ok((code, names, declared))
}

/// Reads the body against [`MAX_STEAM_PAGE_OCTETS`], inside the pull's own deadline.
/// A declared length
/// over that bound is refused before a single body octet is taken, which is what
/// makes the page bound a decision rather than an exhaustion.
fn read_body(
    wire: &mut Wire,
    declared: Option<u64>,
    mut body: Vec<u8>,
    deadline: Instant,
) -> Result<Vec<u8>, PageFailure> {
    let bound = u64::try_from(MAX_STEAM_PAGE_OCTETS).unwrap_or(u64::MAX);
    match declared {
        Some(length) if length > bound => Err(permanent(format!(
            "the store page declares {length} octets, over the {}-octet page read budget",
            MAX_STEAM_PAGE_OCTETS
        ))),
        Some(length) => {
            while (body.len() as u64) < length {
                match fill(wire, &mut body, deadline)? {
                    Progress::Closed => {
                        return Err(transient("the store closed the connection mid-body"));
                    }
                    Progress::Filled => {}
                }
            }
            body.truncate(usize::try_from(length).unwrap_or(body.len()));
            Ok(body)
        }
        None => loop {
            if body.len() as u64 > bound {
                return Err(permanent(format!(
                    "the store page ran past the {}-octet page read budget",
                    MAX_STEAM_PAGE_OCTETS
                )));
            }
            match fill(wire, &mut body, deadline)? {
                Progress::Closed if body.is_empty() => {
                    return Err(transient("the store closed the connection mid-body"));
                }
                Progress::Closed => return Ok(body),
                Progress::Filled => {}
            }
        },
    }
}

/// The named `steam pull deadline`, paid at every read and not only between page
/// attempts. §The store pull promises it as the guarantee against a slow store; a peer
/// that holds the connection open and dribbles a page never trips a socket timeout, so
/// a deadline checked only between attempts would let one page run for hours —
/// pre-bind, that is the process's bind held.
fn within_pull_deadline(deadline: Instant) -> Result<(), PageFailure> {
    if Instant::now() >= deadline {
        return Err(PageFailure::Final(deadline_refusal()));
    }
    Ok(())
}

/// How one read-into-a-buffer call went.
enum Progress {
    /// The peer closed while the buffer still had room.
    Closed,
    /// The window was consumed; the buffer grew by whatever it took.
    Filled,
}

/// Reads one window into `sink`, keeping everything the store has already sent.
/// Returns [`Progress::Closed`] only for a peer-driven close, which the caller
/// reads as end-of-body or as a truncated one.
///
/// The header block's tail is handed in by the caller: a single read window can
/// carry the blank line *and* the first body octets, and dropping them would leave
/// a body that never parses.
fn fill(wire: &mut Wire, sink: &mut Vec<u8>, deadline: Instant) -> Result<Progress, PageFailure> {
    within_pull_deadline(deadline)?;
    let mut window = [0_u8; 8_192];
    match wire.read(&mut window) {
        Ok(0) => Ok(Progress::Closed),
        Ok(read) => {
            sink.extend_from_slice(&window[..read]);
            Ok(Progress::Filled)
        }
        Err(error) if error.kind() == ErrorKind::Interrupted => Ok(Progress::Filled),
        Err(error) => Err(transient(format!(
            "the store's page could not be read: {}",
            io_reason(&error)
        ))),
    }
}

/// Decodes the A17 envelope. Anything that is not that envelope — no `apps` list,
/// a record whose `appid` is not a decimal integer — fails the page permanently:
/// the shape is not something a retry changes.
fn decode_page(body: &[u8]) -> Result<Page, PageFailure> {
    let envelope: Value = serde_json::from_slice(body)
        .map_err(|_| permanent("the store's answer is not the envelope this registry reads"))?;
    let records = envelope
        .get("apps")
        .and_then(Value::as_array)
        .ok_or_else(|| permanent("the store's answer carries no `apps` list"))?;
    let have_more_results = envelope
        .get("have_more_results")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut apps: Vec<SnapshotApp> = Vec::with_capacity(records.len());
    for record in records {
        let fields = record
            .as_object()
            .ok_or_else(|| permanent("a store record is not an object"))?;
        let appid = fields
            .get("appid")
            .and_then(Value::as_u64)
            .ok_or_else(|| permanent("a store record's `appid` is not a decimal integer"))?;
        // A record whose `name` is missing or not a string takes the posture §The store
        // pull already fixes for a title that normalizes to nothing: the label is empty,
        // the record and its appid stay. It is a third spelling of the same case rather
        // than a fourth rule — the alternative, failing the page over a store field we
        // did not ask for, is the remote, permanent denial of supply the same section
        // refuses, and the identity does not depend on the label (mint step 3's
        // `steam-<appid>`).
        let name = fields
            .get("name")
            .and_then(Value::as_str)
            .map(normalize_store_name)
            .unwrap_or_default();
        apps.push(SnapshotApp { appid, name });
    }
    Ok(Page {
        records: apps,
        have_more_results,
    })
}

/// §The store pull's name rule, owned here because no other seam can apply it: the
/// control bytes go first, under our own `char::is_control()` filter — the shipped
/// rule drops no other control byte, so crediting it with the strip would have this
/// instance publish a document its own loader refuses — and then the shipped
/// [`discovery::game::friendly_name`] does what A7 records it as doing: trim, collapse
/// interior whitespace runs to one space, and clamp to the longest
/// `MAX_NATIVE_NAME_OCTETS` prefix that ends on a character boundary. A7 and Rollout 3
/// name *that call*, so it is called rather than reconstructed here. A title that is
/// nothing but control characters or whitespace normalizes to empty; its record stays
/// in the snapshot either way.
fn normalize_store_name(raw: &str) -> String {
    let stripped: String = raw
        .chars()
        .filter(|character| !character.is_control())
        .collect();
    discovery::game::friendly_name(&stripped)
}

/// Percent-encodes one query parameter in the form the store documents. The
/// unreserved set is RFC 3986's, so a key made of the characters Steam issues
/// passes through byte-for-byte and nothing else can smuggle a delimiter.
fn form_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(*byte));
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}
