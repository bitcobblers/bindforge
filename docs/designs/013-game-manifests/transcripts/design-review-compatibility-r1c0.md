# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T13:09:10.535Z
- ended: 2026-11-13T13:10:20.842Z
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
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/013-game-manifests.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/013-game-manifests/reviews/design/compatibility-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → The contract, Rules — Row encoding: \"the remainder a native name of 1–128 octets with no ASCII control character\"","problem":"Non-ASCII native names — the case driving assumption A13 (Steam IGA names 'with localized text') — are not settled or pinned: the permission sentence enumerates only spaces, `#`, and escaped glyphs, never raw non-ASCII octets; no valid fixture carries a non-ASCII native name; Test strategy #1/#5 round-trip only spaces, `#`, `\"`, `\\`. The octets-based cap hints raw UTF-8 is intended but the corpus that defines 'supported' pins nothing, and the matrix's non-Rust reader row makes spec.md+fixtures the whole contract for third-party readers, so valid/invalid classification and the byte-exact round-trip for localized names are undefined.","fix":"Add one clause to the row-encoding rule stating whether raw UTF-8 octets are legal in native names (and count toward MAX_NATIVE_NAME_OCTETS) or names are escape-only, and pin it with one valid fixture (e.g. non-ascii-native-names.toml) plus a round-trip case in Test strategy #1/#5."},{"severity":"minor","location":"Options considered → 'Constraints over keys, or over actions' + Compatibility matrix: \"all three constraint entries name input ids from the closed vocabulary (A8)\"","problem":"The design never states its per-platform OS input-API posture (X11 / Wayland / Windows Raw Input / macOS HID) or says outright that it targets none; the closed-vocabulary abstraction is the justification but is only implied via A8. Checklist item 2 requires the posture to be named or the abstraction justified in the text.","fix":"One line under Compatibility matrix: the manifest format performs no OS input capture, so per-OS input-API posture is unchanged by this design; input ids are vocabulary-level (A8) and their OS interpretation belongs to runtime/codec designs (ADR-017+)."},{"severity":"nit","location":"Compatibility matrix: \"Toolchain floor (1.85.0, edition 2024) | yes\" + Test strategy #8: \"the three gate commands ... are green on the Linux CI runner (the merge PR's job is the leg)\"","problem":"The matrix claims the toolchain floor as a supported environment, but no named test leg establishes compilation at 1.85.0 specifically; the CI leg does not identify its toolchain as the floor. The claim is plausible (std-only, Cargo.lock byte-unchanged) but unpinned.","fix":"One clause in the matrix row or Test strategy #8 noting that the CI rust job's pinned toolchain is the 1.85.0 floor (making the Linux leg the floor test), or that the floor claim rides the existing CI setup unchanged."}]}
```
