# Transcript: code-review:correctness r1c0

## Provenance
- stage: code-review:correctness (round r1c0)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-10T15:06:33.271Z
- ended: 2026-11-10T15:18:18.018Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/004-mirror-src-hierarchy-in-tests/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff d5092f39755c4a0903dfd19207e4951dfc1cd33a...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/004-mirror-src-hierarchy-in-tests.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/004-mirror-src-hierarchy-in-tests/reviews/code/correctness-r1c0.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict":"approve","findings":[{"severity":"minor","location":"docs/designs/004-mirror-src-hierarchy-in-tests/03-build-notes.md:577-586","problem":"The acceptance evidence the proposal makes load-bearing (Rollout 3 / Risk 4: `npm run check` green, 26 test files / 228 tests, and 23 files / 190 tests for `npm test -- tests/pipeline`) is recorded as prose from the test-update stage and cannot be reproduced in any review seat (no shell granted). Every static precondition that could make it red was checked and is clean, but the counts themselves remain unverified at the merge commit.","fix":"Paste the vitest reporter tail verbatim (Test Files 26 passed / Tests 228 passed, and the 23/190 nested-selection run) into 03-build-notes.md at the merge commit, so Rollout 3's acceptance evidence is an artifact rather than a recollection."},{"severity":"nit","location":"tools/orchestration/tests/pipeline/proposal-size.test.ts:10-14; tools/orchestration/tests/pipeline/prompts.test.ts:214-216, 254-256, 265-268, 279-281, 304-306","problem":"Six of the ¶2 depth rewrites exceeded printWidth 100 after gaining `../`, so prettier re-wrapped them: the report.js import became a five-line statement and the five `.opencode/skills` reads became three-line fileURLToPath blocks. The change is forced by the format:check half of the gate and semantically inert (no other line in those files differs), but Test strategy #1 words the allowed set line-per-site and the build notes' post-move enumeration says only 'nothing outside it touched', so a diff review counting changed lines sees more than 62+16+1.","fix":"Add one clause to 03-build-notes.md naming the re-wrapped sites as prettier output of the ¶2 rewrite, so the allowed set reads as 'these statements, formatted however prettier requires'."}]}
```
