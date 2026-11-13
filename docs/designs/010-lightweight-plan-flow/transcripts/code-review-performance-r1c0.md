# Transcript: code-review:performance r1c0

## Provenance
- stage: code-review:performance (round r1c0)
- agent: codereview/code-performance
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-12T22:57:39.943Z
- ended: 2026-11-12T23:10:42.437Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 60faf8c24438ed55121757057b578182d2815083...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/010-lightweight-plan-flow.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/performance-r1c0.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"tools/orchestration/src/pipeline/runner.ts:2620 (with 1659-1682)","problem":"The oversized-plan gate re-measures the document with a second proposalSize call, repeating a measurement the same stage's inference half already made and recorded. executeInference computes size for docStages (['plan'] in light) at 1659-1662 and stores docLines on the ledger entry at 1674-1682; finalizeStage then sweeps and commits, so at 2620 the plan is in HEAD and the meter takes its full path: one readFileSync of the plan, one split('\\n'), and two spawnSync git calls. Measured on this machine: 3.52 ms median per proposalSize invocation, 3.3 ms of it the two process spawns (cat-file -e 1.43 ms, diff --numstat 1.84 ms). The value gated on is already on the entry just pushed, and the second call's added/removed are recomputed against a HEAD the commit moved, then discarded.","fix":"Gate on the recorded value (state.results.at(-1)?.docLines, handling the no-meter case) or return size on the StageOutcome, instead of re-invoking proposalSize; keeps the gate's number identical to the printed one and drops 2 subprocess spawns + 1 file read per plan stage."},{"severity":"minor","location":"tools/orchestration/src/pipeline/gates.ts:96-101 (with runner.ts:2083, 2142)","problem":"pipeline run reads and frontmatter-parses the source document twice per entry: the pre-lock read at runner.ts:2083 (readFileSync + parseSource) and then sourceAdmissible, which re-readFile's the same path and re-runs parseSource. The design requires the opposite — one frontmatter read per entry, with the post-lock sourceAdmissible gate taking the parsed result as an argument instead of re-reading. The implementation passes only the flow value, so the boot read is duplicated and the two reads can disagree on a file that changed in between.","fix":"Take the parsed result as the parameter (sourceAdmissible(parsed: ParsedSource, kind, flow), or pass the content) and hand it the object built at runner.ts:2083, carrying the pre-lock read's failure through for the unreadable-file denial. In the same pass, list docs/designs once and pass it to both nnnOfExistingRun and takenNames (identity.ts:67-68), which currently readdirSync the same directory twice per entry."},{"severity":"nit","location":"tools/orchestration/src/pipeline/gates.ts:371,388 (with runner.ts:282)","problem":"The ownership sweep calls stagePathAllowed once per touched path, and each call allocates a path.split('/') array to look for . / .. segments and does a flowDef(flow) registry lookup; runner.ts:282 evaluates flowOf(state) inside the filter callback, so both are per path. For a stage touching N paths that is N throwaway arrays and N registry lookups for a value constant across the sweep. Cost is unmeasurable at realistic N (one stage's diff, tens of paths) — this is the checklist's avoidable-allocation shape point, not a measured cost.","fix":"Hoist const flow = flowOf(state) above the filter, and make the traversal test allocation-free: /(^|\\/)\\.{1,2}(\\/|$)/.test(path), same verdict on the git-plumbing spellings the intake guarantees (A28)."},{"severity":"nit","location":"tools/orchestration/src/pipeline/runner.ts:2739 (and 2725)","problem":"stageVars re-resolves the config on every stage entry with configFor(flowOf(state)), and reviewFilesFor does the same on its fallback branch, while the drive loop that calls them already holds the resolved cfg (drive -> runWithRetry -> stageVars at 2247/2613). Each call runs the validation contract twice (validateConfig on base and merged, including the SEAT_NAMES.includes scans and a phasesRun scan of the 12-stage list) and allocates a merged config object per stage entry; the design's rule is that budgets and seats are resolved once from state.flow. It also resolves with an empty base, so it is not the operator's cfg the loop drives with. Cost is sub-millisecond (466-test suite in 5.60 s; a full light drive ~278 ms), so the issue is the second authority plus repeated validation, not the clock.","fix":"Thread the loop's cfg into stageVars and reviewFilesFor (the call site at 2613 is inside the loop that has it), keeping configFor's production call sites at the four places with no resolved config in hand: runPipeline, resumePipeline, statusLines, escalationMarkdown. Consider building SEAT_NAMES as a Set, which is the structure its only use (.includes membership) intends."}]}
```
