---
status: proposed
date: 2026-11-12
supersedes: 009
series: 011-015
depends-on: [011]
---

# ADR 013: Define game manifests as the game-facts contract

Third installment of the 011-015 series. We will define the **manifest** —
`data/manifests/<game>/manifest.toml` — as the record of what a game *is*,
never how a player *should* bind it. The manifest is the right-hand side of
the translation join: it maps a game's native action names onto the taxonomy
(011) and pins the facts that make a shared profile applicable to this
specific game.

## Context

Translation joins a profile to a game. The profile speaks taxonomy ids; the
game ships its own native binding names and its own geometry of constraints —
keys that are hard-docked, actions that share a control, presets the game
ships built in. Without a recorded, versioned description of those facts,
"apply this profile here" is a guess, and a game patch silently breaks a
profile the player trusts.

Of the series, this contract is the least built: `data/manifests/` holds only
a README, and nothing in `crates/core` mentions manifests. What the earlier
runs did give us is the machinery a new format needs, and 013 does not pay for
it twice: `schemas/`'s versioned-directory layout with a golden-fixture
corpus, the shared parse path (`document.rs`'s `parse` with its `Document`
enum, the closed per-format entry sets, the size and nesting budgets, the
version gate that refuses an unsupported major by naming the supported ones),
and ADR-011's posture that a document arriving from elsewhere is parsed as
data and is untrusted at every entry point.

## Decision

A new format, **`schemas/manifests/v1/`**, with a `Document::Manifest` arm and
a `SUPPORTED_FORMATS` entry, records as versioned data:

- **game id + detected config fingerprint** — the game this manifest
  describes, and a fingerprint of the config it was read from.
- **codec family id** — an identifier only; codec implementations and
  config-path discovery are ADR-017. This ADR reserves the id space.
- **the action table** — game-native action name ↔ taxonomy id (011) + axis
  kind. This is the join's right-hand side, and ids are validated against the
  loaded registry, so a manifest cannot invent an action the taxonomy does not
  name.
- **constraints** — exclusivity groups, dead keys, and hard-docked UI
  constraints, so transforms (015) and profiles (014) never target a key the
  game will not honour.
- **built-in presets** — presets the game ships, which profiles may reference
  with `seeded-from` (014).

Manifests are data, versioned, and CI-validated through the conformance
harness that already runs the other two formats' corpora. Profiles pin a
manifest id **and version**; CI re-validates every dependent profile on each
manifest bump, so game-patch drift surfaces as a visible CI failure, not a
mid-game surprise.

## Alternatives considered

**Encode how-to-bind guidance in the manifest.** It would fuse game facts with
player policy and force a manifest edit for every policy idea. The manifest
states facts; policy lives in profiles and transforms.

**A single rolling manifest per game, unversioned.** Cheapest, and it makes
game-patch drift invisible: a silent manifest edit breaks profiles already in
the wild with no signal. Versioning plus CI re-validation is the whole point.

**Reusing the profile format for manifests.** Both are TOML documents, so one
format looks economical; it would force the closed entry sets to admit entries
one format forbids, dissolving the refusals the fixtures pin. Separate format
directory, same parse path.

**Codec implementations alongside the manifest.** Rejected as scope: the
manifest needs a codec *id* to route, not a codec; implementations are ADR-017.

## Consequences

Becomes easy: profile↔game translation through the taxonomy join;
per-manifest-version drift detection; transforms preconditioning on real game
constraints.

Costs: every supported game needs a maintained, versioned manifest; CI carries
a re-validation matrix of profiles-per-manifest-version; a third format joins
the conformance corpus and the `supported_majors` list.

Not decided here: codec family implementations and config-path discovery
(ADR-017); which preset sets ship as seeds (ADR-016); how a manifest is
authored or reviewed by a human.
