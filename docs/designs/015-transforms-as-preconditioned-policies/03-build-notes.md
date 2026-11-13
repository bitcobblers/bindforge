# Build notes — 015 transforms as preconditioned policies

Implementation of `02-proposal.md` on `pipeline/015-transforms-as-preconditioned-policies`,
on top of the test commit `7080008`.

## What shipped

- `crates/core/src/document.rs` — the `policy` parse arm (`POLICY_V1_ENTRIES`),
  the `Policy` / `Transform` / `TagPattern` model, the row grammar
  (`parse_transforms`, `parse_transform`, `parse_tag`, `parse_bound`), the
  `Policy` arm of `Document`, `serialize_policy` (canonical re-spelling, array
  order preserved), and the two exported budgets
  `MAX_POLICY_TRANSFORMS = 64`, `MAX_RELOCATE_TARGETS = 16`.
- `crates/core/src/policy.rs` — `load_policy`: slug check before any filesystem
  question, `<id>.toml` in the major directory, stem ↔ `id` agreement,
  `check_dir_version`, format-arm refusal for a non-policy document.
- `crates/core/src/lib.rs` — re-exports (`Policy`, `Transform`, `TagPattern`,
  `load_policy`, the two constants).
- `crates/transforms/` — the new crate: `Cargo.toml` (depends on
  `bindforge-core` and nothing else), `src/lib.rs`, `src/apply.rs` (the
  sequential atomic stack, the untouchable guard, the match rule, the ten
  `NotApplicableReason` spellings, the produced base).
- `schemas/policy/v1/spec.md` + `schemas/policy/v1/fixtures/{valid,invalid}/` —
  6 valid and 30 invalid fixtures, exactly the names the contract lists.
- `schemas/README.md` — `policy` in the format list, the `policy/v1` scaffold
  sentence, the ADR-015 budget bullet, "all nine constants" → "eleven".
- `Cargo.lock` — one new package (`bindforge-transforms`), committed as the gate
  requires (`--locked`).

## Gate legs, run in this workspace

`cargo fmt --all` was run while working; the check leg:

```
$ cargo fmt --all -- --check
(no output)
```

`cargo clippy --workspace --all-targets -- -D warnings` — the product code is
clean (`cargo clippy --workspace --lib --bins -- -D warnings` → `Finished`), but
the gate fails inside three **test files** (see "Test-side defects"):

```
error: redundant pattern matching
  --> crates/transforms/tests/zz_api_probe.rs:73:9
error: could not compile `bindforge-transforms` (test "zz_api_probe") due to 1 previous error
error[E0308]: mismatched types
   --> crates/core/tests/policy.rs:169:45
    | .filter(|(name, _)| !names.contains(name))
    |                    expected `&&str`, found `&String`
error[E0308]: mismatched types
   --> crates/transforms/tests/apply.rs:938:9
    | Document::Profile(profile) => profile,
    | expected `Result<Document, CoreError>`, found `Document`
error: could not compile `bindforge-transforms` (test "apply") due to 1 previous error
error: could not compile `bindforge-core` (test "policy") due to 1 previous error
```

`cargo test --workspace --locked` reports the same two compile failures — cargo
stops the run there, so no suite result is printed for the whole workspace. Run
target by target, every suite that compiles is green:

```
$ cargo test --workspace --locked --offline --lib --test action --test action_ids \
    --test chain_guard --test chain_ops --test conformance --test gate_contract \
    --test grammar_closure --test input --test manifest --test parse_budget \
    --test placement --test reach_tables --test reach_v2 --test version_gate \
    --test zz_api_probe --no-fail-fast
core   lib 0 · action 16 · action_ids 14 · chain_guard 4 · chain_ops 74 ·
         conformance 9 · gate_contract 8 · grammar_closure 15 · input 6 ·
         manifest 49 · parse_budget 11 · placement 17 · reach_tables 5 ·
         reach_v2 18 · version_gate 11 · zz_api_probe 5
transforms  lib 0 · zz_api_probe 2
→ 264 passed, 0 failed
```

`conformance.rs` (the `schemas/*/v*/` walk), `grammar_closure.rs` and
`version_gate.rs` are the suites that exercise the new corpus, and they pass:
`policy/v1` joins the walk, its 36 fixtures parse-or-refuse as the contract
states, every refusal is one bounded line, and `format_majors("policy")` is
`["v1"]` with the crate-wide majors union unchanged.

