---
status: accepted
date: 2026-11-12
supersedes: 009
series: 011-015
depends-on: [011]
---

# ADR 012: Grade every key by per-hand reach class

Second installment of the 011-015 series. We will fill in the **reach-class
taxonomy** — the graded comfort of each key, per hand — name physical inputs
OS-neutrally, and define the **placement-legality predicate** that decides
whether a movement cluster may move to another region of the keyboard. This
is the placement mathematics the product needs: it makes cluster placement
(015) a checkable property rather than an opinion.

## Context

ADR-008 already landed the container for this contract and ADR-011 left it
explicitly unfinished. On `main` today: `schemas/reach-classes/v1/` with
`left.toml`/`right.toml`, one table per hand, whose `spec.md` states the file
layout, the file-name/hand agreement rule, the size budget and the closed
grammar subset — and whose class ids are `classes = ["near", "mid",
"extended"]`, commented *"placeholders pending ADR-009's taxonomy"*. In
`crates/core`: `reach.rs` (`Hand`, `ReachClass`, `ReachTable`, `ReachTables`,
`load_reach_tables`) and `document.rs`'s `Document::ReachClasses` arm.

That shipped shape is the constraint: **a v1 table lists the class ids a hand
uses; it does not say which key carries which class.** The contract this ADR
actually owes is the key→class mapping, and adding it to the closed v1 entry
set is a breaking change — a v1 reader meets entries it must refuse. So the
work is a new major of the format, not an edit of the old one.

The keyboard's halves are also asymmetric: the right pinky freely reaches `;`
and `'`; the left's does not. "Can this hand reach this key" is therefore not
a symmetric function, and the move most left-handed players want — relocate
the WASD cluster to the right hand with its internal shape preserved — is only
legal if each action lands on a key that hand reaches as well as before.

## Decision

**One: `schemas/reach-classes/v2/`, leaving v1 as it stands.** A v2 table
keeps everything v1 pins (`format`, `version`, `hand`, the per-hand rule, the
grammar subset, the size budget) and adds the mapping: `keys`, an array of
`"<input-id> <class-id>"` pairs. `crates/core`'s `supported_majors()` gains
`v2`; the existing version-gate test that pins the majors to the directories
under `schemas/` then covers both majors, and a v1 reader refuses a v2
document by naming the supported majors rather than misreading it.

**Two: the class ids**, replacing the placeholders in the v2 tables:
`home · stretch · far · thumb · off-reach`. Graded by hand posture, not by
distance. `left.toml` and `right.toml` carry **different** grades for the
same key by definition, and both are filled in for ANSI-US geometry.

**Three: the input vocabulary.** Physical inputs are named OS-neutrally:
`keyboard:<key>` (US-QWERTY glyph under ANSI geometry in v1),
`mouse:left|right|middle|side-1|side-2|wheel-*`, `gamepad:*`. Physical-position
naming for other layouts is deferred; the id shape reserves it. Reach grades
attach to `keyboard:*` only — mouse and gamepad inputs are named but not
reach-graded.

**Four: placement legality.** A placement is a shape-preserving assignment of
a movement cluster to a target region (IJKL and OKL; are two legal placements
of the WASD shape). It is legal when (a) the cluster's internal shape is
preserved and (b) no action's reach class is degraded beyond a declared bound.
This ADR defines that predicate over the v2 tables — a pure function over
loaded data, so 015 can precondition on it. It does not decide placement as a
policy.

**Not re-decided here:** the grammar subset, the parse budget, the version
gate, the per-hand distinctness rule and the loader shape are ADR-008's and
stay as they are; extending them means a new major, never widening `v1`.

## Alternatives considered

**Widen `reach-classes/v1` in place.** Rejected on the rule ADR-011 already
committed to: the v1 entry set is closed and its fixtures and `supported_majors`
pin it, so a widened v1 breaks every v1 loader of those files. Widening rides
`v2`, where the existing version gate refuses cleanly.

**A symmetric reach function.** Cheap, and wrong: the halves are not
symmetric, so it green-lights left-hand stretches a left hand cannot make.

**Reach as per-key pixel distance.** Distance is not reach — hand posture,
not centimetres, decides comfort.

**Naming keys by physical position (LayoutX) now.** Right long-term answer
for non-US keyboards; it needs per-layout position data we do not have, so
the id shape reserves it and v1 ships glyph naming.

## Consequences

Becomes easy: placement legality (015) computed from data; adding a keyboard
geometry is new table files, not new logic; reach claims auditable against the
table; the placeholder ids stop being load-bearing.

Costs: two majors of the format live at once, and the loader grows a v2 shape
while keeping v1 refusals byte-exact. Tables are ANSI-US at first. A
third-party reader of `reach-classes/v1` — the corpus is language-neutral —
sees placeholder ids until it moves to v2.

Not decided here: the transforms that consume reach classes (015); which
placements a shipped policy chooses; non-US geometries.
