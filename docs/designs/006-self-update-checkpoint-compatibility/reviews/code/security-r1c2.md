# Code review — security, round 1 pass c2 (ratification)

Scope as given: `git diff 640c289bdd01611c1fcd07a504af78eb59fb3fc7..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/006-self-update-checkpoint-compatibility.md`.
Inputs: that diff, the proposal, the ADR, my own r1c1 findings. No other review reports read.

## What the scope contains

`git log --oneline 640c289..HEAD` lists one commit, `2d49432 docs: 006-self-update-checkpoint-compatibility
code-review stage`, and `git diff --name-only 640c289..HEAD -- ':(exclude)docs/designs'` returns **no
files**. Every change in range is a run record under `docs/designs/**` (the run's `.pipeline.json`,
review reports, transcripts), which the scope deliberately excludes. `git diff --stat HEAD` shows only
the working-tree `.pipeline.json` run record. So the source and test tree at HEAD is byte-identical to
the tree I approved as the security seat in r1c1 — the line anchors in my r1c1 findings still land where
I recorded them (`runner.ts:2783–2792` is still the `finalizeDone` `try/catch` guard;
`checkpoint-integration.test.ts:279–326` is still the mid-run directory fixture that plants the directory
at the first prompt), which confirms the read being ratified is the read of this exact state.

Nothing landed after the approval can therefore have broken what I approved. I re-verified the checklist
items against HEAD rather than inferring the result from the empty diff:

1. **External input at the boundary** — `saveState` re-validates through `StateSchema.safeParse` before
   any write (`state-store.ts:318–324`); a refused snapshot rejects its caller only. No code evaluation
   of checkpoint content. Unchanged.
2. **Path handling** — the §1 fence compares `/`-separated segments case-insensitively against the
   reserved-name set (`types.ts:195–208`), so case tricks and deep burial are not routes around it; the
   destructive sweep derives its paths from NUL-separated git output
   (`status --porcelain=v1 -z --untracked-files=all --no-renames`, `runner.ts:225–229`;
   `diff-tree -z`, `runner.ts:221–223`), so quoting/unicode escapes cannot synthesize a path, and every
   removal goes through `resolve(repoDir, p)` with array-argument git calls — no `..`-from-untrusted-data
   or interpolated shell. Unchanged.
3. **Injection** — all git calls are `spawnSync('git', argsArray, …)`; `STATE_TMP_RE`
   (`types.ts:181`) is anchored with a bounded `{8}` character class — no backtracking exposure; tmp names
   come from `randomBytes`, not from checkpoint content. Unchanged.
4. **Secrets/PII** — the only new on-disk artifacts are the lock (`slug`, `pid`, `stage`, `at`) and
   `ESCALATION.md`; no credentials, and the escalation report renders gate reasons, not secrets. Unchanged.
5. **New dependencies** — none in range, none anywhere in the branch's source paths. Clean.
6. **Filesystem writes, least privilege and atomicity** — checkpoint writes are tmp+rename with a failed
   rename unlinking the tmp (`state-store.ts:326–347`); the escalation marker is tmp+rename
   (`runner.ts:2878–2883`); lock acquisition is a single `'wx'` exclusive create with the stale-holder
   posture (`runner.ts:455–506`) and the boundary re-assert re-takes with `'wx'` and refuses rather than
   running beside a rival (`runner.ts:536–570`); the lock is now a reserved segment and `stageAll` never
   stages it, so committing it is not a route to trusting it. Unchanged.
7. **Clean rejection** — `CheckpointWriteFailure` names the `rm -r` + `git checkout HEAD --` repair
   instead of a bare errno; the §3 branches convert unwritable-checkpoint crashes into escalations with a
   record, and `restoreCommittedCheckpoint` (`runner.ts:2810–2820`) keeps planted bytes out of the terminal
   commit. Unchanged.

## Findings

### minor — `tools/orchestration/src/pipeline/runner.ts:2783–2792`

**Problem:** carried forward unchanged from my r1c1 finding — the `done`-boundary save guard is correct by
inspection but untested. `finalizeDone` is module-private and no integration test fails the save at that
boundary: the mid-run directory fixture (`tests/pipeline/checkpoint-integration.test.ts:287–295`) plants
the directory at the first prompt, so escalation fires at a stage-tail save instead, and the §3-class
conversion at the completion boundary is a defensive branch no test exercises. I re-checked after the
later revisions and no test in `tests/pipeline/` references `finalizeDone` or a completion-boundary save
failure, so the gap is exactly as large as when I approved it — it is not a regression introduced by the
revisions, and it stays non-blocking for the same reason as in r1c1 (the branch's behavior at that
boundary is the same escalation posture the tested boundaries already prove).

**Fix:** add a fixture hook that replaces `.pipeline.json` with a directory only after the last
code-phase stage completes (or export `finalizeDone` for a unit test shaped like
`escalation.test.ts:280–324`), and assert the run ends `escalated` with `ESCALATION.md`, the `rm -r`
repair printed, and the escalation commit recorded — not a crash.

## FYI (no severity)

The c2 scope diff is empty of source and test paths; if the panel intended a later revision to be
reviewed here, it has not been committed — the only working-tree change is the run's `.pipeline.json`.

## Verdict

approve — zero open blocker/major findings; one carried-forward minor, unchanged by the revisions in range.
