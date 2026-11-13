# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T11:36:43.554Z
- ended: 2026-11-13T11:39:10.528Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/012-input-vocabulary-and-reach-classes.md,
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
re-open one only if it contradicts docs/adrs/012-input-vocabulary-and-reach-classes.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/012-input-vocabulary-and-reach-classes/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Test strategy #5 — \"a one-step degradation (`home`→`stretch`) is legal at `(1)` and illegal at `(0)`\"","problem":"The degradation is two rank steps, not one: rank is defined as the class's index in the shared classes order, and the pinned order is [\"home\", \"thumb\", \"stretch\", \"far\", \"off-reach\"], so home→stretch = 2−0 = 2 and is illegal at GradeBound(1), not legal. The pinned suite case fails as written (the claim appears to use the ADR's set-listing order instead of this design's pinned severity order).","fix":"Make the example a real one-step move — home→thumb (e.g. a home key → keyboard:space) legal at (1), illegal at (0) — or relabel and move the bounds: home→stretch legal at (2), illegal at (1)."},{"severity":"major","location":"Chosen design, Worked examples — \"WASD→ESDF are legal at bound 0, which is the left-hander's WASD-to-the-right-hand move (the research's folklore rationale for the IJKL cluster...)\"","problem":"The hand attribution contradicts the shipped tables: left→right, A (left home, rank 0) → S (right off-reach, rank 4) degrades 4, so WASD→ESDF is illegal at bound 0 to the right hand; it is legal at bound 0 only as a same-left-hand one-column shift (E/S/D/F are left-hand keys). The folklore right-hand move is WASD→IJKL (legal at 0 to right). No single (from, to) hand pair makes all three pinned examples legal, and Test strategy #5 states no hands, so implemented from the prose it pins a failing case.","fix":"Attach the folklore clause to WASD→IJKL (from left, to right), describe WASD→ESDF as a same-hand (left→left) cluster shift one column right, and state each example's (from, to) hand pair in Test strategy #5."},{"severity":"minor","location":"The model / reach.rs — \"load_reach_tables gains two checks for a v2 directory: the declared version must equal the loading directory's version segment\"","problem":"Scoping the version-segment check to v2 directories misses the dangerous direction: a v2 document in a v1 directory passes the version gate (v2 is now supported) and would load unchecked, while Test strategy #3 pins the general rule (\"a document whose declared version differs from its loading directory's segment is refused\").","fix":"State the version-segment equality check as running for every reach-classes table at load, v1 directories included; the v1-on-v1 case passes trivially, keeping the reach_tables.rs suite green."},{"severity":"nit","location":"The model — \"InputGrade is the parsed pair: input: InputId, class: String\" beside \"grade(&InputId) -> Option<&ReachClass>\"","problem":"The class concept appears under two shapes (String in the parsed pair, ReachClass in the lookup); the shipped code already carries ReachClass.","fix":"Make InputGrade.class a ReachClass, or add one clause stating the loader validates the pair's class id against the canonical taxonomy and converts, so one concept has one type."},{"severity":"nit","location":"Placement legality, rule 4 — \"The PlacementCheck result carries the shape delta and the per-action degradation\"","problem":"Singular shape delta conflicts with Test strategy #5, which pins that a non-translation move set makes \"the check report both deltas\"; the result shape for the non-uniform case is unspecified by rule 4.","fix":"Say rule 4's shape delta is reported when the moves agree and the per-move deltas when they do not — the shape #5 already pins."}]}
```
