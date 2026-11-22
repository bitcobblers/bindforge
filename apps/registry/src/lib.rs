//! The library face of the BindForge registry (design 027, §The store pull's own
//! wording assumes it).
//!
//! `apps/registry` ships one binary, and ADR-004's module-mirrored test shape means
//! the parts of the design that have **no route through that binary** — the snapshot
//! document's loader, the store pull's codec, the transport's assurance half, the
//! refresh thread's decision — are only testable as a library. `tests/steam.rs` is
//! that suite, and it drives the *production* items below rather than a test-only
//! re-spelling of them: the connect address and the name verified against it are
//! separate parameters, the trust anchors and the pinned authority are the shipped
//! ones, and the codec, budgets, and retry rules are the ones a hosted instance
//! runs.
//!
//! The face is the store resolver ([`steam`]) and the seams it stands on
//! ([`constants`]' budgets, [`persist`]'s durable reads and writes), the mint rule
//! ([`mint`]) whose last rung no served query can reach — `keys.rs` caps the joined key
//! at 128 octets, so `mint identity over bound` is observable only at the seam — and
//! the instance's own state ([`state`]) with the documents it stands on ([`corpus`],
//! [`retired`]) and the key grammar ([`keys`]). That last group joined because ADR-027's
//! startup clause narrowed what a binary boots on: a snapshot is loaded only while it is
//! inside `027-B2`'s age bound, so the age-bound postures — the named `503`s a stale or
//! absent snapshot answers with — have **no route through a running instance any more**,
//! and a pin the design lists is not a pin if no face can reach it.
//!
//! The serving and admin faces stay bin-only until the CLI-mode suites move; the
//! binary keeps its own module declarations until then.

pub mod constants;
pub mod corpus;
pub mod keys;
pub mod mint;
pub mod persist;
pub mod retired;
pub mod state;
pub mod steam;
