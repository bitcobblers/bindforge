# Transcript: design-review:correctness r3c0

## Provenance
- stage: design-review:correctness (round r3c0)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:43:46.463Z
- ended: 2026-11-12T02:45:02.886Z
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
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "Budget resolution, \"Which knob binds in the light code phase\" — \"The session cap is checked at **pass entry**, ahead of that backstop (A16), so inside the first code round the backstop prints first.\"",
    "problem": "re-raised (partially closed). The false prediction \"so it never fires\" is gone, and the new cumulative-cap sentence is sound: reviewSessionsUsed counts every `code-review:` result (panel.ts:50-51), so the cap binds only at a pass entry in a later code round, which the `--stage code-review` force example reaches and #14(c) asserts. The causal clause is unchanged from r1c1 finding 2: read literally, \"cap checked at pass entry, ahead of the backstop\" predicts the session-budget string at spent 6, which #14(a) says never prints in that round. The reason the backstop wins inside the first round is upstream — decideNext calls planPass (states.ts:181) and escalates the blocked pass-backstop reason (states.ts:183) at the top of the drive-loop iteration, before the review branch reaches runner.ts:2357 — so the iteration at spent 6 ends in escalation and no pass entry happens at the cap.",
    "fix": "Replace the `so`-clause's basis: \"the pass backstop is decided upstream, in decideNext's planPass call (states.ts:181-183), so the iteration at spent 6 escalates on it and no pass entry happens at the cap.\""
  },
  {
    "severity": "minor",
    "location": "Ownership — \"so the `flow` parameter parses no path; case is **not** normalized — as today.\"",
    "problem": "re-raised (r1c1 finding 4 not applied). The revision rewrote the sentences around this clause (the A28 intake site) and re-asserted the blanket. namesReservedSegment lowercases each segment (types.ts:195-207), which is how the code comment at gates.ts:294-300 reads (\"no stage class … can own them at any depth or under another case\"), and the same paragraph puts that rule first and winning in both flows, so a statement that case is not normalized \"as today\" is false for it. Test strategy #7 scopes the claim correctly (\"case variants of `tests/`/`docs/` keep today's case-sensitive regex verdict\"), so the paragraph and the test disagree.",
    "fix": "Scope it: \"case is not normalized for the `tests/`/`docs/` regexes, as today; the reserved-name rule is case-insensitive by design\"."
  },
  {
    "severity": "minor",
    "location": "Risks, Ownership row mitigation — \"ADR-010's Consequences records the widened surfaces\"",
    "problem": "new. The surfaces this row now lists (a light `plan` rewriting `docs/adrs/**`, `docs/prompts/**` or a sibling run's design directory unreverted) are added by this revision, and ADR-010's Consequences does not record them: it records stagePathAllowed gaining a flow parameter that widens the **coder's** write surface, the shared `NNN` space, promotion overwriting the plan, and the plan stage's small-model hazards. The plan stage's `docs/**` write surface appears nowhere in the ADR, and Rollout 4 leans on the same reference for the same purpose. The mitigation that defers the record to a document that does not carry it is the one a reader relies on.",
    "fix": "Name the missing entry and when it lands: \"ADR-010's Consequences gains the plan stage's `docs/**` write surface (this row) when the ADR is accepted; today it records only the coder's widened surface.\""
  },
  {
    "severity": "nit",
    "location": "Test strategy #7 — \"`plan` allows **every** path under `docs/` — `docs/designs/**`, `docs/adrs/**`, `docs/workflows.md`, a sibling run's directory\"",
    "problem": "new. The same test's preceding clause denies \"every reserved name — `.pipeline.json`, the lock, `ESCALATION.md`, a `saveState` tmp … for `plan`\", and the checkpoint is `docs/designs/<slug>/.pipeline.json`, which is under `docs/`; with namesReservedSegment first, the two assertions are unsatisfiable together as literally written.",
    "fix": "\"allows every **non-reserved** path under `docs/`\"."
  },
  {
    "severity": "nit",
    "location": "Budget resolution, seat-reader paragraph — \"no call site keeps a `*_SPECIALTIES` import\"",
    "problem": "new. The very next sentence says `states.ts:152` keeps `DESIGN_SPECIALTIES` (editorial only, Risks row 4), which is a call site that keeps one; Options also lists a surviving `CODE_SPECIALTIES` import as the rejected design's cost, so the absolute phrasing overstates the closure.",
    "fix": "Scope it: \"no seat-panel reader keeps a `*_SPECIALTIES` import (`states.ts:152`'s editorial constant is the only one left)\"."
  },
  {
    "severity": "nit",
    "location": "Rollout 6 — \"a run that escalates legitimately sits above it, at ≈ 9 sessions — plan 1, implement 1, the fix rounds, and at most the code phase's 6 cumulative (Budget resolution)\"",
    "problem": "new. The named components sum to 10, and they cannot all be maximal at once: 6 cumulative code sessions is the pass-backstop shape, which dies inside the first round and spends no fix session (1+1+6 = 8), while the 2 fix rounds is the round-entry shape, which reaches only 4 code sessions (1+1+2+4 = 8). Both ceilings equal #12's `2 + seats × 3` = 8, so no pre-grant escalation sits \"above\" that bound; the --extend-budget grants are the only thing that carries a run past it, and the sentence lists them after.",
    "fix": "State the bound as ≤ 8 pre-grant (each escalation shape saturates at 8, not 10) and name the grants as what takes a run above #12's bound."
  },
  {
    "severity": "nit",
    "location": "State graph delta vs. Budget resolution — \"the deadlock stop … fires at 2 seats; with `maxRounds: 2` it is likelier to be the terminal record\" vs. \"`maxPassesPerRound: 3` is the record that prints on **every** shape of the round it starts\"",
    "problem": "re-raised (r1c1 finding 6, partially closed). The added \"of the round it starts\" narrows the claim to a round, which is the half that was vague, but it does not exclude the shape the neighbouring sentence names: the deadlock stop returns its `deadlock:` blocked reason from the same blocked site (panel.ts:121-129) and is that round's terminal record, so the pass-count string is not printed there. The two sections still disagree on which record ends the phase.",
    "fix": "Qualify: \"every pass-count shape that is not the deadlock stop (`panel.ts:121-129`)\"."
  },
  {
    "severity": "nit",
    "location": "Options considered — \"**The ADR's rejected alternatives, adopted unchanged here.** … each rejected there on the ownership-matrix and session-count grounds\"",
    "problem": "new. Delegating the record to ADR-010 is fine and the five names listed match the ADR's Alternatives exactly, but the summary of grounds is not true \"each\": \"No plan artifact at all\" is rejected on audit-record grounds (the seat stamp and the run record) and \"`lane` as the field name\" on a word-collision ground.",
    "fix": "Drop the ground summary: \"each rejected there on its own grounds; this design adopts the ADR's choice without re-arguing it\"."
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
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/correctness-r3c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Assumption register A12 — \"no document-stage primary agent reads, writes or shells — `designer.md` and `researcher.md` both deny `transcripts/**` (read and edit) and `shell: \"*\"`\"","problem":"new (added by this revision). The clause is stated as **verified** and is false as written; the cited basis covers only two of the four document-stage primary agents. `STAGE_BASES` (stages.ts:115-150) puts four primary agents in the document-stage class that gates.ts:314 names (\"research, proposal*, reviews, doc-sync\"): research/researcher, design/designer (proposal, proposal-revise), design/editor (proposal-final, stages.ts:122-126), ops/doc-sync (doc-sync, stages.ts:150). ops/doc-sync.md:20-25 denies shell then **allows** `shell: \"git diff*\"`, mode primary — so a document-stage primary agent does shell today. The universal claim in a `verified` row is contradicted by the repo, and doc-sync.md is not in the basis column; the Revision log's r2 entry makes A12 the carrier of the planner's shape, so the register is the surface a reviser relies on. Fix: scope the clause to the agents the basis names and record the exception.","fix":"Scope it and complete the basis: \"no document-stage primary agent reads transcripts, writes transcripts, or shells beyond read-only git — designer.md and researcher.md deny `docs/designs/**/transcripts/**` (read and edit) and `shell: \"*\"` verbatim; design/editor.md:4-22 is identical; ops/doc-sync.md:23-25 is the one exception, an allowed `git diff*`\" — adding editor.md and doc-sync.md to the basis column."},{"severity":"minor","location":"The `plan` stage, planner bullet — \"No document-stage agent has either privilege today.\"","problem":"new (same revision as the A12 clause). The sentence asserts the privilege pair is absent across all document-stage agents, but ops/doc-sync.md:23-25 gives the doc-sync primary agent a shell allow (`git diff*`), so the comparative claim is false. The bullet's operative decision is unaffected — \"`design/designer`'s permission list verbatim\" is the right shape — so this is the added comparative sentence, not the design.","fix":"Name the exception: \"Designer, editor and researcher deny transcripts (read and edit) and shell; `ops/doc-sync` is the one document-stage agent with a shell allow (`git diff*`), and `planner.md` copies the designer list, not that one.\""}]}
```
