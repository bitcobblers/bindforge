# complexity review r1 (dissent pass c2)

verdict: approve

Scope: the blob diff `7b2ddc6 → 5bbcc09` of `02-proposal.md`, then the full current
document, `docs/adrs/009-canonical-profile-and-manifest-model.md`, the Revision log,
and my r0/r1 findings. Checklist applied: `references/complexity.md` (items 1–7).

## Closure check (my r1 findings, in order)

1. **[major, r1] invocation validation split across `bindforge-core` and `bindforge-transforms` — CLOSED.**
   The fix I proposed is the one taken, in full: §4 now says "This spec carries the **closed
   invocation table** … Core's reader owns only the `stack` shape, and
   `schemas/policy/v1/fixtures/invalid/` covers exactly what it can see: the refused-TOML
   cases and a bad `stack` shape. `apply_policy` validates every invocation against the
   table"; Test #10 moves the per-shape cases to `crates/transforms/tests/policy_semantics.rs`
   and adds "the registry's names, arity, and argument grammars equal the `policy/v1/spec.md`
   invocation table"; Adversarial posture reads "`stack`'s shape at parse, the invocation
   table's arity and grammars in `apply_policy` (§4) — so no transform runs with a bad
   argument"; Rollout 3 says the registry's entries are "exactly its `policy/v1/spec.md`
   row". Core no longer needs the transform vocabulary and no registry-driven validation
   hook is implied — the one-way dependency edge is respected. Residue below is finding 1.
