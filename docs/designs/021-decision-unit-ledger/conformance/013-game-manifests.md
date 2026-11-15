---
status: accepted
date: 2026-11-12
supersedes: 009
series: 011-015
depends-on: [011]
ledger: yes
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

A new format, **`schemas/manifests/v1/`**, with a `Document::Manifest` arm
and a `SUPPORTED_FORMATS` entry, records as versioned data. Manifests are
data, versioned, and CI-validated through the conformance harness that
already runs the other two formats' corpora.

The decision is recorded below as a **decision-unit ledger**: each unit is a
bounded choice with a pointer to its normative text in
`schemas/manifests/v1/spec.md` — the spec is the contract; this ledger is
the decision record, in application order. A backticked unit id at the end
of a spec line names the unit that governs that line's claim.

### Contracts

- **013-C1** The new format `schemas/manifests/v1/`: selector
  `format = "manifests"`, a `Document::Manifest` arm, a `SUPPORTED_FORMATS`
  entry, shipping `v1` alone; it joins the shared parse path, the
  conformance corpus, and `supported_majors`.
  → spec: Grammar, Version gate, Fixtures.
- **013-C2** Data location: one file per game at
  `data/manifests/<id>/manifest.toml`; the game directory's name equals the
  `id` entry byte-for-byte. → spec: Table data.
