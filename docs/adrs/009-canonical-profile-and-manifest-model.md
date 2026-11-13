---
status: superseded
date: 2026-11-11
supersedes: —
superseded-by: [009a, 009b, 009c, 009d, 009e]
---

# ADR 009: Define the canonical profile, transform, and manifest model

> **Superseded** 2026-11-12 by the 009 series — 009a (action taxonomy +
> `crates/core` skeleton), 009b (input vocabulary + reach classes), 009c
> (game manifests), 009d (profiles as chains), 009e (transforms as policies).
> The single ADR bundled five independently-testable contracts into one
> stage; the series splits the same decisions in dependency order.


We will define BindForge's core vocabulary and file model as the public
contract in `schemas/` — a semantic action taxonomy, a per-hand reach-class
model of the keyboard, profiles expressed as chains (base + patch),
composable transform policies, and game manifests that record what a game
*is*, never how a player *should* bind it — with `crates/core` as the
reference implementation (ADR-008's workspace). This is the first product
model ADR; every later one — codecs, apply, presets, the hub — speaks
through these types.

## Context

Most PC games ship one binding layout (WASD, right-handed) and leave
left-handed players choosing between an incompatible default or per-game
guesswork. Three observations from the people this is for shape the model:

1. **"Lefty" is not one transform.** The useful move for movement is
   relocating the WASD cluster to the right hand with its internal shape
   preserved — not a geometric mirror, which scatters cluster members (F and
   E do not move with it) and fails anyway because the keyboard's halves are
   asymmetric (the right pinky freely reaches `;` and `'`; the left's do not).
   Community practice disagrees on placement even for one cluster: IJKL and
   Returnal's built-in southpaw OKL; are both common, and IJKL breaks the
   home-row anchor while OKL; does not. Many left-handed players never swap
   mouse buttons at all.
2. **A preset is a starting point, not an answer.** Returnal ships a
   well-designed southpaw layout; an experienced player keeps ~95% of it and
   tweaks the rest. The artifact a player actually owns forever is their
   personal layout, built on someone else's base.
3. **The model must not be pigeonholed to handedness.** Players with
   disabilities and alternative controllers are the same pattern with
   different constraints — a named policy plus personal tweaks — and prove
   the abstraction when it serves them equally.

Meanwhile the product is greenfield: the repo holds only the orchestration
harness, and `apps/`/`tests/` are empty. The vocabulary decision comes first
because translation ("apply this shared profile to *this* game") is a matching
problem between profiles and game descriptions, and both sides must agree on
action identity before any game is supported.

## Decision

Five contracts. Their canonical definitions live in `schemas/` as versioned,
language-neutral format specs with golden-fixture conformance tests;
`crates/core` implements them (serde-based), and `crates/transforms` owns
the policy machinery.

### 1. Action taxonomy (identity, not placement)

Each bindable action is identified by a dotted lowercase id from a shared
namespace: `move.steer-left`, `move.strafe-left`, `aim.camera`,
`combat.melee`, `interact.primary`, `item.slot-1`, `ui.map`, `system.*`.
Two orthogonal attributes:

- **Axis kind** — `discrete-lateral` (strafe: a mirrorable binary),
  `continuous-lateral` (camera look: never a key swap; mouse-x inversion is
  a game option, not a binding), or `none`. Transforms may only flip
  polarity for `discrete-lateral`.
- **Emphasis is *not* in the taxonomy.** Where a game puts `combat.melee`
  (next to the index finger or on `F13`) is a game fact recorded in the
  manifest, because emphasis varies per game. The taxonomy identifies
  actions across games; it never claims a natural position for them.

The id set starts small (~40 ids covering third-person shooter/ARPG needs —
Remnant 2 is the first exercise) and grows only through manifest review, so
ids stay stable for profiles already in the wild.

### 2. Input vocabulary and reach classes

- Physical inputs are named OS-neutrally: `keyboard:<key>`,
  `mouse:left|right|middle|side-1|side-2|wheel-*`, `gamepad:*`. v1 names
  keyboard keys by US-QWERTY glyph (`keyboard:w`) with ANSI geometry;
  physical-position naming for other layouts is deferred, and the id shape
  reserves it.
- A **reach class** grades each key for each hand: `home`, `stretch`,
  `far`, `thumb`, `off-reach`, with left/right tables distinct — the halves
  are asymmetric by definition. The tables live in `schemas/` as data;
  `crates/core` loads them.
- A **cluster placement** is a shape-preserving assignment of a movement
  cluster to a target region (IJKL and OKL; are two legal placements of the
  WASD shape). Placement legality = preserved internal shape + per-action
  reach class not degraded beyond a bound.

### 3. Profiles are chains

A profile file (`*.bindprofile`, TOML — human-editable, diff-friendly) is
either a **base** or a **patch**, and references what it derives from:

```toml
for = "remnant2@0.1"            # manifest id + version pin
based-on = "profiles/lefty-r2.bindprofile"   # or game-default | preset id
generated-by = "policy:lefty-mouse-relocate" # recorded, never trusted
version = "1.0.0"
```

A patch lists action→input deltas (set / clear / swap) against its base.
Resolution is deterministic: apply base, then each patch in chain order.
`bindforge` operations `diff`, `fork`, and `merge` operate on chains; an
adopter mounts at any rung (take the chain whole, or take its base and patch
independently). Provenance fields record `seeded-from` (game built-in
preset), `generated-by`, or hand authorship. Provenance labels the chain;
it never substitutes for validation or adoption as the trust mechanism.

### 4. Transforms produce bases, never overwrite players

A **policy** is an ordered stack of named transforms
(`relocate-cluster(move.steer-*, →right, placement=OKL;)`,
`swap-polarity(move.strafe-*)`, `swap-mouse-buttons()` — default **off**).
Each transform declares the tags it touches and **preconditions**
("locomotion cluster is contiguous and left-handed in this base"); an
inapplicable transform reports and no-ops, never mangles. Transforms apply
only to bases (game defaults, presets); a personal patch sits above them in
the chain and survives base regeneration untouched.

### 5. Manifests state game facts

A manifest (`data/manifests/<game>/manifest.toml`) records: game id +
detected config fingerprint; codec family id (this ADR defines the *id* only,
not codecs); the action table (game-native action name ↔ taxonomy id + axis
kind); exclusivity groups, dead keys, and hard-docked UI constraints; and
any built-in presets, which profiles may reference with `seeded-from`.
Manifests are data, versioned, CI-validated; profiles pin a manifest version,
and CI re-validates dependents on every bump, surfacing game-patch drift as
a visible failure instead of a mid-game surprise.

### Crates

`crates/core` (taxonomy, reach tables, profile/patch resolution, manifest
types — the reference implementation of `schemas/`) and `crates/transforms`
(policy stacks) are created in ADR-008's workspace shape. `schemas/`,
`data/manifests/`, and `data/profiles/` are created with the first entries.
`apps/cli` lands in ADR-012, not here.

## Alternatives considered

**Flat per-game profiles, no shared taxonomy.** What every forum paste does
today. It cannot translate: a shared WASD profile meets a game whose melee is
elsewhere and the player's original guesswork remains, now with extra steps.

**One global mirror transform.** Cheapest to build, and wrong twice: it
reflects keys that don't move together (the cluster isn't the neighborhood),
and button swap must be opt-in because many left-handed players keep LMB/RMB
as-is. Handedness is a stack of independent decisions.

**AI-generated layouts as the primary artifact.** Generation is genuinely
useful for bootstrapping games with no profiles yet, but the placement math
stays deterministic (reach classes, placement legality) so drafts are
auditable; the model reserves `generated-by` and treats AI output like any
other base — gated and ranked by adoption, not trusted by provenance.

**Placement in the taxonomy ("melee belongs near the index").** Collapses on
contact: a melee-heavy FPS and a shooter that isn't bind it differently on
purpose. Emphasis is the game's, recorded in the manifest.

## Consequences

Becomes possible: translating one shared profile across games through the
manifest join; authoring a personal handedness/accessibility policy once and
re-applying it to every game BindForge supports; detecting binding drift per
manifest version; sharing at any granularity because chains are forkable.

Accepted costs: the taxonomy is a compatibility promise — ids in the wild
cannot churn, so extension goes through manifest review. The reach tables
are ANSI-US only at first; other keyboard layouts need their own tables and
physical-position key naming later. Gamepad binding ids are reserved but
unmodeled in v1 (controllers are ambidextrous; the itch is keyboard+mouse).
TOML file formats plus schema validation double the validation surface vs.
a JSON-only design.

Not decided here: codec implementations and config path discovery (ADR-012),
which policies ship as the seed preset set (ADR-011), the hub service and
voting (ADR-014), and where a user's personal policy document lives.
