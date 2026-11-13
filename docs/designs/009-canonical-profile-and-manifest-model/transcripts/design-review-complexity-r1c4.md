# Transcript: design-review:complexity r1c4

## Provenance
- stage: design-review:complexity (round r1c4)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:11:13.560Z
- ended: 2026-11-11T23:14:47.326Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the ratification pass c4: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 5bbcc09d65524a3e2c522e58501451480f23d8ee f7521536a6bca93f4f9b4d4a4cfce3446567cdc7` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design → 4. Transforms produce bases — \"`apply_policy(base: &Resolved, policy: &PolicyDoc, ctx: &Ctx) -> PolicyOutcome` with `PolicyOutcome { base: Resolved, skipped: Vec<Skipped> }`\" + \"`apply_policy` validates every invocation against the table … and a mismatch is the shipped `InvalidValue { key, message }` naming the invocation, raised before any transform runs\"",
    "problem": "new (residue of my accepted r1 closure). Moving the invocation check out of core's reader left the pinned signature unable to carry the pinned rejection: `apply_policy` returns a bare `PolicyOutcome`, which has no error channel, so the `CoreError::InvalidValue` that §4, Adversarial posture (\"the invocation table's arity and grammars in `apply_policy`\") and Test strategy #10 all require has nowhere to go. The Goals pin function signatures as the deliverable, so an implementer hits a contradiction between two statements of the same section. The r1 theme (validation split across crates) is closed; this is only the return channel.",
    "fix": "Make the return type `Result<PolicyOutcome, CoreError>` in that one signature (replacing `-> PolicyOutcome`), and state that the whole-`stack` table check runs first, so its `InvalidValue` arrives as `Err` while inapplicable transforms keep returning `Ok(PolicyOutcome)` with `skipped` populated. One signature, no new type."
  },
  {
    "severity": "minor",
    "location": "Chosen design → 2. Input vocabulary + Reach classes loader rules + Test strategy #3 — \"`schemas/README.md` carries the admitted keyboard-name list: the admitted `keyboard:` vocabulary the reach tables draw from, enforced by the reach loader (a key outside it is `InvalidValue`)\" / \"every key is a `keyboard:` id in the admitted list, else `InvalidValue`\"",
    "problem": "new. The admitted list gained its first consumer this round, which makes true a duplication the same paragraph denies. The list is prose in `schemas/README.md`, and no machine-readable admitted-name table exists anywhere in the repo (nothing under `crates/` or `schemas/` names a `keyboard:` key today), so `reach.rs` must carry a hard-coded US-ANSI name set of its own — a second key-name table kept in step with the README by eye. Test strategy #3 pins only the behavior of one out-of-list key, never the list's contents, so README and code can drift in either direction, and \"No second key-name table is invented here\" is false as the text stands.",
    "fix": "One clause naming where the machine-readable admitted list lives (a public constant in the crate that owns input ids, e.g. `keyboard_names()`, which `reach.rs` and every input-id reader consults), replacing the prose-list-only reading, with the README pointing at it so \"no second table\" stays true. Extend Test #3's existing clause to assert the loader's admitted set equals the README enumeration, so the pair is pinned."
  },
  {
    "severity": "nit",
    "location": "Rollout 2 vs Chosen design → Adversarial posture — \"the entry sets and the `CoreError` variants named in Adversarial posture\"",
    "problem": "new. The Adversarial-posture sentence now excludes one variant from this slice (\"each pinned by a fixture in this slice except `MergeConflict` — it lands with `fork`/`merge` in the ADR-012 slice\"), while Rollout 2's blanket phrase names all thirteen variants as shipping in the core step, so the r2 carve-out reads as a contradiction at the rollout step.",
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
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/complexity-r1c4.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Reach classes and placements → Authoring bar — \"grades are non-decreasing in grid distance from that hand's home-graded keys, every rejected condition pinned by Test strategy #3's model tests\"","problem":"new (defect of this round's closure). The replacement for the deleted \"one invalid fixture per rejected condition\" names Test strategy #3 as the pin for every rejected condition, but Test #3's stated enumeration (per-hand distinctness, `<class> <row> <col>` parse, cross-hand coordinate agreement, mirrored-key asymmetry, graded-set coverage, `off-reach` default, admitted-list `InvalidValue`) has no monotonicity clause; \"non-decreasing in grid distance\" appears exactly once in the document, in this sentence. With the fixture obligation removed this round — r3's own reason is that a non-parse-visible condition cannot be pinned by a fixture — that CI-enforced grading condition now names no consumer at all. The clause also covers the `placement/v1/spec.md` half of the authoring checklist, whose conditions are Test #4's, not #3's. The theme (pinning claims naming no check) is closed everywhere else; this is the residue the closure left.","fix":"Extend Test #3's existing enumeration with the monotonicity assertion (grades non-decreasing in grid distance from that hand's home-graded keys), and scope the clause to \"every reach-table condition pinned by Test strategy #3's model tests, every placement condition by #4\" — the same two tests, no new suite."},{"severity":"minor","location":"Chosen design → 2. Input vocabulary + Reach classes loader rules + Test strategy #3 — \"`schemas/README.md` carries the admitted keyboard-name list: the admitted `keyboard:` vocabulary the reach tables draw from, enforced by the reach loader (a key outside it is `InvalidValue`) … No second key-name table is invented here.\"","problem":"re-raised (my r1c3 finding 2; text unchanged this round, and the Revision log's r2 entry for that section records only the list's grading obligation, not a rebuttal of this point). The loader rule is a machine check, and the list's only stated home is prose in `schemas/README.md`; no `keyboard:` key name exists anywhere under `crates/` or `schemas/`, so `reach.rs` must carry its own US-ANSI name set to enforce it — a second key-name table kept in step with the README by eye, which the sentence denies. Test #3 pins only the behavior of one out-of-list key, never the list's contents, so README and code can drift in either direction.","fix":"One clause naming the machine-readable home (a public constant in the crate that owns input ids, e.g. `keyboard_names()`, consulted by `reach.rs` and every input-id reader), with the README pointing at it so \"no second table\" stays true, and extend Test #3's existing admitted-list clause to assert the loader's set equals the README enumeration."},{"severity":"nit","location":"Rollout 2 vs Chosen design → Adversarial posture — \"the entry sets and the `CoreError` variants named in Adversarial posture\"","problem":"re-raised. The rewritten Adversarial-posture sentence still defers one variant (\"`MergeConflict` lands with `fork`/`merge` in the ADR-012 slice, which is where its producer is\"), while Rollout 2's blanket phrase names all thirteen variants as landing in the core step, so the carve-out reads as a contradiction at the rollout step.","fix":"Two words in Rollout 2: \"the `CoreError` variants named in Adversarial posture, except the ones it defers to ADR-012\"."}]}
```
