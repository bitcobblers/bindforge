# Transcript: code-review:testing r1c1

## Provenance
- stage: code-review:testing (round r1c1)
- agent: codereview/code-testing
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T00:24:47.248Z
- ended: 2026-11-11T00:30:29.986Z
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
Your own previous findings (testing seat):
[
  {
    "severity": "major",
    "location": "tools/orchestration/tests/pipeline/reply-contract.test.ts:183-197; tools/orchestration/tests/pipeline/moves.test.ts:517-527; production branch tools/orchestration/src/pipeline/moves.ts:338-347 and runner.ts:1136-1140",
    "problem": "Test strategy #6's third clause ('a moves-rollback-failed: outcome escalates to the operator and starts no attempt 2') and #5's rollback-failure clause are unbacked: the only pin is retryInSameSession(...) === false, which merely selects fresh-session attempt 2 — runWithRetry (runner.ts:1550-1591) always runs attempt 2 for a non-ok outcome and nothing special-cases the 'moves-rollback-failed' prefix — so the code starts attempt 2 on the known-damaged worktree while the test and code comments claim it starts nothing. No test ever executes the rollbackFailed === true path (moves.applyMoves's failed undo, finalizeStage's skip of git add -A); its only case is it.skip."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/tests/pipeline/moves.test.ts:37-41 (local MovePair), :66-88 (issueOf), :90-104 (applyFailureOf)",
    "problem": "The tests never pin the validator/applier contracts: issueOf and applyFailureOf accept null/string/{issue}/{message}/{error}/thrown-error interchangeably and flatten the result, so MoveApplyFailure.rollbackFailed — the single discriminator finalizeStage switches on — is asserted nowhere, and the tests would pass if applyMoves returned a bare string or hardcoded rollbackFailed false. The file also redeclares MovePair instead of importing it, so types.ts's schema can drift without a compile error."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:770-780 (assistant baseline read before the prompt) and :993-1000 (no re-baseline after the invalid-reply nudge)",
    "problem": "Two changes to the shared inference lane that affect every stage are unpinned: every stub in the suite delays turn visibility by at least one poll (reply-contract.test.ts:137/:376 visibleFrom = lists + 2; budget.test.ts visible = min(lists - 1, prompts)), so both new #9 and #10 tests pass unchanged against the pre-change code; reverting either edit keeps the suite green. Neither edit appears in the proposal's Test strategy or Revision log."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/moves.ts:107-115 (segment ending in '.' or space; empty segment) and :238-246, :252-258 (rule 6 nesting clauses)",
    "problem": "Rule clauses implemented with no unit case despite #2's 'one case per rule clause': the trailing-dot/space and empty-segment branches of shapeIssue and matchingIssue's self-nesting and cross-pair prefix checks are untested, although reachable (e.g. {from: 'src/old.ts', to: 'src/old.ts/inner.ts'} passes rules 4-5-7 and is rejected only by the self-nesting check)."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/tests/pipeline/moves-stage.test.ts:164-199",
    "problem": "The motivating payload is never landed end to end: every integration round trip moves a single src/** or docs/** file as implement, and fix is exercised only on the rejection side (#10). No test has test-update land a multi-pair list into tests/**, where rule 3's ownership semantics, the sweep invariant (#4) and single-commit rename detection must hold together — run 004's actual failure shape (A12, 23 test-file moves)."
  },
  {
    "severity": "minor",
    "location": "report-level: tools/orchestration `npm run check` (checklist item 6)",
    "problem": "Checklist item 6 could not be evaluated in this seat: the review sandbox's shell policy permitted only git (npm run check, npx, node, ls, which were each denied) and the worktree has no node_modules, so no green output tail can be pasted and the suite's status is unverified from this seat."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/tests/pipeline/moves.test.ts:493-515",
    "problem": "The EXDEV case reports 'passed' while asserting nothing on a root-but-unprivileged host: it.runIf(getuid() === 0) plus `catch { return; }` after `mount -t tmpfs` silently skips the assertions, and where the mount does succeed the suite performs a real system mount/umount."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:757-762 (movesValidationIssue try/catch and headBefore === undefined guard); claim at tools/orchestration/tests/pipeline/reply-contract.test.ts:465-468",
    "problem": "Unreachable throw-guard presented as tested: validateMoves is total under its own contract (git non-zero exit reads as untracked, an unreadable repoDir returns an issue), and the test that claims #2's 'a rule that throws still lands as invalid-reply: moves' uses from: 'CON', which shapeIssue rejects normally, so no throw is ever exercised."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/code/testing-r1c1.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:1411-1417 (lastMovesIssue at 1191, 1436-1438)", "problem": "The exhausted-nudge fallback that relabels `invalid-reply: not JSON` as `invalid-reply: moves — <lastMovesIssue>` (added by 17710f6 to name the rejected list when a nudged session goes silent/narrates) is never executed by any test: every moves-rejection stub in reply-contract.test.ts answers each prompt with parseable moves JSON, so exhaustion always exits through the parse/schema path at runner.ts:1473. The branch is reachable in a real shape (session answers the moves nudge with prose until the budget runs out) and its behavior — the gate log and repair prompt naming the pair — is otherwise unpinned.", "fix": "Add a sequenceStubServer case whose texts are [moves-invalid reply, narration x 8] and assert the attempt's failure is `invalid-reply: moves — <issue naming the pair>`, not `invalid-reply: not JSON`."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:1068-1069", "problem": "Residue of round-1 finding 8: the fail-closed issue for a moves-bearing reply arriving without a base revision is unreachable under today's schemas (only executeStage supplies moves-capable replies, always passing headBefore; review schemas carry no moves) and has no test — checklist item 7 (untestable addition). The throw-guard half of the finding is resolved by the injected git-fault test.", "fix": "Either pin the guard at the movesValidationIssue boundary (export-for-test or a pass-level stub replying with a moves-bearing object) or record it in 03-build-notes as an intentional fail-closed guard kept for a future schema slip."}]}
```
