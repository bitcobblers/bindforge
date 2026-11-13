---
status: proposed
date: 2026-11-12
supersedes: 009
series: 011-015
depends-on: [011]
---

# ADR 012: Define the input vocabulary and per-hand reach classes

Second installment of the 011-015 series. We will name physical inputs
OS-neutrally and grade every key by **reach class** for each hand, as data
in `schemas/` loaded by `crates/core`. This is the placement mathematics the
product needs: it decides which key a player can comfortably reach, and it
makes cluster placement (014/015) a checkable property rather than an
opinion.

## Context

The keyboard's halves are asymmetric: the right pinky freely reaches `;` and
`'`; the left's do not. So "can this hand reach this key" cannot be a
symmetric function, and handedness-aware placement needs a per-hand table.
The move most left-handed players want — relocate the WASD cluster to the
right hand with its internal shape preserved — is only legal if each action
lands on a key the hand reaches as well as before. That legality test needs
reach classes, which is this contract.

## Decision

**Input vocabulary.** Physical inputs are named OS-neutrally:
`keyboard:<key>`, `mouse:left|right|middle|side-1|side-2|wheel-*`,
`gamepad:*`. v1 names keyboard keys by the US-QWERTY glyph (`keyboard:w`)
under ANSI geometry; physical-position naming for other layouts is deferred
and the id shape reserves it.

**Reach classes.** Each key is graded, per hand, from
`home · stretch · far · thumb · off-reach`, with **left and right tables
distinct** by definition. The tables live in `schemas/` as data;
`crates/core` loads and exposes them. They are ANSI-US at first; other
geometries add their own table files, not new code.

**Cluster placement.** A placement is a shape-preserving assignment of a
movement cluster to a target region — IJKL and OKL; are two legal placements
of the WASD shape. A placement is legal when (a) the cluster's internal
shape is preserved and (b) no action's reach class is degraded beyond a
declared bound. This ADR defines the legality predicate over the reach
tables; 015 is the transform that uses it.

`schemas/` carries the reach-table format and the input-id spec, with
conformance fixtures.

## Alternatives considered

**A symmetric reach function.** Cheap, and wrong: the halves are not
symmetric, so it green-lights left-hand stretches a left hand cannot make.

**Reach as per-key pixel distance.** Distance is not reach — hand posture,
not centimetres, decides comfort; the graded classes encode posture.

**Naming keys by physical position (LayoutX) now.** The right long-term
answer for non-US keyboards, but it needs per-layout position data we do not
have; the id shape reserves it and v1 ships glyph naming.

## Consequences

Becomes possible: placement legality (015) is computed from data; adding a
keyboard geometry is a new table, not new logic; reach claims are auditable
against the table.

Accepted costs: tables are ANSI-US at first; non-US layouts and
physical-position key naming are deferred; gamepad inputs are named
(`gamepad:*`) but not reach-graded in v1 (controllers are ambidextrous).

Not decided here: the transforms that consume reach classes (015); cluster
*placement as a policy* lives with the transforms, not with the vocabulary.
