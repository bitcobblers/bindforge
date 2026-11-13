# Design: ADR-013 — the game manifest as the game-facts contract

## Context

ADR-013 ([docs/adrs/013-game-manifests.md](../../adrs/013-game-manifests.md))
defines the **manifest** — `data/manifests/<game>/manifest.toml` — as the
record of what a game *is*: game id + detected config fingerprint, a codec
family id, the action table (game-native name ↔ taxonomy id), constraints,
and built-in presets (A15). It is the join's right-hand side: ADR-014 pins a
profile to a manifest id and version, ADR-015 preconditions transforms on the
manifest's constraints (A11). The contract machinery a new format needs
already ships and is paid for once: `Document`'s format arms, the closed
per-format entry sets, the per-format version gate, and the conformance
harness that walks `schemas/*/v*/` and picks a new directory up with no
harness change (A1, A3). What is new is one format directory
(`schemas/manifests/v1/`), one `Document` arm, one parse/serialize module, and
the data rules the research showed are load-bearing: a table encoding that
survives spaces in game-native names (A13), no restatement of registry-owned
facts, and budgets for the new shapes (A6).

The research's eleven open questions (01-research.md) are each answered in
Chosen design or listed under Deferred / out of scope.

## Goals / Non-goals

- Goals:
  - `schemas/manifests/v1/`: `spec.md` + a valid/invalid fixture corpus, so
    the corpus — not the code — is what makes the format supported (A3).
  - A closed entry set recording exactly the five fact groups the ADR names:
    identity + fingerprint, codec family id, action table, constraints,
    presets.
  - A row encoding that is unambiguous inside the one-line string-array
    grammar, even when a game-native name contains spaces (A5, A13).
  - Taxonomy ids validated against the loaded registry at load, so a manifest
    cannot name an action the taxonomy does not own — and axis kind stays a
    one-source fact answered by `ActionRegistry::axis_kind` (A7).
  - A manifest identity and data-revision rule that ADR-014's `for` pin can
    reference, plus the bump rule that keeps the pin honest (A11, A12).
  - Named parse budgets for every new shape — each array and the native
    name — beside the existing `MAX_DOCUMENT_BYTES` family (A6).
  - `crates/core`: typed `Manifest`, a `load_manifest` loader, a `serialize`
    arm that round-trips (the harness re-serializes this format, A3).
