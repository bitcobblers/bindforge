# Transcript: design-review:complexity r1c1

## Provenance
- stage: design-review:complexity (round r1c1)
- agent: review/design-complexity
- model: strata/qwen3.8-flash-next-nvfp4-nvfp4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-14T21:59:33.875Z
- ended: 2026-11-14T22:09:52.323Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":55636,"output":29413,"reasoning":0,"cache.read":682744,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/021-decision-unit-ledger/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 67e7b5740b46b427129312d4aa93e403e2ef5d2e 94fdca92af8afeb042234edb2d53f4b6edd00143` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/021-decision-unit-ledger.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/021-decision-unit-ledger/reviews/design/complexity-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "Chosen design ¶8, `docs/adrs/021-decision-unit-ledger.md` bullet (with Options \"`021-R1`'s owner stays `unassigned`. Rejected\" and Deferred) — \"`021-R1`'s owner becomes `ADR-022 (reserved)`, 021-L5's legal form for 'not written yet' (A18), with a one-line `Reserved: 022` note in its text\"",
    "problem": "Nothing but the design's own owner grammar requires this moving part, and it reserves exactly the number the allocator hands out next. 021-L5 as shipped admits two forms — \"an existing or reserved ADR number, or this ADR's own run — or carries an explicit `unassigned` marker\" — and 021-R1 already carries one (docs/adrs/021-decision-unit-ledger.md:101, \"Owner: `unassigned` — a rollout task, not an ADR slot\"). The re-mark is needed only because ¶2 accepts the bare `Owner: unassigned` and so rejects the shipped backticked spelling, which test #5 states outright (\"`owner-missing` for `021-R1`'s pre-run `unassigned` text with no marker\"); the Options rebuttal \"021-L5's whole point is a named owner\" therefore misstates the law it cites, which is why this rejected finding is re-reported. The cost is concrete: it rewrites the run's own source ADR's Decision against that unit's \"a rollout task, not an ADR slot\", moves the four product backfills and their order to \"ADR-022 (reserved, 021-R1's ne… [elided by the checkpoint cap]",
    "fix": "Accept the shipped spelling: ¶2's owner rule reads `Owner: unassigned` backticked or bare. Then ¶8 drops the R1 re-mark, the `Reserved: 022` note and the ADR-022 reservation; Deferred keeps pointing the backfills at 021-R1's `unassigned` rollout task as the ADR words it; test #5's first case becomes a clean case. If a named owner is wanted anyway, reserve a number the allocator cannot reach (≤ 021) and list the R1 re-mark under Goals with its reason."
  },
  {
    "severity": "minor",
    "location": "Chosen design ¶8, template bullet — \"`docs/adrs/template.md`: frontmatter gains `series:`, `depends-on:`, `ledger: yes`\"",
    "problem": "Only `ledger: yes` traces to a Goal (\"`ledger: yes` as the machine-readable marker of a ledger-bearing ADR\"); the other two are knobs with no consumer. No code in tools/orchestration/src reads `series` (grep: no match), and in the corpus it is a literal label — all five series ADRs spell `series: 011-015` — so the template would have 017–019, which Rollout 4 makes ledger-bearing at birth, declare a series they are not in. `depends-on:` is read by ¶7's `depends-order`, but 021-L3 makes it optional (\"Where a `depends-on` key is present\"), so a template default serves nobody.",
    "fix": "The template gains `ledger: yes` only; if the optional keys should be visible at all, use the template's existing commented style for `depends-on:` and leave `series:` out."
  },
  {
    "severity": "minor",
    "location": "Chosen design ¶3 (Amendments) with ¶7's taxonomy — \"No shipped unit uses either (research, Risks), so the check is symmetry, not a workflow\"",
    "problem": "Four of the sixteen codes (`amends-dangling`, `amends-backlink`, `amends-order`, `unit-id-reuse`), the `amends`/`amended-by` parsing and all of Test strategy #8 serve a notation with zero corpus instances, and none of the four classes 021-R2 names includes it; the source ADR explicitly leaves \"the gate's exact error taxonomy\" to this run, so deferring it is this design's call to make.",
    "fix": "Keep one sentence in ¶3 defining the notation 021-L4 asks for, and move the symmetry checks to Deferred (\"Owner: the first amendment that carries `amends`\"), dropping the four codes from ¶7's list (twelve findings, not sixteen) and deleting test #8. The same two readers and two hook points remain; #11 can only exercise shapes the corpus ships, so no goal loses coverage."
  },
  {
    "severity": "minor",
    "location": "Chosen design ¶4 — \"A handle is the maximal trailing run of backticked unit ids on a source line: one or more code spans, separated by single spaces\"",
    "problem": "The \"one or more code spans\" clause has no consumer: every shipped handle is a single trailing span, and the multi-id case sits inside one span (`013-B1 013-B2 013-B3 013-B4`, spec.md:111); test #2 pins only one multi-id span and one cell-final span, never two spans on a line. The clause is a grammar branch no fixture exercises and it makes \"maximal trailing run\" ambiguous for a span-text-span line.",
    "fix": "Define a handle as one trailing code span whose content is ids separated by single spaces, keeping the trailing-spaces-and-one-`|` allowance; C2's \"a line may carry several\" is then satisfied inside the span and #2 needs no change."
  },
  {
    "severity": "nit",
    "location": "Chosen design ¶7 — \"denies with the first twelve findings as `file:line code detail` lines plus `+N more` — bounded the way `boundedLegs` bounds a gate record\"",
    "problem": "The bound is an unnamed literal stated twice (¶7 and test #10) while the repo exports `MAX_GATE_LEGS = 32` for exactly this and routes every other bounded list through it (boundedLegs at gates.ts:457, boundedSurfacePaths/elisionMarker at runner.ts:1881–1892), so the two statements can drift.",
    "fix": "Name one exported constant (or reuse `MAX_GATE_LEGS`) and have ¶7 and #10 cite the constant instead of the literal \"twelve\"."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/021-decision-unit-ledger.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/021-decision-unit-ledger/reviews/design/complexity-r1c1.md. That file is the only path this session
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
{
  "verdict": "request-changes",
  "findings": [
    {
      "severity": "major",
      "location": "Chosen design ¶8, 021 bullet — \"the shipped `Rollout:` prose lines end with `021-C3`, `021-R1` and `021-R2`, and `021-R1`'s owner stays its shipped backticked `unassigned`, which ¶2 accepts\"",
      "problem": "new in the revised text (regression: the r1 wording — 'the Rollout paragraph re-set as a numbered list whose items 1–3 end with `021-C3`, `021-R1`, `021-R2`' — did the needed edit; the revision dropped it and asserted it was already true). The premise is false on the shipped text: docs/adrs/021-decision-unit-ledger.md:110–112 is one wrapped paragraph whose lines end with 'the product', '017–019' and 'adopt at landing.'; the ids sit mid-line and unbackticked ('backfills land per 021-R1; (3) the gate lands per 021-R2'), and `021-C3` does not appear in the Rollout text at all. ¶4 defines a handle as one trailing code span, so none of these is a handle. Chain: ¶8's template note block carries handles for C1–C3 and L1–L5 only; ¶7's exemption covers `021-R1` alone (a reservation carrying `unassigned`); so `021-R2` — 'Owner: this ADR's run', this run's own unit — finishes the conformance pass with no handle → `unit-unhandled` → Test #10 ('`checkLedgers` over **this repository** is green') red → Rollout 4's '`npm run check` green is the acceptance evidence' unreachable, and the Goal 'The tree green at merge: 021's ten units each carry a `→` and a handle' unmet. A17 holds (grep over docs/adrs matches only 013 and 021), so 021's Rollout prose is the only place `021-R2` can be handled.",
      "fix": "Keep the r1 simplification (no re-mark, no ADR-022) and restore only the handle edit in the same bullet: 'the Rollout paragraph's three lines are re-wrapped so they end with `021-C3`, `021-R1` and `021-R2`' (backticked, line-final per ¶4). One line of ¶8, no new mechanism, and Test #10's green claim becomes true."
    },
    {
      "severity": "minor",
      "location": "Chosen design ¶2 — \"A reservation's owner is the text from `Owner:` to the end of the line, backticks stripped, so `ADR-017 (reserved)`, `unassigned` — bare or backticked … all parse\" with \"an owner matching neither `ADR-\\d{3}`, `unassigned` nor `this ADR's run` is `owner-unknown`\"",
      "problem": "The capture extent and the acceptance test differ and nothing reconciles them. `021-R1`'s shipped line is \"Owner: `unassigned` — a rollout task, not an ADR slot.\" (ADR:101), so the captured text is 'unassigned — a rollout task, not an ADR slot', which matches none of the three forms exactly; yet Test #5 requires it clean ('`unassigned` with `021-R1`'s shipped trailing prose') while requiring `owner-unknown` for `Owner: someday`. Acceptance is therefore an unstated prefix/leading-form match — and under a containment reading it is over-broad: 'Owner: ADR-017, ADR-019' or 'unassigned forever' would pass. The rule is wider than any shipped line needs: in the corpus the owner form is always what immediately follows `Owner:` (013-R1–R4 `ADR-017 (reserved).`, 021-R1 backticked `unassigned`, 021-R2 `this ADR's run.`).",
      "fix": "State one rule at one extent: 'the owner is the form immediately after `Owner:`, backticks stripped: `ADR-\\d{3}` | `unassigned` | `this ADR's run`; whatever follows it on the line is prose; text after `Owner:` that begins with none of the three is `owner-unknown`.' Same code, no extra branch; Test #5 unchanged and now unambiguous."
    },
    {
      "severity": "nit",
      "location": "Rollout 1 / Test strategy #9 — \"Only `.ts` files land here, so the format leg is untouched\" and \"no committed fixture tree, so prettier never rewrites a deliberately malformed ledger\"",
      "problem": "The stated reason is inverted. The leg is `format:check` → `prettier --check .` run from `tools/orchestration` (package.json:20,23; .github/workflows/check.yml:54–56), so it does check the landing `.ts` files, and it never sees the corpus (`docs/**`, `schemas/**` are outside that directory). The narrower, true reason is that committed `.md` fixtures would sit under the leg's path — `tests/pipeline/fixtures/` is the repo's existing fixture convention (fixtures/checkpoints/*.json) — and prettier owns markdown table padding and line-final whitespace, the exact shape Test #2 pins.",
      "fix": "Reword both clauses to that reason: fixtures are built in the test module and written to the scratch repo of the existing `tests/pipeline/tmp-dir.ts` helper, so no committed markdown under `tools/orchestration` can be reformatted out from under a shape pin. No design change."
    }
  ]
}
```