## Test-side defects (why `testsNeedUpdate` is true)

The two ADR-015 suites never compiled, so the stack could not be run against
them from inside the repo. Each defect is in a test file this stage may not
edit; the minimal fix for each is:

1. `crates/core/tests/policy.rs:169` — `!names.contains(name)` where
   `names: &[&str]` and `name: &String`; `slice::contains` wants `&&str`, so the
   target fails with `E0308`. Minimal fix:
   `!names.iter().any(|entry| *entry == name.as_str())`.
2. `crates/transforms/tests/apply.rs:938` — the `match` arms pattern-match a
   `Document` against `bindforge_core::parse(&written)`, whose type is
   `Result<Document, CoreError>`; `E0308`. Minimal fix: wrap the arm pattern,
   `Ok(Document::Profile(profile)) => profile,`.
3. `crates/transforms/tests/apply.rs:213` and `:225` — the fixture binds
   `move.steer-left` to `gamepad:north`, and `north` is not a member of the
   closed `gamepad:` vocabulary (`dpad-up` is), so `profile_of` panics before the
   assertion runs. Minimal fix: `gamepad:dpad-up` in the `set` element and in the
   `assert_offender_names` needle.
4. `crates/transforms/tests/zz_api_probe.rs:73` — `matches!(refused, Err(_))`
   trips `clippy::redundant_pattern_matching`, and the gate runs clippy with
   `-D warnings`, so the lint is a gate failure. Minimal fix:
   `refused.is_err()`.

Each fix was verified **outside** the repo, in copies of the committed suites
(`crates/transforms/tests/common/mod.rs` and `crates/core/tests/common/mod.rs`
copied beside them, their `repo_root` pointed at this workspace so the shipped
contract data is still what the suites read):

- a copy of `crates/core/tests/policy.rs` with fix (1) → `test result: ok. 26
  passed; 0 failed`, including
  `the_policy_corpus_ships_every_fixture_the_contract_names`,
  `round_trips_every_valid_policy_document`, and every cap/grammar/refusal case;
- a copy of `crates/transforms/tests/apply.rs` with fixes (2) and (3) →
  `test result: ok. 27 passed; 0 failed` — all ten `NotApplicableReason` cases,
  atomicity, the match rule, the produced base, and the determinism case.

So the implementation answers the suites as written; only those four lines are
in the way.

## Deviations from the proposal

- **`TagPattern::spelling()` is public.** The proposal leaves it unlisted; the
  `match-count` report quotes the author's tag, and the tag is typed away after
  parse, so core has to answer the spelling. It re-renders from the typed
  pattern, so it cannot disagree with the document.
- **The produced base copies the source base's `seeded_from`.** The proposal
  lists `kind`, `id`, `derives`, `for`, `set`, `generated-by`; a base seeded
  from a built-in preset would otherwise lose that provenance through one
  application. It stays provenance only, never a selection key (014's
  Decision Four).
- **The untouchable guard runs after `not-keyboard` and before
  `not-contiguous`.** The proposal states the guard as an unconditional rule
  without pinning its position in the per-kind order. Within the guard, the
  docked-action check precedes the reserved-source check, which precedes the
  reserved-target check, so the offender a report names is the first thing the
  row touches, in row order.
- **`is_cluster` answers "yes" when a source has no grid coordinate.** A
  missing coordinate is a data gap, not a disconnected cluster: the row then
  reaches `placement_is_legal`, whose `InvalidValue` propagates, exactly what
  the spec's "a `keyboard:*` id the grid does not place … propagates the
  predicate's refusal" clause requires. Reporting `not-contiguous` here would
  hide a contract gap behind an application verdict.
- **Row order inside `relocate` follows the proposal's list**: empty match →
  `unbound-action` → `match-count` → `not-keyboard` → guard → `not-contiguous`
  → `occupied-target` → the predicate. An unbound matched id is therefore
  reported even when the target count also disagrees.