- Non-goals: codec implementations and config-path discovery (ADR-017); the
  seed-preset set and preset *contents* (ADR-016); the CI re-validation
  matrix over dependent profiles (ADR-014's pin first); growing the closed
  input vocabulary; authoring UX. Everything else this stage does not answer
  is written once, with its owning document, under Deferred / out of scope.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | `Document` has four arms, `SUPPORTED_FORMATS` lists four names, `format_majors` is the per-format half of the gate; a manifest is one new arm, one `SUPPORTED_FORMATS` entry, one closed entry-set constant, one `format_majors` line, and one `serialize` branch | verified | `crates/core/src/document.rs:57-62,125-131,192-197,327` |
| A2 | `supported_majors()` already returns `["v1", "v2"]`, and the pinned invariant is equality with the majors present under `schemas/*/v*/`; a third format at `v1` keeps it green — what grows is the *per-format* list, so the ADR's "`supported_majors` list" consequence overstates the change | verified | `crates/core/src/document.rs:114-116`; `crates/core/tests/version_gate.rs:205-217`; 01-research.md, Findings |
| A3 | The harness re-parses `serialize(document)` and requires a semantically equal value for every format except `reach-classes`, so manifests are contractually round-trip-bound; the zero-fixture rule and the lowercase-ASCII case-unique fixture-name rule bind the new directory | verified | `crates/core/tests/conformance.rs:59-91,209-224,326-365`; `schemas/README.md:36-40` |
| A4 | The grammar is line-oriented flat key/value: values are basic strings, decimal integers, booleans, or one-line arrays of those; table headers, dotted keys, multi-line arrays, floats and hex integers are legal TOML and refused, each with a fixture; the escape table is closed | verified | `schemas/action-ids/v1/spec.md:21-34`; `schemas/README.md:44-51` |
| A5 | The contract's table-as-data precedent is a space-joined string whose members carry no spaces, and a wrong field count is a pinned invalid fixture | verified | `crates/core/src/document.rs:405-416,574-579`; `schemas/keyboard-geometry/v1/fixtures/invalid/pair-two-fields.toml` |
| A6 | Parse budgets today are exactly `MAX_DOCUMENT_BYTES` and `MAX_NESTING_DEPTH`, both exported; nothing bounds rows, groups, group members, or name lengths | verified | `crates/core/src/toml.rs:36-39`; `schemas/README.md:66-84` |
| A7 | `load_action_registry`, `ActionRegistry::contains` and `ActionRegistry::axis_kind` exist; `registry.toml` is the single id list (40 ids), and axis kind is a property of the id, not of the game — the contract has no disagreement rule because nothing restates it | verified | `crates/core/src/action.rs:157-243`; `schemas/action-ids/v1/registry.toml`; `schemas/action-ids/v1/spec.md:74-91` |
| A8 | The input vocabulary is closed — `keyboard:` 48 members, `mouse:` 7, `gamepad:` 18 — an id outside a set is a refusal, and widening a member set rides a vocabulary major | verified | `crates/core/src/input.rs:72-121,186-198`; `schemas/reach-classes/v2/spec.md` |
| A9 | Real games bind well beyond that set (TF2 names `ESC`, `F1`–`F12`, arrows, `Tab`, modifiers, `MOUSE1`–`MOUSE5`) | verified | https://wiki.teamfortress.com/wiki/Scripting (01-research.md, Findings) |
| A10 | Nothing validates cross-file relationships today; CI is two PR jobs (`fmt`/`clippy`/`test --locked`, 15-minute timeouts) plus a scheduled, deliberately-not-a-gate cargo-deny job | verified | `crates/core/src/chain.rs`; `.github/workflows/check.yml`; `.github/workflows/advisories.yml` |
| A11 | ADR-014 adds `for` ("the manifest id and version pin") and `seeded-from` ("a game built-in preset, 013"); ADR-015 preconditions transforms on "a hard-docked or exclusive-group key is never a target" | verified | `docs/adrs/014-profiles-as-chains.md:47-52`; `docs/adrs/015-transforms-as-preconditioned-policies.md:48-50` |
| A12 | The superseded ADR-009 spells the pin `for = "remnant2@0.1"` — one lowercase segment plus a dotted version — next to the taxonomy's two-segment id grammar and the formats' `vN` majors: three incompatible conventions for the one string every profile pins | verified | `docs/adrs/009-canonical-profile-and-manifest-model.md:105`; 01-research.md, Risks |
| A13 | Game-native action names routinely contain spaces and multi-word display names (Steam IGA "Move"/"Camera" with localized text; Elite Dangerous XML multi-word control names), so a whitespace-split row must tolerate spaces in one column | unverified | 01-research.md, Risks (https://partner.steamgames.com/doc/features/steam_controller/iga_file; https://github.com/ngollan/Elite-Binds) |
| A14 | Every file under `data/manifests/` arrived from elsewhere and is untrusted input going through the same guards every contract document gets; the id-stability rule lives in the action-ids spec, not the README | verified | `data/manifests/README.md` |
| A15 | Codec implementations, config-path discovery, and therefore any source location for the fingerprint bytes are ADR-017's; the ADR reserves the codec id space only | verified | `docs/adrs/013-game-manifests.md:43-45,90-92` |
| A16 | Integrity metadata is conventionally an algorithm token plus a base64 digest with a closed, strength-ordered set; SRI warns that a digest of small, guessable content is brute-forceable and worthless over an unauthenticated channel | verified | https://www.w3.org/TR/SRI/ (01-research.md) |
| A17 | The native names and constraint facts a first shipped manifest would carry have no external authority the repo can check them against; correctness rests on per-manifest review, with the fingerprint as provenance only | unverified | 01-research.md, Summary and Risks (no surveyed equivalent for the constraint half) |
| A18 | The slug grammar `[a-z][a-z0-9-]*` already ships once, as `placement::is_geometry_name` (`pub(crate)`), imported by `document.rs` | verified | `crates/core/src/placement.rs:72-90`; `crates/core/src/document.rs:47,586` |
| A19 | The reference reader strips `#` as a comment only at line start or after a complete entry, so a `#` inside a quoted array element is data | verified | `crates/core/src/toml.rs:109,347-354` |

## Options considered

**The action row: three columns (id, name, axis kind), two columns with the
axis kind left to the registry, or a new nested structure.** The grammar
admits no tables, so nesting is out before it starts (A4). A third column
would restate a fact `registry.toml` already owns per id — two answers to one
fact, the drift channel the research flags; the contract's own
"presence in `fixtures/valid/` is the definition of supported" posture is
built to refuse exactly that (A7). Chosen: each `actions` element is
`"<action-id> <native-name>"`; axis kind is answered at load by
`ActionRegistry::axis_kind`, the registry is the single source, and the ADR's
"+ axis kind" clause is honoured by the *answer*, not a copy.

**Where the split point is, when names carry spaces: split on every space, a
dedicated separator, quoting, or remainder-splitting.** Every-space splitting
is ambiguous by construction for a name like "Move Left" (A13). A dedicated
separator (`|`) invents a character that native names may legitimately
contain, and quoting-then-splitting interacts with the closed escape table in
ways no fixture pins today. Chosen: split at the *first* space and take the
rest of the element as the name — sound because action ids never contain a
space (the id grammar that already pins it, A7), so the split point is unique
and a name may hold spaces, `#` (comment-stripping happens outside quotes,
A19), and every escaped glyph the reader already decodes. The name carries
its own grammar (length cap, no control characters, no leading or trailing
space) so the encoding round-trips byte-exactly.

**Manifest identity: reuse the two-segment action-id grammar, carry a store
id (Steam AppID precedent), or use a lowercase slug equal to the data directory
name.** The two-segment dotted grammar collides with the `@`-joined pin
ADR-014 will spell and buys nothing for a single game name (A12); a
store-assigned id covers only store games, and config/store discovery is
017's (A15) — adopting it now would make non-store games unidentifiable
forever. Chosen: `id = "<slug>"`, the already-shipped slug grammar (A18),
byte-equal to the `data/manifests/<game>/` directory name the README already
keys on — the Elite-Binds lesson that name↔location mismatch is what a
validator exists to catch (01-research.md).

**Data version: a `major_revision`/`minor_revision` pair, semver, or one
integer.** A pair re-imports Steam's flags-for-whom machinery, whose whole
purpose is player-side re-adoption prompts, which 013 deliberately resolves
in CI instead (01-research.md); semver strings hide that the contract has
one breaking level per directory already. Chosen: `revision`, a decimal
integer ≥ 1 (the grammar admits integers natively, A4), monotone per game.
Grammar changes ride a new `schemas/manifests/vN/` directory; game-fact
changes ride `revision`.

**Constraints over keys, or over actions.** The ADR's sentence — "so
transforms and profiles never target *a key the game will not honour*" — and
ADR-015's "a hard-docked or exclusive-group *key* is never a target" both
read as input ids (A11). Action-level exclusivity (the TF2
`+moveleft`/`+moveright` cancel) is a real fact but is not a key fact, and
the ADR's constraint clause is about targets. Chosen: all three constraint
entries name input ids from the closed vocabulary (A8). The consequence — a
manifest cannot yet claim `Tab` or `Esc` is dead (A8, A9) — is carried once,
under Deferred, as a vocabulary-major addition rather than forcing one here;
and action-level exclusivity is deferred the same way.

**Presets: record the shipped layout as data, or only stable preset ids.**
Recording layouts means shipping the game's default bindings as manifest
data, which duplicates what 017's codec reads from the config and would make
`actions` and preset blobs two answers to one join. Chosen: `presets` lists
preset ids (slug grammar, stable across a game's renames because BindForge
review assigns them), enough for 014's `seeded-from` to name (A11); preset
*contents* belong to the seed set (ADR-016) and the codec (ADR-017).

**The fingerprint: required, absent, or deferred entirely; which grammar.**
The ADR records "a fingerprint of the config it was read from" (A15) — a
manifest never read against a config has no fingerprint to record, so
required would invent one. Deferring the entry entirely strands the ADR
clause. Chosen: optional entry, SRI-shaped `sha256-<base64>` with a closed
one-algorithm set (A16); it is provenance data, not a guard — which bytes
are hashed and what a mismatch means are 017's, because 017 is the first
design that can name the bytes (A15).

**CI re-validation now, or its inputs now and its matrix later.** The
matrix's operands do not exist yet: no manifest data, no
`profile/v2`-with-`for` format, no cross-file machinery (A10, A12). Building
a scheduled job over an empty matrix buys a lint of nothing. Chosen: 013
ships the operands — stable id, monotone revision, a stated bump rule, a
loader that refuses a manifest the registry does not back — and the matrix
rides ADR-014's `for` pin; recorded in Deferred with its owner.

## Chosen design

### Delta inventory

New: `schemas/manifests/v1/` (`spec.md` + `fixtures/valid/` +
`fixtures/invalid/`, no shipped game data — the first real manifest is a
later PR, see Rollout 6), `crates/core/src/manifest.rs`, and
`crates/core/tests/manifest.rs` (module-mirrored per ADR-004).

Edited: `crates/core/src/document.rs` (the `manifests` arm, `MANIFEST_ENTRIES`,
`SUPPORTED_FORMATS`, the `format_majors` line, the `serialize` branch),
`crates/core/src/lib.rs` (module, re-exports, public-surface doc),
`crates/core/src/placement.rs` and both `is_geometry_name` call sites (the
grammar is renamed `is_slug_name` — one definition, named for what it is now
that three entry kinds share it, A18), `crates/core/tests/grammar_closure.rs`
(the manifest entry-set cases), `crates/core/tests/version_gate.rs` (the
manifest per-format refusal case), `crates/core/tests/conformance.rs` (the
expected-directory list gains `manifests/v1`; the *walk* itself stays
unchanged), `crates/core/tests/zz_api_probe.rs` (the new public names),
`schemas/README.md` (format list, one pointer line), `data/manifests/README.md`
(the loader exists; the directory stays empty until the first game PR).

Unchanged: `toml.rs`, `error.rs`, `chain.rs`, `action.rs`, `input.rs`,
`reach.rs`, `placement.rs`'s predicate, every existing fixture and shipped
format, `Cargo.toml`, `Cargo.lock`, both workflows.

### The contract: `schemas/manifests/v1/`

```
schemas/manifests/v1/
  spec.md
  fixtures/valid/    basic-manifest.toml  minimal-manifest.toml  spaced-native-names.toml
  fixtures/invalid/  unknown-entry.toml  major-not-shipped.toml  id-uppercase.toml
                     revision-zero.toml  row-no-name.toml  name-trailing-space.toml
                     name-control-char.toml  name-past-cap.toml  duplicate-action-id.toml
                     duplicate-native-name.toml  unknown-input-id.toml
                     dead-key-docked.toml  group-one-member.toml
                     group-duplicate-member.toml  group-past-member-cap.toml
                     rows-past-cap.toml  presets-past-cap.toml
                     fingerprint-bad-algorithm.toml  fingerprint-not-base64.toml
```

The grammar is the same documented TOML subset (A4); the entry set is closed.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `manifests` |
| `version` | string | yes | the grammar major this document is written against (`v1`) |
| `id` | string | yes | the game slug (`[a-z][a-z0-9-]*`, A18), byte-equal to the `data/manifests/<id>/` directory name |
| `revision` | integer | yes | the data revision, `>= 1`, monotone per game (Chosen design → Identity and revision) |
| `codec` | string | yes | the codec family id (slug grammar); the id space is reserved here, implementations are ADR-017 (A15) |
| `fingerprint` | string | no | `"sha256-<base64>"`, the digest of the config this manifest was read from; absent means never read against a config |
| `actions` | array of strings | yes | each element `"<action-id> <native-name>"`; may be empty |
| `exclusive-groups` | array of strings | no | each element `"<input-id> <input-id>[ <input-id>…]"`, 2..=16 members; at most 64 groups |
| `dead-keys` | array of strings | no | each element one input id: a key the game will not honour at all |
| `docked-actions` | array of strings | no | each element `"<action-id> <input-id>"`: an action the game hard-docks to that input, never a transform target (A11) |
| `presets` | array of strings | no | each element one preset id (slug grammar): a built-in the game ships, referenceable by `seeded-from` (A11) |

Any other entry is refused naming the entry, as every other format's closed
set refuses (A1); `fixtures/invalid/unknown-entry.toml` pins it, and
`major-not-shipped.toml` pins the per-format half of the version gate for
this format.

Rules, each with a fixture or a named check in Test strategy:

- **Row encoding.** An `actions` element splits at its first space: the head
  must be a grammar-valid action id, the remainder a native name of 1–128
  octets with no ASCII control character, no leading space (an empty
  remainder), and no trailing space. Names may contain spaces, `#` (A19),
  and any glyph the closed escape table decodes (A4), so
  `spaced-native-names.toml` round-trips one with `"` and `\` in it.
- **One fact per document.** Each action id appears once in `actions`; each
  native name appears once; each dead key appears once; each docked action
  id appears once; each preset id appears once; a group's members are
  distinct and an input id appears in at most one group; `dead-keys` and
  `docked-actions` are disjoint (a key the game both honours and refuses is
  two answers to one fact).
- **Budgets.** Named constants in `manifest.rs`, exported like the existing
  budget family (A6): `MAX_MANIFEST_ROWS = 256` bounds each array-shaped
  entry (`actions`, `dead-keys`, `docked-actions`, `presets`);
  `MAX_CONSTRAINT_GROUPS = 64`; `MAX_GROUP_MEMBERS = 16`;
  `MAX_NATIVE_NAME_OCTETS = 128`. Each cap sits well above the baselines
  surveyed for it (40 registry ids, A7; TF2-scale bind lists and multi-word
  native names, A9, A13); every cap is a parse-level refusal, and the
  shared 1 MiB / 16-deep budgets still bind first (A6).
- **Fingerprint grammar.** `sha256-` prefix (the closed algorithm set —
  algorithm agility is a future format major, A16) followed by standard
  base64 that decodes to exactly 32 octets. Grammar only: what bytes are
  hashed and what a mismatch means belong to ADR-017 (A15), and nothing
  consumes the value in this run.
- **Identity grammar.** `id`, `codec`, and each preset id are slug-grammar
  checked (`is_slug_name`, A18) at parse, each at most 63 octets — the same
  DNS-label bound RFC 1035 §2.3.1 publishes and the action-id grammar already
  borrows, so no unbounded string rides a displayed refusal. A malformed id
  is an `InvalidValue` echoing it escaped and truncated, as every other id
  does.

Example — what a first manifest will look like (not shipped this run):

```toml
# data/manifests/remnant-2/manifest.toml
format = "manifests"
version = "v1"
id = "remnant-2"
revision = 3
codec = "unreal-engine-5"
fingerprint = "sha256-2Z7Gx9m0YU3bJ8kT4sQw1n7dC6vH0pL2xR5yB8aN3eM="
actions = ["move.steer-left Turn Left", "aim.camera Mouse Camera", "combat.fire Primary Attack"]
exclusive-groups = ["keyboard:w keyboard:s"]
dead-keys = ["keyboard:`"]
docked-actions = ["ui.inventory Tab"]
presets = ["default", "southpaw"]
```

(`ui.inventory` is not yet in `registry.toml` and `Tab` is not yet in the
closed vocabulary (A8): see Registry growth order and Deferred.)

### The model: `crates/core/src/manifest.rs` and the `document.rs` edits

`manifest.rs` holds the model and the format rules the registry join needs:

```rust
pub struct Manifest {
    pub version: String,
    pub id: String,
    pub revision: i64,
    pub codec: String,
    pub fingerprint: Option<String>,
    pub actions: Vec<ManifestAction>,
    pub exclusive_groups: Vec<ExclusivityGroup>,
    pub dead_keys: Vec<InputId>,
    pub docked_actions: Vec<DockedAction>,
    pub presets: Vec<String>,
}
pub struct ManifestAction { pub action: ActionId, pub native_name: String }
pub struct ExclusivityGroup { pub inputs: Vec<InputId> }
pub struct DockedAction { pub action: ActionId, pub input: InputId }
```

`document.rs` gains: `Document::Manifest(Manifest)`;
`MANIFEST_ENTRIES` (the eleven sorted names, beside the other entry-set
constants); `"manifests"` in `SUPPORTED_FORMATS`;
`format_majors("manifests") == Some(&[MAJOR_V1])` — the *only* half that
grows, since the union already carries `v1` and the majors-equality invariant
stays green untouched (A2); a parse arm that runs the closed-set check and the
parse-level rules above; and a `serialize` branch that writes the entries in
the fixed order `format, version, id, revision, codec, [fingerprint], actions,
exclusive-groups, dead-keys, docked-actions, presets`, preserving each array's
document order so the harness's re-parse comparison is equality, not luck
(A3). No new `CoreError` variant enters the contract: every refusal is
`InvalidValue`, `MissingEntry`, or the version gate (the error shape is the
012 precedent). `lib.rs` re-exports `Manifest`, `ManifestAction`,
`ExclusivityGroup`, `DockedAction`, `load_manifest`, and the four budget
constants.

### Load-time validation: the registry join

`load_manifest(game_dir: &Path, registry: &ActionRegistry) ->
Result<Manifest, CoreError>` — the `load_action_registry` pattern (A7):

1. `parse_file(game_dir.join("manifest.toml"))` — size budget before the
   read, all parse-level rules (A6, A14).
2. **Location rule:** the directory's file-name segment must equal the `id`
   entry byte-for-byte; a mismatch is an `InvalidValue` that never echoes a
   local path (the error contract's channel for paths is `source_path`). The
   id grammar is lowercase-only, so the equality is case-unique on a
   case-insensitive contributor filesystem — the same platform posture as
   fixture names.
3. **Registry join:** every taxonomy id the document names — each
   `actions[].action` and each `docked_actions[].action` — must pass
   `registry.contains`; one that does not is refused, naming the id. (Group and
   dead-key members are input ids, closed at parse by `InputId::parse` (A8),
   so they have no registry side to join.) Axis kind is never read from the
   document: a consumer that wants it asks `registry.axis_kind(id)` (A7).
   Codec and preset ids are not registry-checked — no implementation set
   exists to check against until ADR-017 (A15).

The split is deliberate: the conformance corpus (parse level, A3) stays
registry-free and every invalid fixture refuses at `parse` alone, while the
two cross-file rules (join, location) live at load, where the registry and
the directory are in hand (A10). Test strategy #4 pins both.

### Identity, revision, and the series pins

Canonical once, cross-referenced from everywhere else: **a manifest's
identity is its slug `id` plus its integer `revision`.** ADR-014's `for`
entry joins them as `"<id>@<revision>"` (e.g. `remnant-2@3`) — that entry's
grammar is `profile/v2`'s (ADR-014 owns the entry; the two halves are defined
here). This retires ADR-009's superseded `remnant2@0.1` spelling and its
dotted minor, which no rule here would keep honest (A12).

**Bump rule:** any edit to `actions`, `codec`, `fingerprint`,
`exclusive-groups`, `dead-keys`, `docked-actions`, or `presets` increments
`revision`; comment-only edits do not. The rule is reviewed in the manifest
PR (core cannot pin a file's history, A10), and the loader enforces
monotonicity forward: a PR whose `revision` did not move while its facts did
is a review refusal, and every already-pinned `for` value names the revision
it was validated against, so a manifest bump is a visible diff against every
dependent pin the moment ADR-014's re-validation matrix lands (Deferred).

**Registry growth order** (the 40-id taxonomy is ARPG-scoped, and ADR-011
grows it only through manifest review): a manifest PR that names an id the
registry does not carry must add that `registry.toml` row in the same PR —
the load-time join otherwise fails the workspace test — so the id and the
game fact that motivated it are reviewed together.

### What the series reads from here

ADR-014 reads the pin halves (`id`, `revision`) and, for `seeded-from`, the
`presets` ids (A11). ADR-015's precondition checks consult the loaded
manifest: a transform's proposed target input is refused if it appears in
`dead_keys`, as any `exclusive_groups` member, or on the input side of any
`docked_actions` row — "a hard-docked or exclusive-group key is never a
target" is a set-membership lookup over loaded data, no new logic surface
(A11). The action side of `docked_actions` names *why* the key is docked, so
a transform can explain its refusal the way `PlacementCheck` explains a grade
failure.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The closed vocabulary cannot name a key a real game constrains (`Tab`, `Esc`, arrows) (A8, A9) | High | Med | The gap is named once (Deferred: vocabulary major via the `reach-classes/v2/spec.md` mechanism, a 012 decision); constraints still record every nameable key; nothing consumes constraints yet, so no game is blocked today |
| 40 ARPG ids vs the first real game → registry grows in batches | High | Med | Same-PR registry+manifest review order (Chosen design → Identity), and the load-time join fails any manifest naming an id the registry does not back |
| First manifest facts are authored without an external authority (A17) | High | Low–Med | Manifests are data reviewed per PR like the grade tables (012's precedent); the optional fingerprint records which config the facts were read from; nothing is trusted by provenance (014's rule) |
| The remainder-split encoding freezes field order forever: a future third column cannot go after the name | Low | Med | The name is last precisely so its spaces are data; any row-shape change rides `manifests/v2` with v1's corpus pinned by fixtures (A2, A3) |
| CI carries no re-validation matrix yet, so a bump over a stale pin is invisible until ADR-014 lands (A10) | Med | High | Zero exposure today: no manifest data, no pinning profiles; the bump rule and the `@<revision>` pin make the matrix a lookup once 014 ships; recorded in Deferred with its owner |
| A digest of a small config file is brute-forceable and, over an unauthenticated channel, worthless (A16) | Low | Low | The fingerprint is provenance data, not a guard; grammar-only semantics; transport is ADR-018's problem and will not be the manifest's algorithm's |
| A hostile manifest abuses the new shapes (many rows, long names echoed in refusals) | Med | Med | The four named budgets cap every array in the entry table before any join; every echoed native name goes through the escape-and-truncate error path; the 1 MiB budget binds before the read (A6, A14) |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux CI runner (`check.yml` `rust` job) | yes | The new corpus runs under the three existing gate commands; no CI change in this run (A10) — the re-validation matrix is Deferred |
| Windows 10/11 contributor | yes | Game-directory names, ids, codec and preset slugs are lowercase-ASCII slugs; `id` == directory name byte-equality plus the existing case-unique name rules are what make case collisions visible |
| macOS contributor | no (v1) | Design 008's matrix; nothing here adds a macOS-specific requirement |
| Toolchain floor (1.85.0, edition 2024) | yes | std-only, no new dependency, `Cargo.lock` byte-unchanged |
| Third-party reader, current Rust core | yes (refuses cleanly) | A manifest document seen by a pre-013 core is refused as `UnknownFormat` naming the supported formats — fail-closed, never reinterpreted; existing refusals stay byte-exact |
| Third-party reader, non-Rust | yes (data only) | `spec.md` + fixtures are the contract; this stage ships no code, binary, or runner contract for any language (Deferred, as in 011/012) |
| Product artifacts (CLI, desktop, hub) | not applicable | This stage ships no binary; the hub's manifest distribution and fingerprint transport posture is ADR-018 |
| Steam Deck / SteamOS | no (v1) | Design 008's matrix; nothing here adds platform-specific work |

## Test strategy

1. Unit (`crates/core/tests/manifest.rs`) — the row encoding: a two-field
   element parses to `{action, native_name}`; `basic-manifest.toml`-shaped
   documents with exactly the defined entries parse; an element with no
   space, an empty remainder, a trailing space, a control character in the
   name (via `\u0007`), a name past 128 octets, and a malformed action id in
   the head are each refused naming the element or entry; a name with spaces,
   `#`, `"`, and `\` parses and round-trips (A13, A19).
2. Unit — uniqueness and disjointness: a repeated action id, a repeated
   native name, a repeated dead key, a repeated docked action id, a repeated
   preset id, a group with a repeated member, a member in two groups, and an
   id both dead and docked are refused, each naming the offender; a group of
   one member and a 17-member group are refused.
3. Unit — identity and version rules: an uppercase, dotted,
   hyphen-prefixed, or 64-octet `id`/`codec`/preset id is refused;
   `revision = 0`, a negative revision, and a string revision are refused;
   a `fingerprint` with an unknown algorithm prefix, a non-base64 body, or a
   body decoding to 31 octets is refused; the version gate refuses a
   `manifests` document declaring `v2`, naming only `manifests`' majors (A2).
4. Unit — load-time validation: `load_manifest` reports a
   grammar-valid-but-unregistered action id as a refusal naming the id, and
   accepts it once a registry built from entries lists it; the directory
   segment must equal `id` (a scratch `Remnant-2/` or `other/` directory is
   refused without echoing a path); budgets — 257 `actions` rows, a 257th
   preset id, and a 65th group — refuse as `InvalidValue`.
5. Unit — round-trip: for every valid fixture and for a hand-parsed document
   carrying all eleven entries, `parse(serialize(parse(text)))` equals the
   first parse, including spaced/punctuation-bearing native names and array
   order (A3).
6. Integration (`crates/core/tests/conformance.rs`) — the unchanged
   `schemas/*/v*/` walk picks up `manifests/v1`: every valid fixture parses
   and re-serializes to a semantically equal value (manifests take the
   re-serialization path, not the reach exemption); every invalid fixture is
   refused with a single-line, control-free, truncating message; the
   zero-fixture rule and the case-unique fixture-name rule cover the new
   directory; `supported_majors()` still equals the majors present on disk
   (A2, A3).
7. Integration — the closed set and the untouched formats:
   `grammar_closure.rs` ships the manifest unknown-entry fixture in its
   per-format loop and accepts a document using exactly the defined entries;
   `version_gate.rs` refuses a `manifests` document declaring `v2`; every
   pre-existing fixture, loader test, and refusal stays byte-unchanged,
   including the `is_slug_name` rename's behaviour.
8. Platform — the three gate commands (`fmt`, `clippy -D warnings`, `test
   --workspace --locked`) are green on the Linux CI runner (the merge PR's
   job is the leg), and the suite runs once on a Windows contributor machine,
   where the slug-grammar and case-unique name rules are what would bite.

## Rollout

1. Add `schemas/manifests/v1/`: `spec.md` (grammar, entry table, row
   encoding, budgets, identity/revision rules, registry join, the
   `"<id>@<revision>"` pin cross-reference), then the fixtures named above.
   The corpus is what makes `manifests/v1` "supported" (`schemas/README.md`).
2. Add `crates/core/src/manifest.rs`; edit `document.rs` (arm, entry set,
   `SUPPORTED_FORMATS`, `format_majors`, `serialize`), rename
   `is_geometry_name` → `is_slug_name` at its three call sites plus the
   `document.rs` import, and extend `lib.rs` re-exports (no new dependency,
   no `Cargo.lock` change, no CI change, A10).
3. Add `tests/manifest.rs`; extend `grammar_closure.rs`, `version_gate.rs`,
   `conformance.rs`'s expected list, and `zz_api_probe.rs` (ADR-004
   mirroring; the public-surface probe names the new exports).
4. Update `schemas/README.md`: the format list gains `manifests`, and the
   budgets paragraph names the four new constants beside the existing
   exported limits.
5. Update `data/manifests/README.md`: the loader exists, so drop the
   "nothing in the tree reads it yet" sentence; the untrusted-input posture
   paragraph stands untouched.
6. Ship no game manifest data. The first real manifest lands as its own PR —
   with its `registry.toml` additions in the same PR (Chosen design →
   Identity) — after the vocabulary-major question for keys like `Tab` is
   settled for that game; this run's fixtures are the only corpus.
7. Evidence: the merge PR's `rust` job posts the `test result:` lines for
   `manifest.rs` and the updated suites; the Windows leg of Test strategy
   #8 runs once by a contributor.

## Deferred / out of scope

- Codec family implementations, config-path discovery, which bytes the
  fingerprint covers, and what a mismatch does — ADR-017.
- The profile side of the pin: the `for` entry's grammar in `profile/v2` —
  ADR-014 (its halves are canonical here, once).
- The CI matrix re-validating every dependent profile on each manifest bump
  — operand owner ADR-014; the scheduled-job side has no follow-up yet and
  needs one before the first pinned profile ships.
- The seed-preset set and preset contents (what a preset's layout *is*, and
  the game's shipped default bindings as a chain base) — ADR-016 (seeds) and
  ADR-017 (reading the shipped config); 013 records only preset ids.
- Modifier and named-key input members (`tab`, `esc`, arrows, F-keys):
  widening the closed sets rides the vocabulary major stated in
  `schemas/reach-classes/v2/spec.md` (a 012 mechanism) — no follow-up yet;
  `dead-keys` and `docked-actions` rows wait on it.
- Action-level exclusivity (the stock-game pair cancel, e.g.
  `+moveleft`/`+moveright`) and Steam-style modal/layer UI facts —
  `manifests/v2` if ADR-015's precondition vocabulary needs them; no
  follow-up yet.
- Manifest authoring and human review tooling — ADR-013 states it is not
  decided there; no follow-up yet.
- Hub distribution of manifests, fingerprint transport, and manifest trust
  over a channel — ADR-018.
- Localization of display names behind native names (Steam's `#token`
  precedent) — v1 records the name the config itself uses; no follow-up yet.

## Revision log

- r1: Chosen design → Budgets — `MAX_MANIFEST_ROWS` extended to `presets`,
  no new constant; new invalid fixture + Test strategy #4 case (major).
- r1: Chosen design → Budgets — each cap tied to the baseline it sits above
  (nit).

## As-built (2026-11-13)

Implementation record, not a design revision. The change set
(`git diff fc71ff96...HEAD`, 34 code files outside `docs/designs/`, commits
`756a40b` test-authoring and `500e9e4` implement) was verified against this
proposal after the round-1c0 code reviews (all five seats approved, zero
open blocker/major). The delta inventory landed as named — every New/Edited
entry present, every Unchanged entry absent from the diff. Drift, each
verified against the diff (reasons and full detail in `03-build-notes.md`):

1. **Fixture element counts** (Chosen design → The contract; Test strategy
   #4): `rows-past-cap.toml` carries 260 rows, `presets-past-cap.toml` 260
   preset ids, `name-past-cap.toml` a 169-octet name — headroom over the
   caps, not the exact 257 / 128+1 counts. Reason: the authoring shell
   intermittently refused scripted file generation, so the corpora were
   written directly; the cap, not the count past it, is what the fixture
   pins.
2. **`unknown-input-id.toml` places the unknown id (`keyboard:tab`) in
   `dead-keys`** (The contract, fixture list). The proposal names the
   fixture but not the entry; `dead-keys` is the entry ADR-015's target
   lookup reads first and names the Deferred vocabulary-major gap at its
   most load-bearing.
3. **`spaced-native-names.toml` pins three names** (The contract; Test
   strategy #1): the `"`/`\`/`#`-bearing name this section calls out,
   verbatim, plus interior-multi-space and parenthesized names. Additive;
   both extras are legal under the name grammar.
4. **`document.rs`'s `string_entry`/`string_value` became `pub(crate)`**
   (Chosen design → The model) so the `manifests` arm reuses them instead
   of restating them — mechanical, inside a file the delta inventory
   already edits.
5. **The location-rule refusal echoes the directory's last path segment
   and the document's `id`** (Chosen design → Load-time validation, step
   2). What this section forbids is echoing a local *path*, which the
   message does not (the scratch-path absence is pinned by
   `tests/manifest.rs`); naming both spellings is what makes a
   name↔location mismatch actionable.

As-built detail, not drift: the parse arm delegates model-building to
`manifest::manifest_from_entries` behind the closed-set check;
`load_manifest` guards that `manifest.toml` holds a manifests document;
`serialize` writes the five array entries unconditionally (an omitted
optional round-trips as `[]`, semantic equality intact) while an absent
`fingerprint` stays absent; the 63-octet slug bound is a private constant,
not an exported budget. One shipped-wording gap is open at HEAD:
`spec.md`'s "The loader enforces monotonicity forward" overstates
`load_manifest`, which enforces only the location rule and the registry
join — monotonicity is enforced by manifest-PR review and the deferred
pin matrix exactly as this section states (security review, minor; spec
wording only, no code behavior changed).
