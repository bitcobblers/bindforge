# Transcript: code-review:correctness r1c1

## Provenance
- stage: code-review:correctness (round r1c1)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T00:20:53.062Z
- ended: 2026-11-11T00:27:38.898Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 29a6bc141f8a6420fb372271ff48d81dcb83202e..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/005-runner-executed-file-moves.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:1122-1132",
    "problem": "The failed-apply path runs `git add -A` over the whole worktree, behavior the proposal never asked for; the design says finalize \"returns without sweeping or committing, leaving the preserved-worktree state that a failed reply leaves today\". The comment's justification (porcelain collapsing a wholly-untracked directory) is false for the production path, since the sweep reads paths via `dirtyFiles` (runner.ts:184-188) which already passes `--untracked-files=all`. Staging also puts un-vetted content in the index, so a crash there leaves state the ownership sweep never examined.",
    "fix": "Remove the staging block and return the failure outcome unchanged, exactly as a failed reply does today; if individually-listed preserved paths are wanted, make it a separate, declared change."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:1134-1140 (with runner.ts:1550-1594 and runner.ts:1213-1216)",
    "problem": "`moves-rollback-failed` does not stop the run. The design makes it the one terminal class that \"escalates to the operator instead of starting attempt 2 on a known-damaged worktree\", but its only effect is that `retryInSameSession` returns false, which in `runWithRetry` means attempt 2 starts with a fresh session on the half-moved worktree; if attempt 2 replies validly the damage is committed. The comment at runner.ts:1216 claims \"starts nothing at all\", which the driver contradicts. Test strategy #6's requirement is pinned only as a prefix assertion (reply-contract.test.ts:187-196) and the unit case is it.skip'd (moves.test.ts:524-527).",
    "fix": "In `runWithRetry`, before launching attempt 2, treat a failure starting with `moves-rollback-failed` as terminal: record the stageFailure/gate entry, `finalizeEscalate`, and `StopDriving`. Pin with a driver-level test that counts attempts/sessions rather than asserting the prefix."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:770-780 and 994-1000",
    "problem": "The reply-polling loop for every stage is rewritten: `assistantBaseline` moved from after the stage prompt to before it, and the re-baseline after the invalid-reply nudge deleted. Neither change is in the proposal; both alter the pre-existing schema-mismatch path as well as the moves path. The removal deletes the documented guard against re-reading the previous turn: with no re-baseline, the idle-count test at runner.ts:829 cannot distinguish \"new turn produced\" from \"idle with no new turn\" (the case the branch at 839-846 exists for), so the runner re-validates the stale reply and burns the nudge budget in seconds — the exact failure mode the original comment was written for.",
    "fix": "Restore the post-prompt baseline read and the post-nudge re-baseline. If a real backend stall motivated this, register it as an assumption and declare it in 03-build-notes.md as its own change instead of folding it into this branch."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/prompts/design-review.md:6-10",
    "problem": "The branch also rewrites the design-review seat's prompt (auditing listing/globbing/grepping of run-record directories). The proposal's Prompts section names exactly four files (implement.md, fix.md, fix-tests.md, test-update.md); this change alters what every future design-review seat is told to do, is unrelated to runner-executed file moves, and has no test pin.",
    "fix": "Drop it from this branch and land it as its own change, or bring it into the proposal's scope and pin it in prompts.test.ts like the four moves sentences."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:969",
    "problem": "`checked.success || nudges >= MAX_NUDGES ? 'go' : await nudgeGate()` skips the nudge pacing floor for moves rejections and removes their access to the `'over'` → `stage-timeout` classification. The design says the moves failure rides the invalid-reply path and behaves \"exactly as for any invalid reply\"; one of that path's three endings is missing for moves only, which also makes finding 3's stale-turn read burn the budget instantly.",
    "fix": "Call `nudgeGate()` for moves rejections as well (the floor is a wall-clock floor between prompts), or record the divergence in the proposal/build notes."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/moves.ts:57-64",
    "problem": "`treeMode` passes the endpoint as a plain `git ls-tree` pathspec, which git glob-matches: a `from` such as `src/*.ts` matches a tracked sibling and satisfies rule 4's regular-file test for a path that is not itself tracked. It only fails closed incidentally via rule 4's on-disk check.",
    "fix": "Pass the pathspec as `:(literal)<relPath>` (or reject `*`, `?`, `[`, `]` in rule 1) so the tracked-file test means exactly that path."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:687-690",
    "problem": "`movesValidationIssue` returns null when `headBefore` is undefined, silently skipping validation for a reply that declares moves. Unreachable today (only `executeStage` passes headBefore), but the permissive default means a future caller running a move-capable stage through the pass fan-out would accept an unvalidated moves reply and still apply it in finalizeStage.",
    "fix": "Fail closed: return an issue such as \"moves declared without a base revision\" when a non-empty moves list arrives with no headBefore."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/code/correctness-r1c1.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": []}
```
