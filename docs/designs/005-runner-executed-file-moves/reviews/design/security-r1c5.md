# security review r1 (ratification pass c5)

verdict: request-changes

Scope: diff 6b21e2f..44c52fa of 02-proposal.md, then the full current document.
Checked against: my r1 findings (both cited sections were rewritten by this
revision, so both closures are in scope), the ADR, and the Revision log's r3
entries.

## Findings

- [major][new] Chosen design — Read-only git — "So neither the config sources
  outside the repo (A6) nor a planted `.git/config` or `.git/hooks` runs at
  runner privilege" — this claim is false as written and it is load-bearing in a
  way the r1 text was not: the r1 design I approved avoided the plant ("no
  attempt 2 and no finalize, since both would run in that worktree under the
  planted config"); this revision instead keeps the escalation's own
  `ESCALATION.md` commit (and every `pipeline resume` git call) running over the
  live plant and justifies that with this inertness claim. But the shared env
  suppresses only the *outside-repo* sources (`GIT_CONFIG_NOSYSTEM`,
  `GIT_CONFIG_GLOBAL` → empty file) and pins exactly two keys,
  `-c core.fsmonitor=false -c core.hooksPath=<empty dir>`. The planted root
  `.git/config` — which is exactly what #13's own `git log --output=.git/config`
  case lands, and which the sweep cannot see or undo (A4) — *is* the repo-local
  config the runner's next git call reads, at runner privilege. Its other
  exec-capable keys survive the pins: `alias.status = '!…'` (git expands an
  alias at the first non-option argument, shadowing the builtin, so the
  escalation's `gitOk`/`git add -A`/`git commit` path execs it), or
  `filter.evil.clean = '!…'` combined with a stage-owned `.gitattributes`
  (clean filters run at `git add` on commitStage). #13's ship gate cannot catch
  this because it pins a single hostile key: "the hostile config is a
  `core.fsmonitor` touching a marker file, which stays absent". Fix: either
  restore the r1 posture — on `git-plant-detected` the runner runs no git over
  the infected worktree (replace the escalation-commit sentence and row 1's
  "the plant pinned inert before the escalation's own commit") — or scope the
  claim to what the pins actually neutralize (fsmonitor, hooks), add the
  unplanted-not keys (`alias.*`, `filter.<driver>.clean`, `diff.external`) to
  the Deferred sandbox line, and extend #13's hostile config with one of them
  (e.g. `filter.evil.clean` matching a stage-written `.gitattributes`, marker
  stays absent across the escalation's `git add -A`) so the gate pins the claim
  the design makes.

- [minor][re-raised] Chosen design — Read-only git — "a child agent's `.git`
  write is audit-blind and the `subagent` deny below is the only cover for that
  route" (and Risks row 1: "child-agent writes are audit-blind, covered by the
  advisory `subagent` deny only") — the closure of my r1 finding 2 is
  incomplete: the deny ships on "the same two configs" only (the two item-5
  seats), while the doc's own allowlist parenthetical ("review seats already run
  the same two of four, `git diff*` and `git log*`") keeps non-item-5 seats
  running git in a writable worktree with no `subagent` deny — for those seats
  the child-session `.git` route has *zero* cover, not "the deny only", and it
  is placed nowhere: the widened Deferred sandbox bullet covers only "config
  writes landing outside the worktree or at its root with no `.git` segment",
  which an in-worktree `.git`-segment plant is not. Fix: one clause on the
  Deferred sandbox bullet placing the non-item-5 seats' child-session `.git`
  route there (my r1 ask, still not made), or extend the `subagent` deny to
  every seat whose shell can write the worktree.

## FYI (no severity)

- The "`GIT_CONFIG_GLOBAL` at an empty file" pin's location is unspecified; if
  the empty file lives inside the writable root, a stage can plant content into
  it (an in-worktree config write with no `.git` segment — currently Deferred,
  but the file's location is the difference between Deferred-by-design and a new
  writable config surface; implementation should put it outside the worktree).
- Compatibility row: lowering the git floor to 2.32 admits versions carrying
  local-surface CVEs that are fixed in later releases; no *unfixed* advisory
  applies to the runner's local-only command set, and no new dependency is
  named, so checklist item 5 does not block.
- Validator rule 7: the deleted rationale "so a checkout under a `project.git`
  ancestor stays legal" left the mechanism intact but also left that legal case
  untested anywhere in the Test strategy — correctness seat's call whether to
  pin it.

## Closures confirmed

- r1 finding 1 (#13's "a subagent-launched one" in the audit-escalation list):
  closed — #13 now reads "every `.git` write completed in the audited attempt's
  own session (`src/.git/config` plant, `git log --output=.git/config`)" and the
  subagent-launched case is gone, consistent with "Its input is one attempt's
  own session".
- Dropping `safe.bareRepository=explicit` (floor 2.38 → 2.32): consistent — the
  doc's cwd-pinning argument holds, the floor is set by `GIT_CONFIG_GLOBAL`
  (git 2.32, correct), and #13 still pins "never discovers a bare repo planted
  in the worktree". Not re-opened.
