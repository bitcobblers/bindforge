# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T01:29:33.317Z
- ended: 2026-11-12T01:31:42.104Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/010-lightweight-plan-flow.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Budget resolution — \"maxParallelReviews: // operator knob applied LAST\"; Test strategy #14 — \"`--extend-budget` on the resume buys sessions under the light cap (the knob an operator actually buys, A16)\"","problem":"The canonical rule orders `...base` then `...FLOWS[flow].config` and names exactly one knob applied after the flow partial (maxParallelReviews). Any budget raise arriving on the channel the CLI uses (A4's \"CLI spread\" / `opts.cfg ?? DEFAULT_CONFIG` seams) is therefore overwritten by the flow partial on a light run, so `--extend-budget`, which buys code-review sessions by raising maxReviews.code above the light 6, is silently discarded — Test #14's assertion has no mechanism as the rule is written, and the budget table gives maxReviews no operator path at all. This re-creates the design's own named hidden failure (\"Budget drift on resume\": operator extends, run escalates again at 6). Introduced by the r1 closure of OQ1/OQ2 (single-resolver rule).","fix":"State where a resume-time extension lands: apply it to the output of `configFor` in the same posture as the `maxParallelReviews` line (e.g. `maxReviews: bump(resolved.maxReviews, extend)`), naming that this displaces `...FLOWS[flow].config` as the last writer of maxReviews on a light resume, and assert the extended number in Test #14."},{"severity":"minor","location":"Budget resolution, \"Which knob binds\" — \"the reachable shape is exactly `c0` + two dissent passes = 3 two-seat passes\" and \"`maxPassesPerRound: 3` is the per-round backstop that can only bind at the same point\"","problem":"The same paragraph says dissent passes are \"≤2 seats each\", and the funnel keeps \"revision to the dissenters only, ratification for stale approvals\", so a one-dissenter round runs single-seat passes: c0 (2 sessions) + dissent (1) + ratification (1) = 3 passes at 4 sessions, where maxPassesPerRound: 3 binds first at 4 sessions and the terminal record is `fix-round backstop`, not the 6-session cap. Test strategy #13 is exactly that shape; #14's \"escalates at 6 code review sessions\" only holds if both seats dissent every pass. So \"maxReviews.code: 6 is the binding stop\" is shape-dependent, not general.","fix":"Qualify the claim: maxReviews.code: 6 binds in the both-seats-dissent shape; maxPassesPerRound: 3 binds first at 4 sessions when a pass runs one seat — and state which escalation string the operator reads in each shape."},{"severity":"minor","location":"The `plan` stage (oversized-plan gate) — \"a missing section counts as 0 items (a faithfulness concern for the seats, not a gate)\"; Risks row 2 — \"`planMaxFiles` catches files-spanned\"","problem":"The file-count gate keys on the bullet count of `## Files to touch`, so a plan that names its files in prose, or under a renamed/absent heading, counts 0 and passes the gate that exists for the very class this design fears (small in lines, wide in consequences). Risks row 2's mitigation claim is therefore formatting-dependent.","fix":"Make the section required: deny with `the light plan requires a ## Files to touch section (N files, cap 10)`, replacing the \"missing counts as 0\" rule and keeping the bullet-count parse."},{"severity":"minor","location":"Context — \"the same 13-stage, five-seat path\"; Compatibility matrix — \"13-stage path, 5 seats, budgets unchanged\"; Chosen design — \"`full` declares today's twelve stages\"","problem":"The full flow's stage count is stated two ways in one document (13 vs twelve). Test strategy #1 pins `FLOWS.full.stages` to \"today's stages exactly\", so the count is part of the contract, not prose, and an implementer cannot tell which is the regression baseline.","fix":"Use the STAGE_BASES row count in all three places."},{"severity":"minor","location":"Budget resolution — \"`funnelSpecialties(phase, cfg)` and `states.ts`'s `specialtiesFor(phase, cfg)` read `cfg.reviewSeats` and fall back to the five specialties when absent\"; Options — \"resolved from the flow exactly once\"","problem":"The seat fallback exists in two implementations (panel and states), so DEFAULT_CONFIG is not the sole default authority for the seat panel, and the fallback direction is the unsafe one: any light call site handed a config literal that is not configFor output fans out to five seats and looks healthy — the failure mode ADR-010 warns about. The \"resolved once\" claim does not hold as written.","fix":"Export one resolver from flows.ts (e.g. `seatsFor(phase, cfg)`) used by both funnelSpecialties and specialtiesFor, so there is a single fallback and a single place to test; name the residual \"absent → full panel\" direction in Risks row 4."},{"severity":"minor","location":"Test strategy #6 — \"`flow: Light` and `flow: sideways` reach a denial\"; Selecting the flow — \"throws `invalid flow \"<value>\" for <source>: expected full or light`\" and \"Unreadable files keep their existing `sourceAdmissible` denial\"","problem":"#6 and #17 describe the same input with two different outcomes: a source-gate denial in #6, a pre-lock usage error in the prose and in #17. The prose reserves denial for unreadable files. As written, the test contract is ambiguous about which outcome is asserted.","fix":"Change #6's verb to the pre-lock usage error and reserve \"denial\" for the unreadable-file case."},{"severity":"nit","location":"The `plan` stage — \"It states no number: `{{planMaxLines}}`/`{{planMaxFiles}}` are interpolated into the prompt from the registry\"; Options, second entry — \"The skill carries the template's five sections and no number\"","problem":"ADR-010 §Light stages states \"The `plan-note` skill carries the template and its length cap\". The design's placement (registry owns the numbers, skill carries the template) is argued and sound, but nowhere records that it supersedes the ADR sentence, so the cap's home is stated in two places across the two documents.","fix":"Add one clause in The `plan` stage (or Deferred): ADR-010's \"skill carries the length cap\" is superseded — the skill carries the template, the registry owns the numbers."}]}
```
