# Transcript: code-review:correctness r1c1

## Provenance
- stage: code-review:correctness (round r1c1)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T03:07:52.679Z
- ended: 2026-11-11T03:16:07.024Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/006-self-update-checkpoint-compatibility/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 4fc97b316349ed487b5ef49430175c04aabc1e61..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/006-self-update-checkpoint-compatibility.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/state-store.ts:235-264 (runnerBytes/holdsRunnerBytes), consumed at tools/orchestration/src/pipeline/runner.ts:284",
    "problem": "The fence's authorship skip is per-process memory that loadState never seeds, so in a fresh process the run's own dirty checkpoint is an unmatched reserved path. The dirty copy is the normal state (the loop-tail save at runner.ts:2440 runs after that stage's commitStage), so a run interrupted mid-loop leaves the newest checkpoint uncommitted; on resume, when the next step is a single stage (proposal-revise, fix, implement, doc-sync, test-*), finalizeStage's sweep (runner.ts:1673) runs before any save in this process, holdsRunnerBytes returns false, and the sweep restores the checkpoint to headBefore and reports it as reverted — the operator sees `ownership: reverted 1 file(s) changed by <stage>: docs/designs/<run>/.pipeline.json`, a deterministic false tamper report on the ADR's own resume scenario. This contradicts proposal §1 (\"the runner's own save is skipped ... even though the file is dirty at every sweep\") and the Deferred rationale \"the §1 fence makes 'dirty ⇒ runner' true\", whi… [elided by the checkpoint cap]",
    "fix": "Seed the in-process record from what the loader read: in loadState, after the schema check, call recordRunnerWrite(file, raw) — or have runPipeline/resumePipeline save once before drive(). A later stage write or delete still mismatches the record and is still reverted, so the fence keeps its teeth while 'dirty ⇒ runner' becomes true across processes."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:294-304",
    "problem": "A reserved-path violation whose path has no blob in headBefore — the run's own untracked checkpoint at the first stage of a new run, after the entry save recorded bytes for it — falls into the else branch and is deleted (git rm -f + rmSync) instead of restored. The run then has no checkpoint file at all until the next save (loop tail runner.ts:2440 or finalizeEscalate), and a crash in that window leaves nothing for `pipeline resume` to load. Proposal §1 promises the sweep \"removes it and restores the file\".",
    "fix": "When namesReservedSegment(p) is true and runnerBytes holds bytes for that absolute path, write those recorded bytes back instead of deleting, so the checkpoint survives tampering regardless of what headBefore contains."
  },
  {
    "severity": "minor",
    "location": "docs/workflows.md (new \"A state file from another pipeline version\" section) with tools/orchestration/src/pipeline/state-store.ts:176-181 and 03-build-notes.md:41-45",
    "problem": "The operator docs and the build notes both claim the version-refusal message names an exit of \"run the version that wrote it\"; the shipped message names version 1 and offers only the proposal's two ADR postures — restore the committed copy, or start a fresh run. The code matches §2, the docs overstate it.",
    "fix": "Drop the \"run the version that wrote it\" clause from docs/workflows.md and the build note, or add that posture to the refusal message and to §2's rule set."
  },
  {
    "severity": "minor",
    "location": "docs/designs/006-self-update-checkpoint-compatibility/03-build-notes.md:83 and :88-131 (Red-gate diagnosis)",
    "problem": "Checklist item 5: the notes' claims are no longer true of HEAD. The three failures reported as living in untouched test files are fixed at HEAD — approveClient in checkpoint-integration.test.ts and runner.test.ts now models turn growth (visible = max(0, min(lists-1, prompts))), ownership.test.ts:221 destructures only dir (the unused-base ESLint error is gone), and commit 4fc97b3 re-pinned the funnel created[0] assertions. Line 83's \"No test files were touched\" is also false of the diff under review.",
    "fix": "Mark the red-gate section as the diagnosis at that fix round and record what the later test commit changed, so the next seat does not chase three already-closed items."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:2741-2745 with tools/orchestration/src/pipeline/state-store.ts:289-294",
    "problem": "The appended marker line re-prefixes a stem the error already carries: CheckpointWriteFailure.message starts with \"checkpoint save failed: \", so ESCALATION.md reads \"checkpoint save failed: checkpoint save failed: EISDIR ...\". The loadState EISDIR message likewise prints the same path twice (${file} and join(designDir, STATE_FILE) are the same string).",
    "fix": "Append printable(saveFailure.message) without adding the prefix, and reuse `file` in both halves of the repair sentence."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/006-self-update-checkpoint-compatibility/reviews/code/correctness-r1c1.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:2765-2791 (finalizeDone's ADR flip before the terminal save, with deviation 11's new CheckpointWriteFailure guard)", "problem": "finalizeDone flips the source ADR (status: proposed → accepted) and writes it to the worktree BEFORE the terminal `done` save. The new guard added in this diff turns a failed done save into finalizeEscalate, whose commitStage then COMMITS the flipped accepted ADR alongside an `escalated` checkpoint and ESCALATION.md — previously the crash left the flip dirty and uncommitted. Proposal Rollout #4 says the ADR flips 'when that run completes'; an escalated run has not completed, yet the accepted ADR lands in history, and a later finalizeDone re-run takes flipAdrStatus's null branch and warns 'ADR status not flipped (unexpected format)'.", "fix": "Move the ADR flip after the terminal save succeeds — it must precede commitStage, not checkpoint(): flip between the successful save and the completion commit, so an escalated commit never carries the accepted flip."}]}
```
