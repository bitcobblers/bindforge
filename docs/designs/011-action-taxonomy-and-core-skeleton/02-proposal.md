# Design: ADR-011 — the action taxonomy and the `crates/core` spine

## Context

ADR-011 ([docs/adrs/011-action-taxonomy-and-core-skeleton.md](../../adrs/011-action-taxonomy-and-core-skeleton.md))
decides two things. **One:** the spine the 011-015 series extends — `crates/core`
as the home of the model types, `schemas/` as the language-neutral contract with
a conformance harness, `data/manifests/` and `data/profiles/` as placeholders.
**Two:** the action taxonomy — dotted lowercase ids in one shared namespace,
one orthogonal axis-kind attribute, and explicitly *no* placement semantics —
which every later contract references: ADR-012 declares `depends-on: [011]`,
ADR-013 maps game-native action names onto these ids, ADR-014's profiles speak
them, and ADR-015 may flip polarity only for `discrete-lateral` ones (A4).

This stage does not start from an empty tree. The ADR-008 design run already
shipped the spine: the workspace manifest, `crates/core` (`bindforge-core`),
`schemas/README.md` with `profile/v1` and `reach-classes/v1` and their fixture
corpora, and the harness `crates/core/tests/conformance.rs` (A1) — reviewed
work that closed with zero open blockers or majors (A2). So this design is
**additive**: it adds the taxonomy as a contract (`schemas/action-ids/v1/`) and
its typed model in `crates/core`, fills the `data/` placeholders, and leaves
the shipped guard (chain resolution, parse budget, `profile/v1`) alone — the
reset reading of "no behaviour beyond this ADR" is considered and rejected in
Options considered (A3). The research's open questions (01-research.md, Open
questions 1-8) are each answered in Chosen design or listed in Deferred.

## Goals / Non-goals

- Goals:
  - `schemas/action-ids/v1/`: the id grammar, the shipped id registry as data,
    and valid+invalid fixtures the existing harness walk picks up with **no
    harness change**.
  - `crates/core`: typed `ActionId`, `AxisKind`, `ActionRegistry`, and a loader,
    std-only, no new dependency.
  - The initial ~40-id set enumerated, not implied — for a third-person
    shooter / ARPG with Remnant 2 as the first exercise (A15).
  - One written rule for axis-kind classification, and one for id extension and
    withdrawal, both in the contract tree where a third-party reader sees them.
  - `data/manifests/` and `data/profiles/` created with READMEs; the stale
    ADR-009 lineage in the contract docs retargeted to the 011-015 series.
- Non-goals: deleting or resetting anything the ADR-008 run shipped, and a
  machine-checkable schema language for the contract. Everything else this
  stage does not answer is written once, with its owning document, under
  Deferred / out of scope.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | The spine already exists on this branch: root `Cargo.toml` with `members = ["crates/*"]`, `crates/core` as package `bindforge-core`, `schemas/README.md`, `schemas/profile/v1`, `schemas/reach-classes/v1`, and the harness in `crates/core/tests/conformance.rs` | verified | `Cargo.toml:12`; `crates/core/Cargo.toml:2`; `schemas/README.md`; `crates/core/tests/conformance.rs:47-77` |
