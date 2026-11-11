# correctness review r1 (dissent pass c3)

verdict: request-changes

Scope: `git diff 89027e8 6b21e2f` over `02-proposal.md`, then the full current
document. Checklist: `references/correctness.md`. Re-checked r1 findings 1–5
against the r3 text and the Revision log; spot-checked the two citations the
changed text leans on hardest — https://opencode.ai/docs/permissions/ (Matching,
Actions, Defaults, Agents: `subagent` action with resource = target agent ID;
"A custom subagent uses its own permissions, not a subset of its parent's
permissions"; whole-value wildcards; case-insensitive matching only on Windows)
and https://git-scm.com/docs/git-config (`safe.bareRepository=explicit`: "Git
only works with bare repositories specified via the top-level `--git-dir` … or
`GIT_DIR`"; "only respected in protected configuration", which `-c` is). Both
support A9's new clause and the floor row's 2.38 basis. Rule 7's relativization
(r1 #3), the predicate/mirror split (r1 #5), and the nested-repo attribution
(r1 #4's first half) are closed as asked.

## Findings

- [major][new] Chosen design — Read-only git (item 5) — "A hit ends the stage as
  `git-plant-detected: <path>`, escalating to the operator like
  `moves-rollback-failed`: no attempt 2 and no finalize, since both would run in
  that worktree under the planted config and the `since` window never re-detects
  it." — Escalation is not git-free in this runner, so the terminal class
  neutralizes nothing: the escalation path ends in `finalizeEscalate`, which
  writes `ESCALATION.md` and then calls `commitStage` (runner.ts:2008-2013) —
  `git add -A` (298) and `git commit -m` (307, no `--no-verify`) in the same
  worktree; the review-pass escalation path likewise still sweeps
  (`finalizeReviewPass(..., false)`, runner.ts:1313, whose own comment at
  1089-1091 says "the sweep still runs") via `git rev-parse`/`git status`/`git
  restore` (180, 213, 236). So the act of escalating is a runner-privilege git
  call over the plant: `git add -A` refreshes the index and fires a planted
  `core.fsmonitor`, and `git commit` executes a planted `.git/hooks/pre-commit`.
  #13's new containment assertion — "with the plant in place the runner's next
  git call must not execute it — the hostile config is a `core.fsmonitor` that
  touches a marker file, which stays absent" — therefore fails against the
  design as written, and the design's own note that "the `since` window never
  re-detects it" leaves the plant live for an operator `pipeline resume`
  (runner.ts:1376) as well. Fix: inside the escalation, name the
  neutralization that runs before any git — the runner deletes the detected
  `.git` path (or pins `-c core.fsmonitor=false -c core.hooksPath=<empty dir>`
  on the escalation-path calls) — replace "no finalize, since both would run in
  that worktree under the planted config" with that step, and have #13's marker
  fixture assert it against `finalizeEscalate`'s commit rather than a bare
  "next git call"; say in the same sentence what `resume` requires of the
  operator (the plant is not cleared by anything in-process).

- [major][re-raised] Chosen design — Read-only git (item 5) + Test strategy #13 —
  "The same two configs append a `subagent` deny: the audit judges one attempt's
  own message list, and gates.ts:257 names subagents as the route around config
  denies" / "every completed `.git` write (`src/.git/config` plant, `git log
  --output=.git/config`, a subagent-launched one) escalates the stage" — the r3
  answer adds a config deny, which is real progress, but the ship gate now
  asserts something the named mechanism cannot produce. The audit reads
  `lastSeen`, the one session id the attempt polled (runner.ts:745-769, 839;
  independence.ts:11 "its own message list"); no text names anything that
  enumerates the attempt's child sessions and feeds their completed
  `write`/`edit`/`patch`/shell parts into the matcher. So with the deny in force
  the launch is refused and there is no child write part to escalate, and with
  the deny routed around — the case the paragraph exists for, since it opens
  "agent-config permissions are only advisory" — the child's `.git` write is
  invisible to the audit and the stage commits under it. Either way #13's
  subagent case cannot pass as an *audit* assertion. Fix: pick one and say it in
  one clause. (a) Add the enumeration the assertion presumes — the runner walks
  the attempt's child sessions reached from its completed `task` parts and runs
  the same matcher over their parts — and keep #13's case; or (b) drop "a
  subagent-launched one" from #13's escalate list, keep it in the refused-by-
  config list, and record the subagent route as covered by the advisory deny
  only (audit-blind) in the paragraph and the Risks row.

- [minor][new] Chosen design — Read-only git (item 5) — "`git()` (runner.ts:106-112)
  stops reading ambient config — `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL` at
  an empty file … so the outside-repo config sources A6 names never execute at
  runner privilege" (and the Risks row's "nothing outside the repo executes at
  runner privilege") — the blanket scope is wider than the single helper that
  ships the env. Two other runner git spawns bypass `git()`: `gitOk`
  (runner.ts:143-145, used by `blobStamp`'s `git cat-file -e`) and the
  `merge-base` call at runner.ts:1417-1420, both plain `spawnSync('git', …)`
  that inherit the operator's environment. #13's "A hostile `~/.gitconfig`
  never runs" and the "a stage commit lands in a tmpRepo holding no global or
  system config at all" cases therefore pin only part of the runner's git
  surface, and the committer-identity pin is likewise only in `git()`. Fix:
  scope the claim to `git()`'s calls, or make it "every runner git call" and
  name the shared env helper that `gitOk` and the `merge-base` call go through.

- [minor][re-raised] Chosen design — Read-only git (item 5) + Compatibility
  matrix — "a bare repo planted in the worktree is already inert, because
  `git()` pins `cwd: repoDir`" / "floor set by `safe.bareRepository=explicit`,
  which the runner's env passes" — the r3 rewrite fixed the attribution I asked
  about but left the flag orphaned: per git-config, `explicit` only refuses a
  bare repository discovered at the invocation's own cwd, and `git()`'s cwd is
  pinned with `.git` present there, so no text now claims anything the flag
  guards — while the git ≥ 2.38 floor still rests on it and #13's "a bare repo
  planted in the worktree is never the discovered repo" passes with or without
  the flag. Fix: either state the one case the flag does guard — `repoDir`
  itself turned into a bare repo (root-level `config`/`HEAD`/`objects/`, the
  only in-worktree config landing with no `.git` segment, which the narrowed
  predicate and the audit do not see) — and add that case to #13; or drop the
  flag and set the floor from the oldest feature actually used
  (`--no-optional-locks`).

- [nit][new] Chosen design — Validator rule 7 — "rule 2's `.git` segment test
  then runs on the resolved path made relative to that realpath, so a checkout
  under a `project.git` ancestor stays legal" — the relativization is what r1
  asked for, but the stated reason went stale in the same revision: with the
  predicate now `.git` segment equality, `project.git` never matched it, so the
  "so" attributes legality to the wrong mechanism and no test pins that layout
  (#13's `git worktree add` exercises the pointer-file case, not a `*.git`
  ancestor). Fix: end the sentence at "…made relative to that realpath"; if the
  bare-clone-with-worktrees layout matters, pin it as a rule 2/7 case in #2.

- [nit][new] Options considered — "**Agent git access, any scope.** Rejected
  … No named-command allowlist fixes that half" — the Non-goals line was
  narrowed this round to "Any agent git **write** access", which is what the
  design ships against; the unchanged Options heading and its last sentence now
  read as rejecting the four-command read-only allowlist the Chosen design
  ships (item 5). Fix: one clause — the rejection is of write-capable agent git
  access, and the read-only allowlist ships only with the audit plus `git()`'s
  config-free env, exactly the split ADR-005's own Alternatives plus item 5
  make.

## FYI (no severity)

- The `subagent` deny's resource pattern is not stated: `resource: "*"` denies
  every child agent for those seats, which also removes any legitimate use of a
  child agent by a coder/test-engineer seat; a narrower deny list (the general
  subagent, the seats that carry their own write allow) may be what the author
  means. Compatibility/complexity seat's call.
- The Revision log labels this round's decisions `r3` while the run's report
  paths say round 1 (dissent pass 3); an editor may want the labels aligned.
- #12's macOS contrast (`notes.md` → `readme.md` in a repo tracking
  `README.md`) is a good A10 probe, but it passes rule 6 only because rule 6
  folds comparisons among *listed* paths, not against the index; that
  distinction is implicit in rule 5's wording and worth one clause for whoever
  implements it.
