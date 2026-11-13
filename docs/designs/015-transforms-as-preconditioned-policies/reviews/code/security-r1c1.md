# Code review — security — round 1, dissent pass c1

## Scope

`git diff 39d4fcb589fd14ce2c81ea01572c84dbd4b28d39..HEAD -- ':(exclude)docs/designs'`
(crates/transforms/src/apply.rs, crates/transforms/src/lib.rs,
crates/transforms/tests/apply.rs, schemas/policy/v1/spec.md), read against
02-proposal.md and docs/adrs/015-transforms-as-preconditioned-policies.md.
This pass checks whether the last revision answers my round-1 findings.

## Verdict: approve

## Round-1 finding — resolved

**major (round 1): first-match `Layout` semantics vs last-writer-wins resolve —
duplicate-action silent-`Applied` defeat and duplicate-input `occupied-target`
bypass.** The fix takes the entry-refusal option I outlined and closes both
shapes:

- *(a) duplicated-action base makes a row report `Applied` over an unchanged
  effective layout.* Closed. `refuse_entry` now runs `duplicated_action`
  (apply.rs:273-306) over the whole `base.set` before the first row runs and
  refuses with `CoreError::InvalidValue { key: "set" }` naming the duplicated
  action — no new variant (A9 honored), one bounded line echoing a
  `ActionId::parse`-checked slug (A14 posture), deterministic offender (first
  duplicate in element order). Pinned by
  `a_base_binding_one_action_twice_refuses_naming_the_duplicated_action`
  (tests/apply.rs), which passes. The invariant is closed downstream too:
  `rebind` only mutates an existing element's input, so the produced base can
  never reintroduce a duplicated action and feeding a produced base back into
  `apply` stays admissible.
- *(b) duplicated-input target bypasses the `occupied-target` guard.* Closed.
  The occupancy check (apply.rs:506-516) now asks `foreign_holder` →
  `holders_of` — every holder of the target key in element order, not the first
  (apply.rs:362-367, 545-549). A matched co-holder still permits the legal
  walk-onto-own-vacated-keys, as the design requires. Pinned by
  `relocate_cluster_occupancy_asks_every_action_holding_the_target_key`
  (keyboard:s held by the matched `move.steer-right` first and the foreign
  `combat.fire` second) — this is exactly the bypass shape from round 1; under
  the old first-match read every precondition passed and the row landed. The
  test asserts the `occupied-target` verdict naming `keyboard:s` and byte-equal
  layout (atomicity). It passes; the spec.md occupancy clause was amended to
  state the every-holder rule, so a conformant non-Rust implementer produces
  the same verdict.
- *The elided swap-row tail of the same finding (duplicate bindings turning
  `swap-polarity` / `swap-mouse-buttons` into a no-op reported as applied)* is
  also closed by the duplicate-action refusal: with one element per action,
  `swap_polarity` and `swap_mouse` always move the one real copy, and the
  exchange is always felt (each action's single element changes input). The
  remaining `action_on` first-match use in `swap_mouse` is not exploitable:
  multi-holder keys are grammar-legal and kept by `resolve`, the row never
  removes a binding, never lands on a key in a way that deletes another
  holder, and cannot be made an identity (one element per action forbids the
  same action on both buttons).

## Security checklist over the diff

1. Boundary validation: improved — an ambiguous hostile-but-legal base is now
   refused at the entry boundary rather than silently mis-applied; grammar,
   size caps, and closed vocabularies untouched.
2. Path handling: no path handling in the diff.
3. Injection: refusal/offender strings echo only grammar-parsed ids
   (`ActionId::parse` slug grammar) into single bounded lines — A14 posture
   held.
4. Secrets/PII: none in code, tests, or fixtures.
5. New dependencies: none.
6. Filesystem writes: none.
7. Clean rejection: refusal is a typed `CoreError` naming the duplicate; the
   occupancy bypass yields a verdict with the row writing nothing
   (byte-equal layout asserted). `cargo test -p bindforge-transforms` green
   (29 apply.rs tests including both new pins, probe tests included).

## Open findings

None.

## FYI (no severity, out of specialty)

The proposal's entry-refusal list (Chosen design → The stack) still enumerates
three refusals while spec.md now states four; the amendment is accepted in the
spec, but a design-text reconciliation is not a security call.
