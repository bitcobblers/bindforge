# Worktree guard: refuse to run from the main checkout on the default branch

`ensureBranch` (`runner.ts`) checks the run branch `pipeline/NNN-<slug>` out
*in the launch directory*. A `pipeline run` started from the main checkout
while it sits on `main` therefore moves that checkout off `main` for the
run's lifetime — the operator's working tree silently becomes the run's
scratch space. `docs/workflows.md` (How work enters / What a run produces)
now codifies the rule: a run is launched from a dedicated worktree, the
main checkout stays on `main`, and the operator squash-merges the run
branch back after `done`. This brief makes the runner enforce the launch
half of that rule. Keep every change inside `tools/orchestration`; existing
tests must stay green.

## 1. The guard

At the entry of both `runPipeline` and `resumePipeline`, before the lock is
taken and before any git mutation: if the launch checkout is the repo's
**main worktree** and its current branch is the repo's **default branch**,
refuse. The refusal is a clean error (exit 1, nothing created, no lock
left behind) whose text names the convention and where it is codified
(workflows.md, How work enters) and says to relaunch from a dedicated
worktree. For `resume`, the message also names the run's worktree — the
restart-boundary summary already does, so the guard's text should point at
the same record.

The two conditions matter separately, so the allow cases are part of the
contract:

- a **linked worktree** on the default branch is allowed — moving it
  disturbs nothing; the main-checkout hazard is about the *main worktree*.
  (How to tell them apart — e.g. `git rev-parse --git-common-dir` vs the
  worktree's own `.git` file, or `git worktree list --porcelain` — is the
  plan's choice; pin the behaviour, not the mechanism.)
- the **main worktree on a non-default branch** or detached HEAD is
  allowed — the operator deliberately put it off the default branch.
- default-branch detection: the `origin/HEAD` target when it resolves,
  falling back to `main`.

## 2. The test escape hatch

The scratch repos under `tests/pipeline/` are main worktrees on their
default branch, so the guard as stated would sink the whole suite. Add a
runner option that bypasses the guard, documented as test-only; the
production CLI must never set it. Pin it: a test asserts the CLI's call
sites pass no such flag, and the existing suite's scratch-repo runs are
the de facto pin that the bypass works.

## 3. Test pins

- main worktree, default branch, fresh run: refused; no `pipeline/NNN-<slug>`
  branch is created, no lock file is left behind, exit code 1.
- main worktree, default branch, `resume` of an existing run: refused; the
  message names the run's worktree.
- linked worktree on the default branch: the run proceeds (this is the
  codified happy path).
- main worktree on a non-default branch: the run proceeds.
- the production CLI path never enables the test bypass.
