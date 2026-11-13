# Transcript: design-review:complexity r3c0

## Provenance
- stage: design-review:complexity (round r3c0)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:29:29.577Z
- ended: 2026-11-11T23:32:23.226Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the ratification pass c0: revisions landed after you approved; you are checking only that they did not break what you approved of round 3. This pass's scope: the sections changed since you last reviewed: run `git diff f7521536a6bca93f4f9b4d4a4cfce3446567cdc7 4d07da4d5c80e4400db6d7f78e4ac2a8879824ce` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/009-canonical-profile-and-manifest-model.md,
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
Your own previous findings (complexity seat):
[
  {
    "severity": "minor",
    "location": "Chosen design → Reach classes and placements → Authoring bar — \"grades are non-decreasing in grid distance from that hand's home-graded keys, every rejected condition pinned by Test strategy #3's model tests\"",
    "problem": "new (defect of this round's closure). The replacement for the deleted \"one invalid fixture per rejected condition\" names Test strategy #3 as the pin for every rejected condition, but Test #3's stated enumeration (per-hand distinctness, `<class> <row> <col>` parse, cross-hand coordinate agreement, mirrored-key asymmetry, graded-set coverage, `off-reach` default, admitted-list `InvalidValue`) has no monotonicity clause; \"non-decreasing in grid distance\" appears exactly once in the document, in this sentence. With the fixture obligation removed this round — r3's own reason is that a non-parse-visible condition cannot be pinned by a fixture — that CI-enforced grading condition now names no consumer at all. The clause also covers the `placement/v1/spec.md` half of the authoring checklist, whose conditions are Test #4's, not #3's. The theme (pinning claims naming no check) is closed everywhere else; this is the residue the closure left.",
    "fix": "Extend Test #3's existing enumeration with the monotonicity assertion (grades non-decreasing in grid distance from that hand's home-graded keys), and scope the clause to \"every reach-table condition pinned by Test strategy #3's model tests, every placement condition by #4\" — the same two tests, no new suite."
  },
  {
    "severity": "minor",
    "location": "Chosen design → 2. Input vocabulary + Reach classes loader rules + Test strategy #3 — \"`schemas/README.md` carries the admitted keyboard-name list: the admitted `keyboard:` vocabulary the reach tables draw from, enforced by the reach loader (a key outside it is `InvalidValue`) … No second key-name table is invented here.\"",
    "problem": "re-raised (my r1c3 finding 2; text unchanged this round, and the Revision log's r2 entry for that section records only the list's grading obligation, not a rebuttal of this point). The loader rule is a machine check, and the list's only stated home is prose in `schemas/README.md`; no `keyboard:` key name exists anywhere under `crates/` or `schemas/`, so `reach.rs` must carry its own US-ANSI name set to enforce it — a second key-name table kept in step with the README by eye, which the sentence denies. Test #3 pins only the behavior of one out-of-list key, never the list's contents, so README and code can drift in either direction.",
    "fix": "One clause naming the machine-readable home (a public constant in the crate that owns input ids, e.g. `keyboard_names()`, consulted by `reach.rs` and every input-id reader), with the README pointing at it so \"no second table\" stays true, and extend Test #3's existing admitted-list clause to assert the loader's set equals the README enumeration."
  },
  {
    "severity": "nit",
    "location": "Rollout 2 vs Chosen design → Adversarial posture — \"the entry sets and the `CoreError` variants named in Adversarial posture\"",
    "problem": "re-raised. The rewritten Adversarial-posture sentence still defers one variant (\"`MergeConflict` lands with `fork`/`merge` in the ADR-012 slice, which is where its producer is\"), while Rollout 2's blanket phrase names all thirteen variants as landing in the core step, so the carve-out reads as a contradiction at the rollout step.",
    "fix": "Two words in Rollout 2: \"the `CoreError` variants named in Adversarial posture, except the ones it defers to ADR-012\"."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/009-canonical-profile-and-manifest-model.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/complexity-r3c0.md. That file is the only path this session
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
{"verdict": "approve", "findings": []}
```
