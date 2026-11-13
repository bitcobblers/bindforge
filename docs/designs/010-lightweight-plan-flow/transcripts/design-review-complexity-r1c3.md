# Transcript: design-review:complexity r1c3

## Provenance
- stage: design-review:complexity (round r1c3)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:11:11.759Z
- ended: 2026-11-12T02:12:56.331Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the ratification pass c3: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 24b804a9c29888a3b458242d27c370d6b48be4fb 4cc9accad12add4e888d0ca7be434acf93530a29` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Budget resolution, seats paragraph — \"`funnelSpecialties` (`panel.ts:31-32`) and `states.ts`'s byte-identical `specialtiesFor` (`states.ts:25-26`) delegate to it, so the `reviewSeats` read and the five-seat fallback live once, in one test\" (new)",
    "problem": "The delegation has no route for `cfg`, and the route it needs is the signature fan-out the Options section says the design avoided. `funnelSpecialties(phase)` is called from `dissentingSeats`, `staleSeats`, `seatsConverged`, `planPass` and `seatSummary` (`panel.ts:57,68,77,103-104,184`), and only `planPass` takes `cfg`; `specialtiesFor(phase)` is called from `runner.ts:2322,2332,2339,2345,2617`, none with `cfg`. Delegating to `seatsFor(phase, cfg)` therefore adds a `cfg` parameter to four panel functions plus their call sites in `states.ts`, `runner.ts` and `report.ts`, while Options records \"A threaded parameter was rejected: six changed signatures\". Unstated, an implementer at `seatsConverged(state, phase)` with no `cfg` in hand re-derives the panel from `state.seats` keys or keeps the `CODE_SPECIALTIES` import — the two-authority drift this paragraph claims to remove — and Test strategy #15 asserts the resolver, not the wrappers that cannot reach it. All five of my round-1 findings … [elided by the checkpoint cap]",
    "fix": "Add one clause naming the four signatures that gain `cfg` (`dissentingSeats`, `staleSeats`, `seatsConverged`, `seatSummary`) and stating that the effective config is already in scope at each of their call sites — `planPass`, `decideNext`, the status lines, `escalationMarkdown` — so nothing outside the funnel learns the seat list; and reword Options' rejection to what it rejects, a seat-list parameter threaded into the funnel from outside. No new mechanism."
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
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/complexity-r1c3.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Budget resolution, `validated` paragraph — \"must keep the two keys `seatsFor` reads non-empty — `reviewSeats.code`, and `reviewSeats.design` … `light`'s `design: []` is legal only because `light.stages` names no design stage, not as a tunable\" (new)","problem":"The rewritten validation contract is stated over \"a resolved config\" unconditionally, while the same section's budget table ships `reviewSeats.design: []` in `FLOWS.light.config`, so `configFor('light', base)` = `validated({...DEFAULT_CONFIG, ...base, ...FLOWS.light.config})` throws on the flow's own registry value and the light flow cannot resolve. The round-1 text I approved was coherent here (\"a non-empty seat list for every phase the flow's `stages` runs\"); the security-review closure in the Revision log (\"validates seat names, non-empty `reviewSeats.design`/`code`\") flattened it into a two-key rule and left the light exemption as a parenthetical the rule forbids. Test strategy #23 now reads as unconditional (\"an empty `reviewSeats.design` throw\") and asserts a throw for exactly the config Test strategy #1 pins (\"`reviewSeats.design` is empty\" for `FLOWS.light`), and Rollout 1 ships `configFor` with this contract. The non-emptiness check has a consumer for the code panel (`seatsConverged` is vacuously true on `[]`) and for design through the editorial path (`states.ts:152`), and no consumer for a flow that runs no design phase, so the scoped rule is also the smaller one.","fix":"Restore the scope in one clause: design non-emptiness required \"for every phase the flow's `stages` runs\", `reviewSeats.code` required unconditionally since every flow runs code review; keep the light exemption as the rule's condition rather than a parenthetical, so `validated` gains no input. Scope Test strategy #23 to `full` (\"an empty `reviewSeats.design` throws under `full`; under `light` the registry's own `[]` resolves\")."},{"severity":"minor","location":"State graph delta, escalation-suffix paragraph — \"The light suffix is appended to every code-phase escalation the light flow reaches: `decideNext`'s pass-backstop string (`states.ts:183`) and fix-round string (`states.ts:187-194`), and the runner's round-entry `code review revise-round backstop reached (2/2 rounds)`\" (new)","problem":"The enumeration is not exhaustive, and the new claim in the same diff that the session cap \"never fires\" is what justifies the omission. The session cap is per-phase cumulative (`reviewSessionsUsed` counts every `code-review:` result in the run, `runner.ts:2358` + `panel.ts:50-52`) while `maxPassesPerRound` is per-round, so `maxReviews.code: 6` equals one non-converging round's ceiling and is spent inside round 1; any later code-round entry satisfies `spent >= cap` and escalates on the session-budget string at `runner.ts:2362-2368`, a code-phase escalation the light flow reaches that the list omits. That entry is reachable by the design's own recovery path: `--stage code-review` clears `assertStageInFlow` for light and forces a new round (`states.ts:88-92`), which that comment calls the obvious thing to force after a round dies — i.e. after the pass backstop at 6 spent.","fix":"Replace the three string edits with one append in the single escalation writer: in `finalizeEscalate(repoDir, state, reason, cfg)` (`runner.ts:2229`, `2362`, `2387`), append the light suffix when `flowOf(state) === 'light'` and the `escalationPhase(state)` already computed at `runner.ts:2861` is `code`. One site covers all four code-phase escalations and none of the plan-stage ones, whose string already names the flow and the promotion recipe; drop the enumeration sentence (and the \"never fires\" clause) in favour of the choke point."},{"severity":"minor","location":"Rollout 6 — \"a run that escalates legitimately sits above it, at 10–12 sessions (Budget resolution)\" (new)","problem":"The escalated bound is not derivable from the section it cites, and that section is the arithmetic this diff rewrote. With `maxReviews.code: 6` as a phase-cumulative cap, a light run's code phase cannot reach 12 sessions: plan 1 + implement 1 + fix rounds + at most 6 code sessions, so the escalated ceiling is about 9 sessions plus the `--extend-budget` grants the operator buys. The number is the rollout's measurement yardstick for A23/A25, so a bound the caps cannot produce will read as a passing result for a run shape that never occurs.","fix":"State the escalated ceiling as what the caps produce (code phase ≤ 6 sessions, whole run ≈ 9, plus one session grant per `--extend-budget`), and remove the \"(Budget resolution)\" pointer for a figure that section does not support. Registry values only; no new mechanism."}]}
```
