---
status: proposed
date: 2026-11-12
supersedes: 009
series: 009
depends-on: [009a, 009c]
---

# ADR 009d: Express profiles as chains of base and patch

Fourth installment of the 009 series. We will express a player's binding
layout as a **chain**: a base (a game default, a preset, a shared profile)
plus ordered patches of deltas on top. The artifact a player actually owns
forever is a personal layout built on someone else's base; chains make that
relationship first-class, forkable, and re-appliable when the base is
regenerated.

## Context

A preset is a starting point, not an answer: a player keeps ~95% of a good
shipped layout and tweaks the rest, and then owns that result — a personal
layout on top of a base. Modelling that as one flat file loses the
relationship: the player's tweaks are welded to the base, so a better base
cannot be adopted without re-doing the tweaks. Chains keep them separate.

## Decision

A profile file (`*.bindprofile`, TOML — human-editable, diff-friendly) is
either a **base** or a **patch**, and references what it derives from:

```toml
for = "remnant2@0.1"            # manifest id + version pin (009c)
based-on = "profiles/lefty-r2.bindprofile"   # a base, game-default, or preset id
generated-by = "policy:lefty-mouse-relocate" # recorded, never trusted
version = "1.0.0"
```

A patch lists action→input deltas (`set` / `clear` / `swap`) against its
base. **Resolution is deterministic:** apply the base, then each patch in
chain order. `bindforge` operations `diff`, `fork`, and `merge` operate on
chains; an adopter mounts at any rung — take the chain whole, or take its
base and patch independently.

Provenance fields record `seeded-from` (a game built-in preset, 009c),
`generated-by`, or hand authorship. Provenance labels the chain; it is
**never** the trust mechanism — validation (009c) and explicit adoption are.

`schemas/` carries the profile/patch format with conformance fixtures;
`crates/core` implements resolution.

## Alternatives considered

**Flat per-game profile.** Loses the base/patch relationship; a better base
forces redoing the tweaks. This is exactly what chains fix.

**A full snapshot per profile (no patches).** Simple to read, impossible to
fork cheaply or rebase; sharing "just my tweaks" becomes a manual diff.

**Provenance as trust** ("generated-by a known policy, so ship it"). Rejected:
provenance labels, it does not vouch. AI-generated bases are useful drafts;
they enter like any base — gated by validation and ranked by adoption.

## Consequences

Becomes possible: fork/merge/rebase of personal layouts; adopting an
improved base while keeping personal patches; sharing at any granularity.

Accepted costs: chain resolution must be deterministic and stable (a
compatibility surface); `diff`/`fork`/`merge` are real operations to build
and keep correct.

Not decided here: the transforms that produce bases (009e); the CLI surface
for the chain operations (ADR-012).
