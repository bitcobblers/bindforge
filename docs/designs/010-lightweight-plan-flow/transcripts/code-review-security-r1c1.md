# Transcript: code-review:security r1c1

## Provenance
- stage: code-review:security (round r1c1)
- agent: codereview/code-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T00:23:19.130Z
- ended: 2026-11-13T00:28:10.189Z
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
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/stages.ts:108",
    "problem": "The full flow's cost ceilings were raised outside the design's authority: stageTimeoutMs 30 -> 90 minutes (stages.ts:108) and maxPassesPerRound 6 -> 8 (stages.ts:105). The proposal's Non-goals state 'No change to the full flow's stages, seats, budgets' and ADR-010's table pins full at 30 min / 6. docs/workflows.md, updated in this same diff, records maxPassesPerRound: 8 (lines 232, 243) but still states stageTimeoutMs: 30 min (lines 235, 251), so the operator-facing budget table contradicts the code on the one knob the doc calls load-bearing ('stage turns are bounded only by this timeout, there is no model-turn/step cap'). With the two attempts per stage at runner.ts:2284 and runner.ts:2320, the per-stage wall-clock ceiling triples to 3 hours for every existing run, not only light ones. No test pins either value: flows.test.ts:118-133 asserts configFor('full', base) equals a literal base (tautological against DEFAULT_CONFIG), leaving the light 12-min pin at flows.test.ts:92 as the only… [elided by the checkpoint cap]",
    "fix": "Restore 30 * 60_000 and 6, or move the two knob changes out of this design's scope with an amendment to the proposal/ADR carrying the run-010 evidence, update docs/workflows.md:235,251 to match the code, and pin the full-flow values with an assertion against DEFAULT_CONFIG."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:2506",
    "problem": "A light run's --extend-budget session grant is never consumed, so the flow's budget-exhaustion stop stops binding. The over-cap pass entry runs `if (!isLight) state.extensionSessions = left - 1;` (runner.ts:2506-2508), leaving extensionSessions intact for a light run, while planPass lifts the pass cap by one for as long as any grant is held (panel.ts:126-131). Every pass entry above maxReviews.code: 6 is therefore admitted for the life of the checkpoint, bounded only by maxPassesPerRound + 1 per round and maxRounds: 2 across rounds: up to 16 code sessions against a cap of 6. light-budget.test.ts:280 pins only the 8 sessions of the first round, leaving the multi-round case untested. Effects: one of ADR-010's two deterministic stops is disabled by a single grant; --extend-budget 5 behaves exactly like --extend-budget 1; and the gate log prints a constant '1 session grant(s) left' on every over-cap entry, overstating the audit record the operator resumes from. Design A16/A27 requires the … [elided by the checkpoint cap]",
    "fix": "Consume the grant in both flows (drop the !isLight guard) and key the light pass-cap lift on the remaining grant; assert in light-budget.test.ts that a second over-cap entry escalates and that the printed grant count decreases."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/flows.ts:164",
    "problem": "Seat names are validated against the union of both panels (SEAT_NAMES, flows.ts:25) instead of per phase, so design-only 'complexity' in reviewSeats.code passes pre-lock validation; the panel then resolves codereview/code-complexity (stages.ts:166), and .opencode/agents/codereview/ contains only code-correctness/compatibility/security/performance/testing. The failure surfaces mid-run as a failed stage attempt ('no model turns — agent ... may not exist', runner.ts:1435) after the lock, entry checkpoint and escalation path are live, burning an attempt at stageTimeoutMs; 'testing' in the design panel resolves review/design-testing the same way. This is the failure the design's Budget resolution cites as the reason the resolver validates at all, and flows.test.ts:214-225 catches only the typo shape. The same intake reaches `TypeError: undefined is not iterable` (flows.ts:160) when a JSON-shaped config omits one phase key, an opaque message rather than the usage error.",
    "fix": "Validate reviewSeats.code against CODE_SPECIALTIES and reviewSeats.design against DESIGN_SPECIALTIES per phase, guard a missing/non-array phase key with the same usage-error message, and add flows.test.ts cases for code: ['complexity'], design: ['testing'] and a one-key reviewSeats."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:2621",
    "problem": "The plan-size stop fails open when the plan document cannot be measured: planOutgrewFlow(size?.docLines ?? 0, cfg) reads 'no measurement' as 'within cap'. proposalSize returns undefined when docs/designs/<run>/02-proposal.md is absent (runner.ts:176-177), so a light plan stage that writes nothing, or writes it under any other name, passes the flow's one deterministic plan stop, spends the implement session, and leaves the two seats reviewing a run with no design document — the artifact their faithfulness check and the seat stamp are built from (A5).",
    "fix": "Deny when the measurement is missing: escalate 'plan document not written: <designDir>/02-proposal.md' when proposalSize returns undefined, and add the missing-file and zero-line cases to light-plan-size.test.ts (it currently covers only 79/80/81-line plans)."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/states.ts:145",
    "problem": "dissentingSeats(state, 'design') drops the cfg argument, so it resolves through panel.ts:59-65's DEFAULT_CONFIG default — the standing five seats — while its caller decideNext holds the resolved config and every sibling site threads it (states.ts:87, panel.ts:123-124). No registry flow sets a design panel today (FLOWS.full.config is empty), so the divergence is latent, but it is the residual the design's Risks row 4 records and the design's Budget resolution requires every panel reader to take the cfg its caller already holds. With a caller-supplied opts.cfg design panel, the dissenter list routed into the proposal-revise prompt is computed from a panel the run is not using, so a dissent from a seat outside the standing five is dropped from the revision input while the funnel keeps its record.",
    "fix": "Pass the config the caller has — dissentingSeats(state, 'design', cfg) — and assert in states.test.ts that a caller-supplied design panel drives the dissenter list."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:2079",
    "problem": "The source document is read and parsed twice per entry, against the design's one-read rule ('one frontmatter read per entry, so the value that cleared the pre-lock check is the value stamped and printed'). The flow value is threaded into the post-lock gate, so the stamped value is the pre-lock one (correct), but sourceAdmissible re-reads and re-parses the file (gates.ts:96-101), so the Context/Decision/status verdict is judged on a second read of a file the first read already parsed.",
    "fix": "Thread the pre-lock ParsedSource (and its content) into sourceAdmissible as an argument and delete the second read."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/gates.ts:88",
    "problem": "sourceAdmissible(sourcePath, 'plan', flow) is unreachable in production: deriveIdentity produces kind 'adr' | 'brief' only (identity.ts:58) and the runner passes id.kind (runner.ts:2142), so the plan branch — its empty-document denial and its pre-lock line cap — is reachable only from gates.test.ts:761. On the light path nothing bounds the source document's size, and documentLines (gates.ts:131-136) duplicates the meter's count at runner.ts:181, against the design's own single-authority argument.",
    "fix": "Call the gate on the light entry path (kind 'plan' when flowOf(state) === 'light'), or delete the branch and reuse proposalSize's count for the gate."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the security checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/security-r1c1.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"tools/orchestration/src/pipeline/runner.ts:2529 (consumer: tools/orchestration/src/pipeline/states.ts:70-72)","problem":"The round-1 fix consumes the session grant in both flows but leaves the round grant unconsumed in the light flow. state.extensionRounds is incremented on every --extend-budget (runner.ts:2224-2225, 2290-2291) and consumed at exactly one site — the round-entry check — which the fix gates on !isLight (runner.ts:2529). Its only other reader is budgetLeft (states.ts:71), so a light run that ever holds a round grant satisfies the red-test budgetLeft tests at states.ts:191 and :197 permanently: the fix -> red tests -> fix loop stays legal for the checkpoint's whole life, spending a coder session per iteration, with no operatorExtension gate-log line (it sits inside the !isLight branch) and no decreasing count in any record. ADR-010's second deterministic stop (\"budget exhaustion ... the reason names the flow\") is thus bypassable on that path by a grant that is never spent and never audited — the same shape as my round-1 finding 2, reported for sessions. The session cap (runner.ts:2604-2626) and the hard codeRound > cfg.maxRounds cap (states.ts:206-210) are unaffected, and the path requires an operator grant plus a red testsGreen, so severity is minor. No light test touches extensionRounds (only budget.test.ts:210,252, full flow).","fix":"Consume the round grant where the light flow's own rounds are entered (charge the red-test fix entry in the light branch, gated on the remaining grant) and emit the operatorExtension gate-log there so the printed count is the checkpoint's count; assert in light-budget.test.ts that a light run with --extend-budget and red tests escalates at the round cap and that extensionRounds decreases."}]}
```
