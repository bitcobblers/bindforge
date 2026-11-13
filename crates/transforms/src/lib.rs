//! The transform stack of ADR-015: `policy/v1` documents applied over a pinned
//! `profile/v2` base, producing a new base (design 015, Chosen design → The
//! stack).
//!
//! This is the second crate in the workspace and the only one that depends on
//! `bindforge-core` (A16): core owns the formats, the grammar, the loaders, and
//! the placement-legality predicate, and this crate owns the one thing core
//! deliberately does not — *which* clusters move where, and what happens when a
//! row's preconditions do not fit the base in front of it. Everything the stack
//! reads is loaded contract data: a `policy/v1` stack, the base and its manifest
//! pin, the `action-ids` registry the match sets are computed over, the
//! `reach-classes/v2` tables and the `keyboard-geometry/v1` grid the placement
//! predicate runs over. Nothing here restates any of it (ADR-008 clause 3).
//!
//! # What the crate re-exports
//!
//! One entry point and the vocabulary around it:
//!
//! - [`apply`] — the application itself: a policy, a base, and the loaded data
//!   they are joined to, answered by an [`Applied`] base and one
//!   [`TransformReport`] per row, or by a [`CoreError`] when the *application*
//!   could not be evaluated at all (a patch as the base, a pin that does not
//!   match the manifest, a policy naming its own base, a base that binds one
//!   action twice).
//! - [`Outcome`], [`NotApplicable`], and [`NotApplicableReason`] — the verdict a
//!   row answers with when it does not fit. A row that does not fit is never a
//!   refusal: the reasons are the ten closed spellings `schemas/policy/v1/spec.md`
//!   states, so a non-Rust implementer of `policy/v1` produces the same verdicts
//!   (A11), and the layout comes out byte-equal with the source.
//!
//! `crates/transforms` is where ADR-015's two guarantees live. *A policy never
//! mangles a base*: a row is atomic, so its writes land only when every one of
//! its preconditions passes. *A policy that does not fit says so*: an
//! inapplicable row reports its reason and the offender it names rather than
//! quietly doing nothing.
//!
//! The stack is pure over parsed values — no clock, no randomness, no filesystem
//! — and every container it walks is sorted, which is what makes regeneration a
//! write a consumer can rely on rather than a re-draw (Chosen design → The
//! output, Regeneration).

mod apply;

pub use apply::{Applied, NotApplicable, NotApplicableReason, Outcome, TransformReport, apply};
