# Transcript: design-review:complexity r3c0

## Provenance
- stage: design-review:complexity (round r3c0)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:48:36.171Z
- ended: 2026-11-12T02:49:38.469Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the ratification pass c0: revisions landed after you approved; you are checking only that they did not break what you approved of round 3. This pass's scope: the sections changed since you last reviewed: run `git diff c1b351d8aba07e7259984fe7bb04131e9edec880 92f71e02b5d66c6da627a93779d50ed5decc402d` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "severity": "minor",
    "location": "Budget resolution — \"Its five seat-list readers take the `cfg` their callers already hold: `dissentingSeats`, `staleSeats`, `seatsConverged`, `seatSummary` (`panel.ts:55,65,74,181`) gain the parameter\"",
    "problem": "new. The enumeration is one reader short of the live threading surface and one reader long of a consumer. Short: the sentence directly above routes `states.ts`'s `specialtiesFor` through `seatsFor(phase, cfg)`, and that function has five call sites (`runner.ts:2322,2332,2339,2345,2617`); the fifth sits behind `reviewFilesFor` (`runner.ts:2600`) inside `stageVars` (`runner.ts:2621`), a chain that holds no `cfg` today, so it gains two new parameters — not \"the `cfg` their callers already hold\". Long: `seatsConverged` has no production caller (only `tests/pipeline/panel.test.ts:127,130`), so threading `cfg` into it puts a parameter on surface no run executes (checklist item 6). The claim matters because \"callers already hold it\" is the argument Options uses to beat the rejected seat-list design and the property that makes this change threading-only. (Round-1 major closed: `planPass(state, phase, cfg)` at `panel.ts:100` does carry the config, its two sites are `panel.ts:103,104`, so a ligh… [elided by the checkpoint cap]",
    "fix": "One clause: state the threading surface as the four live panel readers plus `specialtiesFor`'s five call sites, naming that `reviewFilesFor`/`stageVars` gain the parameter (or scope the delegation to the two config-governed phases and leave the pre-seat fallback on the constant), and drop `seatsConverged` from the threaded list, noting it is asserted through `planPass`. Threading bookkeeping only; no new mechanism."
  },
  {
    "severity": "minor",
    "location": "State graph delta — \"gated on `flowOf(state) === 'light'` and `state.lastStage` in `implement`/`fix`/`code-review`: one append covers every escalation those stages reach\"",
    "problem": "new. The gate is a hand-written triple of three of the light flow's four stages, and this revision deleted the clause that justified the exclusion (\"skips the plan stop, whose string already names the flow and the promotion recipe\"). The triple excludes more than the plan-size stop: every escalation a light `plan` reaches lands in `finalizeEscalate` with `lastStage: 'plan'` — the 12-minute `stageTimeoutMs` (`runner.ts:1343`), the stage-failure escalations (`runner.ts:2249,2283`), the gate escalations (`runner.ts:2569`) — so a light run that stalls in `plan` prints no flow, against ADR-010's Status clause (\"An escalated light run must read as 'this was too big for the light flow', not as an unexplained stall\") and this design's own Risks mitigation \"the escalation names the flow, so an operator reads it as a light-flow limit\", which the stage set satisfies for `implement` only. It is also the flow's stage list written a second time, so a registry gain of a stage silently stops naming th… [elided by the checkpoint cap]",
    "fix": "Key the append on the flow's declared stages (`FLOWS[flow].stages`, registry data) and exclude the plan-size stop by the string the gate composes (that text already names the flow and `--force`); state in The `plan` stage that a light `plan` timeout or stage failure carries the suffix. Same single append site, one enumeration removed rather than added."
  },
  {
    "severity": "nit",
    "location": "Flow registry — \"The one seat-panel resolver; `funnelSpecialties` and `specialtiesFor` delegate to it. `export function seatsFor(phase: ReviewPhase, cfg: PipelineConfig)`\"",
    "problem": "new. The resolver is typed over three phases while the config it reads governs two: `reviewSeats` has `design` and `code` keys, and `ReviewPhase` also carries `editorial`, which `specialtiesFor` is called with (`runner.ts:2322-2345`). The \"one place\" resolver therefore needs a third branch answering from the five-seat constant, and the document never says which value it returns for a light run.",
    "fix": "Type the resolver with the panel's own `FunnelPhase` (`'design' | 'code'`) — exactly the phases `reviewSeats` has keys for — and leave the editorial answer in `specialtiesFor`, where the design already places it (`states.ts:152` keeps `DESIGN_SPECIALTIES`, editorial only). One type narrowed; no new code."
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
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/complexity-r3c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[]}
```
