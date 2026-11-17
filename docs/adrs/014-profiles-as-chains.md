---
doc_type: adr
status: accepted
date: 2026-11-12
supersedes: 009
series: 011-015
depends-on: [011, 013]
---

# ADR 014: Express profiles as chains of base and patch

Fourth installment of the 011-015 series. We will express a player's binding
layout as a **chain**: a base (a game default, a preset, a shared profile)
plus ordered patches of deltas on top. The artifact a player actually owns
forever is a personal layout built on someone else's base; chains make that
relationship first-class, forkable, and re-appliable when the base is
regenerated.

## Context

A preset is a starting point, not an answer: a player keeps ~95% of a good
shipped layout and tweaks the rest, and then owns that result — a personal
layout on top of a base. Modelling that as one flat file loses the
relationship: the tweaks are welded to the base, so a better base cannot be
adopted without redoing them.

ADR-008 already decided more of this contract than the original ADR-009 knew.
On `main`: `schemas/profile/v1/spec.md` fixes the entry set — `format`,
`version`, `id`, and `derives`, an array of ancestors, nearest first, default
empty, closed on purpose — and `crates/core/src/chain.rs` ships
`MAX_CHAIN_DEPTH = 32` plus `resolve_chain`, which walks the ancestor chain
iteratively and refuses a self-derived chain with `CoreError::Cycle` and an
over-deep one with `ChainTooDeep` before any deeper traversal. Its own header
says so: chain *validation* is decided, "all other chain semantics are
ADR-014's."

So the delta is not "invent a chain". It is what the chain's links *mean* and
what a link contributes — and adding that to the closed v1 entry set is a
breaking change, so it rides a new major.

## Decision

**One: `schemas/profile/v2/`, v1 untouched.** v2 keeps `format`, `version`,
`id` and `derives` — `derives` stays the chain field, nearest ancestor first,
so the shipped loader, fixtures and `resolve_chain` keep their meaning — and
adds the parts v1 deferred:

- `for` — the manifest id and version pin (013) this profile applies to.
- `kind` — `base` or `patch`. A base stands alone; a patch lists deltas against
  the link below it.
- `set` / `clear` / `swap` — the delta lists: action id → input id, action ids
  to drop, pairs to exchange.
- provenance: `seeded-from` (a game built-in preset, 013) and `generated-by`.

**Two: resolution is deterministic.** Resolve the chain with `resolve_chain`,
apply the deepest base, then each patch in chain order to the top. The result
is a function of the chain, so two runs over the same chain agree; that
determinism is a compatibility surface from here on.

**Three: `diff`, `fork`, `merge` operate on chains.** `fork` copies the chain
and appends a patch; `merge` reconciles two patches hanging off one base;
`diff` reports the deltas between two chains. An adopter can mount at any
rung — take the chain whole, or take its base and patch independently.

**Four: provenance labels, never vouches.** Provenance records where a link
came from; it is not the trust mechanism. Validation (013) and explicit
adoption are. AI-generated bases enter like any base — gated by validation,
ranked by adoption.

## Alternatives considered

**Widen `profile/v1` in place.** Its entry set is closed on purpose and its
fixtures plus `supported_majors` pin it; adding `kind`/`set`/`for` would make
existing v1 readers meet unknown entries and refuse files that are valid today.
A new major is the mechanism the version gate already provides.

**A flat per-game profile.** Loses the base/patch relationship; a better base
forces redoing the tweaks. This is exactly what chains fix.

**A full snapshot per profile (no patches).** Simple to read, impossible to
fork cheaply or rebase; sharing "just my tweaks" becomes a manual diff.

**A new chain field (`based-on`) instead of `derives`.** Rejected: `derives`
ships, is pinned by fixtures, and already carries nearest-first order. Two
names for one relation invites a profile that quietly derives from nothing.

**Provenance as trust** ("generated-by a known policy, so ship it"). Rejected:
provenance labels, it does not vouch.

## Consequences

Becomes easy: fork/merge/rebase of personal layouts; adopting an improved base
while keeping personal patches; sharing at any granularity; the 32-deep bound
and cycle refusal already hold against hostile uploads.

Costs: two profile majors live at once; resolution must stay deterministic and
stable because it becomes a compatibility surface; `diff`/`fork`/`merge` are
real operations to build and keep correct; a patch is only meaningful against
the base it names, so a re-pinned base can silently change what "the same"
patch means.

Not decided here: the transforms that produce bases (015); the CLI surface for
the chain operations (ADR-017); where a player's personal chain is stored.
