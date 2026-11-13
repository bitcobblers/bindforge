# Complexity review r1 (dissent pass c1)

verdict: request-changes

Scope: the r0→r1 diff of `02-proposal.md` (blob `cc78a21` → `7b2ddc6`), then the
full current document. Inputs: the proposal, ADR-009, the in-proposal Revision
log, and my r0 findings. Checklist: `references/complexity.md`.

## Closure check of my r0 findings

- **policy-level `hand` (r0 major) — CLOSED.** The policy entry set is now
  `format`, `version`, `id`, `stack`, the example drops both knobs, and §4 says
  outright "There is no policy-level `hand` (the invocation carries it, as
  ADR-009 spells it)". Matches ADR Decision 4's spelling.
- **Role-qualified prefix keys (r0 major) — CLOSED.** `action`/`default`/
  `exclusive` are the nested-array rows the Encoding rules already allow (A13);
  "The entry set stays the shipped closed allow-list — no prefix rule"; the
  prefix Risk row, the unknown-prefix fixture and Test #1's prefix clause are
  gone. My r0 sketch is what shipped, and `validate`'s checks are unchanged in
  substance.
- **Full-keyboard grading (r0 major) — CLOSED in substance**, with a loose end
  in the enumeration source (finding 2 below): the authoring bar is now the
  graded set, an ungraded key reads as that hand's `off-reach` (which also makes
  the emphasis reading total), and the coverage clause is out of Test #3.
- **`mouse-swap` / `MouseSwapNotOptedIn` (r0 minor) — CLOSED.** Entry, variant,
  fixture and Test clause all removed; "a policy enables a transform by listing
  it, which is the ADR's default-off".
- **`revision` on `profile`/`patch` (r0 minor) — CLOSED.** Kept on `manifest/v1`
  only, "the only pinnable revision is a manifest's", and the defaults line now
  reads "`for` absent = not bound to a game".
- **`fork`/`merge` ahead of need (r0 minor) — CLOSED in substance**, one residue
  (finding 3): the algebra and its suite moved to the ADR-012 slice, the rule
  stays as the `patch/v1/spec.md` contract, `diff` ships, and Goals/Rollout 5/
  Test #7/Deferred were updated consistently.
- **Two homes for reviewed data (r0 nit) — CLOSED.** The data-homes rule is one
  line in `schemas/README.md` (Files created), and Rollout 6 no longer adds a
  `repo-practices` line.

Closures did not add net volume: the two new walk constants are a bound on the
new index walk (a real consumer), the `serialize` arms are what A8's unmodified
harness demands of every format, and the `generated-by` grammar gate is one
shape rule on the one published field. Value check (checklist #7): the design
after r1 is proportionate to the five contracts ADR-009 fixes, and checklist #3
has no 30–50% smaller shape left in it that keeps the five contracts and the
seed. Checklist #4: no new dependency (core stays depends-on-nothing,
transforms depends on core only).

## Findings

