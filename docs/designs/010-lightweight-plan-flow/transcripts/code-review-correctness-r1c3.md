# Transcript: code-review:correctness r1c3

## Provenance
- stage: code-review:correctness (round r1c3)
- agent: codereview/code-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T02:03:46.617Z
- ended: 2026-11-13T02:06:41.954Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 945143568beb315686c133f53f30ff7b5ef02d78..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/010-lightweight-plan-flow.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:1868-1877, 2731-2760; tools/orchestration/src/pipeline/states.ts:118-124; tools/orchestration/src/pipeline/report.ts:301-307",
    "problem": "The plan stop is bypassed by the repair command its own record prints, and the round-1 fix is what moved the landing point. The fix to finding 2 assigns state.lastStage = stageName in the pre-commit refusal, so a light run stopped by the size gate checkpoints lastStage: 'plan' with codeBudgetUsed 0 and codeRound 0. ESCALATION.md ends with 'Fix the underlying cause (usually the brief or the proposal), then run: `pipeline resume 002-x`', and that resume takes decideNext's case 'plan' -> stage('implement'): the 120-line plan the flow refused — now tracked in HEAD by the terminal escalation commit, the posture finding 6 asked for — becomes the document implement builds against and the two seats check faithfulness against, with the 12-minute implement session spent and no cap downstream. Before the fix lastStage was '' at that escalation, so a resume re-planned; the changed code makes the bypass real. The record therefore prints two instructions with opposite outcomes (the gate line says pr… [elided by the checkpoint cap]",
    "fix": "Make the stop sticky and the record single-voiced: record the refusal on the state as an additive optional field (ADR-006 §2 style), have case 'plan' escalate on the promotion record while that marker stands and clear it when a plan pass clears planOutgrewFlow, so a plain resume re-plans; and render the promotion recipe as that record's repair line in escalationMarkdown (it has the gate reason) instead of the generic `pipeline resume`. Pin the shape: a light run resumed after a plan stop never spends an implement session on the refused plan."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/tests/pipeline/gates.test.ts:706-708; tools/orchestration/tests/pipeline/runner.test.ts:385-398; tools/orchestration/package.json:23",
    "problem": "Round-1 finding 4 (the package's own gate red at format:check) cannot be confirmed from this review's inputs: the cited spots now read prettier-canonical at the repo's printWidth 100 — gates.test.ts:706-708 is exactly the wrap prettier emits for a 105-column call, runner.test.ts:395-397 likewise — so there is no evidence of an open defect, but this environment grants no shell, so `prettier --check .` and the vitest run behind `npm run check`'s && chain were never executed, and Test strategy #21's 'npm run check stays green' is unverified rather than passed.",
    "fix": "One run of `npm run check` in the package (the test owner's paths) confirming the chain reaches vitest, and restate the notes' headline as 'npm run check green' from that run's output."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/correctness-r1c3.md, and write no other file: an audit checks
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"tools/orchestration/src/pipeline/report.ts:247-249 (hasPlanStop), call sites report.ts:271, report.ts:382, runner.ts:3294-3301; producers runner.ts:2818, 2839, 2458-2465, 2492-2500, 2789","problem":"The round-1 fix records the stop on the state (`PipelineState.planStop`) and makes `decideNext`'s `case 'plan'` escalate on it, which closes the bypass — but the predicate that selects the plan-stop record keys on the stage, not the stop: `hasPlanStop = flowOf(state) === 'light' && state.lastStage === 'plan'`, ignoring `planStop` whose own doc (`types.ts:247-248`) says \"its absence is 'no stop standing'\". Two reachable light shapes then read as a plan stop. (a) A light `implement` stage that fails: `state.lastStage = name` runs only after `runWithRetry` returns (runner.ts:2810-2818), and every stage-failure escalation inside it (reply fails twice at 2492-2500, half-applied moves at 2457-2465) escalates and throws `StopDriving`, so `lastStage` stays `'plan'` — the value the *successful* plan stage persisted at 2839 — and `planStop` was cleared at 2789. ESCALATION.md then prints \"The plan stop is sticky: resuming this run as it stands re-escalates on this record, so the refused plan is never built from\" and offers only `pipeline run <source> --force` and `pipeline resume --stage plan <id>`, while the repair the state machine honors — plain `pipeline resume <slug>`, which falls through `states.ts:136-139` to `stage('implement')` — is the one it omits. (b) A healthy running light run: after a plan that clears the cap the checkpoint holds `lastStage: 'plan'`, no `planStop`, `status: 'running'`, and `pipeline status` prints `plan stop: the plan gate refused this plan — a plain resume re-escalates; the moves that respect it: …` for the whole 12-minute implement window (runner.ts:3294-3301). Both are new in this revision; the design's plan-stop behavior itself (#19) is met.","fix":"Key the predicate on the stop: `return flowOf(state) === 'light' && state.planStop !== undefined;`. For the real plan stop the two printed moves are unchanged (the gate sets the field before the escalation's checkpoint save), the `state.planStop ?? '…'` fallback at statusLines:3296 loses its dead branch, and the implement-failure record and the running status fall back to the plain-resue shape the state machine answers. Update the states the report tests hand it — escalation.test.ts:171-176 builds `{ lastStage: 'plan', flow: 'light' }` with no `planStop`, a shape the gate never writes — to carry `planStop: planReason`, and add two pins: a light escalation with `lastStage: 'plan'` and no `planStop` prints the plain `pipeline resume <slug>` and no plan-stop claim; a running light state at `lastStage: 'plan'` with no `planStop` prints no `plan stop:` line."},{"severity":"nit","location":"tools/orchestration/package.json:23; tools/orchestration/tests/pipeline/gates.test.ts:706-708; tools/orchestration/tests/pipeline/runner.test.ts:395-397","problem":"Unchanged in kind from round 1: this pass granted no shell, so `prettier --check .` and the vitest run behind `npm run check`'s `&&` chain were never executed, and Test strategy #21's \"`npm run check` stays green\" is unverified rather than passed — a checklist item this seat could not evaluate. The static reading shows no open defect: the two cited spots read prettier-canonical at the repo's printWidth 100, and every line over 100 columns in the reviewed src/pipeline and tests/pipeline files is an unbreakable string/template literal (report.ts:219, runner.ts:3164, flows.ts:70, runner.test.ts:363).","fix":"One run of `npm run check` in tools/orchestration (the test owner's path) confirming the chain reaches vitest and exits 0, and restate the build-notes headline as \"npm run check green\" from that run's output."}]}
```
