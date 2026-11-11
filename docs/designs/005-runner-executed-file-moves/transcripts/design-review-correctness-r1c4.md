# Transcript: design-review:correctness r1c4

## Provenance
- stage: design-review:correctness (round r1c4)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T18:06:10.920Z
- ended: 2026-11-10T18:08:55.826Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c4: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 6b21e2f4897d0c0e93a8694889904bcfdbf4f922 44c52fa6533e711e4a34c1987afa1d6694260c86` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design — Read-only git (item 5) — \"A hit ends the stage as `git-plant-detected: <path>`, escalating to the operator like `moves-rollback-failed`: no attempt 2 and no finalize, since both would run in that worktree under the planted config and the `since` window never re-detects it.\"",
    "problem": "new. Escalation is not git-free in this runner, so the new terminal class neutralizes nothing: the escalation path ends in `finalizeEscalate`, which writes ESCALATION.md and then calls `commitStage` (runner.ts:2008-2013) — `git add -A` (298) and `git commit -m` (307, no `--no-verify`) in the same worktree; the review-pass escalation path likewise still sweeps (`finalizeReviewPass(..., false)` at runner.ts:1313, whose own comment at 1089-1091 says \"the sweep still runs\") via `git rev-parse`/`git status`/`git restore` (180, 213, 236). So the act of escalating is itself a runner-privilege git call over the plant: `git add -A` refreshes the index and fires a planted `core.fsmonitor`, and `git commit` executes a planted `.git/hooks/pre-commit`. #13's new containment assertion — \"with the plant in place the runner's next git call must not execute it — the hostile config is a `core.fsmonitor` that touches a marker file, which stays absent\" — fails against the design as written, and the paragr… [elided by the checkpoint cap]",
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
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/005-runner-executed-file-moves.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/correctness-r1c4.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design — Read-only git (item 5) — \"`git()`'s env (106-112) becomes the shared env for every runner git call — `gitOk` (143-145) and the raw `merge-base` spawn (1417-1420) included\"","problem":"re-raised. My r1 env-scope finding is partly closed — the text now claims \"every runner git call\" and names gitOk and merge-base, and #13, the Risks row and the Compatibility row follow — but the enumeration still misses a fourth spawn site: `writeEditorialDiff` (called from runner.ts:1507 in the review/editorial loop) runs `spawnSync('git', ['diff','--no-index',…], {cwd: repoDir})` at runner.ts:1933-1937, outside `git()`/`gitOk`, inheriting the operator's environment and reading the worktree's config. So the absolute claim in the paragraph and in #13's gate (\"every runner git call, `gitOk` and `merge-base` included\") is false as written, and an implementer who treats the list as exhaustive leaves that call un-pinned.","fix":"Name all non-`git()` sites: \"`gitOk` (143-145), the raw `merge-base` spawn (1417-1420) and `writeEditorialDiff`'s `git diff --no-index` (1933-1937) included\", and add the editorial-diff call to #13's \"every runner git call\" list; or state the rule as \"every `spawnSync('git')` site goes through the one env helper\", which keeps the claim true as sites change."},{"severity":"minor","location":"Chosen design — Read-only git (item 5) — \"So neither the config sources outside the repo (A6) nor a planted `.git/config` or `.git/hooks` runs at runner privilege\" (and the Risks row's \"nothing outside the repo executes at runner privilege\")","problem":"new. The conclusion is wider than the mechanism named two sentences earlier. `GIT_CONFIG_NOSYSTEM`/`GIT_CONFIG_GLOBAL` suppress only the system and per-user levels (I confirmed against git(1): setting GIT_CONFIG_GLOBAL means neither $HOME/.gitconfig nor $XDG_CONFIG_HOME/git/config is read), and `-c core.fsmonitor=false -c core.hooksPath=<empty dir>` overrides exactly two keys; the repo-local `.git/config` is still read, and A6 — cited in this same paragraph — states config is also read via \"`include.path` from any of them\". A planted `.git/config` with `[include] path = /home/<operator>/.gitconfig` re-opens the outside-repo sources at runner privilege, and other exec-capable repo-local keys that fire in the escalation's own git calls (e.g. `filter.<driver>.clean` under `git add -A` at runner.ts:298, selected by a planted `.gitattributes`, which is not a `.git` segment and so is neither config-denied nor audit-matched) are untouched by the pins. #13's fsmonitor-marker fixture passes while the blanket guarantee stays false.","fix":"Scope the sentence to what ships: \"neither the system nor per-user config sources A6 names are read, and the two A6 mechanisms that fire in these calls — `core.fsmonitor` on an index refresh and hooks — are overridden; other exec-capable repo-local keys (filters, external diff) and an `include.path` inside a planted `.git/config` are the Deferred sandbox's\" — or add the pins that would make the blanket form true."},{"severity":"nit","location":"Chosen design — Read-only git (item 5) — closing sentences of the paragraph (previously \"If Test strategy #13 lets a vector land a plant, or a runner git call run over one, the change does not merge.\")","problem":"new. This revision dropped the merge-blocking clause. Goals still promises item 5 lands \"conditional on the controls and ship gate named under Read-only git\", and #13 still labels itself \"Item 5's ship gate asserts containment, not just detection\", but the section that names the gate no longer states its consequence, so the condition the Goals row relies on has no stated effect.","fix":"Restore the one sentence at the end of the paragraph: \"If Test strategy #13 lets a vector land a plant, or a runner git call run over one, the change does not merge.\""}]}
```
