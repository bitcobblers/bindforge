# Transcript: code-review:compatibility r1c5

## Provenance
- stage: code-review:compatibility (round r1c5)
- agent: codereview/code-compatibility
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T02:17:42.590Z
- ended: 2026-11-13T02:27:44.615Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the ratification pass c5: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff eb8b40ad413a397f71b977f6d0dcd6c903f0d17c..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/010-lightweight-plan-flow.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "docs/workflows.md:235,251",
    "problem": "The change's per-flow budget table and defaults list both document the full flow's stageTimeoutMs as 30 min, naming tools/orchestration/src/pipeline/stages.ts as the authority, while this diff ships stageTimeoutMs: 90 * 60_000 (stages.ts:108). The sibling knob in the same table was corrected to the shipped value (maxPassesPerRound 8 at lines 232/243 vs stages.ts:105), so the timeout row is the one the doc pass missed. The proposal's budget table and its matrix row 'Existing ADRs and briefs … budgets unchanged' describe the 30-minute flow, so an operator sizing a full run computes a stage bound three times smaller than the runner applies. The light row's 12 min (flows.ts:79) is correct.",
    "fix": "Write 90 min in docs/workflows.md:235 and :251, noting that light overrides it to 12 min through configFor, and record the deviation so A25's 12-minute adequacy is compared against the shipped full default."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/gates.ts:56,59 (with runner.ts:2083)",
    "problem": "The frontmatter scan is anchored ^---, so a UTF-8 byte-order mark at byte 0 makes front null: parseSource returns status 'none' and flow 'full', and the pre-lock check reads that same invented 'full' and passes. For a brief the body is non-empty, so sourceAdmissible admits it and a document stating flow: light silently runs the twelve-stage five-seat path — the exact outcome the matrix's bad-value row and Test strategy #6 forbid ('never silently full'). It is reached through an encoding spelling the matrix never considered, and the matrix's CRLF row shows the scan was built for exactly this class of authored file.",
    "fix": "Strip a leading U+FEFF from content as the first line of parseSource and scan the stripped value (this also fixes status: for BOM-authored ADRs), and add a light-gates case: a flow: light brief with a leading BOM resolves as light."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:3086",
    "problem": "statusLines renames the full flow's status label from run: to slug:. Test strategy #10 requires that every existing line is byte-equal for a full state, and the matrix's rows for existing checkpoints, ADRs and briefs claim today's behavior; this line is in the full flow's output, so the rename reaches runs whose flow was not selected. The flow line at 3082-3084 is the additive part the design asked for; the rename is not, and no test asserts the label set (nothing in tests/ matches run: or slug:), so the byte-equality promise is unasserted and the same class of rename passes again.",
    "fix": "Print the identity under the label the full flow already used, keeping the flow line additive, and assert the exact full-flow status block (at minimum its label column) in the statusLines test, which is what #10 asks for."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/package.json:23 (.github/workflows/check.yml:52-54)",
    "problem": "Compatibility checklist item 7 (Test strategy #21, matrix linux row) claims npm run check stays green on linux at the engines Node floor; that leg could not be evaluated in this review, which had no shell, so lint, format:check and vitest were never executed. The platform half is verifiable and true (the orchestration CI job runs npm run check on ubuntu-latest pinned to node-version 22.23.4, and no new test carries a runIf/skipIf), and the failures recorded mid-run look closed at HEAD — the seven light-*.test.ts files end at their final }); with no trailing blank line, gates.test.ts:738-751 is in prettier's shape — so the claim is plausible, but the change carries no evidence for it.",
    "fix": "Run npm run check in tools/orchestration at Node 22.23.4 on linux and record the exit status and vitest count in the run's build notes, so the compatibility row rests on a datum; the CI job is the standing proof afterwards."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/code/compatibility-r1c5.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict":"approve","findings":[{"severity":"minor","location":"tools/orchestration/src/pipeline/report.ts:290,298 (with runner.ts:3277-3278)","problem":"The plan-stop repair added in this pass interpolates the checkpoint's source path unquoted: `pipeline run ${source} --force`. That path is the document's real repo-relative path (runner.ts:2224) and is never sanitized — only the identity's slug is (identity.ts:34-45). A brief whose name contains a space, apostrophe or shell metacharacter is a legal light entry (sourceKind classifies by prefix, slugFromSource folds the bad characters out of the slug, identity.ts:69-73), and the repair then prints `pipeline run docs/prompts/mirror tests, src.md --force`, which yargs parses as a source plus a stray positional (cli.ts:109-131). statusLines compounds it by joining the two commands with `   or   ` (runner.ts:3277-3278). Since a plain resume re-escalates by design (states.ts:136-138), this is the operator's only route out of a sticky plan stop, and it points at a command that does not run.","fix":"Quote the path when it is not shell-safe — `const arg = /^[A-Za-z0-9._/-]+$/.test(source) ? source : `'${source.replace(/'/g, `'\\\\''`)}'`` — and use `arg` in both repair steps; add a light-promotion case for a brief whose path needs quoting, asserting the printed `pipeline run …` line round-trips through the CLI's `run <source>` parser, and assert the same line status prints."},{"severity":"nit","location":"tools/orchestration/src/pipeline/runner.ts:3231-3237 (with report.ts:228-235,279-309)","problem":"Moving the footer into `RepairStep` to give the plan stop two moves put a label column on the terminal print — `  ${step.label}: ${step.command}` — so a full run's escalation terminal now reads `  retry after a fix: pipeline resume 002-x`. The proposal's Non-goals rule out changing the full flow's escalation text, and this seat cannot verify from the change alone that the label is the pre-branch string: this pass had no shell, so the range's before-image of that line is unreadable, and no test pins the label (escalation.test.ts:366-368 asserts only that the print contains `pipeline ESCALATED` and the resume command). ESCALATION.md's own footer is safe (note + command, report.ts:392-394, asserted at :188 and :204).","fix":"Confirm `retry after a fix` is the label the full flow's terminal printed before this branch and keep it if so; if it is new, restore the pre-branch label for the non-plan-stop step. Either way, extend the escalation test to assert the full-flow footer block (label + command) so the Non-goals promise is pinned and a later shared-shape refactor cannot move it silently."},{"severity":"minor","location":"tools/orchestration/package.json:23,30-32 (.github/workflows/check.yml:36,48,52-54)","problem":"Checklist item 7 — that `npm run check` stays green on linux at the engines Node floor — remains unevaluated by this seat, which was denied the shell: lint, format:check and vitest were never executed and the suite's pass count is unknown to this review. The static half of the claim holds and nothing in the post-approval fixes undermines it: the orchestration CI job runs `npm run check` on ubuntu-latest pinned to node-version 22.23.4, exactly the engines floor; no new test carries a runIf/skipIf or platform guard (the suite's only platform guards are moves.test.ts's, from an earlier run); the new light suites use only the existing mkdtempSync(tmpdir()) pattern; and the touched files add no API above the floor (structuredClone is the newest). The change still carries no executed datum.","fix":"Run `npm run check` in tools/orchestration at Node 22.23.4 on linux and record the exit status and vitest suite/test counts in the run's build notes, so the compatibility row rests on a datum; the CI job is the standing proof afterwards."}]}
```
