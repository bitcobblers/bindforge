# policy format, v1

A policy document is an **opt-in stack of transforms**: it names a base layout's
shape changes by tag, in the order the author wrote them, and nothing else. It
carries no bindings, no file paths, and no reference to any other document: the
base it applies to, and everything it reads about that base, come from the
caller. A policy is the fourth untrusted document class — it arrives from disk,
from another user, and from the network exactly as a profile, manifest, or
reach-class table does — so the closed-entry-set, escape-and-truncate, and
parse-budget posture of `schemas/README.md` applies to it unchanged.

This spec states the per-document rules **and** the load-level and application
rules, so a non-Rust implementer can conform to the whole policy contract and
produce the same verdicts as `bindforge-core` plus `bindforge-transforms`. The
fixture corpus stays single-document, like every other format's.

## Grammar (the same documented subset of TOML v1.1.0)

The grammar is `profile/v2`'s, unchanged: line-oriented entries, one per line —
comments (`#`) and `key = value` pairs whose key is a TOML bare key
(`[A-Za-z0-9_-]+`) or a quoted basic string, and whose value is a basic string,
a decimal integer, a boolean, or an array of those written **on one line**.
Basic strings may use the escapes `\b`, `\t`, `\n`, `\f`, `\r`, `\"`, `\\`,
`\uXXXX`, and `\UXXXXXXXX`, and no other. In an integer, `_` may only separate
digits; there is no leading zero and no `0x`/`0o`/`0b` prefix. Table headers,
dotted keys, multi-line arrays, floats, and hex integers are legal TOML and
outside this grammar: each is rejected, and each has a fixture under
`fixtures/invalid/` pinning it.

A `transforms` element is one string holding a space-joined transform row. No
id grammar contains a space, so a row splits at its single spaces
unambiguously, and a row's field count is fixed by its kind.

## Entries

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `policy` |
| `version` | string | yes | the grammar major this document is written against (`v1`) |
| `id` | string | yes | the policy's identifier: slug grammar (`[a-z][a-z0-9-]*`), at most `MAX_SLUG_OCTETS` octets — the shared bound `schemas/README.md` states once (`024-B1`) — it is the file stem `load_policy` joins and the name a produced base carries in `generated-by` |
| `transforms` | array of strings | yes | the ordered stack, `1..=MAX_POLICY_TRANSFORMS` (64) elements, each one transform row; **array order is application order** — order is data |

Any other entry is rejected, naming the entry. The entry set is closed on
purpose: an alias of a defined entry (`transforms_`, `Transforms`) would
otherwise read as that entry being **absent** — a policy whose stack silently
empties does nothing while looking as though it does something.
`fixtures/invalid/unknown-entry.toml` is the corpus case.

## Transform rows

```
relocate-cluster <tag> <from-hand> <to-hand> <bound> <target-input>…   (1..=16 targets)
swap-polarity <tag>
swap-mouse-buttons
```

The kind set is closed and compiled into the grammar. A row naming a fourth
kind is refused naming the row: an open name space would make an unresolved
name somebody else's semantics. A row whose field count contradicts its kind is
refused naming the row — never padded with a default for the field left out and
never ignoring the extra one. An empty row is refused, not read as a no-op.
Fixtures: `transform-unknown-kind.toml`, `transform-empty-row.toml`,
`relocate-few-fields.toml`, `swap-polarity-extra-field.toml`,
`mouse-row-carries-tag.toml`.

- **`<tag>`** — an exact action id, or the prefix form `<segment>.<prefix>-*`,
  where `<segment>` is a well-formed id first segment (`[a-z][a-z0-9-]*`) and
  `<prefix>` is `[a-z][a-z0-9-]*`. Matching is byte-exact: the prefix form
  reaches every id whose second segment begins `<prefix>-`, so `move.steer-*`
  matches `move.steer-left` and `move.steer-up` and never `move.strafe-left` or
  `aim.camera`; an exact tag matches the id it spells and is not a prefix. The
  shape is checked at parse — a star anywhere else, an uppercase spelling, an
  empty prefix, and a third segment are each refused naming the tag — while
  whether the pattern matches a *loaded* registry id is an application fact,
  never grammar. Fixtures: `tag-embedded-glob.toml`, `tag-uppercase.toml`,
  `tag-empty-prefix.toml`, `tag-three-segments.toml`.
- **`<from-hand>`, `<to-hand>`** — `left` or `right`, the shipped hand grammar.
  They stay *inputs the row declares*: a hand is never derived from the data.
  `fixtures/invalid/relocate-bad-hand.toml` is the corpus case.
- **`<bound>`** — a decimal integer `0..=4`: the reach-taxonomy's five severity
  classes make four the largest possible one-action degradation, and past that
  a bound answers nothing. A letter, a negative, a float, and `5` are refused
  naming the row. Fixtures: `relocate-bound-letter.toml`,
  `relocate-bound-past-scale.toml`.
