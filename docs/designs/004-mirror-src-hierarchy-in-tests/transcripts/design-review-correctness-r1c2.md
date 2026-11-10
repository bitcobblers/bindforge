# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T13:48:26.196Z
- ended: 2026-11-10T13:53:42.372Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/004-mirror-src-hierarchy-in-tests/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 599636095daad2a723b1072ec8906ee3d57a3c84 bedc93808c6f8655646ebcb5b2dc527ef41a4c50` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "severity": "major",
    "location": "Test strategy (intro + item 1) vs Rollout 1 — \"the move is proven by the existing suite being unchanged, plus the one added assertion block in item 4\" / \"every previously existing test still exists and passes unmodified — assertion counts per file are identical before and after the move, and the moved files' diff contains only ¶2's allowed set\"",
    "problem": "Rollout 1 now lands a test-only commit before the move that adds assertions to the gate suite, tests/pipeline-gates.test.ts (existing describe('stagePathAllowed') block at pipeline-gates.test.ts:294-313) — one of the 23 files the move relocates. Against the branch's pre-work state, item 1's three invariants are all false for that file: the suite is not unchanged (the intro admits it), its per-file assertion count differs, and its diff across the two commits contains an assertion block that is neither an import line nor the comment line. Risks row 4 (\"compares the vitest reporter's file/test counts to a pre-move baseline\") inherits the same undefined baseline, and item 2's \"with the pre-move test count\" fails outright if the added block is a new case rather than extra expects. The baseline is never defined, so the acceptance evidence for Rollout 3 cannot be produced as written.",
    "fix": "Name the baseline once: \"before the move\" = the move commit's parent (i.e. after the Rollout-1 test-only commit), and record the single expected delta — pipeline-gates.test.ts gains assertions inside the existing stagePathAllowed block while its test count and the 26-file count stay unchanged. Amends Test strategy #1 and Risks row 4's \"pre-move baseline\" rather than adding a new check."
  },
  {
    "severity": "major",
    "location": "Goals/Non-goals bullet 2 + Chosen design ¶2 vs Rollout 1 — \"Inside the move commit: no change to test logic, test names, fixtures, or file splits — 26 test files before, 26 after (Decision, bullet 2)\"",
    "problem": "The revision adopts the strict reading of ADR-004 Decision bullet 2 (\"Bullet 2 permits import depth alone\") to justify amending it for one comment line, then silently narrows the same bullet to \"inside the move commit\" in order to admit a test-logic change — item 4's added assertions — that bullet 2's own text forbids (\"The only content change permitted is the import depth … No test logic, test names, fixtures, or file splits\", ADR:42-44) and that ADR Consequences:78 makes an acceptance gate (\"gate is `npm run check` green with an unchanged test count\"). The comment deviation is declared and tasked to an amendment; this deviation from the same clause is neither, so the document reads bullet 2 two different ways.",
    "fix": "State one reading: add a sentence to Rollout 1 (or Deferred) that bullet 2 constrains the move commit's diff only, that the Rollout-1 commit is a separate test change outside ADR-004's subject, and that it leaves test and file counts unchanged — or fold it into the same one-clause amendment to bullet 2."
  },
  {
    "severity": "major",
    "location": "Test strategy item 2 + Risks row 1 — \"fails closed on a half-applied move: a grep over `tools/orchestration/tests/` finds zero leftover `pipeline-*.test.ts` paths\"",
    "problem": "Read as written (item 6's \"grep over\" means a content grep), the check cannot pass: tools/orchestration/tests/pipeline-independence.test.ts:145 carries the fixture string `tools/orchestration/tests/pipeline-lane.test.ts`, which matches `pipeline-*.test.ts` and which the document deliberately keeps (Deferred: \"Refreshing fixture path strings that mimic the old naming … no follow-up yet\"). Verified: that line and pipeline-parallel-escalation.test.ts:45 are the only two occurrences of that shape in the tests directory. So the new fail-closed check either fails spuriously or pushes the implementer into the fixture edit the Deferred section defers — and Risks row 1's mitigation for the half-mirrored-tree risk rests on it. The deferral itself matches ADR-004 and is not re-opened; the new check contradicts the deferral.",
    "fix": "Make item 2 filename-level: \"a file listing over `tools/orchestration/tests/` (e.g. `git ls-files`) finds zero files named `pipeline-*.test.ts`\", with a note that in-file fixture strings are excluded by Deferred. Same single sentence carries Risks row 1."
  },
  {
    "severity": "minor",
    "location": "Chosen design ¶1 — \"The move is one commit in `tools/orchestration/` with no config change, in four parts\"",
    "problem": "The commit that section tasks is not confined to tools/orchestration/: Rollout 2 also commits the two skill-file amendments (.opencode/skills/repo-practices/SKILL.md:21 and .opencode/skills/test-strategy/SKILL.md:27) and, newly, the ADR-004 Decision bullet 2 amendment in docs/adrs/. The path qualifier contradicts the tasked content.",
    "fix": "Drop the qualifier or name the areas: \"one commit touching tools/orchestration/, the two skill files, and ADR-004 Decision bullet 2; no config change\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design ¶2 — \"never a content-wide `../src/` substring substitution, which would also rewrite the fixture path strings fed to the gate (A14)\"",
    "problem": "The stated hazard does not exist in the repo: the gate's fixture strings are `tools/orchestration/tests/…` and `docs/…` paths (pipeline-gates.test.ts:296-313, pipeline-independence.test.ts:145) and none contains `../src/`. Meanwhile the real hole in a `from '../src/`-keyed rewrite goes unmentioned — non-import relative specifiers, `vi.mock('../src/opencode-runner.js', …)` and `importOriginal<typeof import('../src/opencode-runner.js')>()` at cli.test.ts:29-30, the only two `../src/` sites beyond A8's 65, which that pattern skips. It is harmless today only because cli.test.ts is root-tier and does not move, a fact the design never states.",
    "fix": "Replace the unsupported clause with the checked fact: \"no moved file contains a relative specifier outside A8's 65 — the only such sites are the vi.mock and type-import in cli.test.ts, which stays at the package-root tier\"; optionally add that fact to A8's basis."
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
Write your report to docs/designs/004-mirror-src-hierarchy-in-tests/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"blocker","location":"Chosen design ¶2 (with Assumption register A8, Test strategy #1, Rollout 2) — \"no moved file holds a relative specifier outside those 65, and the only other `../src/` sites are the `vi.mock` and type-import in `cli.test.ts:29-30`, which stays at the package-root tier\"","problem":"new — The newly added completeness claim is false, and it hides the one site class the move actually breaks. grep `\\.\\./ ` over tools/orchestration/tests/ finds 87 matches, not 67: the 65 from-imports (62 of them in moved files) + 2 cli.test.ts non-import `../src/` sites + 16 `new URL('<rel>', import.meta.url)` specifiers in 12 moved files — `'../prompts'`/`'../prompts/'` at pipeline-fix-trigger.test.ts:8, pipeline-prompts.test.ts:21, pipeline-budget.test.ts:85, pipeline-reply-contract.test.ts:83, pipeline-stage-lifecycle.test.ts:108, pipeline-funnel.test.ts:187, pipeline-proposal-size.test.ts:191, pipeline-parallel-integration.test.ts:191, pipeline-pass-finalize.test.ts:192, pipeline-parallel-fanout.test.ts:199, pipeline-parallel-escalation.test.ts:203 — plus `'../../../.opencode/skills/…'` at pipeline-prompts.test.ts:213,251,261,274,297 — + 4 inert `'../escape'` fixture strings (pipeline-identity.test.ts:65,68,103; pipeline-state-store.test.ts:156). new URL resolves against the moved file's own location, so one tier deeper turns `../prompts` into `tests/prompts` (absent) and `../../../.opencode/…` into `tools/.opencode/…` (absent), and these are live disk-read paths: loadTemplate is readFile(join(promptsDir, name)) at prompts.ts:26-27; pipeline-fix-trigger.test.ts:60 reads fix-tests.md through its PROMPTS_DIR; pipeline-prompts.test.ts:212-215 reads the skill files directly; the other nine suites feed that promptsDir into executeStage/resumePipeline, which render stage templates from disk. As written, the design rewrites only A8's 65 imports plus one comment line, and Test strategy #1 bounds the moved-file diff to exactly that allowed set — the ENOENT breakage is neither tasked nor permitted, `npm run check` cannot be green after the move, and ADR Decision bullet 2 (\"the only content change permitted is the import depth\") forbids the repair. The A8 row itself is narrowly true about imports and `../src/`, but its two cited greps structurally cannot see the URL sites, so it cannot support ¶2's claim; the 65 also span all 26 files (62 in moved files), so \"the 65\" is not the in-moved-file rewrite count either.","fix":"One coherent amendment: extend ¶2's rewrite set, Test strategy #1's allowed diff set, and Rollout 2's move-commit contents to the 16 depth-sensitive `new URL(..., import.meta.url)` sites (`'../prompts'` → `'../../prompts'`, `'../../../.opencode/...'` → `'../../../../.opencode/...'`), recite them in the bullet-2 amendment alongside the comment clause (replacing the current comment-only clause with e.g. \"a relative path reference — code or comment — naming a location this move passes takes its new depth\"), and replace ¶2's clause and A8's basis with the checked enumeration: grep `\\.\\./` = 87 sites = 65 from-imports (62 in moved files) + 2 cli.test.ts non-import `../src/` sites + 16 `new URL` sites + 4 inert `'../escape'` fixture strings. This replaces the false sentence rather than adding a new check; all five of my r1 findings are otherwise closed for real."}]}
```