- **The stack's work sits on a private `Stack<'a>`** (one row of registry,
  tables, geometry, guard, and working layout) rather than free functions taking
  six arguments each — `clippy::too_many_arguments` is a `-D warnings` failure.
  `apply`'s public signature is exactly the proposal's.
- **Entry-refusal message shape.** The pin-mismatch refusal is keyed
  `InvalidValue { key: "for" }` and quotes both pins inside 80 octets, because
  `CoreError::one_line` truncates each field at 80 and a truncated pin would
  name neither side. The patch-as-base and id-collision refusals key `kind` and
  `id` respectively; no `CoreError` variant was added (A9).
- **`MAX_GRADE_BOUND` is a private constant** in `crates/core/src/document.rs`
  (verified at close-out: it sits beside the row grammar, which validates a
  bound as `0..=4` against the shipped severity-class count); the proposal
  exports only the two budget constants.

## Not done here

Nothing was moved, so `moves` is empty; no rename was needed. `diff` across a
transformation, the produced-chain location, and the policy CLI stay Deferred in
the proposal (ADR-016/017/018).

---

# Fix round r1 — the gate after `f290e19` (test-update)

Re-run in this workspace against HEAD (`f290e19`). The `.github/workflows/check.yml`
gate is `npm run check` plus the three cargo legs; every leg was executed here.

**`npm run check` (`tools/orchestration`) — green.**

```
$ npm run check            # lint + prettier --check + vitest
Checking formatting...
All matched files use Prettier code style!
 Test Files  42 passed (42)
      Tests  505 passed | 2 skipped (507)
```

**`cargo fmt --all -- --check` — green** (no output). `cargo fmt --all` changes
nothing to commit; the red leg is not formatting.

**`cargo test --workspace --locked` — green.** The two suites that could not
compile in r0 now compile and pass, so the r0 "Test-side defects" list is fully
resolved on the test side:

```
core        lib 0 · action 16 · action_ids 14 · chain_guard 4 · chain_ops 74 ·
            conformance 9 · gate_contract 8 · grammar_closure 15 · input 6 ·
            manifest 49 · parse_budget 11 · placement 17 · policy 26 ·
            reach_tables 5 · reach_v2 18 · version_gate 11 · zz_api_probe 5
transforms  lib 0 · apply 27 · zz_api_probe 2
→ 317 passed, 0 failed; doc-tests 0; every target compiles
```

**`cargo clippy --workspace --all-targets -- -D warnings` — red**, five hits, all
five inside one test file:

```
error: using `contains()` instead of `iter().any()` is more efficient
   --> crates/core/tests/policy.rs:169:30   (clippy::manual_contains)
error: this expression creates a reference which is immediately dereferenced by the compiler
   --> crates/core/tests/policy.rs:283:26   (clippy::needless_borrow)
error: doc list item without indentation
   --> crates/core/tests/policy.rs:455:5, :456:5, :457:5   (clippy::doc_lazy_continuation)