- **`<target-input>`** — a closed-vocabulary input id, grammar-checked at parse
  by the shipped `InputId` rule whose member sets `schemas/reach-classes/v2/spec.md`
  states: `keyboard:tab` is outside the keyboard set, so it refuses here rather
  than waiting for an application that cannot place it. One to
  `MAX_RELOCATE_TARGETS` (16) targets, no repeats. Fixtures:
  `relocate-target-unknown-input.toml`, `relocate-target-past-cap.toml`.

A policy `id` outside the slug grammar or past the shared `MAX_SLUG_OCTETS`
bound (`schemas/README.md` states it once; `024-B1`) is refused: the same rule
a manifest id and a profile id ride, because this id is also a file name and
must stay case-unique on a case-insensitive filesystem.
Fixtures: `id-uppercase.toml`, `id-past-cap.toml`.

`fixtures/valid/` ships the shapes a reader must accept: a one-row stack
(`minimal-policy.toml`), the two no-geometry kinds together
(`polarity-and-mouse.toml`), a relocate (`cluster-to-the-right-hand.toml`), a
three-row stack (`lefty-stack.toml`), two rows touching one tag
(`same-tag-stack.toml`), and exact-id tags (`exact-tag-policy.toml`). Whether a
row is *legal* against a real base is an application fact pinned in
`crates/transforms/tests`; the corpus pins documents.

## Version gate

A document that declares no `version`, or a major the reader does not support,
is rejected with a structured error naming the supported majors — never
defaulted. The majors are the ones this **format** ships: `policy` ships `v1`
alone, so a `policy` document declaring `v2` is refused naming `v1`, and
`profile/v2` never widens what a policy reader accepts. A new transform kind
rides `policy/v2` with this gate holding older readers closed.
`fixtures/invalid/missing-version.toml` and
`fixtures/invalid/major-not-shipped.toml` are the corpus cases.

## Missing entries

`format`, `version`, `id`, and `transforms` are required: an absent one is a
structured refusal naming the entry, never a policy with no name or an empty
stack. `fixtures/invalid/missing-id.toml` and
`fixtures/invalid/missing-transforms.toml` are the corpus cases. An empty
`transforms` array is a separate refusal — the stack is `1..=64`, and a policy
that names no transform is a document whose meaning nobody agreed
(`fixtures/invalid/transforms-empty.toml`).

## Loading a policy

`load_policy(dir, id)` reads one policy document, and its rules are the shipped
loader discipline:

1. **Grammar before filesystem.** `id` is slug-checked *before* any filesystem
   question, so `..`, a separator, an absolute path, or an uppercase spelling
   refuses naming the grammar and nothing else: no file is opened, and no local
   path appears in the message.
2. **One path, in the major directory.** The document is `<id>.toml`, a direct
   child of `dir`. Nothing else names a location.
3. **Major from the directory.** The declared version must equal the `vN`
   segment of the directory the file sits in; a `v1` document inside `v2/`
   refuses.
4. **Name ↔ location.** The file's stem must equal the document's `id` entry.
5. **Format arm.** A document at the policy path whose `format` is not `policy`
   refuses on the format arm; a loader does not reinterpret a profile or a
   manifest as a policy.

Every refusal reuses a shipped error variant — no policy-specific variant
exists — in the single bounded line `schemas/README.md` fixes.

## Applying the stack

`apply(policy, base, manifest, registry, tables, geometry)` takes one **base**
profile document and answers one produced base plus one report per row. A patch
is never an input, never opened, and never rewritten.

**Entry refusals** — the whole application could not be evaluated, so it ends in
the same structured-error channel as the grammar:

- the base's `kind` is not `base` (transforming a patch would touch the
  player's layer);
- the base's `for` pin is not `"<manifest id>@<manifest revision>"` — the
  `profile/v2` pin-equality rule, cross-referenced rather than restated;
- the policy's `id` equals the base's `id` (a produced base cannot name its own
  input: a chain directory holds one file per id);
- the base's `set` binds one action twice. `profile/v2` calls a repeated action
  id data and lets chain resolution settle it last-writer-wins, but a stack
  reads every precondition off one layout and writes into that same layout, so
  two copies of one binding are two layouts to disagree about: preconditions
  would be checked against one copy, the write would land on it, and the other
  copy would stay effective — a row reporting `Applied` over a change no player
  can feel. The refusal names the duplicated action.

**Sequential and atomic.** Rows apply in `transforms` array order; each row is
atomic, its writes landing in the working layout only if every check below
passes; a row's checks see the layout its predecessors produced. A row never
partially applies, so "never mangles a base it does not fit" is structural.

