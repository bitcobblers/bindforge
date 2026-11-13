# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T01:51:54.987Z
- ended: 2026-11-12T01:55:47.087Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 9ae29b5ab2cae633405db0d35b4d4adaae4c0ebd 24b804a9c29888a3b458242d27c370d6b48be4fb` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/010-lightweight-plan-flow.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Budget resolution — \"maxParallelReviews: // operator knob applied LAST\"; Test strategy #14 — \"`--extend-budget` on the resume buys sessions under the light cap (the knob an operator actually buys, A16)\"",
    "problem": "The canonical rule orders `...base` then `...FLOWS[flow].config` and names exactly one knob applied after the flow partial (maxParallelReviews). Any budget raise arriving on the channel the CLI uses (A4's \"CLI spread\" / `opts.cfg ?? DEFAULT_CONFIG` seams) is therefore overwritten by the flow partial on a light run, so `--extend-budget`, which buys code-review sessions by raising maxReviews.code above the light 6, is silently discarded — Test #14's assertion has no mechanism as the rule is written, and the budget table gives maxReviews no operator path at all. This re-creates the design's own named hidden failure (\"Budget drift on resume\": operator extends, run escalates again at 6). Introduced by the r1 closure of OQ1/OQ2 (single-resolver rule).",
    "fix": "State where a resume-time extension lands: apply it to the output of `configFor` in the same posture as the `maxParallelReviews` line (e.g. `maxReviews: bump(resolved.maxReviews, extend)`), naming that this displaces `...FLOWS[flow].config` as the last writer of maxReviews on a light resume, and assert the extended number in Test #14."
  },
  {
    "severity": "minor",
    "location": "Budget resolution, \"Which knob binds\" — \"the reachable shape is exactly `c0` + two dissent passes = 3 two-seat passes\" and \"`maxPassesPerRound: 3` is the per-round backstop that can only bind at the same point\"",
    "problem": "The same paragraph says dissent passes are \"≤2 seats each\", and the funnel keeps \"revision to the dissenters only, ratification for stale approvals\", so a one-dissenter round runs single-seat passes: c0 (2 sessions) + dissent (1) + ratification (1) = 3 passes at 4 sessions, where maxPassesPerRound: 3 binds first at 4 sessions and the terminal record is `fix-round backstop`, not the 6-session cap. Test strategy #13 is exactly that shape; #14's \"escalates at 6 code review sessions\" only holds if both seats dissent every pass. So \"maxReviews.code: 6 is the binding stop\" is shape-dependent, not general.",
    "fix": "Qualify the claim: maxReviews.code: 6 binds in the both-seats-dissent shape; maxPassesPerRound: 3 binds first at 4 sessions when a pass runs one seat — and state which escalation string the operator reads in each shape."
  },
  {
    "severity": "minor",
    "location": "The `plan` stage (oversized-plan gate) — \"a missing section counts as 0 items (a faithfulness concern for the seats, not a gate)\"; Risks row 2 — \"`planMaxFiles` catches files-spanned\"",
    "problem": "The file-count gate keys on the bullet count of `## Files to touch`, so a plan that names its files in prose, or under a renamed/absent heading, counts 0 and passes the gate that exists for the very class this design fears (small in lines, wide in consequences). Risks row 2's mitigation claim is therefore formatting-dependent.",
    "fix": "Make the section required: deny with `the light plan requires a ## Files to touch section (N files, cap 10)`, replacing the \"missing counts as 0\" rule and keeping the bullet-count parse."
  },
  {
    "severity": "minor",
    "location": "Context — \"the same 13-stage, five-seat path\"; Compatibility matrix — \"13-stage path, 5 seats, budgets unchanged\"; Chosen design — \"`full` declares today's twelve stages\"",
    "problem": "The full flow's stage count is stated two ways in one document (13 vs twelve). Test strategy #1 pins `FLOWS.full.stages` to \"today's stages exactly\", so the count is part of the contract, not prose, and an implementer cannot tell which is the regression baseline.",
    "fix": "Use the STAGE_BASES row count in all three places."
  },
  {
    "severity": "minor",
    "location": "Budget resolution — \"`funnelSpecialties(phase, cfg)` and `states.ts`'s `specialtiesFor(phase, cfg)` read `cfg.reviewSeats` and fall back to the five specialties when absent\"; Options — \"resolved from the flow exactly once\"",
    "problem": "The seat fallback exists in two implementations (panel and states), so DEFAULT_CONFIG is not the sole default authority for the seat panel, and the fallback direction is the unsafe one: any light call site handed a config literal that is not configFor output fans out to five seats and looks healthy — the failure mode ADR-010 warns about. The \"resolved once\" claim does not hold as written.",
    "fix": "Export one resolver from flows.ts (e.g. `seatsFor(phase, cfg)`) used by both funnelSpecialties and specialtiesFor, so there is a single fallback and a single place to test; name the residual \"absent → full panel\" direction in Risks row 4."
  },
  {
    "severity": "minor",
    "location": "Test strategy #6 — \"`flow: Light` and `flow: sideways` reach a denial\"; Selecting the flow — \"throws `invalid flow \"<value>\" for <source>: expected full or light`\" and \"Unreadable files keep their existing `sourceAdmissible` denial\"",
    "problem": "#6 and #17 describe the same input with two different outcomes: a source-gate denial in #6, a pre-lock usage error in the prose and in #17. The prose reserves denial for unreadable files. As written, the test contract is ambiguous about which outcome is asserted.",
    "fix": "Change #6's verb to the pre-lock usage error and reserve \"denial\" for the unreadable-file case."
  },
  {
    "severity": "nit",
    "location": "The `plan` stage — \"It states no number: `{{planMaxLines}}`/`{{planMaxFiles}}` are interpolated into the prompt from the registry\"; Options, second entry — \"The skill carries the template's five sections and no number\"",
    "problem": "ADR-010 §Light stages states \"The `plan-note` skill carries the template and its length cap\". The design's placement (registry owns the numbers, skill carries the template) is argued and sound, but nowhere records that it supersedes the ADR sentence, so the cap's home is stated in two places across the two documents.",
    "fix": "Add one clause in The `plan` stage (or Deferred): ADR-010's \"skill carries the length cap\" is superseded — the skill carries the template, the registry owns the numbers."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/010-lightweight-plan-flow.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Budget resolution, \"Which knob binds in the light code phase\" + Test strategy #14(a) — \"both seats dissenting admits a 2-seat pass at 5 spent and closes the phase at 7 — the shape where `maxReviews.code: 6` binds, on the session-budget record\"","problem":"The light flow cannot reach that record, and 7 is not reachable in that shape. reviewSessionsUsed counts per-seat sessions (panel.ts:50-52, results entries `code-review:<specialty>`), and a dissent keeps the same round (states.ts:83: `review('code', open.length > 0 ? state.codeRound : state.codeRound + 1)`), so the both-seats shape is one round of three passes: 2+2+2 = 6 sessions, with pass entries at spent 0, 2, 4 — `spent >= cap` (runner.ts:2357-2359) never fires and spent is always even, so \"5 spent\" and \"closes at 7\" are unreachable. The next planPass returns {kind:'blocked'} at `cycle >= cfg.maxPassesPerRound` (panel.ts:113-119), which decideNext escalates (states.ts:183): the terminal record is `code review: pass backstop: 3 review passes already run in this code round (cap 3 passes per round) — the funnel is not converging` at 6 sessions. With 2 seats a round's 3 passes cost exactly the cap, so maxPassesPerRound: 3 prints on every shape (6 sessions both seats dissenting, 4 with one dissenter) and maxReviews.code: 6 never prints. Test #14(a) asserts an escalation record and session count the cited mechanism cannot produce, so implementation chases the test.","fix":"Pick one and name the mechanism. (a) Keep maxPassesPerRound: 3 and rewrite the paragraph to: the pass-backstop record prints at 6 sessions (both seats dissenting) and 4 (one dissenter); maxReviews.code: 6 is the ceiling it coincides with, not the string an operator reads — and make #14(a) assert the pass-backstop string at 6 sessions with the light suffix. (b) If the session-budget string is meant to be the light cap's record (ADR-010: \"Budget exhaustion … the reason names the flow\"), set light maxPassesPerRound: 4, so a pass entry at spent 6 hits `spent >= cap` and prints `code review session budget exhausted (6/6 sessions)`; then state the per-round band as 4–8 sessions and drop \"closes at 7\" (it needs a 1-seat pass leaving 5 spent)."},{"severity":"minor","location":"Budget resolution, \"`validated` is part of the contract\" — \"must keep `planMaxLines` a positive integer\" and \"must carry a non-empty seat list for every phase the flow's `stages` runs\"","problem":"Both clauses have no true reading for the full flow's own resolved config, which the same section promises is untouched. planMaxLines is a new optional field absent from DEFAULT_CONFIG (stages.ts:75-84) and from the budget table (\"absent (no gate)\"), so a resolved full config cannot make it \"a positive integer\", while Test #5 asserts the absent case works. The seat clause is keyed on the flow's stages, and full.stages runs `editorial-review`, while reviewSeats is keyed { design, code } and ReviewPhase includes 'editorial' (states.ts:13, called with 'editorial' at runner.ts:2322-2345): the editorial phase has no seat-list key to validate, and the specialtiesFor delegate hands seatsFor exactly that value.","fix":"Write \"absent, or a positive integer\", name the phase mapping in seatsFor (editorial reads reviewSeats.design — today's `phase === 'code' ? CODE : DESIGN` shape, panel.ts:31-32), and scope the non-emptiness clause to the design/code keys."},{"severity":"minor","location":"State graph delta, \"`case 'code-review'` backstop exhausted | escalate, naming the revise-round backstop\", and Budget resolution, \"`maxRounds: 2` bounds the mutating `fix` loop, whose record is the revise-round backstop\"","problem":"The code phase has two escalation strings and the revision renamed the one this row reaches. From decideNext's `case 'code-review'` the string is `… and the fix-round backstop is exhausted (2/2)` (states.ts:185-194) — r1's wording, which was correct; `code revise-round backstop reached (2/2 rounds)` comes from the runner's round-entry check (runner.ts:2382-2395). The row promises \"same message plus `the light flow (flow: light); promote …`\", so which sites carry the light suffix is now unstated, and Test #8 asserts only \"escalation naming the light flow\".","fix":"Restore \"naming the fix-round backstop\" in the row, and state that the light suffix is appended at both code-phase escalation sites (states.ts:187-194 and the runner's round-entry escalation), naming the two strings."},{"severity":"minor","location":"Ownership — \"the new name `plan` is governed by the **document-stage** rule in both flows — writes confined to the run's `docs/designs/NNN-<slug>/` paths\"","problem":"The cited rule is `return inDocs` with `inDocs = /^docs\\//` (gates.ts:304,314-315): it confines a document stage to any path under docs/, which is how doc-sync writes docs/workflows.md. A light `plan` can therefore write `docs/adrs/…`, and Test #7 (\"`plan` allows the run's `docs/designs/**` and denies `src/**`\") does not assert the run-directory confinement the paragraph claims, so the mitigation stated for Risks row 3 is narrower in reality.","fix":"Say the document-stage rule is `docs/**` and that the run's own directory is enforced by the template/agent config, not the predicate — or name the predicate change that narrows document stages to the run's directory and add that case to Test #7."},{"severity":"minor","location":"Ownership + Test strategy #7 — \"separator and case variants are settled before the predicate sees them\" / \"a path spelled with a foreign separator or a case variant arrives normalized and repo-root-relative, so it gets the verdict of its canonical spelling\"","problem":"Only the reserved-name rule is case-insensitive (namesReservedSegment, types.ts:195-207); TEST_PATH_RE (gates.ts:286) and inDocs (gates.ts:304) are case-sensitive, so `Tests/x.test.ts` is allowed under `full` where the canonical spelling is denied, and `Docs/designs/plan.md` is denied where the canonical spelling is allowed. The separator half is sound: paths reach the predicate from `git diff-tree --name-only -z` and dirtyFiles (runner.ts:261-278), POSIX and repo-root-relative.","fix":"Keep the separator claim, drop \"a case variant … canonical spelling\" from the guarantee and from #7, and record that case variants of the tests//docs/ prefixes are settled by the case-sensitive regexes, exactly as today."},{"severity":"nit","location":"Chosen design — \"`configFor('full', base)` is deep-equal to a **complete** `base`\"","problem":"The qualifier appears only here; the Compatibility row, Rollout 1 and Test strategy #3 all say plain \"deep-equal to `base`\", and #3 is the assertion, so an implementer who passes a partial literal gets a failure the design declares impossible.","fix":"Qualify Test strategy #3 (\"a complete `PipelineConfig` literal\"); the Compatibility and Rollout mentions can stay as they are."},{"severity":"nit","location":"Rollout 6 — \"check them against the derived bound — ≤ 8 sessions, ≤ 96 min at the 12-min stage bound in the serial default\"","problem":"That is Test #12's converging-path bound (`total sessions ≤ 2 + seats × 3`), not a number derived from the declared budgets: a light run that goes to the ceiling (plan + implement + 2 fix + 6 review sessions) lands at 10–12 sessions and 120–144 min, so the first light run that legitimately escalates reads as missing the bound.","fix":"Name it the converging-run bound and cite #12's formula, keeping 96 min as the wall-clock ceiling for that path."}]}
```
