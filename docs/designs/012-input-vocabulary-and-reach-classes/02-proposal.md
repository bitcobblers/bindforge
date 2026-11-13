# Design: ADR-012 — the input vocabulary, the reach taxonomy, and placement legality

## Context

ADR-012 ([docs/adrs/012-input-vocabulary-and-reach-classes.md](../../adrs/012-input-vocabulary-and-reach-classes.md))
owes three things the shipped contract left open: the **reach-class taxonomy**
— five per-hand grades plus the key→class mapping the v1 tables deliberately
omit (A1); an **OS-neutral physical-input vocabulary** (`keyboard:*`,
`mouse:*`, `gamepad:*`); and the **placement-legality predicate** ADR-015
preconditions its transforms on (A5). The shipped shape is the constraint:
`schemas/reach-classes/v1/` pins layout and load rules only, its class ids are
labelled placeholders, and its entry set is closed — so the mapping rides a
new major, `schemas/reach-classes/v2/`, exactly as Decision One states, with
the existing version gate refusing a v2 document for a v1-only reader
(A2). The research (01-research.md) found every mature key registry names
physical keys from an ANSI-US reference geometry, so this design follows that
common denominator rather than inventing a sixth scheme (A4), and it found no
published key→grade data or legality encoding to reuse — the taxonomy and the
predicate are BindForge's own contract data (A8).

The research's six open questions are each answered in Chosen design or
listed under Deferred / out of scope.

## Goals / Non-goals

- Goals:
  - `schemas/reach-classes/v2/`: the five real class ids, the `keys` mapping,
    and both ANSI-US hand tables filled in; v1 untouched (A2).
  - An input-id grammar — namespace, closed member sets, space-free ids —
    stated in the contract where a third-party reader reads it.
  - `schemas/keyboard-geometry/v1/`: the ANSI-US key grid as table data, so
    "adding a keyboard geometry is new table files, not new logic" (ADR
    Consequences).
  - `crates/core`: typed `InputId`, graded `ReachTable`, a geometry loader,
    and the placement-legality predicate as a pure function over loaded data.
  - The conformance harness extended only where it hard-codes `v1`; the
    `schemas/*/v*/` walk still needs no harness change for new directories.
- Non-goals: choosing placements or regions (015), non-US geometry *data*,
  physical-position naming, reach-grading mouse or gamepad inputs. Everything
  else this stage does not answer is written once, with its owning document,
  under Deferred / out of scope.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | On `main`, the v1 tables pin only layout and load rules; their class ids are placeholders; the v1 entry set (`classes`, `format`, `hand`, `version`) is closed; `supported_majors()` returns exactly `["v1"]` | verified | `schemas/reach-classes/v1/spec.md:1-45`; `schemas/reach-classes/v1/left.toml:1-6`; `crates/core/src/document.rs:35,45,61-63` |
| A2 | The contract's versioning rule (one directory per major, additive minors, fixtures define "supported", declared majors equal the majors present under `schemas/*/v*/`) makes `v2` the only in-contract route for `keys`, and the equality test covers it once the directory exists | verified | `schemas/README.md:16-26`; `crates/core/tests/version_gate.rs:92-105` |
| A3 | The documented TOML subset already admits one-line string arrays and the closed escape set, so `keys` pair strings — including a backslash glyph as `\\` — need no reader change | verified | `schemas/reach-classes/v1/spec.md:18-30`; `schemas/README.md:32-40`; `crates/core/src/document.rs:355-377` |
| A4 | Every mature key registry (USB HID, evdev, SDL, W3C `code`, macOS kVK) names physical keys from a US-ANSI reference geometry; Windows `VK_OEM_*` shows the geometry assumption must be explicit | verified | 01-research.md, "The input-vocabulary half" (https://www.usb.org/sites/default/files/hut1_3.pdf; https://www.w3.org/TR/uievents-code/; https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes) |
| A5 | 015 preconditions its transforms on this design's placement-legality predicate without 012 choosing placements | verified | `docs/adrs/015-transforms-as-preconditioned-policies.md`; 01-research.md, "The contract as it stands" |
| A6 | The home-row method grades per-hand comfort by posture from the anchor, and the right pinky's home position is `;` while the left pinky's is `a` with nothing analogous on its right | verified | https://en.wikipedia.org/wiki/Touch_typing; `docs/adrs/012-input-vocabulary-and-reach-classes.md:35-39` |
| A7 | Mouse practice is five named buttons plus a wheel that is an event, not a button; the Web standard-gamepad layout names the 16 pad buttons; no global gamepad id registry exists | verified | https://www.w3.org/TR/pointerevents4/; https://www.w3.org/TR/gamepad/; 01-research.md, "The input-vocabulary half" |
| A8 | No external benchmark validates the five-class table; the 48-key grades shipped below are authored BindForge data whose correctness rests on fixtures and review | unverified | 01-research.md, Risks & unknowns ("No external benchmark") |
| A9 | Placing `thumb` directly above `home` on the shared severity scale (a thumb reaching from a resting hand is the least demanding posture after the home anchors) is a posture judgment of this design, not a published fact | unverified | 01-research.md, Risks & unknowns (thumb "is a class name for a finger, not a posture") |
| A10 | The graded set — 26 letters, 10 digits, 11 ANSI-US punctuation keys, and space (48 inputs) — covers the keys a movement cluster binds in v1; modifiers and named keys do not | unverified | `docs/adrs/012-input-vocabulary-and-reach-classes.md:56-61` (glyph naming, ANSI-US scope); enumeration in Chosen design |
| A11 | Existing `CoreError` variants already express every refusal this design needs, with the single-line, truncating, path-free message shape | verified | `schemas/README.md:68-74`; `crates/core/src/error.rs`; `crates/core/tests/reach_tables.rs:176-195` |
| A12 | The cargo gate already runs the fixture corpora on every PR, so new format directories need no CI change | verified | `.github/workflows/check.yml:29-33` (`cargo fmt`, `cargo clippy`, `cargo test --workspace --locked`) |
| A13 | No design-stage leg has run the three gate commands against this tree yet; the merge PR's job remains the evidence leg | verified | `docs/designs/011-action-taxonomy-and-core-skeleton/02-proposal.md:475-478` (011's A16 open at 2026-11-13) |
| A14 | The suites reach the contract tree through `env!("CARGO_MANIFEST_DIR")`, so nothing here needs a cwd convention | verified | `crates/core/tests/common/mod.rs:50-62` |

