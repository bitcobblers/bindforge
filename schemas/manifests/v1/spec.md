# manifests format, v1

The game manifest of ADR-013: `data/manifests/<game>/manifest.toml` is the
record of what a game *is* — a game id plus a fingerprint of the config it was
read from, a codec family id, the action table (the game's own names mapped to
the taxonomy ids in `schemas/action-ids/v1/registry.toml`), the constraints
that bound a transform, and the ids of the presets the game ships. It is the
join's right-hand side: a profile pins a manifest id and revision, and a
transform's preconditions read this document's constraints. `013-C4 013-L3`

Keeping the game facts in a manifest is what keeps one fact one home: axis
kind is a property of a taxonomy id that `registry.toml` owns per id, so no
entry here restates it, and a preset's *contents* are the seed set's (ADR-019)
and the codec's (ADR-017), so no entry here records a shipped layout.
`013-L2`

**Unit handles.** A handle is one trailing code span on a line, holding the
unit ids that govern that line's claim, separated by single spaces; an id
anywhere else on the line governs nothing. The ids here are the units of
`docs/adrs/013-game-manifests.md` (Decision); a line without a handle is
descriptive, or is governed by another ADR's unit — the cross-attributions are
listed in that ADR's Backfill findings.

## Table data

One file per game, sitting at `data/manifests/<id>/manifest.toml`: the game
directory's name must equal the `id` entry byte-for-byte — the same
name-agreement rule `reach-classes` and `keyboard-geometry` state for a hand
and a geometry. `013-C2 013-L5`
The id grammar is lowercase-only, so the agreement is case-unique on a
case-insensitive filesystem. Reading a manifest is subject to the same size
budget as every other document: a file larger than the budget is refused
before it is read into memory.

## Grammar (a documented subset of TOML v1.1.0)

Every valid document in this format is a legal TOML v1.1.0 document; the
grammar is the same documented subset `schemas/README.md` states and
`reach-classes/v1/spec.md` spells out. Line-oriented entries, one per line:
comments (`#`), and `key = value` pairs whose key is a TOML bare key
(`[A-Za-z0-9_-]+`) or a quoted basic string, and whose value is a basic string,
a decimal integer, a boolean, or an array of those written on one line. Basic
strings may use the escapes `\b`, `\t`, `\n`, `\f`, `\r`, `\"`, `\\`, `\uXXXX`,
and `\UXXXXXXXX`, and no other. In an integer, `_` may only separate digits;
there is no leading zero and no `0x`/`0o`/`0b` prefix. Table headers, dotted
keys, multi-line arrays, floats, and hex integers are legal TOML and outside
this grammar, and are rejected — which is why every array here is written on
one line, and why a `#` inside a quoted array element is data, never a comment.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `manifests` — `013-C1` |
| `version` | string | yes | the major this document is written against (`v1`) — `013-C1` |
| `id` | string | yes | the game slug (`[a-z][a-z0-9-]*`, at most 63 octets), byte-equal to the `data/manifests/<id>/` directory name — `013-L5` |
| `revision` | integer | yes | the data revision, `>= 1`, monotone per game (see Identity and revision) — `013-L4` |
| `codec` | string | yes | the codec family id (slug grammar, at most 63 octets); the id space is reserved here, implementations are ADR-017 — `013-C3 013-R1` |
| `fingerprint` | string | no | `"sha256-<base64>"`, the digest of the config this manifest was read from; absent means never read against a config — `013-C5` |
| `actions` | array of strings | yes | each element `"<action-id> <native-name>"`; may be empty — `013-C3 013-C6` |
| `exclusive-groups` | array of strings | no | each element `"<input-id> <input-id>[ <input-id>…]"`, 2..=16 members; at most 64 groups — `013-C3 013-B2 013-B3` |
| `dead-keys` | array of strings | no | each element one input id: a key the game will not honour at all — `013-C3` |
| `docked-actions` | array of strings | no | each element `"<action-id> <input-id>"`: an action the game hard-docks to that input, never a transform target — `013-C3` |
| `presets` | array of strings | no | each element one preset id (slug grammar): a built-in the game ships, referenceable by a profile's `seeded-from` — `013-C3 013-R4` |

Any other entry is rejected, naming the entry: `fixtures/invalid/unknown-entry.toml`
is the corpus case, and the refused alternative it pins — `axis-kinds` — is the
one the design considered and rejected: it would restate per row a fact
`registry.toml` already owns per id (the two answers, one fact, the drift
channel the closed sets exist to refuse). `013-C3 013-L2`

Input ids are spelled by the grammar `schemas/reach-classes/v2/spec.md`
states — that section is the canonical home of the id grammar for every format
that carries ids — and a member outside its closed sets is a refusal: v1
therefore cannot name a key like `tab` or `esc` as dead or docked; widening
those sets rides the vocabulary major that spec states. Action ids are spelled
by `schemas/action-ids/v1/spec.md`, and the registry join that checks them
against the loaded taxonomy is a *load-time* rule (Load-time validation), not
a grammar rule, so the corpus stays registry-free. `013-L1`