error: could not compile `bindforge-core` (test "policy") due to 5 previous errors
```

Nothing under `src/` is implicated: `cargo clippy --workspace --lib --bins -- -D
warnings` → `Finished`, and `cargo clippy -p bindforge-transforms --all-targets
-- -D warnings` → `Finished`. These three lints are syntactic facts about
`crates/core/tests/policy.rs` — a filter closure, an argument borrow, a doc
comment — so no implementation edit in `crates/core/src` or
`crates/transforms/src` can clear them, and suppressing them with a workspace
`[lints]` allowance would gut the `-D warnings` posture check.yml pins. That is
why this round answers `testsNeedUpdate: true` instead of touching the file: the
fix is a test-engineer edit, and this stage reverts test paths anyway.

## The three test-side edits that turn the gate green

Each is one line (or three doc-comment lines), rustfmt-shaped, no assertion
touched:

1. `crates/core/tests/policy.rs:169` — `clippy::manual_contains`. r0's suggested
   `!names.iter().any(|entry| *entry == name.as_str())` is what trips the lint;
   clippy's own spelling is the right one: `!names.contains(&name.as_str())`
   (`names: &[&str]`, so `&&str` is what `contains` wants).
2. `crates/core/tests/policy.rs:283` — `clippy::needless_borrow`. `err` is
   already `&CoreError`: `assert_message_shape(err)`.
3. `crates/core/tests/policy.rs:453-457` — `clippy::doc_lazy_continuation` ×3.
   The `proves:` comment starts line 454 with `+ A19 — …`, which makes 455-457 a
   lazy continuation of a list item. The file's own convention elsewhere (lines
   341, 380, 466, 493, 643) ends the `proves:` line with `+` and opens the next
   line with `A19 —`; adopting it here removes all three hits and matches the
   rest of the suite. (Indenting 455-457 by two spaces is clippy's other
   suggestion and is equally clean.)

Verified outside the repo: a standalone probe crate reproducing all three
shapes (`/tmp/opencode/lintprobe`, edition 2024, the same clippy 1.99.0) comes
out clean under `cargo clippy -- -D warnings`, so the three edits compile and
introduce no follow-on lint. After them the workspace should land green: the
remaining legs are already green above.

## Deviations from the proposal (r1)

- None. This round changed no implementation code and no design surface; the r0
  deviation list above stands unchanged. The only file this round touches is
  this one.

---

# Fix round c1 — the security seat's major finding (code review round 1)

Opening finding: `reviews/code/security-r1c0.md` — *first-match `Layout`
semantics let a hostile-but-legal base defeat the guards*
(`crates/transforms/src/apply.rs:276-309, 436-449, 512-519, 552-553`). The other
round-1 seats approved; their minors (correctness F1-F7) are untouched here,
since this stage answers the security seat's finding.

## What changed

`crates/transforms/src/apply.rs`

- **`refuse_entry` gained a fourth entry refusal**: a base whose `set` binds one
  action twice is `InvalidValue { key: "set" }` naming the duplicated action
  (`duplicated_action`, a `BTreeSet` first-repeat scan — no new `CoreError`
  variant, A9). `apply`'s whole model is one binding per action: preconditions
  read `input_of` and writes land through `rebind`, and with two copies of one
  action those two answers are different elements. `resolve` settles the
  duplicate last-writer-wins, so the old behaviour moved the *stale* copy, left
  the effective copy alone, and reported `Applied` — the silent no-op the design
  refuses. Refusing rather than folding also keeps the "the produced `set`
  preserves the source base's element order" clause untouched: no element is
  ever dropped, so the order question the reviewer flags for the folding
  alternative never arises. This is the fix both seats asked for
  (security-r1c0 Fix, correctness-r1c0 F2).
- **`Layout::holders_of` (new) + `Stack::foreign_holder` (new)**: the
  `occupied-target` loop now asks *every* action sitting on a target input, not
  the one the element list happens to carry first. A base may bind one input to
  two actions — the grammar does not forbid it and `resolve` keeps both holders —
  so the old first-answer check passed whenever the in-match-set holder came
  first, and the row then landed a matched action onto a key a never-named action
  still held. The verdict keeps its shape: `occupied-target` naming the target,
  with the foreign holder quoted.
- **`Layout`'s two invariants are now written down** (module doc, the `Layout`
  doc, and the `input_of` / `action_on` / `rebind` docs): one element per action,
  settled by the entry refusal, which is why the two swap rows can no longer
  rewrite a hidden copy and call it a swap; and an input may serve several
  actions, which is why occupancy is a `holders_of` question. `action_on` stays,
  and its doc says why the first answer is the right one there — the mouse row
  names the buttons and exchanges the action it found on each.

`crates/transforms/src/lib.rs` — the crate doc's entry-refusal enumeration gains
the new case. `schemas/policy/v1/spec.md` — "Applying the stack" states the
fourth entry refusal and rewords the `occupied-target` clause as
"every action holding that key is asked", so a non-Rust implementer produces the
same refusal and the same verdict.

The `relocate` check order is unchanged (the security FYI about the guard
running after `not-keyboard` stays as recorded in r0's deviation list — it is
out of specialty and write-free either way).

## Why the duplicated-input base is a verdict, not a refusal

The reviewer's cheapest shape was to refuse *both* shapes at the entry
("…or binds one input twice"). This round refuses only the duplicated-action
base, and answers the duplicated-input base with `occupied-target` naming the
input — the outcome the finding's own pinned-test sentence asks for ("duplicated-input
`occupied-target` refusal naming the input"). Refusing a base that puts two
actions on one key would deny the stack legal, meaningful documents — such a base
resolves cleanly and transforms cleanly once occupancy reads all its holders —
and it is stricter than the "same answer `resolve` gives" principle the finding
states. The `holders_of` check closes the bypass without narrowing the domain.

## Gate legs, run in this workspace

```
$ cargo fmt --all -- --check
(no output)