**The match rule, stated once.** A row's *match set* is the loaded registry's
ids its tag matches, sorted in id byte order. `relocate-cluster` pairs the
k-th matched id with the k-th target in row order. The produced base's `set`
list preserves the source base's element order in place. Byte order is the one
order stable across a registry round-trip, so both the pairing and the offender
a report names are fixed by the data.

**The untouchable guard** — unconditional, not author-declared: a row that
touches an action named by the manifest's `docked-actions`, or reads or writes
an input appearing in `docked-actions`, `dead-keys`, or any
`exclusive-groups` member, is NotApplicable `reserved-input` naming the input.

**Per-row checks, each landing a named verdict.**

- `relocate-cluster` — empty match set → `unmatched-tags`; a matched action
  unbound in the working layout → `unbound-action` naming it; match count ≠
  target count → `match-count`; any source or target input that is not a
  `keyboard:` input → `not-keyboard` naming it; source keys not connected by
  rook-adjacency (`|Δcol| + |Δrow| = 1`) in the loaded grid → `not-contiguous`;
  a target already bound to an action outside the match set → `occupied-target`
  naming it — every action holding that key is asked, not one answer at a time,
  since a base may bind one input to two actions and answering with the holder
  inside the match set would let the row land a matched action on a key a
  never-named action still holds. Walking onto the cluster's own vacated keys
  stays legal; finally the
  shipped placement predicate for `from`, `to`, the source→target pairs, and the
  row's bound — an illegal verdict → `placement-illegal`, carrying the
  predicate's check as the evidence. A `keyboard:` id the grid does not place or
  the tables do not grade is a fact the contract lacks: it propagates the
  predicate's structured refusal rather than reporting a verdict.
- `swap-polarity` — empty match set → `unmatched-tags`; match count ≠ 2 →
  `match-count`; any matched id whose registry axis kind is not the discrete
  lateral kind → `axis-kind` naming the id (camera look is never a key swap);
  otherwise the two actions exchange bindings, and an unbound side stays
  unbound. Applying the same row twice is the identity.
- `swap-mouse-buttons` — the actions currently bound to `mouse:left` and
  `mouse:right`; either unbound → `unbound-input` naming the mouse id;
  otherwise the two actions exchange bindings. Nothing moves on a grid, so this
  row needs no geometry and no grades.

The reason vocabulary is closed, and `NotApplicableReason`'s names are these
contract spellings: `unmatched-tags`, `unbound-action`, `match-count`,
`not-keyboard`, `not-contiguous`, `occupied-target`, `placement-illegal`,
`reserved-input`, `axis-kind`, `unbound-input`. A report carries the row's index,
its kind as spelled here, its outcome, and an offender that is always a
grammar-checked id — never raw document text — so a report stays one bounded,
truncating line like a refusal.

## The output

The produced document is an ordinary `profile/v2` **base**: `kind = "base"`,
`id = <policy id>`, `derives` empty, `for` copied from the source base, `set` =
the working layout, and `generated-by = "policy:<policy id>"`. A consumer puts
it in a chain and gates it by validation rather than trusting that a policy
wrote it; `generated-by` is provenance and is never consumed as trust.

**Regeneration is deterministic.** Same inputs ⇒ byte-identical output: no
clock, no randomness, sorted containers throughout. A consumer regenerates by
writing `<policy id>.toml` beside the chain and bumping pins as a review — after
a manifest revision bump, a patch still pinned to the old revision refuses at
load naming the link, so nothing silently re-resolves under changed game facts,
and the patch file is untouched either way.

## Input budget

One application reads at most one policy document (≤ `MAX_DOCUMENT_BYTES`) plus
the one base document the chain rules already bounded, and evaluates at most
`MAX_POLICY_TRANSFORMS` (64) rows of at most `MAX_RELOCATE_TARGETS` (16)
predicate moves each over the capped loaded tables, geometry, and registry. The
composite is stated the way `profile/v2` states its own: `64 × 16` predicate
rows out of budgets that already exist, no new aggregate cap. A policy
references no other document by path, so a per-document cap cannot be defeated
by document count.

## Fixture index

`fixtures/invalid/` is the contract for rejection behavior; each name pins one
rule above: `unknown-entry`, `major-not-shipped`, `missing-version`,
`missing-id`, `missing-transforms`, `transforms-empty`, `transforms-past-cap`,
`transform-unknown-kind`, `transform-empty-row`, `relocate-few-fields`,
`relocate-target-past-cap`, `relocate-bad-hand`, `relocate-bound-letter`,
`relocate-bound-past-scale`, `relocate-target-unknown-input`,
`swap-polarity-extra-field`, `mouse-row-carries-tag`, `tag-embedded-glob`,
`tag-uppercase`, `tag-empty-prefix`, `tag-three-segments`, `id-uppercase`,
`id-past-cap`, `forged-log-line`, `table-header`, `dotted-key`,
`multi-line-array`, `float-value`, `hex-integer`, `nesting-past-limit`.
