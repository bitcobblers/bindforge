# Design: ADR-015 — transforms as named, preconditioned policies

## Context

ADR-015 ([docs/adrs/015-transforms-as-preconditioned-policies.md](../../adrs/015-transforms-as-preconditioned-policies.md))
defines the transform/policy layer: an **ordered stack of named transforms**
(`relocate-cluster`, `swap-polarity`, `swap-mouse-buttons` — each *off* unless
the stack names it), each declaring the taxonomy tags it touches and
preconditions checked against loaded data, an inapplicable transform
**reporting and no-opping — never mangling** — and applying **only to bases**,
with a personal patch (014) sitting above untouched. This design closes as
decisions the research's open questions: the `schemas/policy/v1/` grammar, what
a *cluster* is as data, what a transform's output *is* in a chain, stack-order
semantics, and the report's shape — every one answered once in Chosen design or
listed under Deferred / out of scope.

Everything this stands on already ships and was shaped for it:
`AxisKind::DiscreteLateral`'s enum comment pre-announces its only consumer
(A2), `placement_is_legal` answers a structured `PlacementCheck` precisely so
"ADR-015's precondition has to be able to say why a placement is illegal" (A4),
the manifest spec states "a transform's preconditions read this document's
constraints" (A6), and the contract tree already names `policy` as the next
format, whose directory the conformance walk absorbs with no harness change
(A8). This ADR is where 011's axis-kind attribute, 012's placement predicate,
and 013's manifest constraints become load-bearing.

## Goals / Non-goals

- Goals:
  - `schemas/policy/v1/`: `spec.md` + a valid/invalid golden-fixture corpus;
    joins the shared parse path, the per-format version gate, and the
    conformance walk with no harness change (A8, A11, A12).
  - A closed v1 grammar: entries `format`, `version`, `id`, `transforms`;
    transform kinds are the ADR's three, compiled as a closed set; an unknown
    kind, a wrong row shape, or a malformed tag is a parse-level refusal (A1).
  - A materialized answer to "what is a transform's output": an ordinary
    `profile/v2` `kind = "base"` document, deterministically regenerable,
    governed by 014's pin-equality rule — never an edit to anything above a
    base (A7).
  - `crates/transforms`, the second ADR-008 workspace crate: the stack —
    sequential application, per-row verdict reports, the manifest-constraint
    guard — over core's public surface (A16).
  - Precondition outcomes are **verdict data** on `PlacementCheck`'s channel,
    never new `CoreError` variants; refusals reuse existing variants (A9).
