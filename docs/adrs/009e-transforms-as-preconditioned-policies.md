---
status: proposed
date: 2026-11-12
supersedes: 009
series: 009
depends-on: [009a, 009b, 009d]
---

# ADR 009e: Define transforms as named, preconditioned policies

Fifth and final installment of the 009 series. We will define the
**transform/policy** layer in `crates/transforms`: an ordered stack of named,
preconditioned transforms that produce bases. A transform never overwrites a
player's personal patch — it operates on a base, and the personal patch sits
above it in the chain (009d) untouched.

## Context

"Lefty" is not one transform. The useful move for locomotion is relocating
the WASD cluster to the right hand with its shape preserved (009b), not a
geometric mirror; mouse-button swap is a separate decision many left-handed
players decline; camera inversion is a game option, not a binding. So
handedness is a *stack of independent, named decisions*, and accessibility
policies (disabilities, alternative controllers) are the same shape with
different constraints. A composable, named policy stack is the abstraction
that serves both — and it must refuse cleanly, because a transform that
"fixes" a base it does not fit mangles the player's layout.

## Decision

A **policy** is an ordered stack of named transforms
(`relocate-cluster(move.steer-*, →right, placement=OKL;)`,
`swap-polarity(move.strafe-*)`, `swap-mouse-buttons()` — default **off**).

Each transform declares:

- the **tags it touches** (taxonomy ids, 009a), and
- **preconditions** — e.g. "the locomotion cluster is contiguous and
  left-handed in this base," checked against reach legality (009b).

An inapplicable transform **reports and no-ops; it never mangles**.
`swap-polarity` may act only on `discrete-lateral` actions (009a); camera
(`continuous-lateral`) is never a key swap.

Transforms apply **only to bases** (game defaults, presets). A personal
patch (009d) sits above them in the chain and survives base regeneration
untouched — the "produce bases, never overwrite players" rule.

`schemas/` carries the policy/transform format with conformance fixtures;
`crates/transforms` implements the stack, on `crates/core`'s types.

## Alternatives considered

**One global mirror transform.** Cheapest, and wrong twice: it reflects keys
that do not move together (the cluster is not the neighbourhood), and it
forces a button swap that many left-handed players keep as-is.

**An AI-generated layout as the primary artifact.** Generation bootstraps
games with no profiles, but placement stays deterministic (009b reach,
placement legality) so drafts are auditable; generated bases enter the chain
like any other, gated by validation, not trusted by provenance.

**Transforms that patch a player's personal layout.** Rejected: it destroys
the base/patch separation (009d). Transforms produce bases only.

## Consequences

Becomes possible: author a handedness/accessibility policy once and re-apply
it to every supported game; compose independent named decisions; a transform
that does not fit degrades visibly and harmlessly.

Accepted costs: every transform is a named, preconditioned, tested unit
(preconditions are real logic, not a flag); the seed preset set is a curated
collection to maintain (ADR-011).

Not decided here: which policies ship as the seed preset set (ADR-011); the
CLI to author/apply policies (ADR-012); the hub and voting (ADR-014).
