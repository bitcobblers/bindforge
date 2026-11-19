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

/// `026-B1` — the cache entry cap, enforced at write time by evicting the oldest
/// `fetched_at`, and re-checked at load.
pub const MAX_CACHE_ENTRIES: usize = 5_000;

/// `026-B2` — the cache document's octet cap. The test is the exact length of the
/// buffer that gets renamed into place, not a sum of per-entry totals (§Budgets).
pub const MAX_CACHE_DOCUMENT_OCTETS: usize = 1_048_576;

/// `026-B3` — the upstream TTL, in seconds, overridable by `--refresh-interval`.
pub const REGISTRY_UPSTREAM_TTL_SECONDS: u64 = 3_600;

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

/// The upstream client's socket timeouts, mirroring the shipped client's (A1).
pub const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// Read timeout for an upstream response, mirroring the shipped client's (A1).
pub const UPSTREAM_READ_TIMEOUT: Duration = Duration::from_secs(5);

/// How long an admitted connection may take to send its one request. Generous on
/// purpose: an idle admitted connection holds its slot, which is what makes the
/// `MAX_CONCURRENT_CONNECTIONS` ceiling observable, so this timeout only bounds
/// low-and-slow input, it does not police idle peers.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(60);

/// How long the answer may take to leave. Pairs with [`REQUEST_READ_TIMEOUT`] to
/// bound both directions of a connection's life (§Risks: low-and-slow).
pub const RESPONSE_WRITE_TIMEOUT: Duration = Duration::from_secs(15);
