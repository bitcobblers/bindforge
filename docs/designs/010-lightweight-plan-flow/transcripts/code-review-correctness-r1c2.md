# Transcript: code-review:correctness r1c2

## Provenance
- stage: code-review:correctness (round r1c2)
- agent: codereview/code-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T01:14:48.467Z
- ended: 2026-11-13T01:18:54.990Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 96f50c2110560362e5fb61f05bd313f04bf503cb..HEAD -- ':(exclude)docs/designs', read against the full current document.
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
    "severity": "blocker",
    "location": "tools/orchestration/src/pipeline/states.ts:191,197; tools/orchestration/src/pipeline/runner.ts:2560,2675-2679,2711-2715; 03-build-notes.md:115-124",
    "problem": "Deviation 10's declared remedy for the repurposed codeBudgetUsed counter is incomplete. The light flow's fix loop entered from a red testsGreen gate is bound by budgetLeft(state, state.codeBudgetUsed, cfg) (states.ts:191,197), not by the codeRound check deviation 10 names, and this diff repurposed that counter: the per-fix charge is skipped for light (runner.ts:2711-2715), the round-entry charge is skipped (runner.ts:2560), and codeBudgetUsed is now the seats a code pass launched (runner.ts:2677-2679). Consequence 1: a light implement with red tests and no code review yet has codeBudgetUsed 0 forever, so decideNext returns stage('fix') on every iteration with no cap anywhere (the pass-entry session cap gates review passes only, codeRound never moves, stageFailures counts failures, and an ok fix is not one) — unbounded fix sessions at 12 min each, with no escalation, so the design's second deterministic stop is disabled. Consequence 2: after the first 2-seat code pass codeBudgetUsed is … [elided by the checkpoint cap]",
    "fix": "Keep the budget test in the unit the design states it in: restore state.codeBudgetUsed += 1 for light at runner.ts:2714 and 2560, drop the seats charge at runner.ts:2677-2679, and print the session count from reviewSessionsUsed(state, 'code') (which the status/report lines already do). Pin both shapes: a light run whose tests stay red from implement escalates with the round-limit record instead of looping, and a light dissent gets its second fix round. Update deviation 10 to name which check binds on which path."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:2384-2389,2763; tools/orchestration/src/pipeline/report.ts:54-62",
    "problem": "The plan stop's escalation reports the design phase with an empty stage name. state.lastStage = name is assigned after runWithRetry returns, and the pre-commit refusal throws StopDriving from inside it, so at finalizeEscalate lastStage is still '' and escalationPhase returns 'design'. The plan stop's ESCALATION.md opens \"Run stopped after stage `` in the **design** phase\" and reportFindings reads the design phase's round — exactly the mislabel A15 and the Reporting section (02-proposal.md:306-310) exist to remove, and Test strategy #10's plan-is-code pin passes only because escalation.test.ts:99-105,161 constructs lastStage: 'plan' by hand. light-plan-size.test.ts:176-197 asserts the size string and the promotion line, never the phase line, so the gap is unpinned.",
    "fix": "Assign the stage before the gate can stop the run (state.lastStage = name in drive before the pre-commit gate, or in executeStage's refusal path at runner.ts:1879-1888), and extend the light-plan-size.test.ts record assertion to ESCALATION.md containing 'in the **code** phase'."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/identity.ts:67,93-115; 03-build-notes.md:70-83",
    "problem": "Scope creep in a shared entry path: the proposal states 'nextNnn and identityClash need no change' (02-proposal.md:322-324, A17), and the diff adds nnnOfExistingRun — for every non-ADR source, deriveIdentity now reads every docs/designs/*/.pipeline.json, matches source.path and pins that NNN. It is not flow-gated, so it changes the full flow's entry behaviour for every brief that has a run: a plain re-run is now a clash needing --force/--slug/--nnn (deviation 4 calls this the accepted side effect). That contradicts the design's compatibility claim that every existing document and checkpoint behaves exactly as it does today (02-proposal.md:13).",
    "fix": "Gate the checkpoint-identity rule on the flow that needs it: resolve the source's flow: key before deriveIdentity (the parse already exists at runner.ts:2176-2181) and apply nnnOfExistingRun only to a light source, which is the promotion path Test strategy #22 pins, leaving the full flow's allocation exactly as A17 states. If the general rule is wanted, land it as its own change with its own design entry."
  },
  {
    "severity": "major",
    "location": "03-build-notes.md:12-17,292-303; 02-proposal.md:441-443 (#21)",
    "problem": "The change lands with the package's own gate red: the notes state that npm run check stops at format:check (gates.test.ts:706-708, runner.test.ts:390 committed unformatted) so the && chain never reaches vitest, while the headline claims 'The suite is green — 41 files, 473 passed / 2 skipped'. Test strategy #21 pins that the suite runs at the engines Node floor and npm run check stays green; the note's own fix is one command.",
    "fix": "Land the formatting (npx prettier --write tests/pipeline/gates.test.ts tests/pipeline/runner.test.ts — the test owner's stage owns those paths) and restate the headline as 'npm run check green' once it is, so the note's opening claim and the gate agree."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/stages.ts:187-192,203-210; tools/orchestration/src/pipeline/flows.ts:78",
    "problem": "The flow registry's own plan stage is not forceable. FlowDef.stages is documented as the --stage gate and assertStageInFlow implements it, but assertForceableStage runs first against STAGE_NAMES, which deliberately excludes plan via LIGHT_ONLY_STAGES. So 'pipeline run <light-doc> --stage plan' and 'pipeline resume <light-slug> --stage plan' are usage errors ('unknown --stage plan') even though plan is the light flow's first stage: an operator cannot re-plan a run the plan stop ended.",
    "fix": "Accept a stage the flow's registry declares (admit names in FLOWS[flow].stages in the --stage validation, keeping STAGE_NAMES as the full-flow regression guard), and pin assertStageInFlow('light','plan') plus the CLI-level acceptance."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:214-224,2751; 03-build-notes.md:187-197",
    "problem": "The plan stop deletes the plan the operator is told to promote from. The design pins the stop at the stage-ok/pre-commit point so the oversized plan is never committed; deviation 17 goes further and removes the document from the worktree (git restore from HEAD, else rmSync). The stop's own record reads ': set flow: full and re-run with --force' and A26 sells promotion on the plan surviving in branch history, so with this rule the refused plan exists nowhere — not committed, not in the worktree. The proposal never asks the runner to delete a stage's in-ownership product: docs/** is plan's allowed surface and the ownership sweep is the design's only revert mechanism.",
    "fix": "Keep the pre-commit posture (no commit) and leave the bytes unstaged in the worktree so the operator can read and promote from them (the refusal path already runs the sweep at runner.ts:1886). If the deletion stays, name it in the escalation string and in the Risks/Deferred row and pin it."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:2567-2580; 03-build-notes.md:57-69",
    "problem": "Deviation 3's stamp rule ('a ratify pass keeps the stamp its round entry set') is shared by both flows and is the mechanism deviation 11 credits with fixing run 010's near-miss, and the note asserts 'Full-flow code-phase tests are unaffected'. No full-flow test exercises the ratify stamp — codeStamp appears in tests only in fixtures and hand-built panel/states states, so the claim about the full flow is unpinned.",
    "fix": "Add one full-flow pass sequence (full → dissent → fix → ratify) asserting the ratify pass does not move state.codeStamp, so the shared-path rule deviation 3 declares has a regression guard."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/correctness-r1c2.md, and write no other file: an audit checks
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"tools/orchestration/src/pipeline/runner.ts:1868-1877, 2731-2760; tools/orchestration/src/pipeline/states.ts:118-124; tools/orchestration/src/pipeline/report.ts:301-307","problem":"The plan stop is bypassed by the repair command its own record prints, and the round-1 fix is what moved the landing point. The fix to finding 2 assigns state.lastStage = stageName in the pre-commit refusal, so a light run stopped by the size gate checkpoints lastStage: 'plan' with codeBudgetUsed 0 and codeRound 0. ESCALATION.md ends with 'Fix the underlying cause (usually the brief or the proposal), then run: `pipeline resume 002-x`', and that resume takes decideNext's case 'plan' -> stage('implement'): the 120-line plan the flow refused — now tracked in HEAD by the terminal escalation commit, the posture finding 6 asked for — becomes the document implement builds against and the two seats check faithfulness against, with the 12-minute implement session spent and no cap downstream. Before the fix lastStage was '' at that escalation, so a resume re-planned; the changed code makes the bypass real. The record therefore prints two instructions with opposite outcomes (the gate line says promote with flow: full and re-run --force, the footer says resume), and the mitigation the Risks row names for an over-scoped light run — the 80-line plan gate — is the thing this path drops.","fix":"Make the stop sticky and the record single-voiced: record the refusal on the state as an additive optional field (ADR-006 §2 style), have case 'plan' escalate on the promotion record while that marker stands and clear it when a plan pass clears planOutgrewFlow, so a plain resume re-plans; and render the promotion recipe as that record's repair line in escalationMarkdown (it has the gate reason) instead of the generic `pipeline resume`. Pin the shape: a light run resumed after a plan stop never spends an implement session on the refused plan."},{"severity":"nit","location":"tools/orchestration/tests/pipeline/gates.test.ts:706-708; tools/orchestration/tests/pipeline/runner.test.ts:385-398; tools/orchestration/package.json:23","problem":"Round-1 finding 4 (the package's own gate red at format:check) cannot be confirmed from this review's inputs: the cited spots now read prettier-canonical at the repo's printWidth 100 — gates.test.ts:706-708 is exactly the wrap prettier emits for a 105-column call, runner.test.ts:395-397 likewise — so there is no evidence of an open defect, but this environment grants no shell, so `prettier --check .` and the vitest run behind `npm run check`'s && chain were never executed, and Test strategy #21's 'npm run check stays green' is unverified rather than passed.","fix":"One run of `npm run check` in the package (the test owner's paths) confirming the chain reaches vitest, and restate the notes' headline as 'npm run check green' from that run's output."}]}
```