## Options considered

**Where the key→class mapping lives: widen `reach-classes/v1` in place, or
ship `v2`.** The ADR decided this, and both alternatives it rejected stay
rejected: `v1`'s entry set is closed and its fixtures and `supported_majors`
pin it, so a widened `v1` breaks every v1 reader of those files (A1), while
`v2` rides the version gate that already refuses cleanly (A2). Symmetric reach,
pixel-distance grading, and physical-position naming are now rejected for the
reasons the ADR's Alternatives section states; this design only adds that the id shape
reserves the physical-position spelling (`<namespace>:<member>` with a
lowercase, space-free member — Chosen design).

**Punctuation ids: literal glyph, spelled word, or W3C `code` style.** `code`
style (`KeyQ`, `Semicolon`) is a fifth scheme next to the five the research
surveyed, and spelled words (`keyboard:semicolon`) contradict the ADR's own
words: `keyboard:<key>` is the "US-QWERTY glyph under ANSI geometry"
(Decision Three). Chosen: a literal glyph member — `keyboard:;`,
`keyboard:'`, `keyboard:\` — which is safe because ids forbid spaces (the
`"<id> <class>"` pair encoding) and every punctuation glyph survives the
closed escape table (`\` round-trips as `\\`, A3). The single exception is
`keyboard:space`: the space bar has no glyph, so it carries the one reserved
word, which cannot collide with a single-character glyph member (A10).

**Where "degraded beyond a declared bound" gets its ordinality: the shared
`classes` array order, an explicit grade number per key entry, or a Rust
order.** A per-key grade number would widen the ADR's pinned two-field pair
encoding; an order hard-coded in Rust puts the taxonomy partly in logic,
against "a pure function over loaded data". Chosen: the `classes` array order
is the severity order, worst last, and it is *shared* — every v2 document must
list exactly the five taxonomy ids, once each, in that order, so two hands
cannot define two scales (v1's order stays meaning-free: v1 is untouched, A1). The bound is then integer rank
arithmetic over table data.

**Grading when several fingers (or either thumb) reach a key: grade per
finger, grade the best finger with a separate finger model, or one
hand-level grade.** A finger model multiplies the entry set for data no v1
consumer reads. Chosen: each hand grades each key exactly once, by the
posture its designated home-row finger (or thumb) uses from the hand's
anchor — the convention the home-row method already states (A6). That is why
`thumb` is a class of its own rather than a home-key qualifier: space is the
one v1 input the thumb designates (A9, A10).

**Shape preservation: coordinates in Rust constants, a new geometry table
format, or no shape check in the predicate.** Rust constants would make the
next geometry a logic change, contradicting the ADR's stated consequence
("new table files, not new logic"). Skipping clause (a) would push the
shape rule into 015's policy layer, where it is no longer checkable —
precisely what Decision Four exists to prevent. Chosen: a small
`keyboard-geometry/v1/` format; ANSI-US ships as `ansi-us.toml`, and another
geometry is one more file in the same directory under the same entry set —
no new major, no new logic.

**Wheel and gamepad member names: mirror vendor naming, or neutral closed
sets.** The research shows three platform vocabularies for the same side
buttons and a letter-vs-position split for pad buttons (01-research.md,
Platform notes). Chosen: the ADR's neutral spellings — `mouse:side-1`,
`mouse:side-2` — plus `mouse:wheel-up` and `mouse:wheel-down` for the wheel's
two directions, with the wheel press named `mouse:middle` because Pointer
Events button 1 *is* the wheel press and a second id for one physical input
invites drift (A7). Gamepad members are the 16 W3C standard-layout names
(`a` … `dpad-right`) plus `left-stick` and `right-stick`; per-stick-direction
naming is deferred (A7).

## Chosen design

### Delta inventory

New: `schemas/reach-classes/v2/` (spec + `left.toml` + `right.toml` +
fixtures), `schemas/keyboard-geometry/v1/` (spec + `ansi-us.toml` +
fixtures), `schemas/reach-classes/v1/fixtures/invalid/keys-entry.toml` (one
fixture pinning v1's refusal of `keys`; v1's grammar and files are otherwise
untouched), `crates/core/src/input.rs`, `crates/core/src/placement.rs`, and
three test targets (`tests/input.rs`, `tests/placement.rs`,
`tests/reach_v2.rs` — module-mirrored per ADR-004).
Edited: `crates/core/src/reach.rs` (grades, the class-order rule),
`crates/core/src/document.rs` (the `v2` entry set, version-aware closed-set
check, `keys` parsing, the serializer arm), `crates/core/src/lib.rs`
(re-exports and the public-surface list), `crates/core/tests/grammar_closure.rs`
(per-format entry lists), `crates/core/tests/conformance.rs` (the reach-corpus
completion lookup hard-codes `v1` at lines 101-104 and must take the
fixture's own major; the load assertion also compares grades),
`schemas/README.md` (format list, one pointer line).
Unchanged: `toml.rs`, `error.rs`, `chain.rs`, `action.rs`, `profile/v1`,
`action-ids/v1`, `reach-classes/v1` grammar and table data, `Cargo.toml`,
`Cargo.lock`, both workflows, every existing fixture.

### The input id: grammar and closed sets

Canonical home: an "Input ids" section of `schemas/reach-classes/v2/spec.md`
— v2 is the first format that carries input ids, so the grammar is stated
where its first corpus validates it; `schemas/README.md` gains a one-line
pointer. A third-party reader that never loads a reach table still reads the
grammar there.

An id is `<namespace>:<member>`: lowercase, no spaces, one colon. The member
grammars are closed in v1 of the vocabulary, so no id can ever contain the
space the pair encoding splits on:

- `keyboard:` — exactly one glyph of the ANSI-US set `{a-z}`, `{0-9}`, `` ` ``
  `-` `=` `[` `]` `\` `;` `'` `,` `.` `/`, or the reserved word `space`
  (48 members, A10). Glyph meaning: the US legend at that position under
  ANSI geometry — the assumption every surveyed registry carries, and the
  reason the spec states the geometry rather than inferring it (A4).
