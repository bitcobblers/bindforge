//! The registry's private budgets (design 026, Chosen design: §Budgets).
//!
//! These three budgets — `026-B1` (cache entries), `026-B2` (cache octets),
//! `026-B3` (upstream TTL) — plus the request-side budgets §Budgets adds for the
//! serving and admin faces live **here and nowhere else**: they are deliberately
//! not members of core's constant family and get no `schemas/README.md` bullet
//! (open question 4). The precedent is the shipped client's own private
//! `STORE_READ_BUDGET_OCTETS` (A22): a bound that only one crate can breach is
//! declared in that crate.

use std::time::Duration;

/// The whole response budget, applied in **both** directions: the shipped client's
/// reader (A1) refuses a header block or body larger than it, so this instance's
/// own `GET /v1/candidates` answer is built inside it as well (`026-C1`'s
/// byte-identical face), and the upstream client refuses anything larger on the
/// way in.
pub const MAX_RESPONSE_OCTETS: usize = 65_536;

/// The inbound header block, including the request line, capped at the same figure
/// the client caps its own outbound header block at (`026-C2`/`026-C4`).
pub const MAX_REQUEST_HEADER_OCTETS: usize = 65_536;

/// The document cap, applied at both ends of a catalog document's life: the
/// metadata-gated read of `--corpus`/the local-entries file, and the exact
/// post-`revision`-bump buffer an admin write renames into place (§Budgets).
pub const MAX_CATALOG_DOCUMENT_OCTETS: usize = 16_777_216;

/// The request body budget *is* the catalog document cap, which is why no
/// per-record `source_keys` bound is needed: any record a loadable corpus could
/// legally hold also fits an upsert.
pub const MAX_REQUEST_BODY_OCTETS: usize = MAX_CATALOG_DOCUMENT_OCTETS;

/// `MAX_CANDIDATES` (A1): the most candidates one answer may carry, in either
/// direction. Truncation is the stable first `MAX_CANDIDATES` in `game_id` order.
/// The outbound seam enforces it three times over: the corpus index truncates the
/// postings, the cache write seam caps a staged entry's list, and the cache load
/// refuses a stored entry over the cap like every other load violation.
pub const MAX_CANDIDATES: usize = 256;

/// How many octets of a refused admin request's declared body are discarded before
/// the connection closes. An admin request that the header block alone already
/// condemns (a disabled face, a wrong credential, a method the route does not
/// carry) is refused *before* its body is read, so [`MAX_REQUEST_BODY_OCTETS`] is
/// never an unauthenticated peer's memory allowance. The bounded discard keeps the
/// answer we have already written readable by a peer that is still sending rather
/// than resetting it; a peer declaring more than this gets the connection's abrupt
/// end it was refused for.
pub const UNREAD_BODY_DRAIN_OCTETS: usize = 65_536;

/// The read timeout that bounds that discard. A peer that stops sending a body it
/// declared holds the admitted slot for this long and no longer.
pub const UNREAD_BODY_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

// `026-B1`/`026-B2`/`026-B3` — the cache entry cap, the cache document's octet cap,
// and the upstream TTL — retired with the caching mode ADR-027 deleted, with the two
// documents and the `--refresh-interval` flag that read them.

/// `026-C7`: `fetched_at` is untrusted input, so a load refuses a stamp older than
/// this many seconds (§Budgets).
pub const FETCHED_AT_PAST_TOLERANCE_SECONDS: u64 = 31_536_000;

/// `026-C7`: … or further in the future than this (§Budgets). A stamp ahead of the
/// wall clock still loads; it is the TTL arithmetic that reads it as expired.
pub const FETCHED_AT_FUTURE_TOLERANCE_SECONDS: u64 = 86_400;

/// One thread per admitted connection, which answers exactly one request before
/// the connection closes (A23); over-cap is closed at accept with no status line.
pub const MAX_CONCURRENT_CONNECTIONS: usize = 64;

/// The version `026-C3` and `026-C7` both spell as the bare integer `1`.
pub const DOCUMENT_VERSION: u64 = 1;

// The upstream client's connect and read timeouts (A1's mirrored pair) retired with
// `upstream.rs`. The store pull's own socket timeouts are `steam`'s.

/// How long an admitted connection may take to send its one request. Generous on
/// purpose: an idle admitted connection holds its slot, which is what makes the
/// `MAX_CONCURRENT_CONNECTIONS` ceiling observable, so this timeout only bounds
/// low-and-slow input, it does not police idle peers.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(60);

