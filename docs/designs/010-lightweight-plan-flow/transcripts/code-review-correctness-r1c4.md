# Transcript: code-review:correctness r1c4

## Provenance
- stage: code-review:correctness (round r1c4)
- agent: codereview/code-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T02:13:44.069Z
- ended: 2026-11-13T02:17:42.475Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c4: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 8bad668511aa451580ae5bfee1e9be9e5f5195d6..HEAD -- ':(exclude)docs/designs', read against the full current document.
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
    "location": "tools/orchestration/src/pipeline/report.ts:247-249 (hasPlanStop), call sites report.ts:271, report.ts:382, runner.ts:3294-3301; producers runner.ts:2818, 2839, 2458-2465, 2492-2500, 2789",
    "problem": "The round-1 fix records the stop on the state (`PipelineState.planStop`) and makes `decideNext`'s `case 'plan'` escalate on it, which closes the bypass — but the predicate that selects the plan-stop record keys on the stage, not the stop: `hasPlanStop = flowOf(state) === 'light' && state.lastStage === 'plan'`, ignoring `planStop` whose own doc (`types.ts:247-248`) says \"its absence is 'no stop standing'\". Two reachable light shapes then read as a plan stop. (a) A light `implement` stage that fails: `state.lastStage = name` runs only after `runWithRetry` returns (runner.ts:2810-2818), and every stage-failure escalation inside it (reply fails twice at 2492-2500, half-applied moves at 2457-2465) escalates and throws `StopDriving`, so `lastStage` stays `'plan'` — the value the *successful* plan stage persisted at 2839 — and `planStop` was cleared at 2789. ESCALATION.md then prints \"The plan stop is sticky: resuming this run as it stands re-escalates on this record, so the refused plan is n… [elided by the checkpoint cap]",
    "fix": "Key the predicate on the stop: `return flowOf(state) === 'light' && state.planStop !== undefined;`. For the real plan stop the two printed moves are unchanged (the gate sets the field before the escalation's checkpoint save), the `state.planStop ?? '…'` fallback at statusLines:3296 loses its dead branch, and the implement-failure record and the running status fall back to the plain-resue shape the state machine answers. Update the states the report tests hand it — escalation.test.ts:171-176 builds `{ lastStage: 'plan', flow: 'light' }` with no `planStop`, a shape the gate never writes — to carry `planStop: planReason`, and add two pins: a light escalation with `lastStage: 'plan'` and no `planStop` prints the plain `pipeline resume <slug>` and no plan-stop claim; a running light state at `lastStage: 'plan'` with no `planStop` prints no `plan stop:` line."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/package.json:23; tools/orchestration/tests/pipeline/gates.test.ts:706-708; tools/orchestration/tests/pipeline/runner.test.ts:395-397",
    "problem": "Unchanged in kind from round 1: this pass granted no shell, so `prettier --check .` and the vitest run behind `npm run check`'s `&&` chain were never executed, and Test strategy #21's \"`npm run check` stays green\" is unverified rather than passed — a checklist item this seat could not evaluate. The static reading shows no open defect: the two cited spots read prettier-canonical at the repo's printWidth 100, and every line over 100 columns in the reviewed src/pipeline and tests/pipeline files is an unbreakable string/template literal (report.ts:219, runner.ts:3164, flows.ts:70, runner.test.ts:363).",
    "fix": "One run of `npm run check` in tools/orchestration (the test owner's path) confirming the chain reaches vitest and exits 0, and restate the build-notes headline as \"npm run check green\" from that run's output."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/correctness-r1c4.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"tools/orchestration/src/pipeline/runner.ts:2740","problem":"The converged-review gate-log reason is hard-coded 'all five seats approve the current revision stamp'. This call site is one the diff threaded (planPass(state, phase, cfg) at 2734), and the light flow's panel is ['correctness', 'testing'], so every light run that converges records — in the checkpoint's gate log, which `pipeline status` prints under `recent gates:` (runner.ts:3321-3324) — that five seats approved when the flow launched two. It is the same class of record inaccuracy the proposal's Reporting section exists to remove (the panel is declarable data and the report must print 'exactly the flow's seats'); the neighbouring lines are already flow-neutral (runner.ts:2636 'all seats already approve the current revision') and the pre-seat fallback reads the flow's panel (2911), so 2740 is the straggler. No state, budget, ownership, or control-flow effect; reachable on the happy path of every light run (light-flow.test.ts:293-317 lands this line).","fix":"Drop the literal count: `all ${String(plan.seats.length)} seats approve the current revision stamp` (the panel planPass/seatsFor resolved), or reuse the flow-neutral wording at runner.ts:2636. Pin it on a light converged run — assert the code-review converged gate-log reason names two seats, not five — so a third flow cannot re-hard-code a number."},{"severity":"nit","location":"tools/orchestration/package.json:23; tools/orchestration/tests/pipeline/gates.test.ts:706-708; tools/orchestration/tests/pipeline/runner.test.ts:395-397","problem":"Unchanged from round 1: no shell was granted this pass, so `prettier --check .` and the vitest run behind `npm run check`'s `&&` chain were never executed by this seat, and Test strategy #21's \"`npm run check` stays green\" remains a claim I could not evaluate — which the checklist counts as a finding, not a pass. The static reading is clean: the cited spots read prettier-canonical at the repo's printWidth 100, and every line over 100 columns in the reviewed src/pipeline and tests/pipeline files is an unbreakable string or template literal.","fix":"One run of `npm run check` in tools/orchestration (the test owner's path) confirming the chain reaches vitest and exits 0, and restate the build-notes headline as \"npm run check green\" from that run's output."}]}
```