- `mouse:` — `left`, `middle`, `right`, `side-1`, `side-2`, `wheel-up`,
  `wheel-down`.
- `gamepad:` — `a`, `b`, `x`, `y`, `lb`, `rb`, `lt`, `rt`, `back`, `start`,
  `l3`, `r3`, `dpad-up`, `dpad-down`, `dpad-left`, `dpad-right`,
  `left-stick`, `right-stick`.

Reach grades attach to `keyboard:*` only: a `keys` pair naming a
`mouse:*` or `gamepad:*` id is a refusal (Decision Three). Growing any
member set widens what validators accept, so it rides a new vocabulary major
— the same posture the contract already takes ("extending them means a new
major, never widening v1"); the id *shape* is what carries the deferred
physical-position naming.

### The contract: `schemas/reach-classes/v2/`

```
schemas/reach-classes/v2/
  spec.md
  left.toml
  right.toml
  fixtures/valid/    full-left-hand.toml  minimal-hand.toml  punctuation-glyphs.toml
  fixtures/invalid/  missing-keys.toml  unknown-class-reference.toml
                     duplicate-input.toml  pair-three-fields.toml  non-string-element.toml
                     mouse-input.toml  word-not-glyph.toml  uppercase-member.toml
                     class-order-mismatch.toml  unknown-class-id.toml
```

The grammar is the same documented TOML subset (A3). The entry set is v1's
plus `keys`: `format` (string, `reach-classes`), `version` (string, `v2`),
`hand` (string, `left` or `right`), `classes` (array of strings — the five
taxonomy ids, in **severity order**, worst last), `keys` (array of strings,
each exactly `"<input-id> <class-id>"`). Any other entry is rejected; v1's
set stays closed forever (A1), and
`reach-classes/v1/fixtures/invalid/keys-entry.toml` pins that refusal. v2
spec rules, each with a fixture or loader check: every `keys` member input
id is grammatically valid and `keyboard:*`; every pair names a class listed
in the same document's `classes`; each input id appears once; and `classes`
is exactly the five taxonomy ids, once each, in the severity order below —
any other list is refused.

The taxonomy, graded by hand posture from the home-row anchor rather than by
distance (A6, A9 — the ADR lists the ids as a set; this design pins their
severity order, `thumb` directly above `home`):

| Class | Meaning, per hand |
|---|---|
| `home` | the key is a home-position anchor for a finger of that hand |
| `thumb` | the thumb's designated key, reached with the hand at rest |
| `stretch` | a designated finger reaches it without moving the hand off its anchor |
| `far` | reached by shifting the hand slightly off the anchor |
| `off-reach` | not reachable from that hand's anchor posture |

The shipped tables grade all 48 inputs for each hand (each id exactly once
per hand; the grades are authored data reviewed as data, A8, A10):

```toml
# left.toml (v2) — classes in severity order; grades for the left hand
format = "reach-classes"
version = "v2"
hand = "left"
classes = ["home", "thumb", "stretch", "far", "off-reach"]
keys = ["keyboard:a home", "keyboard:d home", "keyboard:f home", "keyboard:s home", "keyboard:space thumb", "keyboard:1 stretch", "keyboard:2 stretch", "keyboard:3 stretch", "keyboard:4 stretch", "keyboard:5 stretch", "keyboard:` stretch", "keyboard:c stretch", "keyboard:e stretch", "keyboard:g stretch", "keyboard:q stretch", "keyboard:r stretch", "keyboard:t stretch", "keyboard:v stretch", "keyboard:w stretch", "keyboard:x stretch", "keyboard:z stretch", "keyboard:- far", "keyboard:. far", "keyboard:6 far", "keyboard:7 far", "keyboard:8 far", "keyboard:= far", "keyboard:[ far", "keyboard:b far", "keyboard:h far", "keyboard:j far", "keyboard:, far", "keyboard:m far", "keyboard:n far", "keyboard:y far", "keyboard:' off-reach", "keyboard:; off-reach", "keyboard:0 off-reach", "keyboard:9 off-reach", "keyboard:/ off-reach", "keyboard:\\ off-reach", "keyboard:] off-reach", "keyboard:i off-reach", "keyboard:k off-reach", "keyboard:l off-reach", "keyboard:o off-reach", "keyboard:p off-reach", "keyboard:u off-reach"]
```

```toml
# right.toml (v2) — same classes, same order; different grades, by definition
format = "reach-classes"
version = "v2"
hand = "right"
classes = ["home", "thumb", "stretch", "far", "off-reach"]
keys = ["keyboard:; home", "keyboard:j home", "keyboard:k home", "keyboard:l home", "keyboard:space thumb", "keyboard:' stretch", "keyboard:- stretch", "keyboard:= stretch", "keyboard:, stretch", "keyboard:. stretch", "keyboard:/ stretch", "keyboard:0 stretch", "keyboard:6 stretch", "keyboard:7 stretch", "keyboard:8 stretch", "keyboard:9 stretch", "keyboard:g stretch", "keyboard:h stretch", "keyboard:i stretch", "keyboard:m stretch", "keyboard:n stretch", "keyboard:o stretch", "keyboard:p stretch", "keyboard:u stretch", "keyboard:y stretch", "keyboard:\\ far", "keyboard:[ far", "keyboard:] far", "keyboard:4 far", "keyboard:5 far", "keyboard:b far", "keyboard:f far", "keyboard:r far", "keyboard:t far", "keyboard:` off-reach", "keyboard:1 off-reach", "keyboard:2 off-reach", "keyboard:3 off-reach", "keyboard:a off-reach", "keyboard:c off-reach", "keyboard:d off-reach", "keyboard:e off-reach", "keyboard:q off-reach", "keyboard:s off-reach", "keyboard:v off-reach", "keyboard:w off-reach", "keyboard:x off-reach", "keyboard:z off-reach"]
```

These tables are the ADR's asymmetry fact made data (A6); Test strategy #3
pins `keyboard:;` and `keyboard:'` both ways.

### The model: `crates/core/src/input.rs` and the reach edits

`input.rs`: `InputNamespace` (`Keyboard`, `Mouse`, `Gamepad`) and `InputId`
— `InputId::parse(&str) -> Result<InputId, CoreError>`, `namespace()`,
`as_str()` — enforcing the member grammars above; a malformed member is an
`InvalidValue` echoing the id escaped to printable characters, as action
ids do (A11).

`reach.rs`: `ReachTable` gains the grades — `grade(&InputId) ->
Option<&ReachClass>` beside `classes()` — and `load_reach_tables` gains two
checks, the first for **every** reach table at load, v1 directories
included: the declared `version` must equal the loading directory's version
segment (a v1 document must not answer from a v2 directory, as
`load_action_registry` already requires; v1-on-v1 passes trivially), and
every `keys` pair is validated as described in the contract — the canonical
`classes` list is a `parse`-level rule, so the loader compares no files (the
ordinality's source is one rule per document).
`Hand` and `hand_matches_file_name` are untouched; every v1 document declares
`v1`, so the check adds no corpus refusal; `reach_tables.rs` stays green.

`document.rs`: `REACH_CLASSES_V2_ENTRIES` beside the v1 list; the
`reach-classes` arm picks the entry set from the declared version, and
`reject_unknown_entry`'s message names that version instead of hard-coding
`v1`, the echoed version taking the same printable-escape-and-truncate path
as an echoed id does (A11). `ReachClassTable` gains `keys: Vec<InputGrade>` — the
parsed pair `input: InputId`, `class: ReachClass`, empty for v1 documents.
The `serialize` arm writes `keys` back as quoted pair strings for a v2
document — `quoted` already escapes `\` so the backslash glyph round-trips —
and reach tables stay asserted by load, not re-serialization. No new
`CoreError` variant enters the error contract (A11). `lib.rs` re-exports
`InputId`, `InputNamespace`, `InputGrade`, and the placement names below.

### Placement legality: `schemas/keyboard-geometry/v1/` and `placement.rs`

The geometry format carries what clause (a) checks — the ANSI-US key grid —
as table data. `schemas/keyboard-geometry/v1/`:

```
schemas/keyboard-geometry/v1/
  spec.md
  ansi-us.toml
  fixtures/valid/    basic-grid.toml  sparse-grid.toml
  fixtures/invalid/  unknown-entry.toml  bad-coordinate.toml
                     pair-two-fields.toml  duplicate-input.toml
```

Entry set: `format` (string, `keyboard-geometry`), `version` (string, `v1`),
`geometry` (string, `[a-z][a-z0-9-]*`), `keys` (array of strings, each
exactly `"<keyboard-input-id> <col> <row>"`: one space-separated triple,
coordinates `0..=999` through checked integer conversion so an oversized
literal is a grammar refusal, each id once). The file-name rule mirrors the
hand rule: the file is `<geometry>.toml` and its `geometry` entry names the
same value.
`ansi-us.toml` pins the 48 graded ids on the ANSI rows (`q` = `0 1`,
`a` = `0 2`, `z` = `0 3`, space = `3 4`):

```toml
# The ANSI-US keyboard grid, v1. Column is position from the row's left,
# row is the row index counting from the digit row. Another geometry is
# another file in this directory — new table files, not new logic.
format = "keyboard-geometry"
version = "v1"
geometry = "ansi-us"
keys = ["keyboard:` 0 0", "keyboard:1 1 0", "keyboard:2 2 0", "keyboard:3 3 0", "keyboard:4 4 0", "keyboard:5 5 0", "keyboard:6 6 0", "keyboard:7 7 0", "keyboard:8 8 0", "keyboard:9 9 0", "keyboard:0 10 0", "keyboard:- 11 0", "keyboard:= 12 0", "keyboard:q 0 1", "keyboard:w 1 1", "keyboard:e 2 1", "keyboard:r 3 1", "keyboard:t 4 1", "keyboard:y 5 1", "keyboard:u 6 1", "keyboard:i 7 1", "keyboard:o 8 1", "keyboard:p 9 1", "keyboard:[ 10 1", "keyboard:] 11 1", "keyboard:\\ 12 1", "keyboard:a 0 2", "keyboard:s 1 2", "keyboard:d 2 2", "keyboard:f 3 2", "keyboard:g 4 2", "keyboard:h 5 2", "keyboard:j 6 2", "keyboard:k 7 2", "keyboard:l 8 2", "keyboard:; 9 2", "keyboard:' 10 2", "keyboard:z 0 3", "keyboard:x 1 3", "keyboard:c 2 3", "keyboard:v 3 3", "keyboard:b 4 3", "keyboard:n 5 3", "keyboard:m 6 3", "keyboard:, 7 3", "keyboard:. 8 3", "keyboard:/ 9 3", "keyboard:space 3 4"]
```

`placement.rs`:

- `KeyboardGeometry { name, coordinates }` with
  `load_geometry(dir: &Path, name: &str)` — `name` matched against the same
  `[a-z][a-z0-9-]*` grammar *before* the join, a non-match refused as
  `InvalidValue` naming no path (A11), then `parse_file` on
  `dir.join(format!("{name}.toml"))` under the same budget — and
  `geometry_matches_file_name`, the pure post-load name rule beside
  `hand_matches_file_name`.
- `GradeBound(u32)` — the declared bound, in severity steps.
- `placement_is_legal(from: Hand, to: Hand, moves: &[(InputId, InputId)],
  bound: GradeBound, tables: &ReachTables, geometry: &KeyboardGeometry) ->
  Result<PlacementCheck, CoreError>`. A `move` pairs one cluster action's
  source key with its proposed target key; the hands are explicit inputs
  because data cannot tell which hand *chose* to type a key (the left hand
  can rest on IJKL too).

The predicate, a pure function over loaded data:

1. **Definedness** — every id is `keyboard:*`, appears in `geometry`, and is
   graded by both hand tables; otherwise a structured refusal, not a verdict.
2. **Shape (clause a)** — `coordinate(target) − coordinate(source)` is the
   same `(dx, dy)` for every move, taken in signed integers over the bounded
   coordinates, and targets are distinct; a rotation, reflection, or shear
   is illegal whatever the grades say.
3. **Bound (clause b)** — rank is the class's index in the shared `classes`
   order; the action's degradation is
   `rank(class(to-hand, target)) − rank(class(from-hand, source))`, and the
   placement is legal when every degradation is at most `bound`. A negative
   degradation (an improvement) never blocks.
4. The `PlacementCheck` result carries the per-move coordinate deltas — one
   common `(dx, dy)` when the moves agree — and each move's degradation, so
   a caller (ADR-015's precondition, 015's policy layer) can explain a
   refusal instead of seeing a bool.

Worked examples the suite pins (Test strategy #5), each with its hand pair:
the ADR's two named placements, WASD→IJKL and WASD→OKL, both `left`→`right`
— the folklore rationale the research records for the IJKL cluster
(01-research.md) comes out of the data here, not from a special case — and
WASD→ESDF `left`→`left`, a same-hand cluster shift one column right; all
three are legal at bound 0. Moving an action from a `home` key to
`keyboard:;`, `off-reach` for the source hand, is illegal at bound 1, and at
any bound below the class gap.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| An authored grade is wrong for real hands (A8) | Med | Med | Grades are table data, auditable per the ADR's Consequences; retuning is a PR to the TOML, predicate code unchanged; fixtures and the review seats are the only authority, and say so |
| The severity scale's shape (thumb's rank, equal-rank classes) is wrong (A9) | Low | Med | The order is pinned in `spec.md` and by the class-order test; changing the *scale* changes bound semantics, so it rides a new major |
| Glyph ids are wrong-not-unmapped for non-US users (AZERTY `ZQSD`) | Med | Med | `spec.md` states the ANSI-US geometry assumption where readers see it; the geometry format is the extension point (new table files); physical-position naming reserved in the id shape |
| Two majors live at once; third-party v1 readers see placeholders | High | Low | ADR-documented cost; `v1` refusals byte-exact, pinned by the new `keys-entry.toml`; the v2 spec is the first stable public taxonomy statement |
| The space-split encoding forbids spaces in ids forever | Low | Med | Closed member grammars make it structurally true; `pair-three-fields.toml` and `word-not-glyph.toml` pin it |
| 48 graded keys miss a key a cluster needs (A10) | Med | Low | The graded set is the grammar's set; a miss is a vocabulary-major addition (new tables + member), no consumer exists yet to break |
| The new suite never runs locally (A13) | High | Med | The merge PR's `check.yml` `rust` job is the evidence leg, as in the 008 and 011 runs |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux CI runner (`check.yml` `rust` job) | yes | The new corpora run under the three existing gate commands; no CI change (A12) |
| Windows 10/11 contributor | yes | `left`/`right`/`ansi-us` stems and fixture names stay lowercase and case-unique; the existing case-collision rule covers the new files |
| macOS contributor | no (v1) | Design 008's matrix; nothing here adds a macOS-specific requirement |
| Toolchain floor (1.85.0, edition 2024) | yes | std-only, no new dependency, `Cargo.lock` byte-unchanged |
| Third-party v1-only reader | yes | Refuses a v2 document through the version gate naming its supported majors, and refuses `keys` in a v1 document naming the entry; v1 refusals stay byte-exact (ADR Decision One) |
| Third-party reader, non-Rust | yes (data only) | This stage ships no code, binary, or runner contract for any language, and the corpora are plain TOML any parser reads; the runner contract stays Deferred |
| Product artifacts (CLI, desktop, hub) | not applicable | This stage ships no binary; how an installed artifact locates `schemas/` data stays open as in design 011 (Deferred) |
| Steam Deck / SteamOS | no (v1) | Design 008's matrix; nothing here adds platform-specific work |

## Test strategy

1. Unit (`crates/core/tests/input.rs`) — the id grammar, one case per rule:
   accept `keyboard:w`, `keyboard:1`, `keyboard:;`, `keyboard:\`,
   `keyboard:space`, `mouse:side-1`, `mouse:wheel-up`, `mouse:middle`,
   `gamepad:dpad-up`, `gamepad:lt`; reject an empty member, an uppercase
   member, a member with a space or a second colon, `keyboard:semicolon`,
   `keyboard:sp`, an unknown namespace, and a member outside a closed set
   (`mouse:thumb`, `gamepad:guide`).
2. Unit (`crates/core/tests/reach_v2.rs`) — the v2 entry set and the pair
   encoding: a document with exactly the five defined entries parses; `keys`
   on a v1 document is refused naming the entry; a v2 document without
   `keys` is `MissingEntry`; a pair that is not exactly two space-separated
   fields, a non-string element, a class reference outside the document's
   `classes`, a repeated input id, and a `classes` array outside the closed taxonomy — wrong order, a duplicated id, an id the taxonomy does not carry — are refused, naming the entry or id;
   a `keys` pair naming `mouse:left` is refused (grades are `keyboard:*`
   only).
3. Unit — the shipped tables' invariants: `load_reach_tables` on
   `schemas/reach-classes/v2/` returns two distinct hand tables; both hands
   grade exactly the same 48 input ids, each once; both `classes` arrays are
   exactly `home`, `thumb`, `stretch`, `far`, `off-reach` in that order (the
   parse-level rule is #2's); `keyboard:;` grades `home` right / `off-reach`
   left and
   `keyboard:'` grades `stretch` right / `off-reach` left; `keyboard:space`
   grades `thumb` on both; a document whose declared version differs from
   its loading directory's segment is refused, v1 directories included.
4. Unit (`crates/core/tests/placement.rs`) — geometry: `load_geometry(dir,
   "ansi-us")` returns 48 distinct coordinates covering exactly the graded
   id set, and a `name` outside the grammar — `..`, a separator — is refused
   before any file is read; a `geometry` entry contradicting its file stem
   is refused without naming a local path; a coordinate outside `0..=999`,
   and a `keys` pair that is not a triple, are refused.
5. Unit — the predicate, each case naming its `(from, to)` hands: WASD→IJKL
   `left`→`right`, WASD→OKL `left`→`right`, and WASD→ESDF `left`→`left` are
   legal with uniform deltas `(6,0)`, `(7,0)`, `(1,0)` at `GradeBound(0)`; a
   source-`home` to `off-reach` move is illegal at `GradeBound(1)`; a
   one-rank degradation (`home`→`thumb`, a home key to `keyboard:space`) is
   legal at `(1)` and illegal at `(0)`; an improving move never blocks; a
   non-translation move set (a swap or rotation of two WASD keys) is illegal
   at every bound and the check reports per-move deltas; a move naming an
   ungraded or uncoordinated id is a structured refusal, not a verdict.
6. Integration (`crates/core/tests/conformance.rs`, unchanged walk) — the
   `schemas/*/v*/` walk picks up `reach-classes/v2` and
   `keyboard-geometry/v1` with no harness change: every valid fixture
   parses and, for reach tables, loads with its grades filed under the
   declared hand (the completion table now taken from the fixture's own
   major, not `v1`); every invalid fixture, including
   `reach-classes/v1/fixtures/invalid/keys-entry.toml`, is refused with a
   single-line bounded message; the zero-fixture rule and the case-unique
   fixture-name rule cover both new directories; `supported_majors()` equals
   `{v1, v2}` and still equals the majors present on disk.
7. Integration — the shipped data and the untouched v1 contract:
   `reach_tables.rs`'s v1 expectations pass byte-unchanged;
   `grammar_closure.rs` carries the closed entry sets for
   `reach-classes/v2` and `keyboard-geometry/v1` beside the existing ones;
   the two hand tables and the geometry agree — every id graded appears in
   `ansi-us.toml` and vice versa — so the predicate's definedness rule can
   never fire on shipped data.
8. Platform — the three gate commands (`fmt`, `clippy -D warnings`, `test
   --workspace --locked`) are green on the Linux CI runner (the merge PR's
   job is the leg, A13), and the suite runs once on a Windows contributor
   machine, where the case-insensitive filesystem is where the name rules
   would bite.

## Rollout

1. Add `schemas/reach-classes/v2/` — `spec.md` (grammar, entry table, input
   ids, taxonomy with the severity order), `left.toml`, `right.toml`, then
   the fixtures named above; add
   `schemas/reach-classes/v1/fixtures/invalid/keys-entry.toml`. The corpus
   is what makes `v2` "supported" (`schemas/README.md:22-24`).
2. Add `schemas/keyboard-geometry/v1/` — `spec.md`, `ansi-us.toml`,
   fixtures.
3. Add `crates/core/src/input.rs` and `src/placement.rs`; edit `reach.rs`,
   `document.rs`, `lib.rs`, `grammar_closure.rs`, `conformance.rs`; add
   `tests/input.rs`, `tests/placement.rs`, `tests/reach_v2.rs`. No new
   dependency, no `Cargo.lock` change, no CI change (A12).
4. Update `schemas/README.md`: the format list gains `keyboard-geometry`,
   and one line points to the input-id grammar's canonical home. Module-doc
   lineage notes (`reach.rs`, `document.rs`) name ADR-012 where they say
   "ADR-012 owns the taxonomy". Docs only.
5. Evidence: the PR's `rust` job posts the `test result:` lines for the new
   targets (A13 stands — the gate has still not been run locally), and the
   Windows leg of Test strategy #8 runs once by a contributor.
6. Migrations and defaults: none; nothing consumes `v2` yet — ADR-015's
   transforms will load it — so third-party readers feel nothing until they
   adopt it (the ADR's documented cost).

## Deferred / out of scope

- Choosing placements, target-region legality (does the numpad count? are the
  arrow keys a region?), and the cluster's reach-context neighbours —
  ADR-015; the predicate is region-agnostic by construction.
- Non-ANSI-US geometry data: grade tables and grid files for AZERTY, QWERTZ,
  Dvorak, ISO, and physical-position (`LayoutX`-style) naming — the id shape
  and the geometry format are the reserved extension points; no follow-up
  yet.
- Modifier and named-key ids (`shift`, `ctrl`, `alt`, `tab`, `enter`,
  `caps-lock`): outside the v1 closed member sets; adding them widens what
  validators accept, so they ride a vocabulary major — no follow-up yet.
- Per-stick-direction and wheel-tilt naming — no follow-up yet; rides the
  same closed sets.
- Grading mouse or gamepad inputs — Decision Three names them without
  grading them; no follow-up yet.
- How an installed artifact locates `schemas/` data — the open question
  design 011 records; ADR-017 (not yet written) is the likely owner.
- An external runner contract so non-Rust readers run the corpora —
  no follow-up yet (same as design 011).
- A process for retuning shipped grades — rides ordinary PR review of the
  contract tree (the tables are the audit surface); no separate document.

## Revision log

- r1: Worked examples + Test strategy #5 — pinned cases now name their
  `(from, to)` hands; `home`→`thumb` replaces the mis-ranked one-step case.
- r1: Compatibility matrix — the non-Rust row commits to `yes (data only)`.
- r2: Placement legality — grammar-checked `name`, bounded signed deltas;
  `PlacementCheck` carries coordinate deltas and each move's degradation.
- r2: The model — version-segment equality at load for every reach table, no
  v1 corpus outcome change; A11-echoed versions; `InputGrade.class` typed.

## As-built (2026-11-13)

Record only: what shipped on `pipeline/012-input-vocabulary-and-reach-classes`
(`7dc61d8` implement … `9c1f601` test-update, reviewed through
`reviews/code/*-r1c1.md` and `reviews/code/*-r1c2.md`) against the approved r2
design above. No decision below is reopened — the numbered items are the
`03-build-notes.md` "Deviations from the design" register, with the drift, the
reason, and the design sections each one touches.

1. **The version gate is per-format, not only crate-wide** (notes deviation 4).
   *Drift:* the design states one list — `supported_majors()` — and the
   A11-shaped refusal naming it; the tree gates each format on `format_majors`
   (`document.rs`), with the crate-wide union still checked first, so a major no
   format ships names the whole union and a major another format ships names
   that format's majors. *Reason:* the round-1 correctness and compatibility
   seats both found the crate-wide list alone fail-opens a `profile` or
   `action-ids` document declaring `v2`, read with that format's `v1` grammar —
   the misreading A2 exists to prevent. *Affected:* Assumption A2 (implemented
   more strictly than stated), Chosen design → The model (`document.rs`),
   Compatibility matrix ("Third-party v1-only reader"), Test strategy #6,
   Rollout 4. The contract carries it too: a new `schemas/README.md`
   versioning-rule bullet, paragraphs in `schemas/profile/v1/spec.md` and
   `schemas/reach-classes/v2/spec.md`, and the corpus pin
   `schemas/profile/v1/fixtures/invalid/major-not-shipped.toml`.

2. **The load-time directory/version rule is conditional** (notes deviation 2).
   *Drift:* the design asks that a declared `version` equal the loading
   directory's segment for every reach table, as `load_action_registry` already
   requires; `document::check_dir_version` enforces it whenever the directory is
   named for a major and passes through when it is not. *Reason:*
   `reach_tables.rs` loads its scratch tables from a directory that is not a
   major directory at all, and that suite's paths sit outside this change; a hard
   rule would refuse it. *Affected:* Chosen design → The model (`reach.rs`), Test
   strategy #3. Pinned both ways in `tests/reach_v2.rs` (v2-in-v1 and v1-in-v2
   refused, v1-on-v1 passes), and stated in the v2 spec's Table data and
   Fixtures sections.

3. **A fourth document variant and a second serializer arm** (notes deviation
   3). *Drift:* `Document::KeyboardGeometry` with `KeyboardGeometryTable` /
   `GeometryKey` and a `serialize` arm writing the `"<id> <col> <row>"` triples
   back; the design names only the reach-classes arm. *Reason:* the conformance
   harness — the part the design leaves unchanged — re-serializes every valid
   fixture whose format is not `reach-classes`. *Affected:* Chosen design →
   The model (`document.rs`), Placement legality (`keyboard-geometry/v1`).

4. **Two files the delta inventory lists under "Unchanged" are edited** (notes
   deviation 5). *Drift:* `crates/core/src/action.rs` (its private `is_major_dir`
   is gone, replaced by the shared `document::major_dir_segment`; the two
   refusals' behavior is byte-identical) and `schemas/profile/v1/spec.md`
   (states the per-format gate, names the new fixture). *Reason:* one definition
   of "a directory is named for a major" (round-1 compatibility nit), and the
   per-format rule needs a home where a third-party reader reads it. *Affected:*
   Chosen design → Delta inventory.

5. **The test inventory is wider than the design's** (notes deviation 1).
   *Drift:* a 14th target `crates/core/tests/zz_api_probe.rs` (public-surface
   probe, not module-mirrored), edits to `tests/action.rs` and
   `tests/version_gate.rs` beside the two targets the inventory names, and two
   helpers in `tests/common/mod.rs` — `quoted_array` (the closed member set
   carries the `,` glyph, so `string_array` cannot split a `keys` array) and
   `format_majors` (the per-format majors read off disk). *Reason:* surface-level
   pinning the seats asked for and the per-format gate's test half. *Affected:*
   Delta inventory, Test strategy #1-#7.

6. **The id grammar's lowercase is stated as ASCII, and the fixture/README
   wording grew** (notes deviation 9). *Drift:* "lowercase" becomes "lowercase
   ASCII" in `reach-classes/v2/spec.md` and in the Rust rule, `keyboard:é` is
   refused naming the member rather than folded, and two invalid fixtures the
   design's lists do not name ship
   (`reach-classes/v2/fixtures/invalid/non-ascii-member.toml`,
   `profile/v1/fixtures/invalid/major-not-shipped.toml`); the README gains a
   vocabulary paragraph and a versioning bullet beyond "format list plus one
   pointer line". *Reason:* the round-1 compatibility seat found the ASCII half
   pinned only in Rust, where a third-party reader never sees it. *Affected:*
   Chosen design → The input id, both fixture inventories, Rollout 4.

7. **Names and homes the design left open** (notes deviations 6, 7, 8, 10).
   `is_geometry_name` lives in `placement.rs` and `document.rs` imports it, so
   the grammar has one definition; definedness reads "graded by both hand
   tables" as graded by the tables the check consults — source id against the
   `from`-hand table, target id against the `to`-hand table — so a same-hand
   shift consults one table twice and no refusal can fire on shipped data;
   `GradeBound(pub u32)`, `PlacementCheck { legal, shape_ok, bound, delta,
   move_deltas, degradations }` with `worst_degradation()`, `ReachTable::grade`
   and `ReachTable::rank` (linear scans over the 48-entry graded set — the
   performance seat accepted the cost); `lib.rs` re-exports `GeometryKey` and
   `KeyboardGeometryTable` beside the names the design lists. Semantics the
   design fixes — one common `(dx, dy)`, per-move deltas and degradations,
   structured path-free refusals — are all present.

8. **A13 and Rollout 5 no longer describe the tree.** The design says no leg has
   run the three gate commands against this tree. The implementation and review
   legs ran all three locally against the final tree and the quoted output is in
   `03-build-notes.md` ("Gate posture"); the merge PR's `rust` job remains the
   leg that gates the merge.

9. **Two evidence gaps stay open as review items, not as design changes.** The
   claim that the reach serializer round-trips the backslash glyph rests on the
   fixture data and the reader — no test calls `serialize` on a v2 document,
   since the harness asserts reach tables by load (testing seat, carried) — and
   the predicate's definedness refusals are pinned through `mouse:*`/`gamepad:*`
   ids rather than a graded-keyboard miss. Both are recorded open in
   `reviews/code/testing-r1c2.md`.
