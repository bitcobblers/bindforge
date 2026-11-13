# complexity review r1c4

Ratification pass c4. Scope read: the diff `5bbcc09..f752153` of `02-proposal.md`,
the full current document, `docs/adrs/009-canonical-profile-and-manifest-model.md`,
the Revision log. Inputs used as named by the caller; no review report or
transcript opened.

verdict: approve

Checklist walk (complexity):
1. New package/module/interface: r3 adds none. `index.rs`'s new format-per-tree rule
   is a match arm in the existing walk; the `apply_policy` change is a signature
   change, not a new type. No reuse left on the table (`tools/` unaffected).
2. Moving parts trace to Goals: the per-tree format check and the widened
   `UnresolvedAncestor` both trace to "Deterministic chain resolution" and the
   Decision-5 drift gate; the `Result<PolicyOutcome, CoreError>` channel traces to
   the pinned-signature Goal. Nothing added is YAGNI.
3. Smaller design: r3 is net smaller than what it replaced — two claims deleted
   ("one invalid fixture per rejected condition", Test #10's fixture parenthetical),
   two shipped variants reused (`InvalidValue`, `UnresolvedAncestor`) in place of new
   ones. No sketch of a 30–50 % smaller shape is warranted by the changed text.
4. New dependencies: none. `bindforge-core` stays depends-on-nothing.
5. Deferrable half: the ADR-012 deferral of `fork`/`merge` (and `MergeConflict`'s
   producer) is retained; the first slice stays thin.
6. Knobs with no consumer / abstraction with one implementation: one consumer gap
   introduced by this round's replacement text — finding 1 below.
7. Value: proportionate. The complexity is all in format/grammar rules that the ADR's
   five contracts require, and the guards are reused shipped budget, not new budget.

## Findings

- [minor][new] Chosen design → Reach classes and placements → Authoring bar (r3-revised) —
  "grades are non-decreasing in grid distance from that hand's home-graded keys,
  **every rejected condition pinned by Test strategy #3's model tests**" —
  the replacement for the deleted "one invalid fixture per rejected condition" names
  Test strategy #3 as the pin for every rejected condition, but Test #3's stated
  enumeration (distinctness, `<class> <row> <col>` parse, cross-hand coordinate
  agreement, mirrored-key asymmetry, graded-set coverage, `off-reach` default,
  admitted-list `InvalidValue`) has no monotonicity clause. "non-decreasing in grid
  distance" appears exactly once in the document — in this sentence — so after the
  fixture obligation was removed this round, that CI-enforced grading condition has
  no named consumer: it is asserted by no fixture (r3's own reason for the deletion
  rules that out, it is not parse-visible) and by no test. The sentence also covers
  the `placement/v1/spec.md` half of the checklist, whose conditions are Test #4's,
  not #3's. This is a defect of the closure, not of the earlier theme: the theme
  (pinning claims that name no check) is closed everywhere else. Fix: extend Test
  #3's existing enumeration with the monotonicity assertion (grades non-decreasing
  in grid distance from that hand's home-graded keys) and scope the clause to
  "every reach-table condition pinned by Test strategy #3's model tests, every
  placement condition by #4" — same two tests, no new suite.

- [minor][re-raised] Chosen design → 2. Input vocabulary + Reach classes loader rules +
  Test strategy #3 — "`schemas/README.md` carries the admitted keyboard-name list:
  the admitted `keyboard:` vocabulary the reach tables draw from, **enforced by the
  reach loader** (a key outside it is `InvalidValue`) … No second key-name table is
  invented here." — unchanged by this round and not rebutted in the Revision log
  (line 306 records only the list's grading obligation, r2). The loader rule is a
  machine check, and the list's only stated home is prose in `schemas/README.md`;
  grep over `crates/` and `schemas/` finds no `keyboard:` key name anywhere, so
  `reach.rs` must carry its own US-ANSI name set to enforce it — a second table kept
  in step with the README by eye, which is what "No second key-name table is invented
  here" denies. Test #3 pins only the behavior of one out-of-list key, never the
  list's contents, so the pair can drift in either direction. Fix: one clause naming
  the machine-readable home (a public constant in the crate that owns input ids, e.g.
  `keyboard_names()`, which `reach.rs` and every input-id reader consults) with the
  README pointing at it, and extend Test #3's existing admitted-list clause to assert
  the loader's set equals the README enumeration.

- [nit][re-raised] Rollout 2 vs Chosen design → Adversarial posture — "the entry sets and
  the `CoreError` variants named in Adversarial posture" vs "`MergeConflict` lands
  with `fork`/`merge` in the ADR-012 slice, which is where its producer is" — Rollout 2's
  blanket phrase names all thirteen variants as landing in the core step, while the
  (rewritten) Adversarial-posture sentence defers one of them. Fix: two words in
  Rollout 2 — "the `CoreError` variants named in Adversarial posture, except the ones
  it defers to ADR-012".

Closed by this round, confirmed: `apply_policy`'s pinned signature is now
`Result<PolicyOutcome, CoreError>` with the whole-`stack` table check running before
the first invocation, `skipped` narrowed to the inapplicable-transform report, and
Adversarial posture + Test #10 restated as the `Err` channel — my r1c3 finding 1 is
fully closed, with no residue in §4's bullet list (`Skipped` stays the inapplicable
report there) or in Test #11. The Adversarial-posture rewrite accounts for all
thirteen variants (fixture: refused-TOML/nesting/size/unknown entry/`BadPin`; #1, #3,
#4, #6, #8, #9; `MergeConflict` deferred), and each named test does contain the clause
for the variant assigned to it.

## FYI (no severity)

- "Each tree accepts only the formats it holds" (§3, r3) reads repo-wide in isolation;
  it is scoped to the two walked trees, since `data/policies/` and `data/placements/`
  are fixed-path lookups (data-homes rule) and carry no index-time format check. A
  two-word scoping ("Each walked tree") would remove the ambiguity — editorial, not
  a design question.
- The widened `UnresolvedAncestor` rule and the `Resolved.manifest` `Some`-whenever-`for`
  claim are now stated in §3's API bullet, §3's Id→file paragraph, Test #9, and the
  Revision log. Consistent at every stop; the repetition is the canonical-statement
  style this document uses, so I am not calling it.
