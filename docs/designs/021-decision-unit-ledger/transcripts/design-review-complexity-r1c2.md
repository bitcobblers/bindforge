# Transcript: design-review:complexity r1c2

## Provenance
- stage: design-review:complexity (round r1c2)
- agent: review/design-complexity
- model: strata/qwen3.8-flash-next-nvfp4-nvfp4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-14T22:28:23.844Z
- ended: 2026-11-14T22:34:30.535Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":26956,"output":20456,"reasoning":0,"cache.read":378269,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/021-decision-unit-ledger/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 94fdca92af8afeb042234edb2d53f4b6edd00143 e66564c0d7fdfd8c438ebf4e1c66d9f3ddd1644a` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/021-decision-unit-ledger/reviews/design/complexity-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "Chosen design ¶8, 021 bullet — \"the shipped `Rollout:` prose lines end with `021-C3`, `021-R1` and `021-R2`, and `021-R1`'s owner stays its shipped backticked `unassigned`, which ¶2 accepts\"",
    "problem": "new in the revised text (regression: the r1 wording — 'the Rollout paragraph re-set as a numbered list whose items 1–3 end with `021-C3`, `021-R1`, `021-R2`' — did the needed edit; the revision dropped it and asserted it was already true). The premise is false on the shipped text: docs/adrs/021-decision-unit-ledger.md:110–112 is one wrapped paragraph whose lines end with 'the product', '017–019' and 'adopt at landing.'; the ids sit mid-line and unbackticked ('backfills land per 021-R1; (3) the gate lands per 021-R2'), and `021-C3` does not appear in the Rollout text at all. ¶4 defines a handle as one trailing code span, so none of these is a handle. Chain: ¶8's template note block carries handles for C1–C3 and L1–L5 only; ¶7's exemption covers `021-R1` alone (a reservation carrying `unassigned`); so `021-R2` — 'Owner: this ADR's run', this run's own unit — finishes the conformance pass with no handle → `unit-unhandled` → Test #10 ('`checkLedgers` over **this repository** is green') red… [elided by the checkpoint cap]",
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
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/021-decision-unit-ledger.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/021-decision-unit-ledger/reviews/design/complexity-r1c2.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design ¶1 — \"so both readers share one regex and one CRLF posture, with a leading `\\uFEFF` dropped (A13)\"","problem":"new in this revision (the only moving part the diff adds) and it traces to no Goal — complexity checklist item 2. No Goal, risk or Deferred clause asks for BOM handling; A13's basis is CRLF only (gates.ts:54–57 matches `\\r?\\n`, and grep over `tools/orchestration/src` finds no `\\uFEFF`/BOM handling anywhere), so the citation points at a precedent that does not exist; Test strategy pins CRLF only (#3), never a BOM; the sole would-be consumer is a checkout on the platform the Compatibility matrix marks \"Not tested — expectation only\", since the corpus, the harness and CI produce BOM-free files on Linux. The clause also opens an offset the text does not resolve: `parseSource` derives its body by slicing the original content at `front[0].length` (gates.ts:57), so stripping inside the shared `frontmatterOf` puts that slice one character out unless `parseSource` is switched to the returned `body` — shared-code behavior plus a coupling, bought for input nothing in the design produces.","fix":"Delete the clause and inherit A13's CRLF-only posture: no other text changes and Test #3 is unchanged. If kept instead: state that `frontmatterOf` returns already-stripped content and that `parseSource` uses the returned `body` rather than slicing `content`, add a BOM case to Test #3, and cite a basis that shows a BOM case exists rather than A13."},{"severity":"nit","location":"Rollout 1 — \"Only `.ts` files land here, prettier-formatted, and no markdown enters the format leg's path (Test #9)\"","problem":"false as written: the leg is `format:check` → `prettier --check .` run from `tools/orchestration` (package.json:20,23; .github/workflows/check.yml:55–56), a path that already contains committed markdown (`README.md`, `prompts/*.md`), so markdown does enter the leg's path. The narrower claim the revision established is true and is what the conclusion needs: no committed ledger markdown lands there, because Test #9's fixtures live in `tmp-dir.ts`'s scratch (`mkdtempSync(join(tmpdir(), prefix))`, tmp-dir.ts:21 — /tmp, outside the leg) and ¶8's markdown sits in `docs/**`, which no leg formats (the other format step is `cargo fmt --all`, check.yml:29; there is no root package.json).","fix":"One clause: \"and no committed markdown fixture lands in the leg's path (Test #9)\". No design change."}]}
```
