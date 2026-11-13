# Design: ADR-014 — profiles as chains of base and patch

## Context

ADR-014 ([docs/adrs/014-profiles-as-chains.md](../../adrs/014-profiles-as-chains.md))
decides that a player's layout is a **chain**: a base plus ordered patches of
deltas on top, resolved deterministically, with `diff`/`fork`/`merge` as chain
operations and provenance as a label that never vouches. The chain's skeleton
already ships and stays: `derives` nearest-first, `resolve_chain`'s iterative
walk, `MAX_CHAIN_DEPTH = 32`, `Cycle`/`ChainTooDeep` (A1, A2). What is new is
one format directory (`schemas/profile/v2/` — v1 untouched, A15), the link
*vocabulary* (`for`, `kind`, `set`/`clear`/`swap`, provenance), the
cross-file resolution machinery the shipped guard does not have (A4), and the
three chain operations — each answered below with the research's open
questions closed in Chosen design or listed under Deferred / out of scope.

## Goals / Non-goals

- Goals:
  - `schemas/profile/v2/`: `spec.md` + a valid/invalid fixture corpus; v1 and
    its fixtures untouched (A15, A3). Chain depth is a load-time property,
    pinned by `chain_ops` tests rather than a re-issued fixture (A16).
  - A closed v2 entry set: v1's four entries unchanged in meaning
    (`format`, `version`, `id`, `derives`), plus `for`, `kind`, `set`,
    `clear`, `swap`, `seeded-from`, `generated-by` (A1, A4).
  - Cross-file chain loading with the same guard rules the shipped single-
    document guard states, plus a named refusal for a missing link (A2, A18).
  - A deterministic resolution — links deepest-first, elements in array
    order, fixed within-patch list order — pinned as a compatibility surface
    (A10, A11).
  - `fork`, `diff`, `merge` as core operations over documents, with a named
    merge-conflict refusal.
  - Provenance entries validated referentially (they must name real things)
    and consumed by nothing (ADR Decision Four).
- Non-goals: the CLI surface for chain operations and personal-chain storage
  (ADR-017); transforms and the bases they produce (ADR-015); seed presets as
  chain bases (ADR-016) and reading a game's shipped config (ADR-017); hub
  upload bundling and transport (ADR-018); widening the closed input
  vocabulary (a `reach-classes` major, A11). Everything else unanswered is
  written once, with its owner, under Deferred / out of scope.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | `profile/v1` fixes exactly four entries (`format`, `version`, `id`, `derives`), `derives` nearest-first and default empty, the entry set closed on purpose; `resolve_chain` walks iteratively and refuses `Cycle`/`ChainTooDeep` before deeper traversal, `MAX_CHAIN_DEPTH = 32` | verified | `schemas/profile/v1/spec.md:23-34,48-58`; `crates/core/src/chain.rs:15-42` |
| A2 | The shipped guard validates one document's `derives` array only; nothing resolves an ancestor id into another document, so a two-file cycle is invisible to it, and no shipped code bounds a resolved chain across files | verified | `crates/core/src/chain.rs:26-42` |
| A3 | The v1 chain boundaries ship as invalid fixtures (`self-deriving-chain.toml`, `chain-past-max-depth.toml`); the harness asserts the full parse→`serialize`→re-parse round-trip for profiles; the zero-fixture rule and the lowercase-`-`-joined case-unique fixture-name rule bind a new version directory | verified | `schemas/profile/v1/fixtures/invalid/`; `crates/core/tests/conformance.rs:67-102`; `schemas/README.md:40-44` |
| A4 | The version gate is per-format: `format_majors` gives `profile` only `[v1]` while `supported_majors()` already declares `["v1","v2"]`, so `profile/v2` changes one `format_majors` line plus the entry-set selection, and the majors-equality invariant stays green untouched; the entry-set arm keyed on the declared version is the shipped `reach-classes` mechanism | verified | `crates/core/src/document.rs:139-156,269-277,325-330`; `crates/core/tests/version_gate.rs:14-25` |
| A5 | `check_dir_version` refuses a document whose declared major differs from the `vN/` directory it loads from | verified | `crates/core/src/document.rs:787-802` |
| A6 | The parse budgets are `MAX_DOCUMENT_BYTES` = 1 MiB (checked before the read; streams take at most one byte over) and `MAX_NESTING_DEPTH` = 16; nothing else bounds array sizes | verified | `crates/core/src/toml.rs:36-39`; `schemas/README.md` |
| A7 | The manifest grammar ships `id` (slug) plus an integer `revision` ≥ 1, pinned by `revision-zero.toml`, and design 013 canonicalized the pin spelling `"<id>@<revision>"` | verified | `schemas/manifests/v1/fixtures/invalid/revision-zero.toml`; design 013 → Identity (`docs/designs/013-game-manifests/02-proposal.md:337-344`) |
| A8 | The contract's shipped encoding for a mapping inside a one-line string array is the space-joined pair, no id grammar carries a space, and wrong field counts are pinned by invalid fixtures | verified | `crates/core/src/input.rs:20,64`; `crates/core/src/document.rs:649-655`; `schemas/keyboard-geometry/v1/fixtures/invalid/pair-two-fields.toml` |
| A9 | The input vocabulary is closed (`keyboard:` 48 members, `mouse:`/`gamepad:` fixed sets); an id outside a set is a refusal — a v2 `set` still cannot name Tab, Enter, Esc, arrows, or F-keys | verified | `crates/core/src/input.rs:70-82`; `schemas/reach-classes/v2/spec.md` |
| A10 | ADR-015: transforms apply only to bases; "a personal patch (014) sits above them in the chain and survives base regeneration untouched" — deterministic re-resolution over a changed base is load-bearing | verified | `docs/adrs/015-transforms-as-preconditioned-policies.md:57-59` |
| A11 | ADR-014's Decision text fixes the v2 entry list, deterministic resolution, the three chain operations, and provenance-as-label; its Alternatives retire `based-on`, flat profiles, snapshot profiles, and provenance-as-trust; its "not decided" list retires transforms, CLI, and storage location to 015/017 | verified | `docs/adrs/014-profiles-as-chains.md:40-102` |
| A12 | Every file under `data/profiles/` arrived from elsewhere and is untrusted input under the version gate, parse budget, and chain guards; the README names ADR-014 as the directory's owner | verified | `data/profiles/README.md` |
| A13 | Every refusal is a typed `CoreError`; document strings echoed in a refusal pass `one_line` (escape + truncate at 80 chars); a local path never enters the message (`source_path` is the logging channel); `Io` carries the path | verified | `crates/core/src/error.rs:1-77,149-169` |
| A14 | The slug grammar `[a-z][a-z0-9-]*` ships as `placement::is_slug_name` (`pub(crate)`) and is already shared by manifest, codec, and preset ids | verified | `crates/core/src/placement.rs`; `crates/core/src/document.rs:105,167-168` |
| A15 | v1 is untouched: its entry set, its `serialize` bytes, `resolve_chain`, and its fixtures stay exactly as shipped; widening `profile/v1` was rejected by the ADR | verified | `crates/core/src/document.rs:69,360-372`; `docs/adrs/014-profiles-as-chains.md:71-74` |
| A16 | The conformance harness parses single documents; there is no shipped fixture-corpus rule for multi-file chains — cross-file rules live at load and are pinned by module tests (the 013 precedent keeps the corpus join-free) | verified | `crates/core/tests/conformance.rs:29-37`; `crates/core/src/manifest.rs:25` |
| A17 | Action ids are grammar-validated at parse by `ActionId::parse`, reused by the manifests arm; the manifest's `actions` table is the per-game action set, joined at load | verified | `crates/core/src/manifest.rs:221`; design 013 → Load-time validation |
| A18 | Today a missing ancestor file is indistinguishable from an I/O failure: `CoreError` has no chain-completeness variant | verified | `crates/core/src/error.rs:26-63` |
| A19 | The grammar refuses table headers and dotted keys with pinned fixtures; delta lists must stay one-line string arrays to remain in-grammar | verified | `schemas/profile/v1/spec.md:8-21`; `schemas/profile/v1/fixtures/invalid/table-header.toml`, `dotted-key.toml` |
| A20 | The research's standards convergences: RFC 6902 applies operations sequentially in array order and has no `swap` (a `move` chain has an intermediate state); RFC 7396 obsoleted RFC 7386 a month after publication; systemd pins merge order as contract; git rebase stops visibly on a patch that no longer fits its base; TF2 shows per-file caps evaded once files reference files; Valve prices a forced base re-adoption with per-population flags and "prompt rather than force" | unverified | 01-research.md (Findings, Prior art, Security lens) |
| A21 | The composite chain cost today is bounded only as a product: up to 32 links × 1 MiB per resolution, and that product is stated nowhere | verified | `crates/core/src/toml.rs:36`; `crates/core/src/chain.rs:18` |

