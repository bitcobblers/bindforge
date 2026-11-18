# profile format, v2

A profile document is one **link** of a chain: it names itself, the single link
below it, the game manifest its bindings apply to, and the delta that link
applies. `profile/v1`'s four entries are unchanged in meaning (the chain is
still `derives`, still nearest-first); what `v2` adds is the vocabulary that
makes a chain readable without guessing — `kind`, the `for` pin, the three
delta lists, and two provenance labels.

Chain semantics are ADR-014's. This spec states the per-document rules and the
load-level rules a chain is checked against, so a non-Rust implementer can
conform to the whole chain contract; the shipped reference reader checks the
load-level rules in `load_chain`, and the fixture corpus stays single-document
like every other format's.

## Grammar (the same documented subset of TOML v1.1.0)

The grammar is `profile/v1`'s, unchanged: line-oriented entries, one per line —
comments (`#`) and `key = value` pairs whose key is a TOML bare key
(`[A-Za-z0-9_-]+`) or a quoted basic string, and whose value is a basic string,
a decimal integer, a boolean, or an array of those written **on one line**.
Basic strings may use the escapes `\b`, `\t`, `\n`, `\f`, `\r`, `\"`, `\\`,
`\uXXXX`, and `\UXXXXXXXX`, and no other. In an integer, `_` may only separate
digits; there is no leading zero and no `0x`/`0o`/`0b` prefix. Table headers,
dotted keys, multi-line arrays, floats, and hex integers are legal TOML and
outside this grammar: each is rejected, and each has a fixture under
`fixtures/invalid/` pinning it.

An array element of `set`, `clear`, or `swap` is one string holding a space-
joined pair; the shipped pair encoding is the only way a mapping is written in
this format, and no id grammar contains a space, so a pair splits at its first
space unambiguously.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `profile` |
| `version` | string | yes | the grammar major this document is written against (`v2`) |
| `id` | string | yes | the link's identifier: slug grammar, at most `MAX_SLUG_OCTETS` octets — the shared bound `schemas/README.md` states once |
| `derives` | array of strings | no | ancestor chain, nearest first, default empty; each element slug grammar, at most the same shared `MAX_SLUG_OCTETS` bound. At `v2` a `patch` carries exactly one element and a `base` none |
| `kind` | string | yes | `base` or `patch`; a base stands alone, a patch modifies the link below it |
| `for` | string | yes | `"<manifest-slug>@<revision>"`: the manifest this link applies to — design 013's canonical identity, `revision` ≥ 1 |
| `set` | array of strings | no | each element `"<action-id> <input-id>"`: bind that action to that input, replacing any existing binding |
| `clear` | array of strings | no | each element one action id: drop that action's binding |
| `swap` | array of strings | no | each element `"<action-id> <action-id>"` (two distinct ids): the two actions exchange their current input bindings; an unbound side stays unbound |
| `seeded-from` | string | no | slug id of a built-in preset this chain's base was seeded from — provenance only, never consumed as trust |
| `generated-by` | string | no | 1–128 octets, no ASCII control or bidi-control characters: a label naming what produced the document (tool, policy, or `human`) — provenance only, never consumed as trust |

Any other entry is rejected. The entry set is closed on purpose, and it is
selected from the declared `version`: `v1`'s four entries are not widened, so a
`profile/v1` reader still refuses `kind`. An alias of a known entry (`derives_`,
`Derives`) would otherwise read as that entry being absent — quietly changing
what the link derives from. `fixtures/invalid/unknown-entry.toml` is the corpus
case.

### Per-link rules

Each rule below has a named fixture.

- **Base/patch shape.** A `base` has empty `derives` and no `clear` or `swap`
  element — there is nothing below it to drop or exchange — but it may carry
  `set`: a base's layout is its `set` list over the all-unbound baseline. A
  `patch` has exactly one `derives` element, its one link below (a second
  ancestor has nothing to point to), and at least one element across
  `set`/`clear`/`swap`; an empty patch is the vacuous link the guard posture
  refuses. A link whose `kind` contradicts its `derives` shape is refused here,
  at parse, and one whose `kind` contradicts its *position* in a loaded chain
  is refused at load (`load_chain`).
