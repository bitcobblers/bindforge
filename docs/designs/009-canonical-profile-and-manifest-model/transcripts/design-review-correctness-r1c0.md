# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:33:55.057Z
- ended: 2026-11-11T22:42:13.751Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/009-canonical-profile-and-manifest-model.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → Encoding rules (Round-trip split); Test strategy #1 — \"pure lookup data (`taxonomy`, `reach-classes`, `placement`) is asserted by load, using the exemption `reach-classes/v1/spec.md` already states\"","problem":"The shipped harness grants the load-asserted exemption only to format \"reach-classes\" (crates/core/tests/conformance.rs:63 gates it on dir.format == \"reach-classes\"); every other format's valid fixtures are re-serialized to an equal value by the unmodified harness (conformance.rs:47-78, exactly as verified assumption A8 states). Taxonomy and placement fixtures authored under the exemption the rule claims for them will fail the harness, contradicting Goals (\"machine-checkable by the existing harness with no harness change\") and Test strategy #14 (\"requires no change to crates/core/tests/conformance.rs\"). The split also leaves `policy` in neither half, though the harness round-trips it and it is a person-edited document.","fix":"State in the Encoding rules and Test strategy #1 that taxonomy, placement, and policy fixtures must re-serialize to an equal value (as the harness enforces today for every format except reach-classes), and reserve \"asserted by load\" for the seed data files (actions.toml, left.toml/right.toml), which live outside fixtures/ and are load-checked by the model tests. This replaces the claimed exemption mechanism; it adds nothing."},{"severity":"minor","location":"Chosen design → Manifests (validate); Test strategy #9 — \"`validate(manifest, &Taxonomy)` enforces: … a profile's `seeded-from` names a declared preset\"","problem":"The rule is a cross-document check (profile's seeded-from against the manifest's presets), but validate's declared inputs are the manifest and the taxonomy only, so it cannot evaluate this condition. Test strategy #9 repeats the mismatch by folding the seeded-from case into \"manifest validation\" with its own fixture.","fix":"Move that condition out of validate(manifest, &Taxonomy) into the chain-resolution/drift step: \"`resolve` rejects a `seeded-from` the pinned manifest does not declare, naming the profile id\" — resolve already loads the pinned manifest, so no new mechanism; mirror the sentence in Test #9."},{"severity":"minor","location":"Chosen design → Encoding rules (`version` bullet) vs Profiles are chains / Manifests — \"a document's own revision is a dotted `<major>.<minor>[.<patch>]` string\"","problem":"The profile section defines revision as \"a `<major>.<minor>` string\" and the `for` pin validates its revision half as \"a dotted `<major>.<minor>`\". A manifest declaring the encoding-rule-permitted patch component (revision = \"0.1.2\") can never be named by a game@revision pin: the same fact appears in two shapes and one is unpinnable.","fix":"Drop the `[.<patch>]` option from the Encoding rule: the v1 revision grammar is `<major>.<minor>` for every document that can be pinned. This replaces the current revision sentence."},{"severity":"minor","location":"Chosen design → Transforms (relocate-cluster) — \"the matched actions' current inputs form a contiguous cluster for `<hand>` in `base`\"","problem":"\"Contiguous cluster\" is a predicate no section defines over the data the design ships (reach tables carry class+row/col; placement::validate computes isometry and reach bound), and it decides apply-vs-Skipped. The design's own Options argument rejects review-only legality claims because \"two implementations could disagree\" — an undefined precondition predicate has the same defect.","fix":"Define the precondition in existing data: the matched actions' current input set must be isometric (same offset vectors) to the named placement's source-key set, else Skipped — replacing the word \"contiguous\"."},{"severity":"nit","location":"Chosen design → Profiles are chains (Chain guard) and Risks row — \"32 links is a base plus 31 patches\"","problem":"The shipped resolve_chain rejects only when derives.len() > MAX_CHAIN_DEPTH (crates/core/src/chain.rs:27), so the accepted maximum chain is 32 derives links = 33 documents — a base plus 32 patches. Test strategy #5's boundary (\"a chain of exactly MAX_CHAIN_DEPTH links resolves and one link deeper is ChainTooDeep\") is only true when links means derives entries, which makes the \"base plus 31\" phrasing off by one.","fix":"State the unit once — MAX_CHAIN_DEPTH counts derives links; the maximum chain is a base plus 32 patches (33 documents) — and carry it into the Risks row."}]}
```
