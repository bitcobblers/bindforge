# Code review — correctness, round 1 dissent pass (c1)

Seat: correctness. Scope: `git diff 2682fb1..HEAD -- ':(exclude)docs/designs'`
(fix commit `eba75fc`, test-update commit `9c1f601`), read against the full
current tree and `02-proposal.md` / ADR-012. This pass first checks whether the
round-1 findings of this seat were actually closed, then reports only what the
changed code newly makes true.

## Round-1 findings — re-check

1. **major — crate-wide version gate fail-opens `profile`/`action-ids` declaring
   `v2`** — **closed.** `document::format_majors` + `check_format_major`
   (document.rs:118-147) run after the crate-wide union check (document.rs:232-241)
   and before any format arm, so a `profile`, `action-ids`, or `keyboard-geometry`
   document declaring `v2` is refused as `UnsupportedVersion` naming that format's
   majors — never read with the v1 grammar. The geometry arm's own `version != v1`
   guard is deleted; the per-format gate subsumes it (pinned by
   `rejects_a_document_declaring_a_major_that_only_another_format_ships`, which
   covers all three formats and survives in the green suite). `supported_majors()`
   stays the union, so the majors-equal-directories pin still passes.
   `reach_classes_entries` can now only ever see `v1`/`v2` for reach-classes, so
   the entry-set choice is total over what reaches it. Pinned in the corpus
   (`profile/v1/fixtures/invalid/major-not-shipped.toml`, refused by the
   conformance walk — the suite runs it) and stated in `schemas/README.md`,
   `profile/v1/spec.md`, and `reach-classes/v2/spec.md`.
   `format_majors`' key set matches `SUPPORTED_FORMATS` exactly, so no parsed
   format bypasses the gate. The `major_dir_segment` refactor in `action.rs`
   keeps the old predicate's behavior byte-for-byte (`v` + nonempty ASCII digits).
   My proposed fix prescribed the check "in each arm"; one call before the match
   is equivalent since every arm shares the same `format`/`version` pair — fine.

2. **minor — serializer could print `keys = [...]` under `version = "v1"`** —
   **closed.** The reach arm now branches on the version alone
   (document.rs:357-366): a v1 document never writes `keys`, a v2 document always
   does; serialize output always re-parses. The comment now states the rule the
   code enforces.

3. **minor — distinct-targets half of clause (a) unpinned** — **closed.**
   `tests/placement.rs::two_moves_onto_the_same_target_key_are_illegal` now runs
   the repeated-target set `[a→s, a→s]` (both deltas `(1,0)`), asserts
   `move_deltas == [(1,0),(1,0)]` and `delta == Some((1,0))` (common-delta half
   passes) before asserting `!shape_ok` and `!legal` — so the verdict can only
   come from `targets_are_distinct` (placement.rs:194, 326-335), and removing
   that check now reddens the suite. The differing-deltas set `a→s, d→s` stays as
   the pin for the other half with `delta == None`. The doc comment's claim
   matches the data this time (the only shape that shares a delta and repeats a
   target repeats the source).

4. **minor — 03-build-notes.md no longer described the tree** — **partially
   closed; superseded by the test-update stage.** The notes were rewritten at
   `eba75fc` and the deviation list (1-10) now genuinely matches the tree
   (deviation 1 names `zz_api_probe.rs` and `quoted_array`; deviations 4, 5, 9
   declare the per-format gate, the `action.rs`/`profile/v1` edits against the
   inventory's "Unchanged", and the two new invalid fixtures; the Gate posture
   claims check out: `cargo fmt --all -- --check` clean, `clippy -D warnings`
   clean, suite green when re-run on this tree). But `9c1f601` — inside this
   pass's scope — then landed the three test-side cases and did not re-touch the
   notes. See new finding 1.

5. **nit — `zz_api_probe.rs` and `quoted_array` undeclared** — **closed**
   (deviation 1).

## New findings (made true by the changed code)

1. **minor** — `docs/designs/012-input-vocabulary-and-reach-classes/03-build-notes.md:98-105, 116-140`
   The test-update stage in this diff (`9c1f601`) implemented exactly the three
   test-side cases the notes' "Round-1 review fixes" section records as "**not**
   done here" — the `version_gate.rs` profile-declares-`v2` case
   (`rejects_a_document_declaring_a_major_that_only_another_format_ships`), the
   `keyboard:é` case in `tests/input.rs`, and the shared-delta repeated-target
   move set in `tests/placement.rs` — and added two tests total, so the quoted
   Gate posture is stale: the tree runs **122** tests (re-run this pass, green),
   with `version_gate.rs` at **6** and `reach_v2.rs` at **18**, not the quoted
   119 / 4 / 17. The merge PR's evidence leg posts `test result:` lines that will
   not match the notes' quoted block. This is checklist item 5 (notes' claims
   true of the tree) again, the same shape as round-1 finding 4.
   **Fix:** re-run the three gate legs against the final tree and update the
   quoted `test result:` lines and the "not done here" paragraph (the three cases
   are now done in the test targets; only the ownership note remains accurate).

2. **nit** — `crates/core/tests/common/mod.rs:103-115`
   The test-update stage added a second outside-inventory helper to the file
   deviation 1 named only for `quoted_array`: `format_majors`, the on-disk
   per-format majors list now shared by `version_gate.rs` and `tests/action.rs`.
   Same bookkeeping gap round 1's nit closed for `quoted_array`, one helper late.
   **Fix:** widen deviation 1's sentence to name `format_majors` beside
   `quoted_array`.

## Checklist notes

- Components/interfaces from the proposal: all present and wired (input ids,
  v2 entry set, grades, geometry loader, predicate, serializer arms); names and
  signatures match or are declared (deviation 8).
- Assumption register: honored; the per-format gate implements A2's versioning
  rule more strictly than the proposal, declared as deviation 4 — an explicit,
  reviewed change, not a silent one.
- Undeclared deviations: none found — every gap between proposal and this diff
  (profile/v1 edits, new fixtures, gate redesign, serializer arm) sits in the
  deviation list.
- Error paths fail closed: profile/action-ids/geometry v2 refusals verified in
  the suite; unknown-format path unaffected (`format_majors` `None` falls to
  `UnknownFormat`).
- Suite re-run on this tree: `fmt` clean, `clippy -D warnings` clean, 122 tests
  green across the 14 targets.

## Verdict

Round-1 blocker/major leg: closed. Remaining: one minor (stale run-record
claims, no effect on shipped behavior), one nit.

**Verdict: approve**