$ cargo clippy --workspace --all-targets -- -D warnings
Checking bindforge-transforms v0.1.0 (/home/mike/repos/bindforge/crates/transforms)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s
```

(leg printed clean; r1's five `crates/core/tests/policy.rs` lint hits are gone —
the test-update commit `39d4fcb` cleared them, so the whole-workspace `-D
warnings` leg passes with no source-side suppression.)

```
$ cargo test --workspace --locked
core        lib 0 · action 16 · action_ids 14 · chain_guard 4 · chain_ops 74 ·
            conformance 9 · gate_contract 8 · grammar_closure 15 · input 6 ·
            manifest 49 · parse_budget 11 · placement 17 · policy 26 ·
            reach_tables 5 · reach_v2 18 · version_gate 11 · zz_api_probe 5
transforms  lib 0 · apply 27 · zz_api_probe 2
→ 317 passed, 0 failed; every target compiles
```

Same 317 as r1's green run: no existing assertion moved.

## Verified outside the repo

Nothing under `tests/` was touched, so the two hostile bases were pinned with a
throwaway probe crate (`/tmp/opencode/hostile-probe`, path-deps on the two
workspace crates, the shipped `reach-classes/v2` tables and `ansi-us` grid):

```
case 1 refused: `set` is invalid: `move.steer-left` is bound twice in the base's `set`: a transform reads and writ…
case 2 verdict: occupied-target — `keyboard:s` is bound to `combat.fire`, which the row never named
case 3 outcome: Applied, layout: [("move.steer-left", "keyboard:s"), ("move.steer-right", "keyboard:d"), ("combat.fire", "keyboard:q")]
case 4 refused: `set` is invalid: `move.steer-left` is bound twice in the base's `set`: …
case 5 refused: `set` is invalid: `combat.fire` is bound twice in the base's `set`: …
case 6 outcome: Applied, layout: [("combat.fire", "mouse:right"), ("ui.menu", "mouse:left"), ("aim.camera", "mouse:left")]
```

- case 1 — the finding's example base `["move.steer-left keyboard:a",
  "move.steer-right keyboard:s", "move.steer-left keyboard:z", "move.steer-right
  keyboard:x"]` → entry refusal naming `move.steer-left` (was: `Applied` over a
  layout `resolve` never produces).
- case 2 — the guard-bypass base, target `keyboard:s` held by `move.steer-right`
  (in the match set, first) and `combat.fire` (foreign, later) →
  `occupied-target` naming `keyboard:s` (was: `Applied`).
- cases 4/5 — the two swap rows over a base carrying a second copy of one action
  (`swap-polarity`, `swap-mouse-buttons`) → refused; the first-match no-op the
  finding names is structurally gone.
- cases 3/6 — the shape is not over-tightened: a plain applicable base still
  applies, and one key serving two actions stays a transformable base.

## The two cases the test engineer still has to pin (`testsNeedUpdate`)

`crates/transforms/tests/apply.rs` carries no base with a duplicated action or a
duplicated input, so neither half of the finding is pinned in-repo yet. The
implementation's answers, ready to assert:

1. `set` naming one action twice (e.g. the reviewer's four-element WASD base) →
   `apply` returns `Err`, `assert_entry_refusal` with needles
   `["move.steer-left"]` (and `set`): the message is
   "`set` is invalid: `<action>` is bound twice in the base's `set`: …", one
   bounded line, the id early so the 80-char echo keeps it.
2. a target input bound to a matched action *and*, later in `set`, to a foreign
   action (e.g. `["move.steer-left keyboard:a", "move.steer-right keyboard:s",
   "combat.fire keyboard:s"]` under `relocate-cluster move.steer-* left left 0
   keyboard:s keyboard:d`) → row 0 reports `occupied-target`, and
   `assert_offender_names` passes on `["keyboard:s"]` (the message also names the
   foreign holder). A second assertion worth having: the produced layout equals
   the source's (the row wrote nothing).

## Deviations from the proposal (c1)

- **A fourth entry refusal.** The proposal's Chosen design → The stack enumerates
  three entry refusals (kind, pin, `policy.id == base.id`); a base that binds one
  action twice is now refused too. Both round-1 code seats asked for it, the
  variant is the shipped `InvalidValue` (A9), and `schemas/policy/v1/spec.md` now
  states it so the contract tree and the code agree. Consequence worth knowing: a
  base repeating the *same* `action input` pair verbatim is refused as well — a
  degenerate document `resolve` would shrug off, and one the stack refuses rather
  than guesses about.
- **The `occupied-target` clause is read over every holder of the target key.**
  The proposal's one sentence ("a target already bound to an action outside the
  match set") is satisfied; what was unspecified was whether "bound" meant the
  first element or any element. The implementation asks all of them — the
  proposal's own "clause (a) checks the final set's distinctness" note shows the
  one-key-one-landing posture on the write side, and the guard exists to refuse
  the displacement, which the first-answer reading missed.
- Nothing else moved: `schemas/policy/v1/` fixtures, the corpus count, the ten
  reason spellings, the budgets, and every other r0 deviation stand as recorded.

---

# Round-1 close-out — test-update r1 and the c2 ratify

Verified in this workspace against HEAD (the code stamp the pipeline records as
`e90525d`; `testsGreen: true`, all five code seats approve it). No implementation
code changed after c1 — the two unpinned cases c1 listed were pinned test-side.

**The two cases c1 left to the test engineer are now pinned** in
`crates/transforms/tests/apply.rs` (test-update r1):

- `a_base_binding_one_action_twice_refuses_naming_the_duplicated_action` — the
  reviewer's duplicated-action base refuses on the entry channel, needles
  `move.steer-left` (c1 probe case 1, and the swap-row no-op shape of
  cases 4/5 is covered by the same refusal).
- `relocate_cluster_occupancy_asks_every_action_holding_the_target_key` — a
  target held first by a matched action and later by `combat.fire` answers
  `occupied-target` naming `keyboard:s`, with the added assertion that the
  produced layout equals the source's (c1 probe case 2). The not-over-tightened
  shape (case 3/6: one key serving two actions stays a transformable base) is
  exactly this base's entry behavior — it resolves and answers a verdict, not a
  refusal.

**Ratify pass c2 (all five seats approve; no fix round followed).** The
correctness, compatibility, and testing minors/nits that c2 accepted without a
code change, and that therefore stay as built (all recorded in the proposal's
As-built note): the `offender` sentence vs spec.md's "always a grammar-checked
id" wording; `swap_polarity`'s docked guard leg running only over bound matched
ids (an unbound docked match reports `Applied` writing nothing); `spec.md` →
The output still omits the `seeded_from` carry-forward; the mouse-button
first-holder exchange reading is not stated in the spec; the "fourteenth
reason" doc comment at `apply.rs:113` (the closed set is ten); the stale
comment in `exact-tag-policy.toml`; the fixed "`patch` is not a base" message
when the document carries no `kind`; and the two determinism details the
compatibility seat flagged — the duplicated-action naming tie-break (first in
`set` element order) and which foreign holder `occupied-target` names (first
outside the match set in `set` order) — stated nowhere in the spec. Still-open
test gaps from testing-r1c0: no invalid fixture pins the `relocate-cluster`
target "no repeats" rule, the `not-keyboard` target arm is untested (only the
source arm is), and the guard has no `swap-polarity` case.

**One r0 correction made during this verification**: `MAX_GRADE_BOUND` lives in
`crates/core/src/document.rs`, not `apply.rs` — the r0 deviation bullet above
was amended in place. `02-proposal.md` gained its dated As-built note (the
first pass; there was none to verify or finish) recording the same drift set
against the design sections it touches.

---

# Close-out pass 2 — the as-built note verified (docs-only)

Scope: the whole run diff (`1af1202` … `22de1c1`, the branch head) read against
`02-proposal.md`. An As-built note already exists, so this pass verifies it
rather than restating it, and completes this file's record of the shipped tree.
**No implementation, fixture, or test file was touched here** — only these two
design records — so the gate legs printed in the c1 section stay the last
executed ones; nothing was re-run to change them.

## The drift set stands, and where it now sits

The newest code commit on the branch is `b15f7da`; `2761d6d` and `22de1c1` after
it carry review and design records only. Every As-built bullet re-reads true
against the shipped files:

| Recorded drift | Verified at |
|---|---|
| Fourth entry refusal (a base binding one action twice, `InvalidValue { key: "set" }`) | `crates/transforms/src/apply.rs:273-306`; `spec.md:163-169` |
| `occupied-target` asks every holder of the target key | `apply.rs:506-516, 362-367, 545-549`; `spec.md:195-199` |
| `swap-polarity`'s docked guard leg sees bound matched actions only | `apply.rs:583-593` |
| `offender` carries a bounded sentence with ids quoted, not a bare id | `apply.rs:683-689` and the `fault` call sites, against `spec.md:220-222` |
| The produced base also carries `seeded_from` | `apply.rs:748`; absent from `spec.md:226-230` |
| `Transform::kind()`, `TagPattern::spelling()` public; `MAX_GRADE_BOUND` private | `crates/core/src/document.rs:405, 480, 180` |
| Guard after `not-keyboard`, before `not-contiguous`; inner order docked → source → target; `is_cluster` answers "yes" for a missing coordinate; private `Stack`, designed `apply` signature | `apply.rs:490-505, 644-661, 700-710, 191-233, 424-430` |
| `swap-mouse-buttons` exchanges the first holder of each button | `apply.rs:350-355, 614-638` |
| `crates/transforms/tests/common/mod.rs` beyond the Delta inventory | present; `crates/core/tests/common/mod.rs` pre-dates the run |

Three further details, recorded in the proposal's pass-2 note and not new
decisions: `not-keyboard` scans sources before targets (`apply.rs:490,
798-811`); the two swap rows' guard legs sit after their kind-specific checks,
and the mouse leg asks the button before the docked action on it
(`apply.rs:569-593, 666-681`); `MAX_GRADE_BOUND` is
`REACH_CLASSES_V2_TAXONOMY.len() - 1`, not the literal 4 (`document.rs:180`).

## Inventory, counted from the tree

- `schemas/policy/v1/`: `spec.md` plus 6 valid and 30 invalid fixtures — the
  exact names `spec.md:250-261` indexes, and the set
  `crates/core/tests/policy.rs:998`
  (`the_policy_corpus_ships_every_fixture_the_contract_names`) pins.
- Test tree, counted by `#[test]` attribute: `crates/core/tests/policy.rs` 26,
  `crates/transforms/tests/apply.rs` 29,
  `crates/transforms/tests/zz_api_probe.rs` 2. c1's 317 total predates the two
  tests the close-out added
  (`a_base_binding_one_action_twice_refuses_naming_the_duplicated_action`,
  `relocate_cluster_occupancy_asks_every_action_holding_the_target_key`), so the
  suite behind the ratified stamp `e90525d` is 319, with no other target moved.
- Core surface: `pub use` of `Policy`, `Transform`, `TagPattern`,
  `MAX_POLICY_TRANSFORMS`, `MAX_RELOCATE_TARGETS`
  (`crates/core/src/lib.rs:102-114`); `crates/transforms/src/lib.rs:44`
  re-exports `apply`, `Applied`, `TransformReport`, `Outcome`, `NotApplicable`,
  `NotApplicableReason`.
- Root `Cargo.toml` untouched: the `crates/*` member glob and the
  `bindforge-core = { path = "crates/core" }` workspace dependency both pre-date
  the run, so only `Cargo.lock` gained a package, as designed (A16).
- `schemas/README.md`: `policy` in the format list (line 5), the `policy/v1`
  scaffold sentence (27-31), the ADR-015 budget bullet (116-125), and "all
  eleven constants" (127) — the nine-to-eleven move Test strategy #11 asks for.

## Still open, unchanged by this pass

The three test gaps testing-r1c0 accepted stay unpinned: no invalid fixture for
the row grammar's target "no repeats" rule (implemented at
`document.rs:1430-1433`), no relocate row naming a non-`keyboard:` **target**
(every `apply.rs` fixture puts the non-keyboard id on the source side), and no
`swap-polarity` `reserved-input` case. And four determinism details stay
unstated in `spec.md`: which duplicated action the entry refusal names, which
foreign holder `occupied-target` names, and the two scan orders above.
