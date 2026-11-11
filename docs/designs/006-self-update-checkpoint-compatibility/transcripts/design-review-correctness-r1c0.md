# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T01:15:43.820Z
- ended: 2026-11-11T01:23:13.039Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/006-self-update-checkpoint-compatibility/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/006-self-update-checkpoint-compatibility.md,
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
re-open one only if it contradicts docs/adrs/006-self-update-checkpoint-compatibility.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/006-self-update-checkpoint-compatibility/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"blocker","location":"§1 Chosen design \"Fence runner-reserved paths in the ownership matrix\" — \"`stagePathAllowed(stage, path)` (`gates.ts:260–273`) gains a first rule that precedes the per-stage rules: return `false` when the path's last segment is `STATE_FILE`\"","problem":"new — The fence lands in the same author-agnostic predicate enforceOwnership uses for its violation set (runner.ts:260–261: own-commit paths + dirtyFiles), which is A3's own cited basis. At every boundary sweep the checkpoint is dirty by the runner's own save (pass writes drain before the sweep at runner.ts:1871; checkpoint() re-saves after the commit at runner.ts:2298, 2336), so the sweep reverts or deletes the runner's own checkpoint writes: mid-run boundary commits never advance the committed copy (contradicting A7's `verified` \"newest good state at every stage boundary\"); a new run's untracked checkpoint is deleted at the first sweep and never committed until terminal, so §3's `git checkout HEAD --` repair, the loadState message, and Test #13 reference a path absent from HEAD; and resume's ESCALATION.md deletion (runner.ts:1983–1989) is restored at the next sweep, making the split-record line misfire. Test #2's \"byte-identical to headBefore\" assertion is exactly the misfire output, so no listed test catches it.","fix":"Keep the stagePathAllowed rule for attribution/reporting, but replace §1's third-bullet behavior (\"the sweep restores the file from `headBefore` and removes the planted tree\") with a reserved-name branch in the sweep: restore the path to the runner's authoritative content — re-save from the in-memory PipelineState via checkpoint() for .pipeline.json, re-apply resume's deletion for ESCALATION.md — instead of headBefore restore or removal."},{"severity":"minor","location":"§2 cross-version resume contract — \"name the operator's two postures — resume with the pipeline that wrote it, or start a fresh run after archiving the run directory (ADR-006:55–57)\"","problem":"new — The cited ADR lines give a different pair: \"`git checkout HEAD -- docs/designs/<run>/.pipeline.json` or a fresh run\". The quoted first posture appears nowhere in the ADR and the ADR's first posture is missing from the pair; the refusal message and docs/workflows.md inherit an unsupported attribution.","fix":"Cite ADR-006:55–57 only as the precedent for naming operator postures in an error, and present the refusal pair as this design's own addition, or restate the pair to match the ADR's."},{"severity":"nit","location":"§1 — \"starts with `${STATE_FILE}.` (the `saveState` tmp pattern, `state-store.ts:186`)\"","problem":"new — The prefix rule fences any name beginning `.pipeline.json.` at any depth, so a legitimate doc file such as `.pipeline.json-notes.md` can never be written by a stage.","fix":"Either state the breadth is deliberate and pin Test #1 on a neighboring name being forbidden, or match the tmp shape exactly (`/^\\.pipeline\\.json\\.[0-9a-f]{8}\\.tmp$/` on the last segment)."}]}
```
