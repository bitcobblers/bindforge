# Transcript: code-review:correctness r1c2

## Provenance
- stage: code-review:correctness (round r1c2)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T03:19:09.803Z
- ended: 2026-11-11T03:26:33.655Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/006-self-update-checkpoint-compatibility/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 640c289bdd01611c1fcd07a504af78eb59fb3fc7..HEAD -- ':(exclude)docs/designs', read against the full current document.
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
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:2765-2791 (finalizeDone's ADR flip before the terminal save, with deviation 11's new CheckpointWriteFailure guard)",
    "problem": "finalizeDone flips the source ADR (status: proposed → accepted) and writes it to the worktree BEFORE the terminal `done` save. The new guard added in this diff turns a failed done save into finalizeEscalate, whose commitStage then COMMITS the flipped accepted ADR alongside an `escalated` checkpoint and ESCALATION.md — previously the crash left the flip dirty and uncommitted. Proposal Rollout #4 says the ADR flips 'when that run completes'; an escalated run has not completed, yet the accepted ADR lands in history, and a later finalizeDone re-run takes flipAdrStatus's null branch and warns 'ADR status not flipped (unexpected format)'.",
    "fix": "Move the ADR flip after the terminal save succeeds — it must precede commitStage, not checkpoint(): flip between the successful save and the completion commit, so an escalated commit never carries the accepted flip."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/006-self-update-checkpoint-compatibility/reviews/code/correctness-r1c2.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:2765-2798 (finalizeDone's ADR flip before the guarded terminal save) and 2917-2922 (finalizeEscalate's commitStage)", "problem": "Carried from r1c1, still open and unchanged by the post-c1 revisions: finalizeDone flips the source ADR (proposed → accepted) before the guarded done save; a CheckpointWriteFailure routes to finalizeEscalate, whose commitStage then commits the accepted ADR alongside an escalated checkpoint and ESCALATION.md. Proposal Rollout #4 ties the flip to run completion; an escalated run has not completed, and a later finalizeDone re-run only warns via flipAdrStatus's null branch. The 640c289 delta neither fixes nor worsens this.", "fix": "Move the ADR flip between the successful terminal save and the completion commit — never before the save — so an escalated commit never carries the accepted flip."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/state-store.ts:263-313 (holdsRunnerBytes recorded-delete precedence added in 640c289)", "problem": "A recorded delete outranks a recorded write permanently: recordRunnerWrite never clears runnerDeleted. If the runner deletes a reserved path and later rewrites it in the same process (resume's marker delete at runner.ts:2192, then a re-escalation's marker write at runner.ts:2884), holdsRunnerBytes returns false even though the file holds exactly the runner's fresh bytes, so any later sweep would revert the runner's own write. Latent only today — no ownership sweep runs between those two actions before finalizeEscalate's commitStage — but the pair is a state machine with a stranded delete flag.", "fix": "Make the authorship records last-action-wins: recordRunnerWrite should runnerDeleted.delete(abs), keeping the delete-outranks-write read without stranding a stale delete record."}]}
```
