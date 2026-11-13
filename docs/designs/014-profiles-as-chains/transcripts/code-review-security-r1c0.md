# Transcript: code-review:security r1c0

## Provenance
- stage: code-review:security (round r1c0)
- agent: codereview/code-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T15:17:56.321Z
- ended: 2026-11-13T15:19:59.243Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/014-profiles-as-chains/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 4cfcd8db4477247e8e6ecb26ecade40451e0ad91...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/014-profiles-as-chains.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the security checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/code/security-r1c0.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"crates/core/src/document.rs:1101-1116 (is_label_char); escape side at crates/core/src/error.rs:173","problem":"The provenance-label policy excludes char::is_control() (C0+C1) plus an explicit bidi/default-ignorable list, but not U+2028 LINE SEPARATOR or U+2029 PARAGRAPH SEPARATOR, and one_line escapes only is_control characters too. toml.rs:230 decodes \\uXXXX escapes, so a generated-by label carrying U+2028 parses and ships, and a refusal echoing kind/format/a delta element can carry that code point into a message the contract fixes as 'a single bounded line' — the conformance assertion only checks for '\\n' and '\\r' (tests/conformance.rs:295). The helper's own comment claims the excluded set is the one that can steer what a reader sees; line separators are exactly that.","fix":"Add '\\u{2028}' | '\\u{2029}' to the matches! list in is_label_char, widen the spec.md generated-by sentence to say so, and pin it with an invalid fixture (generated-by-line-separator.toml) beside generated-by-bidi-char.toml; if the log-line invariant is meant to cover echoed values generally, also escape those two code points in one_line."},{"severity":"minor","location":"crates/core/src/chain_ops.rs:650-656 (inherit_provenance), applied at 334-335","problem":"The function's doc says a label is inherited only when the two operands agree and a disagreement is dropped, but the arm (Some(one), None) => Some(one) copies a one-sided label onto the merged document: merging a patch asserting generated-by = \"human\" with a silent one yields a document authored by merge that claims a producer neither operand asserted for it — the provenance-drift class the contract refuses elsewhere. Impact is bounded by ADR Decision Four (provenance never consumed as trust), hence minor.","fix":"Inherit only when both operands carry equal labels ((Some(a), Some(b)) if a == b) and drop otherwise, or state the one-sided rule in spec.md's provenance paragraph and correct the comment; pin the chosen rule with a merge test."},{"severity":"minor","location":"crates/core/src/chain_ops.rs:315-333 (merge emit path), with merge_touch_order at 679-690 and emit_composition at 746-789","problem":"MAX_PATCH_DELTA_ELEMENTS is enforced only at parse (document.rs:914-928); nothing bounds what merge writes out. Touched actions per operand reach 1024 (256 set + 256 clear + 2x256 swap halves) and the union 2048, and emit_composition writes one element per touched action, so a grammar-valid pair of operands can yield a merged patch whose set or clear list exceeds the 256 cap — a Profile no reader of the same grammar accepts, so the refusal surfaces at the next load_chain instead of at the operation, and the composite-cost statement in spec.md no longer holds for merge output.","fix":"After emit_composition, refuse with the same InvalidValue shape delta_elements uses when any emitted list passes MAX_PATCH_DELTA_ELEMENTS, and pin the refusal with a test in tests/chain_ops.rs."}]}
```
