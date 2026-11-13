# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-13T20:16:55.975Z
- ended: 2026-11-13T20:18:14.432Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":29697,"output":13901,"reasoning":0,"cache.read":286070,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/020-restart-boundary-for-harness-self-changes/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 15ba04f5008f2e18b9958fbe24a7727b9f01c441 0b1355604ef1e9dec4bbabc9f7ddd54dd27be710` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/020-restart-boundary-for-harness-self-changes.md,
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
verify what the design claims about the code, and the report at docs/designs/020-restart-boundary-for-harness-self-changes/reviews/design/correctness-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "Chosen design §1 (also the Options \"Per-commit hooks\" citation) — \"runs `git diff --name-only <headBefore>..HEAD`\"",
    "problem": "git enables rename detection in `diff` by default, and `--name-only` prints only the destination path for a detected rename. A window that removes a surface file by renaming it out of the two prefixes (e.g. a template moved from tools/orchestration/prompts/ to docs/) can show only the non-surface destination and slip through the predicate; §1's directory-scope argument covers additions, not this removal shape. The failure mode is loud (stage-entry load throws), so it is not the silent-staleness class.",
    "fix": "Specify `git diff --name-only --no-renames <headBefore>..HEAD` in §1, so renames appear as delete+add and both sides are filtered through the predicate; the Options section's citation of the same command inherits the fix."
  },
  {
    "severity": "minor",
    "location": "Chosen design §2 — \"`restartRequired?: { stage: string; paths: string[]; at: number }`\"",
    "problem": "ADR-020 Decision 2 requires the pause to stop \"with a `restart-required` marker naming the changed paths and the stage that will run next\", but every artifact the design names (the §2 record, the `restart-boundary` gate entry, the §3 status line \"the stage that opened the boundary\", the printed summary) names the completed stage, not the next stage. The next stage is derivable from `lastStage` plus the flow, but the design never says where the ADR's \"stage that will run next\" is carried.",
    "fix": "Add one sentence in §2 stating that the next stage is carried by the checkpoint's `lastStage` + flow and surfaced by the status line's resume posture — or add the next stage to the printed summary's content list."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/020-restart-boundary-for-harness-self-changes.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/020-restart-boundary-for-harness-self-changes/reviews/design/correctness-r1c2.md. That file is the only path this session
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
  "verdict": "approve",
  "findings": [
    {
      "severity": "minor",
      "location": "Chosen design §3 (with §2) — \"`pipeline status` while the record stands gains one line … the stage that opened the boundary, how many harness files it changed, and the resume posture\"",
      "problem": "re-raised (unchanged by this round, and not recorded as rejected in the Revision log): still short of ADR-020 Decision 2's marker \"naming the changed paths and the stage that will run next\". Every artifact the design names — the §2 record, the `restart-boundary` gate entry, the status line, the printed summary — names the stage that completed; the next stage is derivable from `lastStage` plus the flow, but the design never says where the ADR's \"stage that will run next\" is carried."
    },
    {
      "severity": "minor",
      "location": "Chosen design §2 (new in r2) — \"the record keeps the `boundedGateEvidence` bounding (first N paths plus the count)\"",
      "problem": "regression introduced by the closure: the count has nowhere to live. The record shape §2 declares is `restartRequired?: { stage, paths, at }`, so a record bounded to \"first N paths\" carries no total, and §3's status line (\"how many harness files it changed\") is rendered by a fresh process that can only read the checkpoint. The cited helper also does not fit the field: `boundedGateEvidence(evidence: GateEvidence, repoDir?)` bounds `evidence.legs` at `MAX_GATE_LEGS = 32` and encodes its elision count by pushing a synthetic entry into the kept array (runner.ts:3057–3075); the analogue for `paths: string[]` puts a non-path string into the record's path list, which then flows into the commit body and summary that the same sentence says carry \"every copy of a path\"."
    }
  ]
}
```
