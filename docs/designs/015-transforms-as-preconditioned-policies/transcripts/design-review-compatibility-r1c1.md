# Transcript: design-review:compatibility r1c1

## Provenance
- stage: design-review:compatibility (round r1c1)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T16:15:10.886Z
- ended: 2026-11-13T16:15:37.018Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/015-transforms-as-preconditioned-policies/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff fc0a1f941a6ba45453bb46102fb477d4f37c5b46 0dacb9675f15c96eeca2ddfc7d0e5f16fcc70498` (both are blob ids of 02-proposal.md), then read the full current document.
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
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "Compatibility matrix (and the Risks row \"Policies are effectively ANSI-US-tested (A20)\")",
    "problem": "The Risks mitigation asserts the ANSI-US geometry floor is \"stated in the Compatibility matrix\", but no matrix row states it: the eight rows cover Linux CI, Windows contributor, macOS contributor, toolchain floor, pre-015 core, existing formats, third-party reader, product artifacts. The row that comes closest (\"Existing formats, fixtures, loaders, `resolve` | yes (untouched)\") names keyboard-geometry only as byte-exact, i.e. not edited — not as single-geometry. This is checklist item 5 (non-QWERTY layouts) and it is load-bearing, not cosmetic: relocate-cluster's whole gate is grid-dependent (\"the source keys not connected by rook-adjacency (`|Δcol| + |Δrow| = 1`) in the loaded grid\" plus placement_is_legal(..., tables, geometry)), so 'a v1 policy is only validated against one grid' is a compatibility property of the layer. new.",
    "fix": "Add one matrix row and let the existing Risks cross-reference point at it: `| Keyboard geometry | yes (`ansi-us` only) | relocate-cluster's contiguity and legality read the loaded grid; exactly one geometry ships (A20). ISO/JIS is new table files, not logic, and no policy may claim non-ANSI support until they land |`. No new mechanism — this states the A20 fact the `ISO/JIS grid tables` Deferred entry already carries."
  },
  {
    "severity": "minor",
    "location": "Deferred / out of scope and Compatibility matrix — keyboard reality",
    "problem": "Of the four keyboard-reality sub-items in checklist item 5, three are handled (non-QWERTY via A20/Risks/ISO-JIS deferral; dead keys via the untouchable guard \"reads or writes an input appearing in `docked-actions`, `dead-keys`, or any `exclusive-groups` member\"; non-ASCII names via the slug grammar \"slug grammar, ≤ 63 octets (A15)\" and the `[a-z][a-z0-9-]*` tag prefix). IME/composition appears nowhere in the document — not in the matrix, Risks, or Deferred — so a reader cannot tell whether it was considered and dismissed or missed. new.",
    "fix": "One clause on the new keyboard-geometry row (or one Deferred line): \"IME/composition — out of scope for v1: the closed `keyboard:` vocabulary names physical inputs (A19) and no text-entry input is modeled or matched; `load_policy`/`apply` never see a composed character.\""
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix — \"Product artifacts (CLI, desktop, hub) | not applicable\"",
    "problem": "The row justifies shipping no binary, but the matrix never names the OS input APIs checklist item 2 asks about (X11, Wayland, Windows Raw Input, macOS HID) for the layer that is shipped — the core/transforms library — so the \"no platform-specific input behaviour\" position is implied rather than stated. new.",
    "fix": "Extend that row's note (or the \"Third-party reader, non-Rust\" row) with: \"v1 touches no OS input API: the contract is parsed documents in, parsed documents out.\""
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/015-transforms-as-preconditioned-policies.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/015-transforms-as-preconditioned-policies/reviews/design/compatibility-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[]}
```