## The action row

Each `actions` element is `"<action-id> <native-name>"`, split at its *first*
space: an action id never carries a space (the id grammar pins it), while a
game-native name routinely does, so the split point is unique and the whole
remainder is the name. The name is the format's only free-text column, so it
carries its own grammar:

- 1 to 128 octets (`MAX_NATIVE_NAME_OCTETS`);
- no ASCII control character — even one the escape table decodes, so a hostile
  manifest cannot carry one into a displayed name;
- no leading space (an empty remainder) and no trailing space — the name must
  round-trip byte-exactly, so two documents differing only by a trailing space
  are never two facts about one game's label.

Spaces, `#`, and every glyph the closed escape table decodes are data:
`fixtures/valid/spaced-native-names.toml` pins a name carrying a `"`, a `\`,
and a `#` inside a quoted element. `013-C6 013-B4`

## One fact per document

- each action id appears once in `actions`;
- each native name appears once;
- each dead key appears once;
- each docked action id appears once;
- each preset id appears once;
- a group's members are distinct, and an input id appears in at most one group;
- `dead-keys` and `docked-actions` are disjoint — a key the game both will not
  honour at all and hard-docks to an action is two answers to one fact. `013-L8`

## Budgets

Named limits, exported by the reference implementation beside the shared
`MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` family (`schemas/README.md`), each
a parse-level refusal, and each well above the baseline it was surveyed for:
`013-B1 013-B2 013-B3 013-B4`

| Constant | Bounds |
|---|---|
| `MAX_MANIFEST_ROWS = 256` | each array-shaped entry: `actions`, `dead-keys`, `docked-actions`, `presets` — `013-B1` |
| `MAX_CONSTRAINT_GROUPS = 64` | the `exclusive-groups` array — `013-B2` |
| `MAX_GROUP_MEMBERS = 16` | one group's members — `013-B3` |
| `MAX_NATIVE_NAME_OCTETS = 128` | one game-native action name — `013-B4` |

## Fingerprint grammar

`sha256-` — the closed one-algorithm set; algorithm agility rides a new format
major — followed by standard base64 (alphabet `A-Za-z0-9+/`, `=` padding at
the end of the final quartet only) that decodes to exactly 32 octets. Grammar
only: which bytes are hashed and what a mismatch mean are ADR-017's, and the
value is provenance data, not a guard. `013-C5 013-R2 013-L7`

## Identity and revision

A manifest's identity is its slug `id` plus its integer `revision`. A profile
pins both as `"<id>@<revision>"` (e.g. `remnant-2@3`) — that entry's grammar
is the profile format's; its two halves are defined here, once. `013-L4 013-C4`

Bump rule: any edit to `actions`, `codec`, `fingerprint`, `exclusive-groups`,
`dead-keys`, `docked-actions`, or `presets` increments `revision`;
comment-only edits do not. The loader enforces monotonicity forward — every
already-pinning profile names the revision it was validated against, so a
manifest bump is a visible diff against every dependent pin. A manifest PR
that names an action id the registry does not carry must add its
`registry.toml` row in the same PR (the load-time join below otherwise
refuses it), so the id and the game fact that motivated it are reviewed
together. `013-L4 013-L6`

## Load-time validation

Two rules the corpus cannot carry, because they need the registry and the
game directory in hand, and the reference loader (`load_manifest`) applies
them after parsing: `013-C7`

1. the game directory's name must equal the `id` entry byte-for-byte, and the
   refusal never echoes a local path; `013-L5`
2. every action id the document names — each `actions` row and each
   `docked-actions` row — must be in the loaded registry; one that is not is
   refused naming the id. `codec` and preset ids are not registry-checked: no
   implementation set exists to check them against until ADR-017. Axis kind
   is never read from the document: a consumer that wants it asks the
   registry. `013-L1 013-L2 013-R3`

Every other rule in this spec refuses at parse alone — every
`fixtures/invalid/` document here is refused with no registry at all.

## Version gate and size budget

Same rules as every format (`schemas/README.md`): a missing or unsupported
`version` is rejected with an error naming the supported majors, never
defaulted, and the gate is per-format — `manifests` ships `v1` alone, so a
manifest declaring `v2` is refused naming `v1`
(`fixtures/invalid/major-not-shipped.toml`). `013-C1`
A rejection message quoting part of the document — an id, a native name — is a
single bounded line with the part echoed escaped to printable characters and
truncated, and never a local filesystem path.

## Fixtures

`fixtures/valid/` — documents this format accepts; the reference reader also
re-serializes each to a semantically equal document, in the fixed entry order
and the document's own array order.
`fixtures/invalid/` — documents every implementer must reject with a structured
error naming what is wrong; each carries one fault and is named after it.

The two load-time rules are not in the corpus: they are pinned against the
reference implementation directly (`crates/core/tests/manifest.rs`), which is
what keeps this directory registry-free. `013-C7`
The conformance harness's `schemas/*/v*/` walk picks this directory up with no
harness change; the corpus, not the code, is what makes the format supported.
