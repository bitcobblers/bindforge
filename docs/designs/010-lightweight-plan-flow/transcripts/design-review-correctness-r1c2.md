# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:05:07.510Z
- ended: 2026-11-12T02:07:22.358Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 24b804a9c29888a3b458242d27c370d6b48be4fb 4cc9accad12add4e888d0ca7be434acf93530a29` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Budget resolution, \"Which knob binds in the light code phase\" + Test strategy #14(a) — \"both seats dissenting admits a 2-seat pass at 5 spent and closes the phase at 7 — the shape where `maxReviews.code: 6` binds, on the session-budget record\"",
    "problem": "The light flow cannot reach that record, and 7 is not reachable in that shape. reviewSessionsUsed counts per-seat sessions (panel.ts:50-52, results entries `code-review:<specialty>`), and a dissent keeps the same round (states.ts:83: `review('code', open.length > 0 ? state.codeRound : state.codeRound + 1)`), so the both-seats shape is one round of three passes: 2+2+2 = 6 sessions, with pass entries at spent 0, 2, 4 — `spent >= cap` (runner.ts:2357-2359) never fires and spent is always even, so \"5 spent\" and \"closes at 7\" are unreachable. The next planPass returns {kind:'blocked'} at `cycle >= cfg.maxPassesPerRound` (panel.ts:113-119), which decideNext escalates (states.ts:183): the terminal record is `code review: pass backstop: 3 review passes already run in this code round (cap 3 passes per round) — the funnel is not converging` at 6 sessions. With 2 seats a round's 3 passes cost exactly the cap, so maxPassesPerRound: 3 prints on every shape (6 sessions both seats dissenting, 4 with … [elided by the checkpoint cap]",
    "fix": "Pick one and name the mechanism. (a) Keep maxPassesPerRound: 3 and rewrite the paragraph to: the pass-backstop record prints at 6 sessions (both seats dissenting) and 4 (one dissenter); maxReviews.code: 6 is the ceiling it coincides with, not the string an operator reads — and make #14(a) assert the pass-backstop string at 6 sessions with the light suffix. (b) If the session-budget string is meant to be the light cap's record (ADR-010: \"Budget exhaustion … the reason names the flow\"), set light maxPassesPerRound: 4, so a pass entry at spent 6 hits `spent >= cap` and prints `code review session budget exhausted (6/6 sessions)`; then state the per-round band as 4–8 sessions and drop \"closes at 7\" (it needs a 1-seat pass leaving 5 spent)."
  },
  {
    "severity": "minor",
    "location": "Budget resolution, \"`validated` is part of the contract\" — \"must keep `planMaxLines` a positive integer\" and \"must carry a non-empty seat list for every phase the flow's `stages` runs\"",
    "problem": "Both clauses have no true reading for the full flow's own resolved config, which the same section promises is untouched. planMaxLines is a new optional field absent from DEFAULT_CONFIG (stages.ts:75-84) and from the budget table (\"absent (no gate)\"), so a resolved full config cannot make it \"a positive integer\", while Test #5 asserts the absent case works. The seat clause is keyed on the flow's stages, and full.stages runs `editorial-review`, while reviewSeats is keyed { design, code } and ReviewPhase includes 'editorial' (states.ts:13, called with 'editorial' at runner.ts:2322-2345): the editorial phase has no seat-list key to validate, and the specialtiesFor delegate hands seatsFor exactly that value.",
    "fix": "Write \"absent, or a positive integer\", name the phase mapping in seatsFor (editorial reads reviewSeats.design — today's `phase === 'code' ? CODE : DESIGN` shape, panel.ts:31-32), and scope the non-emptiness clause to the design/code keys."
  },
  {
    "severity": "minor",
    "location": "State graph delta, \"`case 'code-review'` backstop exhausted | escalate, naming the revise-round backstop\", and Budget resolution, \"`maxRounds: 2` bounds the mutating `fix` loop, whose record is the revise-round backstop\"",
    "problem": "The code phase has two escalation strings and the revision renamed the one this row reaches. From decideNext's `case 'code-review'` the string is `… and the fix-round backstop is exhausted (2/2)` (states.ts:185-194) — r1's wording, which was correct; `code revise-round backstop reached (2/2 rounds)` comes from the runner's round-entry check (runner.ts:2382-2395). The row promises \"same message plus `the light flow (flow: light); promote …`\", so which sites carry the light suffix is now unstated, and Test #8 asserts only \"escalation naming the light flow\".",
    "fix": "Restore \"naming the fix-round backstop\" in the row, and state that the light suffix is appended at both code-phase escalation sites (states.ts:187-194 and the runner's round-entry escalation), naming the two strings."
  },
  {
    "severity": "minor",
    "location": "Ownership — \"the new name `plan` is governed by the **document-stage** rule in both flows — writes confined to the run's `docs/designs/NNN-<slug>/` paths\"",
    "problem": "The cited rule is `return inDocs` with `inDocs = /^docs\\//` (gates.ts:304,314-315): it confines a document stage to any path under docs/, which is how doc-sync writes docs/workflows.md. A light `plan` can therefore write `docs/adrs/…`, and Test #7 (\"`plan` allows the run's `docs/designs/**` and denies `src/**`\") does not assert the run-directory confinement the paragraph claims, so the mitigation stated for Risks row 3 is narrower in reality.",
    "fix": "Say the document-stage rule is `docs/**` and that the run's own directory is enforced by the template/agent config, not the predicate — or name the predicate change that narrows document stages to the run's directory and add that case to Test #7."
  },
  {
    "severity": "minor",
    "location": "Ownership + Test strategy #7 — \"separator and case variants are settled before the predicate sees them\" / \"a path spelled with a foreign separator or a case variant arrives normalized and repo-root-relative, so it gets the verdict of its canonical spelling\"",
    "problem": "Only the reserved-name rule is case-insensitive (namesReservedSegment, types.ts:195-207); TEST_PATH_RE (gates.ts:286) and inDocs (gates.ts:304) are case-sensitive, so `Tests/x.test.ts` is allowed under `full` where the canonical spelling is denied, and `Docs/designs/plan.md` is denied where the canonical spelling is allowed. The separator half is sound: paths reach the predicate from `git diff-tree --name-only -z` and dirtyFiles (runner.ts:261-278), POSIX and repo-root-relative.",
    "fix": "Keep the separator claim, drop \"a case variant … canonical spelling\" from the guarantee and from #7, and record that case variants of the tests//docs/ prefixes are settled by the case-sensitive regexes, exactly as today."
  },
  {
    "severity": "nit",
    "location": "Chosen design — \"`configFor('full', base)` is deep-equal to a **complete** `base`\"",
    "problem": "The qualifier appears only here; the Compatibility row, Rollout 1 and Test strategy #3 all say plain \"deep-equal to `base`\", and #3 is the assertion, so an implementer who passes a partial literal gets a failure the design declares impossible.",
    "fix": "Qualify Test strategy #3 (\"a complete `PipelineConfig` literal\"); the Compatibility and Rollout mentions can stay as they are."
  },
  {
    "severity": "nit",
    "location": "Rollout 6 — \"check them against the derived bound — ≤ 8 sessions, ≤ 96 min at the 12-min stage bound in the serial default\"",
    "problem": "That is Test #12's converging-path bound (`total sessions ≤ 2 + seats × 3`), not a number derived from the declared budgets: a light run that goes to the ceiling (plan + implement + 2 fix + 6 review sessions) lands at 10–12 sessions and 120–144 min, so the first light run that legitimately escalates reads as missing the bound.",
    "fix": "Name it the converging-run bound and cite #12's formula, keeping 96 min as the wall-clock ceiling for that path."
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
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "Budget resolution, \"Which knob binds in the light code phase\" — \"The session cap is checked at **pass entry**, ahead of that backstop (A16), so it never fires\"", "problem": "new. The conclusion is right, the stated mechanism is not, and the stated mechanism predicts the opposite string. The cap check (runner.ts:2357-2359) is ahead of the in-loop planPass (runner.ts:2422), so read literally the sentence says the session-budget record wins at spent 6 — the exact thing Test strategy #14(a) asserts never prints. The reason it never fires is the other half of A16: decideNext calls planPass at states.ts:181 and escalates the blocked pass-backstop reason at states.ts:183, and decideNext runs at the top of every drive-loop iteration, before the review branch's cap check, so the iteration at spent 6 ends in escalation and no pass entry ever happens at the cap. The paragraph's arithmetic (4–6 band, 6/4 sessions, cap never firing) is verified against the code; only its causal clause is wrong.", "fix": "Replace \"ahead of that backstop (A16), so it never fires\" with: the pass backstop is decided upstream, in decideNext's planPass call (states.ts:181-183), before the loop reaches the pass-entry cap check (runner.ts:2357), so the iteration at spent 6 escalates on the backstop and no pass entry happens at the cap."}, {"severity": "minor", "location": "Budget resolution, \"`validated` is part of the contract\" + Test strategy #23 — \"must keep the two keys `seatsFor` reads non-empty — `reviewSeats.code`, and `reviewSeats.design`\" / \"`reviewSeats.code: []` and an empty `reviewSeats.design` throw\"", "problem": "re-raised (part of r1c1 finding 2; partially closed — the mapping is named, planMaxLines is now absent-or-positive). The light partial in the budget table sets reviewSeats.design to [], so every light resolved config has an empty design list. #23 carries no flow qualifier, so an implementer who resolves configFor('light', …) and asserts the throw gets a failure the same section declares legal (\"`light`'s `design: []` is legal only because `light.stages` names no design stage\"). The legality clause gives a reason, not the mechanism the validation keys on.", "fix": "Key the clause on the flow's stages — \"`reviewSeats.design` non-empty for a flow whose `stages` run a design or editorial stage (full's proposal-final → editorial reads it through states.ts:25-26), exempting `light`'s registry `[]`\" — and scope #23's design assertion to a `full`-flow resolution."}, {"severity": "minor", "location": "State graph delta — \"The light suffix is appended to every code-phase escalation the light flow reaches: `decideNext`'s pass-backstop string (`states.ts:183`) and fix-round string (`states.ts:187-194`), and the runner's round-entry `code review revise-round backstop reached (2/2 rounds)`\"", "problem": "new. \"Every\" and the three-site list disagree: a fourth code-phase escalation is reachable in light and is not listed — escalate('tests not green at the code-review budget limit') at states.ts:170-178, taken from case 'implement'/case 'fix' with testsGreen: false once codeBudgetUsed >= maxRounds. Light is the flow where that stop is likeliest (no test engineer, A13/A14), and ADR-010's Status section requires an escalated light run to read as \"too big for the light flow\". An implementer who does the three named sites leaves that escalation naming no flow. The three quoted strings themselves are verbatim correct.", "fix": "Add that string to the enumeration, naming states.ts:172,178, or narrow the sentence to \"the three review escalations\" and add a row for the red-tests limit."}, {"severity": "nit", "location": "Ownership — \"Paths reach the predicate repo-root-relative and POSIX-normalized (A8/A9), so the `flow` parameter parses no path; case is **not** normalized — as today.\"", "problem": "new. As a blanket statement this is false for the rule the same paragraph puts first: namesReservedSegment lowercases each segment (types.ts:195-207), which is how the code comment at gates.ts:294-300 reads (\"no stage class … can own them at any depth or under another case\"). The narrowing that closed r1c1 finding 5 overreached from tests//docs/ to all of the predicate.", "fix": "Scope it: \"case is not normalized for the `tests/`/`docs/` regexes, as today; the reserved-name rule is case-insensitive by design\"."}, {"severity": "nit", "location": "Ownership vs. The `plan` stage — \"confining `plan` to the run's own directory is template and agent-config work, not predicate work\" vs. \"edits `docs/designs/**`\"", "problem": "new. The named agent config grants every run's directory, so the confinement the closure points to as agent-config work is not implemented by the config the design specifies; Risks row 3's mitigation stays narrower than the paragraph implies.", "fix": "Say the planner's grant is `docs/designs/**` and that run-directory confinement is only the template's `{{designDir}}` output path (no predicate, no agent rule), or narrow the planner grant to the run's directory."}, {"severity": "nit", "location": "Budget resolution vs. State graph delta — \"`maxPassesPerRound: 3` is the record that prints on **every** shape\"", "problem": "new. The State graph delta says of the same phase that the deadlock stop \"is seat-count-agnostic and fires at 2 seats; with `maxRounds: 2` it is likelier to be the terminal record\", and planPass returns that `deadlock:` blocked reason from the same blocked site (panel.ts:121-129), so the pass-count record is not the terminal record on that shape; the two sections disagree on which record ends the phase.", "fix": "Qualify to \"every pass-count shape that is not the deadlock stop (`panel.ts:121-129`)\"."}]}
```
