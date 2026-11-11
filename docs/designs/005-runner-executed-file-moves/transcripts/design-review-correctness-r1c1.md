# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:20:46.771Z
- ended: 2026-11-10T17:27:38.801Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 188d705ce82601480eb1b3318daec99a7595b254 64ae876fb3689573e750e2d79a658f530bbd0579` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design — Read-only git for implement/fix/test-update seats (item 5): \"`git status --no-optional-locks*` (the pinned form avoids the index-refresh side effect and the lock contention with the runner's own git calls, A6)\"",
    "problem": "The pinned form is the wrong spelling. git(1)'s synopsis places --no-optional-locks among the options that precede <command>, it appears in none of git-status's own OPTIONS, and git-status's BACKGROUND REFRESH section (and this run's own 01-research.md) spell the remedy `git --no-optional-locks status`. As pinned, the only allowed `git status` form is one git rejects with a fatal unknown option, so the goal \"the coder and test-engineer seats can run `git status`\" is not delivered as written and the claim that the pin avoids the index refresh is false for that flag position.",
    "fix": "Replace the allow entry with `git --no-optional-locks status*` (the documented git(1) spelling); deny refinements and the rest of the allowlist unchanged."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Read-only git section: \"review seats already run the same three of four, code-correctness.md:27-31\"",
    "problem": "The cited file and every other agent config carry exactly two of the four named commands: `git diff*` and `git log*` (design-review seats only `git diff*`); `git show*` and `git status*` appear in no agent definition. The precedent holds, the count does not.",
    "fix": "Change to \"the same two of four (`git diff*`, `git log*)\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Validator rule 7: \"the realpath of `from` and of `to`'s nearest existing ancestor stay inside the repo's realpath\"",
    "problem": "'Nearest existing ancestor' deliberately tolerates a `to` whose parent directory does not exist: such a pair passes every pre-flight rule, then deterministically fails at apply with renameSync ENOENT, is classified `moves-apply-failed` as 'the model's reply was fine, the failure is environmental' — false for this case — and bypasses the in-attempt nudge path, burning a fresh session and (for fix) a budget unit. ADR-005 item 2's own standard is to check everything before touching anything, and this is a pure pre-touch disk check.",
    "fix": "Add to rule 5: \"`to`'s parent directory exists on disk\"; add the case to Test strategy #2's one-case-per-rule list."
  },
  {
    "severity": "nit",
    "location": "Assumption register A13: \"saveState re-validates | verified | tools/orchestration/src/pipeline/state-store.ts:145-155\"",
    "problem": "Lines 145-155 are loadState's version gate and safeParse; saveState's re-validation is at 178-184. The claim is true but the cited range does not cover the saveState half of the row.",
    "fix": "Cite state-store.ts:145-155, 178-184."
  },
  {
    "severity": "nit",
    "location": "Assumption register A6: \"`git status` refreshes the index by default | verified | https://git-scm.com/docs/git-log, https://git-scm.com/docs/git-diff, https://git-scm.com/docs/git-config\"",
    "problem": "The claim is true (git-status BACKGROUND REFRESH) but none of the three cited pages carries it; the source is git(1)/git-status(1).",
    "fix": "Add https://git-scm.com/docs/git (or git-status) to the row's basis."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/005-runner-executed-file-moves.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design — Read-only git for implement/fix/test-update seats (item 5) — \"The enforcement is the harness-side write deny: `coder.md` and `test-engineer.md` gain `edit` denies on `.git` and `.git/**` … It is load-bearing, because the sweep cannot see or undo a config plant\"","problem":"re-raised? no — new. The r1 rewrite moved item 5's enforcement onto the agent-config permission layer and demoted the runner-side check to \"Defense in depth\", but this repo states twice that the permission layer is not the control: gates.ts:256-258 (\"Agent-config permissions are only advisory (a model can route around them via subagents), so the runner enforces this matrix deterministically\") and independence.ts's header (\"permissions alone did not hold, so the runner verifies after the fact\"). The design's own sentence concedes the deterministic half is blind here (`.git/**` is never a dirty path), and the `stagePathAllowed` line it keeps only runs over move endpoints and sweep output — never over an `edit` call. So the config plant that Options 1 cites as the reason to refuse any agent git access is left with no deterministic control while the allowlist re-opens exactly that lever, and Test strategy #13's fixture exercises only the direct write the advisory fence already claims to stop.","fix":"Make the enforcement the mechanism the repo already uses for fences permissions do not hold: extend the existing runner-side tool-call audit (`auditSessionIndependence`, independence.ts, called at runner.ts:839-849 — \"The permissions say so; this checks up on it\") with a `.git`-segment matcher over completed write/edit tool calls that fails the attempt, and keep the agent-config deny as the advisory mirror (what r0 said). Alternatively register \"the harness `edit` deny cannot be routed around by the stage's own tools or subagents\" as an unverified assumption and make #13's fixture use the route gates.ts:257 names."},{"severity":"minor","location":"Chosen design — Apply — \"Immediately before the first rename it re-runs validator rules 4–7 as one guard, so a resume re-entering finalize on a checkpointed reply never applies against a stale `headBefore` or stale disk\"","problem":"The premise is false for this codebase: `finalizeStage` has one call site, `executeStage`, immediately after a fresh `executeInference` (runner.ts:1070-1081), and resume re-runs the stage from inference (resumePipeline → drive → runWithRetry → executeStage, runner.ts:1453-1484); the checkpoint stores no replayable reply. The real window the guard closes is validate-at-inference → apply-at-finalize (which the Risks row states correctly). The stated premise is also self-defeating: were finalize re-entered on an already-applied list, rules 4–7 would reject it forever (rule 4 wants `from` on disk, rule 5 wants `to` absent), contradicting \"A crash between apply and commit converges on resume\".","fix":"Restate the guard's rationale as the inference→finalize window the Risks row already names, and delete the checkpointed-reply clause."},{"severity":"minor","location":"Chosen design — Apply — \"Immediately before the first rename it re-runs validator rules 4–7 as one guard\"","problem":"The guard's failure class is the only exit in that paragraph left unclassified: rename errors get `moves-apply-failed` (attempt 2, fresh session, worktree preserved) and rollback errors get `moves-rollback-failed` (stops at the operator), but a guard refusal names no outcome, no preserved-worktree state, and no test class — #5 pins that the guard refuses without naming the result, and #6 enumerates only the other three. The refusal also happens after the reply was accepted, where the `invalid-reply:` nudge path is no longer available.","fix":"One clause: a guard refusal takes the same `moves-apply-failed` outcome and preserved-worktree state as a rename error; pin it in Test strategy #5/#6."},{"severity":"minor","location":"Chosen design — Read-only git (item 5) vs Options considered — \"If Test strategy #13 cannot show the harness refusing a `.git/config` write, item 5 does not ship in this change.\" against \"**Land `moves` without item 5.** Rejected: the ADR decided item 5\"","problem":"The gate's fallback is exactly the option the Options section rejects, and ADR-005 item 5 is decided, so the design leaves the ship-failure outcome undecided (whole change held, or `moves` lands with ADR-005 item 5 unimplemented?). Meanwhile the Goals bullet \"The coder and test-engineer seats can run `git --no-optional-locks status`, `git log`, `git diff`, `git show` read-only\" states the conditional outcome unconditionally.","fix":"Name the consequence in the same sentence — the change does not merge with item 5 unproven, or item 5 moves to a named follow-up with the ADR item left open — and qualify the Goals bullet to match."},{"severity":"nit","location":"Chosen design — Validator rule 6 — \"all `from`s distinct and all `to`s distinct compared case-folded\"","problem":"On the case-sensitive reference platform two tracked files differing only by case (`src/a.ts`, `src/A.ts`) are distinct blobs at `headBefore` (rule 4 proves it), so a two-pair list over them is legal input that the validator now deterministically rejects as a reply error, burning a code-budget unit (A3). The rule is safe, hence not unsound, but the text does not say the Linux rejection is intended, so an implementer cannot tell rule from bug.","fix":"One clause stating the conservatism (\"case-folded distinctness is enforced on every platform; a repo with case-variant filenames cannot move both in one reply\"), or gate the fold on the checkout's `core.ignoreCase`."},{"severity":"nit","location":"Assumption register A6 — \"`core.fsmonitor` fires on an index refresh — which `git status`, `git diff`, and `git show` all perform by default, `git --no-optional-locks` aside\"","problem":"The row's own basis supports two of the three: git-status(1) BACKGROUND REFRESH covers `git status`, and `diff.autoRefreshIndex` (\"defaults to true … affects only git diff Porcelain\") covers `git diff`; nothing cited documents an index refresh for `git show`, which by default compares commits/trees rather than the worktree. The row's conclusion is unaffected, but a `verified` row carries an unsupported sub-claim.","fix":"Drop \"and `git show`\" from that clause, or add the basis that shows the refresh for it."}]}
```
