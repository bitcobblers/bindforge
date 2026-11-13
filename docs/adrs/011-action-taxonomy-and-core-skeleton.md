---
status: accepted
date: 2026-11-12
supersedes: 009
series: 011-015
depends-on: []
---

# ADR 011: Define the action taxonomy and the `crates/core` skeleton

The first product-model ADR, and the first installment of the 011-015 series
(011–015, superseding the oversized ADR 009). We will define BindForge's
**action taxonomy** as the identity layer the whole product speaks through,
and stand up the shared spine the rest of the series extends: the `crates/`
workspace, the `schemas/` directory with a golden-fixture conformance
harness, and `crates/core` as the reference implementation.

## Context

Translation — "apply this shared profile to *this* game" — is a matching
problem between profiles and game descriptions. Both sides must agree on
action **identity** before any game is supported, so identity is the root of
the model and every later contract references it. ADR 009 bundled identity,
input vocabulary, profiles, transforms, and manifests into one decision; that
scope is a single stage's work, and the 010 run showed how badly the harness
degrades when one `implement` stage carries a contract that large. The 009
series splits the same decisions into five contracts in dependency order;
011 is the root and carries the one-time bootstrap so 012–015 only add.

The product is greenfield: the repo holds only the orchestration harness.

## Decision

**One:** establish the spine, once, in ADR-008's workspace shape:

- `crates/core` — a compiling skeleton: no behaviour beyond this ADR, the
  home of every 009-series model type.
- `schemas/` — the language-neutral, versioned contract directory, with a
  **conformance harness**: every contract in the series ships a schema file
  plus golden fixtures (valid + invalid) that CI runs. This ADR creates the
  harness; later ADRs add schema/fixture pairs to it.
- `data/manifests/` and `data/profiles/` — created empty with a README,
  populated by 013/014.

**Two:** the action taxonomy. Each bindable action is identified by a dotted
lowercase id from a shared namespace: `move.steer-left`, `move.strafe-left`,
`aim.camera`, `combat.melee`, `interact.primary`, `item.slot-1`, `ui.map`,
`system.*`. Two attributes, one of them orthogonal and load-bearing:

- **Axis kind** — `discrete-lateral` (strafe: a mirrorable binary),
  `continuous-lateral` (camera look: never a key swap; mouse-x inversion is
  a game option, not a binding), or `none`. Only `discrete-lateral` actions
  may have polarity flipped by a transform (015); this ADR records the
  attribute, it does not act on it.
- **Emphasis is *not* in the taxonomy.** Where a game places `combat.melee`
  is a game fact, recorded in the manifest (013). The taxonomy identifies
  actions across games; it never claims a natural position.

The id set starts small (~40 ids covering a third-person shooter / ARPG,
Remnant 2 as the first exercise) and grows only through manifest review, so
ids already in the wild stay stable.

`crates/core` implements the taxonomy as typed ids with their axis kind, and
`schemas/` carries the id spec with conformance fixtures.

## Alternatives considered

**Flat per-game profiles, no shared taxonomy.** What a forum paste does
today; it cannot translate, because a shared WASD profile meets a game whose
melee lives elsewhere and the player's guesswork returns with extra steps.

**Placement in the taxonomy** ("melee belongs near the index"). Collapses on
contact: a melee-heavy FPS and a shooter that is not bind melee
deliberately differently. Placement is the game's, recorded in the manifest.

**One oversized ADR (the original 009).** Rejected on delivery risk: five
contracts in one stage, not five staged contracts.

## Consequences

Becomes possible: every later 009-series contract references stable action
ids; the conformance harness is in place before the second contract ships.

Accepted costs: the taxonomy is a compatibility promise — ids in the wild
cannot churn, so extension runs through manifest review; the id set is ARPG
scoped at first.

Not decided here: input naming and reach tables (012), manifests (013),
profile chains (014), transform policies (015), codecs and the CLI
(ADR-017).
