# correctness review r1 (dissent pass c2)

verdict: request-changes

Scope: `git diff 7b2ddc6d10df95a81e738c734fe5c9673d530181 5bbcc09d65524a3e2c522e58501451480f23d8ee`
(02-proposal.md), then the full current document, the ADR, the Revision log, and
my own r1 findings. Inputs used: 02-proposal.md,
`docs/adrs/009-canonical-profile-and-manifest-model.md`, the Revision log, the
shipped code the design cites (`crates/core/src/{reach.rs,error.rs}`,
`crates/core/tests/{conformance.rs,common/mod.rs}`), and the register's research
URLs (spot-checked: toml.io/en/v1.1.0 → A12, rfc-editor.org/rfc/rfc6902 → A14,
w3.org/TR/gamepad → A20; all live and supporting). No review report or
transcript was opened.

## Closure check (my r1 findings)

- r1 #1 (major, registry on the far side of the dependency edge) — **closed**,
  by a better route than my fix: the closed invocation table is
  `policy/v1/spec.md`'s contract, core's reader owns only `stack`'s shape,
  `apply_policy`'s registry carries exactly that table, Test #10 asserts
  registry == table, Rollout 3/1 match. No duplicate home for the fact. The
  closure introduced a new defect (finding 1 below).
- r1 #2 (minor, admitted keyboard-name list gates nothing) — **closed**: the
  reach loader now enforces the list (`InvalidValue`), the list is described as
  the vocabulary the tables draw from with no grading obligation, Test #3
  iterates the content-derived graded set, and the Files-created parenthetical
  matches. Rollout/Risk claims about full-keyboard grading did not creep back.
- r1 #3 (minor, "each pinned by a fixture" over-promises) — **closed for the
  three variants I named** (`MergeConflict` and the two index limits now
  carry explicit exceptions, and Test #1 says "a walk over a synthetic tree").
  A narrower residue of the same class remains (finding 3).
- r1 #4 (minor, manifest pin-key collision) — **closed**: `DuplicateId` covers
  two manifest documents declaring one `game` + `revision`, the reason is
  stated (walk-order-dependent drift verdict), Test #9 mirrors it, and the
  Revision log records that a manifest carries no `id`, so only the key can
  collide. No new variant.
- r1 #5 (minor, `seeded-from` check vacuous when unpinned) — **closed for the
  unpinned case**: `seeded-from` requires `for` at parse, Test #6 asserts the
  `InvalidValue`, and the drift gate's precondition is stated. The closure's
  claim "the check always has a manifest" is not true as written (finding 2).
- r1 #6 (minor, README guard section) — **closed**: the Files-created row and
  Rollout 1 add the two limits and their rejections, and the posture sentence
  keeps the README's "public names for these limits" claim true.
- r1 #7 (nit, stale pointers) — **closed** ("§1" for the emphasis row,
  "(Risks)" dropped; the deleted prefix-rule Risk row is no longer cited).
- r1 #8 (nit, extension enumeration) — **closed**; the `.toml` clause now
  covers the `data/` manifest, placement and policy documents, matching §2, §4,
  §5 and Rollout 4.

Also re-checked, no finding: the `generated-by` closed producer list
`policy|preset|generator` covers every producer the document names (`policy:`
example, `generator:` for the Deferred layout-generation row) and cannot
collide with the input namespaces; `docked`'s demotion is consistent across
§5, Test #8, the drift paragraph and the new Deferred entry; the Windows row's
Test strategy **#14** renumber is correct (the list has 14 items, Rollout 7
cites #14); the r2 Revision-log entries match the body text they describe.

## Findings