/// How long the answer may take to leave. Pairs with [`REQUEST_READ_TIMEOUT`] to
/// bound both directions of a connection's life (§Risks: low-and-slow).
pub const RESPONSE_WRITE_TIMEOUT: Duration = Duration::from_secs(15);

/// `027-B3` — the issued document's record cap. At the cap a *novel* key is the
/// named `503` `mint cap reached` (an identity that cannot be persisted is not an
/// identity), while a key the document already holds answers from it forever,
/// because §The hosted lookup policy's step 2 precedes this check (§Budgets).
pub const MAX_ISSUED_ENTRIES: usize = 200_000;

/// `027-B3`'s octet companion: the issued document's own cap, larger than the
/// corpus's because 200,000 records at the ~150-octet typical size is ~30 MiB.
/// Enforced twice like every other document budget: metadata-gated at load, and
/// against the exact buffer a mint renames into place. A mint that breaches it is
/// refused the same way a mint that breaches [`MAX_ISSUED_ENTRIES`] is.
pub const MAX_ISSUED_DOCUMENT_OCTETS: usize = 67_108_864;

/// `027-L2`'s memory: the retired-ids document's octet cap, ≈250k ids at the
/// 128-octet id bound. A delete whose retired-list write would breach it is the
/// named refusal `retired id list full` and mutates nothing: failing open there
/// re-opens a retired id, which is `027-L2` broken by arithmetic.
pub const MAX_RETIRED_IDS_OCTETS: usize = 33_554_432;

// ADR-027's store-resolver family (§Budgets) is declared here with the rest of the
// crate's budgets: §Budgets names this file as the family's one home, "kept in the
// one file". `steam.rs` re-exports the names for the pull seam's readers, so the
// code that can breach them spells them the way §The store pull does.

/// `027-B1` — the snapshot document's octet cap, measured on the file's metadata
/// before a single octet is read, as the catalog documents' cap is, and paid again
/// against the exact buffer a publish renames.
pub const MAX_STEAM_SNAPSHOT_OCTETS: usize = 33_554_432;

/// The one page's read *budget*: its declared length is refused against it before
/// the body is drained. Not a worst case — a store's over-bound title makes a
/// page's true size unbounded, and the over-cap page is the residual §Risks
/// carries. It sits above the 50k-records-at-~240-octets estimate A17's shape
/// returns at the pinned page size.
pub const MAX_STEAM_PAGE_OCTETS: usize = 12_582_912;

/// The record count pinned explicitly on *every* page request. The endpoint's
/// documented default is a tenth of this, and answering at the default would trip
/// `027-B4`'s page budget against a catalog in A5's band (open Q5).
pub const STEAM_PAGE_MAX_RESULTS: usize = 50_000;

/// `027-B4` — the most pages one pull may ask for. A listing still promising more
/// at that point fails the pull: a partial worldlist is never published.
pub const STEAM_REFRESH_MAX_PAGES: usize = 12;

/// How many times one page is attempted before the pull gives up.
pub const STEAM_PAGE_ATTEMPTS: usize = 3;

/// The gap between those attempts. This is *not* [`STEAM_RETRY_SECONDS`]: one retry
/// constant for the page loop and the refresh thread is the collision §Revision log
/// r1→r2 records.
pub const STEAM_PAGE_RETRY_SECONDS: u64 = 30;

/// The whole pull's own deadline. The value is the page grid this family
/// authorizes — `12 × (3 × (2 + 30) + 2 × 30) = 1_872` — rounded up, so a stalled
/// pull is cut off without `027-B4`'s page budget being overridden first. It bounds
/// the reading of a page too, not only the gaps between attempts: a peer that
/// dribbles a body never trips a socket timeout, and the named `steam pull deadline`
/// is the refusal that family owes it.
pub const STEAM_PULL_DEADLINE_SECONDS: u64 = 1_920;

/// The pull's connect timeout, mirroring the shipped client's (A1).
pub const STEAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// The read timeout one page's socket carries.
pub const STEAM_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// `027-B2` — how often the refresh thread pulls a delta.
pub const STEAM_SNAPSHOT_REFRESH_SECONDS: u64 = 86_400;

/// `027-B2` — how old a published snapshot may get before its misses stop being
/// certifiable answers and become the named `503`. Startup reads it too: the
/// snapshot is loaded only while it is inside this bound, and a past-bound one is
/// the full pull's case (`027-C3`).
pub const STEAM_SNAPSHOT_MAX_AGE_SECONDS: u64 = 604_800;

/// The gap the refresh thread waits after a *failed* pull, keeping the old snapshot
/// published in the meantime. No startup path sleeps it.
pub const STEAM_RETRY_SECONDS: u64 = 3_600;
