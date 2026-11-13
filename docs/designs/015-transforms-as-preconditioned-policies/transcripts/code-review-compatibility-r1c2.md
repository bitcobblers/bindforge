# Transcript: code-review:compatibility r1c2

## Provenance
- stage: code-review:compatibility (round r1c2)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T17:27:30.232Z
- ended: 2026-11-13T17:29:55.382Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/015-transforms-as-preconditioned-policies/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 39d4fcb589fd14ce2c81ea01572c84dbd4b28d39..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/015-transforms-as-preconditioned-policies.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/015-transforms-as-preconditioned-policies/reviews/code/compatibility-r1c2.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "schemas/policy/v1/spec.md:195-200 (code: crates/transforms/src/apply.rs:535-549, test: crates/transforms/tests/apply.rs:340)", "problem": "The amended occupancy clause states that every action holding a target key is asked, but never says which holder the report names when a key has more than one holder outside the match set. Core answers with the first foreign holder in source-base `set` element order (`holders_of(target).find(|h| !matched.contains(*h))`), while spec.md:179-381 and apply.rs:29-31 declare the named offender part of the contract's determinism surface, and the Compatibility matrix's \"Third-party reader, non-Rust\" row makes spec.md the whole contract — so a conforming non-Rust implementer may name a different holder (e.g. the byte-order-smallest) and still claim to implement policy/v1. Before this diff the rule was a single `action_on` read, so the omission was harmless; the diff turns it into an unstated selection rule, and no test pins it: the new case needles only the target key (`assert_offender_names(offender, &[\"keyboard:s\"], ...)`) and its base has exactly one foreign holder.", "fix": "State the selection rule in the `occupied-target` clause — e.g. \"the named holder is the first action, in the source base's `set` element order, that holds the target and sits outside the match set\" — mirror it in `foreign_holder`'s doc comment, and pin it with a case whose target is held by two foreign actions asserting the expected holder by name."}, {"severity": "nit", "location": "schemas/policy/v1/spec.md:163-169 (code: crates/transforms/src/apply.rs:301-306, test: crates/transforms/tests/apply.rs:1133-1162)", "problem": "\"The refusal names the duplicated action\" is ambiguous for a base that duplicates more than one action, a shape the new rule makes reachable. Core names the first duplicate in `set` element order (`duplicated_action` scans the unsorted element list), whereas crates/transforms/src/lib.rs:37-40 and apply.rs:29-31 attribute the crate's determinism to sorted containers. The new test duplicates exactly one action, so the multi-duplicate answer is unpinned and two conforming readers can name different ids.", "fix": "Name the tie-break in the spec bullet (\"the first duplicated id in `set` element order\", or byte-order-smallest if that is intended), keep the code doc worded the same, and add one base duplicating two ids asserting which one the refusal names."}]}
```
