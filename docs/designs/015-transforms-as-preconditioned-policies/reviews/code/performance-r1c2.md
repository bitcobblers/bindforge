# Code review — performance seat, round 1 check 2 (ratification)

Scope: `git diff 39d4fcb589fd14ce2c81ea01572c84dbd4b28d39..HEAD -- ':(exclude)docs/designs'`,
read against the current `02-proposal.md`, `docs/adrs/015-transforms-as-preconditioned-policies.md`,
and `schemas/policy/v1/spec.md`. Purpose of this pass: confirm the post-approval fix
(`b15f7da fix(transforms): close the duplicate-binding guard gap in the stack`) did not
break the performance properties approved in round 1. My round-1 verdict held no findings.

Changed source/test paths under review: `crates/transforms/src/apply.rs`,
`crates/transforms/src/lib.rs`, `crates/transforms/tests/apply.rs`, `schemas/policy/v1/spec.md`.

## Verdict

**approve** — no open blocker or major findings. Zero performance findings: the fix's added
work is bounded by caps that already ship, sits on a once-per-application path, and does not
change the asymptotic cost of any lookup.

## What the fix adds, cost-wise

1. `refuse_entry` → `duplicated_action(&base.set)` (`crates/transforms/src/apply.rs:273-306`):
   one pass over the base's `set` with a function-local `BTreeSet<&ActionId>`.
2. `Layout::holders_of` + `Stack::foreign_holder` (`apply.rs:362-367`, `apply.rs:545-549`):
   the `occupied-target` check now asks every action holding a target input instead of the
   first one in element order.

Both are on the document-processing path, not an event path: `apply` is called by a future CLI
(ADR-017), never per keystroke.

## Checklist

1. **Event/keystroke path.** No fs or network calls added (`duplicated_action`, `holders_of`,
   `foreign_holder` are pure over already-loaded values). No per-event table rebuild: the
   `BTreeSet` in `duplicated_action` is scoped to one `refuse_entry` call and dropped
   immediately. One allocation per `apply`, and it is ordered *before* `Layout::new`'s
   `to_vec()` (`apply.rs:199` vs `apply.rs:206`) and after the three cheaper entry checks, so
   the new scan costs nothing when a cheaper refusal already fired. Pass.
2. **Complexity.** The concern for this change was whether asking every holder turns the
   occupancy check superlinear. It does not, and the bounds are shipped ones:
   - `|base.set| ≤ 256` — `parse_set` reads through `delta_elements`, which caps every delta
     list at `MAX_PATCH_DELTA_ELEMENTS = 256` (`crates/core/src/document.rs:1121-1136`,
     `1141-1148`), so a base — not just a patch — cannot carry thousands of elements;
   - `|matched| = |targets| ≤ MAX_RELOCATE_TARGETS = 16`, and the `match-count` guard
     (`apply.rs:479`) runs *before* the occupancy loop (`apply.rs:506`), so
     `matched.contains(*holder)` in `foreign_holder` is a ≤16-element linear test — constant,
     and `matched` is a `Vec<ActionId>` (`apply.rs:437`), so no map is warranted here;
   - rows ≤ `MAX_POLICY_TRANSFORMS = 64` (`document.rs:1364`).
   Worst case per application is therefore ≤ 64 rows × 16 targets × 256 elements ≈ 2.6×10⁵
   element compares, versus the same 256-element bound the round-1 code already paid; the
   delta is at most one extra partial element scan per target that is held only by matched
   actions (the legal walk-onto-own-vacated-keys case). Same asymptotics as what I approved;
   the documented "lookups are linear on purpose" posture (`apply.rs:313-314`) still holds and
   the new doc comment (`apply.rs:316-324`) restates it accurately rather than contradicting it.
   The entry dedup is O(|set| log |set|) ≈ 2×10³ compares, once. Pass.
3. **Unbounded growth.** No new cache, registry, or long-lived collection. `seen` in
   `duplicated_action` and the iterator adapter in `holders_of` are call-scoped; `Layout`
   remains a per-`apply` copy of a ≤256-element list. Pass.
4. **Startup.** Nothing moved into a loader or parse path: `load_policy`, `document::parse`,
   and the conformance walk are untouched by this diff, so boot reads no extra bytes and the
   new refusal only runs when a caller actually applies a stack. Pass.
5. **Cache invalidation.** No cache exists in the crate, so there is no invalidation logic to
   cover; the fix introduces none. The duplicate-binding refusal is in fact what keeps the
   single linear lookup the only answer to "what key is this action on", which is the property
   my round-1 pass relied on. Pass (nothing to cover).
6. **Benchmark-like test or timing note.** The diff adds no cost assertion, no timing test,
   and no new cost claim; `spec.md`'s "Input budget" paragraph (`schemas/policy/v1/spec.md:239-248`)
   is unchanged, and it stays accurate: it bounds documents read, rows, and predicate moves,
   and the fix adds no document read and no predicate move. Nothing new to run. Note on
   verification: the gates could not be executed from this review environment (`cargo` is not
   permitted for the review seat), so the numbers above are read off the shipped constants
   (`MAX_PATCH_DELTA_ELEMENTS`, `MAX_POLICY_TRANSFORMS`, `MAX_RELOCATE_TARGETS`) rather than
   measured. This limits the *precision* of my cost claim, not its direction: the caps bound
   the added work by construction, and no unbounded `n` was introduced.

## Also checked against round 1's approvals

- Determinism surface unchanged: the new refusal fires before any row runs, and `foreign_holder`
  walks element order, so which offender a report names stays fixed by the data.
- No new allocation per row or per target (`holders_of` borrows `self.elements`; the iterator is
  consumed in place).
- New tests (`crates/transforms/tests/apply.rs`, two cases) use the existing 5-element fixtures
  and the same `shipped_inputs()`/`load_default_registry()` setup as the rest of the suite; no
  fixture is inflated, so the suite's runtime is unchanged in kind.

## FYI (out of specialty, no severity)

`swap-mouse-buttons` still resolves each button through `Layout::action_on`, i.e. the *first*
holder (`apply.rs:350-355`, and the fix's new comment at `apply.rs:347-349` blesses it), while
the occupancy fix establishes that an input may legally hold several actions. If that asymmetry
is deliberate for the mouse row, the correctness/security seats may want it stated in
`spec.md`'s `swap-mouse-buttons` bullet (`schemas/policy/v1/spec.md:211-214`) the way the
`relocate-cluster` bullet now states the every-holder rule.
