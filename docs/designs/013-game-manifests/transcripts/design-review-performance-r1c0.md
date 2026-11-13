# Transcript: design-review:performance r1c0

## Provenance
- stage: design-review:performance (round r1c0)
- agent: review/design-performance
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T13:12:33.520Z
- ended: 2026-11-13T13:13:09.642Z
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
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/013-game-manifests.md.
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/013-game-manifests/reviews/design/performance-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → The contract → Rules → Budgets — \"`MAX_MANIFEST_ROWS = 256` bounds each of `actions`, `dead-keys`, and `docked-actions`\" (and the `presets` entry-table row)","problem":"`presets` is the one string-array in the closed entry set with no count cap; it is bounded only by the shared 1 MiB budget, so a hostile manifest can carry ~10⁴ preset ids. The per-element slug check and the \"each preset id appears once\" uniqueness rule then run superlinear in that count, contradicting the Risks table's mitigation that \"the four named budgets refuse before any join\" for the many-rows abuse. The fixture list and Test strategy #4 pin budgets that omit it.","fix":"Bring `presets` under the named budget family by reusing `MAX_MANIFEST_ROWS` (no new constant), add a `presets-past-cap.toml` invalid fixture, and extend the Test strategy #4 budgets case to \"257 `actions` rows, 257 presets, a 65th group — refuse\".","severity_note":"new"},{"severity":"minor","location":"Chosen design → The contract → Rules → Budgets — \"Named constants in `manifest.rs`, exported like the existing budget family (A6)\"","problem":"The four cap values (256 / 64 / 16 / 128) carry no derivation, while the adjacent Identity grammar bullet derives its 63-octet slug bound from RFC 1035. The values sit safely above the cited research baseline (40-id taxonomy, A7; TF2-scale bind lists, A9), so there is no contradiction, but the checklist's baseline-quoting item wants the caps tied to an observation rather than asserted.","fix":"Add one sentence after the constants stating the baseline each cap sits above (e.g. 256 rows ≈ several TF2-sized bind lists; 128 octets ≥ the longest surveyed multi-word native name).","severity_note":"new"}]}
```
