# Transcript: design-review:complexity r1c2

## Provenance
- stage: design-review:complexity (round r1c2)
- agent: review/design-complexity
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-13T20:20:29.249Z
- ended: 2026-11-13T20:21:53.707Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":30031,"output":15698,"reasoning":0,"cache.read":296522,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/020-restart-boundary-for-harness-self-changes/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 15ba04f5008f2e18b9958fbe24a7727b9f01c441 0b1355604ef1e9dec4bbabc9f7ddd54dd27be710` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/020-restart-boundary-for-harness-self-changes.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/020-restart-boundary-for-harness-self-changes/reviews/design/complexity-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "minor",
    "location": "Chosen design §1 — \"Detection is the window's **net** diff. `finalizeStage` and `finalizeReviewPass` (runner.ts:1848–1892, 1940–1961)\"",
    "problem": "The second hook is unreachable under the design's own A3, which §1 restates: \"only `implement`/`fix` can commit under the two prefixes, so the boundary opens only after a coder window\". finalizeReviewPass runs only for pass-level review stages (runner.ts:2184–2187) and for the single-stage pre-commit refusal (runner.ts:1924), and the only preCommit in the tree is the plan stage's (runner.ts:2816) — all document stages, which stagePathAllowed confines to docs/** (gates.ts:588–589); on the refusal path commit:false means no commit is minted, so the headBefore..HEAD range is empty. So the hook can never fire, and Test strategy #3's \"a `finalizeStage` and a `finalizeReviewPass` window whose net diff touches a template or a src file set the record\" can only be satisfied by fabricating a state the ownership fence forbids.",
    "fix": "Instrument finalizeStage alone and say so in §1 in the sentence that already concludes the coder-window rule; drop the review-pass half of Test #3. If the hook is kept deliberately as defence against a future ownership-matrix change, add one clause in §1 saying that, and mark the Test #3 review-pass case a rule-bypass fixture rather than a live window. Same red-on-drift property either way; saves one unreachable call site and one impossible test case."
  },
  {
    "severity": "minor",
    "location": "Test strategy #7 — \"walking the import graph from the pipeline entry points (`src/pipeline/cli.ts`, `src/cli.ts`, `src/opencode-runner.ts`)\"",
    "problem": "The import-graph walk is the largest new mechanism in the design and tools/orchestration has no dependency providing one (devDeps: typescript, vitest, eslint, prettier, tsx), so it means hand-written specifier parsing plus an entry-root resolver — and it still misses any module none of the three roots imports. The rule it guards is directory-scoped, so the walk adds nothing over a flat check: every module under tools/orchestration/src/ is already surface by §1, so the only drift that matters is a module file living outside src/, or a prompt read root outside prompts/.",
    "fix": "Replace the walk with a package-wide enumeration: every tracked .ts/.js under tools/orchestration/ that is not under tests/ must sit under src/, plus the PROMPTS_DIR assertion §7 already makes (cli.ts:20, the same spelling the provenance line uses at runner.ts:1344). About fifteen lines with the fs/git helpers the pipeline tests already use, no new dependency, identical \"new executable location turns the suite red\" property."
  },
  {
    "severity": "nit",
    "location": "Chosen design §2 — \"mints a terminal `chore: <nnn>-<slug> restart boundary` commit\"",
    "problem": "boundary is already a loaded term in the file this lands in: the lock records stage 'boundary' (runner.ts:606), and \"boundary commits\", \"the boundary re-assertion\" and \"at the next boundary\" all name the ADR-006 stage boundary (runner.ts:328, 571, 2376). Bare \"boundary\" in new comments, status text or identifiers will read as the wrong concept.",
    "fix": "Add one clause in §2 (or the §1 heading): \"'restart boundary' is ADR-020's pause; the runner's existing 'boundary' is the ADR-006 stage boundary\", and keep every new identifier and label qualified — restartRequired, finalizeRestartBoundary, restart-boundary — never a bare boundary."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/020-restart-boundary-for-harness-self-changes.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/020-restart-boundary-for-harness-self-changes/reviews/design/complexity-r1c2.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design §1 — \"`finalizeStage` and `finalizeReviewPass` (runner.ts:1848–1892, 1940–1961) already receive the window's `headBefore` and mint its commit\"","problem":"[re-raised] The r2 edit rewrote this sentence (added `-z --no-renames`, named `nulPaths`) and left both hooks, so the r1 problem survives verbatim: finalizeReviewPass is called only at pass level (runner.ts:2184, 2187) and on the single-stage pre-commit refusal with commit:false (runner.ts:1924); the only preCommit in the tree is the plan stage's (runner.ts:2816); stagePathAllowed confines every non-implement/fix stage to docs/** (gates.ts:559–589). Under §1's own rule that only implement/fix can commit under the two prefixes, the second hook can never fire, and Test strategy #3's \"a `finalizeStage` and a `finalizeReviewPass` window whose net diff touches a template or a src file set the record\" is only satisfiable by fabricating a state the ownership fence forbids.","fix":"Instrument finalizeStage alone and say so in the sentence that already states the coder-window rule; drop the review-pass half of Test #3. If the hook is kept deliberately against a future ownership-matrix change, add one clause saying so and mark the Test #3 review-pass case a rule-bypass fixture rather than a live window. Same red-on-drift property either way; one unreachable call site and one impossible test case cheaper."},{"severity":"minor","location":"Test strategy #7 — \"walking the import graph from the pipeline entry points (`src/pipeline/cli.ts`, `src/cli.ts`, `src/opencode-runner.ts`), every in-repo module reached is under `tools/orchestration/src/`\"","problem":"[re-raised] r2 deleted §1's description of this mechanism (\"The ADR's \\\"test that fails when a new executable path is added\\\" is Test strategy #7.\") but left the mechanism, which is still the largest new thing in the design and still has no provider: tools/orchestration/package.json devDeps are typescript, vitest, eslint, prettier, tsx and two plugins — nothing that resolves an import graph — so it means hand-written specifier parsing plus an entry-root resolver, and it still misses any module none of the three roots imports. The rule it guards is directory-scoped by §1, so the walk adds nothing over a flat check.","fix":"Replace the walk with a package-wide enumeration: every tracked .ts/.js under tools/orchestration/ that is not under tests/ must sit under src/, plus the PROMPTS_DIR root assertion the test already names (cli.ts:20). The readdirSync-based helpers the tests/pipeline/ suites already use cover it: no new dependency, ~15 lines instead of a resolver, identical red-on-drift property."},{"severity":"minor","location":"Chosen design §2 — \"the record keeps the `boundedGateEvidence` bounding (first N paths plus the count)\"","problem":"[new] The r2 clause imports a bound the design's own record shape cannot hold. boundedGateEvidence (runner.ts:3057) is typed GateEvidence -> GateEvidence, and its first-N-plus-count works only because an elided leg is a whole extra leg object with a command/tail to carry the count; restartRequired?: { stage, paths, at } has no slot for a count, and the only bound named in the tree is MAX_GATE_LEGS (gates.ts:310), a gate-leg cap, not a path cap. As written the implementer either adds a fourth checkpoint field for the count or drops it silently — and dropping it makes §3's status line, \"how many harness files it changed\", print the capped length rather than the number of changed files.","fix":"Bound where it renders, not what is recorded: keep paths whole and state that the gate-log line, commit body and summary are brief-bounded (brief, independence.ts:448, beside the printable the sentence already names), with paths.length as the count §3 prints. If the record itself is meant to be capped, name the constant and add total to the §2 shape so §3's count stays honest."},{"severity":"nit","location":"Chosen design §2 — \"mints a terminal `chore: <nnn>-<slug> restart boundary` commit\"","problem":"[re-raised] Unaddressed by r2 and not recorded as rejected in the Revision log. boundary is already loaded in this file's vocabulary: the lock records a stage named boundary (runner.ts:606), and \"boundary commits\", \"the boundary re-assertion\" and \"at the next boundary\" name the ADR-006 stage boundary (runner.ts:328, 571, 2376). §1's loop-top check and §2's \"gate-logs the boundary it is crossing\" lean on that second meaning in the same breath as ADR-020's pause.","fix":"One clause in §1's heading or §2 — \"'restart boundary' is ADR-020's pause; the runner's existing 'boundary' is the ADR-006 stage boundary\" — and keep every new identifier and label qualified (restartRequired, finalizeRestartBoundary, restart-boundary), never a bare boundary."}]}
```
