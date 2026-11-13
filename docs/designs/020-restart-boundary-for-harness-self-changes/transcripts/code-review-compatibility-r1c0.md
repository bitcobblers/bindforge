# Transcript: code-review:compatibility r1c0

## Provenance
- stage: code-review:compatibility (round r1c0)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ 3fbf8466c8d0
- started: 2026-11-13T21:07:31.426Z
- ended: 2026-11-13T21:09:40.511Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":82374,"output":19901,"reasoning":0,"cache.read":1015804,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/020-restart-boundary-for-harness-self-changes/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 0179375b8d037833c6b6c5150312d88af0162211...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/020-restart-boundary-for-harness-self-changes.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/020-restart-boundary-for-harness-self-changes/reviews/code/compatibility-r1c0.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-13T21:02:48.947Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/020-restart-boundary-for-harness-self-changes/reviews/code/compatibility-r1c0.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "tools/orchestration/src/pipeline/cli.ts:43 (exitCodeFor); tools/orchestration/tests/pipeline/cli.test.ts", "problem": "Decision 3's exit-code table — the driver-compatibility contract ('2 keeps its exact current meaning, so no existing driver automates past an escalation by code collision'; risk table: 'distinct codes pinned both ways... pinned by Test #6') — is asserted by no test. Test strategy #6 names the integers (done→0, record→3, escalated→2, error→1), but the shipped cli.test.ts pins only the widened seam's shape (status/restartRequired presence); exitCodeFor appears in no test, so a future swap of 3/2 leaves the suite green while every documented driver loop (docs/workflows.md `while [ $? -eq 3 ]`) silently automates past escalations.", "fix": "Pin the integers, not just the fields: assert exitCodeFor({status:'done'})===0, a standing-record result===3, {status:'escalated'} without a record===2, and that the run/resume handlers return exitCodeFor's value (the thrown-error→1 leg is already pinned by the rejection test)."}, {"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:3020 and :3036 (drive's finalizeEscalate exits) with runResult at :2287", "problem": "The record set by finalizeStage/finalizeReviewPass (:1957/:~2032) can still be standing when drive's per-iteration checkpoint save fails or an unhandled throw escalates before the loop-top check: finalizeEscalate does not clear state.restartRequired, so runResult ships {status:'escalated', restartRequired} and exitCodeFor maps the standing record to 3 — a run with a committed ESCALATION.md exits 3, and the documented driver loop resumes it automatically, breaking the matrix's promise that 2 keeps its exact meaning in that corner (and the escalation checkpoint renders the boundary status line beside the escalation). finalizeRestartBoundary's own save-failure path already applies the correct posture ('the escalation is the terminal record from here', :3592-3600); the generic escalation exits do not.", "fix": "Clear state.restartRequired in finalizeEscalate (runner.ts:3656) so the terminal escalation never carries a pause, and pin it with an escalate-after-surface-hit fixture asserting exit mapping 2 and no record in the committed checkpoint."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:1861-1869 (boundedSurfacePaths), consumed at :3801 (restartBoundaryStatusLines), the gate reason and the printed summary", "problem": "The elision sentence is appended into record.paths, so for windows with more than MAX_GATE_LEGS hit paths the printed 'changed N harness file(s)' uses the post-bounding length: a 40-path window prints 'changed 33 harness file(s)' and counts the elision sentence as a file. §2's wording is 'first N paths plus the count' (the shape mirrors boundedGateEvidence at :3182-3191, so this is wording fidelity, not drift).", "fix": "Take the reported count from the pre-bounding total (paths.length before the slice) in the status line, gate reason, and summary — or store the total count on the record."}]}
```
