# Transcript: code-review:performance:attempt2 r1c0

## Provenance
- stage: code-review:performance:attempt2 (round r1c0)
- agent: codereview/code-performance
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-12T23:10:42.507Z
- ended: 2026-11-12T23:17:24.752Z
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

Previous attempt's gate failure (log excerpt): independence violation: shell accessed run records: cd /home/mike/repos/bindforge && stat -c '%y %n' docs/designs/010-lightweight-plan-flow/….
Do not repeat the behavior that caused that failure.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "tools/orchestration/src/pipeline/stages.ts:108", "problem": "DEFAULT_CONFIG.stageTimeoutMs was raised 30 min → 90 min, tripling the full flow's worst-case stage stall. The knob is the deadline of the per-session poll loop (runner.ts:1348, 1384-1388), which re-requests listSessionMessages every pollIntervalMs (2 s default) until it expires: a wedged server now costs 90 minutes and ~2,700 poll round-trips per attempt (~900 today), and runWithRetry's repair attempt makes a stalled stage ~3 h. It lands on every stage of every full-flow run, whose compatibility row promises \"budgets unchanged\" and whose Non-goals rule out full-flow budget changes.", "fix": "Restore the 30-minute default and express any extra headroom per stage in the flow registry (the pattern this change adds). If the real need is \"this stage is legitimately slow\", detect liveness (no new assistant turn plus exhausted nudge budget) instead of tripling the absolute cap, which charges the wedged-session case for the slow-but-alive one."}, {"severity": "major", "location": "tools/orchestration/src/pipeline/stages.ts:105", "problem": "DEFAULT_CONFIG.maxPassesPerRound was raised 6 → 8. At five seats a full-flow design round can now enter 8 passes, moving the binding stop from the backstop (6×5 = 30 sessions) to the phase cap maxReviews.design: 40 — a +33% worst-case review cost per design round, over up to maxRounds: 5 — and a single-dissenter code panel now runs 8 passes where 6 capped it. The design fixes the full flow at 6 passes (budget table) and lists full-flow budget changes as a non-goal.", "fix": "Restore 6 and fix the run-010 failure the comment describes in the funnel rather than in the global price: this branch already adds the ratify-pass-must-not-restamp rule (runner.ts:2468-2470), which is the convergence that round was one pass from, and --extend-budget is the designed per-run channel for one more pass."}, {"severity": "major", "location": "tools/orchestration/src/pipeline/runner.ts:2506-2507", "problem": "The light over-cap pass entry never consumes the grant (`if (!isLight) state.extensionSessions = left - 1;`), and the entry check escalates only when left <= 0, so a single --extend-budget 1 makes the phase-cumulative cap maxReviews.code: 6 non-binding for the rest of the run: every later over-cap pass is admitted and logs, and planPass's +1 pass lift (panel.ts:133-136) is gated on the same unspent grant so it stays lifted. Bound: 2 code rounds × (3+1 passes) × 2 seats = up to 16 review sessions against a 6-session budget (~2.7×, design's Rollout 6 expects ≈9 plus 1-2 per grant), and extensionSessions is state-side so it survives every resume. light-budget.test.ts:270-284 pins only the single-round case (8 sessions), leaving the multi-round and resume paths unpinned.", "fix": "Decrement the grant at the pass-entry check in both flows, and have planPass's lift read that same consumed value so one grant buys exactly one pass; pin the total session count in a test that drives a light run into a second code round with a grant in hand."}, {"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:2620", "problem": "The plan-size gate re-runs proposalSize for a document that was metered one frame earlier: docStages includes 'plan', so executeInference already computed it (runner.ts:1660-1661) and recorded it in the StageResult. proposalSize costs existsSync + readFileSync + two spawnSync git calls (cat-file -e, diff --numstat), and the gate reads only size?.docLines and discards added/removed — so two subprocess spawns and a second read are pure overhead on the only path where the count is already in hand.", "fix": "Read docLines from the StageResult the stage just recorded (thread it onto StageOutcome), or use the line-count-only path — documentLines (gates.ts:127-131) over a single read — so the gate pays one read and no subprocess."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:2083", "problem": "The source document is read and frontmatter-parsed twice per run entry: the pre-lock `parseSource(readFileSync(sourceAbs, 'utf8'), id.kind)` for the flow, then sourceAdmissible re-reads the file (gates.ts:99 `await readFile`) and re-runs parseSource. The design asks for one frontmatter read per entry and for sourceAdmissible to take the parsed result as an argument; the implementation passes only the flow string, so the read and scan duplicate.", "fix": "Pass the ParsedSource (or the content string) from the pre-lock parse into sourceAdmissible, making that single parse the authority for both the stamped flow and the admissibility verdict."}]}
```