- **Pin.** `for` splits at its one `@`: the head is slug grammar, the tail a
  decimal integer ≥ 1 — the shipped manifest `revision-zero` refusal shape.
- **Id grammars.** `id` and each `derives` element are slug-grammar checked
  (`[a-z][a-z0-9-]*`) and capped at `MAX_SLUG_OCTETS` octets — the shared bound
  `schemas/README.md` states once (`024-B1`), because a `v2` id is a chain
  lookup key and a file name: `profile-id ↔ file-name` equality must be
  case-unique. Malformed ids refuse as `InvalidValue`, the id echoed through
  the escape-and-truncate path.
- **Delta elements.** `set` and `swap` split at the first space into exactly
  two fields; the action halves are grammar-validated by `ActionId::parse` and
  the input half by `InputId::parse` at parse, so an input outside the closed
  vocabulary refuses here, and a wrong field count refuses naming the element.
  Each of `set`, `clear`, `swap` holds at most `MAX_PATCH_DELTA_ELEMENTS` (256)
  elements.
- **Duplicates are data.** A repeated action id inside one patch — two `set`s,
  or `set` plus `clear` — is legal; the resolution rules below decide what it
  means. `set-twice-last-element.toml` and `set-then-clear-same-action.toml`
  pin it.

## Version gate

A document that declares no `version`, or a major the reader does not support,
is rejected with an error naming the supported majors — never defaulted. The
majors are the ones this **format** ships: `profile` ships `v1` and `v2`, so a
document declaring `version = "v3"` is refused naming `v1` and `v2`, and
`reach-classes/v2` never widens what a profile reader accepts.
`fixtures/invalid/missing-version.toml` and
`fixtures/invalid/major-not-shipped.toml` are the corpus cases.

## Chain guard

`resolve_chain` is `profile/v1`'s guard and stays exactly as shipped: it walks
one document's `derives` list iteratively and refuses

- a chain that derives from itself, as `Cycle`;
- a chain longer than `MAX_CHAIN_DEPTH` links (32; ADR-014 may tune the bound,
  not remove it), as `ChainTooDeep`,

before and instead of any deeper traversal. `fixtures/invalid/self-deriving-chain.toml`
is the corpus case. Chain depth is a **load-time** property of a cross-file
chain, and the cross-file boundary cases are pinned by `chain_ops` tests rather
than by a re-issued single-document fixture.

## Loading a chain

A chain is a directory of link documents. `load_chain(head)` opens the head and
walks each link's single `derives` element — one path, never a tree — and
applies these rules to the documents it opens:

1. **One directory.** Every link is read from the head's own directory: the
   path is always `<major dir>/<link id>.toml`, a direct child of the head, and
   nothing else names a location. The slug grammar refuses `..` and `/` as
   `InvalidValue` before a path is built.
2. **Major from the directory.** The head's declared major must equal the
   `vN/` directory it sits in, and that major fixes the chain's: a `profile/v2`
   chain is all-`v2`, because every link is read with that directory's grammar.
   A mismatch is `InvalidValue` naming the file, the declared major, and the
   directory — the one refusal every loader in this crate answers for that rule,
   so a chain cannot answer a second variant for the identical fault. The
   version gate's own refusal (see *Version gate*) is the one for a major the
   format does not ship at all.
3. **A missing link is a refusal, not an I/O detail.** An absent
   `<id>.toml` is `ChainLinkMissing` naming the id: a chain is a contract
   object, and "link missing" is not something a caller should have to
   distinguish from a permission error. That refusal covers only a link the walk
   constructed from a `derives` element, whose id the grammar already accepted;
   the head is the path the caller named, so an absent head is the read refusal
   whose payload carries that path. A file whose `id` disagrees with its
   file name is `InvalidValue` — a chain's identity is its documents' contents,
   not their names.