2. **[minor, r1] one test named two enumerations — CLOSED.** `schemas/README.md`'s list is now
   "the admitted `keyboard:` vocabulary … carrying no grading obligation of its own — the
   graded set below is content-derived, and that is the enumeration Test strategy #3
   iterates", Test #3 spells the graded set inline ("the keys the loaded `data/placements/`
   and the seed manifest's `default` rows reference"), and the Files created row repeats the
   split. No full-keyboard grading obligation returns; the `off-reach` default keeps its
   point. Residue (the list's new enforcing consumer) is finding 2.
3. **[minor, r1] `MergeConflict` listed as fixture-pinned with no producer — CLOSED.** The
   sentence now reads "each pinned by a fixture in this slice except `MergeConflict` — it
   lands with `fork`/`merge` in the ADR-012 slice, which is where its producer is — and the
   two index limits, pinned by Test strategy #1's synthetic walk, not by a fixture document".
   Test #1's synthetic-walk clause supplies the pin for `IndexTooLarge`/`IndexTooDeep`, which
   the r1 "each pinned by a fixture" claim also could not satisfy. Residue is finding 3 (nit).
4. **[minor, r1] `docked` as an unenforced documented guarantee — CLOSED** by the second option
   I offered: §5's description is demoted to "`docked` (actions the game hard-docks)", the
   "cannot escape" claim is gone, `validate`'s text names its own consumer ("That membership
   check is `docked`'s only v1 consumer — honoring a docked action is apply-time work, with
   ADR-012 (Deferred)"), and Deferred now lists "honoring a manifest's `docked` actions when a
   binding is applied" under ADR-012. Keeping the entry is ADR-required (Decision 5: "hard-docked
   UI constraints"), so this is not a re-opened deferral.

## Checklist

1. **Reuse.** The changed text reuses rather than adds: the reach loader rule "the file's name
   agrees with its in-file `hand`" restates the shipped `reach.rs:55` `hand_matches_file_name`
   rule; the policy path stays on the shipped `parse`/`parse_file`/`parse_reader` guard; the
   invocation table is prose in an existing `spec.md`, not a new mechanism; `DuplicateId` is
   reused for the manifest key collision instead of minting a variant. Nothing in `tools/`
   overlaps (no `tools/` index/loader for `data/`; the harness walk is test-only).
2. **Every moving part traces to a Goal.** Each changed addition has a named consumer:
   closed invocation table → `apply_policy` + Test #10 (ADR Decision 4); `seeded-from` requires
   `for` → makes `resolve`'s preset check non-vacuous (Decision 5, drift gate); manifest
   `game`+`revision` collision → drift-gate determinism (Test #9); README guard-section additions
   → keeps the shipped "public names for these limits" claim true.
3. **Smaller design?** The changed text is net-neutral-to-smaller: it removes a parse-time
   validation layer from core and removes an unenforced guarantee from `docked`. No 30–50 %
   reduction available in the touched sections — the three v1 transforms, the 3-row table, and the
   one content-derived graded-set rule are already the floor.
4. **New dependencies.** None added by this revision; core stays depends-on-nothing, transforms
   depends on the path crate only (Compatibility matrix, Rust toolchain row).
5. **Second-half deferral.** The revision defers more, not less: honoring `docked` and
   `fork`/`merge` (with `MergeConflict`) move to ADR-012, so the first merged slice is thinner.
6. **Knobs with no consumer / abstraction with one implementation.** The two r1 instances are
   closed. The new closed producer list `policy|preset|generator` is not a knob — it gates
   nothing at runtime, replaces free text, and its three values are the ADR's named provenance
   sources (policy, preset base, layout generation); closing it now avoids widening a closed
   set later. No new abstraction layer: one table, one registry, one loader.
7. **Value check.** Proportionate. Every clause added in this round buys a check that has a
   named consumer in the same slice, and the total machinery (six formats, five core modules,
   one leaf crate, three transforms) is what ADR-009's five contracts cost; nothing in the diff
   adds a layer for a hypothetical second implementation.

## Findings

- [minor][new] Chosen design → 4. Transforms produce bases — "`apply_policy(base: &Resolved, policy: &PolicyDoc, ctx: &Ctx) -> PolicyOutcome` with `PolicyOutcome { base: Resolved, skipped: Vec<Skipped> }`" + "`apply_policy` validates every invocation against the table … and a mismatch is the shipped `InvalidValue { key, message }` naming the invocation, raised before any transform runs" — the relocation of the invocation check out of core's reader (my r1 fix, accepted) left the pinned signature unable to carry the pinned rejection: `apply_policy` returns a bare `PolicyOutcome`, which has no error channel, so the `CoreError::InvalidValue` that §4, Adversarial posture ("the invocation table's arity and grammars in `apply_policy`") and Test #10 all require has nowhere to go. The Goals pin function signatures as the deliverable, so an implementer following the text hits a contradiction between two statements of the same section. This is the residue of the accepted closure, not a re-opened theme.
  Fix: make the return type `Result<PolicyOutcome, CoreError>` in that one signature (replacing `-> PolicyOutcome`), and state that the whole-`stack` table check runs first, so its `InvalidValue` arrives as `Err` while inapplicable transforms keep returning `Ok(PolicyOutcome)` with `skipped` populated. One signature; no new type.
- [minor][new] Chosen design → 2. Input vocabulary + Reach classes loader rules + Test strategy #3 — "`schemas/README.md` carries the admitted keyboard-name list: the admitted `keyboard:` vocabulary the reach tables draw from, enforced by the reach loader (a key outside it is `InvalidValue`)" / "every key is a `keyboard:` id in the admitted list, else `InvalidValue`" — the list gained its first consumer this round, which makes a duplication true that the same paragraph denies: the list is prose in `schemas/README.md` and there is no machine-readable admitted-name table anywhere in the repo (nothing in `crates/` or `schemas/` names a `keyboard:` key today), so `reach.rs` must carry a hard-coded US-ANSI name set of its own — a second key-name table, kept in step with the README by eye. Test #3 pins only the behavior for one out-of-list key, not the list's contents, so README and the code set can drift silently in either direction, and the sentence's "No second key-name table is invented here" claim is false as the text stands.
  Fix: one clause naming where the machine-readable admitted list lives (a public constant in the crate that owns input ids, e.g. `keyboard_names()`, which `reach.rs` and any input-id reader consults) — replacing the prose-list-only reading, and keep "no second table" true by pointing the README at it. Add to Test #3's existing clause that the loader's admitted set is the README list (compare the code set to the README enumeration), so the pair is pinned, not eyeballed.
- [nit][new] Rollout 2 vs Chosen design → Adversarial posture — "the entry sets and the `CoreError` variants named in Adversarial posture" — the Adversarial-posture sentence now excludes one variant from this slice ("each pinned by a fixture in this slice except `MergeConflict` — it lands with `fork`/`merge` in the ADR-012 slice"), and Rollout 2's blanket phrase names all thirteen variants as shipping in the core step, so the carve-out the r2 closure added reads as a contradiction at the rollout step.
  Fix: two words in Rollout 2 — "the `CoreError` variants named in Adversarial posture, except the ones it defers to ADR-012".

## FYI (no severity)

- Test #10's "the registry's names, arity, and argument grammars equal the `policy/v1/spec.md` invocation table" can only be executed as a hand-written expectation list, so the table exists in three places (spec prose, registry, test). That is this repo's fixture-pinning habit, and with three v1 transforms the cost is trivial — noting it for the test-strategy seat, which owns whether that assertion is expressible as written.
- `swap-mouse-buttons()` is the one shipped transform with no case in Test #10 (`swap-polarity` and `relocate-cluster` both have one) — a coverage question for the test-strategy seat, not a complexity one.
