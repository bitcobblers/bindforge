---
doc_type: adr
status: accepted
date: 2026-11-12
supersedes: 009
series: 011-015
depends-on: [011, 012, 014]
---

# ADR 015: Define transforms as named, preconditioned policies

Fifth and final installment of the 011-015 series. We will define the
**transform/policy** layer in a new `crates/transforms` crate: an ordered
stack of named transforms, each with preconditions, that produces bases. A
transform never overwrites a player's personal patch — it operates on a base,
and the personal patch sits above it in the chain (014) untouched.

## Context

"Lefty" is not one transform. The useful move for locomotion is relocating the
WASD cluster to the right hand with its shape preserved (012), not a geometric
mirror; mouse-button swap is a separate decision many left-handed players
decline; camera inversion is a game option, not a binding. So handedness is a
*stack of independent, named decisions*, and accessibility policies
(disabilities, alternative controllers) are the same shape with different
constraints. A composable, named policy stack serves both — and it must refuse
cleanly, because a transform that "fixes" a base it does not fit mangles the
player's layout.

Nothing in `crates/core` does this today, and that is deliberate: 011 records
the axis-kind attribute without acting on it, 012 defines placement legality as
a predicate without choosing placements, and 013 records game constraints
without applying them. This ADR is where those three become load-bearing,
which is why it runs last in the series. It builds on `crates/core`'s types —
`ActionRegistry`/`AxisKind`, `ReachTables` and the placement predicate,
`resolve_chain` and `MAX_CHAIN_DEPTH` — and adds a format, `schemas/policy/v1/`.

## Decision

A **policy** is an ordered stack of named transforms
(`relocate-cluster(move.steer-*, →right, placement=OKL;)`,
`swap-polarity(move.strafe-*)`, `swap-mouse-buttons()` — each **off** unless
the stack names it).

Each transform declares:

- the **tags it touches** — taxonomy ids (011), and
- **preconditions** — e.g. "the locomotion cluster is contiguous and
  left-handed in this base", checked against reach legality (012) and the
  manifest's constraints (013): a hard-docked or exclusive-group key is never
  a target.

An inapplicable transform **reports and no-ops; it never mangles**.
`swap-polarity` may act only on `discrete-lateral` actions — `continuous-lateral`
(camera look) is never a key swap — which is exactly the attribute 011
reserved for this rule.

Transforms apply **only to bases** (game defaults, presets, chain bases). A
personal patch (014) sits above them in the chain and survives base
regeneration untouched — the "produce bases, never overwrite players" rule.

`schemas/policy/v1/` carries the policy/transform format with its golden
fixtures, joins the shared parse path and the version gate, and
`crates/transforms` implements the stack on `crates/core`'s types.

## Alternatives considered

**One global mirror transform.** Cheapest, and wrong twice: it reflects keys
that do not move together (the cluster is not the neighbourhood), and it forces
a button swap that many left-handed players keep as-is.

**Acting on a resolved layout instead of a base.** It destroys the base/patch
separation (014): the player's own tweaks become the thing a policy edits.

**An AI-generated layout as the primary artifact.** Generation bootstraps
games with no profiles, but placement stays deterministic (012 reach,
placement legality) so drafts are auditable; generated bases enter the chain
like any other, gated by validation, not trusted by provenance.

**Conditions as free-form predicates.** Expressive, and unauditable: each one
is new logic to review per game. A fixed precondition vocabulary over loaded
data keeps a policy reviewable as data.

## Consequences

Becomes easy: author a handedness or accessibility policy once and re-apply it
to every supported game; compose independent named decisions; a transform that
does not fit degrades visibly and harmlessly; the axis-kind rule that 011 only
recorded now has teeth.

Costs: every transform is a named, preconditioned, tested unit — preconditions
are real logic, not a flag; the policy format joins the conformance corpus and
`supported_majors`; the seed preset set is a curated collection to maintain
(ADR-019).

Not decided here: which policies ship as the seed preset set (ADR-019); the CLI
to author and apply policies (ADR-017); the hub and voting (ADR-018).
