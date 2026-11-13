# Code review — testing seat — r1c2 (ratification)

- Verdict: **approve**
- Scope: `git diff 39d4fcb589fd14ce2c81ea01572c84dbd4b28d39..HEAD -- ':(exclude)docs/designs'`,
  read against the full current `02-proposal.md` and the ADR. The diff is
  `crates/transforms/src/apply.rs` (+121/−23), `crates/transforms/src/lib.rs` (doc
  only), `crates/transforms/tests/apply.rs` (+104), `schemas/policy/v1/spec.md`
  (+15): the accepted round-1 security major's fix — `refuse_entry` now refuses a
  base whose `set` binds one action twice (`duplicated_action`, apply.rs:273-306),
  and the relocate occupancy check asks every holder of a target key
  (`holders_of`/`foreign_holder`, apply.rs:362-367, 506-516, 545-549) — plus two
  tests pinning both.
- This pass checks only that the later fixes did not break what round 1 approved,
  plus the testing quality of the fix itself.

## The fix's tests (checklist item 3 — revision round)

The accepted round-1 security major now has tests that would have caught it:

- `relocate_cluster_occupancy_asks_every_action_holding_the_target_key`
  (tests/apply.rs:316-349) reproduces the reported bypass exactly: `keyboard:s`
  is the cluster's own vacated source key (legal overlap), its first holder in
  array order is the matched `move.steer-right`, and `combat.fire` — never named
  by the row — holds the same key second. Under the pre-fix first-holder code
  this row would have reported `Applied`; the test asserts `occupied-target`
  naming `keyboard:s` and a byte-equal layout. It runs and passes.
- `a_base_binding_one_action_twice_refuses_naming_the_duplicated_action`
  (tests/apply.rs:1133-1162) pins the entry-refusal half: the duplicated
  `move.steer-left` base answers on the `CoreError` channel naming the action,
  with the helper's one-line/no-control-character checks. Under the pre-fix code
  the row would have reported `Applied` over a layout whose effective binding
  (`resolve`, last-writer-wins) never changed — the silent no-op the finding
  reported. It runs and passes.
- Both arms of the new `foreign_holder` filter are pinned across the suite: the
  matched-holders-are-legal arm by the pre-existing apply case
  (tests/apply.rs:98-121, steering onto its own `keyboard:s`) and the
  foreign-holders-reported arm by the pre-existing single-holder case
  (tests/apply.rs:274-297) plus the new multi-holder case. No new branch is
  untested; `duplicated_action`'s Some/None arms are both exercised (27 of 29
  apply-suite bases have unique actions).

## Did not break what was approved (regression check)

- No source or test path outside the fix is touched: `crates/core` (including the
  document.rs:1431 target-list no-repeats refusal and the fixture corpus) is
  byte-unchanged, and the relocated refactor keeps `action_on`'s first-answer
  semantics for `swap-mouse-buttons` verbatim.
- The whole workspace suite is green, all 20 test targets, with every
  round-1-approved assertion still present and passing — transforms apply suite
  29 passed (27 pre-existing + 2 new); conformance walk, grammar closure, version
  gate, and the stack-semantics cases all pass unmodified.
- The occupancy refactor is behavior-identical on bases with no duplicated
  input-holder (the only behavior change is the intended one), which the passing
  pre-existing suite confirms.
- Test discipline of the new cases: both cite `// proves: design 015's Test
  strategy #4 / #8 … as amended by the accepted round-1 security finding`; names
  describe the contract behavior the amended spec.md spells; determinism holds
  (scratch dirs, shipped contract data, no clock- or network-dependent
  assertions).

## Checklist item 6 — gates (round-1 blocker-of-evidence, now resolved)

This seat could not run shells in round 1. In this pass they ran. All three
gate commands green on the workspace (Linux):

- `cargo fmt --all --check` — no output (clean).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` —
  `Finished `dev` profile [unoptimized + dev profile] target(s)` — no warnings.
- `cargo test --workspace --locked` — tail:

  ```
  Running tests/policy.rs            test result: ok. 26 passed; 0 failed
  Running tests/reach_tables.rs      test result: ok. 5 passed; 0 failed
  Running tests/reach_v2.rs          test result: ok. 18 passed; 0 failed
  Running tests/version_gate.rs      test result: ok. 11 passed; 0 failed
  Running tests/zz_api_probe.rs (core)   test result: ok. 5 passed; 0 failed
  Running tests/apply.rs (transforms)    test result: ok. 29 passed; 0 failed
  Running tests/zz_api_probe.rs (transforms) test result: ok. 2 passed; 0 failed
  Doc-tests bindforge_core           test result: ok. 0 passed; 0 failed
  Doc-tests bindforge_transforms     test result: ok. 0 passed; 0 failed
  ```

  (all 20 targets reported `ok`, 0 failed; filtered run confirms both new tests
  execute: `... a_base_binding_one_action_twice_refuses_naming_the_duplicated_action ... ok`
  and `... relocate_cluster_occupancy_asks_every_action_holding_the_target_key ... ok`.)

## Round-1 findings status

- #5 (gates un-evaluable from this seat) — resolved by this pass's green runs above.
- #1 (no fixture/test for the relocate target no-repeats refusal), #2 (dead
  `None` arm of the axis-kind match, now apply.rs:578), #3 (untouchable-guard
  path of `swap-polarity`, apply.rs:591, untested), #4 (target arm of
  `first_not_keyboard`, apply.rs:807-811, untested), #6 (axis-kind test docstring
  nit) — untouched by the fix and unchanged; they remain open at their approved
  severities and are not re-litigated in this pass.

## New findings on the fix itself

1. **nit** — apply.rs:287-306 / tests/apply.rs:1133-1162: the determinism claim
   in the docstring ("The **first** action a base's `set` answers twice") is not
   pinned — the refusal test carries exactly one duplicated action, so which of
   two differently-duplicated actions a report names is unpinned, against the
   suite's posture that offender naming is part of the determinism surface. Fix:
   extend the refusal test's base with a second duplicated action later in the
   array and assert the message names the first one.
2. **nit** — apply.rs:347-355 / tests/apply.rs (mouse cases): the amended model
   now documents first-answer `action_on` as intended `swap-mouse-buttons`
   semantics ("the action it exchanges is the one it found on the button") while
   the grammar admits a base binding one mouse button to two actions; no test
   pins that multi-holder mouse shape (what exchanges, what stays, byte-equal
   otherwise). Fix: add one `swap-mouse-buttons` case with a third action also
   holding `mouse:left`, asserting the documented first-answer reading.

## FYI (no severity)

The proposal's Chosen design → The stack still spells the pre-amendment rules —
three entry refusals (02-proposal.md:331-336) and the occupancy clause without
the every-holder wording (02-proposal.md:367-369) — while `schemas/policy/v1/spec.md`
(now the contract text with the amendment) and the code/tests carry the amended
form; the new tests cite the amendment by its review provenance. Design-document
currency is other seats' call.
