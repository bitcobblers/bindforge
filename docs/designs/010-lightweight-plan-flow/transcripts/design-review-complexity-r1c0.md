# Transcript: design-review:complexity r1c0

## Provenance
- stage: design-review:complexity (round r1c0)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T01:36:45.722Z
- ended: 2026-11-12T01:38:18.279Z
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
Your own previous findings (complexity seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/010-lightweight-plan-flow.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/complexity-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"The `plan` stage, `testOwnership` bullet + Assumption register A14 — \"the last rendered from the flow into `implement.md:4` and `fix.md:4-5` in place of the hard-coded \\\"Never edit test files\\\"\" (new)","problem":"The `fix` stage has a third live template the design never lists, and it carries the same rule plus the escape hatch to the deleted stage. `runner.ts:1200-1208` (`stageTemplate`) returns `fix-tests.md` for any `fix` entered with `state.testsGreen === false`, with no flow branch; `prompts/fix-tests.md:8-10` says \"a test file that is unformatted ... do not touch them: set testsNeedUpdate ... and the test engineer's stage fixes the files it owns\", and `:14` says \"Never edit test files; if a test itself must change, set testsNeedUpdate in your reply\". A light `fix` entered from a red testsGreen gate is told it owns nothing in `tests/**` and is routed to a stage light deletes — exactly the prompt-contradicts-the-ownership-matrix defect A14 was raised to close. `tests/pipeline/fix-trigger.test.ts:31,58` shows the template is live; the design's coverage list (A14, Rollout step 2, Test strategy #11) omits it, so the suite stays green over the gap.","fix":"Extend the same `{{testOwnership}}` variable to `fix-tests.md` — no new mechanism, the design's own variable in the third template, with its `testsNeedUpdate` sentence rendering the light branch (\"no test engineer exists in this flow\"). Add `fix-tests.md` to A14's site list, Rollout step 2, and Test strategy #11's render assertions."},{"severity":"minor","location":"Budget resolution — \"maxParallelReviews: // operator knob applied LAST / base?.maxParallelReviews ?? DEFAULT_CONFIG.maxParallelReviews ?? 1\" (new)","problem":"This key is dead logic that hard-codes a third copy of a default. The budget table says the knob is \"shared; `--max-parallel` applied last\", so no `FLOWS[flow].config` partial sets it: `...base` in the spread already lands the operator's value after the flow partial, and `...DEFAULT_CONFIG` already supplies `1` (`stages.ts:83`). The `?? 1` literal is a pinned default that outlives `DEFAULT_CONFIG` — raise the package default to 2 and `configFor` keeps returning 1 — contradicting the comment on the line above it, \"remains the sole default authority (A4)\".","fix":"Drop the key from the spread and add one clause: \"no flow partial may set `maxParallelReviews`, which is why spread order alone satisfies the ADR's applied-last rule.\" Test strategy #2's \"the operator's `maxParallelReviews` intact\" assertion covers it unchanged."},{"severity":"minor","location":"Budget resolution, seats paragraph — \"`funnelSpecialties(phase, cfg)` and `states.ts`'s `specialtiesFor(phase, cfg)` read `cfg.reviewSeats` and fall back to the five specialties when absent\" (new)","problem":"Those two functions are byte-identical today (`panel.ts:31-32`, `states.ts:25-26`, both `phase === 'code' ? CODE_SPECIALTIES : DESIGN_SPECIALTIES`). Threading `cfg` into both creates two implementations of the seat-resolution rule and two copies of the five-seat fallback — the drift the design cites against a cap living in two places (\"a cap in the skill *and* the registry is two authorities that drift\"), rebuilt in code. Test strategy #15 asserts the effective numbers, not the seat list, so a divergence between the two resolvers goes uncaught.","fix":"Name one resolver: `specialtiesFor(phase, cfg)` delegates to `funnelSpecialties(phase, cfg)` (both are already imported by `runner.ts`), so the `reviewSeats` read and the five-seat fallback exist once. Add a clause noting the direct `DESIGN_SPECIALTIES` use at `states.ts:152` is the editorial path, which light never reaches, so it needs no flow read."},{"severity":"minor","location":"The `plan` stage, oversized-plan gate — \"when the `## Files to touch` section's bullet count exceeds `cfg.planMaxFiles`\" (new)","problem":"Two costs for one mechanism. (a) ADR-010 fixes \"Two deterministic stops\" (the 80-line meter, the budget) and accepts as a cost that \"nothing in the tool stops an operator from picking `light`\"; the file ceiling is a third stop, and A24 concedes the constant has no datum — \"no repo datum measures it\". (b) The measured quantity is the bullet count the planner types, not the files the change touches, so the class the gate is for — \"small-in-lines, wide-in-consequences\" — clears it by grouping 20 files under 8 bullets: a knob whose signal is self-reported prose escalates on formatting while missing the risk it cites.","fix":"Count the section's path-shaped tokens (the same path strings `moves.ts`'s ownership predicate reads) rather than bullets, and land the ceiling in the data step the design already has — Rollout 6 settles A23-A25 from the first light runs — so the first merged slice ships the ADR's two stops. If it lands now, say in the gate paragraph that bullet count is a proxy and record the proxy's known miss."},{"severity":"nit","location":"The `plan` stage, skill bullet — \"Skill `.opencode/skills/plan-note/SKILL.md` (new): the template and its sections\" alongside \"Template `tools/orchestration/prompts/plan.md` (new) renders the five sections\" (new)","problem":"The five sections then live in two files — the shape the Options section rejects for the cap (\"a cap in the skill *and* the registry is two authorities that drift\"). The repo already has the single-source pattern: `prompts/proposal.md` is three lines pointing at `.opencode/skills/design-proposal/references/template.md`, which holds the structure.","fix":"Give `prompts/plan.md` the same 3-line pointer shape and keep the section list once, in `plan-note/references/template.md`: the skill owns the sections, the registry owns the numbers."}]}
```
