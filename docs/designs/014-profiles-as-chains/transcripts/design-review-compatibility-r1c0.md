# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T14:05:51.654Z
- ended: 2026-11-13T14:07:03.954Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/014-profiles-as-chains/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/014-profiles-as-chains.md,
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
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/014-profiles-as-chains.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/design/compatibility-r1c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Delta inventory (Edited) — \"`PROFILE_ENTRIES` renamed `PROFILE_V1_ENTRIES`, new `PROFILE_V2_ENTRIES`\"","problem":"The rename is presented as an ordinary edit, but the proposal never states whether PROFILE_ENTRIES is private or part of core's public surface; the zz_api_probe.rs edit is listed for 'the new public names' only, so nothing says whether existing downstream users of the old name break, and the Compatibility matrix has no core-API row.","fix":"State in the Edited list (or a matrix row) that the rename is internal because PROFILE_ENTRIES is private — as the assumption register already does for placement::is_slug_name (pub(crate)) — or, if it is public, record the rename as a public-surface break in the matrix; do not add a compat alias."},{"severity":"minor","location":"The contract: schemas/profile/v2/ — \"The `spec.md` also states the load-level rules below (the directory rule, pin equality, the joins)\"","problem":"The keyboard-reality check is only half covered: non-ASCII binding names are refused (slug grammar, uppercase fixtures) and named keys are Deferred consistently with the ADR, but chain documents bake input ids like `keyboard:d` into artifacts shared across players (the example carries `set = [\"move.steer-left keyboard:d\"]`), and nowhere — contract, matrix, or Deferred — is it stated that the physical-key semantics of the closed `keyboard:` vocabulary (character vs position on non-QWERTY layouts) are inherited unchanged from the shipped input/reach-classes contract and are not this design's to define.","fix":"One sentence appended to the spec.md load-rule list pinned in Rollout 1: input-id semantics are inherited unchanged from the shipped contract and a chain is layout-neutral data. It adds no mechanism; it clarifies the boundary the determinism rules already assume."},{"severity":"nit","location":"Compatibility matrix — \"macOS contributor | no (v1)\"","problem":"The status cell \"no (v1)\" is cryptic: a first-time reader cannot tell which posture '(v1)' names or whether a 014-era contribution on macOS is in or out.","fix":"Spell the cell out, e.g. \"no — outside design 008's supported matrix\", and drop the bare \"(v1)\" marker."}]}
```