| A2 | That tree is the reviewed, shipped output of the ADR-008 design run, including the adversarial guard (chain guard, parse budget) that design decided; its ratification pass recorded five approves with zero blockers and zero majors | verified | `docs/designs/008-rust-for-the-product/03-build-notes.md:1-46`, `:533` |
| A3 | ADR-011's "the product is greenfield" describes product behaviour, not the absence of the workspace tree the same ADR asks for | unverified | `docs/adrs/011-action-taxonomy-and-core-skeleton.md:30` against A1; resolved by Options considered (last option) |
| A4 | Later contracts hang off these ids: 012 declares `depends-on: [011]`, 013's action table is game-native name ↔ taxonomy id + axis kind, 015 flips polarity only for `discrete-lateral` | verified | `docs/adrs/012-input-vocabulary-and-reach-classes.md:6`; `docs/adrs/013-game-manifests.md:37`; `docs/adrs/011-action-taxonomy-and-core-skeleton.md:52-54` |
| A5 | RFC 1035 §2.3.1 constrains a DNS label to letters/digits/hyphen, letter first, letter-or-digit last, ≤63 octets, and declares it case-insensitive | verified | https://www.rfc-editor.org/rfc/rfc1035 |
| A6 | `toml-test` states that invalid tests "test one thing and one thing only" and are named after the fault they expose | verified | https://github.com/toml-lang/toml-test |
| A7 | The golden-fixture model (language-neutral corpora, conversion left to each implementer) is what this repo's harness already instantiates | verified | https://github.com/json-schema-org/JSON-Schema-Test-Suite; `crates/core/tests/conformance.rs:1-9` |
| A8 | Cargo compiles each file directly under `tests/` as its own integration target; `tests/common/mod.rs` is a shared module, not a target | verified | https://doc.rust-lang.org/cargo/reference/cargo-targets.html; `crates/core/tests/common/mod.rs:1-5` |
| A9 | The cargo gate already runs the fixture corpora on every PR, so a new format under `schemas/` needs no CI change | verified | `.github/workflows/check.yml:33` (`cargo test --workspace --locked`) |
| A10 | The v1 TOML subset already admits hyphenated bare keys and one-line string arrays including empty ones, so the axis-grouped registry shape needs no reader change | verified | `crates/core/src/toml.rs:179`; `crates/core/src/document.rs:284-301`; `schemas/profile/v1/fixtures/valid/basic-profile.toml:5`; the 33-link one-line array in `schemas/profile/v1/fixtures/invalid/chain-past-max-depth.toml:8` |
| A11 | git tracks no empty directory, so `data/manifests/` and `data/profiles/` only exist if each carries a committed file | verified | `.opencode/skills/repo-practices/SKILL.md:52-55`; `Cargo.toml:7-9` |
| A12 | SemVer's published deprecation vocabulary is announce-in-a-minor, remove-in-a-major | verified | https://semver.org/ |
| A13 | Growth-by-review of a shared name database works at community scale: SDL_GameControllerDB requires a contribution to improve and not degrade the mapping before merge | verified | https://github.com/gabomdq/SDL_GameControllerDB/blob/master/README.md |
| A14 | Published code registries handle the one-way door by written discipline: evdev freezes legacy codes and mandates a new constant over repurposing a name | verified | https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h |
| A15 | The 40 ids enumerated below cover a third-person shooter / ARPG with Remnant 2 as the first exercise; they have not been checked against a game's own binding list | unverified | `docs/adrs/011-action-taxonomy-and-core-skeleton.md:59-61`; enumeration in Chosen design |
| A16 | No stage of the 008 run could invoke a toolchain, so the gate commands have never been run against this tree; the merge PR's `rust` job is the evidence leg for this stage's tests as well | verified | `docs/designs/008-rust-for-the-product/03-build-notes.md:48-63` |
| A17 | No external standard grades a game action as `discrete-lateral` / `continuous-lateral` / `none`; the meaning of axis kind is whatever this contract's own spec and fixtures say it is | unverified | 01-research.md, Risks & unknowns ("Axis kind is a taxonomy-local concept") |

## Options considered

**Where the id set lives: `schemas/` data loaded by `crates/core`, a Rust
literal inside `crates/core`, or a build-script-generated table.** A Rust
literal makes the crate a second copy of the registry with a drift test between
them, and it contradicts the precedent the repo already set for a taxonomy-ish
table — the reach classes live in `schemas/`, "not in Rust literals"
(`crates/core/src/reach.rs:1-7`). A `build.rs` that turns the TOML into consts
adds a build step whose correctness depends on `rerun-if-changed` firing for a
file outside the package, for no benefit over a loader. Chosen: the registry is
data in the version directory and `crates/core` loads it through the same
`parse_file` path every contract document takes, exactly as
`load_reach_tables` does (`crates/core/src/reach.rs:103-131`).

**Registry shape: three axis-grouped string arrays, `id:kind` pair strings, or
table headers / dotted keys.** Table headers and dotted keys are legal TOML that
the v1 grammar already refuses, each with its own invalid fixture
(`schemas/profile/v1/spec.md:18-21`); admitting them for one format would widen
the contract for every reader. `id = "kind"` pair strings invent a mini-language
inside a string value — the machine-checkable schema language the contract
deliberately declines (`schemas/README.md:41-43`). Chosen: three arrays named by
axis kind, each a one-line string array (A10). This makes membership and axis
kind one fact: the arrays *are* the id set, so there is no id without an axis
kind and no axis kind to forget.

