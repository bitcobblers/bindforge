# Transcript: design-review:correctness r1c3

## Provenance
- stage: design-review:correctness (round r1c3)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:53:21.424Z
- ended: 2026-11-10T18:00:20.207Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 89027e8abc18ce75659a7237f8544692866db24f 6b21e2f4897d0c0e93a8694889904bcfdbf4f922` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design — Read-only git (item 5) — \"in the shape of its existing `transcripts` matcher, and a hit fails the attempt. It carries the plant the sweep cannot see or undo\"",
    "problem": "new. The control detects and fails the attempt but never neutralizes the payload, and the retry it triggers re-exposes the runner to it. auditSessionIndependence judges only messages at/after this attempt's prompt (independence.ts:46-50: \"A retried attempt must not be failed again for reads an earlier attempt already paid for\"), and an `independence violation:` failure is not resumable (retryInSameSession, runner.ts:1121-1126), so attempt 2 runs a fresh session in the SAME worktree with the plant still in `.git` — nothing restores it, which is the design's own premise (\"`.git/**` is never dirty and has no blob at `headBefore`\"). Attempt 2's finalize then runs `git add -A` (runner.ts:298) and `git commit -m` (307, no `--no-verify`) under the planted `.git/config` (core.fsmonitor, include.path) or a planted `.git/hooks/pre-commit`, executing at runner privilege, and no later attempt re-detects it. Test strategy #13 pins only \"fails the attempt\" — exactly the half that does not protect th… [elided by the checkpoint cap]",
    "fix": "Make a `.git`-segment audit hit terminal for the run — escalate to the operator as `moves-rollback-failed` does, no attempt 2 — or name the neutralization step (delete/restore the detected `.git` path) that must happen before any later attempt; and extend #13 to assert the following attempt's runner `git commit` does not execute the plant."
  },
  {
    "severity": "major",
    "location": "Chosen design — Read-only git (item 5) — \"The enforcement is runner-side, because agent-config permissions are only advisory: a model can route around them (gates.ts:256-258, independence.ts)\"",
    "problem": "new (same theme as r1 #1, aimed at the new mechanism). The route those lines name is subagents — gates.ts:257: \"a model can route around them via subagents\" — and the chosen control is blind to it. auditSessionIndependence scans `lastSeen`, this attempt's own session message list (runner.ts:759-769, 839; independence.ts:10-12 \"its own message list\"). A9's newly cited docs state \"A custom subagent uses its own permissions, not a subset of its parent's permissions\" with a base policy of `{action:\"*\",resource:\"*\",effect:\"allow\"}`, and no config under .opencode/agents denies the subagent action for coder.md or test-engineer.md. So a `.git/config` write by a spawned subagent is invisible to the audit, invisible to the sweep, and ungoverned by the two configs' denies — the exact lever Options 1 uses to refuse agent git access. #13's three vectors all run in the seat's own session, so item 5's ship gate never exercises the route the justification cites.",
    "fix": "Name how the attempt's child sessions are audited (the mechanism that enumerates them and feeds their completed write/edit/patch and shell parts into the same matcher), or add a subagent-performed `.git` write to #13 and, if uncoverable, record the subagent route as an explicit uncovered vector instead of claiming runner-side enforcement over \"the plant the sweep cannot see or undo\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Validator rule 7 — \"rule 2's `.git` test runs on those resolved paths too\"",
    "problem": "new. Combined with the new predicate \"any path with a case-folded `.git` segment or a segment ending in `.git`\", running the test over absolute resolved paths rejects every endpoint whenever an ancestor of the checkout carries such a segment. The bare-clone-with-worktrees layout (`~/src/project.git/wt-feature`) is exactly the linked-worktree case the Compatibility matrix marks `yes` and #13 exercises via `git worktree add`; there every move fails rule 7, so the feature is unusable rather than merely `.git`-refusing. The planted-symlink case the clause exists for is already caught by the containment clause.",
    "fix": "State that the segment test runs on the resolved path made relative to the repo's realpath, with containment remaining the absolute check."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Read-only git (item 5) — \"`-c safe.bareRepository=explicit`, so the config sources outside the repo that A6 names and a nested repo planted in the worktree never execute at runner privilege\"",
    "problem": "new. The nested-repo half is mis-attributed: safe.bareRepository (introduced in git 2.38, so the new floor row's basis holds) governs whether git will operate on a bare repository it discovers at the invocation's cwd, and `git()` already pins `cwd: repoDir` (runner.ts:106-112), so a nested repo in the worktree is never the discovered repository with or without the flag. Consequently #13's \"the runner's git calls ignore … a nested bare repo\" fixture passes whether or not the flag ships, so a control that both the enforcement paragraph and the Compatibility floor rest on is pinned by a test that cannot fail.",
    "fix": "Attribute the nested-repo half to the pinned cwd and keep the flag claim to the ambient-config half; either drop the flag from the sentence and the 2.38 floor rationale, or make #13's case a runner git call whose cwd is inside the planted bare repo so the flag is actually exercised."
  },
  {
    "severity": "nit",
    "location": "Chosen design — Read-only git (item 5) — \"The two agent configs mirror it as `edit` denies on `.git`, `.git/**`, `**/.git/**` and `**/*.git`\"",
    "problem": "new. Under the whole-value wildcard matching A9 now records (`*` matches zero or more characters including `/`), `**/*.git` denies only paths that end in `.git`, so the mirror does not express the predicate's \"a segment ending in `.git`\" for its children (e.g. `src/mirror.git/config` remains permitted by the config); and the docs make path matching case-insensitive only on Windows, so the `.GIT/hooks/pre-commit` refusal the paragraph attributes to the mirror is runner-side alone.",
    "fix": "One clause: the advisory mirror covers the `.git` dir/pointer and nested `.git/` dirs; the `*.git`-segment and case-folded halves exist only in `stagePathAllowed`."
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
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/correctness-r1c3.md. That file is the only path this session
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
  "verdict": "request-changes",
  "findings": [
    {
      "severity": "major",
      "location": "Chosen design — Read-only git (item 5) — \"A hit ends the stage as `git-plant-detected: <path>`, escalating to the operator like `moves-rollback-failed`: no attempt 2 and no finalize, since both would run in that worktree under the planted config and the `since` window never re-detects it.\"",
      "problem": "new. Escalation is not git-free in this runner, so the new terminal class neutralizes nothing: the escalation path ends in `finalizeEscalate`, which writes ESCALATION.md and then calls `commitStage` (runner.ts:2008-2013) — `git add -A` (298) and `git commit -m` (307, no `--no-verify`) in the same worktree; the review-pass escalation path likewise still sweeps (`finalizeReviewPass(..., false)` at runner.ts:1313, whose own comment at 1089-1091 says \"the sweep still runs\") via `git rev-parse`/`git status`/`git restore` (180, 213, 236). So the act of escalating is itself a runner-privilege git call over the plant: `git add -A` refreshes the index and fires a planted `core.fsmonitor`, and `git commit` executes a planted `.git/hooks/pre-commit`. #13's new containment assertion — \"with the plant in place the runner's next git call must not execute it — the hostile config is a `core.fsmonitor` that touches a marker file, which stays absent\" — fails against the design as written, and the paragraph's own note that \"the `since` window never re-detects it\" leaves the plant live for an operator `pipeline resume` (runner.ts:1376) too.",
      "fix": "Name the neutralization that runs before any git on this path: the runner deletes the detected `.git` path (or pins `-c core.fsmonitor=false -c core.hooksPath=<empty dir>` on the escalation-path calls). Replace \"no finalize, since both would run in that worktree under the planted config\" with that step, make #13's marker fixture assert it against `finalizeEscalate`'s commit rather than a bare \"next git call\", and state what `resume` requires of the operator (nothing in-process clears the plant)."
    },
    {
      "severity": "major",
      "location": "Chosen design — Read-only git (item 5) + Test strategy #13 — \"The same two configs append a `subagent` deny: the audit judges one attempt's own message list, and gates.ts:257 names subagents as the route around config denies\" / \"every completed `.git` write (`src/.git/config` plant, `git log --output=.git/config`, a subagent-launched one) escalates the stage\"",
      "problem": "re-raised. The `subagent` deny is real progress, but the ship gate now asserts what the named mechanism cannot produce. The audit reads `lastSeen`, the one session id the attempt polled (runner.ts:745-769, 839; independence.ts:11 \"its own message list\"); no text names anything that enumerates the attempt's child sessions and feeds their completed `write`/`edit`/`patch`/shell parts into the matcher. With the deny in force the launch is refused, so there is no child write part to escalate; with the deny routed around — the case the paragraph exists for, since it opens \"agent-config permissions are only advisory\" — the child's `.git` write is invisible to the audit and the stage commits under it. Either way #13's subagent case cannot pass as an audit assertion, and the paragraph still claims runner-side enforcement over \"the plant the sweep cannot see or undo\".",
      "fix": "Pick one and say it in one clause. (a) Add the enumeration the assertion presumes — the runner walks the attempt's child sessions reached from its completed `task` parts and runs the same matcher over their parts — and keep #13's case; or (b) drop \"a subagent-launched one\" from #13's escalate list, keep it in the refused-by-config list, and record the subagent route as covered by the advisory config deny only (audit-blind) in both the paragraph and the Risks row."
    },
    {
      "severity": "minor",
      "location": "Chosen design — Read-only git (item 5) — \"`git()` (runner.ts:106-112) stops reading ambient config — `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL` at an empty file … so the outside-repo config sources A6 names never execute at runner privilege\" (and the Risks row's \"nothing outside the repo executes at runner privilege\")",
      "problem": "new. The blanket scope is wider than the one helper that ships the env. Two other runner git spawns bypass `git()`: `gitOk` (runner.ts:143-145, which serves `blobStamp`'s `git cat-file -e`) and the `merge-base` call at runner.ts:1417-1420, both plain `spawnSync('git', …)` inheriting the operator's environment. #13's \"A hostile `~/.gitconfig` never runs\" and \"a stage commit lands in a tmpRepo holding no global or system config at all\" therefore pin only part of the runner's git surface, and the committer-identity pin likewise lives only in `git()`.",
      "fix": "Scope the claim to `git()`'s calls, or make it \"every runner git call\" and name the shared env helper that `gitOk` and the `merge-base` call go through."
    },
    {
      "severity": "minor",
      "location": "Chosen design — Read-only git (item 5) + Compatibility matrix — \"a bare repo planted in the worktree is already inert, because `git()` pins `cwd: repoDir`\" / \"floor set by `safe.bareRepository=explicit`, which the runner's git env passes\"",
      "problem": "re-raised. The rewrite fixed the mis-attribution I asked about but left the flag orphaned: per git-config, `explicit` only refuses a bare repository discovered at the invocation's own cwd, and `git()`'s cwd is pinned with `.git` present there, so no sentence in the design now claims anything the flag guards — while the git ≥ 2.38 floor still rests on it and #13's \"a bare repo planted in the worktree is never the discovered repo\" passes with or without the flag.",
      "fix": "Either state the one case the flag does guard — `repoDir` itself turned into a bare repo (root-level `config`/`HEAD`/`objects/`, the only in-worktree config landing with no `.git` segment, invisible to the narrowed predicate and the audit) — and add that case to #13; or drop the flag and set the floor from the oldest feature actually used (`--no-optional-locks`)."
    },
    {
      "severity": "nit",
      "location": "Chosen design — Validator rule 7 — \"rule 2's `.git` segment test then runs on the resolved path made relative to that realpath, so a checkout under a `project.git` ancestor stays legal\"",
      "problem": "new. The relativization is the fix r1 asked for, but its stated reason went stale in the same revision: with the predicate narrowed to `.git` segment equality, `project.git` never matched it, so the \"so\" attributes legality to the wrong mechanism, and no test pins that layout (#13's `git worktree add` exercises the pointer-file case, not a `*.git` ancestor).",
      "fix": "End the sentence at \"…made relative to that realpath\"; if the bare-clone-with-worktrees layout matters, pin it as a rule 2/7 case in Test strategy #2."
    },
    {
      "severity": "nit",
      "location": "Options considered — \"**Agent git access, any scope.** Rejected … No named-command allowlist fixes that half\"",
      "problem": "new. The Non-goals line was narrowed this round to \"Any agent git write access, any scope\", which is what the design ships against; the unchanged Options heading and its last sentence now read as rejecting the four-command read-only allowlist the Chosen design ships (item 5).",
      "fix": "One clause: the rejection is of write-capable agent git access, and the read-only allowlist ships only with the audit plus `git()`'s config-free env — the same split ADR-005's Alternatives plus its item 5 make."
    }
  ]
}
```
