# Build notes — design 014 (profiles as chains) implementation

Round 1's code review (`reviews/code/correctness-r1c0.md`) is addressed here;
the original implementation record is kept below, with the two rows it logged as
deviations now withdrawn.

## Round 1 fixes

| Finding | What changed |
|---|---|
| 1 — blocker, `merge` could never emit a `swap` | `chain_ops::emit_composition`: a reciprocal pair's partner is now stepped past (`continue`), not faulted. The partner visit was the *other half of a pair already emitted*, so the old `!reciprocal \|\| exchanged.contains(other)` guard refused every composition that keeps an exchange, and `emitted.swap` could only ever end empty. `!reciprocal` still refuses a one-sided composition, and the re-check proof is unchanged. |
| 2 — major, a shipped refusal string was edited for a test | `error.rs`: `CoreError::Cycle`'s `Display` arm is back to design 008's bytes — `chain derives from itself`. `Cycle` carries no payload, so no wording can name `link-a`/`link-b`; the variant assertion is the contract (Test strategy #3). The module doc now states that a shipped arm's wording is not re-worded to satisfy a new assertion. |
| 3 — minor, a second spelling of the directory-major rule | `chain_ops::check_link_version` is gone; the chain calls `document::check_dir_version` like every other loader (`reach`, `placement`, `action`), so the rule has one definition and one variant — `InvalidValue { key: "version" }` naming the file, the declared major, and the directory — as the proposal's Options sentence and step 1 require. `schemas/profile/v2/spec.md` rule 2 now states that variant and points at the version gate's own refusal for a major the format does not ship. |
| 4 — minor, `merge` carried provenance labels | `merge` no longer sets `seeded_from`/`generated_by` on its output and `inherit_provenance` is deleted. `generated-by` names what produced the document it is written on; neither operand produced the merge. The caller writes its own. Documented in `merge`'s doc comment. |
| 5 — minor, a missing head reported as `ChainLinkMissing` | `load_chain` split into `open_head` (the caller's path → `parse_file`, whose `Io` carries the path for `source_path`) and `open_ancestor` (a link the walk constructed from a parse-validated slug → `ChainLinkMissing`). A raw file stem is never echoed as a profile id. `error.rs`'s variant doc and `spec.md` rule 3 say which is which. |
| 6 — nit, the symbolic `swap` assumption | Stated in `merge`'s and `apply_lists`' doc comments: the model assumes the base binds both sides of every exchange, and a merge that would need otherwise refuses — `merge` never reads the base, so any answer depending on its bindings would be a guess. The proposal's merge section still lacks that sentence; it is the editor's text, so it is left to a proposal amendment rather than rewritten here. |
| 7 — nit, conditional `kind`/`for` in the v2 writer | `serialize_profile_link` writes `kind` and `for` for every v2 document — the writer branches on `version` alone, as the proposal says. A hand-built v2 model with `kind: None` writes `kind = ""`, which the reader refuses naming the entry, never a document that drops a required entry; `serialize`'s and the function's doc comments state that precondition. |
| FYI — per-action scan in the action join | `check_joins` builds one `BTreeSet<&ActionId>` of the manifest rows per resolution and tests membership, matching the cost paragraph's "one per-resolution lookup of the `MAX_MANIFEST_ROWS`-capped action table". Row order is untouched. |
| FYI — the duplicated slug cap | Not touched. `manifest.rs` is on the proposal's Unchanged list, so sharing one helper between it and `document.rs` is a change that needs the proposal, not a fix round. |

`git status` after the round: `crates/core/src/chain_ops.rs`,
`crates/core/src/document.rs`, `crates/core/src/error.rs`,
`schemas/profile/v2/spec.md` — plus the run's own `.pipeline.json`. No test file,
no `Cargo.toml`, no `Cargo.lock`, no workflow.

## Test updates the test engineer owes (testsNeedUpdate: true)

*Completed by the test-update stage `aaf0796` — see "Test-update round" below;
kept as the record of what was owed.*

The implementation changed three behaviors the current assertions pin; no test
file was edited (stage rule), so these are the assertions to change:

1. `crates/core/tests/chain_ops.rs:1281` — drop `"Cycle"` from
   `assert_refusal_names(&err, &["Cycle", "link-a", "link-b"])`. The variant is
   already asserted two lines above (1277–1280), which is what Test strategy #3
   pins; `Cycle` has no payload, so no message can name `link-a`/`link-b`.
   Failure text: `the refusal must name the offender it refused, one of
   ["Cycle", "link-a", "link-b"], got 'chain derives from itself'`.
2. `crates/core/tests/chain_ops.rs:1422` — expect
   `CoreError::InvalidValue { key: "version", .. }`, not
   `UnsupportedVersion`, for a v2 head in a `v1/` directory; the test's own doc
   comment ("exactly as the shipped loaders already refuse it") is now true, and
   `reach_v2.rs`'s `InvalidValue` pin for the identical fault is what it matches.
   Failure text: `the directory/declared-major mismatch is the version gate's
   refusal, got InvalidValue { key: "version", message: "`retune.toml` declares
   `v2` in a `v1` directory" }`.
3. New cases the review asked for and the suite does not have:
   - a merge whose output keeps an exchange: `p1 = swap a b`, `p2 = set c k`
     (disjoint actions) must merge to one patch with `swap ["a b"]` plus the
     second patch's element, and resolve over the base to the same layout as the
     `base → p1 → p2` chain. Verified here by scratch harness, pinned by no test;
     `merge_composes_compatible_patches_into_one_patch` asserts `swaps_of` *empty*
     and the two swap-carrying merge cases both expect a refusal, so today's
     suite cannot see this finding either way.
   - `merge` output carries neither `seeded-from` nor `generated-by` even when
     both operands agree on them (finding 4).
   - a missing head is `CoreError::Io` with `source_path()` naming the path,
     while a missing constructed link stays `ChainLinkMissing` naming the id
     (finding 5).
   - optionally, `serialize` of a hand-built v2 model with `kind: None` refuses
     on re-parse instead of silently dropping a required entry (finding 7).

## Gate evidence (round 1 fix round, run in this stage's shell)

`cargo fmt --all -- --check` — clean (no output).

`cargo clippy --workspace --all-targets -- -D warnings` — clean:

```
Checking bindforge-core v0.1.0 (/home/mike/repos/bindforge/crates/core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.51s
```

`cargo test --workspace --locked --no-fail-fast` — 251 passed, 2 failed, the two
being exactly rows 1 and 2 of the list above:

```
Running unittests src/lib.rs       test result: ok. 0 passed;  0 failed
Running tests/action.rs            test result: ok. 16 passed; 0 failed
Running tests/action_ids.rs        test result: ok. 14 passed; 0 failed
Running tests/chain_guard.rs       test result: ok. 4 passed;  0 failed
Running tests/chain_ops.rs         test result: FAILED. 69 passed; 2 failed
Running tests/conformance.rs       test result: ok. 9 passed;  0 failed
Running tests/gate_contract.rs     test result: ok. 8 passed;  0 failed
Running tests/grammar_closure.rs   test result: ok. 13 passed; 0 failed
Running tests/input.rs             test result: ok. 6 passed;  0 failed
Running tests/manifest.rs          test result: ok. 49 passed; 0 failed
Running tests/parse_budget.rs      test result: ok. 11 passed; 0 failed
Running tests/placement.rs         test result: ok. 17 passed; 0 failed
Running tests/reach_tables.rs      test result: ok. 5 passed;  0 failed
Running tests/reach_v2.rs          test result: ok. 18 passed; 0 failed
Running tests/version_gate.rs      test result: ok. 9 passed;  0 failed
Running tests/zz_api_probe.rs      test result: ok. 3 passed;  0 failed
failures:
    refuses_a_head_whose_directory_major_differs_from_its_declared_version
    refuses_a_two_file_cycle_as_a_cycle
```

`chain_guard.rs` (4) and `reach_v2.rs` (18) passing unchanged is the evidence
that the reverted `Cycle` wording and the reused `check_dir_version` are the
shipped contract, and every pre-014 fixture and refusal is byte-unchanged.

Because the stage rule keeps test files out of reach, the blocker's fix was
verified with a scratch binary outside the repo
(`/tmp/opencode/merge-check`, a path dependency on `bindforge-core`, nothing
added to the workspace). Its 19 checks all pass: the merge that keeps an
exchange emits one `swap ["move.a move.b"]` plus the second patch's element;
`serialize`-ing that merged patch, writing it beside the base, and
`load_chain`ing it resolves to
`["move.a keyboard:e", "move.b keyboard:w", "aim.c keyboard:f", "ui.d keyboard:t"]`
— byte-identical to the `base → p-one → p-two` chain's layout; a `swap` under a
`clear` of one swapped action, an identity-cancelling pair, and a `set` under a
`swap` all still refuse; the directory-major fault is `InvalidValue` and never
`UnsupportedVersion`; an absent head is `Io` (path in `source_path()`, not in the
message) while an absent constructed link is `ChainLinkMissing`; `Cycle`'s
wording is design 008's; and a `kind: None` v2 model writes `kind = ""` and
refuses on re-parse.

## What landed

Exactly the proposal's delta inventory (Chosen design → Delta inventory), plus
the corpus:

- New: `crates/core/src/chain_ops.rs` (the `Chain` type only `load_chain`
  builds, `load_chain` with the one-directory rule and the resolved-chain
  guards, `resolve` with the six determinism rules and the three load-time
  joins, and `fork` / `diff` / `merge` as operations over documents), and
  `schemas/profile/v2/` — `spec.md` plus the corpora named in the proposal:
  **7 valid and 37 invalid fixtures, each one exactly the name the proposal's
  inventory lists**, each invalid fixture carrying the one fault it is named
  for.
- Edited: `crates/core/src/document.rs` (`PROFILE_ENTRIES` → `PROFILE_V1_ENTRIES`,
  `PROFILE_V2_ENTRIES`, the `profile_entries(version)` selector beside
  `reach_classes_entries`, the v2 branch of the profile arm — `ProfileKind`,
  the extended `Profile` (`kind`, `for_manifest`, `set`, `clear`, `swap`,
  `seeded_from`, `generated_by`) — the version-branched `serialize`, and
  `MAX_PATCH_DELTA_ELEMENTS` / `MAX_GENERATED_BY_OCTETS`; `format_majors`
  widened for `profile` to `[v1, v2]`, the only half that grows, A4),
  `crates/core/src/chain.rs` (doc-header pointer only — its code and its
  fixtures are untouched, A15), `crates/core/src/error.rs` (one new variant,
  `ChainLinkMissing`, and its `Display` arm), `crates/core/src/lib.rs` (the
  module, the re-exports, the public-surface doc paragraph),
  `schemas/README.md` (the series sentence records `profile/v2` landing; the
  budget paragraph names the two new constants and the composite statement),
  `data/profiles/README.md` (the loader exists; the untrusted-input posture
  paragraph untouched).

## Deviations from the proposal

The earlier list's rows 1 and 2 (the `Cycle` wording, and the chain's own
`UnsupportedVersion` for a directory-major mismatch) are **withdrawn**: the
review found them to be divergences from sentences the proposal states, and the
code now follows the proposal. What remains:

1. **Parse-level faults on a chain link are re-attributed to the link.**
   `name_the_link` wraps `parse_file`'s `InvalidValue` so the message names the
   link id the walk was opening. The proposal asks for refusals that name the
   offending link; a parse fault inside a link document otherwise names only the
   entry or element. The variant and the escape-and-truncate path are unchanged.
2. **`merge` composes symbolically and re-checks its own output.** The proposal
   says `merge` composes two patches over one base; as built it works out the
   composition over symbols — `Binding::Base` (the base's own binding, a symbol
   `merge` never reads), `Written(InputId)`, `Unbound` — then emits the result
   in the fixed `set` → `clear` → `swap` order and re-verifies the emitted
   patch against those symbols. When a composition has no one-patch form under
   that order, `merge` refuses with `InvalidValue` naming the colliding action
   id rather than emitting a patch that resolves differently; a composition that
   cancels to the identity refuses as the empty-patch case. The assumption that
   the base binds both sides of every exchange (review finding 6) is stated in
   the code docs; a sentence in the proposal's merge section is the editor's.
3. **`merge` and `diff` do not re-check `MAX_PATCH_DELTA_ELEMENTS` on their
   output.** The cap is a parse rule (it binds whoever *reads* a document), and
   these operations return a `Profile` value, not text; the emitted lists are
   bounded by the union of the operands' elements, so a merge of two
   near-cap patches can emit past the cap and the result is refused when it is
   next parsed.
4. **`chain_ops` uses `BTreeMap`/`BTreeSet`, not `HashMap`/`HashSet`.**
   `ActionId` and `InputId` derive `Ord` but not `Hash`, and the delta inventory
   declares `action.rs` and `input.rs` unchanged. The ordered maps also make the
   *choice* of which colliding action a refusal names deterministic.
5. **Three `document.rs` items widened to `pub(crate)`** so `chain_ops` reuses
   them instead of restating them — mechanical, inside a file the delta
   inventory already edits: `MAJOR_V2`, `check_profile_slug`, and
   `format_majors` (the last one only for the first round's local version check;
   after finding 3 `chain_ops` no longer calls it, and it stays crate-internal
   either way). `check_dir_version` (finding 3's reuse), `major_dir_segment`,
   `parse_file`, and `supported_majors` were already reachable.
6. **The 63-octet id cap is a private constant.** `MAX_SLUG_OCTETS` and the
   slug-rule phrasing live as private items in `document.rs`; the proposal
   states the bound in the entry table but names only `MAX_PATCH_DELTA_ELEMENTS`
   and `MAX_GENERATED_BY_OCTETS` as new constants, and `zz_api_probe.rs` pins
   exactly those two in the exported budget family.
7. **A `generated-by` refusal never echoes the label.** `one_line` escapes ASCII
   control characters but not the bidi control range, so the check quotes the
   *length* and the reason instead of the value.
   `generated-by-bidi-char.toml` pins the refusal.
8. **Two `schemas/README.md` sentences beyond the proposal's edit list**, both
   consequences of `profile/v2` landing rather than new contract: the versioning
   rule's "`reach-classes` is the only format with two majors today" sentence,
   and the budget paragraph's constant count.
9. **`schemas/profile/v2/spec.md` states the load-level rules in prose**, as the
   proposal requires, and adds the fixture-name discipline paragraph. Its
   error-variant names match the implementation, which is the point of writing
   it after the code — including this round's rule-2 rewrite.

Fixture notes, not deviations: `hex-integer.toml` carries `revision = 0xFF`, so
it is outside the closed v2 entry set as well as being an out-of-grammar integer
— exactly the shape `profile/v1`'s fixture of the same name uses;
`nesting-past-limit.toml` nests 17 levels in a `set` array, mirroring v1's
`derives` spelling; `dotted-key.toml` uses `for.slug` rather than a fresh key,
because a second invisible `for` is the smuggling the rule exists to stop.

No rule, entry, error variant, budget value, public name, or fixture name
deviates from the proposal. The proposal's Unchanged list (`toml.rs`,
`chain.rs`'s code, `action.rs`, `input.rs`, `reach.rs`, `placement.rs`'s
predicate, `manifest.rs`, every pre-existing fixture) appears nowhere in the
working tree's changes.

## Test-update round (stage `aaf0796`, after the fix stage `f299913`)

The test engineer took the full list above into `crates/core/tests/chain_ops.rs`
(the stage owned test files; src files untouched):

1. `refuses_a_two_file_cycle_as_a_cycle` (:1272) — the needle list is dropped;
   the case pins the `Cycle` *variant* plus the non-empty, one-line,
   control-free shape, which is what Test strategy #3 pins and what A15's
   frozen `Display` bytes allow.
2. `refuses_a_head_whose_directory_major_differs_from_its_declared_version`
   (:1467) — now expects `CoreError::InvalidValue { key: "version" }` naming
   the file, both majors, and the directory, and asserts the scratch path
   stays out of the message; the doc comment states the
   `InvalidValue`-vs-`UnsupportedVersion` distinction.
3. All three owed cases landed:
   `refuses_an_absent_head_as_the_read_refusal_carrying_its_path` (:1378 —
   `Io`, `source_path() == head`, path absent from the message);
   `merge_keeps_an_exchange_the_second_patch_does_not_touch` (:2060 — the
   non-empty `swap` leg is asserted, the merged patch is written beside the
   base, `load_chain`ed as a file-level move, and its layout asserted equal to
   the stacked `base → p1 → p2` chain — the pin the fix round could only verify
   by scratch harness); `merge_leaves_the_provenance_labels_absent_from_its_
   output` (:2135 — labels absent even when both operands agree). The optional
   fourth (a `kind: None` model refusing on re-parse) was not added as a test;
   the fail-closed behavior is documented on `serialize`/`serialize_profile_link`
   and was verified by the scratch harness.

## Ratification passes (r1c1 correctness dissent, r1c2 ratification)

- Correctness dissent pass (`reviews/code/correctness-r1c1.md`): approve,
  0 blockers / 0 majors. Findings 1–5 and 7 verified RESOLVED *in the code*,
  each with the pin the fix round named. One nit carried: the merge
  base-binding assumption (finding 6) lives in the rustdoc but not in the
  proposal's merge paragraph or `spec.md` — design-text territory outside a fix
  stage's remit; it is recorded in the proposal's As-built note (2026-11-13)
  instead.
- Ratification passes c2 (testing, compatibility, security, performance): all
  four approve; the post-approval revisions broke no approved pin and added no
  new finding. Carried, deliberately unfixed minors/nits (follow-up candidates,
  none design-changing):
  - testing #1: `check_positions`' step-4 branch is unreachable (the parse
    shape rules subsume it) and three tests' `proves:` comments still cite it;
    testing #2: `check_link`'s non-profile arm and `link_below`'s no-kind fault
    are unpinned hostile-chain shapes; testing #3: Test strategy #9 is cited by
    no test.
  - compatibility #1–3: `spec.md`'s case-unique id↔file-name claim is not
    enforced against case-insensitive filesystems (`try_exists` answers a
    case-only mismatch); `spec.md`'s `generated-by` row is narrower than the
    reader's label set; the slug grammar admits Windows reserved device
    basenames (`con`, `aux`, …).
  - security #1: `one_line` passes U+2028/U+2029 raw, and the `check_dir_version`
    reuse added one echo path through that gap; security #2: `merge` output
    stays unbounded against `MAX_PATCH_DELTA_ELEMENTS` — that is deviation 3
    above, accepted there.
  - performance nits: O(d²) dedupe in `touched_actions`/`merge_touch_order` and
    the linear find in `diff`'s layout compare — both bounded by the parse
    budgets, load-time only.

