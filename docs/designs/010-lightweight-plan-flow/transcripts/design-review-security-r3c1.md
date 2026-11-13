# Transcript: design-review:security r3c1

## Provenance
- stage: design-review:security (round r3c1)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T02:53:48.445Z
- ended: 2026-11-12T02:55:02.813Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 3. This pass's scope: the sections changed since you last reviewed: run `git diff 92f71e02b5d66c6da627a93779d50ed5decc402d 4635f0dc4efca2cf5a04189d48c3674ddcf07b50` (both are blob ids of 02-proposal.md), then read the full current document.
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
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Budget resolution — one canonical rule: \"A violation throws as a usage error at the config-read seam, as a bad `flow` value does.\"",
    "problem": "Regression introduced by this revision. The round-3 text pinned the throw at \"the bad-`flow` posture\", which the document defines (Selecting the flow) as pre-lock and \"never a half-started run: no lock, no design directory, no session\". The new site is placed after the lock by the document's own sequence: configFor's first argument is state.flow (flowOf(state)), and state.flow is set \"immediately after `freshState`, so the entry save carries it (`runner.ts:2102`)\", with ADR-010 resolving budgets \"from `state.flow` inside `runPipeline` and `resumePipeline`\". So a caller-supplied opts.cfg with reviewSeats.code: [] — the exact fail-open the r1 security closure exists for (seatsConverged vacuously true, code-review converging with zero sessions) — throws after the lock, the design directory and the entry checkpoint save, i.e. as a partially-applied run, not as the clean usage error the sentence claims equivalence with. Security checklist 7: malformed input must reject cleanly, never partia… [elided by the checkpoint cap]",
    "fix": "State that the caller-supplied panel is validated at the same pre-lock posture as the `flow` value — validate opts.cfg through the existing `validated` rule before the lock and before `freshState` — and extend Test strategy #17 to a cfg with `reviewSeats.code: []`, asserting no lock file, no design directory, no entry checkpoint, and zero client calls. One sentence plus one test case; no new mechanism."
  },
  {
    "severity": "minor",
    "location": "Assumption register A12 / The `plan` stage: \"no document-stage primary agent reads, writes or shells — `designer.md` and `researcher.md` both deny `transcripts/**` (read and edit) and `shell: \"*\"`\"",
    "problem": "The clause added in this diff states a universal over document-stage primary agents and marks it verified, but the Basis cites two agent files. The design's own flow description (Context, and ADR-010's `full` stage list) names further document-stage primary agents — the editor and the doc-sync stage — whose configs are not cited, so the enumeration supporting the claim is incomplete as written. It matters because that claim is what licenses copying designer's permission list onto the planner: if any document stage outside the two cited files holds shell, the sentence is false and an implementer working from the summary rather than designer.md:4-22 has cover to grant the planner shell.",
    "fix": "Cite every document-stage primary agent in A12's Basis, or narrow the claim to the evidence it carries: \"neither `designer.md` nor `researcher.md` grants transcripts access or shell\"."
  },
  {
    "severity": "nit",
    "location": "The `plan` stage: \"`design/designer`'s permission list verbatim — `mode: primary`, edits `docs/designs/**`, and denies `.pipeline.json`, `transcripts/**` for read and edit, and `shell: \"*\"` (A12)\"",
    "problem": "A12's summary of that same permission list omits `.pipeline.json`, so the planner's shape is stated twice with different entries; a reader implementing from A12 alone misses one denial.",
    "fix": "Add `.pipeline.json` to A12's parenthetical so the one stated authority lists all four entries."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/security-r3c1.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "nit", "location": "Assumption register A12 — \"all four document-stage primary agents — `researcher`, `designer`, `editor` (`proposal-final`), `doc-sync` — deny `.pipeline.json`, `transcripts/**` (read and edit) and `shell: \"*\"`; `doc-sync.md:23-25` is the one exception, an allow for `git diff*`\"", "problem": "new (this diff). The universal and the exception clause collide on a literal reading: the sentence asserts doc-sync denies `shell: \"*\"` and then names doc-sync \"the one exception\". Two readings of doc-sync.md:20-25 stay open to a reader of the register alone — a wildcard deny plus one narrow allow (universal holds, only `git diff*` added), or an allow map with no wildcard deny (universal false for doc-sync, exception clause covering only `git diff`). A12 is the document's one stated shape authority for the planner's permission list, so that shape should not be readable two ways. Impact is limited: The `plan` stage pins the operative claim — `designer.md:4-22` verbatim, and \"`ops/doc-sync`'s `git diff*` allow is not part of it\" — so planner-as-specified has no shell, and doc-sync is not a light stage. My r2 minor (incomplete enumeration) is closed; the enumeration now names all four document-stage primary agents, each cited.", "fix": "Make the denied thing the wildcard key and the exception additive: \"deny `.pipeline.json`, `transcripts/**` (read and edit) and the `shell: \"*\"` wildcard; `doc-sync.md:23-25` adds one allow, for `git diff*`\"."}, {"severity": "nit", "location": "Budget resolution — \"a caller-supplied `opts.cfg` is validated **pre-lock** — `validated` runs on it in `runPipeline`/`resumePipeline` once the flow is known, before `acquireLock` and `freshState`\"", "problem": "new (introduced by the r3 closure). The subject of the pre-lock check is the bare caller value (\"runs on it\"), while the rule enforced is scoped to the resolved config: non-emptiness reads \"keep `reviewSeats` non-empty for every phase the flow's `stages` runs\", and a CLI-shaped `opts.cfg` (`{ ...DEFAULT_CONFIG, maxParallelReviews }`) carries no `reviewSeats` at all (A3 makes the field new and optional), so on the bare value the check is vacuous for absence and binding only for a caller who supplies a panel. The fail-open itself is closed under either reading — `reviewSeats.code: []` is present-and-empty, so Test strategy #17's assertions (no lock file, no design directory, no entry checkpoint, no `ESCALATION.md`, zero client calls) hold — but two call sites of `validated` now exist with an unspecified subject, and the pre-lock check is meant to be the same predicate `configFor` runs at the read seam. Residual, not a finding: a concurrent `--force` promotion between the pre-lock flow read and `acquireLock` can move the validation's scope, leaving the read seam's throw reachable, which is the design's stated authority for a running run.", "fix": "Name the resolved value as the subject, one clause: \"the resolved `configFor(flow, opts.cfg)` is computed pre-lock, the same call the read seam makes\"."}]}
```
