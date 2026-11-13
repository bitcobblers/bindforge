# Transcript: design-review:correctness r1c4

## Provenance
- stage: design-review:correctness (round r1c4)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T11:51:57.242Z
- ended: 2026-11-13T11:52:43.401Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md.
This is the ratification pass c4: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff b08752d231afb7a16804b0e64695b96789c4e9e5 92beeaa7f1643f271998f178ab30f94d15b533dd` (both are blob ids of 02-proposal.md), then read the full current document.
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
[
  {
    "severity": "minor",
    "location": "Placement legality, worked examples + Test strategy #5 — \"Moving an action from a `home` key to `keyboard:;`, `off-reach` for the source hand, is illegal at bound 1\" / \"the predicate, each case naming its `(from, to)` hands\"",
    "problem": "The revision promises a `(from, to)` hand pair per case but this example names none, and its verdict is hand-pair dependent: rule 3 grades the target by the to hand, and `keyboard:;` is `home` for the right hand, so under `left`→`right` — the document's own motivating left-hander move — the degradation is 0 and the move is legal even at bound 0, contradicting the example. Test strategy #5's matching case (\"a source-`home` to `off-reach` move is illegal at `GradeBound(1)`\") is well-defined only if \"off-reach\" is read as the to-hand grade, while the worked example deliberately keys off the source hand.",
    "fix": "Pin the example's hand pair (e.g. `left`→`left`) or restate it as \"`off-reach` for the placement's to hand\", and either apply the promised per-case hand pairs to the remaining #5 cases or scope the promise to the three WASD cases."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/012-input-vocabulary-and-reach-classes.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/012-input-vocabulary-and-reach-classes/reviews/design/correctness-r1c4.md. That file is the only path this session
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
