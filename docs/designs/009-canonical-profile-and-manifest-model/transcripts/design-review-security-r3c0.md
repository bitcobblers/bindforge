# Transcript: design-review:security r3c0

## Provenance
- stage: design-review:security (round r3c0)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:26:48.005Z
- ended: 2026-11-11T23:31:59.042Z
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
Your own previous findings (security seat):
[
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains → Resolution (`resolve` bullet) — \"`Resolved { bindings: …, chain: Vec<String>, manifest: Option<ManifestPin> }`, whose `manifest` is `Some` whenever the profile declares `for`\" / \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare\"",
    "problem": "The r3 closure made the pin's existence total (a dangling `for` is `UnresolvedAncestor`) but left *which* manifest a chain pins unruled, so the check can run against the wrong manifest. `for` is an optional entry on every chain document (§3: `patch/v1` \"carries the same optional chain entries\"), while a chain has one `manifest` field and one \"pinned manifest\" (§5: \"resolves every profile … against the manifest revision it pins\"). A shared chain whose base pins `remnant2@0.1` and whose tip pins an existing `remnant2@0.2` is a fully valid-document state with no stated rule: `resolve(root, 1)` (base alone) and `resolve(root, chain.len())` then evaluate `seeded-from` against different manifests, and the drift gate's verdict for one profile id becomes implementation/chain-order dependent — the same \"walk-order dependent\" determinism loss the adjacent `DuplicateId` clause exists to prevent, on the mechanism the design names as the only trust mechanism (validation, not provenance). This is th… [elided by the checkpoint cap]",
    "fix": "One clause in §3: the pin is a chain fact — a chain's documents declare one `for`, and a document whose `for` differs from the chain's is `InvalidValue` (no new `CoreError` variant, no new mechanism) — plus one Test #9 case beside the existing pin lookups, so `Resolved.manifest` has exactly one referent."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Profiles are chains → Id → file — \"Each tree accepts only the formats it holds — `data/profiles/` `profile`/`patch`, `data/manifests/` `manifest`\"",
    "problem": "The new format gate is stated as \"each tree\" but enumerates only the two indexed trees, while the design ships four walked `data/` trees: `data/placements/*.toml` is loaded content (Test #3, #4) and `data/policies/` carries the policy content Test #11 applies, and `schemas/README.md`'s data-homes rule says all of `data/` \"holds content discovered by walking\". A misfiled document in those two trees is again dispatched on the document's own declared `format`, with no gate named — the fail-closed family this clause closed for the indexed trees.",
    "fix": "Extend the parenthetical to `data/placements/` → `placement` and `data/policies/` → `policy`, same shipped `InvalidValue` at load; no new mechanism, and Test #9's planted-document sentence already covers the cases."
  },
  {
    "severity": "nit",
    "location": "Test strategy #8 (with Adversarial posture) — \"a manifest naming an unknown action id, an axis kind that disagrees with the taxonomy (`AxisKindConflict`), … is rejected, each with its own fixture\"",
    "problem": "The r3 pinning rule says such conditions are not carried by a `fixtures/invalid/` document (parse cannot see the taxonomy, and the shipped harness asserts that every file under `fixtures/invalid/` is rejected through `parse` — crates/core/tests/conformance.rs:138-154), yet #8's untouched \"each with its own fixture\" reads as if they live there. A reader who places #8's cases under `schemas/manifest/v1/fixtures/invalid/` makes the harness walk fail on documents that parse cleanly, instead of exercising the `validate` check intended.",
    "fix": "One phrase in #8: its cases are data under the model suite (as the posture's \"over fixture data\" already says), never under `schemas/*/v1/fixtures/invalid/`."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/security-r3c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Profiles are chains → Resolution (`resolve` bullet) + entry-set paragraph — \"whose `manifest` is `Some` whenever the profile declares `for`\" / \"`patch/v1` … carries the same optional chain entries\" / Revision log (r4): \"a rule local to `seeded-from`, since `for` stays optional\"","problem":"re-raised (open since r3, not a regression). The pin's existence is total, but which manifest a chain pins is unruled, and r4 makes that state plainly legal: `for` is an optional entry on every chain document (base and patch), while `Resolved` carries one `manifest` field and the design speaks of one \"pinned manifest\" (§5, \"the manifest revision it pins\"). A chain whose base declares `for = \"remnant2@0.1\"` and whose tip declares a valid `for = \"remnant2@0.2\"` is a fully valid document set with no stated rule, so `resolve(root, 1)` and `resolve(root, chain.len())` evaluate the `seeded-from` check against different manifests, and the drift gate reports a different revision for one profile id depending on which document's pin the implementation reads. That is the walk-order/implementation-dependent determinism loss the adjacent `DuplicateId` clause exists to prevent (\"an ambiguous pin key would make the drift gate's verdict walk-order dependent\"), on the mechanism ADR-009 names as the only trust mechanism (\"Provenance labels the chain; it never substitutes for validation\").","fix":"One clause in the §3 entry-set paragraph: the pin is a chain fact — a document whose `for` differs from the `for` of the document its `derives[0]` names is the shipped `InvalidValue` (no new `CoreError` variant, no new mechanism; an unbound chain declares `for` in neither place, so the accessibility case is untouched) — plus one Test strategy #9 case beside the existing pin-lookup sentences, so `Resolved.manifest` has exactly one referent."},{"severity":"nit","location":"Chosen design → Profiles are chains → Id → file — \"Each tree accepts only the formats it holds — `data/profiles/` `profile`/`patch`, `data/manifests/` `manifest`\"","problem":"re-raised, unchanged by r4. The gate is stated per tree and enumerates two of the four `data/` trees the design walks and loads (`data/{manifests,profiles,policies,placements}/`, Files created; `data/placements/` is load input for Test #3/#4, `data/policies/` for #10/#11), while `schemas/README.md`'s data-homes rule says `data/` \"holds content discovered by walking\". A document misfiled in the two unlisted trees is dispatched on its own declared `format` with no gate named — the fail-closed family this clause closed for the indexed trees.","fix":"Extend the parenthetical to `data/placements/` → `placement` and `data/policies/` → `policy`, same shipped `InvalidValue` at load; Test #9's planted-document sentence already covers the cases."},{"severity":"nit","location":"Test strategy #8 — \"a manifest naming an unknown action id, an axis kind that disagrees with the taxonomy (`AxisKindConflict`), … is rejected, each with its own fixture\"","problem":"re-raised, unchanged by r4. The r3 posture rule (\"pinned by a `fixtures/invalid/` document **only where `parse` can see the condition** … and otherwise, over fixture data, by the model suite\") excludes these conditions from a fixtures directory, and #8's `AxisKindConflict`/`UnknownAction`/`UnknownInput` cases are exactly the ones Adversarial posture routes to #8 as model-suite cases; the untouched \"each with its own fixture\" reads as `schemas/*/v1/fixtures/invalid/` placement, where the shipped harness asserts every file is refused through `parse` — the walk then fails on documents that parse cleanly, instead of exercising the intended `validate` check.","fix":"One phrase in #8: \"each with its own case in the model suite over fixture data\", never under `schemas/*/v1/fixtures/invalid/`."}]}
```
