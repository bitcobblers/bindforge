# Transcript: code-review:correctness r1c0

## Provenance
- stage: code-review:correctness (round r1c0)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T02:18:06.156Z
- ended: 2026-11-11T02:25:59.338Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/006-self-update-checkpoint-compatibility/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 846ca090053b43ec99295a7cedafb47910043df9...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/006-self-update-checkpoint-compatibility.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/006-self-update-checkpoint-compatibility/reviews/code/correctness-r1c0.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "tools/orchestration/src/pipeline/runner.ts:1931-1959 (rethrow) with runner.ts:2767-2772/357 (commitStage)", "problem": "A seat's in-pass checkpoint write failure (CheckpointWriteFailure, now an escalation cause per §3) is rethrown by `for (const s of settled) … throw s.reason` before either `finalizeReviewPass` call, so the pass's ownership sweep never runs; `drive`'s new catch then routes it to `finalizeEscalate`, whose `commitStage` runs `git add -A`. A sibling seat's out-of-ownership file (e.g. planted `tools/evil.ts`, or a file in another design's docs) is committed into the run's terminal commit — the committed copy can carry model-authored bytes again, which is exactly what §1's fence exists to prevent, and what the comment at runner.ts:1940-1947 promises cannot happen. Pre-existing behavior minted no commit on this path (it crashed), so the hole is introduced by this change; no fixture covers it (the sabotage fixture plants only inside the checkpoint directory, which `restoreCommittedCheckpoint` removes).", "fix": "Before rethrowing a rejected seat reason, run the same best-effort sweep used at runner.ts:1951-1958 — `try { finalizeReviewPass(opts.repoDir, state, args.headBefore, args.stage, false) } catch { log and continue }` — then rethrow. Add a fixture where one seat writes `tools/evil.ts` and replaces the checkpoint path, asserting the file is reverted while the escalation still lands."}, {"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:2124-2126 and 2669", "problem": "`finalizeDone` is called outside the `try` whose catch recognizes `CheckpointWriteFailure` (runner.ts:2441-2455), and its terminal save goes through the same failing write path. If the checkpoint becomes unwritable at the last step, the run crashes with no terminal record, checkpoint left `running`, no marker, so `status` prints no split-record line and no repair command — the remaining gap in §3's goal.", "fix": "Guard that save: on `CheckpointWriteFailure`, print the `rm -r <cp> && git checkout HEAD -- <cp>` repair and return the completed status (escalating would be wrong, the work is finished) — or declare the exclusion explicitly in 03-build-notes.md §Deviations."}, {"severity": "minor", "location": "docs/designs/006-self-update-checkpoint-compatibility/03-build-notes.md:4 and 88-131", "problem": "Checklist item 5 (build-notes claims true of the code) fails on two counts: line 4 asserts the package tests pass while the same file's red-gate diagnosis lists failures, and two of that section's \"not touched\" claims no longer match this diff — ownership.test.ts:222-225 destructures only `const { dir }` (the cited ESLint no-unused-vars at ownership.test.ts:223 is gone), and both `approveClient` stubs now model turn growth (checkpoint-integration.test.ts:87-96,155; runner.test.ts:210-217,270) with the `created[0]` assertions relaxed. This reviewer also could not execute the shell, so `npm run check` being green is unverified in the review record.", "fix": "Update the red-gate diagnosis (or mark it resolved-by-round) to match the tree, keep line 4's green claim backed by a recorded `npm run check` run, and restate which items were later fixed by the test-update stage."}, {"severity": "minor", "location": "docs/workflows.md:137-169 (with tools/orchestration/src/pipeline/state-store.ts:180-184)", "problem": "§2 requires the canonical contract statement in both the StateSchema header and the operator docs. The new section documents fencing, the repair, split records and refusal, but never states the operator-facing guarantee — that any runner loads and resumes any `version: 1` checkpoint written by an earlier runner (the self-update promise Rollout 4 depends on). It also says the refusal message \"says to run the version that wrote it\", while the code only names that version and advises the ADR's two postures (restore the committed copy, or start fresh).", "fix": "Add one sentence stating the version-1 resume guarantee to the Self-update checkpoint recovery section, and reword the refusal description to match the message the code prints."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:2722-2744", "problem": "The comment promises \"Atomic tmp+rename, like every other checkpoint write: this is the one record the operator must be able to read afterwards\", but the marker is written with a plain `writeFileSync(join(repoDir, reportRel), markdown)`. Now that ESCALATION.md is the terminal truth in the split-record posture (status line, and resumePipeline's failure-line grep at runner.ts:2670-2674), an interrupted write leaves a truncated marker.", "fix": "Write the marker tmp+rename as `saveState` does (and record the bytes for the authorship test), or reword the comment so it covers only the guarded `checkpoint()` call."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/types.ts:183-189 (against tools/orchestration/src/pipeline/moves.ts:133-147)", "problem": "The comment says the reserved-name comparison is case-insensitive \"the way the moves validator already compares these names\", but moves.ts compares STATE_FILE/LOCK_FILE case-sensitively (only `.git` is lowercased). The fence being stronger is per design and `ownershipIssue` closes the move endpoint via stagePathAllowed, but the comment mis-describes the neighbour.", "fix": "Drop or correct the cross-reference: say the fence matches the moves validator's segment-wise shape but is case-insensitive, unlike moves.ts:136-139."}]}
```
