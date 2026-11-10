# Transcript: design-review:correctness r1c3

## Provenance
- stage: design-review:correctness (round r1c3)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T13:56:15.368Z
- ended: 2026-11-10T14:00:07.052Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/004-mirror-src-hierarchy-in-tests/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff bedc93808c6f8655646ebcb5b2dc527ef41a4c50 1d1e68a6726648633a3a2139ed1616eedda39a07` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/004-mirror-src-hierarchy-in-tests.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "blocker",
    "location": "Chosen design ¶2 (with Assumption register A8, Test strategy #1, Rollout 2) — \"no moved file holds a relative specifier outside those 65, and the only other `../src/` sites are the `vi.mock` and type-import in `cli.test.ts:29-30`, which stays at the package-root tier\"",
    "problem": "new — The newly added completeness claim is false, and it hides the one site class the move actually breaks. grep `\\.\\./ ` over tools/orchestration/tests/ finds 87 matches, not 67: the 65 from-imports (62 of them in moved files) + 2 cli.test.ts non-import `../src/` sites + 16 `new URL('<rel>', import.meta.url)` specifiers in 12 moved files — `'../prompts'`/`'../prompts/'` at pipeline-fix-trigger.test.ts:8, pipeline-prompts.test.ts:21, pipeline-budget.test.ts:85, pipeline-reply-contract.test.ts:83, pipeline-stage-lifecycle.test.ts:108, pipeline-funnel.test.ts:187, pipeline-proposal-size.test.ts:191, pipeline-parallel-integration.test.ts:191, pipeline-pass-finalize.test.ts:192, pipeline-parallel-fanout.test.ts:199, pipeline-parallel-escalation.test.ts:203 — plus `'../../../.opencode/skills/…'` at pipeline-prompts.test.ts:213,251,261,274,297 — + 4 inert `'../escape'` fixture strings (pipeline-identity.test.ts:65,68,103; pipeline-state-store.test.ts:156). new URL resolves against the moved… [elided by the checkpoint cap]",
    "fix": "One coherent amendment: extend ¶2's rewrite set, Test strategy #1's allowed diff set, and Rollout 2's move-commit contents to the 16 depth-sensitive `new URL(..., import.meta.url)` sites (`'../prompts'` → `'../../prompts'`, `'../../../.opencode/...'` → `'../../../../.opencode/...'`), recite them in the bullet-2 amendment alongside the comment clause (replacing the current comment-only clause with e.g. \"a relative path reference — code or comment — naming a location this move passes takes its new depth\"), and replace ¶2's clause and A8's basis with the checked enumeration: grep `\\.\\./` = 87 sites = 65 from-imports (62 in moved files) + 2 cli.test.ts non-import `../src/` sites + 16 `new URL` sites + 4 inert `'../escape'` fixture strings. This replaces the false sentence rather than adding a new check; all five of my r1 findings are otherwise closed for real."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/004-mirror-src-hierarchy-in-tests.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/004-mirror-src-hierarchy-in-tests/reviews/design/correctness-r1c3.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design ¶2 — \"`../prompts/` → `../../prompts/`\"","problem":"new — The mapping as written covers 15 of the 16 URL sites: pipeline-fix-trigger.test.ts:8 uses `new URL('../prompts', import.meta.url)` with no trailing slash, so a rewrite keyed to the quoted pattern leaves one site one tier shallow. Test strategy #1 only asserts the moved-file diff stays inside the allowed set, so it cannot catch a missed site; only the acceptance run (item 5) would.","fix":"Key the rewrite to the call rather than the string literal — \"each `new URL('<rel>', import.meta.url)` in a moved file gains one `../`\" — keeping the two mappings as examples instead of the operative pattern."},{"severity":"minor","location":"Chosen design ¶2 (clause written into ADR bullet 2) — \"a relative path reference — code or comment — naming a location this move passes takes its new depth\"","problem":"new — At the edge this revision closed, the clause reads wide enough to cover A8's fourth class: the 4 `'../escape'` strings are relative path references in code (verified inert: pipeline-state-store.test.ts:156 writes them into a mkdtemp checkpoint; pipeline-identity.test.ts:65,68,103 feed them to sanitizeSlug/deriveIdentity), and Deferred deliberately keeps other in-test path strings stale. The exclusion currently rests only on A8's word \"inert\", so the durable ADR sentence and the proposal's closed enumeration can be read against each other at implementation time.","fix":"Qualify the clause by resolution rather than by label: \"a path reference that the moved file resolves against its own location — code or comment — takes its new location\", which excludes the escape strings and A14's inert fixtures by definition instead of by an appended label."},{"severity":"minor","location":"Chosen design ¶2 — \"takes its new depth\" with \"(the comment site: A14)\"","problem":"new — The comment A14 names is `tests/pipeline-reply-contract.test.ts` at pipeline-parallel-escalation.test.ts:45 (verified verbatim). It carries no `../`, so it is not inside A8's 87-site enumeration, and its edit is a directory insertion plus prefix drop, not a depth change. r1's clause (\"takes its new path\") sanctioned that edit; the replacement clause's \"new depth\" does not describe it, so the comment line is covered only by the parenthetical while the ADR sentence is what a reviewer of the commit checks.","fix":"One word in the clause — \"takes its new location\" — and in ¶2 name the actual edit (\"the comment's path gains the `pipeline/` tier and drops the prefix: A14\"), replacing the current verb rather than adding a new caveat."},{"severity":"minor","location":"Goals / Non-goals — \"no change there to test logic, test names, fixture inputs, or file splits\"","problem":"new — This restates Decision bullet 2, which says \"No test logic, test names, fixtures, or file splits\" (ADR:42-44). Rollout 2 adds exactly one clause to bullet 2, so the narrowing of \"fixtures\" to \"fixture inputs\" exists only in the proposal and is not recorded in the Revision log — while the move commit now rewrites 16 path constants that feed readFile/loadTemplate.","fix":"Quote bullet 2's own word (\"fixtures\") and state that ¶2's clause is what licenses those path-constant changes, or fold the word change into the Revision log's ¶2 entry; either replaces the silent paraphrase rather than adding a further qualification."},{"severity":"nit","location":"Assumption register A8 — \"all 87 `../` occurrences in the 26 files\"","problem":"new — The cited grep counts 87 matching lines; the 5 `../../../.opencode/…` lines each contain three `../`, so literal occurrences are 97. The row's arithmetic (65+2+16+4=87) only closes at line granularity.","fix":"Write \"all 87 `../`-bearing lines\" so the claim matches the cited grep verbatim."}]}
```