- [major][new] Chosen design → Transforms produce bases + Test strategy #10 —
  "Each invocation must match its registry entry's declared arity and the
  grammars of its arguments … a mismatch is the shipped `InvalidValue { key,
  message }`, one `fixtures/invalid/` case per shape" / "an unknown transform
  name is rejected at parse (`InvalidValue`), before `apply_policy` runs".
  **What is wrong (checklist #6, duplicated source):** the r1 validation is
  placed in two crates at once. Policy documents are parsed by `bindforge-core`
  (the section says so, because the harness requires a reader per format
  directory), the invalid cases are `schemas/policy/v1/fixtures/invalid/` files
  rejected with a `CoreError` variant (Test #1), `InvalidValue` is a shipped
  core variant, and Adversarial posture says "no registry entry is reached with
  a bad argument" — so the arity/hand/placement-id/pattern/transform-name checks
  run in core, before the registry. But Rollout 3 puts "the registry — each
  entry carrying its declared arity and argument grammars" in
  `bindforge-transforms`. The text therefore implies one of two permanent costs:
  core hard-codes the v1 transform vocabulary (a closed list in the wrong crate,
  widened every time transforms grow, and it dissolves ADR-009's "`crates/transforms`
  owns the policy machinery" split), or core grows a registry-driven validation
  hook — an abstraction layer with one implementation today. **Fix:** state that
  core's policy reader validates only the shape it owns (`stack` = array of
  non-empty arrays of strings, first element the transform name) and that
  `apply_policy` validates each invocation against its registry entry's arity
  and argument grammars; reword §4's "a mismatch is the shipped `InvalidValue`"
  as an `apply_policy` outcome and Test #10's "rejected at parse … before
  `apply_policy` runs" as rejected by `apply_policy` before any transform runs.
  This replaces the parse-time clause, keeps the registry (and its per-shape
  cases) in one crate, and keeps the fixture-per-shape count: the shapes become
  `crates/transforms/tests/policy_semantics.rs` cases, the
  `schemas/policy/v1/fixtures/invalid/` pair keeps the refused-TOML and
  bad-shape cases core can actually see.

- [minor][regression] Chosen design → Input vocabulary + Files created —
  "`schemas/README.md` carries the admitted keyboard-name list — the single
  enumeration the graded set below and Test strategy #3 iterate" / "the admitted
  keyboard-name list (the enumeration Test strategy #3 iterates)".
  **What is wrong:** the r1 closure names two different enumerations for one
  test. §2/Authoring bar define the graded set as content-derived — "every key
  the shipped placements and the seed manifest's `default` rows reference,
  extended only by review" — and Test #3 reads "every key in the **graded set**
  is graded in both hands". The README list is the admitted US-ANSI name set,
  i.e. the whole keyboard. Read as written, Test #3 iterates the full admitted
  list, which is the coverage obligation r0 asked to drop and which the
  Revision log records as removed ("the full-keyboard coverage clause is gone
  from the authoring bar and Test #3"). Left ambiguous, the corpus size and CI
  surface grow back with the name list, and the `off-reach` default loses its
  point. **Fix:** one clause in both places — the README list is the admitted
  vocabulary (the grammar domain for `keyboard:<name>`), and Test #3's
  enumeration is the graded set, derived from the loaded `data/placements/`
  keys plus the seed manifest's `default` rows; state that the admitted list
  carries no grading obligation.

- [minor][new] Chosen design → Adversarial posture — "New `CoreError` variants,
  each pinned by a fixture: … `MergeConflict`" with the r1 deferral "`fork` …
  and `merge` … land with their suite in the ADR-012 slice".
  **What is wrong (checklist #6 residue of the deferral):** the variant list is
  the first slice's, and `MergeConflict` is the one entry in it with no producer
  in the first slice. Merge conflicts come from `merge()`, not from a document
  reader, so no `fixtures/invalid/` case can pin it — the "each pinned by a
  fixture" claim and Test #1's per-fixture rejection clause cannot be satisfied
  for it, and the shipped variant is constructed by nothing until ADR-012.
  **Fix:** split the sentence — the shipped variants (the list minus
  `MergeConflict`) ship with fixtures in this slice; `MergeConflict` lands with
  `fork`/`merge` in the ADR-012 slice, alongside the `patch/v1/spec.md` rule
  that already names it.

- [minor][new] Chosen design → Manifests state game facts + Test strategy #8 —
  "`docked` (hard-docked actions a binding move cannot escape)" / "every
  `exclusive` and `docked` member appears in the `action` rows".
  **What is wrong (checklist #6):** r1 widened `validate` to `docked`, so the
  entry's only v1 consumer is a membership check; the stated rule — a binding
  move cannot escape it — is read by nothing. `relocate-cluster` matches by
  action pattern and an isometry precondition, `swap-polarity` is taxonomy-gated,
  and neither consults `docked`, so a policy can relocate a hard-docked action
  the manifest says is unmovable. The entry is ADR-mandated data, so the entry
  stays; the unenforced claim is the cost. **Fix:** one clause stating who
  honors it — either `relocate-cluster`'s precondition excludes `docked`
  matches (`Skipped`, naming the action), or state that `docked` is read at
  apply time and gate the enforcement with ADR-012, and drop the
  "cannot escape" claim from §5 to a reference to that owner.

## FYI (no severity)

- `manifest/v1` carries no `id` entry, so the manifest index keyed on declared
  `game` + `revision` has no named collision variant (`DuplicateId` is scoped to
  "two documents declaring one id"); the completeness seat may want the collision
  named.
- The shipped reach table's in-file `hand = "left"` entry and the
  `{left,right}.toml` file name are two sources of the hand fact; the shape is
  shipped (A7) so re-deciding it here is not this design's business, but a
  loader-rule line saying the file name must agree with `hand` would close it.
- "`resolve` also rejects a `seeded-from` the pinned manifest does not declare,
  naming the profile id — a profile fact read against a manifest, so not
  `validate`'s (contract 5)" ends on the manifest contract while the check is
  placed at contract 3; the citation reads backwards.
