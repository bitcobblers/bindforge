# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T16:07:19.816Z
- ended: 2026-11-13T16:10:19.186Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/015-transforms-as-preconditioned-policies/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/015-transforms-as-preconditioned-policies.md,
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
re-open one only if it contradicts docs/adrs/015-transforms-as-preconditioned-policies.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/015-transforms-as-preconditioned-policies/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → The stack / The match rule — \"the produced `set` list preserves the source base's element order in place\"; The stack → swap-mouse-buttons — \"finds the actions currently bound to `mouse:left` and `mouse:right`\"","problem":"The shipped layout model is action→input (chain_ops.rs:167 BTreeMap<ActionId, InputId>), and profile/v2 never refuses one input carrying two actions — only reach tables and geometry explicitly refuse repeated inputs (document.rs:1231,1276) — while duplicate set elements are legal data (profile/v2/spec.md:81 'Duplicates are data'). On a grammar-valid, hostile-authored source base (the design's own untrusted-input posture), 'the action currently bound to mouse:left', 'a target already bound to an action outside the match set', and 'element order in place' for a duplicated action id are each underdetermined — the one corner where the byte-identical-output and closed-reason-vocabulary contracts are not fully pinned.","fix":"One sentence in The stack: apply first collapses the base's `set` to resolve's own action→input map (last element wins), and a source base with more than one action bound to one input is an entry refusal reusing InvalidValue naming the input (the A9 'could not evaluate' channel); pin both with invalid-base tests in crates/transforms/tests/apply.rs."},{"severity":"nit","location":"Assumption register A17 — \"ADR-004 binds a test tree to mirror `src/`: a module-named test file is that module's unit test\" (Basis: docs/adrs/004-mirror-src-hierarchy-in-tests.md:23-33)","problem":"The cited range is ADR-004's Context (the TS pipeline- prefix list), and ADR-004's title and Decision are scoped to 'the orchestration test tree'; the naming rule the row states is at lines 49-53 ('Naming rule going forward… New subsystems follow the same shape'). The claim is true as repo practice — core already ships module-mirrored Rust tests (tests/placement.rs, tests/manifest.rs, tests/chain_ops.rs) — but the cited lines do not contain it, so the verified row's basis citation is wrong.","fix":"Cite docs/adrs/004-mirror-src-hierarchy-in-tests.md:49-53 and add the shipped module-mirrored Rust test files (e.g. crates/core/tests/placement.rs) as the practice anchor."},{"severity":"nit","location":"Assumption register A13 — \"the exported budget family is nine constants — … `MAX_CHAIN_DEPTH` 32, the four manifest caps, the two chain caps\"","problem":"The nine exported MAX_ constants are the two toml caps, the four manifest caps, MAX_CHAIN_DEPTH, and the two 014 profile caps (MAX_PATCH_DELTA_ELEMENTS and MAX_GENERATED_BY_OCTETS at document.rs:165,170); the enumeration double-counts MAX_CHAIN_DEPTH and labels the 014 profile caps 'chain caps', and the cited basis omits document.rs. The nine-count itself is confirmed by grep.","fix":"Replace 'the two chain caps' with 'the two `profile/v2` caps from 014 (`MAX_PATCH_DELTA_ELEMENTS`, `MAX_GENERATED_BY_OCTETS`)' and add crates/core/src/document.rs to the Basis."}]}
```
