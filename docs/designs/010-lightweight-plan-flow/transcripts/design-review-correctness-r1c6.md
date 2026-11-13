# Transcript: design-review:correctness r1c6

## Provenance
- stage: design-review:correctness (round r1c6)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:27:05.954Z
- ended: 2026-11-12T02:30:04.999Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the ratification pass c6: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 4cc9accad12add4e888d0ca7be434acf93530a29 c1b351d8aba07e7259984fe7bb04131e9edec880` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "severity": "minor",
    "location": "Budget resolution, \"Which knob binds in the light code phase\" — \"The session cap is checked at **pass entry**, ahead of that backstop (A16), so it never fires\"",
    "problem": "new. The conclusion is right, the stated mechanism is not, and the stated mechanism predicts the opposite string. The cap check (runner.ts:2357-2359) is ahead of the in-loop planPass (runner.ts:2422), so read literally the sentence says the session-budget record wins at spent 6 — the exact thing Test strategy #14(a) asserts never prints. The reason it never fires is the other half of A16: decideNext calls planPass at states.ts:181 and escalates the blocked pass-backstop reason at states.ts:183, and decideNext runs at the top of every drive-loop iteration, before the review branch's cap check, so the iteration at spent 6 ends in escalation and no pass entry ever happens at the cap. The paragraph's arithmetic (4–6 band, 6/4 sessions, cap never firing) is verified against the code; only its causal clause is wrong.",
    "fix": "Replace \"ahead of that backstop (A16), so it never fires\" with: the pass backstop is decided upstream, in decideNext's planPass call (states.ts:181-183), before the loop reaches the pass-entry cap check (runner.ts:2357), so the iteration at spent 6 escalates on the backstop and no pass entry happens at the cap."
  },
  {
    "severity": "minor",
    "location": "Budget resolution, \"`validated` is part of the contract\" + Test strategy #23 — \"must keep the two keys `seatsFor` reads non-empty — `reviewSeats.code`, and `reviewSeats.design`\" / \"`reviewSeats.code: []` and an empty `reviewSeats.design` throw\"",
    "problem": "re-raised (part of r1c1 finding 2; partially closed — the mapping is named, planMaxLines is now absent-or-positive). The light partial in the budget table sets reviewSeats.design to [], so every light resolved config has an empty design list. #23 carries no flow qualifier, so an implementer who resolves configFor('light', …) and asserts the throw gets a failure the same section declares legal (\"`light`'s `design: []` is legal only because `light.stages` names no design stage\"). The legality clause gives a reason, not the mechanism the validation keys on.",
    "fix": "Key the clause on the flow's stages — \"`reviewSeats.design` non-empty for a flow whose `stages` run a design or editorial stage (full's proposal-final → editorial reads it through states.ts:25-26), exempting `light`'s registry `[]`\" — and scope #23's design assertion to a `full`-flow resolution."
  },
  {
    "severity": "minor",
    "location": "State graph delta — \"The light suffix is appended to every code-phase escalation the light flow reaches: `decideNext`'s pass-backstop string (`states.ts:183`) and fix-round string (`states.ts:187-194`), and the runner's round-entry `code review revise-round backstop reached (2/2 rounds)`\"",
    "problem": "new. \"Every\" and the three-site list disagree: a fourth code-phase escalation is reachable in light and is not listed — escalate('tests not green at the code-review budget limit') at states.ts:170-178, taken from case 'implement'/case 'fix' with testsGreen: false once codeBudgetUsed >= maxRounds. Light is the flow where that stop is likeliest (no test engineer, A13/A14), and ADR-010's Status section requires an escalated light run to read as \"too big for the light flow\". An implementer who does the three named sites leaves that escalation naming no flow. The three quoted strings themselves are verbatim correct.",
    "fix": "Add that string to the enumeration, naming states.ts:172,178, or narrow the sentence to \"the three review escalations\" and add a row for the red-tests limit."
  },
  {
    "severity": "nit",
    "location": "Ownership — \"Paths reach the predicate repo-root-relative and POSIX-normalized (A8/A9), so the `flow` parameter parses no path; case is **not** normalized — as today.\"",
    "problem": "new. As a blanket statement this is false for the rule the same paragraph puts first: namesReservedSegment lowercases each segment (types.ts:195-207), which is how the code comment at gates.ts:294-300 reads (\"no stage class … can own them at any depth or under another case\"). The narrowing that closed r1c1 finding 5 overreached from tests//docs/ to all of the predicate.",
    "fix": "Scope it: \"case is not normalized for the `tests/`/`docs/` regexes, as today; the reserved-name rule is case-insensitive by design\"."
  },
  {
    "severity": "nit",
    "location": "Ownership vs. The `plan` stage — \"confining `plan` to the run's own directory is template and agent-config work, not predicate work\" vs. \"edits `docs/designs/**`\"",
    "problem": "new. The named agent config grants every run's directory, so the confinement the closure points to as agent-config work is not implemented by the config the design specifies; Risks row 3's mitigation stays narrower than the paragraph implies.",
    "fix": "Say the planner's grant is `docs/designs/**` and that run-directory confinement is only the template's `{{designDir}}` output path (no predicate, no agent rule), or narrow the planner grant to the run's directory."
  },
  {
    "severity": "nit",
    "location": "Budget resolution vs. State graph delta — \"`maxPassesPerRound: 3` is the record that prints on **every** shape\"",
    "problem": "new. The State graph delta says of the same phase that the deadlock stop \"is seat-count-agnostic and fires at 2 seats; with `maxRounds: 2` it is likelier to be the terminal record\", and planPass returns that `deadlock:` blocked reason from the same blocked site (panel.ts:121-129), so the pass-count record is not the terminal record on that shape; the two sections disagree on which record ends the phase.",
    "fix": "Qualify to \"every pass-count shape that is not the deadlock stop (`panel.ts:121-129`)\"."
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
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/correctness-r1c6.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Budget resolution, \"Which knob binds in the light code phase\" — \"The session cap is checked at **pass entry**, ahead of that backstop (A16), so inside the first code round the backstop prints first.\"","problem":"re-raised (partially closed). The false prediction \"so it never fires\" is gone, and the new cumulative-cap sentence is sound: reviewSessionsUsed counts every `code-review:` result (panel.ts:50-51), so the cap binds only at a pass entry in a later code round, which the `--stage code-review` force example reaches and #14(c) asserts. The causal clause is unchanged from r1c1 finding 2: read literally, \"cap checked at pass entry, ahead of the backstop\" predicts the session-budget string at spent 6, which #14(a) says never prints in that round. The reason the backstop wins inside the first round is upstream — decideNext calls planPass (states.ts:181) and escalates the blocked pass-backstop reason (states.ts:183) at the top of the drive-loop iteration, before the review branch reaches runner.ts:2357 — so the iteration at spent 6 ends in escalation and no pass entry happens at the cap.","fix":"Replace the `so`-clause's basis: \"the pass backstop is decided upstream, in decideNext's planPass call (states.ts:181-183), so the iteration at spent 6 escalates on it and no pass entry happens at the cap.\""},{"severity":"minor","location":"Ownership — \"so the `flow` parameter parses no path; case is **not** normalized — as today.\"","problem":"re-raised (r1c1 finding 4 not applied). The revision rewrote the sentences around this clause (the A28 intake site) and re-asserted the blanket. namesReservedSegment lowercases each segment (types.ts:195-207), which is how the code comment at gates.ts:294-300 reads (\"no stage class … can own them at any depth or under another case\"), and the same paragraph puts that rule first and winning in both flows, so a statement that case is not normalized \"as today\" is false for it. Test strategy #7 scopes the claim correctly (\"case variants of `tests/`/`docs/` keep today's case-sensitive regex verdict\"), so the paragraph and the test disagree.","fix":"Scope it: \"case is not normalized for the `tests/`/`docs/` regexes, as today; the reserved-name rule is case-insensitive by design\"."},{"severity":"minor","location":"Risks, Ownership row mitigation — \"ADR-010's Consequences records the widened surfaces\"","problem":"new. The surfaces this row now lists (a light `plan` rewriting `docs/adrs/**`, `docs/prompts/**` or a sibling run's design directory unreverted) are added by this revision, and ADR-010's Consequences does not record them: it records stagePathAllowed gaining a flow parameter that widens the **coder's** write surface, the shared `NNN` space, promotion overwriting the plan, and the plan stage's small-model hazards. The plan stage's `docs/**` write surface appears nowhere in the ADR, and Rollout 4 leans on the same reference for the same purpose. The mitigation that defers the record to a document that does not carry it is the one a reader relies on.","fix":"Name the missing entry and when it lands: \"ADR-010's Consequences gains the plan stage's `docs/**` write surface (this row) when the ADR is accepted; today it records only the coder's widened surface.\""},{"severity":"nit","location":"Test strategy #7 — \"`plan` allows **every** path under `docs/` — `docs/designs/**`, `docs/adrs/**`, `docs/workflows.md`, a sibling run's directory\"","problem":"new. The same test's preceding clause denies \"every reserved name — `.pipeline.json`, the lock, `ESCALATION.md`, a `saveState` tmp … for `plan`\", and the checkpoint is `docs/designs/<slug>/.pipeline.json`, which is under `docs/`; with namesReservedSegment first, the two assertions are unsatisfiable together as literally written.","fix":"\"allows every **non-reserved** path under `docs/`\"."},{"severity":"nit","location":"Budget resolution, seat-reader paragraph — \"no call site keeps a `*_SPECIALTIES` import\"","problem":"new. The very next sentence says `states.ts:152` keeps `DESIGN_SPECIALTIES` (editorial only, Risks row 4), which is a call site that keeps one; Options also lists a surviving `CODE_SPECIALTIES` import as the rejected design's cost, so the absolute phrasing overstates the closure.","fix":"Scope it: \"no seat-panel reader keeps a `*_SPECIALTIES` import (`states.ts:152`'s editorial constant is the only one left)\"."},{"severity":"nit","location":"Rollout 6 — \"a run that escalates legitimately sits above it, at ≈ 9 sessions — plan 1, implement 1, the fix rounds, and at most the code phase's 6 cumulative (Budget resolution)\"","problem":"new. The named components sum to 10, and they cannot all be maximal at once: 6 cumulative code sessions is the pass-backstop shape, which dies inside the first round and spends no fix session (1+1+6 = 8), while the 2 fix rounds is the round-entry shape, which reaches only 4 code sessions (1+1+2+4 = 8). Both ceilings equal #12's `2 + seats × 3` = 8, so no pre-grant escalation sits \"above\" that bound; the --extend-budget grants are the only thing that carries a run past it, and the sentence lists them after.","fix":"State the bound as ≤ 8 pre-grant (each escalation shape saturates at 8, not 10) and name the grants as what takes a run above #12's bound."},{"severity":"nit","location":"State graph delta vs. Budget resolution — \"the deadlock stop … fires at 2 seats; with `maxRounds: 2` it is likelier to be the terminal record\" vs. \"`maxPassesPerRound: 3` is the record that prints on **every** shape of the round it starts\"","problem":"re-raised (r1c1 finding 6, partially closed). The added \"of the round it starts\" narrows the claim to a round, which is the half that was vague, but it does not exclude the shape the neighbouring sentence names: the deadlock stop returns its `deadlock:` blocked reason from the same blocked site (panel.ts:121-129) and is that round's terminal record, so the pass-count string is not printed there. The two sections still disagree on which record ends the phase.","fix":"Qualify: \"every pass-count shape that is not the deadlock stop (`panel.ts:121-129`)\"."},{"severity":"nit","location":"Options considered — \"**The ADR's rejected alternatives, adopted unchanged here.** … each rejected there on the ownership-matrix and session-count grounds\"","problem":"new. Delegating the record to ADR-010 is fine and the five names listed match the ADR's Alternatives exactly, but the summary of grounds is not true \"each\": \"No plan artifact at all\" is rejected on audit-record grounds (the seat stamp and the run record) and \"`lane` as the field name\" on a word-collision ground.","fix":"Drop the ground summary: \"each rejected there on its own grounds; this design adopts the ADR's choice without re-arguing it\"."}]}
```
