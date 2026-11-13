# Code review — correctness, round 1 ratification pass c2

Scope: `39d4fcb589fd14ce2c81ea01572c84dbd4b28d39..HEAD`, excluding
`docs/designs/`. Read against `02-proposal.md`,
`docs/adrs/015-transforms-as-preconditioned-policies.md`, and my own round-1
findings. The change in range is the fix round c1 (`fix(transforms): close the
duplicate-binding guard gap in the stack`) plus the test-update that pins it:
`crates/transforms/src/apply.rs` (`refuse_entry`'s fourth refusal +
`duplicated_action`, `Layout::holders_of`, `Stack::foreign_holder`, the
`Layout`/`input_of`/`action_on`/`rebind` docs, module and crate docs),
`crates/transforms/src/lib.rs`, `schemas/policy/v1/spec.md`,
`crates/transforms/tests/apply.rs`.

**Verdict: approve.** No blocker or major finding, open or introduced. Nothing
the c1 fix touched regressed anything I approved in round 1.

## What the c1 fix changed, checked for breakage

- **Fourth entry refusal — a base that binds one action twice**
  (`apply.rs:273-283`, `duplicated_action` at `301-306`). This is the fix my
  round-1 F2 asked for (the "refuse at `refuse_entry`" branch). Checked:
  - It is a *declared* deviation from the proposal, which enumerates three
    entry refusals: `03-build-notes.md` → "Deviations from the proposal (c1)"
    records it, names the shipped `InvalidValue` variant (A9), and flags the
    consequence (a verbatim-duplicated `action input` pair is refused too).
    `schemas/policy/v1/spec.md:163-169` carries the rule, so the contract tree
    and the code agree for a third-party reader. No undeclared deviation, no
    scope creep: it refuses a shape the stack cannot answer, on the channel the
    design already uses for "could not evaluate".
  - It closes F2's silent-`Applied` hole rather than papering over it:
    `Layout::input_of`/`rebind` (`340-345`, `372-379`) are now exact, not
    first-of-several, because no second copy exists. `Layout::new` still copies
    the element list, so the produced `set` keeps the source base's element
    order untouched — no element is dropped, which the proposal's order rule
    requires.
  - Check order (`kind` → pin → id-collision → duplicate) leaves the three
    round-1 refusals answering exactly as before; every existing test base
    (`tests/common/mod.rs:65-70` `DEFAULT_SET`, and each case's own set) binds
    one action once, so no approved expectation moved.
  - The refusal message echoes only a grammar-checked `ActionId`, keyed `set`,
    id early in the line for the 80-char echo posture (A14). No new
    `CoreError` variant: `error.rs` untouched.
- **`holders_of` + `foreign_holder` (`362-367`, `545-549`)** — the
  `occupied-target` question is now asked of every holder of the target key.
  Checked against the proposal's one sentence and its "walking onto the
  cluster's own vacated keys is legal" clause: holders inside `matched` are
  still ignored, so the legal overlap survives (`apply.rs:98-121` still applies)
  and only the foreign second holder flips the verdict
  (`tests/apply.rs:315-347`). No over-tightening: a base with one key serving
  two actions is still transformable, not refused.
- **Docs claims vs code**: the `Layout` invariants, the `action_on` first-answer
  justification, the crate doc's entry-refusal enumeration (`lib.rs:19-24`) and
  the module doc (`apply.rs:33-39`) match the shipped behavior; the c1 notes'
  claims (`duplicated_action` is a `BTreeSet` first-repeat scan; the
  `occupied-target` wording change in `spec.md`) all check out against the
  source.
- **Grammar/model surface** is untouched by the fix, and I re-confirmed the
  pieces the fix leans on: `ActionRegistry::ids()` is `BTreeMap` key order
  (`action.rs:149,157-159`), so the match rule's byte order and the offender
  determinism claim hold; `TagPattern::matches`/`spelling` (`document.rs:461-486`)
  and the row grammar's fixed counts, bound `0..=4`, and no-repeat targets
  (`document.rs:1389-1469`) are as the contract states. The two new exported
  budgets and `load_policy` are re-exported at the core root
  (`lib.rs:93,102-103,114`); `zz_api_probe` resolves the promised names and pins
  the ten reason spellings.
- **Test side of the range**: the new suite cases pin both halves of the fix —
  the duplicated-action entry refusal (`tests/apply.rs:1132-1162`) and the
  foreign-second-holder verdict (`315-347`) — and the 29 `apply.rs` cases assert
  the same ten reasons, the same pairing, the same element order, and the same
  produced-base rules as the round-1 approved suite. No previously approved
  assertion was moved to accommodate the fix.

## Still open from round 1 (unchanged by this range; minors, not blocking)

The c1 notes say the correctness minors were untouched here, and that is what I
find — F2 is fixed and pinned; the rest are exactly as I recorded them.

1. **F1 — offender contract mismatch** (`apply.rs:103-109` + `fault()` at
   `766-772`; `schemas/policy/v1/spec.md:216-222`; proposal → The report).
   spec.md still says a report carries "an offender that is always a
   grammar-checked id — never raw document text", while every `fault()` call
   fills it with a composed sentence, and the field doc still reads "The id,
   tag, or key the row tripped over". `tests/common/mod.rs:346-355` now says
   out loud that "which sentence names it is not part of the contract" — which
   is the opposite of what spec.md states a third-party implementer must
   produce.
2. **F3 — the untouchable guard misses an unbound matched action in
   `swap-polarity`** (`apply.rs:583-593`, guard at `644-652`). `moves` is still
   built by `filter_map` over *bound* matched ids, so `guard_fault`'s
   `docked_at(action)` leg never sees a matched id the base leaves unbound; the
   row then falls through the `(Some, Some)` test at `595-600` and reports
   `Applied` having written nothing, where spec.md:183-186 promises
   `reserved-input` for "a row that touches an action named by the manifest's
   `docked-actions`". Repro: `swap-polarity move.steer-*`, base binds only
   `move.steer-right keyboard:d`, manifest carries
   `docked-actions = ["move.steer-left keyboard:m"]`.
3. **F4 — `seeded_from` carry-forward** (`apply.rs:748`). Still declared only in
   `03-build-notes.md` (r0 deviations, with a reason); `spec.md` → The output
   (`226-230`) still enumerates only `kind`, `id`, `derives`, `for`, `set`,
   `generated-by`, so a non-Rust implementer ships a produced document that
   differs from this one.
4. **F5 — entry-refusal message for a `kind`-less document**
   (`apply.rs:243-249`): the test is `!matches!(base.kind,
   Some(ProfileKind::Base))`, the message is the fixed "`patch` is not a base".
5. **F6 — stale doc count** (`apply.rs:113-114`): "A fourteenth reason is a
   `policy/v2` question" against the ten-arm set the same file enumerates
   (`116-152`, pinned at `tests/zz_api_probe.rs:85-96`).
6. **F7 — fixture contract comment**
   (`schemas/policy/v1/fixtures/valid/exact-tag-policy.toml:1-6`) still reasons
   about `move.strafe-left` / `move.strafe-left-extra` while the rows carry
   `swap-polarity move.steer-left` and `relocate-cluster move.strafe-right …`.

## New in this range (minor scope: only what the fix made contract-visible)

- **`swap-mouse-buttons` over a one-key-many-actions base is left unstated in
  the contract** (`apply.rs:350-355`, `614-638`; `spec.md:211-214`). c1 made a
  base that binds one input to several actions an explicitly legal, pinned
  shape (and `spec.md` now says so for the occupancy clause), and its own probe
  case shows the mouse row exchanging only the action `action_on` finds first
  while the other holder stays put — a reasonable reading, but a third-party
  implementer reading "the actions currently bound to `mouse:left` and
  `mouse:right`" may move both, and the produced bases differ. One sentence in
  `spec.md`'s `swap-mouse-buttons` clause (first holder in `set` order, the
  rest untouched) closes it. Nit: the row does no mangling and is its own
  inverse either way.

## FYI (out of specialty, no severity)

- `03-build-notes.md` → c1 says "Same 317 as r1's green run" and lists the two
  hostile-base cases as "still to pin"; the test-update commit in this range
  pinned both (`apply.rs` now carries 29 cases), so that section and the count
  are stale. Docs are outside the reviewed range.