## Options considered

**Where cross-file resolution lives: widen `resolve_chain`, or a new
`load_chain`.** Widening the shipped function changes the meaning of what its
fixtures and `chain_guard.rs` pin (one document's `derives` list) and couples
the guard to the filesystem. Chosen: `resolve_chain` stays byte-exact; a new
`load_chain` in a new module applies the same two refusal rules — iterative
walk, refuse before descending — to the *resolved* chain (A2, A15).

**Where ancestor documents come from: a search path, the hub, or the head's
own directory.** A search path invents precedence nobody pinned; the hub is
ADR-018's, and storage location is explicitly undecided (A11). Chosen: the
same-major-directory rule — every link is `<profile-id>.toml` directly in the
major directory of the head document. It reuses `check_dir_version` (A5),
makes a hub upload bundle exactly one such directory, and closes the
mixed-major question by construction: one directory, one grammar.

**May a v2 chain include a v1 link: default its `kind` to `base`, or refuse.**
Defaulting invents semantics v1 never stated (a v1 profile has no layout
content at all — A1) and makes the grammar of one chain depend on which file
answers. Chosen: refuse — a chain is single-major by the directory rule; a
v1 document is carried forward by re-issuing it as a v2 `base`, which v2's
`set` makes expressible.

**What `kind` means: a free label, position-only inference, or a label with
positional rules.** The ADR leaves the `kind`/`derives` contradiction
representable (a `patch` with no `derives`, a `base` mid-chain) — research
flags it; the contract's pattern is a refusal naming the contradiction.
Chosen: `kind` stays a required label, and the label is checked against
`derives` per document at parse and against chain position at load. Inference
alone would strand the label the ADR names.

**The delta encoding: space-joined pairs, quoting, or a separator.** Tables
are out — the grammar refuses them (A19). A dedicated separator invents a
character ids may carry; quoting interacts with the closed escape table with
no fixture precedent. Chosen: the shipped pair encoding (A8) — `set` and
`swap` elements are two space-joined ids, `clear` one — sound because no
input or action id carries a space (A8).

**What `swap` exchanges: two actions' bindings, two inputs' bindings, or
spelled pair types.** Two inputs' bindings is expressible today as two `set`
elements, so omitting it loses nothing; spelled pair types multiply the
grammar. Chosen: `swap` is always action↔action — the two named actions
exchange their current input bindings, an unbound side stays unbound —
keeping every delta list's subject an action id. RFC 6902 has no `swap` at
all, and its `move` chain forces every implementer to agree on an
intermediate state (A20); an exchange primitive avoids that whole class.

**Within-patch collisions: refuse, or last-write-wins over array order.**
Refusing strands meaningful documents — `set X a` then `clear X`, or a
retune `set X a` then `set X b`, are how a patch is *edited*. Chosen: the
RFC 6902 convention (A20): elements apply sequentially in array order, last
write wins, and the fixed list order `set` → `clear` → `swap` applies per
patch.
Array order is data, pinned by valid fixtures and stated in the determinism
rules.

**`for` disagreement at resolution: refuse, warn, or a resolver option.**
A warning makes resolution result-plus-side-effect, and a resolver option
means two run settings can disagree about the same chain — determinism is a
compatibility surface (A11). Chosen: refuse at load, naming the link, with
pin equality defined once in Chosen design; Steam priced the silent-drift
alternative with flags and prompts (A20), and a named refusal is the
stop-visibly answer (A20).

**`merge`'s conflict rule: git's stop-with-markers, RFC 6902's abort, or
last-patch-wins.** Conflict markers are an artifact outside the grammar;
silent last-patch-wins guesses at what "the same patch means" over a moved
base, the exact damage the ADR prices (A11). Chosen: refuse the whole merge
with a typed error naming the colliding action id; a conflict is representable
iff the composed effect cannot be written as one patch (Chosen design →
fork/diff/merge).

**Provenance validation: unchecked labels, or referential joins.** Unchecked
labels let `seeded-from` name nothing, which is the second-answer-to-one-fact
drift the contract refuses everywhere else. Chosen: parse checks the grammar,
load checks the referent exists in the joined manifest — while nothing ever
consumes the value to *trust* a document (ADR Decision Four).

**Profile ids: keep v1's "not empty", or tighten to the slug grammar.** v2 ids
are chain lookups and path components (the directory rule) and feed case-
insensitive contributor filesystems; a v1 id like `My Profile` cannot become a
v2 file name. Chosen: tighten `id` and every `derives` element to the shipped
slug grammar (A14) at v2 parse; v1 stays as shipped (A15).

**Composite chain budget: a new total cap, or state the product.** A new
aggregate constant duplicates the two bounds that already multiply out
(A6, A21) and would need its own tuning history. Chosen: state the composite
as `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` in the spec, and name one new cap,
`MAX_PATCH_DELTA_ELEMENTS = 256` per delta list, mirroring the shipped
`MAX_MANIFEST_ROWS = 256` that bounds one game's whole action table (A17).

## Chosen design

### Delta inventory

New: `schemas/profile/v2/` (`spec.md` + `fixtures/valid/` +
`fixtures/invalid/`, no shipped profile data — see Rollout 6),
`crates/core/src/chain_ops.rs` (cross-file loading, resolution, chain
operations), and `crates/core/tests/chain_ops.rs` (module-mirrored per
ADR-004).

Edited: `crates/core/src/document.rs` (`PROFILE_ENTRIES` renamed to
`PROFILE_V1_ENTRIES`, new `PROFILE_V2_ENTRIES`, a `profile_entries(version)`
selector beside `reach_classes_entries`, the v2 branch of `parse_profile`, a
`serialize` branch by version, the two budget constants),
`crates/core/src/chain.rs` (doc-header pointer only — its code is untouched,
A15), `crates/core/src/error.rs` (one new variant, `ChainLinkMissing`, and
its `Display` arm), `crates/core/src/lib.rs` (module, re-exports,
public-surface doc), `crates/core/tests/grammar_closure.rs` (the v2
entry-set and alias cases), `crates/core/tests/version_gate.rs` (a
`profile/v3` refusal naming `profile`'s majors; the majors-equality case
stays green untouched, A4), `crates/core/tests/conformance.rs` (the expected-
directory list gains `profile/v2`; the *walk* stays unchanged),
`crates/core/tests/zz_api_probe.rs` (the new public names),
`schemas/README.md` (the series sentence, the new budget constants),
`data/profiles/README.md` (the loader exists).

Unchanged: `toml.rs`, `chain.rs`'s code, `action.rs`, `input.rs`, `reach.rs`,
`placement.rs`'s predicate, `manifest.rs`, every existing fixture and shipped
format including all of `profile/v1`, `Cargo.toml`, `Cargo.lock`, both
workflows.

### The contract: `schemas/profile/v2/`

```
schemas/profile/v2/
  spec.md
  fixtures/valid/    minimal-base.toml  seeded-base.toml  retune-patch.toml
                     full-patch.toml  set-then-clear-same-action.toml
                     set-twice-last-element.toml  swap-pair-applied-twice.toml
  fixtures/invalid/  unknown-entry.toml  major-not-shipped.toml  missing-version.toml
                     missing-kind.toml  missing-for.toml  missing-derives-patch.toml
                     kind-outside-vocabulary.toml  base-derives-nonempty.toml
                     base-clear-element.toml  base-swap-element.toml
                     patch-empty-deltas.toml  for-without-at.toml
                     for-revision-zero.toml  for-uppercase-id.toml  id-uppercase.toml
                     id-past-cap.toml  derives-element-uppercase.toml  derives-two-elements.toml
                     set-one-field.toml  set-three-fields.toml
                     set-unknown-input.toml  clear-two-fields.toml
                     swap-one-field.toml  swap-three-fields.toml
                     swap-pair-same-action.toml  seeded-from-uppercase.toml
                     generated-by-past-cap.toml  generated-by-control-char.toml
                     generated-by-bidi-char.toml  self-deriving-chain.toml  forged-log-line.toml
                     table-header.toml  dotted-key.toml  multi-line-array.toml
                     float-value.toml  hex-integer.toml  nesting-past-limit.toml
```

The grammar is the same documented TOML subset (A19); the entry set is
closed, and the set is selected from the declared `version` by the shipped
mechanism (A4). The `spec.md` also states the load-level rules below (the
directory rule, pin equality, the joins) so a non-Rust implementer can
conform to the whole chain contract, while the fixture corpus stays
single-document like every other format's — the join-level rules are pinned
by core tests, the 013 precedent (A16).

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `profile` |
| `version` | string | yes | the grammar major this document is written against (`v2`) |
| `id` | string | yes | the profile's identifier: slug grammar (A14), ≤ 63 octets — v1's "not empty" is tightened because a v2 id is a chain lookup key and a file name |
| `derives` | array of strings | no | ancestor chain, nearest first, default empty — v1's entry unchanged in meaning (A1); each element slug grammar, ≤ 63 octets; at v2 a `patch` carries exactly one element and a `base` none |
| `kind` | string | yes | `base` or `patch`; a base stands alone, a patch modifies the link below it |
| `for` | string | yes | `"<manifest-slug>@<revision>"`: the manifest this link applies to, exactly design 013's canonical identity (A7); `revision` ≥ 1 |
| `set` | array of strings | no | each element `"<action-id> <input-id>"`: bind that action to that input, replacing any existing binding |
| `clear` | array of strings | no | each element one action id: drop that action's binding |
| `swap` | array of strings | no | each element `"<action-id> <action-id>"` (two distinct ids): the two actions exchange their current input bindings; an unbound side stays unbound |
| `seeded-from` | string | no | slug id of a built-in preset this chain's base was seeded from (manifest `presets`, A7) — provenance only, never consumed as trust |
| `generated-by` | string | no | 1–128 octets, no ASCII control, default-ignorable, or bidi-control characters: a label naming what produced the document (tool, policy, or `human`) — provenance only, never consumed as trust |

Per-link rules, each with a named fixture above:

- **Base/patch shape.** A `base` has empty `derives` and no `clear`/`swap`
  elements (nothing to drop or exchange below it) but may carry `set` — a
  base's layout is its `set` list over the all-unbound baseline. A `patch`
  has exactly one `derives` element — its one link below; a second ancestor
  has nothing to point to (`derives-two-elements.toml`) — and at least one
  element across `set`/`clear`/`swap`; an empty patch is the vacuous link
  the guard posture refuses.
- **Pin.** `for` splits at its one `@`: the head must be slug grammar (A14),
  the tail a decimal integer ≥ 1 — the shipped `revision-zero` refusal shape
  (A7).
- **Id grammars.** `id` and each `derives` element are slug-grammar checked
  and capped at 63 octets; malformed ids refuse as `InvalidValue` echoing the
  id through the escape-and-truncate path (A13).
- **Delta elements.** `set`/`swap` split at the first space into exactly two
  fields; the action halves are grammar-validated by `ActionId::parse` and
  the input half by `InputId::parse` at parse (A8, A17) — an input outside
  the closed vocabulary refuses here (A9); wrong field counts refuse naming
  the element. Each of `set`, `clear`, `swap` holds at most
  `MAX_PATCH_DELTA_ELEMENTS` (256) elements — one patch can retouch one
  game's whole action table, the cap the manifest row cap already prices
  (A17).
- **Duplicates are data.** A repeated action id inside one patch (two `set`s,
  `set` + `clear`) is legal; the resolution rules decide what it means
  (below), and `set-twice-last-element.toml` / `set-then-clear-same-action.toml`
  pin it.

Example — what a first chain looks like (not shipped this run):

```toml
# schemas/profile/v2/fixtures/valid/retune-patch.toml
format = "profile"
version = "v2"
id = "mike-remnant-retune"
derives = ["remnant-2-default"]
kind = "patch"
for = "remnant-2@3"
set = ["move.steer-left keyboard:d"]
clear = ["aim.zoom"]
swap = ["combat.dodge-left combat.dodge-right"]
generated-by = "human"
```

(The action ids shown are registry-shaped, not registry members — the first
real chain lands with its registry and manifest rows in the same PR, as
design 013's growth order states.)

### The model: `document.rs` and the v2 parse/serialize arms

`Profile` (A1) gains the v2 fields and a `ProfileKind` enum; a v1-parsed
`Profile` keeps them at defaults and its `serialize` output stays byte-exact
(A15) — the writer branches on `version` alone, exactly as `reach-classes`
grades do:

```rust
pub enum ProfileKind { Base, Patch }

pub struct Profile {
    pub version: String,
    pub id: String,
    pub derives: Vec<String>,
    pub kind: Option<ProfileKind>,          // v2: required; v1: None
    pub for_manifest: String,               // v2: "<id>@<rev>", grammar-checked at parse; v1: ""
    pub set: Vec<(ActionId, InputId)>,
    pub clear: Vec<ActionId>,
    pub swap: Vec<(ActionId, ActionId)>,
    pub seeded_from: Option<String>,
    pub generated_by: Option<String>,
}
```

`parse`'s `profile` arm picks `PROFILE_V1_ENTRIES` or `PROFILE_V2_ENTRIES`
from the declared version through a `profile_entries(version)` selector
beside `reach_classes_entries` (A4), and the refusal names the declared
version. `serialize` writes the eleven entries in the fixed order
`format, version, id, [derives], [kind, for], [set, clear, swap],
[seeded-from], [generated-by]` — v2-only entries written for v2 documents
only; delta arrays written unconditionally for v2 (an absent one round-trips
as `[]`, the 013 precedent), each array's **document order preserved**, since
delta order is data and the harness's re-parse comparison is equality
(A3).

No `CoreError` variant is removed or changed; one is added —
`ChainLinkMissing { id: String }`, the completeness failure no existing
variant names (A18) — with a `Display` line echoing the id through
`one_line` (A13). Every other v2 refusal is `InvalidValue` or `MissingEntry`
(the 012/013 error-shape precedent).

### Cross-file loading: `load_chain` in `chain_ops.rs`

```rust
pub struct Chain { links: Vec<Profile> }  // only load_chain builds one; deepest base first … head last

pub fn load_chain(head: &Path) -> Result<Chain, CoreError>;
```

Iterative, no recursion, and each step applies the shipped guards to what it
has loaded so far:

1. `parse_file(head)`; `check_dir_version` pins the head's declared major
   against its `vN/` directory (A5). The directory segment also fixes the
   chain's major: a `profile/v2` chain is all-v2, because every link is read
   with the grammar of that directory (the mixed-major question, closed).
2. Walk each link's single `derives` element — a v2 chain is one path, not
   a tree — looking up the ancestor `<id>.toml` *directly in that same
   directory*: the constructed path is always a direct child of the head,
   `<major dir>/<parse-validated slug>.toml`, and no other entry names a
   location (the slug grammar refuses `..` and `/` as `InvalidValue`).
   An absent file is `ChainLinkMissing` naming the id (a chain is a contract
   object, and "link missing" is not an I/O detail — A18); a file whose
   `id` disagrees with its name is `InvalidValue` — the location rule,
   mirroring `load_manifest`'s id↔directory equality, which pins the
   lookup but not the value: a chain's identity is its documents' contents.
3. Before each descent, the same two refusals as the shipped guard, now over
   the *resolved* chain: a repeated id is `Cycle`, and more than
   `MAX_CHAIN_DEPTH` documents opened is `ChainTooDeep` — the guard's unit
   is documents opened, one per link under step 2; both refuse before any
   deeper traversal (A1, A2). `resolve_chain`
   itself is untouched: its single-document fixtures stay the contract for
   what it pins (A15).
4. After the walk: the deepest link must carry `kind = base` and every other
   link `kind = patch`; a link contradicting its position is `InvalidValue`
   naming the link id — the `kind`/`derives` contradiction rule the research
   says the ADR left open.

A hub upload bundle is exactly such a directory (ADR-018 ships the
transport; this is the shape it transports — recorded in Deferred). Every
document `load_chain` opens — fixture, `data/profiles/`, hub bundle, or
personal chain directory — parses under one version gate, byte budget, and
chain guards, source-independent; trust stays with adoption (Decision
Four). Where a player's personal chain lives on disk stays undecided
(A11); decided here is only *how a chain resolves once a head is given*.

### Resolution: deterministic, and pinned as a compatibility surface

```rust
pub struct ResolvedBinding { pub action: ActionId, pub input: InputId }

pub fn resolve(chain: &Chain, manifest: &Manifest) -> Result<Vec<ResolvedBinding>, CoreError>;
```

**The determinism rules — stated once, cross-referenced everywhere else**
(Options, Risks, and Test strategy cite this list, A11):

1. Links apply deepest-first; each link's deltas apply to the layout the
   links below it produced.
2. Within a link, the lists apply in the fixed order `set` → `clear` →
   `swap`, and within each list, elements apply sequentially in array order;
   array order is data.
3. `set` replaces any binding for that action (last writer wins); `clear`
   unbinds; `swap` exchanges the two actions' current bindings, unbound
   sides staying unbound.
4. An action with no binding contributes nothing to the output — the layout
   is partial by construction.
5. Output is ordered by the manifest's `actions` array order — no sort rule
   of its own (the "row order is data" precedent, A17).
6. An unbound `base` link is legal and resolves to an empty layout; seed
   presets as real bases are ADR-016/017's data (Deferred).

Given the same chain documents and the same manifest, two runs — any
implementer, any machine — must agree; these six rules are part of the
contract from merge on, and changing one rides a future `profile/v3`.

**Load-time joins** (`resolve`'s refusals, each naming its offender):

- **Manifest identity:** every link's `for` must equal the joined manifest's
  `"<id>@<revision>"` exactly — id *and* revision. A chain mid-adoption
  (a re-pinned base under patches that still name the old revision) refuses
  with `InvalidValue` naming the link, rather than silently re-resolving
  "the same" patch over changed game facts — the stop-visibly rule the
  prior art converges on (A20), and the ADR's named cost priced with a
  refusal instead of Steam's flags-and-prompts. Adoption is a review: the
  PR that bumps a base bumps its chain's pins, and the diff shows exactly
  what re-resolves (the CI re-validation matrix that automates this stays
  deferred with its 013 owner).
- **Action join:** every action id in any `set`/`clear`/`swap` must be a row
  of that manifest's `actions` table (A17); one that is not refuses naming
  the id. Input ids are closed at parse (A9).
- **Preset join:** a `seeded-from` id must appear in the manifest's
  `presets` (A7) — provenance must name a real thing; it is still never
  consumed as trust (Decision Four).

The composite cost is bounded and now stated (A21): one resolution opens
at most `MAX_CHAIN_DEPTH` documents — one per link — totalling at most
`MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB), and applies at most
`MAX_CHAIN_DEPTH × 3 × MAX_PATCH_DELTA_ELEMENTS` delta elements through
one per-resolution lookup of the `MAX_MANIFEST_ROWS`-capped action table —
the per-document budgets plus the link bound, with no new aggregate cap.

**"Mount at any rung" is data, not an operation:** `load_chain` returns the
whole chain of documents; a consumer mounts a prefix by `load_chain`ing a
lower link as head, or reads the documents singly with `parse_file` — never
a suffix without its base, which `load_chain` never returns.
ADR-015's rule — a personal patch survives base regeneration (A10) — holds
because resolution is the only thing that consumes deltas, never an edit to
a resolved layout.

### Chain operations: `fork`, `diff`, `merge`

```rust
pub fn fork(chain: &Chain, id: &str) -> Result<Profile, CoreError>;
pub fn diff(a: &Chain, b: &Chain, manifest: &Manifest, id: &str) -> Result<Profile, CoreError>;
pub fn merge(p1: &Profile, p2: &Profile, id: &str) -> Result<Profile, CoreError>;
```

- **`fork`** builds the appended patch: fresh caller-supplied `id` (slug
  grammar, and refused if it collides with any link id in the chain),
  `derives = [head id]`, `kind = patch`, `for` inherited from the head so
  the forked chain is loadable against the same manifest, and empty delta
  lists — a shell that `load_chain` refuses as `patch-empty-deltas` until the
  caller fills one element. Copying the chain's *files* is the consumer's
  move — core builds documents, not filesystems (storage is ADR-017's).
- **`diff`** requires the two chains' deepest links to be the *same
  document* — equal parsed `Profile`s, ids being unique only within one
  directory — and refuses naming both heads otherwise; chains resolving to
  equal layouts refuse too (no legal non-empty output). Its output is a
  v2 patch document (the diff *is* the grammar): the caller's `id`
  (`fork`'s collision rule against every link id of `a`), `derives = [a's
  head id]`, `for` the joined pin. Appended to `a`, it resolves to
  `b`'s layout — a `set` per differing action, a `clear` per action `b`
  unbinds — pinned by a corpus test.
- **`merge`** takes two patches that both name the same `derives` base id
  and the same `for` (or refuses naming the mismatch — the pin equality
  rule above, cross-referenced); the output carries the caller's `id`
  (`fork`'s collision rule against `p1` and `p2`) and inherits `derives`
  and `for` from `p1`. It composes the two delta lists in chain
  order (`p1` below, `p2` above) over every action either touches — exactly, not heuristically — and
  emits one patch producing that composition on the base. When some action's
  composed effect has no one-patch form under the fixed list order — lower
  `swap X Y` with both bound in the base, upper `clear X`: the net needs
  the `swap` before the `clear` — `merge` refuses the whole result with `InvalidValue`
  naming the colliding action id (Options: no markers, no guessing, A20);
  a composition that cancels to the identity refuses as the empty-patch case.
  Compatible overlaps are not conflicts: equal sets emit one element,
  equal clears one `clear`, a `set` under a `clear` composes to the lone
  `clear`. Output element order is deterministic — first-patch touch
  order, then second-patch-only actions.

### Provenance labels

`seeded-from` and `generated-by` are grammar-checked at parse (slug /
bounded label), joined referentially at load (the preset must exist, A7),
and consumed by nothing that grants, ranks, or ships anything — validation
(013) and explicit adoption remain the only trust mechanisms, exactly
Decision Four. The label is a public, shared artifact — never secret, never
displayed as authority — and reaches logs only through `one_line`, the same
escape-and-truncate path whenever a refusal echoes it (A13).

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Cross-file machinery is new attack surface (a hostile chain of real documents) | Med | High | The resolved-chain walk reuses `Cycle`/`ChainTooDeep`/`MAX_CHAIN_DEPTH` unchanged, opens at most `MAX_CHAIN_DEPTH` documents (one path, one ancestor per patch), refuses `ChainLinkMissing` before descending, and the composite ≤ 32 documents / 32 MiB is stated and bounded (A12, A18, A21) |
| A re-pinned base silently changes what "the same" patch means (the ADR's named cost) | Med | High | Pin equality at load refuses and names the link — adoption is an explicit, reviewable edit; the automated CI matrix stays deferred with its 013 owner, so *partial* staleness surfaces at load, not at review |
| Last-write-wins makes a misplaced element silent | Med | Med | Element order is stated as data in the determinism rules with two valid fixtures pinning it; `diff` exposes the net effect of any patch chain at review time |
| Determinism freezes: the six rules can never change in place | High | Med | That is the ADR's decision restated; the rules are enumerated once, fixture-pinned, and any change rides `profile/v3` with v2's corpus intact (A3, A4) |
| Implementers diverge on `swap` intermediate states | Low | Med | `swap` is one exchange over named actions (no `move` chain, A20); its semantics are one entry-table row plus fixtures |
| Delta caps are the wrong size (256 per list too tight for a huge game) | Low | Low | The cap mirrors the manifest row cap that bounds one game's whole action table (A17); raising it is an additive v2 rule change — data, not a grammar break |
| Slug-tightened ids strand v1 ids on migration | Med | Low | v1 has no shipped data (A12); when it gains any, id re-assignment is part of the v1→v2 re-issue step in Rollout |
| The closed vocabulary cannot name Tab/Enter/arrows in a `set` (A9) | High | Low–Med | Same gap 013 carried; widening rides the `reach-classes` vocabulary major (012's mechanism) — recorded in Deferred |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux CI runner (`check.yml` `rust` job) | yes | The v2 corpus and `chain_ops` tests run under the three existing gate commands; no CI change this run — the pin re-validation matrix stays deferred (A4) |
| Windows 10/11 contributor | yes | v2 ids and pin slugs are lowercase-ASCII slugs, so profile-id↔file-name equality is case-unique; the case-unique fixture-name rule already applies (A3, A14) |
| macOS contributor | no (v1) | Design 008's matrix; nothing here adds a macOS-specific requirement |
| Toolchain floor (1.85.0, edition 2024) | yes | std-only, no new dependency, `Cargo.lock` byte-unchanged |
| Pre-014 core reading a v2 document | yes (refuses cleanly) | The per-format gate refuses `version = "v2"` naming `profile`'s then-majors (`v1` alone) — fail-closed, never reinterpreted (A4) |
| Existing v1 documents, fixtures, `resolve_chain` | yes (untouched) | v1's entry set, `serialize` bytes, chain guard, and every v1 fixture stay byte-exact (A15) |
| Third-party reader, non-Rust | yes (data only) | `profile/v2/spec.md` + fixtures are the contract, including the load-level rules stated in the spec; no language runner contract this stage |
| Product artifacts (CLI, desktop, hub) | not applicable | This stage ships no binary; the chain-operations CLI is ADR-017, hub bundling/transport ADR-018 |

## Test strategy

1. Unit (`crates/core/tests/chain_ops.rs`, grammar cases) — the v2 entry
   set: a document using exactly the defined entries parses; each invalid
   fixture above is refused naming the entry or element — `kind` missing or
   outside `{base, patch}`, a `base` with `derives`/`clear`/`swap`, a
   `patch` with empty `derives`, a second `derives` element
   (`derives-two-elements.toml`), or no delta elements, `for` with no `@`,
   `@0`, or an uppercase id half, `set`/`clear`/`swap` wrong field counts,
   an `input` id outside the closed vocabulary (A9), a `swap` pair naming one
   action twice, an uppercase, 64-octet, `..`- or `a/b`-shaped `id` or
   `derives` element (the slug refusal, named `InvalidValue`),
   `seeded-from` malformed, `generated-by` past 128 octets or control-
   character-bearing, and a 257th element in any delta list.
2. Unit — resolution semantics as one table (the determinism rules): the
   same chain resolves identically across repeated runs; last-element-wins
   over repeated `set`s; `set` → `clear` unbinds; `swap` exchanges and an
   unbound side stays unbound; a `base` with `clear` refuses at load naming
   the link; a chain whose deepest link is a `patch` refuses; a `base`
   mid-chain refuses naming it; output order equals the manifest's
   `actions` row order (A17).
3. Unit — `load_chain` guards: a two-file cycle (A derives B, B derives A)
   refuses as `Cycle`; a chain of exactly `MAX_CHAIN_DEPTH` opened links
   resolves and one more refuses as `ChainTooDeep` — the sole depth pin,
   counting documents opened; a missing ancestor refuses as
   `ChainLinkMissing` naming the id; a file whose `id` disagrees with its
   name refuses; a v2 head derives nothing that resolves outside its own
   major directory (A5).
4. Unit — load-time joins: a link's `for` off by id *or* revision from the
   joined manifest refuses naming the link; a delta naming an action that
   is not a manifest row refuses naming the id; a `seeded-from` not in the
   manifest's `presets` refuses naming it.
5. Unit — chain operations: `fork` derives the head, inherits the head's
   `for`, and refuses a fresh id that collides with a chain link or breaks
   the slug grammar, its empty-delta shell refusing to load until one delta
   is added; `diff` round-trips over the corpus pairs — appending its output
   to `a` is a file-level move — write it beside `a`'s head, `load_chain` it as
   the new head — resolving to `resolve(b, manifest)`; chains whose deepest
   link differs, by id or by contents, and chains with equal resolved
   layouts both refuse naming both heads; `merge` of compatible patches emits the
   deterministic merged patch (equal-sets emit once), and a
   non-representable pair (a `swap X Y` under a `clear X`) refuses the
   whole merge naming the colliding action id; an identity-cancelling
   composition refuses as the empty-patch case.
6. Unit — round-trip: `parse(serialize(parse(text)))` equals the first parse
   for every v2 valid fixture and for a hand-built document carrying all
   eleven entries, delta array order included (delta order is data, A3); a
   v1 `Profile` serializes byte-for-byte as before (A15).
7. Integration (`crates/core/tests/conformance.rs`) — the unchanged
   `schemas/*/v*/` walk picks up `profile/v2`: every valid fixture parses,
   passes the guards, and re-serializes to a semantically equal value
   (profiles keep the round-trip path); every invalid fixture is refused
   with a single-line, control-free, truncating message — `forged-log-line.toml`
   pins that for newline-bearing values; the zero-fixture
   rule and the case-unique fixture-name rule cover the new directory (A3).
8. Integration — the untouched halves stay green: every pre-014 fixture,
   loader test, and refusal byte-unchanged; `version_gate.rs` gains the case
   that a `profile` document declaring `v3` is refused naming `v1, v2` while
   the majors-equality invariant needs no change (A4); `grammar_closure.rs`
   refuses a `profile/v2` document aliasing `derives_`; `zz_api_probe.rs`
   names the new public exports.
9. Platform — the three gate commands (`fmt`, `clippy -D warnings`,
   `test --workspace --locked`) are green on the Linux CI runner (the merge
   PR's job is the leg), and the suite runs once on a Windows contributor
   machine, where id↔file-name case uniqueness is what would bite.

## Rollout

1. Add `schemas/profile/v2/spec.md`: the grammar and entry table, the
   per-link rules, the determinism rules, the directory/pin/load rules, the
   composite budget statement, and the fixture-name discipline — then the
   fixtures listed above. The corpus is what makes `profile/v2` "supported"
   (`schemas/README.md`).
2. Edit `document.rs`: rename `PROFILE_ENTRIES` → `PROFILE_V1_ENTRIES`, add
   `PROFILE_V2_ENTRIES`, the `profile_entries(version)` selector, the v2
   parse branch, the version-branched `serialize`, and
   `MAX_PATCH_DELTA_ELEMENTS` / `MAX_GENERATED_BY_OCTETS`; widen
   `format_majors("profile")` to `[MAJOR_V1, MAJOR_V2]` — the only half that
   grows (A4).
3. Add `crates/core/src/chain_ops.rs` (the `Chain` type, `load_chain`,
   `resolve`, `fork`, `diff`, `merge`); add `CoreError::ChainLinkMissing` to
   `error.rs`; extend `lib.rs` re-exports and the public-surface doc; add a
   one-line pointer in `chain.rs`'s header (its code untouched, A15).
4. Add `tests/chain_ops.rs`; extend `grammar_closure.rs`, `version_gate.rs`,
   `conformance.rs`'s expected list, and `zz_api_probe.rs` (ADR-004
   mirroring; the probe names the new exports).
5. Update `schemas/README.md`: the series sentence records `profile/v2`
   landed, and the budget paragraph names the two new constants beside the
   existing exported limits. Update `data/profiles/README.md`: ADR-014's
   loader exists and reads the directory through the chain rules; the
   untrusted-input posture paragraph stands untouched.
6. Ship no profile data. The first real chains land as their own PRs —
   seed-preset bases with ADR-016, the first hub-shared chains with ADR-018 —
   each carrying its manifest and registry rows per design 013's growth
   order; any v1 ids that exist by then are re-issued as slug ids in that
   step.
7. Evidence: the merge PR's `rust` job posts the `test result:` lines for
   `chain_ops.rs` and the updated suites; the Windows leg of Test strategy
   #9 runs once by a contributor.

## Deferred / out of scope

- The transforms that produce bases, `schemas/policy/v1/`, and a transform
  output becoming a chain base (015's open question about a new document
  with a new id): ADR-015.
- The CLI surface for `diff`/`fork`/`merge`, re-adoption prompts over a
  bumped base, and where a player's personal chain is stored: ADR-017.
- Hub upload bundling (a directory of chain documents), transport, and
  profile trust over a channel: ADR-018.
- Seed presets as real chain bases and reading a game's shipped default
  bindings into a base: ADR-016 (seeds) and ADR-017 (codec); this design
  only makes an all-`set` base expressible.
- The CI matrix re-validating dependent chains on each manifest bump: the
  operand side shipped here (the pin, the equality rule); the matrix stays
  deferred exactly as design 013 left it — needed before the first
  pinned chain ships.
- Named-key and modifier input members (`tab`, `esc`, arrows, F-keys):
  widening the closed sets rides the `reach-classes` vocabulary major
  (012's mechanism); a v2 `set` simply cannot name them until then.
- `diff` between chains that share no base, and merge-conflict resolution
  UX beyond the typed refusal: ADR-017.
- v1→v2 migration tooling: no follow-up yet — no v1 data exists to migrate
  (A12), so the migration is a documented re-issue step, not a tool.

## Revision log

- r1:
  - **The contract / Cross-file loading** — a v2 `patch` carries one `derives`,
    a `base` none; link paths are `<major dir>/<parse-validated slug>.toml`,
    chain identity is the documents' contents, one gate per opened source.
  - **Chain operations** — `merge` refuses per-action non-representability
    under the fixed list order (a `swap X Y` under a `clear X`), and an
    identity-cancelling composition refuses as the empty-patch case;
    `diff`/`merge` outputs pin `id`/`derives`/`for` and refuse collision
    ids against the named set (`a`'s links; `p1`, `p2`).
  - **Resolution / The model** — `Chain` comes only from `load_chain`; mounting
    and the round-trip are file-level moves (lower link as head, `parse_file`,
    `load_chain` of written output); `fork`'s shell is unloadable until filled.

## As-built (2026-11-13)

Record of what shipped, written after the r1 code-review passes (all seats
approve) against `git diff 4cfcd8d...HEAD`. The range's code files are exactly
this proposal's delta inventory — the New and Edited lists, plus
`schemas/profile/v2/` with 7 valid and 37 invalid fixtures under exactly the
names listed — and no file on the Unchanged list appears in the diff. No chosen
option was changed. The points below are where the code refines, exceeds, or
narrows the design text; `03-build-notes.md` carries the full record.

1. **Cross-file loading — two refusal refinements.** (a) A parse fault inside
   a link document is re-attributed to the link (`name_the_link` prefixes the
   link id), so every chain refusal names the link it refused. (b) A missing
   *head* is `parse_file`'s `Io`, whose variant carries the path through
   `source_path`; `ChainLinkMissing` answers only links the walk constructed
   from a `derives` element, because a head's file stem has never been
   validated as a profile id (A13). Variants and the escape-and-truncate path
   unchanged. Affected: Chosen design → Cross-file loading, steps 1–2.
2. **Cross-file loading step 1 — the directory-major fault answers the shared
   `check_dir_version`'s `InvalidValue`** (naming file, declared major, and
   directory), the one spelling every shipped loader uses; `UnsupportedVersion`
   stays reserved for a major the format does not ship. Affected: Cross-file
   loading step 1 (A5); The model's error-shape sentence.
3. **Cross-file loading step 4 — the position rule ships subsumed.**
   `check_positions` is implemented as stated, but the per-document parse shape
   rules already refuse every contradiction it could reach, so it is an
   unreachable belt-and-braces; what load actually answers is an `InvalidValue`
   naming a link whose document fails its parse-level shape rules. Affected:
   Cross-file loading step 4; Test strategy #2/#3 (three tests' `proves:`
   comments carry this as a review minor).
4. **Chain operations → `merge` — two refinements stated in the code.** The
   composition runs over symbols (`Base` / `Written` / `Unbound`), emits in the
   fixed list order, and re-checks the emitted patch against those symbols
   before returning. It assumes the base binds *both* sides of every exchange
   either patch names; a composition whose answer would depend on the base's
   bindings (`set X a` under `swap X Y`) has no one-patch form on the symbols
   and refuses — the proposal's example prices one case, the code refuses the
   whole class without ever reading the base. And `merge` carries no
   provenance label onto its output even when both operands agree: neither
   operand produced the merged document (Decision Four). Affected: Chain
   operations → `merge`.
5. **The model — container choice.** `chain_ops` uses `BTreeMap`/`BTreeSet`:
   `ActionId`/`InputId` derive `Ord` but not `Hash`, and the delta inventory
   declares `action.rs`/`input.rs` unchanged. A side benefit: which colliding
   id a refusal names is itself deterministic. Affected: The model.
6. **The model / Rollout 2 — visibility, and one unnamed constant.** Three
   `document.rs` items are `pub(crate)` so `chain_ops` reuses them instead of
   restating rules (`MAJOR_V2`, `check_profile_slug`, `format_majors`), and the
   63-octet id cap is a private `MAX_SLUG_OCTETS` — the exported budget family
   is exactly the two named constants, which is what `zz_api_probe.rs` pins.
   Affected: The model; Rollout 2–4.
7. **The contract — label refusals never echo the label, and two `spec.md`
   sentences run narrower than the reader.** A `generated-by` refusal quotes
   the length and the reason rather than the value (`one_line` escapes ASCII
   control but not bidi characters); `generated-by-bidi-char.toml` pins the
   refusal. The reader refuses the default-ignorable/soft-hyphen set as this
   proposal's entry table requires, but `spec.md`'s `generated-by` row names
   only ASCII-control and bidi-control; and `spec.md`'s "case-unique"
   id↔file-name sentence describes a lookup the loader does not enforce in one
   direction on case-insensitive filesystems. Both are wording gaps carried as
   open review minors; the grammar itself is as designed. Affected: The
   contract (`generated-by` row, per-link rules); third-party-reader matrix
   row.
8. **Cost paragraph — the delta cap is a parse rule only.** `merge` and `diff`
   do not re-check `MAX_PATCH_DELTA_ELEMENTS` on the `Profile` they return;
   the emitted lists are bounded by the operands' union, so a merge of two
   near-cap patches can emit past the cap and the result is refused at the
   next parse, not at the operation. Affected: Resolution (composite-cost
   paragraph); Chain operations.
9. **Docs beyond the edit list, and carried minors.** `schemas/README.md`
   gained two further sentences (the two-majors-today note; the constants
   count) and `spec.md` a fixture-name discipline paragraph — consequences of
   `profile/v2` landing, not new contract. Carried unfixed at merge (none
   design-changing): `check_link`'s non-profile arm and `link_below`'s no-kind
   fault are unpinned defensive refusals; `one_line` passes U+2028/U+2029
   raw; the slug grammar admits Windows reserved device basenames. Affected:
   Rollout 5; Compatibility matrix (Windows, third-party-reader rows).

Two round-1 divergences (the re-worded `Cycle` `Display`, and a second
directory-major spelling answering `UnsupportedVersion`) were withdrawn in the
fix round; HEAD follows the proposal on both, so they are not recorded as
drift.