**Id grammar: borrow the DNS label rule, or keep it loose ("lowercase and
dotted"), or fold case.** A loose rule cannot generate invalid fixtures and
leaves `Move.Steer-Left`, `move..steer`, and `move-steer` as undecided one-way
questions. Case-folding (DNS is case-insensitive, A5) would let two spellings
name one action, which is exactly the churn ADR-011 forbids. Chosen: the DNS
label shape, restricted to lowercase and **exact** — an uppercase segment is a
rejection, not a canonicalization (fixtures below).

**Unknown and unprefixed ids: reject, pass through, or canonicalize.** Passing
an unknown id through makes a typo a valid binding that matches nothing, the
same failure the closed entry sets already refuse
(`crates/core/tests/grammar_closure.rs:4-9`). Chosen, two rules and no third
mechanism: the grammar refuses an id that is not exactly two well-formed
segments (so `steer-left` alone is a malformed id, not a valid unprefixed one),
and the registry's closed set makes an unregistered id simply not present
(`axis_kind` → `None`). Validation of *other* formats' ids is ADR-013's.

**Withdrawal: an alias/tombstone mechanism in v1, or version-directory
withdrawal.** Aliases need a resolution order, a deprecation signal, and a
second identity — machinery with no consumer yet. Chosen: the major-directory
route whose rule Stability states, which the existing versioning rule already
defines (`schemas/README.md:16-25`) and SemVer's vocabulary names (A12).

**Reset `crates/core` to a bare skeleton, or extend the shipped spine.** The
literal reading of "no behaviour beyond this ADR" would delete chain resolution,
the reach loader, the parse budget, and two fixture corpora that a five-seat
review approved (A2) — and it would strand `schemas/README.md`, the version-gate
test, and the guard ADR-008's design decided for the hub's upload path. ADR-011's
own Consequences ("the conformance harness is in place before the second
contract ships") is already satisfied by what is on the branch. Chosen: extend.
This is the reading A3 commits to.

## Chosen design

### Delta inventory

New: `schemas/action-ids/v1/` (spec + registry + fixtures),
`crates/core/src/action.rs`, two test targets, `data/manifests/README.md`,
`data/profiles/README.md`. Edited: `crates/core/src/document.rs` (third format),
`crates/core/src/lib.rs` (re-exports and the public-surface list),
`crates/core/tests/grammar_closure.rs` (its per-format list), `schemas/README.md`
and three module-doc lineages. Unchanged: `conformance.rs`, `common/mod.rs`,
`chain.rs`, `reach.rs`, `toml.rs`, `error.rs`, `Cargo.toml`, `Cargo.lock`, both
workflows, every existing fixture.

### The contract: `schemas/action-ids/v1/`

```
schemas/action-ids/v1/
  spec.md
  registry.toml                     # the shipped id set — the only id list
  fixtures/valid/    basic-registry.toml  empty-axis-arrays.toml  numeric-slot-ids.toml
  fixtures/invalid/  unknown-entry.toml  missing-axis-arrays.toml  uppercase-id.toml
                     leading-digit-segment.toml  leading-hyphen-segment.toml
                     trailing-hyphen-segment.toml  empty-segment.toml  underscore-id.toml
                     unprefixed-id.toml  third-segment.toml  segment-past-63-octets.toml
                     duplicate-id.toml  non-string-element.toml  non-ascii-id.toml
```

`registry.toml` is one `action-ids` document; the arrays are one line each
because multi-line arrays are outside the v1 grammar. This is the enumeration
the ADR leaves unwritten and the research flags as blocking every fixture
(01-research.md, Risks & unknowns):

```toml
# The action taxonomy, v1. This file is the only id list: bindforge-core loads
# it, it is not repeated in Rust. Adding an id means a PR to this file.
format = "action-ids"
version = "v1"
discrete-lateral = ["move.steer-left", "move.steer-right", "move.strafe-left", "move.strafe-right"]
continuous-lateral = ["aim.camera"]
none = ["move.forward", "move.backward", "move.jump", "move.sprint", "aim.sights", "aim.cycle-target", "combat.melee", "combat.fire", "combat.reload", "combat.dodge", "combat.block", "combat.skill-1", "combat.skill-2", "combat.skill-3", "interact.primary", "interact.secondary", "interact.pickup", "item.slot-1", "item.slot-2", "item.slot-3", "item.slot-4", "item.slot-5", "item.slot-6", "item.heal", "item.cycle", "ui.map", "ui.inventory", "ui.menu", "ui.journal", "ui.chat", "ui.scoreboard", "system.pause", "system.settings", "system.respawn", "system.screenshot"]
```

40 ids over seven namespaces: `move` 8, `aim` 3, `combat` 8, `interact` 3,
`item` 8, `ui` 6, `system` 4. Namespaces are a naming convention, not a closed
grammar set — the registry is the closed set, so `system.*` needs no special
case, a new namespace is just new rows, and the research's reserved-namespace
and unprefixed-id questions are answered by the two rules in Options considered
rather than by a third mechanism.

`spec.md` states, canonically (the contract, not the reader, is where a
third-party reader reads this):

- **Grammar** — the same documented TOML subset as every other format
  (`schemas/reach-classes/v1/spec.md:18-30`), and the entry table: `format`
  (string, required, `action-ids`), `version` (string, required, equal to the
  version segment of the directory holding the file), `discrete-lateral` /
  `continuous-lateral` / `none` (arrays of strings, all required, may be
  empty; length bounded by the shared parse budget). Any other entry is
  rejected: an entry naming a fourth axis kind or an `emphasis` is refused, not
  ignored, as ADR-011's "emphasis is *not* in the taxonomy" requires.
- **Id grammar** — an id is exactly two `.`-separated segments; each segment is
  ASCII lowercase letter first, then `[a-z0-9-]`, letter-or-digit last, and at
  most 63 octets (A5). Comparison is byte-exact: no case folding, no
  canonicalization. The v1 registry's namespaces are the seven above.
- **Uniqueness** — an id appears once across the three arrays; a repeat is a
  rejection naming the id and the array.
- **Classification rule** — see "Axis kind" below.
- **Extension and withdrawal** — see "Stability" below, which also names which
  copy counts: bindforge-core answers only from the `registry.toml` shipped in
  this versioned directory, so an `action-ids` document from elsewhere is data —
  never loaded as the registry, never consulted for `axis_kind`.

The fixtures follow `toml-test`'s rule: one fault each, file named after the
fault (A6). `unknown-entry.toml` carries `emphasis = ["combat.melee"]`.

### The model: `crates/core/src/action.rs`

- `AxisKind` — `DiscreteLateral`, `ContinuousLateral`, `None`, with `name()` and
  the closed-set `from_name()` mirroring `Hand` (`crates/core/src/reach.rs:21-48`).
- `ActionId` — a validated id: `ActionId::parse(&str) -> Result<ActionId, CoreError>`
  plus `as_str()`. It refuses a malformed segment or a wrong segment count.
- `ActionRegistry` — `version` plus the ids in document order, with `ids()`,
  `contains(&ActionId)`, and `axis_kind(&ActionId) -> Option<AxisKind>`.
- `load_action_registry(dir: &Path) -> Result<ActionRegistry, CoreError>` —
  `parse_file(dir.join("registry.toml"))`, the variant check, and a check that
  the declared `version` equals `dir`'s version segment, the shape of
  `load_reach_tables` (`crates/core/src/reach.rs:103-131`) so the size budget
  binds before the registry is read. `dir` is the directory shipped with the
  crate: a document from elsewhere is never loaded here (spec.md, above).

`document.rs` gains `ACTION_IDS_ENTRIES`, an `action-ids` arm in `parse`, a
`Document::ActionIds(ActionRegistry)` variant, an entry in `SUPPORTED_FORMATS`,
and a `serialize` arm that writes the three arrays back in document order — so
the corpus round-trips like `profile` does, and `conformance.rs` needs no new
special case (unlike `reach-classes`, which is asserted by load). Every refusal
reuses an existing `CoreError` variant — `InvalidValue` for a malformed id, a
duplicate, an out-of-set entry, or a version that disagrees with its directory,
`MissingEntry` for an absent axis array — so no new variant enters the error
contract and every rejection keeps the single-line, truncating, path-free
message the contract already promises, with an offending id echoed,
escaped to printable characters (`schemas/README.md:67-72`). `lib.rs`
re-exports the four new names and its public-surface list gains them.

### Axis kind

Canonical rule, stated in `spec.md`: an action is **`discrete-lateral`** when it
is one half of a left/right pair bound as two separately-bound endpoints, so
mirroring swaps two bindings; **`continuous-lateral`** when it is one graded
lateral control whose mirror is a sign flip on that control — camera look, whose
inversion is a game option, never a key swap; **`none`** otherwise. The
discriminator is the shape of the binding, not the device: a game that drives
steering from an analog stick still binds `move.steer-left` and
`move.steer-right` as two endpoints, so `move.steer-*` is `discrete-lateral`,
which is the classification the research flagged as silently consequential
(01-research.md, Risks & unknowns). Forward/back is not lateral, so it is
`none`. The registry pins the `discrete-lateral` set to exactly those four
ids, and Test strategy #2 asserts the set; 015 flips polarity inside it and
nowhere else. Nothing in this stage acts on the attribute (A17).

### Stability

The registry is a compatibility promise (ADR-011, Consequences), so `spec.md`
states: ids are added by PR to `registry.toml`, reviewed before merge — the
extend-don't-mutate discipline evdev writes down for its own code set (A14) and
SDL's DB enforces by review (A13); within a major, no id is renamed, retargeted,
or removed; withdrawal is a new major directory (`action-ids/v2`). Once ADR-013's
manifests exist, an id addition arrives through manifest review, which is the
gate ADR-011 names. Widening the *grammar* for ids already in the registry rides
that same directory: a third segment in `action-ids/v1/registry.toml` makes the
file invalid for every v1 loader (Test strategy #6), so it goes to
`action-ids/v2`, where the version gate is what refuses it for v1 readers.

### Placeholders and lineage

`data/manifests/` and `data/profiles/` are created with a README each (A11): one
paragraph saying what lands there and which ADR populates it (013 and 014), that
nothing reads them yet, that whatever is dropped there arrived from elsewhere
and is untrusted input once 013/014 read it, and pointing at the stability rule
in `schemas/action-ids/v1/spec.md` rather than restating it. `schemas/README.md`
adds `action-ids` to its format list and replaces the "arrive with ADR-009"
lineage with the 012-015 series; the `ADR-009` references in the module docs of
`crates/core/src/lib.rs`, `document.rs`, and `chain.rs` — the ones that
describe future model ownership — are retargeted to the same series. Docs
only — no behaviour, no fixture, no gate change.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A wrong id is published and cannot be recalled (one-way door, A14) | Med | High | Closed registry + fixtures; no rename/removal within a major; additions are reviewed PRs; withdrawal is a major directory |
| An id is graded with the wrong axis kind, silently widening or narrowing what 015 may flip (A17) | Low | High | The discriminator is in `spec.md`; the registry pins the four `discrete-lateral` ids; Test strategy #2 asserts that set exactly |
| The Rust model and the shipped registry drift apart | Low | Med | One source of truth: the TOML is the only id list and nothing in `src/` repeats it (Options considered, first) |
| The grammar turns out too strict for a later need | Med | Low | Grammar widening rides a major directory (Stability); the 63-octet bound is a borrowed, published number, not a guess (A5) |
| 40 ids do not cover the first game exercised (A15) | Med | Low | Additions are registry PRs; the set is explicitly ARPG-scoped (ADR-011, Consequences); no consumer exists yet to break |
| The new suite never runs locally (A16) | High | Med | The merge PR's `check.yml` `rust` job is Test strategy #7's leg, as it is for the 008 run's outstanding checklist; its `test result:` lines go in 008's build notes |
| Stale ADR-009 lineage misleads the next contributor | Med | Low | Retargeted in Rollout 4; `schemas/README.md` is the entry-point doc |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux CI runner (`check.yml` `rust` job) | yes | The new corpus runs under the three existing gate commands; no CI change (A9) |
| Windows 10/11 contributor | yes | Ids are lowercase-only and the existing case-unique fixture-name rule still holds; the registry is one file, so the `Left.toml` case-collision nit from the 008 run has no analogue here |
| macOS contributor | no (v1) | Design 008's matrix; nothing in this stage adds a macOS-specific requirement |
| Toolchain floor (1.85.0, edition 2024) | yes | std-only, no new dependency, `Cargo.lock` byte-unchanged |
| Third-party reader, non-Rust | no (v1) | Corpora are language-neutral TOML, but no non-Rust leg exists until the Deferred runner contract does |
| Product artifacts (CLI, desktop, hub) | not applicable | This stage ships no binary; how an installed artifact locates registry data is open, see Deferred |
| Steam Deck / SteamOS | no (v1) | Design 008's matrix; nothing here adds platform-specific work |

## Test strategy

1. Unit (`crates/core/tests/action.rs`, module-named per ADR-004/A8) — id
   grammar, one case per rule: accept `move.strafe-left`, `item.slot-6`,
   `aim.cycle-target`, and a 63-octet segment; reject an uppercase segment, a
   non-ASCII letter in a segment, a leading digit, a leading or trailing hyphen,
   an underscore, an empty segment (`move..steer`), a single-segment id, a
   three-segment id, a 64-octet segment, and the empty string.
2. Unit — axis kind is total and pinned: `AxisKind::from_name` accepts exactly
   the three spellings; over the shipped registry every id sits in exactly one
   array and `axis_kind` returns that kind; an unregistered id yields `None`;
   the `discrete-lateral` set equals exactly `move.steer-left/right` and
   `move.strafe-left/right`, so a re-grade fails the gate.
3. Unit — closed entry set for the format: a document with exactly the four
   defined entries parses; an unknown entry (an `emphasis`, or a fourth axis
   array) is refused naming the entry; a missing axis array is `MissingEntry`; a
   non-string array element and a duplicate id (same array, and across arrays)
   are refused naming the id.
4. Unit — version gate for the new format: an `action-ids` document declaring no
   version, or `v99`, or a `version` other than the version segment of the
   directory it is loaded from, is refused with the error naming the supported
   majors, and the majors `bindforge-core` declares still equal the majors
   present under `schemas/*/v*/`.
5. Integration (`crates/core/tests/action_ids.rs`) — the harness is unchanged:
   `conformance.rs`'s `schemas/*/v*/` walk exercises `action-ids/v1`, every
   valid fixture parses and re-serializes to a semantically equal document,
   every invalid fixture is refused with a single-line bounded message, and the
   zero-fixture rule fires if either directory is emptied;
   `grammar_closure.rs`'s per-format unknown-entry list includes `action-ids`.
6. Integration — the shipped registry loads: `load_action_registry` on
   `schemas/action-ids/v1/` returns 40 ids, every one grammar-valid and listed
   once, over the seven namespaces with the counts in Chosen design; a missing
   `registry.toml`, or one holding another format's document, is refused
   without naming a local path.
7. Platform — the three gate commands are green on the Linux CI runner (the
   merge PR's job is the leg, A16) and the suite is run once on a Windows
   contributor machine, where case-insensitive lookup is where a name rule
   would bite.

## Rollout

1. Add `schemas/action-ids/v1/spec.md` and `registry.toml`, then the fixtures
   named above; the corpus is what makes the format "supported"
   (`schemas/README.md:22-24`).
2. Add `crates/core/src/action.rs` with the `document.rs` and `lib.rs` edits of
   The model, then `crates/core/tests/action.rs`, `crates/core/tests/action_ids.rs`,
   and the `grammar_closure.rs` entry. No dependency, no `Cargo.lock` change, no
   change to `conformance.rs` or `tests/common/mod.rs`.
3. Create `data/manifests/README.md` and `data/profiles/README.md` (content in
   Placeholders and lineage). Confirm both directories exist in the commit — git
   tracks no empty ones (A11).
4. Retarget lineage text in `schemas/README.md` and the three `crates/core`
   module docs. No behaviour change.
5. No CI or gate change (A9). The PR's `rust` job posts the `test result:` lines
   for the two new targets into the outstanding evidence checklist that
   `docs/designs/008-rust-for-the-product/03-build-notes.md` carries (A16), and
   the Windows leg of Test strategy #7 is run once by a contributor.
6. Migrations and defaults: none. Nothing consumes the taxonomy yet — the first
   consumers are ADR-012's reach work and ADR-013's manifests — and no id is in
   the wild, so the compatibility promise starts at v1 with this file.

## Deferred / out of scope

- Game-native name ↔ taxonomy id tables, constraints, presets, and validating a
  manifest's ids against the registry — ADR-013.
- Emphasis/placement as a game fact, and anything that reads it — ADR-013.
- Polarity flips and any other use of axis kind — ADR-015.
- Input vocabulary (`keyboard:*`, `gamepad:*`), reach tables, and replacing the
  placeholder class ids in `schemas/reach-classes/v1/` — ADR-012.
- Profile chains, `seeded-from`, and profile-id grammar (distinct from action
  ids, which `profile/v1` does not reference) — ADR-014.
- Codecs, config-path discovery, and the CLI — ADR-017 (not yet written).
- Seed profile and manifest sets — ADR-016 (not yet written).
- A `toml-test`-style external runner contract so non-Rust readers can run the
  corpora without cargo — no follow-up yet.
- Id aliases or tombstones, if an id turns out wrong after publication — no
  follow-up yet; this design's answer is major-directory withdrawal.
- How an installed artifact locates `schemas/` data (same open question as the
  reach tables) — no follow-up yet; ADR-017's CLI is the likely owner.

## Revision log

- r1: Compatibility matrix (Steam Deck / SteamOS) — row rewritten to inherit 008's `no (v1)` status rather than point at it.
- r1: Compatibility matrix (non-Rust reader) — unexercised `yes (fixtures)` leg deleted; the row states the Deferred runner contract instead of a claim.
- r1: Stability × Risks (grammar widening) — the "additive change" posture deleted: a widened id in `action-ids/v1/` breaks every v1 loader, so it rides `action-ids/v2` and the version gate refuses it.
- r1: The contract (`fixtures/invalid`) × Test strategy #1 — `non-ascii-id.toml` added to pin the byte-exact ASCII grammar.
- r1: spec.md Extension and withdrawal × `load_action_registry` × Placeholders and lineage — the source is classified: only the `registry.toml` shipped in this versioned directory is the registry, so a document from elsewhere is parsed as data and never consulted for `axis_kind`.
- r1: spec.md entry table × `load_action_registry` × Test strategy #4 — a declared `version` that differs from the loading directory's version segment is an `InvalidValue` refusal; id count is bounded by the parse budget.
- r1: The model (error rule) — an offending id is echoed escaped to printable characters.

## As-built (2026-11-13)

This section records what was built. No decision above is changed by it: the
taxonomy, the closed sets, the axis-kind rule, the stability rule, and the
delta's scope all shipped as designed. Head compared: `e8bf4f4` against this
run's base `4aeb57c` (30 non-doc files, +1860/−37). The running implementation
record is `03-build-notes.md`; the items below are the places where the shipped
tree differs from a sentence of this design, each with its reason and the
section it touches.

1. **`ActionRegistry` keeps ids in id order, not the file's array order.**
   The model stores a `BTreeMap<ActionId, AxisKind>`; `ids()` answers sorted and
   `serialize` writes the three arrays in the entry table's kind order
   (`discrete-lateral`, `continuous-lateral`, `none`) with sorted ids.
   *Reason:* `toml::read_document` hands entries over as a `BTreeMap` and the
   delta inventory keeps `toml.rs` unchanged, so source line order is not
   recoverable in the model; what Test strategy #5 and `conformance.rs` assert is
   semantic round-trip equality, which holds, and nothing pins file order.
   *Affects:* Chosen design → The model (`ActionRegistry`); Test strategy #5.
2. **`ids()` returns `Vec<ActionId>` rather than a borrow.** *Reason:* the map
   pairs each id with its kind, so a `&[ActionId]` cannot be handed out.
   `contains` and `axis_kind` take `&ActionId` as designed. *Affects:* Chosen
   design → The model (interface shape).
3. **The entry-set build lives in `action.rs` as `registry_from_entries`
   (`pub(crate)`), called from `document::parse`'s `action-ids` arm** — not as a
   `parse_action_ids` helper inside `document.rs`. *Reason:* the id grammar and
   the once-across-three-arrays rule are taxonomy rules; `document.rs` keeps the
   entry table (`ACTION_IDS_ENTRIES`), the closed-set check, the
   `Document::ActionIds(ActionRegistry)` variant, and the serializer. *Affects:*
   Chosen design → The model; Delta inventory (which files hold which piece).
4. **`ActionRegistry::version` is a public field, not a getter.** *Reason:* it
   mirrors `Profile::version` and `ReachClassTable::version` in the same module.
   *Affects:* Chosen design → The model.
5. **The 63-octet segment bound is a module-private constant
   (`MAX_SEGMENT_OCTETS`)**, not exported beside `MAX_CHAIN_DEPTH` /
   `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH`. *Reason:* it is part of the id
   grammar, whose canonical statement is `spec.md`, and this design's
   public-surface list names only `ActionId`, `AxisKind`, `ActionRegistry`, and
   the loader; the number still appears in the refusal text. *Affects:* Chosen
   design → The model (its public surface) and The contract (the id grammar);
   the bound stays canonical in `spec.md` either way.
6. **One refusal beyond the design's list:** `load_action_registry` first
   refuses a directory whose own name is not a major segment (`v` + digits)
   before comparing versions. *Reason:* a mistyped path must not yield a
   registry that silently claims a major nobody declared. It reuses
   `InvalidValue`, one line, no path in the message, so the error contract is
   still unchanged. *Affects:* Chosen design → The model (loader); Deferred
   ("how an installed artifact locates `schemas/` data").
7. **Malformed-id refusals inside an axis array are re-keyed to name the array**
   (`name_the_entry`): the `InvalidValue`'s `key` is the axis array's name and
   the message still echoes the offending id through `one_line` (escape +
   truncate). *Reason:* Test strategy #3 asks the array-level refusals to name
   the entry/id; the variant and the message shape are exactly as designed.
   *Affects:* Chosen design → The model (error rule).
8. **Two `document::parse` arms were reshaped for the formatter, and
   `string_array` became `pub(crate)`.** The `reach-classes` and `action-ids`
   arms go through a local binding (`let table = …; Ok(Document::…(table))`)
   instead of a nested one-expression call, and one array-of-strings check is
   shared with `action.rs`. *Reason:* the gate's `cargo fmt --all -- --check`
   leg rejects the nested shape; the change is formatting and visibility only,
   with every refusal and every test expectation unchanged. *Affects:* Chosen
   design → The model; Rollout 2.
9. **The lineage retarget reached one line further than the delta inventory.**
   `schemas/README.md`'s `MAX_CHAIN_DEPTH` tune note and the matching line in
   `schemas/profile/v1/spec.md` now name ADR-014 rather than ADR-009.
   *Reason:* `chain.rs` is in the inventory's retarget list, and leaving the
   README and the profile spec on ADR-009 would have them disagree with the code
   they document. Docs only. *Affects:* Chosen design → Placeholders and
   lineage; Delta inventory. (The files still on ADR-009 lineage are enumerated
   in `03-build-notes.md`, deviation 7.)
10. **This design's Test strategy #3 says "the four defined entries"; the entry
    table defines five** (`format`, `version`, and the three axis arrays), and
    `ACTION_IDS_ENTRIES` ships five, which is what the entry table and the
    `missing-axis-arrays` fixture require. The code follows the entry table; the
    sentence here is the slip, and it is left as written. *Affects:* Test
    strategy #3.

Verified as designed at this head: the four named model names and the loader
exist with the named signatures; `ACTION_IDS_ENTRIES`, the `parse` arm, the
`SUPPORTED_FORMATS` entry, and the `serialize` arm are all present; `lib.rs`
re-exports the four names and its public-surface list carries them;
`registry.toml` is this design's enumeration (40 ids — 4 `discrete-lateral`,
exactly the four `move.steer-*` / `move.strafe-*` endpoints, 1
`continuous-lateral`, 35 `none`; namespaces 8/3/8/3/8/6/4); the corpus is the 3
valid + 14 invalid fixtures named here, one fault each and named after it;
`conformance.rs` and `tests/common/mod.rs` are untouched and the `schemas/*/v*/`
walk picked the new corpus up with no harness change; `grammar_closure.rs`'s
per-format list includes `action-ids`; both `data/` directories carry READMEs;
and there is no new `CoreError` variant, no dependency, no `Cargo.toml` /
`Cargo.lock` change, and no CI change.

Open at this date (recorded in `03-build-notes.md`, not fixed by this pass):
A16 stands — the three gate commands have still never been run against this
tree, so Test strategy #7's CI and Windows legs have no evidence yet; and code
review round 1's five seats approved with zero blocker and zero major, leaving
their minors and nits open in the tree.
