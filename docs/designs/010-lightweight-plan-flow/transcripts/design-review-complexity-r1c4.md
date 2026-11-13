# Transcript: design-review:complexity r1c4

## Provenance
- stage: design-review:complexity (round r1c4)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:19:32.583Z
- ended: 2026-11-12T02:21:41.574Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c4: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 4cc9accad12add4e888d0ca7be434acf93530a29 640d904da019a272ecbfd1f69444626748d997f6` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Budget resolution, `validated` paragraph — \"must keep the two keys `seatsFor` reads non-empty — `reviewSeats.code`, and `reviewSeats.design` … `light`'s `design: []` is legal only because `light.stages` names no design stage, not as a tunable\" (new)",
    "problem": "The rewritten validation contract is stated over \"a resolved config\" unconditionally, while the same section's budget table ships `reviewSeats.design: []` in `FLOWS.light.config`, so `configFor('light', base)` = `validated({...DEFAULT_CONFIG, ...base, ...FLOWS.light.config})` throws on the flow's own registry value and the light flow cannot resolve. The round-1 text I approved was coherent here (\"a non-empty seat list for every phase the flow's `stages` runs\"); the security-review closure in the Revision log (\"validates seat names, non-empty `reviewSeats.design`/`code`\") flattened it into a two-key rule and left the light exemption as a parenthetical the rule forbids. Test strategy #23 now reads as unconditional (\"an empty `reviewSeats.design` throw\") and asserts a throw for exactly the config Test strategy #1 pins (\"`reviewSeats.design` is empty\" for `FLOWS.light`), and Rollout 1 ships `configFor` with this contract. The non-emptiness check has a consumer for the code panel (`seatsCon… [elided by the checkpoint cap]",
    "fix": "Restore the scope in one clause: design non-emptiness required \"for every phase the flow's `stages` runs\", `reviewSeats.code` required unconditionally since every flow runs code review; keep the light exemption as the rule's condition rather than a parenthetical, so `validated` gains no input. Scope Test strategy #23 to `full` (\"an empty `reviewSeats.design` throws under `full`; under `light` the registry's own `[]` resolves\")."
  },
  {
    "severity": "minor",
    "location": "State graph delta, escalation-suffix paragraph — \"The light suffix is appended to every code-phase escalation the light flow reaches: `decideNext`'s pass-backstop string (`states.ts:183`) and fix-round string (`states.ts:187-194`), and the runner's round-entry `code review revise-round backstop reached (2/2 rounds)`\" (new)",
    "problem": "The enumeration is not exhaustive, and the new claim in the same diff that the session cap \"never fires\" is what justifies the omission. The session cap is per-phase cumulative (`reviewSessionsUsed` counts every `code-review:` result in the run, `runner.ts:2358` + `panel.ts:50-52`) while `maxPassesPerRound` is per-round, so `maxReviews.code: 6` equals one non-converging round's ceiling and is spent inside round 1; any later code-round entry satisfies `spent >= cap` and escalates on the session-budget string at `runner.ts:2362-2368`, a code-phase escalation the light flow reaches that the list omits. That entry is reachable by the design's own recovery path: `--stage code-review` clears `assertStageInFlow` for light and forces a new round (`states.ts:88-92`), which that comment calls the obvious thing to force after a round dies — i.e. after the pass backstop at 6 spent.",
    "fix": "Replace the three string edits with one append in the single escalation writer: in `finalizeEscalate(repoDir, state, reason, cfg)` (`runner.ts:2229`, `2362`, `2387`), append the light suffix when `flowOf(state) === 'light'` and the `escalationPhase(state)` already computed at `runner.ts:2861` is `code`. One site covers all four code-phase escalations and none of the plan-stage ones, whose string already names the flow and the promotion recipe; drop the enumeration sentence (and the \"never fires\" clause) in favour of the choke point."
  },
  {
    "severity": "minor",
    "location": "Rollout 6 — \"a run that escalates legitimately sits above it, at 10–12 sessions (Budget resolution)\" (new)",
    "problem": "The escalated bound is not derivable from the section it cites, and that section is the arithmetic this diff rewrote. With `maxReviews.code: 6` as a phase-cumulative cap, a light run's code phase cannot reach 12 sessions: plan 1 + implement 1 + fix rounds + at most 6 code sessions, so the escalated ceiling is about 9 sessions plus the `--extend-budget` grants the operator buys. The number is the rollout's measurement yardstick for A23/A25, so a bound the caps cannot produce will read as a passing result for a run shape that never occurs.",
    "fix": "State the escalated ceiling as what the caps produce (code phase ≤ 6 sessions, whole run ≈ 9, plus one session grant per `--extend-budget`), and remove the \"(Budget resolution)\" pointer for a figure that section does not support. Registry values only; no new mechanism."
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
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/complexity-r1c4.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Budget resolution — \"The four funnel readers — `dissentingSeats`, `staleSeats`, `seatsConverged`, `seatSummary` (`panel.ts:55,65,74,181`) — gain the `cfg` parameter their callers in `runner.ts` and `states.ts` already hold, so no call site keeps a `*_SPECIALTIES` import\" (new)","problem":"The enumeration names four of the five readers of the seat list, and the missing one is the reader that chooses which seats launch. `funnelSpecialties(phase)` is called at `panel.ts:57,68,77,103,104,184`; 103-104 are inside `planPass` (`panel.ts:100`), whose `{ pass: 'full', seats: [...funnelSpecialties(phase)] }` is handed straight to `runReviewPass` as `seats` (`runner.ts:2453`) — the proposal's own A2 says \"the pass loop itself takes seats from `planPass`\". Threaded as written, `planPass` keeps the un-scoped `funnelSpecialties(phase)`, so a light run's first code pass launches the five `CODE_SPECIALTIES` seats: the light flow pays the exact cost ADR-010 exists to remove, `maxReviews.code: 6` is spent by pass one plus a partial, and #12's `2 + seats × 3` bound and Rollout 6's yardstop stop holding; the ledger then records five seats while `dissentingSeats`/`seatsConverged`/`seatSummary` read two, so dissent from the three uncounted seats is invisible. It is also the site Test strategy #9 pins (\"a full pass names both seats\"), so the test asserts behavior the change list never adds. `planPass` is the one reader that already takes `cfg` (`panel.ts:100`), so the fix costs zero new parameters.","fix":"Name `planPass` in the enumeration (five readers: `panel.ts:55,65,74,100,181`), stating that its two `funnelSpecialties` sites (`panel.ts:103,104`) pass the `cfg` its signature already carries, so the full-pass seat list is the flow's panel. Threading only; no new mechanism."},{"severity":"minor","location":"Budget resolution — the `configFor` pseudo-code, \"`configFor(flow, base) = validated({ ...DEFAULT_CONFIG, ...base, ...FLOWS[flow].config })`\" (new)","problem":"The validation rule this round re-scoped is flow-dependent — \"the check is scoped to the phases the flow's `stages` runs\" (#23), \"A flow that names no design stage resolves the registry's `design: []`\" — but the canonical block passes `validated` the merged config only, where it can see no `stages`. An implementer following the block writes the unconditional two-key check again and reopens the defect this round closed: `configFor('light', base)` throws on the registry's own `design: []`.","fix":"Pass the flow (or its `stages`) into the helper in the block — `validated(merged, flow)` — so the block states the rule the prose now carries. One argument; no new mechanism."},{"severity":"nit","location":"State graph delta — \"The light suffix is appended at **one** site — `finalizeEscalate` (`runner.ts:2845`), which already computes `escalationPhase(state)` — gated on `flowOf(state) === 'light'` and a code-phase escalation from `implement`/`fix`/`code-review`: ... and skips the plan stop\" (new)","problem":"The gate is right, but the cited discriminator cannot do the skipping once the Reporting change lands: `plan` joins `CODE_STAGES` in `report.ts`, and `CODE_STAGES` feeds only `escalationPhase` (`report.ts:49-57`), so at the plan-size stop `escalationPhase(state)` is `code` and a phase-gated append doubles the promotion recipe onto a string that already names the flow and `--force`.","fix":"One clause: the gate keys on the stage set (`implement`/`fix`/`code-review`), noting that `escalationPhase` is not the discriminator because a light `plan` stop reads `code` by the Reporting change."}]}
```