- Non-goals:
  - The seed policy collection (ADR-016); the CLI that authors or applies
    policies, and the directory a produced chain is stored in (ADR-017);
    policy distribution and voting on the hub (ADR-018). This run ships no
    policy data.
  - Preconditions composed with and/or/not, and any fourth transform kind:
    both ride a future `policy/v2` (Options considered).
  - Deriving hands from data — 012 refused it, and this design restates the
    refusal rather than reopening it (A4).
  - Widening the closed input vocabulary (named keys, mouse geometry): that is
    the `reach-classes` vocabulary major, 012's mechanism (A19).

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | The ADR fixes: a policy is an ordered stack of named transforms, each off unless the stack names it; each transform declares the tags it touches and preconditions; an inapplicable transform reports and no-ops, never mangles; transforms apply only to bases and a personal patch survives base regeneration; `schemas/policy/v1/` carries the format and `crates/transforms` implements the stack | verified | `docs/adrs/015-transforms-as-preconditioned-policies.md:39-63` |
| A2 | `AxisKind::DiscreteLateral` documents "the only kind a polarity flip may touch (ADR-015)"; `ContinuousLateral` is camera look, "never a key swap" | verified | `crates/core/src/action.rs:54-60` |
| A3 | `registry.toml` is the only id list and ships exactly four `discrete-lateral` ids (`move.steer-*`, `move.strafe-*`), one `continuous-lateral` (`aim.camera`), and 35 `none` — 40 ids (the research counted 39; the file is the contract) | verified | `schemas/action-ids/v1/registry.toml:1-7` |
| A4 | `placement_is_legal(from, to, moves, bound, tables, geometry)` returns `PlacementCheck {legal, shape_ok, bound, delta, move_deltas, degradations}` because a bool cannot say why a placement is illegal; hands are **inputs** to the predicate, never facts derived from data; the predicate is region-agnostic — which placements a policy picks is ADR-015's | verified | `crates/core/src/placement.rs:1-25,97-119,164-208` |
| A5 | Reach grades and the predicate cover `keyboard:*` only; a mouse or gamepad id in a move is a definedness refusal, not a verdict | verified | `crates/core/src/placement.rs:225-236`; `schemas/reach-classes/v2/fixtures/invalid/mouse-input.toml` |
| A6 | The manifest's constraints are typed and bounded — `dead_keys`, `docked_actions` ("an action the game hard-docks to that input, never a transform target"), `exclusive_groups` (2..=16 members, ≤64 groups), `presets` — and the spec states a transform's preconditions read this document | verified | `crates/core/src/manifest.rs:45-57,83-114`; `schemas/manifests/v1/spec.md:7-9,51-52` |
| A7 | `profile/v2` ships with typed deltas (`set`/`clear`/`swap`), a required `kind`, the `for` pin, and a 128-octet `generated-by` label; a `base` has empty `derives` and no `clear`/`swap` elements — a base's layout *is* its `set` list; `load_chain`/`resolve`/`fork`/`diff`/`merge` ship in `chain_ops` | verified | `crates/core/src/document.rs:84-96,264-284`; `crates/core/src/chain_ops.rs:69-310`; `schemas/profile/v2/fixtures/invalid/base-clear-element.toml`, `base-swap-element.toml`, `base-derives-nonempty.toml` |
| A8 | The contract already names `policy`: `error.rs` counts policy documents among the untrusted classes; `format_majors`' doc says where a future `policy` major gets declared; the harness comment says the walk picks up `policy` with no harness change | verified | `crates/core/src/error.rs:3-5`; `crates/core/src/document.rs:191-197`; `crates/core/tests/conformance.rs:378-385` |
| A9 | The channel split is shipped: a refusal is a `CoreError` variant and `action.rs` deliberately adds none, while a *verdict* is structured data (`PlacementCheck`); "could not evaluate" is a refusal, "evaluated and does not fit" is a verdict | verified | `crates/core/src/action.rs:20-21`; `crates/core/src/placement.rs:20-25` |
| A10 | The re-application algebra is already contract at the patch level: a `swap` pair applied twice, `set` then `clear`, `set` twice last-wins — all valid fixtures | verified | `schemas/profile/v2/fixtures/valid/swap-pair-applied-twice.toml`, `set-then-clear-same-action.toml`, `set-twice-last-element.toml` |
| A11 | The version gate is per-format: `supported_majors()` already unions to `["v1","v2"]`, `format_majors` answers each format's majors, and `check_dir_version` refuses a declared major differing from its `vN/` directory | verified | `crates/core/src/document.rs:187-221` |
| A12 | The harness walks every `schemas/*/v*/` fixture with the zero-fixture rule, the lowercase-`-`-joined case-unique name rule, and the parse→`serialize`→re-parse round-trip for every format except `reach-classes` | verified | `crates/core/tests/conformance.rs:67-102,193-235,332-376`; `schemas/README.md:44-50` |
| A13 | The exported budget family is nine constants — `MAX_DOCUMENT_BYTES` 1 MiB checked pre-read, `MAX_NESTING_DEPTH` 16, `MAX_CHAIN_DEPTH` 32, the four manifest caps, the two chain caps — and 014 set the posture of stating the composite instead of a new aggregate | verified | `crates/core/src/toml.rs:36-39`; `crates/core/src/chain.rs:23`; `crates/core/src/manifest.rs:45-57`; `schemas/README.md:74-112` |
| A14 | Refusal posture: only typed variants; every echoed document string escapes and truncates at 80 chars; a local path never enters a message (`source_path` is the logging channel) | verified | `crates/core/src/error.rs:1-29,166-186` |
| A15 | The slug grammar `[a-z][a-z0-9-]*` (checked before any filesystem touch) and the 63-octet label cap are shipped and shared by geometry names, manifest ids, and profile ids | verified | `crates/core/src/placement.rs:67-93`; `crates/core/src/document.rs:172-180` |
| A16 | `crates/transforms` is the planned second crate; the workspace's `crates/*` glob admits it automatically; `crates/core` depends on nothing and stays the dependency root | verified | `Cargo.toml:5-12,21`; `docs/adrs/008-rust-for-the-product.md:35-46`; `crates/core/src/lib.rs:1-5` |
| A17 | ADR-004 binds a test tree to mirror `src/`: a module-named test file is that module's unit test | verified | `docs/adrs/004-mirror-src-hierarchy-in-tests.md:23-33` |
| A18 | A `generated-by` label may carry any printable character except controls, default-ignorables, and bidi controls, within 128 octets — a `:` in a label like `policy:<id>` passes the grammar | verified | `crates/core/src/document.rs:1083-1125` |
| A19 | The closed vocabulary: `mouse:` has `left` and `right` among seven members; the 48-member `keyboard:` set names no `tab`, `esc`, arrows, or F-keys, so some moves (WASD→arrows) are not expressible at all | verified | `crates/core/src/input.rs:86-97`; `schemas/manifests/v1/spec.md:63-66` |
| A20 | Exactly one keyboard geometry ships (`ansi-us`); the shape rule reads the grid, so another geometry is new table files, not new logic — a policy shipped today is effectively ANSI-US-tested | verified | `schemas/keyboard-geometry/v1/`; `crates/core/src/placement.rs:9-11` |
| A21 | No type or shipped data models a "cluster": the word is prose in `placement.rs`'s docs; a cluster concept must be built here, over base bindings and grid coordinates | verified | grep of `crates/core/src` — `cluster` occurs only in `placement.rs` doc text |
| A22 | The encoding house style: a mapping inside a one-line string array is space-joined fields — sound because no id grammar carries a space — and wrong field counts are pinned by invalid fixtures | verified | `crates/core/src/document.rs:943-1013`; `schemas/keyboard-geometry/v1/fixtures/invalid/pair-two-fields.toml` |
| A23 | Prior-art convergences this design leans on: XKB's closed vocabulary of independently opt-in named options; Terraform's `precondition` with a mandatory message; CSS `@supports`' closed condition grammar over capability data; RFC 6902's sequential application; AutoHotkey as the free-form caution; PowerToys' orphan warning as "reports, never mangles" | unverified | 01-research.md (Prior art; Preconditions as data; Security lens) |
| A24 | The shipped v2 reach taxonomy is exactly five severity classes in fixed order (`home`, `thumb`, `stretch`, `far`, `off-reach`), so the largest possible one-action degradation is four ranks | verified | `crates/core/src/document.rs:145-150`; `schemas/reach-classes/v2/left.toml` |

## Options considered

**Transform kinds: a closed set compiled into core, or an open name space.**
An open name space means an unresolved name is somebody else's semantics — the
free-form-predicate risk the ADR rejects, one layer up (A23). Chosen: closed at
v1, exactly the ADR's three kinds, compiled into a core enum; a row naming an
unknown kind is a parse-level `InvalidValue` naming the row. Growing the set
rides `policy/v2`: the version gate fails old readers closed, and v1's corpus
stays intact forever.

**Preconditions: a separate author-declared list, or carried by the kind.** A
parallel list is a second grammar over the same facts — each atom re-inherits
the closed-entry-set discipline and buys no reviewability the row lacks. Every
precondition the ADR exemplifies is one of the checks its three kinds already
run. Chosen: each kind carries a fixed, documented precondition set, each check
mapped one-to-one to a named verdict (Chosen design → The stack); the
vocabulary stays fixed, loaded data stays the only input, and a row stays
reviewable as one line (A1, A23).

