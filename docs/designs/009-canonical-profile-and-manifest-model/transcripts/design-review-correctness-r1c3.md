# Transcript: design-review:correctness r1c3

## Provenance
- stage: design-review:correctness (round r1c3)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:08:06.078Z
- ended: 2026-11-11T23:11:13.447Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 5bbcc09d65524a3e2c522e58501451480f23d8ee f7521536a6bca93f4f9b4d4a4cfce3446567cdc7` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/009-canonical-profile-and-manifest-model.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Transforms (§4) — \"`apply_policy(base: &Resolved, policy: &PolicyDoc, ctx: &Ctx) -> PolicyOutcome` with `PolicyOutcome { base: Resolved, skipped: Vec<Skipped> }`\" + \"`apply_policy` validates every invocation against the table — its registry carries exactly that table — and a mismatch is the shipped `InvalidValue { key, message }` naming the invocation, raised before any transform runs\" (Test strategy #10: \"rejected by `apply_policy` (`InvalidValue`)\")",
    "problem": "new — the r2 closure moved the invocation-table check out of core's parse and into `apply_policy`, but left `apply_policy`'s declared signature unchanged, returning `PolicyOutcome`, whose only field is `skipped: Vec<Skipped>`. There is no error channel: the `InvalidValue` the design now requires `apply_policy` to raise (Adversarial posture, §4, Test #10) has nowhere to go, and `skipped`'s stated semantics are the report of an *inapplicable* transform that no-ops, not a typed refusal. As written, a signature the design's own Goals promise as concrete Rust cannot express a rejection three sections pin, and a caller cannot distinguish \"stack rejected\" from \"stack applied with skips\". This is the defect the closure of my r1 #1 introduced: the parse-time check needed no error channel, the apply-time one does.",
    "fix": "Make the return `Result<PolicyOutcome, CoreError>` (`bindforge-transforms` already depends on `bindforge-core`, so `CoreError` is in scope), keep `skipped` for the inapplicable-transform no-op, and state that the whole `stack` is validated against the invocation table before the first invocation runs. Test #10's wording then holds unchanged."
  },
  {
    "severity": "major",
    "location": "Chosen design → Profiles are chains (`resolve`) — \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare, naming the profile id — a profile fact read against a manifest, so not `validate`'s; `for` is required at parse, so the check always has a manifest.\"",
    "problem": "new — `for` being required at parse guarantees a pin, not a manifest: the pin's halves are grammar-checked only (`BadPin`), and §3, §5, Test #9 and Test #12 state nothing about a grammatical `game` + `revision` that names no document in the manifest index. Under the design's own shapes that case is silently green: `Resolved { …, manifest: Option<ManifestPin> }` is documented as `None` for \"a profile not bound to a game\", so an unresolvable pin answers `None` as well — one shape carrying two facts — and both `resolve`'s `seeded-from` check and the drift gate (\"resolves every profile … against the manifest revision it pins — a read of the manifest index\") have no manifest to check against, so a dangling `seeded-from` on a profile pinning a manifest the seed does not ship passes CI, which is the residual hole the r2 closure states it closed. The claim as written is simply false: a pin is not a manifest."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Adversarial posture + Authoring bar (§2) — \"New `CoreError` variants, each pinned by a fixture in this slice except `MergeConflict` … and the two index limits\" and \"one invalid fixture per rejected condition\"",
    "problem": "re-raised (narrowed from my r1 #3) — the exception list carves out only `MergeConflict` and the walk limits, but the shipped harness pins an invalid fixture by refusing its text through `bindforge_core::parse` (plus `resolve_chain` for a profile), one file at a time (`crates/core/tests/conformance.rs:137-154`, `crates/core/tests/common/mod.rs:237-243`). The remaining listed conditions are invisible to `parse`: `GeometryConflict` (needs both hand tables), the r2-added admitted-list `InvalidValue` and the class-vocabulary rule (reach loader, not parse), `DuplicateId` (two documents), `UnresolvedAncestor` (document + index), `AxisKindConflict`/`UnknownAction`/`UnknownInput` (manifest vs taxonomy), `PlacementNotIsometric`/`PlacementDegraded` (placement vs reach tables), `DuplicateDelta` (patch application). Worse, the authoring bar's \"one invalid fixture per rejected condition\" now applies to reach-classes loader conditions r2 added: an author who puts a `keyboard:banana` reach table in `s… [elided by the checkpoint cap]",
    "fix": "Qualify the posture sentence once: \"pinned by a fixture document where `parse` can see the condition; `GeometryConflict`, `DuplicateId`, `UnresolvedAncestor`, `AxisKindConflict`, `PlacementNotIsometric`, `PlacementDegraded`, `UnknownAction`, `UnknownInput`, `DuplicateDelta` are pinned over fixture data by the model suites that call the loader, index and validators (#3, #4, #6, #8, #9, #10)\" — and in the authoring bar state that reach-classes' rejected conditions are pinned by Test #3's model tests, `fixtures/invalid/` carrying only parse-refusable cases (Rollout 1's refused-TOML-construct cases). No new mechanism, no harness change."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/009-canonical-profile-and-manifest-model.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/correctness-r1c3.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Reach classes and placements → Authoring bar (§2) — \"CI enforces what it can: the graded set … is graded in both hands, coordinates agree across hands, grades are non-decreasing in grid distance from that hand's home-graded keys, every rejected condition pinned by Test strategy #3's model tests\"","problem":"new — the r3 closure makes Test #3 the sole pin for \"every rejected condition\", and two items in that same CI-enforced list break it. (a) Grid-distance monotonicity appears in no test anywhere: Test #3's clauses are per-hand distinctness, row grammar + class vocabulary, cross-hand coordinates, mirrored asymmetry, graded-set coverage, off-reach default, admitted-list InvalidValue — monotonicity is not among them, and no other Test strategy item names it, so the condition the bar says CI enforces is exactly the un-pinned one r1 #3 was about. (b) The bar governs both placement/v1/spec.md and reach-classes/v1/spec.md, and Adversarial posture maps PlacementNotIsometric/PlacementDegraded to #4, so naming only #3 contradicts the posture's own mapping. Fix: write \"every rejected condition pinned by Test strategy #3 and #4's model tests\", and add the monotonicity clause to Test #3 — or drop it from the CI-enforced list to review-only, which \"extended only by review\" already implies for the graded set.","fix":"Add \"and #4\" to the authoring bar's pinning clause, and add \"grades are non-decreasing in grid distance from that hand's home-graded keys\" as a Test #3 clause (or demote the condition to review-only)."},{"severity":"minor","location":"Test strategy #4 and #8 — \"a non-isometric fixture is rejected `PlacementNotIsometric`; a fixture exceeding `max-degrade` is rejected `PlacementDegraded`\" / \"… is rejected, each with its own fixture\"","problem":"new — the closure fixed this wording for reach-classes only (Test #3's new \"in the loader, not in a fixture document\"); #4's and #8's conditions are produced by `validate`, which `parse` cannot see, so the bare word \"fixture\" points an implementer at `schemas/{placement,manifest}/v1/fixtures/invalid/`. A document placed there is asserted by the shipped fixture-walking harness (Test #1: \"each `fixtures/invalid/` case is rejected with the pinned `CoreError` variant\"; Test #13: \"requires no change to `crates/core/tests/conformance.rs`\") — it parses clean, so the harness assertion fails and the no-harness-change claim breaks. Adversarial posture's general rule (\"only where `parse` can see the condition … otherwise, over fixture data, by the model suite\") already forbids it; only these two test-strategy lines are loose.","fix":"Carry #3's clause across once per site: in #4, \"a non-isometric placement pair is rejected PlacementNotIsometric by `validate`, not in a fixture document\" (same for the degraded and ungraded-key cases); in #8, \"each over its own manifest data, not in `fixtures/invalid/`\"."},{"severity":"minor","location":"Chosen design → Profiles are chains (`resolve`) — \"`Resolved { …, manifest: Option<ManifestPin> }`, whose `manifest` is `Some` whenever the profile declares `for`\" + \"`for` is required at parse and a `for` naming no manifest document in the index is `UnresolvedAncestor` …, so the check never runs against nothing\"","problem":"new — the `Some`-whenever claim is stated of one document, while a chain carries the entry on every rung: §3 gives `patch/v1` \"the same optional chain entries\", so a base can pin `remnant2@0.1` (with `seeded-from` beside it) under a leaf patch that pins nothing, or pins `0.2`. `Resolved.manifest` is one slot for N pins, and the guarantee \"never runs against nothing\" follows only if the `seeded-from` check is read per-document against that document's own `for`; read against the rooted document's pin — the natural reading of \"the profile\" in `resolve(index, root, …)`, and the value the drift gate hands to `validate` (\"resolves every profile … against the manifest revision it pins\") — a chain whose base carries `seeded-from` and whose leaf carries no `for` answers `manifest: None`, and the dangling-preset case r1 #2 raised is vacuously green again for chains.","fix":"One clause: `Resolved.manifest` is the rooted document's pin; the `seeded-from` check runs per-document against that document's own `for` (parse already pairs them); a chain document whose `for` is present and differs from the rooted document's is a rejection naming both document ids."}]}
```
