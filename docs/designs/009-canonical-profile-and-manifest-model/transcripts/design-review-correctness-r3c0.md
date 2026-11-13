# Transcript: design-review:correctness r3c0

## Provenance
- stage: design-review:correctness (round r3c0)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:26:48.005Z
- ended: 2026-11-11T23:31:44.916Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the ratification pass c0: revisions landed after you approved; you are checking only that they did not break what you approved of round 3. This pass's scope: the sections changed since you last reviewed: run `git diff f7521536a6bca93f4f9b4d4a4cfce3446567cdc7 4d07da4d5c80e4400db6d7f78e4ac2a8879824ce` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "severity": "minor",
    "location": "Chosen design → Reach classes and placements → Authoring bar (§2) — \"CI enforces what it can: the graded set … is graded in both hands, coordinates agree across hands, grades are non-decreasing in grid distance from that hand's home-graded keys, every rejected condition pinned by Test strategy #3's model tests\"",
    "problem": "new — the r3 closure makes Test #3 the sole pin for \"every rejected condition\", and two items in that same CI-enforced list break it. (a) Grid-distance monotonicity appears in no test anywhere: Test #3's clauses are per-hand distinctness, row grammar + class vocabulary, cross-hand coordinates, mirrored asymmetry, graded-set coverage, off-reach default, admitted-list InvalidValue — monotonicity is not among them, and no other Test strategy item names it, so the condition the bar says CI enforces is exactly the un-pinned one r1 #3 was about. (b) The bar governs both placement/v1/spec.md and reach-classes/v1/spec.md, and Adversarial posture maps PlacementNotIsometric/PlacementDegraded to #4, so naming only #3 contradicts the posture's own mapping. Fix: write \"every rejected condition pinned by Test strategy #3 and #4's model tests\", and add the monotonicity clause to Test #3 — or drop it from the CI-enforced list to review-only, which \"extended only by review\" already implies for the grad… [elided by the checkpoint cap]",
    "fix": "Add \"and #4\" to the authoring bar's pinning clause, and add \"grades are non-decreasing in grid distance from that hand's home-graded keys\" as a Test #3 clause (or demote the condition to review-only)."
  },
  {
    "severity": "minor",
    "location": "Test strategy #4 and #8 — \"a non-isometric fixture is rejected `PlacementNotIsometric`; a fixture exceeding `max-degrade` is rejected `PlacementDegraded`\" / \"… is rejected, each with its own fixture\"",
    "problem": "new — the closure fixed this wording for reach-classes only (Test #3's new \"in the loader, not in a fixture document\"); #4's and #8's conditions are produced by `validate`, which `parse` cannot see, so the bare word \"fixture\" points an implementer at `schemas/{placement,manifest}/v1/fixtures/invalid/`. A document placed there is asserted by the shipped fixture-walking harness (Test #1: \"each `fixtures/invalid/` case is rejected with the pinned `CoreError` variant\"; Test #13: \"requires no change to `crates/core/tests/conformance.rs`\") — it parses clean, so the harness assertion fails and the no-harness-change claim breaks. Adversarial posture's general rule (\"only where `parse` can see the condition … otherwise, over fixture data, by the model suite\") already forbids it; only these two test-strategy lines are loose.",
    "fix": "Carry #3's clause across once per site: in #4, \"a non-isometric placement pair is rejected PlacementNotIsometric by `validate`, not in a fixture document\" (same for the degraded and ungraded-key cases); in #8, \"each over its own manifest data, not in `fixtures/invalid/`\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains (`resolve`) — \"`Resolved { …, manifest: Option<ManifestPin> }`, whose `manifest` is `Some` whenever the profile declares `for`\" + \"`for` is required at parse and a `for` naming no manifest document in the index is `UnresolvedAncestor` …, so the check never runs against nothing\"",
    "problem": "new — the `Some`-whenever claim is stated of one document, while a chain carries the entry on every rung: §3 gives `patch/v1` \"the same optional chain entries\", so a base can pin `remnant2@0.1` (with `seeded-from` beside it) under a leaf patch that pins nothing, or pins `0.2`. `Resolved.manifest` is one slot for N pins, and the guarantee \"never runs against nothing\" follows only if the `seeded-from` check is read per-document against that document's own `for`; read against the rooted document's pin — the natural reading of \"the profile\" in `resolve(index, root, …)`, and the value the drift gate hands to `validate` (\"resolves every profile … against the manifest revision it pins\") — a chain whose base carries `seeded-from` and whose leaf carries no `for` answers `manifest: None`, and the dangling-preset case r1 #2 raised is vacuously green again for chains.",
    "fix": "One clause: `Resolved.manifest` is the rooted document's pin; the `seeded-from` check runs per-document against that document's own `for` (parse already pairs them); a chain document whose `for` is present and differs from the rooted document's is a rejection naming both document ids."
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
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/correctness-r3c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Manifests state game facts (Drift mechanism), with the same wording in Risks — \"The `cargo test` gate resolves every profile in `data/profiles/` against the manifest revision it pins\" / \"The canonical drift mechanism (contract 5) resolves every `data/profiles/` chain against its pinned revision inside `cargo test`\"","problem":"new — r4's narrowing made these two universals refer to nothing for part of their domain. r4 withdrew r3's \"`for` is required at parse\" and restored `for` as an optional entry (line 143: \"absent = a profile not bound to a game, which is what serves the ADR's accessibility and alternative-controller case\"; Revision log: \"a rule local to `seeded-from`, since `for` stays optional\"; Defaults: \"`for` absent = not bound to a game\"). These two sentences were written while every profile pinned a manifest and were not amended with the change: a profile/patch in `data/profiles/` with no `for` pins no revision, so \"the manifest revision it pins\" has no referent for exactly the case r4 says the model serves. The gate's behaviour for an unbound profile (resolve + `validate` with `manifest: None`, and no `seeded-from` possible per line 143) is inferable from §3, so the mechanism is not blocking, but the drift suite's scope is now set by inference in the one place that defines it — Test strategy #12 already carries the scoped form (\"for every profile that pins it\"), leaving §5 and the Risks row as the only unscoped statements.","fix":"Scope both by the key the lookup reads: §5 — \"resolves every profile in `data/profiles/` that declares `for` against the manifest revision it pins (a profile with no `for` is resolved and `validate`d with `manifest: None`, and cannot carry `seeded-from`)\"; Risks — \"resolves every `data/profiles/` chain that pins `for` against its pinned revision\". Replaces the universal quantifier with the one the gate uses; adds no mechanism."}]}
```
