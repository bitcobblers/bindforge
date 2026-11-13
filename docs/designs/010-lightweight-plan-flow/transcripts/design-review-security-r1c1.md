# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T01:56:53.546Z
- ended: 2026-11-12T01:57:57.965Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 9ae29b5ab2cae633405db0d35b4d4adaae4c0ebd 24b804a9c29888a3b458242d27c370d6b48be4fb` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/010-lightweight-plan-flow.md,
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
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Options considered, \"Seats in `PipelineConfig`\" + Chosen design → Budget resolution: \"`funnelSpecialties(phase, cfg)` and `states.ts`'s `specialtiesFor(phase, cfg)` read `cfg.reviewSeats` and fall back to the five specialties when absent\"",
    "problem": "The review panel — the pipeline's audit gate — becomes unvalidated config data. `configFor` spreads `base` (A4's `opts.cfg`/CLI seam) before `FLOWS[flow].config`, and `FLOWS.full.config` is documented as \"an empty partial\", so for a full run a caller-supplied `reviewSeats` is the effective panel with no validation anywhere in the design: `reviewSeats.code: []` makes `seatsConverged` vacuously true (A2 lists it as a consumer of the seat list), so `case 'code-review'` converged routes to `doc-sync`/`done` with zero review sessions and no escalation, and `reviewSeats.code: ['corectness']` reaches seat launch with a name no agent backs. The design itself canonicalises the empty list (`reviewSeats.design: []`), so \"empty = this phase has nobody\" is now legal in the same field that carries the code panel. `planMaxLines`/`planMaxFiles` are equally unbounded. Fix: put the validation in the documented `configFor` contract and replace the bare \"fall back to the five specialties when absent\" clau… [elided by the checkpoint cap]",
    "fix": "State the seat/config validation inside the `configFor` contract (known-specialty names; non-empty panel for every phase the flow runs; positive-integer plan caps; throw pre-lock), replacing the \"fall back to the five specialties when absent\" clause with a presence-based read plus those rules, and pin it with three Test strategy cases (empty code panel, unknown specialty, base seats overridden by the light registry)."
  },
  {
    "severity": "minor",
    "location": "The `plan` stage: \"`mode: primary`, edits `docs/designs/**`, denies `.pipeline.json`\"",
    "problem": "The plan's write surface is specified only as agent permissions, and the design's own reasoning says that is not the gate: \"the gate is code — agent permissions and skill text are advisory (A13)\". The Ownership section's deltas name `implement`/`fix`, \"test-engineer stages and document stages\", and `stagePathAllowed` keys on the stage name (A8), so the new name `plan` appears in no stated code rule. Test strategy #18 pins only `plan` writing `src/**`, and Test strategy #7's reserved-name enumeration (\"`implement`, `fix`, `test-authoring`, `test-update` and document stages\") omits `plan`, whose designated directory is exactly where `.pipeline.json` sits.",
    "fix": "Add one clause to Ownership: `plan` is governed by the document-stage rule in both flows (writes confined to the run's `docs/designs/NNN-<slug>/` document paths, `namesReservedSegment` first), and add `plan` to Test strategy #7's reserved-name list (`.pipeline.json`, the lock, `ESCALATION.md`, a `saveState` tmp) for both flows."
  },
  {
    "severity": "minor",
    "location": "The `plan` stage (Oversized-plan gate) + Risks: \"a missing section counts as 0 items (a faithfulness concern for the seats, not a gate)\"",
    "problem": "The file gate fails open in exactly the case it was added for. It is an exact heading string plus a bullet count, so `## Files to Touch`, `## Files to touch:`, an indented list, or a numbered list all count 0 — \"80 lines naming 20 files passes a line count with room to spare\" also passes the file count silently. The section the gate reads is the only signal that section carries, and the two seats left are `correctness` and `testing`, not design reviewers, so the stated hand-off to seat faithfulness does not cover it; the mitigation row \"`planMaxFiles` catches files-spanned\" is then untrue as written.",
    "fix": "Make the matcher canonical (case-insensitive exact heading after trimming; items = `-`/`*`/numbered entries at any indent under the section) and treat a missing section as a denial with its own escalation string (`plan names no Files to touch section: set flow: full and re-run with --force`), replacing \"a missing section counts as 0 items\" — `prompts/plan.md` renders the section, so absence is a contract violation, never a legal plan."
  },
  {
    "severity": "nit",
    "location": "Selecting the flow: \"`state.flow` is set in `runPipeline` immediately after `freshState`, so **the entry save carries it**\"",
    "problem": "The section describes two reads — the pre-lock validation \"reads the source text right after `deriveIdentity`\", then the entry save — and never says the stamped value is the value that passed validation. If the save re-parses, an edit to the source document between the two reads changes the effective flow after the pre-lock usage-error gate approved a different one.",
    "fix": "State that the value validated pre-lock is carried on the `deriveIdentity` result and is the sole input to `state.flow`: one frontmatter read per entry, no re-parse at save."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/010-lightweight-plan-flow.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Ownership — the security spine, widened in one flow — \"Paths reach `stagePathAllowed`/`ownershipIssue` repo-root-relative and POSIX-normalized (A8/A9), so the `flow` parameter parses no path: separator and case variants are settled before the predicate sees them.\"","problem":"This is the load-bearing premise for the section's path-safety argument and for the new Test strategy #7 clause, and it is cited to A8/A9, which do not record it: A8 records the predicate's shape (`namesReservedSegment` first, `TEST_PATH_RE` denials), A9 that move validation reuses the same predicate. No register row names the code that canonicalizes paths, rejects absolute forms, or rejects `..`, so the claim the security argument rests on is not confirmable from the design's own inputs, and no behavior is stated for a path that arrives non-canonical — it silently inherits the verdict of a spelling the predicate did not intend (checklist 3, 7). Test strategy #7 compounds it: the clause \"a path spelled with a foreign separator or a case variant arrives normalized and repo-root-relative, so it gets the verdict of its canonical spelling\" asserts a property of the ingestion plumbing, while #7 is a unit test of the predicate, which cannot observe what it is handed.","fix":"Add one verified assumption row naming the path-intake site that feeds `runner.ts:279` and `moves.ts` — paths reach the predicate repo-root-relative and POSIX-normalized, with absolute and `..` forms denied there — cite that row in place of \"(A8/A9)\" in the Ownership sentence, and move the separator/case-variant/`..` assertion into the integration test that drives the real sweep (#18), leaving #7 to assert the predicate's verdict on the canonical spelling."}]}
```
