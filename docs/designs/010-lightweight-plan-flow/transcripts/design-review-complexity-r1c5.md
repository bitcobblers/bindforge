# Transcript: design-review:complexity r1c5

## Provenance
- stage: design-review:complexity (round r1c5)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:24:04.835Z
- ended: 2026-11-12T02:26:09.656Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c5: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 640d904da019a272ecbfd1f69444626748d997f6 c1b351d8aba07e7259984fe7bb04131e9edec880` (both are blob ids of 02-proposal.md), then read the full current document.
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
[
  {
    "severity": "major",
    "location": "Budget resolution — \"The four funnel readers — `dissentingSeats`, `staleSeats`, `seatsConverged`, `seatSummary` (`panel.ts:55,65,74,181`) — gain the `cfg` parameter their callers in `runner.ts` and `states.ts` already hold, so no call site keeps a `*_SPECIALTIES` import\" (new)",
    "problem": "The enumeration names four of the five readers of the seat list, and the missing one is the reader that chooses which seats launch. `funnelSpecialties(phase)` is called at `panel.ts:57,68,77,103,104,184`; 103-104 are inside `planPass` (`panel.ts:100`), whose `{ pass: 'full', seats: [...funnelSpecialties(phase)] }` is handed straight to `runReviewPass` as `seats` (`runner.ts:2453`) — the proposal's own A2 says \"the pass loop itself takes seats from `planPass`\". Threaded as written, `planPass` keeps the un-scoped `funnelSpecialties(phase)`, so a light run's first code pass launches the five `CODE_SPECIALTIES` seats: the light flow pays the exact cost ADR-010 exists to remove, `maxReviews.code: 6` is spent by pass one plus a partial, and #12's `2 + seats × 3` bound and Rollout 6's yardstop stop holding; the ledger then records five seats while `dissentingSeats`/`seatsConverged`/`seatSummary` read two, so dissent from the three uncounted seats is invisible. It is also the site Test strateg… [elided by the checkpoint cap]",
    "fix": "Name `planPass` in the enumeration (five readers: `panel.ts:55,65,74,100,181`), stating that its two `funnelSpecialties` sites (`panel.ts:103,104`) pass the `cfg` its signature already carries, so the full-pass seat list is the flow's panel. Threading only; no new mechanism."
  },
  {
    "severity": "minor",
    "location": "Budget resolution — the `configFor` pseudo-code, \"`configFor(flow, base) = validated({ ...DEFAULT_CONFIG, ...base, ...FLOWS[flow].config })`\" (new)",
    "problem": "The validation rule this round re-scoped is flow-dependent — \"the check is scoped to the phases the flow's `stages` runs\" (#23), \"A flow that names no design stage resolves the registry's `design: []`\" — but the canonical block passes `validated` the merged config only, where it can see no `stages`. An implementer following the block writes the unconditional two-key check again and reopens the defect this round closed: `configFor('light', base)` throws on the registry's own `design: []`.",
    "fix": "Pass the flow (or its `stages`) into the helper in the block — `validated(merged, flow)` — so the block states the rule the prose now carries. One argument; no new mechanism."
  },
  {
    "severity": "nit",
    "location": "State graph delta — \"The light suffix is appended at **one** site — `finalizeEscalate` (`runner.ts:2845`), which already computes `escalationPhase(state)` — gated on `flowOf(state) === 'light'` and a code-phase escalation from `implement`/`fix`/`code-review`: ... and skips the plan stop\" (new)",
    "problem": "The gate is right, but the cited discriminator cannot do the skipping once the Reporting change lands: `plan` joins `CODE_STAGES` in `report.ts`, and `CODE_STAGES` feeds only `escalationPhase` (`report.ts:49-57`), so at the plan-size stop `escalationPhase(state)` is `code` and a phase-gated append doubles the promotion recipe onto a string that already names the flow and `--force`.",
    "fix": "One clause: the gate keys on the stage set (`implement`/`fix`/`code-review`), noting that `escalationPhase` is not the discriminator because a light `plan` stop reads `code` by the Reporting change."
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
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/complexity-r1c5.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Budget resolution — \"Its five seat-list readers take the `cfg` their callers already hold: `dissentingSeats`, `staleSeats`, `seatsConverged`, `seatSummary` (`panel.ts:55,65,74,181`) gain the parameter\"","problem":"new. The enumeration is one reader short of the live threading surface and one reader long of a consumer. Short: the sentence directly above routes `states.ts`'s `specialtiesFor` through `seatsFor(phase, cfg)`, and that function has five call sites (`runner.ts:2322,2332,2339,2345,2617`); the fifth sits behind `reviewFilesFor` (`runner.ts:2600`) inside `stageVars` (`runner.ts:2621`), a chain that holds no `cfg` today, so it gains two new parameters — not \"the `cfg` their callers already hold\". Long: `seatsConverged` has no production caller (only `tests/pipeline/panel.test.ts:127,130`), so threading `cfg` into it puts a parameter on surface no run executes (checklist item 6). The claim matters because \"callers already hold it\" is the argument Options uses to beat the rejected seat-list design and the property that makes this change threading-only. (Round-1 major closed: `planPass(state, phase, cfg)` at `panel.ts:100` does carry the config, its two sites are `panel.ts:103,104`, so a light full pass launches the flow's two seats and #12's bound holds; the round-1 minor and nit on the `validated` argument and the escalation discriminator are also closed.)","fix":"One clause: state the threading surface as the four live panel readers plus `specialtiesFor`'s five call sites, naming that `reviewFilesFor`/`stageVars` gain the parameter (or scope the delegation to the two config-governed phases and leave the pre-seat fallback on the constant), and drop `seatsConverged` from the threaded list, noting it is asserted through `planPass`. Threading bookkeeping only; no new mechanism."},{"severity":"minor","location":"State graph delta — \"gated on `flowOf(state) === 'light'` and `state.lastStage` in `implement`/`fix`/`code-review`: one append covers every escalation those stages reach\"","problem":"new. The gate is a hand-written triple of three of the light flow's four stages, and this revision deleted the clause that justified the exclusion (\"skips the plan stop, whose string already names the flow and the promotion recipe\"). The triple excludes more than the plan-size stop: every escalation a light `plan` reaches lands in `finalizeEscalate` with `lastStage: 'plan'` — the 12-minute `stageTimeoutMs` (`runner.ts:1343`), the stage-failure escalations (`runner.ts:2249,2283`), the gate escalations (`runner.ts:2569`) — so a light run that stalls in `plan` prints no flow, against ADR-010's Status clause (\"An escalated light run must read as 'this was too big for the light flow', not as an unexplained stall\") and this design's own Risks mitigation \"the escalation names the flow, so an operator reads it as a light-flow limit\", which the stage set satisfies for `implement` only. It is also the flow's stage list written a second time, so a registry gain of a stage silently stops naming the flow — against the registry-as-data goal in ADR-010's Consequences.","fix":"Key the append on the flow's declared stages (`FLOWS[flow].stages`, registry data) and exclude the plan-size stop by the string the gate composes (that text already names the flow and `--force`); state in The `plan` stage that a light `plan` timeout or stage failure carries the suffix. Same single append site, one enumeration removed rather than added."},{"severity":"nit","location":"Flow registry — \"The one seat-panel resolver; `funnelSpecialties` and `specialtiesFor` delegate to it. `export function seatsFor(phase: ReviewPhase, cfg: PipelineConfig)`\"","problem":"new. The resolver is typed over three phases while the config it reads governs two: `reviewSeats` has `design` and `code` keys, and `ReviewPhase` also carries `editorial`, which `specialtiesFor` is called with (`runner.ts:2322-2345`). The \"one place\" resolver therefore needs a third branch answering from the five-seat constant, and the document never says which value it returns for a light run.","fix":"Type the resolver with the panel's own `FunnelPhase` (`'design' | 'code'`) — exactly the phases `reviewSeats` has keys for — and leave the editorial answer in `specialtiesFor`, where the design already places it (`states.ts:152` keeps `DESIGN_SPECIALTIES`, editorial only). One type narrowed; no new code."}]}
```