4. **The guard, over the resolved chain.** Before each descent, the same two
   refusals as the shipped guard applied to the chain resolved so far: a
   repeated id is `Cycle`, and more than `MAX_CHAIN_DEPTH` documents opened is
   `ChainTooDeep`. The guard's unit at load is documents opened, one per link.
5. **Position matches kind.** After the walk, the deepest link must carry
   `kind = base` and every other link `kind = patch`. A link contradicting its
   position is `InvalidValue` naming the link id.

`load_chain` returns the whole chain, deepest base first. Mounting at a lower
rung is data, not an operation: a consumer `load_chain`s that link as head. A
chain is never returned without its base.

### Load-time joins

`resolve(chain, manifest)` refuses, naming its offender:

- **Manifest identity.** Every link's `for` must equal the joined manifest's
  `"<id>@<revision>"` exactly — id *and* revision. A chain mid-adoption (a
  re-pinned base under patches still naming the old revision) refuses as
  `InvalidValue` naming the link, rather than silently re-resolving the same
  patch over changed game facts.
- **Action join.** Every action id in any `set`, `clear`, or `swap` must be a
  row of that manifest's `actions` table; one that is not refuses naming the
  id. Input ids are closed at parse.
- **Preset join.** A `seeded-from` id must appear in the manifest's `presets`:
  provenance must name a real thing. It is still never consumed as trust.

## Resolution — the determinism rules

Given the same chain documents and the same manifest, two runs — any
implementer, any machine — must produce the same layout. These six rules are
part of the contract, stated once here; changing one rides a future `profile/v3`.

1. Links apply deepest-first; each link's deltas apply to the layout the links
   below it produced.
2. Within a link, the lists apply in the fixed order `set` → `clear` → `swap`,
   and within each list, elements apply sequentially in array order. **Array
   order is data.**
3. `set` replaces any binding for that action (last writer wins); `clear`
   unbinds; `swap` exchanges the two actions' current bindings, unbound sides
   staying unbound.
4. An action with no binding contributes nothing to the output — the layout is
   partial by construction.
5. Output is ordered by the manifest's `actions` array order; resolution has no
   sort rule of its own.
6. An unbound `base` link is legal and resolves to an empty layout.

## Size budget and rejection messages

Documents larger than 1 MiB, or with arrays nested deeper than 16 levels, are
rejected before any value is built (`schemas/README.md`);
`fixtures/invalid/nesting-past-limit.toml` is the nesting case. Each delta list
holds at most `MAX_PATCH_DELTA_ELEMENTS` (256) elements, and a `generated-by`
label at most `MAX_GENERATED_BY_OCTETS` (128) octets.

The composite cost of one resolution is the product of the budgets that already
exist — no new aggregate cap: at most `MAX_CHAIN_DEPTH` documents opened,
totalling at most `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB), applying at
most `MAX_CHAIN_DEPTH × 3 × MAX_PATCH_DELTA_ELEMENTS` delta elements through one
per-resolution lookup of the `MAX_MANIFEST_ROWS`-capped action table.

A rejection message that quotes part of the document — a declared `format`, a
declared `version`, an entry name, an id, a label — is a single bounded line: it
escapes control characters and truncates, so it may not carry a decoded
newline into a log, may not echo an unbounded value, and may not disclose a
local filesystem path. `fixtures/invalid/forged-log-line.toml` is the corpus
case.

## Fixtures

- `fixtures/valid/` — documents this format accepts; an implementer must also be
  able to re-serialize them to a semantically equal document, each delta array
  in its document order.
- `fixtures/invalid/` — documents every implementer must reject with a
  structured error naming what is wrong.

Fixture file names are lowercase ASCII words joined by `-`, and case-unique
within a fixture directory (`schemas/README.md`). Chain-level boundary cases —
a chain of exactly `MAX_CHAIN_DEPTH` links, a repeated id across files, a link
missing from the directory — are pinned by core tests, not by fixtures: the
corpus is single-document, because no shipped fixture-corpus rule describes
multi-file chains.