**Hands: derived from data, or declared in the row.** 012 kept hands *inputs*
to the predicate — "nothing in a table says which hand *chose* to type a key"
(A4) — and the research correctly flags that "the cluster is left-handed" is
not answerable from shipped data. Deriving it would invent the fact core
refused. Chosen: a `relocate-cluster` row *declares* the hands it leaves and
lands on (the predicate's own parameters); the checks evaluate what data does
answer — contiguity, grades, legality. The handedness claim is the author's;
where a base disagrees, a check fails visibly with a report, not a mangle.

**Transform output: a materialized `profile/v2` base, or a step inside
`resolve`.** Computing inside `resolve` folds transform semantics into 014's
frozen determinism surface (a change would ride `profile/v3`), produces no file
to share, review, or gate by validation, and leaves the research's id question
unanswered. Chosen: materialize — the output is an ordinary `kind = "base"`
document, "gated by validation, not trusted by provenance" (the ADR's words
for generated bases, true of any produced base). Regeneration is deterministic
by rule and pinned by tests, and 014's pin equality — not a new machinery —
answers regeneration under a bumped manifest: an old-pinned patch refuses at
load naming the link, while the patch file itself is untouched (A7, A10).

**Row encoding: space-joined fields, or tables / nested structures.** Tables
and dotted keys are outside every shipped grammar, each refusal fixture-pinned
(014 A19). Chosen: the shipped pair encoding (A22) — single-space-joined
fields, fixed field counts per kind except `relocate-cluster`'s trailing
target list — with wrong-field-count refusals and fixtures, as
`set`/`swap`/manifest rows already do.

**Tags: exact ids plus one `-`-anchored prefix form, or general wildcards.**
Embedded `*`, `?`, or character classes invent a match language over a closed
id set; xremap's condition-matching shows where the rule-correlation hazard
starts (A23). Chosen: a tag is either an exact action id or
`<segment>.<prefix>-*`; matching is byte-exact prefixing over the loaded
registry, which keeps 011's byte-exact posture, and a glob matching no id is
an inapplicable report naming the emptiness — never a silent no-op transform.

**Stack order: sequential, or every row preconditions on the untouched base.**
Two rows declaring the same tag need one answer. RFC 6902's sequential rule is
the shipped patch-level default — array order is data, last write wins,
fixture-pinned (A10); base-relative ordering would ask a second row's predicate
about keys the first row already moved, a fact about no document anyone can
see. Chosen: sequential — each row's preconditions see the output of the rows
above it — pinned by a valid corpus fixture and a transforms test.

**Report channel: new `CoreError` variants, or verdict data.** "Reports and
no-ops" describes a verdict: shipped refusals mean *could not evaluate at all*
(A9). Chosen: a `PlacementCheck`-shaped verdict — typed closed reason enum with
contract spellings, an offender id, and the `PlacementCheck` itself as evidence
— while policy-level refusals reuse existing variants and add none, the
`action.rs` posture (A9).

**Policy budgets: one new aggregate cap, or per-structure caps and a stated
composite.** An aggregate duplicates the bounds that already multiply out —
014's standing choice (A13). Chosen: two per-structure caps mirroring the
manifest's, `MAX_POLICY_TRANSFORMS = 64` (mirrors `MAX_CONSTRAINT_GROUPS`) and
`MAX_RELOCATE_TARGETS = 16` (mirrors `MAX_GROUP_MEMBERS`), and `spec.md` states
the composite the way `profile/v2`'s does.

## Chosen design

### Delta inventory

New: `schemas/policy/v1/` (`spec.md` + `fixtures/valid/` +
`fixtures/invalid/`, no shipped policy data — see Rollout 6),
`crates/core/src/policy.rs` (`load_policy`),
`crates/core/tests/policy.rs` (module-mirrored, A17), and the second crate:
`crates/transforms/` (`Cargo.toml`, `src/lib.rs`, `src/apply.rs`) with
`crates/transforms/tests/apply.rs` and
`crates/transforms/tests/zz_api_probe.rs`.

Edited: `crates/core/src/document.rs` (`SUPPORTED_FORMATS` gains `"policy"`,
the `format_majors("policy")` arm, `POLICY_V1_ENTRIES`, the `Document::Policy`
arm, the `parse`/`serialize` arms, the `Policy`/`Transform`/`TagPattern`
model, and the two budget constants), `crates/core/src/lib.rs` (re-exports and
the public-surface doc), `crates/core/tests/grammar_closure.rs` (the policy
entry set and alias cases), `crates/core/tests/version_gate.rs` (a `policy`
document declaring `v2` refused naming `policy`'s majors; the majors-equality
invariant stays green untouched, A11), `crates/core/tests/conformance.rs` (the
expected-directory list gains `policy/v1`; the *walk* stays unchanged, A8),
`crates/core/tests/zz_api_probe.rs` (the new public names),
`schemas/README.md` (the scaffold sentence, the budget paragraph — the exported
constants go from nine to eleven, A13), and `Cargo.lock` (one new package).

Unchanged: `crates/core/src/error.rs` — **no new `CoreError` variant** (A9) —
and `toml.rs`, `chain.rs`, `chain_ops.rs`, `placement.rs`, `action.rs`,
`input.rs`, `reach.rs`, `manifest.rs`, every existing format directory and
fixture, and every existing test assertion.

### The contract: `schemas/policy/v1/`

```
schemas/policy/v1/
  spec.md
  fixtures/valid/    minimal-policy.toml  polarity-and-mouse.toml
                     cluster-to-the-right-hand.toml  lefty-stack.toml
                     same-tag-stack.toml  exact-tag-policy.toml
  fixtures/invalid/  unknown-entry.toml  major-not-shipped.toml  missing-version.toml
                     missing-id.toml  missing-transforms.toml  transforms-empty.toml
                     transforms-past-cap.toml  transform-unknown-kind.toml
                     transform-empty-row.toml  relocate-few-fields.toml
                     relocate-target-past-cap.toml  relocate-bad-hand.toml
                     relocate-bound-letter.toml  relocate-bound-past-scale.toml
                     relocate-target-unknown-input.toml  swap-polarity-extra-field.toml
                     mouse-row-carries-tag.toml  tag-embedded-glob.toml
                     tag-uppercase.toml  tag-empty-prefix.toml  tag-three-segments.toml
                     id-uppercase.toml  id-past-cap.toml  forged-log-line.toml
                     table-header.toml  dotted-key.toml  multi-line-array.toml
                     float-value.toml  hex-integer.toml  nesting-past-limit.toml
```

The grammar is the same documented TOML subset (A22's neighbors: every
grammar refusal has its fixture); the entry set is closed, and the version gate
is the shipped per-format mechanism (A11).

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `policy` |
| `version` | string | yes | the grammar major this document is written against (`v1`) |
| `id` | string | yes | the policy's identifier: slug grammar, ≤ 63 octets (A15) — its file stem and the name a produced base carries in `generated-by` |
| `transforms` | array of strings | yes | the ordered stack, 1..=`MAX_POLICY_TRANSFORMS` elements; each element is one transform row; **array order is application order** — order is data |

Transform rows, fields single-space-joined (A22):

```
relocate-cluster <tag> <from-hand> <to-hand> <bound> <target-input>…   (1..=16 targets)
swap-polarity <tag>
swap-mouse-buttons
```

- `<tag>` — an exact action id, or the prefix form `<segment>.<prefix>-*` with
  `<segment>` a well-formed first segment and `<prefix>` `[a-z][a-z0-9-]*` (the
  form is grammar-checked at parse; registry membership is an application
  fact). Example: `move.steer-*` matches every registry id whose second segment
  begins `steer-`.
- `<from-hand>`, `<to-hand>` — `left` or `right`, the shipped hand grammar.
- `<bound>` — a decimal `0..=4`: the v2 taxonomy's five severity classes make
  four the largest possible one-action degradation (A24); past that a bound
  answers nothing.
- `<target-input>` — a closed-vocabulary input id, grammar-checked at parse
  (`InputId::parse`, A19); 1..=`MAX_RELOCATE_TARGETS`, no repeats.

Example (a valid fixture — whether the move is *legal* is an application fact
pinned in `crates/transforms/tests`; the corpus pins documents):

```toml
# schemas/policy/v1/fixtures/valid/lefty-stack.toml
format = "policy"
version = "v1"
id = "lefty-stack"
transforms = ["relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k", "swap-polarity move.strafe-*", "swap-mouse-buttons"]
```

`same-tag-stack.toml` pins two rows touching one tag (the stack-order rule,
below, is document-visible as data). `spec.md` also states the load-level and
application rules below — the location rule, the sequential order, the guard,
the closed reason vocabulary, the produced-base rules, the determinism rule,
and the composite budget — so a non-Rust implementer can conform to the whole
policy contract, while the corpus stays single-document (A12, 013/014
precedent).

### The model: `document.rs` arms

```rust
pub struct Policy {
    pub version: String,
    pub id: String,
    pub transforms: Vec<Transform>,
}

pub enum Transform {
    RelocateCluster { tags: TagPattern, from: Hand, to: Hand, bound: GradeBound, targets: Vec<InputId> },
    SwapPolarity { tags: TagPattern },
    SwapMouseButtons,
}
```

`TagPattern` (core — matching *is* grammar): exact or prefix form, with
`pub fn matches(&self, id: &ActionId) -> bool` doing byte-exact prefixing
(Options: Tags). `parse`'s `policy` arm selects `POLICY_V1_ENTRIES`, types each
row by kind with the fixed field counts above, and refuses an unknown kind or
a wrong count as `InvalidValue` naming the row. `serialize` writes rows back
from the typed fields in canonical spelling and preserves the `transforms`
array order — order is data and the harness's re-parse comparison is equality
(A12). `Document` gains the `Policy` arm; `format_majors("policy")` answers
`[v1]`; `supported_majors()` is **unchanged** — its union is already
`["v1","v2"]`, so the majors-equality invariant (`schemas/README.md`) stays
green with the new `policy/v1` directory (A11; Test strategy #10).

Two new exported constants join the budget family (A13):
`MAX_POLICY_TRANSFORMS = 64`, `MAX_RELOCATE_TARGETS = 16`.

### Loading: `crates/core/src/policy.rs`

```rust
pub fn load_policy(dir: &Path, id: &str) -> Result<Policy, CoreError>;
```

A mirror of `load_geometry`'s discipline: `id` is slug-checked *before* any
filesystem question (a `..`, a separator, an uppercase spelling refuses
naming nothing but the grammar, A15); the file is `<id>.toml` directly in
`dir`; `check_dir_version` pins the declared major against the directory
segment (A11); the file stem must equal the document's `id` entry, the same
name↔location rule the geometry and manifest loaders pin. No new `CoreError`
variant: every refusal here is `InvalidValue`, `MissingEntry`, `Io`, or the
version gate (A9, A14).

### The stack: `crates/transforms/src/apply.rs`

```rust
pub fn apply(
    policy: &Policy,
    base: &Profile,
    manifest: &Manifest,
    registry: &ActionRegistry,
    tables: &ReachTables,
    geometry: &KeyboardGeometry,
) -> Result<Applied, CoreError>;

pub struct Applied {
    pub base: Profile,               // the produced base document
    pub reports: Vec<TransformReport>,
}
```

Everything in sight is core's public surface; `crates/transforms` depends on
`bindforge-core` and nothing else, and core stays dependency-free (A16).

**Entry refusals** (the whole application could not be evaluated — the
`CoreError` channel, A9): `base.kind != Base` (transforming a patch would
touch the player's layer, the ADR's boundary); `base.for_manifest` unequal to
the joined manifest's `"<id>@<revision>"` — 014's pin-equality rule,
cross-referenced rather than restated; `policy.id == base.id` — a produced
base cannot name its own input (a chain directory holds one file per id).

**Sequential stack** (Options: Stack order): rows apply in `transforms` array
order; each row is **atomic** — its writes land in the working layout only if
every check below passes; a row's checks see the layout produced by the rows
above it. A row never partially applies, so "never mangles" is structural, not
aspirational (A1).

**The match rule — stated once, cited by Tests #4–#7 and Risks:** the *match
set* of a row's tag is the registry ids the pattern matches, sorted byte-order
(the `BTreeMap` posture that also makes which offender a refusal names
deterministic); `relocate-cluster` pairs the k-th matched id (byte order) with
the k-th target in row order; the produced `set` list preserves the source
base's element order in place.

**The untouchable guard** — unconditional, not author-declared, the ADR's
"hard-docked or exclusive-group key is never a target" made load-bearing
(A6): a row that touches an action named by a `docked-actions` row, or reads
or writes an input appearing in `docked-actions`, `dead-keys`, or any
`exclusive-groups` member, is NotApplicable `reserved-input` naming the input.
Every mature system carries such a set (A23); this one is loaded data.

Per-kind checks, each landing a named verdict (Chosen design → The report):

- `relocate-cluster` — match set empty → `unmatched-tags`; a matched action
  unbound in the working layout → `unbound-action` naming it; match count ≠
  target count → `match-count`; any source or target input not `keyboard:*` →
  `not-keyboard` naming it (the predicate's own refusal for a non-keyboard id
  is reserved for data gaps, A5); the source keys not connected by
  rook-adjacency (`|Δcol| + |Δrow| = 1`) in the loaded grid → `not-contiguous`
  — the grid computation A21 says core does not ship, stated here once; a
  target already bound to an action outside the match set → `occupied-target`
  naming it (overlapping the cluster's *own* source keys is legal — a
  translation can walk onto its own vacated keys, and clause (a) checks the
  final set's distinctness); finally `placement_is_legal(from, to, pairs,
  GradeBound(bound), tables, geometry)` — an illegal verdict →
  `placement-illegal` carrying that `PlacementCheck` as the evidence, exactly
  the answer a bool cannot give (A4). A `keyboard:*` id the grid does not
  place or the tables do not grade propagates the predicate's `InvalidValue` —
  a fact the contract lacks, not a base that does not fit (A4, A5).
- `swap-polarity` — match set empty → `unmatched-tags`; match count ≠ 2 →
  `match-count`; any matched id whose registry `axis_kind` is not
  `DiscreteLateral` → `axis-kind` naming the id — `continuous-lateral` (camera
  look) is never a key swap, the enum comment made load-bearing (A2, A3);
  unbound sides pass through, 014's `swap` semantics (A7).
- `swap-mouse-buttons` — finds the actions currently bound to `mouse:left` and
  `mouse:right` (closed-set members, A19); either unbound → `unbound-input`
  naming the mouse id; otherwise the two actions exchange bindings. The
  exchange lands in the produced base as `set` elements — which closes the
  research's "`swap` deltas are action-pair-wise, the mouse swap is
  input-wise": the transform is input-wise at read time, action-wise where it
  materializes, and needs no geometry or grades because nothing moves on a
  grid (A5).

### The report — "reports" in one shape

```rust
pub struct TransformReport {
    pub index: usize,               // row position in `transforms`
    pub kind: &'static str,         // the contract spelling of the row's kind
    pub outcome: Outcome,
}

pub enum Outcome {
    Applied,
    NotApplicable(NotApplicable),
}

pub struct NotApplicable {
    pub reason: NotApplicableReason,
    pub offender: Option<String>,   // a grammar-checked id, never raw text
    pub check: Option<PlacementCheck>,  // filled only for `placement-illegal`
}

pub enum NotApplicableReason {
    UnmatchedTags, UnboundAction, MatchCount, NotKeyboard, NotContiguous,
    OccupiedTarget, PlacementIllegal, ReservedInput, AxisKind, UnboundInput,
}
```

`NotApplicableReason::name()` answers the closed contract spellings
(`"unmatched-tags"` … `"unbound-input"`) the way `ProfileKind::name()` does
today; `spec.md` states the vocabulary so a non-Rust implementer produces the
same verdicts. Every string a report carries is a grammar-checked id, so the
bounded-echo posture holds without a `CoreError` in sight (A14); 017's CLI and
the hub display these, and this is the shape they display (research risk
"Report shape", answered).

### The output — a base in a chain

The produced `Profile`: `kind = "base"`, `id = policy.id`, `derives` empty,
`for` copied from the source base (pin equality already joined it to the
manifest), `set` = the working layout, and
`generated-by = "policy:<policy-id>"` — grammar-legal (A18), provenance that
is never consumed as trust, exactly 014's Decision Four restated. Personal
patches are not inputs, not opened, and not rewritten — apply's whole contract
is one base in, one base out (A1).

**Regeneration — the canonical answer to the research's chain question:** same
inputs ⇒ byte-identical output — `apply` is a pure function over parsed
values with sorted containers, no clock, no randomness; a test pins two applies
`serialize`-equal. A consumer regenerates by writing `<policy-id>.toml` beside
the chain and bumping pins as a review: after a manifest revision bump, a
patch still pinned to the old revision **refuses at load naming the link**
(014's pin equality), so nothing silently re-resolves under changed game facts,
and the patch file — the player's work — is untouched either way (A1, A7).
Where produced chains live on disk is storage, ADR-017's (Deferred).

`diff` across the transformation is not 014's `diff` (its deepest-link rule
refuses two chains over different base documents) and is not widened here —
Deferred lists its owner.

### Budget and cost

No new aggregate cap (A13): one application reads at most one policy document
(≤ `MAX_DOCUMENT_BYTES`) plus the one base document the caller already bounded
by the chain rules, and evaluates ≤ 64 rows × ≤ 16 predicate moves over the
capped loaded tables, geometry, and registry — bounded, and stated in
`spec.md` the way `profile/v2` states its composite. The TF2 exec-chain lesson
(per-file caps defeated by document count, A23/A13) is precluded because a
policy references no other document by path.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Policy is the fourth untrusted document class (a hostile corpus of rows) | High | High | Closed entry set + fixed row counts + tag grammar + `MAX_POLICY_TRANSFORMS`/`MAX_RELOCATE_TARGETS` at parse; `load_policy`'s slug-before-filesystem rule; the escape-and-truncate refusal posture everywhere (A8, A12, A14, A15) |
| A stack mangles a base it does not fit | Med | High | Row atomicity + the untouchable guard + the placement predicate as gate; every failure is a named report (Test strategy #4–#7 pins each reason); "never mangles" is structural |
| Base regeneration silently changes what an old patch means | Med | High | 014's pin equality refuses at load naming the link; the patch document is never edited — regeneration is a write, adoption is a review (A1, A7) |
| The closed kind set is too tight for the seed collection (ADR-016) | Med | Med | New kinds ride `policy/v2` with the version gate holding old readers closed; v1 ships no data, so the cost today is zero and the migration is one new directory (A11) |
| Byte-order pairing surprises an author (a target pairs with an id, not a position on screen) | Med | Low | The match rule is stated once (Chosen design → The stack) and fixture-pinned; a wrong pairing fails the shape clause and reports `placement-illegal` with the check as evidence; the produced base's PR diff shows the net effect |
| The determinism surface (reason spellings, pairing, order preservation) freezes | High | Med | That is the contract from merge; it is enumerated once, fixture-pinned, and any change rides `policy/v2` (A11) |
| Registry growth widens an old tag beyond its intent | Low | Low | The match rule re-evaluates against the loaded registry each apply; a wider match is a `match-count` report — a visible no-op, never a corruption — and tag review rides the registry PR (the manifest growth-order precedent) |
| Policies are effectively ANSI-US-tested (A20) | High | Low–Med | The same floor 012 and 014 shipped; the Compatibility matrix's Keyboard geometry row states what no v1 policy may claim |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux CI runner (the `check` gate's `rust` job, ADR-008 clause 4) | yes | The corpus rides the existing walk and the new crate rides `cargo test --workspace --locked`; no CI change this run |
| Windows 10/11 contributor | yes | Policy ids and file stems are lowercase-ASCII slugs and fixture names are case-unique (A12, A15) — the same discipline 014 shipped for chain documents |
| macOS contributor | no (v1) | Design 008's matrix; nothing here adds a macOS-specific requirement |
| Toolchain floor (1.85.0, edition 2024) | yes | std-only; `crates/transforms` depends only on `bindforge-core`; `Cargo.lock` gains exactly one package |
| Keyboard geometry | yes (ansi-us only) | `relocate-cluster`'s contiguity and legality read the loaded grid, so a v1 policy is validated against one grid: exactly one geometry ships (A20) — ISO/JIS is new table files, not logic, and no policy may claim non-ANSI support until they land. IME/composition is out of scope for v1: `apply` and `load_policy` see only action ids and closed-vocabulary physical input ids (A19), never a composed character |
| Pre-015 core reading a policy document | yes (refuses cleanly) | `format = "policy"` was not a `SUPPORTED_FORMATS` entry: `UnknownFormat`, fail-closed, never reinterpreted (A8) |
| Existing formats, fixtures, loaders, `resolve` | yes (untouched) | `profile/v1`+`v2`, `manifests/v1`, `reach-classes`, `keyboard-geometry`, `action-ids` byte-exact; `supported_majors()` unchanged (A11) |
| Third-party reader, non-Rust | yes (data only) | `policy/v1/spec.md` + fixtures are the contract, load rules and stack semantics included; no language-runner contract this stage |
| Product artifacts (CLI, desktop, hub) | not applicable | This stage ships no binary; the policy CLI is ADR-017, distribution ADR-018. What does ship — `core` + `crates/transforms` — touches no OS input API (X11, Wayland, Windows Raw Input, macOS HID): parsed documents in, parsed documents out |

## Test strategy

1. Unit — the v1 grammar (`crates/core/tests/policy.rs`, module-mirrored per
   ADR-004, plus `grammar_closure.rs`): a document using exactly the defined
   entries parses; each invalid fixture above refuses naming the entry, row,
   or field — unknown kind, empty row, `relocate-cluster` under six fields or
   over `MAX_RELOCATE_TARGETS` targets, a hand outside `{left, right}`, a
   bound that is a letter or exceeds 4, a target outside the closed input
   vocabulary, an extra field on `swap-polarity`, any argument on
   `swap-mouse-buttons`, the four malformed-tag shapes, `transforms` missing
   or empty or past `MAX_POLICY_TRANSFORMS`, and an uppercase or 64-octet
   `id`; `grammar_closure.rs` refuses a policy document aliasing
   `transforms_`.
2. Unit — round-trip: `parse(serialize(parse(text)))` equals the first parse
   for every valid fixture, including `transforms` array order and the typed
   rows' canonical re-spelling (order is data, A12).
3. Unit — `load_policy`: `..`, a separator, or an uppercase name refuses
   without a filesystem touch; a file whose stem disagrees with its `id` entry
   refuses; a document whose declared major differs from the `vN/` directory
   refuses; a non-policy document at the path refuses on the format arm
   (A11, A15).
4. Unit — `relocate-cluster` application (`crates/transforms/tests/apply.rs`),
   one test per named outcome: a contiguous legal move applies, pairing by the
   match rule and emitting in source `set` order; empty match →
   `unmatched-tags`; an unbound matched action → `unbound-action` naming it;
   match ≠ targets → `match-count`; a gamepad-bound source → `not-keyboard`;
   disconnected sources → `not-contiguous`; a foreign-occupied target →
   `occupied-target` naming it; a bound-violating move → `placement-illegal`
   with the carried `PlacementCheck` (`shape_ok`, `worst_degradation`) as the
   evidence; a docked action or dead/exclusive input on either side →
   `reserved-input`; every failure leaves the working layout byte-equal
   (atomicity).
5. Unit — `swap-polarity`: a bound discrete-lateral pair exchanges; a match
   that includes `aim.camera` → `axis-kind` naming the id (the camera rule has
   teeth, A2); count ≠ 2 → `match-count`; an unbound side stays unbound; the
   same row applied twice is the identity (mirrors the shipped
   `swap-pair-applied-twice` algebra, A10).
6. Unit — `swap-mouse-buttons`: actions bound to `mouse:left`/`mouse:right`
   exchange, landing as `set` elements on the produced base; either unbound →
   `unbound-input` naming the mouse id; a docked mouse input → `reserved-input`
   (no geometry or grades consulted — A5 states why none exist here).
7. Unit — stack semantics: two rows touching one tag apply sequentially, the
   second's preconditions seeing the first's output (a pair pinned legal only
   after the first row moved, and one whose legality the first row breaks); an
   all-inapplicable policy reports every row and produces a base equal to the
   source except `id` and `generated-by`; `report.index` equals the row
   position.
8. Unit — the produced document is a chain citizen: it parses under
   `profile/v2` rules (`kind = base`, empty `derives`, no `clear`/`swap`);
   `for` equals the source pin; `generated-by = "policy:<policy-id>"`; a
   `policy.id` colliding with `base.id` refuses naming the id; a patch as the
   base argument refuses; determinism — applying twice answers `serialize`-
   identical bytes.
9. Integration (`crates/core/tests/conformance.rs`) — the unchanged walk picks
   up `policy/v1`: every valid fixture parses, passes the guards, and
   re-serializes to a semantically equal value; every invalid one refuses with
   a single-line, control-free, truncating message (`forged-log-line.toml`
   pins that); the zero-fixture rule and the case-unique fixture-name rule
   cover the new directory; the expected-directory list gains `policy/v1` and
   the *walk* is untouched (A8, A12).
10. Integration — gate and surface invariants: `version_gate.rs` gains the
    case that a `policy` document declaring `v2` refuses naming `policy`'s
    majors while the majors-equality invariant needs no change to
    `supported_majors()` (A11); `zz_api_probe.rs` (core) resolves `Policy`,
    `Transform`, `TagPattern`, `load_policy`, and the two new constants at the
    crate root, and `crates/transforms/tests/zz_api_probe.rs` resolves
    `apply`, `Applied`, `TransformReport`, and `NotApplicableReason` at that
    crate's root; every pre-015 fixture and test stays byte-unchanged and
    `Cargo.lock`'s diff is exactly the new package.
11. Platform — the three gate commands (`cargo fmt`, `cargo clippy -D
    warnings`, `cargo test --workspace --locked`) are green on the Linux CI
    runner (the merge PR's job is the leg), and the suite runs once on a
    Windows contributor machine, where slug↔file-name case uniqueness is what
    would bite.

## Rollout

1. Add `schemas/policy/v1/spec.md`: the entry table, the row grammar and tag
   grammar, the hand/bound/target field rules, the sequential-order and
   atomicity rules, the untouchable guard, the closed reason vocabulary, the
   produced-base rules and the determinism rule, the composite budget, and the
   fixture-name discipline — then the fixtures listed above. The corpus is what
   makes `policy/v1` "supported" (`schemas/README.md`).
2. Edit `document.rs`: `SUPPORTED_FORMATS`, the `format_majors` arm,
   `POLICY_V1_ENTRIES`, the `parse`/`serialize` arms, the model types, and the
   two constants; add `policy.rs` (`load_policy`) and the `lib.rs` re-exports
   and public-surface sentence. No edit to `error.rs` — no new variant (A9).
3. Add `crates/transforms`: `Cargo.toml` (inherits the workspace package
   block; depends on `bindforge-core`; the `crates/*` glob admits it, A16),
   `src/lib.rs` (module doc: what the crate is, what surface it re-exports),
   `src/apply.rs`.
4. Add `crates/core/tests/policy.rs` and
   `crates/transforms/tests/{apply.rs,zz_api_probe.rs}`; extend
   `grammar_closure.rs`, `version_gate.rs`, `conformance.rs`'s expected list,
   and `zz_api_probe.rs` (ADR-004 mirroring; the probes name the new exports).
5. Update `schemas/README.md`: the scaffold paragraph records `policy/v1` and
   the walk picking it up with no harness change; the budget paragraph names the
   two new constants and the composite, and the exported-constants count moves
   from nine to eleven.
6. Ship no policy data. The seed policy collection lands with ADR-016 — with
   its own `data/` directory and README when it arrives — and the first
   applications run through ADR-017's CLI; each ships its own fixtures per
   design 013's growth order.
7. Evidence: the merge PR's `rust` job posts the `test result:` lines for
   `policy.rs`, `apply.rs`, and the updated suites; the Windows leg of Test
   strategy #11 runs once by a contributor.

## Deferred / out of scope

- The seed policy collection: ADR-016. AI-generated draft bases (the ADR's
  named alternative, entering chains "gated by validation, not trusted by
  provenance"): no follow-up yet — any generated base is just input to
  `apply`'s or a chain's validation, so no new contract here awaits it.
- The CLI to author and apply policies, the display of `TransformReport`s, and
  where a produced chain is stored: ADR-017.
- Hub upload bundling of policy documents, transport, and voting: ADR-018.
- Diff across a transformation (transformed base vs its source base): beyond
  014's `diff` (its same-deepest-link rule refuses), owned by ADR-017's review
  surface.
- A fourth transform kind, and precondition composition (and/or/not): ride a
  future `policy/v2` — no follow-up yet; the version gate makes the old-reader
  refusal automatic (Options considered).
- Named-key members (`tab`, `esc`, arrows, F-keys) and any mouse geometry:
  widening the closed sets rides the `reach-classes` vocabulary major (012's
  mechanism, A19).
- ISO/JIS grid tables: owner unscheduled; Compatibility matrix → Keyboard
  geometry states what may not be claimed until they land (A20).
- Reading a game's shipped default bindings into a source base: ADR-017
  (codec), exactly as design 014 deferred it.

## Revision log

- Compatibility matrix → Keyboard geometry (r1): added the row Risks cites —
  the ANSI-US-only floor (A20), with ISO/JIS and IME/composition out of scope.
- Compatibility matrix → Product artifacts (r1): v1 touches no OS input API.

Editor notes: Test strategy — "grammar_closure.rs refuses a policy document aliasing `transforms_`" is ambiguous; author to clarify.

## As-built (2026-11-13)

Record only — the design above stands as approved. Implementation shipped on
`pipeline/015-transforms-as-preconditioned-policies` (code stamp `e90525d`,
approved by all five code-review seats); the per-round record, gate legs, and
probe evidence are in `03-build-notes.md`. Where the code drifted from this
design, it is listed here with its reason and the section it touches.

- **Chosen design → The stack (Entry refusals) — a fourth refusal shipped.**
  Besides the three enumerated, a base whose `set` binds one action twice is an
  `InvalidValue { key: "set" }` naming the first duplicated id in `set` element
  order. Reason: code-review finding `security-r1c0` (major) — with
  first-match-only layout reads, a hostile-but-legal base could make a row
  report `Applied` over the stale copy of a binding while `resolve` kept the
  other effective, and could bypass the occupancy guard. `schemas/policy/v1/
  spec.md` states the rule, so contract tree and code agree.
- **Chosen design → The stack (`occupied-target` clause) — occupancy reads
  every holder.** "A target already bound to an action outside the match set"
  is evaluated over *every* action holding the target key, not the first `set`
  element; the foreign holder a report names is the first outside the match
  set in `set` element order (that tie-break is an implementation detail the
  spec leaves unstated). Same `security-r1c0` finding; the guard's purpose —
  refusing the displacement — is unachievable with a first-answer reading.
- **Chosen design → The stack (the untouchable guard) — `swap-polarity` reads
  it over bound matched actions only.** A matched action that is hard-docked
  *and* unbound in the working layout is not seen by the guard's docked leg;
  the row reports `Applied` writing nothing. Raised by the correctness seat in
  both review rounds (minor) and approved as shipped.
- **Chosen design → The report (`offender`) — the field carries a bounded
  explanation sentence with the grammar-checked ids quoted inside it** (e.g.
  `` `keyboard:s` is bound to `combat.fire`, which the row never named ``),
  not a bare id string. Raised both rounds (minor), approved; `spec.md` keeps
  the design's wording, while the suite's posture is that the sentence naming
  the ids is not part of the determinism contract.
- **Chosen design → The output — the produced base also copies
  `seeded_from` from the source base.** Not in this design's field list and
  still absent from `spec.md`'s output section. Reason: a base seeded from a
  built-in preset would otherwise lose that provenance through one application
  (provenance only, never a selection key — 014's Decision Four).
- **The model — two helper methods beyond the listed surface.** `Transform::
  kind()` (the contract spelling carried on `TransformReport.kind`) and
  `TagPattern::spelling()` (re-renders the author's tag so `match-count` and
  `unmatched-tags` verdicts can quote it without disagreeing with the
  document) are public. The bound cap `0..=4` is the private constant
  `MAX_GRADE_BOUND` in `document.rs`, enforced by the row grammar; the
  exported budget family is exactly the two new constants, as designed.
- **Chosen design → The stack (check order and internal shape).** The guard
  runs after `not-keyboard` and before `not-contiguous` (the design fixes the
  checks, not their order); inside the guard, docked-action before
  reserved-source before reserved-target, in row order. `is_cluster` answers
  "yes" when a source has no grid coordinate — a data gap propagates the
  predicate's `InvalidValue` instead of masking as `not-contiguous`. Rows run
  on a private `Stack` struct; `apply`'s public signature is exactly this
  design's. Entry refusals key `kind` / `for` / `id` / `set`, the pin refusal
  quoting both pins inside the 80-octet echo bound.
- **`swap-mouse-buttons` under a multi-holder button (not specified here).**
  The row exchanges the action each button carries *first* in `set` order and
  leaves any other holder of that button in place — an implementation reading
  of this design's sentence, pinned in code, not stated in `spec.md`.
- **Delta inventory — test-side additions.** The test tree also ships a shared
  helper module, `crates/transforms/tests/common/mod.rs` (mirroring the
  pre-existing `crates/core/tests/common/mod.rs`), not named in the inventory.
  The corpus is exactly the 6 valid + 30 invalid fixture names the contract
  lists; known unpinned corners (review-approved): no invalid fixture pins the
  row-grammar "no repeats" target rule, and the `not-keyboard` target arm and
  the `swap-polarity` guard path have no test case.

### Verified against this branch's head (pass 2, 2026-11-13)

Nothing above changed. Each bullet was re-read against the shipped tree at
`22de1c1`; the newest code commit on the branch is `b15f7da`
(`fix(transforms): close the duplicate-binding guard gap`) and the two commits
after it touch only these design records, so the drift set is the same one the
first pass recorded — no bullet was corrected, none was withdrawn. Where each
recorded drift now sits in the shipped code: the fourth entry refusal
(`crates/transforms/src/apply.rs:273-306`), occupancy over every holder
(`apply.rs:506-516, 545-549` with `Layout::holders_of` at `362-367`),
`swap-polarity`'s bound-only guard leg (`apply.rs:583-593`), the sentence-shaped
`offender` (`apply.rs:683-689` and the `fault` call sites) against `spec.md`'s
"always a grammar-checked id" (`schemas/policy/v1/spec.md:220-222`), the
`seeded_from` carry-forward (`apply.rs:748`, still absent from `spec.md`'s
output section, `spec.md:226-230`), `Transform::kind()` and
`TagPattern::spelling()` (`crates/core/src/document.rs:405, 480`), the guard's
position and inner order (`apply.rs:490-505, 644-661`), `is_cluster`'s
missing-coordinate answer (`apply.rs:700-710`), the private `Stack` under
`apply`'s designed signature (`apply.rs:191-233, 424-430`), and the mouse row's
first-holder exchange (`apply.rs:350-355, 614-638`).

Three implementation details this pass found that the design leaves open and the
first pass did not list — readings of rules the design already fixes, not new
decisions:

- **Chosen design → The stack (`not-keyboard` offender choice).** Sources are
  asked for a non-`keyboard:` input before targets are
  (`apply.rs:490, 798-811`), so which key a mixed base's verdict names is fixed
  by the row's own data. The design states the check, not its scan order.
- **Chosen design → The stack (guard position for the two swap rows).**
  `swap-polarity` runs the guard after the `axis-kind` check
  (`apply.rs:569-593`), and `swap-mouse-buttons` asks the reserved button before
  the docked action sitting on it (`apply.rs:666-681`). Only the relocate row's
  guard position is recorded above; none of these orders is stated in
  `spec.md`, and none changes which verdict a row answers.
- **The model (`MAX_GRADE_BOUND`'s value).** The cap is not the literal 4 but
  `REACH_CLASSES_V2_TAXONOMY.len() - 1` (`document.rs:180`), so the row grammar
  tracks the shipped severity-class count instead of restating it (A24). The
  exported family stays the two designed constants.