- [major][new] Chosen design → Transforms (§4) — "`apply_policy(base: &Resolved,
  policy: &PolicyDoc, ctx: &Ctx) -> PolicyOutcome` with `PolicyOutcome { base:
  Resolved, skipped: Vec<Skipped> }`" + "`apply_policy` validates every
  invocation against the table … and a mismatch is the shipped `InvalidValue {
  key, message }` naming the invocation, raised before any transform runs" (and
  Test strategy #10: "rejected by `apply_policy` (`InvalidValue`), before any
  transform runs") — moving the invocation check out of core's parse put the
  rejection inside `apply_policy`, but the declared signature is unchanged and
  returns `PolicyOutcome`, which has **no error channel**: its only field is
  `skipped: Vec<Skipped>`, whose stated semantics are the report of an
  *inapplicable* transform that no-ops ("An inapplicable transform reports and
  no-ops"), not a typed refusal. As written, the design's own Goal ("Concrete
  Rust: … function signatures") names a function that cannot express the
  rejection three sections now require, and the caller (Rollout 4/5's seed
  pipeline, the future CLI) has no way to tell "stack rejected" from "stack
  applied with skips". Fix: make the return `Result<PolicyOutcome, CoreError>`
  (`bindforge-transforms` already depends on `bindforge-core`), keep `skipped`
  for the inapplicable-transform no-op, and state that the whole `stack` is
  validated against the table before the first invocation runs.

- [major][new] Chosen design → Profiles are chains (`resolve`) — "`for` is
  required at parse, so the check always has a manifest" — `for` being required
  at parse guarantees a *pin*, not a *manifest*: the pin's halves are only
  grammar-checked (`BadPin`), and nothing in §3, §5, Test #9 or Test #12 says
  what happens when a grammatical `game` + `revision` names no document in the
  manifest index. Under the design's own shapes that case is silently green:
  `Resolved { …, manifest: Option<ManifestPin> }` is documented as `None` for
  "a profile not bound to a game", so an unresolvable pin answers `None` too —
  one shape carrying two facts — and `resolve`'s `seeded-from` check and the
  drift gate ("resolves every profile … against the manifest revision it
  pins") both have nothing to check against, which is exactly the dangling
  `seeded-from` the r2 closure says it closed. Fix: one clause, no new
  mechanism — "a `for` whose `game` + `revision` names no manifest document in
  the index is `UnresolvedAncestor`, naming the pin", plus "`Resolved.manifest`
  is `Some` whenever the profile declares `for`", and mirror it in Test #9.

- [minor][re-raised] Chosen design → Adversarial posture + Authoring bar
  (§2) — "New `CoreError` variants, each pinned by a fixture in this slice
  except `MergeConflict` … and the two index limits" and "one invalid fixture
  per rejected condition" — the exception list carves out only the
  walk-limit pair, but the harness pins an invalid fixture by refusing its
  text through `bindforge_core::parse` (plus `resolve_chain` for a profile) —
  `crates/core/tests/conformance.rs:137-154`,
  `crates/core/tests/common/mod.rs:237-243` — one file at a time. The
  remaining listed conditions are not visible to `parse` at all:
  `GeometryConflict` (needs both hand tables), the new admitted-list
  `InvalidValue` and the class-vocabulary rule (reach loader, not parse),
  `DuplicateId` (two documents), `UnresolvedAncestor` (a document plus the
  index), `AxisKindConflict`/`UnknownAction`/`UnknownInput` (manifest against
  the taxonomy), `PlacementNotIsometric`/`PlacementDegraded` (placement against
  the reach tables), `DuplicateDelta` (patch application). So "each pinned by a
  fixture" is satisfiable only in the loose sense of "fixture data fed to a
  model suite", and the authoring bar's "one invalid fixture per rejected
  condition" is actively dangerous for the reach-classes conditions r2 added:
  an author who puts a `keyboard:banana` reach table in
  `schemas/reach-classes/v1/fixtures/invalid/` gets a parse-clean document, and
  the shipped harness panics "{label} is an invalid fixture and must be
  rejected". Fix: qualify the posture sentence once — "pinned by a fixture
  document where `parse` can see the condition; `GeometryConflict`,
  `DuplicateId`, `UnresolvedAncestor`, `AxisKindConflict`,
  `PlacementNotIsometric`, `PlacementDegraded`, `UnknownAction`,
  `UnknownInput` and `DuplicateDelta` are pinned over fixture data by the model
  suites that call the loader, index and validators (#3, #4, #6, #8, #9, #10)"
  — and in the authoring bar say the reach-classes rejected conditions are
  pinned by Test #3's model tests, `fixtures/invalid/` carrying only
  parse-refusable cases (Rollout 1's "refused-TOML-construct cases").

## FYI (no severity)

- Test strategy #10's "the registry's names, arity, and argument grammars equal
  the `policy/v1/spec.md` invocation table" is machine-checkable only against a
  hardcoded transcription of that prose table, so it guards code drift, not
  prose drift — worth one clause in `policy/v1/spec.md` saying the table is the
  transcription's source (complexity/consistency seat).
- The admitted keyboard-name list now has a loader consumer, so the
  enumeration exists twice (README prose + the const the reach loader checks).
  That mirrors the shipped grammar-in-README/enforced-in-code pattern, so it is
  not a finding, but the README row may want to say the list is
  loader-enumerated (docs seat).
- Test strategy #1's synthetic walk is a `crates/core/tests` case (index is in
  core); Rollout 5 names only `model_walk.rs`, `manifest_drift.rs`,
  `policy_semantics.rs`, so the file that carries #1/#9's index cases is still
  unnamed (rollout-completeness seat).
