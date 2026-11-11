# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:38:40.323Z
- ended: 2026-11-10T17:45:07.081Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 64ae876fb3689573e750e2d79a658f530bbd0579 89027e8abc18ce75659a7237f8544692866db24f` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design — Read-only git for implement/fix/test-update seats (item 5) — \"The enforcement is the harness-side write deny: `coder.md` and `test-engineer.md` gain `edit` denies on `.git` and `.git/**` … It is load-bearing, because the sweep cannot see or undo a config plant\"",
    "problem": "re-raised? no — new. The r1 rewrite moved item 5's enforcement onto the agent-config permission layer and demoted the runner-side check to \"Defense in depth\", but this repo states twice that the permission layer is not the control: gates.ts:256-258 (\"Agent-config permissions are only advisory (a model can route around them via subagents), so the runner enforces this matrix deterministically\") and independence.ts's header (\"permissions alone did not hold, so the runner verifies after the fact\"). The design's own sentence concedes the deterministic half is blind here (`.git/**` is never a dirty path), and the `stagePathAllowed` line it keeps only runs over move endpoints and sweep output — never over an `edit` call. So the config plant that Options 1 cites as the reason to refuse any agent git access is left with no deterministic control while the allowlist re-opens exactly that lever, and Test strategy #13's fixture exercises only the direct write the advisory fence already claims to st… [elided by the checkpoint cap]",
    "fix": "Make the enforcement the mechanism the repo already uses for fences permissions do not hold: extend the existing runner-side tool-call audit (`auditSessionIndependence`, independence.ts, called at runner.ts:839-849 — \"The permissions say so; this checks up on it\") with a `.git`-segment matcher over completed write/edit tool calls that fails the attempt, and keep the agent-config deny as the advisory mirror (what r0 said). Alternatively register \"the harness `edit` deny cannot be routed around by the stage's own tools or subagents\" as an unverified assumption and make #13's fixture use the route gates.ts:257 names."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Apply — \"Immediately before the first rename it re-runs validator rules 4–7 as one guard, so a resume re-entering finalize on a checkpointed reply never applies against a stale `headBefore` or stale disk\"",
    "problem": "The premise is false for this codebase: `finalizeStage` has one call site, `executeStage`, immediately after a fresh `executeInference` (runner.ts:1070-1081), and resume re-runs the stage from inference (resumePipeline → drive → runWithRetry → executeStage, runner.ts:1453-1484); the checkpoint stores no replayable reply. The real window the guard closes is validate-at-inference → apply-at-finalize (which the Risks row states correctly). The stated premise is also self-defeating: were finalize re-entered on an already-applied list, rules 4–7 would reject it forever (rule 4 wants `from` on disk, rule 5 wants `to` absent), contradicting \"A crash between apply and commit converges on resume\".",
    "fix": "Restate the guard's rationale as the inference→finalize window the Risks row already names, and delete the checkpointed-reply clause."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Apply — \"Immediately before the first rename it re-runs validator rules 4–7 as one guard\"",
    "problem": "The guard's failure class is the only exit in that paragraph left unclassified: rename errors get `moves-apply-failed` (attempt 2, fresh session, worktree preserved) and rollback errors get `moves-rollback-failed` (stops at the operator), but a guard refusal names no outcome, no preserved-worktree state, and no test class — #5 pins that the guard refuses without naming the result, and #6 enumerates only the other three. The refusal also happens after the reply was accepted, where the `invalid-reply:` nudge path is no longer available.",
    "fix": "One clause: a guard refusal takes the same `moves-apply-failed` outcome and preserved-worktree state as a rename error; pin it in Test strategy #5/#6."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Read-only git (item 5) vs Options considered — \"If Test strategy #13 cannot show the harness refusing a `.git/config` write, item 5 does not ship in this change.\" against \"**Land `moves` without item 5.** Rejected: the ADR decided item 5\"",
    "problem": "The gate's fallback is exactly the option the Options section rejects, and ADR-005 item 5 is decided, so the design leaves the ship-failure outcome undecided (whole change held, or `moves` lands with ADR-005 item 5 unimplemented?). Meanwhile the Goals bullet \"The coder and test-engineer seats can run `git --no-optional-locks status`, `git log`, `git diff`, `git show` read-only\" states the conditional outcome unconditionally.",
    "fix": "Name the consequence in the same sentence — the change does not merge with item 5 unproven, or item 5 moves to a named follow-up with the ADR item left open — and qualify the Goals bullet to match."
  },
  {
    "severity": "nit",
    "location": "Chosen design — Validator rule 6 — \"all `from`s distinct and all `to`s distinct compared case-folded\"",
    "problem": "On the case-sensitive reference platform two tracked files differing only by case (`src/a.ts`, `src/A.ts`) are distinct blobs at `headBefore` (rule 4 proves it), so a two-pair list over them is legal input that the validator now deterministically rejects as a reply error, burning a code-budget unit (A3). The rule is safe, hence not unsound, but the text does not say the Linux rejection is intended, so an implementer cannot tell rule from bug.",
    "fix": "One clause stating the conservatism (\"case-folded distinctness is enforced on every platform; a repo with case-variant filenames cannot move both in one reply\"), or gate the fold on the checkout's `core.ignoreCase`."
  },
  {
    "severity": "nit",
    "location": "Assumption register A6 — \"`core.fsmonitor` fires on an index refresh — which `git status`, `git diff`, and `git show` all perform by default, `git --no-optional-locks` aside\"",
    "problem": "The row's own basis supports two of the three: git-status(1) BACKGROUND REFRESH covers `git status`, and `diff.autoRefreshIndex` (\"defaults to true … affects only git diff Porcelain\") covers `git diff`; nothing cited documents an index refresh for `git show`, which by default compares commits/trees rather than the worktree. The row's conclusion is unaffected, but a `verified` row carries an unsupported sub-claim.",
    "fix": "Drop \"and `git show`\" from that clause, or add the basis that shows the refresh for it."
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
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design — Read-only git (item 5) — \"in the shape of its existing `transcripts` matcher, and a hit fails the attempt. It carries the plant the sweep cannot see or undo\"","problem":"new. The control detects and fails the attempt but never neutralizes the payload, and the retry it triggers re-exposes the runner to it. auditSessionIndependence judges only messages at/after this attempt's prompt (independence.ts:46-50: \"A retried attempt must not be failed again for reads an earlier attempt already paid for\"), and an `independence violation:` failure is not resumable (retryInSameSession, runner.ts:1121-1126), so attempt 2 runs a fresh session in the SAME worktree with the plant still in `.git` — nothing restores it, which is the design's own premise (\"`.git/**` is never dirty and has no blob at `headBefore`\"). Attempt 2's finalize then runs `git add -A` (runner.ts:298) and `git commit -m` (307, no `--no-verify`) under the planted `.git/config` (core.fsmonitor, include.path) or a planted `.git/hooks/pre-commit`, executing at runner privilege, and no later attempt re-detects it. Test strategy #13 pins only \"fails the attempt\" — exactly the half that does not protect the run. The claim that the audit \"carries the plant\" is therefore false as written.","fix":"Make a `.git`-segment audit hit terminal for the run — escalate to the operator as `moves-rollback-failed` does, no attempt 2 — or name the neutralization step (delete/restore the detected `.git` path) that must happen before any later attempt; and extend #13 to assert the following attempt's runner `git commit` does not execute the plant."},{"severity":"major","location":"Chosen design — Read-only git (item 5) — \"The enforcement is runner-side, because agent-config permissions are only advisory: a model can route around them (gates.ts:256-258, independence.ts)\"","problem":"new (same theme as r1 #1, aimed at the new mechanism). The route those lines name is subagents — gates.ts:257: \"a model can route around them via subagents\" — and the chosen control is blind to it. auditSessionIndependence scans `lastSeen`, this attempt's own session message list (runner.ts:759-769, 839; independence.ts:10-12 \"its own message list\"). A9's newly cited docs state \"A custom subagent uses its own permissions, not a subset of its parent's permissions\" with a base policy of `{action:\"*\",resource:\"*\",effect:\"allow\"}`, and no config under .opencode/agents denies the subagent action for coder.md or test-engineer.md. So a `.git/config` write by a spawned subagent is invisible to the audit, invisible to the sweep, and ungoverned by the two configs' denies — the exact lever Options 1 uses to refuse agent git access. #13's three vectors all run in the seat's own session, so item 5's ship gate never exercises the route the justification cites.","fix":"Name how the attempt's child sessions are audited (the mechanism that enumerates them and feeds their completed write/edit/patch and shell parts into the same matcher), or add a subagent-performed `.git` write to #13 and, if uncoverable, record the subagent route as an explicit uncovered vector instead of claiming runner-side enforcement over \"the plant the sweep cannot see or undo\"."},{"severity":"minor","location":"Chosen design — Validator rule 7 — \"rule 2's `.git` test runs on those resolved paths too\"","problem":"new. Combined with the new predicate \"any path with a case-folded `.git` segment or a segment ending in `.git`\", running the test over absolute resolved paths rejects every endpoint whenever an ancestor of the checkout carries such a segment. The bare-clone-with-worktrees layout (`~/src/project.git/wt-feature`) is exactly the linked-worktree case the Compatibility matrix marks `yes` and #13 exercises via `git worktree add`; there every move fails rule 7, so the feature is unusable rather than merely `.git`-refusing. The planted-symlink case the clause exists for is already caught by the containment clause.","fix":"State that the segment test runs on the resolved path made relative to the repo's realpath, with containment remaining the absolute check."},{"severity":"minor","location":"Chosen design — Read-only git (item 5) — \"`-c safe.bareRepository=explicit`, so the config sources outside the repo that A6 names and a nested repo planted in the worktree never execute at runner privilege\"","problem":"new. The nested-repo half is mis-attributed: safe.bareRepository (introduced in git 2.38, so the new floor row's basis holds) governs whether git will operate on a bare repository it discovers at the invocation's cwd, and `git()` already pins `cwd: repoDir` (runner.ts:106-112), so a nested repo in the worktree is never the discovered repository with or without the flag. Consequently #13's \"the runner's git calls ignore … a nested bare repo\" fixture passes whether or not the flag ships, so a control that both the enforcement paragraph and the Compatibility floor rest on is pinned by a test that cannot fail.","fix":"Attribute the nested-repo half to the pinned cwd and keep the flag claim to the ambient-config half; either drop the flag from the sentence and the 2.38 floor rationale, or make #13's case a runner git call whose cwd is inside the planted bare repo so the flag is actually exercised."},{"severity":"nit","location":"Chosen design — Read-only git (item 5) — \"The two agent configs mirror it as `edit` denies on `.git`, `.git/**`, `**/.git/**` and `**/*.git`\"","problem":"new. Under the whole-value wildcard matching A9 now records (`*` matches zero or more characters including `/`), `**/*.git` denies only paths that end in `.git`, so the mirror does not express the predicate's \"a segment ending in `.git`\" for its children (e.g. `src/mirror.git/config` remains permitted by the config); and the docs make path matching case-insensitive only on Windows, so the `.GIT/hooks/pre-commit` refusal the paragraph attributes to the mirror is runner-side alone.","fix":"One clause: the advisory mirror covers the `.git` dir/pointer and nested `.git/` dirs; the `*.git`-segment and case-folded halves exist only in `stagePathAllowed`."}]}
```