## Final state at merge (HEAD `5d9cbaf`)

- Code commits end at the test-update stage `aaf0796`; `3852b63` and `5d9cbaf`
  are docs-only. The full range's name-status is exactly the proposal's delta
  inventory — New: `crates/core/src/chain_ops.rs`, `crates/core/tests/chain_ops.rs`,
  `schemas/profile/v2/` (`spec.md`, 7 valid + 37 invalid fixtures, every name
  as listed); Edited: `document.rs`, `chain.rs` (doc-header pointer only),
  `error.rs`, `lib.rs`, `grammar_closure.rs`, `version_gate.rs`,
  `conformance.rs`, `zz_api_probe.rs`, `schemas/README.md`, `data/profiles/README.md`
  — plus the run's own artifacts (research, proposal, build notes, reviews,
  transcripts, `.pipeline.json`). Nothing on the Unchanged list appears.
- The deviation list (1–9) above is unchanged by the test-update and
  ratification rounds; all nine still describe HEAD, and the proposal's
  As-built note mirrors them.
- Gates re-run at HEAD: `cargo fmt --all -- --check` — clean;
  `cargo clippy --workspace --all-targets -- -D warnings` — clean;
  `cargo test --workspace --locked` — all 16 targets green, 256 passed /
  0 failed: `chain_ops.rs` 74, `conformance.rs` 9, `grammar_closure.rs` 13,
  `version_gate.rs` 9, `zz_api_probe.rs` 3, `chain_guard.rs` 4 and
  `reach_v2.rs` 18 unchanged from the pre-fix run. `git status`: only the run's
  `.pipeline.json` modified.
- Test strategy #9: the Linux CI leg is the green gate run above; the Windows
  contributor leg stays contributor-side (Rollout 7).
