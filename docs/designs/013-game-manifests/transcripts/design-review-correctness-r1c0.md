# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T13:07:51.871Z
- ended: 2026-11-13T13:09:10.486Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/013-game-manifests/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/013-game-manifests.md,
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
re-open one only if it contradicts docs/adrs/013-game-manifests.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/013-game-manifests/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Delta inventory — Edited list: \"`crates/core/src/placement.rs` and both `is_geometry_name` call sites\" vs Unchanged list: \"`placement.rs`'s predicate\"","problem":"The same rename is listed as both edited and unchanged in adjacent lists; an implementer reading only the Unchanged list could skip the `is_geometry_name` → `is_slug_name` rename that the Edited list, Rollout 2, and Test strategy #7 require.","fix":"Qualify the Unchanged entry: \"`placement.rs`'s predicate *logic* (name change only, per the Edited list)\"."},{"severity":"minor","location":"Chosen design → Identity, revision, and the series pins — \"the loader enforces monotonicity forward: a PR whose `revision` did not move while its facts did is a review refusal\"","problem":"The sentence attributes enforcement to the loader while conceding in the same paragraph that \"core cannot pin a file's history\"; no shipped or planned code reads file history, and a facts-changed/revision-static document loads cleanly, so the stated enforcement does not exist.","fix":"Attribute the rule to review alone: \"review enforces monotonicity forward; the loader enforces only the parse rules above.\""},{"severity":"minor","location":"Chosen design → Identity, revision, and the series pins — \"the load-time join otherwise fails the workspace test\" (mitigation repeated in the Risks row \"the load-time join fails any manifest naming an id the registry does not back\")","problem":"No artifact this design ships or names loads `data/manifests/` in CI: the conformance walk covers `schemas/*/v*/` fixtures only, Rollout 6 ships no game data, and the cross-file matrix is deferred to ADR-014; the join fires only where a caller invokes `load_manifest`, so the claimed automatic failure for real manifests is unbacked.","fix":"State the enforcement owner: extend Rollout 6 with \"the first manifest PR adds a `load_manifest` test over its `data/manifests/` entry\" and qualify the Risks mitigation identically."},{"severity":"minor","location":"Goals / Non-goals and Chosen design → Identity — the pin ADR-014 spells for the ADR's \"manifest id and version\", alongside the entry table's `version` row: \"the grammar major this document is written against (`v1`)\"","problem":"The word \"version\" names two different values in the same design: the document's `version` entry (grammar major) and the ADR's \"id and version\" pin half, which this design defines as `revision`; a pin author could spell `remnant-2@v1` instead of `remnant-2@3`.","fix":"Add one sentence in Identity: \"the ADR's 'version' in 'id and version pin' is this document's `revision`, not its `version` entry (the grammar major).\""},{"severity":"nit","location":"Options considered — \"(`#` comment-stripping happens outside quotes, A19)\"","problem":"A19's pinned basis is that the reader strips `#` only at line start or after a complete entry; neither statement carries the operative fact that a `#` inside a quoted array element is data, which is what the Rules' \"Names may contain spaces, `#`\" relies on.","fix":"Align the parenthetical with the A19 wording: comment-stripping happens at line start or after a complete entry, never inside a quoted element."},{"severity":"nit","location":"The contract: `schemas/manifests/v1/` — example row `docked-actions = [\"ui.inventory Tab\"]`","problem":"The row cannot parse or load under the design's own rules today (`Tab` is outside the closed vocabulary; `ui.inventory` is not yet in the registry); the passage is annotated as not shippable, but the document's one full illustrative manifest is one `load_manifest` would refuse twice.","fix":"Make the example loadable with current-vocabulary rows and keep the deferred-vocabulary illustration in the parenthetical."}]}
```
