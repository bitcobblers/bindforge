---
status: proposed
date: 2026-11-10
---

# ADR 006: Let stages declare file moves; the runner executes them

## Context

Run `004-mirror-src-hierarchy-in-tests` escalated with `tests not green at
the code-review budget limit` after six `fix <-> test-update` rounds, all
reproducing byte-identical failures. The cause (its build notes, Deviations
1 and 4): the approved change moves 23 test files, and **no stage agent can
rename or delete a file**. The stage shells allow only npm commands; the
runner is the only committer (`runner.ts:104`); the coder may not touch
`tests/**` while the test engineer owns them but cannot move them; a copy-
to-nested pass leaves both forms tracked and fails the layout assertions
worse. The pipeline therefore cannot execute any file move, and any ADR
that moves files livelocks exactly this way, spending its whole code budget
on six honest rounds that cannot change anything.

The fix is not agent git access. `commit`/`checkout`/`reset` would void the
two invariants the review protocol stands on: ADR-002 verdicts pin to
revision stamps (an agent that moves HEAD invalidates approvals), and
`enforceOwnership` works by diffing the dirty worktree against `headBefore`
(agent commits turn out-of-ownership edits into history the sweep cannot
revert). Even a narrow allowlist of named git commands leaks: a stage can
already write the worktree, so it can plant hooks or repo-local config that
any permitted `git` invocation then executes.

## Decision

Moves become data in stage replies, executed by the runner — the same shape
as every other stage mutation.

1. **Reply field.** The `implement`, `fix`, and `test-update` reply schemas
   gain `moves: { from: string, to: string }[]`, default empty, zod-
   validated at the checkpoint boundary like every reply field. Paths are
   repo-relative.
2. **Validate all pairs before touching anything.** Every `from` must be a
   tracked path at `headBefore`; every `to` must not exist; both endpoints
   must satisfy `stagePathAllowed` for the executing stage; neither may be a
   runner-reserved path (`STATE_FILE`, `.pipeline.lock`). Any violation
   rejects the whole `moves` list and routes the stage to retry, exactly
   like an invalid reply today.
3. **Runner executes.** On a valid list, the runner performs the renames
   through its own git plumbing during the stage's finalize - before the
   ownership sweep and `commitStage` - so the moves land in that stage's
   single commit, and rename detection survives (A15 of run 004's proposal
   showed 91-99% similarity for content-identical moves).
4. **Sweep accounts for sanctioned deletions.** `enforceOwnership` treats a
   deleted `from` path as sanctioned by the validated reply instead of a
   stray removal; unsanctioned deletions keep today's revert behavior.
5. **Read-only git in stage shells.** Stage agent shells additionally allow
   `git status`, `git log`, `git diff`, `git show` - plumbing that cannot
   write refs, the index, or HEAD. `git config` stays denied because config
   controls hook execution. This closes the smaller half of run 004's
   friction: fix stages burned rounds re-deriving state they could not even
   `git status` for.

## Alternatives considered

**Agent git access (any scope):** rejected as above - stamp integrity,
sweep inversion, and worktree-planted hooks executing through any allowed
git command.

**Keep moves operator-only (today):** rejected - it works, but every file-
moving ADR pays a full livelocked code budget plus one escalation before
the operator can land the move; run 004 proved the cost and the diagnosis
takes six rounds to state.

**`git mv` alone in stage shells:** rejected - the runner must stay the only
writer of the index and HEAD for the sweep and stamp logic to hold, and
`git mv` still resolves through repo-local config and hooks.

## Consequences

Becomes possible: file-moving ADRs run end to end with no operator stop;
the move lands reviewably in one stage commit.

Accepted costs: finalize gains a validated mutation surface (bounded by
item 2's checks); `moves` is a new checkpoint field (additive with
defaults, ADR-003's rule); stage prompts for implement/fix/test-update
must teach the field. Untouched: review protocol, gates, ownership
semantics for non-move edits, and ADR-005's checkpoint-trust scope.

Ordering: run after 004 completes. Dogfood acceptance: a later run of any
file-moving change, green without a single escalation.
