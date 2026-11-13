# Transcript: code-review:correctness r1c1

## Provenance
- stage: code-review:correctness (round r1c1)
- agent: codereview/code-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T00:23:19.129Z
- ended: 2026-11-13T00:31:56.664Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff eb8b40ad413a397f71b977f6d0dcd6c903f0d17c..HEAD -- ':(exclude)docs/designs', read against the full current document.
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
    "location": "tools/orchestration/src/pipeline/stages.ts:105,108",
    "problem": "The full flow's budgets were changed by this diff: DEFAULT_CONFIG.maxPassesPerRound is 8 (base 6, per 01-research.md:48-50 and the proposal's table at 02-proposal.md:171) and stageTimeoutMs is 90*60_000 (base 30*60_000, proposal 02-proposal.md:172). The proposal's Non-goals (02-proposal.md:36) forbid changing the full flow's budgets; 03-build-notes.md lists eight deviations and never mentions these knobs (checklist item 3, undeclared deviation); docs/workflows.md:235,251, landed in this same diff, still document 30 min, so the change ships a doc/code contradiction; no test pins the values (flows.test.ts:122,145 assert literals carrying 6).",
    "fix": "Restore maxPassesPerRound: 6 and stageTimeoutMs: 30 * 60_000 — FLOWS.light.config already sets 3 and 12*60_000, so the light flow needs neither. If the 8-pass/90-min values are wanted, land them as their own change: declare them in 03-build-notes.md, reconcile docs/workflows.md:235,251, and add a test asserting the full flow's resolved numbers."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/panel.ts:129-135; tools/orchestration/src/pipeline/runner.ts:2506-2514",
    "problem": "The funnel was modified for the light flow: planPass lifts the pass cap by one while an unspent --extend-budget grant is held, and the pass-entry cap check no longer decrements that grant for light. The proposal states the funnel runs unmodified on the light seat list (02-proposal.md:244-245) and that the grant is consumed at the cap checks (02-proposal.md:162-163, A27). Undeclared, and untested. Effect: one grant lifts the pass cap in every later pass and round of the run, the operatorExtension gate log prints a grant count the state never spends, and planPass's blocked reason prints 'cap 3 passes per round' after 4 passes ran.",
    "fix": "Drop the passCap expression (keep cycle >= cfg.maxPassesPerRound) and restore state.extensionSessions = left - 1 for both flows. If an operator-facing pass extension is wanted, it needs its own design, grammar in the --extend-budget message, and a reason string naming the effective cap."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/gates.ts:404-411",
    "problem": "The light test-path widening is narrower than the design's. TEST_PATH_RE (gates.ts:361) covers .test.ts paths anywhere, but the light branch is `!inTests || /(^|\\/)tests\\//`, so a co-located test such as src/x.test.ts is denied for light implement/fix. A8 and the Ownership section (02-proposal.md:289-293) define the widening by the test-path predicate, and TEST_OWNERSHIP_LIGHT (prompts.ts:22-23) tells the coder it owns the test files, so the write is silently reverted by the sweep (runner.ts:282) and the resulting testsNeedUpdate flag is ignored in light (states.ts:183,191). gates.test.ts:596-609 and light-ownership.test.ts:72-94 only use paths under a tests/ directory, so the gap is unpinned; the narrowing exists only as a code comment (gates.ts:381), not in 03-build-notes.md.",
    "fix": "Widen as designed: inside the implement/fix branch, `if (flow === 'light') return true;` (the reserved-name rule at gates.ts:396 already wins), and pin stagePathAllowed('implement','src/x.test.ts','light') === true in the ownership matrix. If the narrower rule is intended, declare it in 03-build-notes.md and make TEST_OWNERSHIP_LIGHT name the tests/ boundary."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/gates.ts:370-372,399-401",
    "problem": "traversesDotSegments adds a new rule to the ownership spine that applies in both flows, refusing any path with a '.'/'..' segment. The proposal assigns the predicate no path parsing ('the flow parameter parses no path', 02-proposal.md:302, with A28's intake argument and moves.ts Rule 1 covering those shapes), and this changes full-flow verdicts: stagePathAllowed('research','docs/../src/x.ts') was true via the inDocs prefix and is now false. gates.test.ts:665-666 pins the new rule while 02-proposal.md:388 (#7) states the opposite expectation and #18 expects that spelling to reach the sweep as git's real path. Undeclared.",
    "fix": "Delete traversesDotSegments and its two assertions, relying on the A28 intake argument and moves.ts Rule 1 as the design does. If the defence is kept, declare it in 03-build-notes.md, state that it applies to both flows, and correct #7/#18 so design and matrix agree."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/gates.ts:88-109,132-135",
    "problem": "The plan-size cap is implemented twice. sourceAdmissible grew a kind: 'plan' arm that gates the document it is handed, but the runner only ever calls it with id.kind ('adr' | 'brief') at runner.ts:2142, so gates.test.ts:715-730 exercise a gate no run takes while the real gate is runner.ts:2616-2633. documentLines is a second line-count authority next to proposalSize's count at runner.ts:181, with nothing keeping them equal.",
    "fix": "Remove the 'plan' arm of sourceAdmissible and documentLines; keep proposalSize's count as the single authority and let light-plan-size.test.ts, which drives the real gate, be its test."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:2616-2633",
    "problem": "The plan stop runs after finalizeStage has committed the plan (executeStage at runner.ts:1810 commits at 1782-1791), where the design places it at the stage-ok/pre-commit point (02-proposal.md:276-279). It also fails open: planOutgrewFlow(size?.docLines ?? 0, cfg) reads a missing 02-proposal.md as 0 lines, so a plan stage that replies ok and writes nothing passes the gate and the run spends an implement session with no design document and an empty seat stamp.",
    "fix": "Gate on the size proposalSize already computes pre-commit at runner.ts:1659-1662, and deny a plan stage whose size is undefined."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/flows.ts:19,22,87,113,126",
    "problem": "The registry's exported names and signatures drift from the proposal's contract (02-proposal.md:98-124) undeclared: FlowId = 'full' | 'light' | (string & {}) instead of Flow = 'full' | 'light'; SeatPhase instead of ReviewPhase; seatsFor returns string[] instead of readonly string[]; configFor takes base: PipelineConfig = DEFAULT_CONFIG instead of base?: Partial<PipelineConfig>. The widening removes compile-time guarding of the flow value: PipelineState.flow?: FlowId (types.ts:238) accepts any string, leaving only the zod enum and the pre-lock check. FlowDef.path (flows.ts:36-37) is an extra field the proposal does not list. 03-build-notes.md deviation 1 declares only the assertStageInFlow parameter order.",
    "fix": "Export Flow = 'full' | 'light' and use it for PipelineState.flow, flowOf, configFor and assertStageInFlow; keep the open string arm for untrusted text (ParsedSource.flow, the registry lookup). Alias SeatPhase to ReviewPhase, return readonly string[], take Partial<PipelineConfig>. List any residue in 03-build-notes.md."
  },
  {
    "severity": "minor",
    "location": "docs/designs/010-lightweight-plan-flow/03-build-notes.md:96-101",
    "problem": "The note claims 'one frontmatter read per entry', which the code does not do: runPipeline reads and parses the source at runner.ts:2083, and sourceAdmissible reads and parses the same file again at gates.ts:97,101. The proposal's mechanism (02-proposal.md:210-220) — parse once after deriveIdentity, carry the parsed frontmatter on the deriveIdentity result, and pass that result to sourceAdmissible — is not implemented, so 'the value that cleared the pre-lock check is the value stamped' rests on the file not changing between reads (checklist item 5).",
    "fix": "Pass the ParsedSource (and its body) into sourceAdmissible and delete its readFile, or state the two reads plainly in 03-build-notes.md."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:2419-2451,2566-2568,3091",
    "problem": "For light, the runner's round-entry maxRounds backstop and round charge are skipped (if (!isLight)) and codeBudgetUsed is repurposed to count review sessions, so the binding maxRounds: 2 is states.ts:202 — behaviorally right, undeclared, and it kills the revise-round record the proposal names as the knob's consumer (02-proposal.md:200-201). The repurposed counter is then printed under its old label: statusLines reports 'budgets: design 0 rounds, code 6 rounds' (runner.ts:3091; report.ts:244-246 in ESCALATION.md) where 6 is review sessions, against the proposal's requirement that status and the report name the effective numbers.",
    "fix": "Print the light record with its unit (code 6 sessions when flowOf(state) === 'light', full flow's line byte-equal), and record in 03-build-notes.md that maxRounds binds for light at states.ts:202."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/correctness-r1c1.md, and write no other file: an audit checks
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
{"verdict":"request-changes","findings":[{"severity":"blocker","location":"tools/orchestration/src/pipeline/states.ts:191,197; tools/orchestration/src/pipeline/runner.ts:2560,2675-2679,2711-2715; 03-build-notes.md:115-124","problem":"Deviation 10's declared remedy for the repurposed codeBudgetUsed counter is incomplete. The light flow's fix loop entered from a red testsGreen gate is bound by budgetLeft(state, state.codeBudgetUsed, cfg) (states.ts:191,197), not by the codeRound check deviation 10 names, and this diff repurposed that counter: the per-fix charge is skipped for light (runner.ts:2711-2715), the round-entry charge is skipped (runner.ts:2560), and codeBudgetUsed is now the seats a code pass launched (runner.ts:2677-2679). Consequence 1: a light implement with red tests and no code review yet has codeBudgetUsed 0 forever, so decideNext returns stage('fix') on every iteration with no cap anywhere (the pass-entry session cap gates review passes only, codeRound never moves, stageFailures counts failures, and an ok fix is not one) — unbounded fix sessions at 12 min each, with no escalation, so the design's second deterministic stop is disabled. Consequence 2: after the first 2-seat code pass codeBudgetUsed is 2, so budgetLeft is 2 < 2 and the very first fix on red tests escalates with 'tests not green at the code-review budget limit' — the Goals' 'fix bounded at 2 rounds' becomes 1 round after review and unbounded before it. Nothing pins either shape: states.test.ts:403-419 asserts routing only, light-budget.test.ts runs only the green path (package.json check exits 0). Side effect: statusLines prints code 6 sessions from codeBudgetUsed and code 6 sessions from reviewSessionsUsed on the same line (runner.ts:3227-3231).","fix":"Keep the budget test in the unit the design states it in: restore state.codeBudgetUsed += 1 for light at runner.ts:2714 and 2560, drop the seats charge at runner.ts:2677-2679, and print the session count from reviewSessionsUsed(state, 'code') (which the status/report lines already do). Pin both shapes: a light run whose tests stay red from implement escalates with the round-limit record instead of looping, and a light dissent gets its second fix round. Update deviation 10 to name which check binds on which path."},{"severity":"major","location":"tools/orchestration/src/pipeline/runner.ts:2384-2389,2763; tools/orchestration/src/pipeline/report.ts:54-62","problem":"The plan stop's escalation reports the design phase with an empty stage name. state.lastStage = name is assigned after runWithRetry returns, and the pre-commit refusal throws StopDriving from inside it, so at finalizeEscalate lastStage is still '' and escalationPhase returns 'design'. The plan stop's ESCALATION.md opens \"Run stopped after stage `` in the **design** phase\" and reportFindings reads the design phase's round — exactly the mislabel A15 and the Reporting section (02-proposal.md:306-310) exist to remove, and Test strategy #10's plan-is-code pin passes only because escalation.test.ts:99-105,161 constructs lastStage: 'plan' by hand. light-plan-size.test.ts:176-197 asserts the size string and the promotion line, never the phase line, so the gap is unpinned.","fix":"Assign the stage before the gate can stop the run (state.lastStage = name in drive before the pre-commit gate, or in executeStage's refusal path at runner.ts:1879-1888), and extend the light-plan-size.test.ts record assertion to ESCALATION.md containing 'in the **code** phase'."},{"severity":"major","location":"tools/orchestration/src/pipeline/identity.ts:67,93-115; 03-build-notes.md:70-83","problem":"Scope creep in a shared entry path: the proposal states 'nextNnn and identityClash need no change' (02-proposal.md:322-324, A17), and the diff adds nnnOfExistingRun — for every non-ADR source, deriveIdentity now reads every docs/designs/*/.pipeline.json, matches source.path and pins that NNN. It is not flow-gated, so it changes the full flow's entry behaviour for every brief that has a run: a plain re-run is now a clash needing --force/--slug/--nnn (deviation 4 calls this the accepted side effect). That contradicts the design's compatibility claim that every existing document and checkpoint behaves exactly as it does today (02-proposal.md:13).","fix":"Gate the checkpoint-identity rule on the flow that needs it: resolve the source's flow: key before deriveIdentity (the parse already exists at runner.ts:2176-2181) and apply nnnOfExistingRun only to a light source, which is the promotion path Test strategy #22 pins, leaving the full flow's allocation exactly as A17 states. If the general rule is wanted, land it as its own change with its own design entry."},{"severity":"major","location":"03-build-notes.md:12-17,292-303; 02-proposal.md:441-443 (#21)","problem":"The change lands with the package's own gate red: the notes state that npm run check stops at format:check (gates.test.ts:706-708, runner.test.ts:390 committed unformatted) so the && chain never reaches vitest, while the headline claims 'The suite is green — 41 files, 473 passed / 2 skipped'. Test strategy #21 pins that the suite runs at the engines Node floor and npm run check stays green; the note's own fix is one command.","fix":"Land the formatting (npx prettier --write tests/pipeline/gates.test.ts tests/pipeline/runner.test.ts — the test owner's stage owns those paths) and restate the headline as 'npm run check green' once it is, so the note's opening claim and the gate agree."},{"severity":"minor","location":"tools/orchestration/src/pipeline/stages.ts:187-192,203-210; tools/orchestration/src/pipeline/flows.ts:78","problem":"The flow registry's own plan stage is not forceable. FlowDef.stages is documented as the --stage gate and assertStageInFlow implements it, but assertForceableStage runs first against STAGE_NAMES, which deliberately excludes plan via LIGHT_ONLY_STAGES. So 'pipeline run <light-doc> --stage plan' and 'pipeline resume <light-slug> --stage plan' are usage errors ('unknown --stage plan') even though plan is the light flow's first stage: an operator cannot re-plan a run the plan stop ended.","fix":"Accept a stage the flow's registry declares (admit names in FLOWS[flow].stages in the --stage validation, keeping STAGE_NAMES as the full-flow regression guard), and pin assertStageInFlow('light','plan') plus the CLI-level acceptance."},{"severity":"minor","location":"tools/orchestration/src/pipeline/runner.ts:214-224,2751; 03-build-notes.md:187-197","problem":"The plan stop deletes the plan the operator is told to promote from. The design pins the stop at the stage-ok/pre-commit point so the oversized plan is never committed; deviation 17 goes further and removes the document from the worktree (git restore from HEAD, else rmSync). The stop's own record reads ': set flow: full and re-run with --force' and A26 sells promotion on the plan surviving in branch history, so with this rule the refused plan exists nowhere — not committed, not in the worktree. The proposal never asks the runner to delete a stage's in-ownership product: docs/** is plan's allowed surface and the ownership sweep is the design's only revert mechanism.","fix":"Keep the pre-commit posture (no commit) and leave the bytes unstaged in the worktree so the operator can read and promote from them (the refusal path already runs the sweep at runner.ts:1886). If the deletion stays, name it in the escalation string and in the Risks/Deferred row and pin it."},{"severity":"nit","location":"tools/orchestration/src/pipeline/runner.ts:2567-2580; 03-build-notes.md:57-69","problem":"Deviation 3's stamp rule ('a ratify pass keeps the stamp its round entry set') is shared by both flows and is the mechanism deviation 11 credits with fixing run 010's near-miss, and the note asserts 'Full-flow code-phase tests are unaffected'. No full-flow test exercises the ratify stamp — codeStamp appears in tests only in fixtures and hand-built panel/states states, so the claim about the full flow is unpinned.","fix":"Add one full-flow pass sequence (full → dissent → fix → ratify) asserting the ratify pass does not move state.codeStamp, so the shared-path rule deviation 3 declares has a regression guard."}]}
```