- **013-C3** What a manifest records: the game `id`, its `revision`, the
  codec family id, the config `fingerprint`, the action table (native name
  ↔ taxonomy id, 011 — the join's right-hand side), the constraints
  (`exclusive-groups`, `dead-keys`, `docked-actions`), and the built-in
  `presets`. Field-level typing and the closed entry set live in the spec.
  → spec: Grammar (entry table).
- **013-C4** The pinning contract: profiles pin a manifest id **and
  revision** as `"<id>@<revision>"`; CI re-validates every dependent
  profile on each manifest bump, so game-patch drift surfaces as a visible
  CI failure, not a mid-game surprise. (The pin entry's grammar is the
  profile format's — 014.) → spec: Identity and revision.
- **013-C5** The `fingerprint` field: optional; `sha256-<base64>` (standard
  base64, decoding to exactly 32 octets); the closed one-algorithm set —
  algorithm agility rides a new format major; absent means never read
  against a config. → spec: Fingerprint grammar.
- **013-C6** The action-row grammar: `"<action-id> <native-name>"`, split
  at its first space; the name is the format's only free-text column — 1 to
  128 octets (013-B4), no ASCII control character, no leading or trailing
  space, round-tripping byte-exactly. *Recovered from the spec — the source
  text is silent.* → spec: The action row.
- **013-C7** A reference loader `load_manifest` in `crates/core` applies
  the two load-time rules after parsing; its behaviour is pinned by
  `crates/core/tests/manifest.rs`, which is what keeps the corpus
  registry-free. *Recovered from the spec — the source text is silent.*
  → spec: Load-time validation, Fixtures.

### Laws

- **013-L1** The registry join: every action id the document names — each
  `actions` row and each `docked-actions` row — must be in the loaded
  registry; one that is not is refused, naming the id. The join is a
  load-time rule, not a grammar rule, so the corpus stays registry-free.
  → spec: Load-time validation.
- **013-L2** One fact, one home: the axis kind is never recorded in the
  document — the registry owns it per id, and a consumer that wants it asks
  the registry; the `axis-kinds` entry is the refused alternative the
  corpus pins. A preset's *contents* are never recorded (the seed set's and
  the codec's — ADR-017, ADR-019). *Reconciled against the source text's
  "+ axis kind" clause — see Backfill findings.*
  → spec: introductory paragraphs, Load-time validation, Grammar (the
  `axis-kinds` refusal).
- **013-L3** Facts, never policy: the manifest records what a game *is*,
  never how a player *should* bind it — no placement, no binding guidance;
  policy lives in profiles (014) and transforms (015). Enforced by the
  closed entry set: it admits no policy entry.
  → spec: introductory paragraphs, Grammar (entry table).
- **013-L4** Identity and revision: a manifest's identity is its `id` plus
  its integer `revision` (`>= 1`, monotone per game, enforced forward by
  the loader); any edit to a tracked entry (`actions`, `codec`,
  `fingerprint`, `exclusive-groups`, `dead-keys`, `docked-actions`,
  `presets`) increments `revision`; comment-only edits do not.
  → spec: Identity and revision.
- **013-L5** Name agreement: the game directory's name must equal the `id`
  entry byte-for-byte; the refusal never echoes a local path.
  → spec: Table data, Load-time validation.
- **013-L6** Review coupling: a manifest PR that names an action id the
  registry does not carry adds its `registry.toml` row in the same PR, so
  the id and the game fact that motivated it are reviewed together.
  → spec: Identity and revision.
- **013-L7** The fingerprint is provenance data, not a guard: no load-time
  rule validates it against a live config. → spec: Fingerprint grammar.
- **013-L8** Document uniqueness: each action id, native name, dead key,
  docked action id, and preset id appears once; a group's members are
  distinct and an input id appears in at most one group; `dead-keys` and
  `docked-actions` are disjoint. *Recovered from the spec — the source text
  is silent.* → spec: One fact per document.

### Budgets

All parse-level refusals, exported by the reference implementation beside
the shared `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` family
(`schemas/README.md`). *All four recovered from the spec — decided in the
run's design, never written back to the source text.* → spec: Budgets.

- **013-B1** `MAX_MANIFEST_ROWS = 256` — each array-shaped entry
  (`actions`, `dead-keys`, `docked-actions`, `presets`). → spec: Budgets.
- **013-B2** `MAX_CONSTRAINT_GROUPS = 64` — the `exclusive-groups` array.
  → spec: Budgets.
- **013-B3** `MAX_GROUP_MEMBERS = 16` — one group's members.
  → spec: Budgets.
- **013-B4** `MAX_NATIVE_NAME_OCTETS = 128` — one game-native action name.
  → spec: Budgets.

The spec asserts each cap sits well above the baseline it was surveyed for;
the baselines themselves are not recorded in the source text.

### Reservations

- **013-R1** Codec implementations and config-path discovery: the manifest
  carries the codec family id only (slug grammar, at most 63 octets) and
  reserves the id space. Owner: ADR-017 (reserved).
  → spec: Grammar (entry table, `codec`), Alternatives.
- **013-R2** Fingerprint semantics: which bytes are hashed and what a
  mismatch means. Owner: ADR-017 (reserved). → spec: Fingerprint grammar.
- **013-R3** Implementation-set checks: `codec` and preset ids are not
  registry-checked — no implementation set exists to check them against.
  Owner: ADR-017 (reserved). → spec: Load-time validation.
- **013-R4** The seed preset set: which presets ship as seeds.
  Owner: ADR-019 (reserved). → ADR-019
- **013-R5** How a manifest is authored or reviewed by a human: deferred,
  with **no owning ADR named** — flagged; the ledger convention should
  require an owner or an explicit `unassigned` marker.
  Owner: unassigned. → unassigned

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
(ADR-017); which preset sets ship as seeds (ADR-019); how a manifest is
authored or reviewed by a human.

## Backfill findings

Worked-example backfill (branch `docs/adr-013-backfill`): the ledger above
re-slices this ADR's normative text — editorial only, the narrative
sections untouched. Findings:

1. **The axis-kind clause contradicts the shipped spec.** The source
   action-table bullet read "native name ↔ taxonomy id (011) + axis kind";
   the shipped spec excludes the axis-kind column (the `axis-kinds` entry is
   the refused alternative the `unknown-entry` fixture pins, and the
   load-time rules never read it from the document). Unit 013-L2 follows
   the spec; the clause was dropped in the re-slice and is recorded here.
2. **The pinning wording was tightened.** The source text said "Profiles pin
   a manifest id **and version**"; the shipped contract (spec: Identity and
   revision) pins the id and the *revision*. Unit 013-C4 follows the spec.
3. **Seven units are recovered from the spec** (013-C6, 013-C7, 013-L8,
   013-B1–B4): decisions made in the run (design and build) but never
   written back to the source text. Provenance:
   `docs/designs/013-game-manifests/` (budgets in `02-proposal.md`).
4. **013-R5 has no owner.** The source "Not decided here" defers manifest
   authoring and review without naming an owning ADR; the ledger
   convention should require an owner or an explicit `unassigned` marker.
5. **Cross-attributed spec claims** — governed by other ADRs' units, to be
   handled by their own backfills: the TOML subset grammar, the version
   gate, the size budget, and the error-message bound line (011); the id
   and input-id grammars and the series name-agreement rule (011, 012);
   the pin entry's grammar (014). Series-wide coverage is complete only
   once every member is backfilled.
6. **The spec's preset-content attribution was corrected.** The shipped
   spec attributed a preset's contents to "ADR-016, ADR-017"; the pairing
   the ledger states (013-L2, 013-R4) is the seed set's ADR-019 and the
   codec's ADR-017. The ledger run corrected the line in
   `schemas/manifests/v1/spec.md` to that pairing; the correction is
   recorded here, not silently absorbed.
