# Design: Runner-executed file moves (ADR-005)

## Context

ADR-005 (docs/adrs/005-runner-executed-file-moves.md) makes file moves data in
stage replies: the `implement`, `fix`, and `test-update` replies gain a
`moves: { from, to }[]` field, the runner validates the whole list before
touching anything, and the runner — still the only committer — executes the
renames during the stage's finalize, before the ownership sweep and
`commitStage`. Run 004 escalated with `tests not green at the code-review
budget limit` because its 23 test-file moves were inexpressible to any stage
(A12). ADR-005 defers its earlier item 5, read-only git in stage shells, to a
follow-up ADR, so this change ships items 1-4 only.
Grounding: docs/designs/005-runner-executed-file-moves/01-research.md; this run
follows run 004 (ADR-005, Ordering).

## Goals / Non-goals

- Goals:
  - Implement/fix/test-update stages can declare file moves; the runner lands
    them in that stage's single commit with rename detection intact.
  - A malformed `moves` list fails before anything is touched, on exactly the
    existing invalid-reply retry/escalation path.
  - A failed apply never leaves a half-applied list inside finalize.
- Non-goals:
  - Any agent git write access, at any scope (ADR-005 Alternatives; see Options 1).
  - Directory moves and case-only renames (see Deferred).
  - Review protocol, gates, and non-move ownership semantics stay untouched
    (ADR-005, Consequences); the checkpoint-trust question belongs to ADR-006.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | Today's reply contracts carry no move primitive: `test-update` is `{commitMessage}`, `implement`/`fix` add `testsNeedUpdate` | verified | tools/orchestration/src/pipeline/types.ts:44-46 |
| A2 | One choke point validates replies: `safeParse`, then nudges, then `invalid-reply:`-prefixed failure → same-session attempt 2 → second failure escalates | verified | tools/orchestration/src/pipeline/runner.ts:912-950, 1121-1126, 1453-1497 |
| A3 | Every `fix` entry burns one code-budget unit, so a rejected reply is a budget event | verified | tools/orchestration/src/pipeline/runner.ts:1712-1714 |
| A4 | The sweep lists dirty paths with `--no-renames` (a rename is delete+add), reverts only paths failing `stagePathAllowed`, and restores violations from `headBefore` | verified | tools/orchestration/src/pipeline/runner.ts:159-182, 231-247 |
| A5 | Rename detection is a diff-time computation, on by default, at a 50% similarity threshold | verified | https://git-scm.com/docs/git-diff |
| A6 | The four read-only-named commands carry write/exec-capable config: `--output=<file>` on `git log`/`git diff`, `--ext-diff`/`--textconv` run config-defined helpers, `diff.external` makes a plain `git diff` exec, `core.pager` is shell-interpreted, `core.sshCommand` and `filter.<driver>.clean`/`smudge` run at git's choice, and `core.fsmonitor` fires on an index refresh — which `git status` and `git diff` perform by default, `git --no-optional-locks` aside. That config is read from more than one place: repo-local `.git/config`, `$XDG_CONFIG_HOME/git/config` and `~/.gitconfig`, `/etc/gitconfig`, and `include.path` from any of them | verified | https://git-scm.com/docs/git, https://git-scm.com/docs/git-status, https://git-scm.com/docs/git-log, https://git-scm.com/docs/git-diff, https://git-scm.com/docs/git-config |
| A7 | A stage agent can write `.git/**` today: agent configs carry no `.git` rule, and the ownership matrix lets `implement`/`fix` touch everything except tests | verified | .opencode/agents/code/coder.md:5-7; tools/orchestration/src/pipeline/gates.ts:260-266 |
| A8 | `fs.renameSync` is a thin wrapper over `rename(2)`: atomic, fails `EISDIR`/`EXDEV`/`EINVAL` | verified | https://nodejs.org/api/fs.html; https://man7.org/linux/man-pages/man2/rename.2.html |
| A10 | On a case-insensitive checkout the on-disk dentry check rejects a move endpoint whose name differs from an on-disk file only in case | unverified | mechanism from an `lstatSync` path lookup + `core.ignoreCase` (https://git-scm.com/docs/git-config); never exercised — run 004 parked the same question as unverified A16 (docs/designs/004-mirror-src-hierarchy-in-tests/02-proposal.md:48-49) |
| A11 | In a linked-worktree checkout `.git` is a `gitdir:` pointer file and the real git directory lives outside the writable root | verified | docs/designs/005-runner-executed-file-moves/01-research.md (Platform notes) |
| A12 | Run 004 livelocked 6 fix↔test-update rounds over 23 inexpressible test-file moves and escalated; the operator landed the moves | verified | docs/designs/004-mirror-src-hierarchy-in-tests/03-build-notes.md:27-59; .pipeline.json gateLog 1181-1201 |
| A13 | Checkpoint growth is additive-safe: `version: 1` unchanged, every post-run-1 field optional, `saveState` re-validates | verified | tools/orchestration/src/pipeline/state-store.ts:145-155, 178-184; docs/adrs/003-parallel-inference-lane.md:115-116 |

## Options considered

**Agent git write access, any scope.** Rejected (inherited from ADR-005's
Alternatives): `commit`/`checkout`/`reset` would invalidate ADR-002 revision stamps
and invert the sweep, which diffs the dirty worktree against `headBefore`; and a
stage that can write the worktree can plant config or hooks that any permitted git
command then executes (A6, A7); no write-capable command allowlist fixes that half.

**Runner executes each pair with a `git mv` subprocess.** Rejected: `commitStage`
already updates the index with `git add -A` (runner.ts:298) and rename detection is
diff-time (A5), so a per-pair subprocess only multiplies A6's config surface.

**Sequential apply, stop at first error (RFC 6902 order).** Rejected: it leaves a
half-applied list as the normal failure state; the ADR wants validate-all-then-apply,
and this design adds in-process rollback so a failed apply is all-or-nothing too.

**Allow directory moves.** Rejected: a directory `from` needs self-containment
(RFC 6902, `rename(2)`'s `EINVAL`, A8), per-child cascade rules, and per-child
tracking at `headBefore`; run 004's payload was 23 file pairs (A12), so
file-only drops that whole rule class.

**A new sweep branch keyed on reply moves (literal ADR-005 item 4).** Rejected:
unreachable, since the shared `stagePathAllowed` invariant already gives item 4's
behavior (see Sweep interaction, pinned by Test strategy #4).

## Chosen design

**Reply field** (`tools/orchestration/src/pipeline/types.ts`). A shared
`MovePairSchema = z.object({ from: z.string().min(1).max(200), to: z.string().min(1).max(200) })` and `moves: z.array(MovePairSchema).default([]).max(64)`
are added to the `implement`, `fix`, and `test-update` entries of
`StageReplySchemas`
(types.ts:44-46). The bounds cap the validation and apply work one reply can make the
runner spend; an endpoint over the length cap or a list over the pair cap fails
`safeParse` like any invalid reply. Optional
with default `[]` keeps the checkpoint additive per A13; `version` stays 1 and
`state-store.ts` is untouched. `replyFormat` renders the schema into every prompt
(runner.ts:709-710), so the field documents itself to the model.

**Validator** (new `tools/orchestration/src/pipeline/moves.ts`,
`validateMoves(repoDir, headBefore, stageName, moves)`). All checks run against
`headBefore` plus the current disk state before anything is touched; every
failure names the offending pair. Rules:

1. Shape: repo-relative POSIX paths compared with `path.posix` segment
   semantics, each endpoint free of `\`, NUL, control characters, and `:`, with
   no absolute path, no `.`/`..` segment, and no segment that is a Windows
   reserved name or ends in `.` or space; `from !== to`.
2. Reserved: no path segment equals `STATE_FILE` (`.pipeline.json`,
   types.ts:143) or `LOCK_FILE` (`.pipeline.lock`, runner.ts:102); no endpoint
   has a case-folded `.git` segment — the pointer file, `.git/**`, or a nested
   `.git/` in an owned dir (a validator-local rule, not a `stagePathAllowed`
   change).
3. Ownership: `stagePathAllowed(stageName, p)` for both endpoints
   (gates.ts:260-273) — so `test-update` can move tests, `implement`/`fix`
   cannot move into tests.
4. `from` is a tracked regular file at `headBefore` (`git ls-tree
   <headBefore> -- <from>` mode 100644/100755 — this is the file-only rule;
   trees and mode-120000 symlinks fail it) and still exists on disk.
5. `to` does not exist anywhere: no disk dentry (`lstatSync` try/catch — a
   dangling symlink is a dentry and `rename(2)` replaces it silently; this is
   also the case-insensitive-checkout guard, A10) and not tracked at
   `headBefore`, while its parent exists, so a missing parent fails as a reply
   error here rather than as an environmental apply failure later.
6. Disjoint pairs: every comparison here is case-folded on every platform — all
   `from`s distinct, all `to`s distinct, no `to` equals any `from`, no listed path
   a path-prefix of another — rejecting swaps (`A→B, B→A`), chains (`A→B, B→C`),
   moves into a moved-out path, and case-only moves (`README.md` → `readme.md`);
   the fold is deliberate conservatism, since a repo with case-variant filenames
   cannot move both in one reply. What remains is a matching whose apply order is
   irrelevant.
7. Containment: the realpath of `from` and of `to`'s parent (rule 5) stay inside the
   repo's realpath, which also catches a symlink escaping into the real gitdir; rule
   2's `.git` segment test then runs on the resolved path relative to that realpath.

**Wiring into retry** (`runner.ts`). `executeInference` gains an optional
`headBefore` parameter, passed only by `executeStage` (runner.ts:1077-1079);
review schemas have no `moves`, so `runReviewPass` passes nothing. After
`schema.safeParse` succeeds (runner.ts:912-916), if the parsed reply carries a
non-empty `moves`, run `validateMoves`; a failure — a returned issue or a throw
from a rule — takes the existing schema-failure branch (nudge with the issue
message, then final failure `invalid-reply: moves — <issue>`), so
`retryInSameSession` (runner.ts:1121-1126), the repair prompt, and
second-failure escalation behave exactly as for any invalid reply.

**Apply** (`moves.applyMoves(repoDir, headBefore, moves)`, called from
`finalizeStage` between reply acceptance and the `enforceOwnership` call, i.e.,
before runner.ts:1049, ahead of `commitStage` at 1053). Immediately before the
first rename it re-runs rules 4–7 as one guard across the inference→finalize
window; a guard refusal takes the same `moves-apply-failed` outcome and
preserved worktree as a rename error. Then per pair, in reply order,
`fs.renameSync` (A8), under the `.pipeline.lock` single-writer invariant
(runner.ts:354-383). On the first error, completed renames roll back in
reverse, which is exact because the pairs are disjoint, and the outcome becomes
`moves-apply-failed: <error>` without the `invalid-reply` prefix: the failure
being environmental, attempt 2 starts a fresh session. Finalize returns without
sweeping or committing, leaving the preserved-worktree state that a failed
reply leaves today (runner.ts:1046-1048). A rollback rename that itself fails
is the one state nothing in-process can undo: `moves-rollback-failed: <error>`
escalates to the operator instead of starting attempt 2 on a known-damaged
worktree.

After a successful apply the existing machinery finishes the job: the sweep
leaves the pair alone (Sweep interaction, below), `git add -A` stages
delete+add, and reviewers see one commit whose `git show --name-status -M`
reports `R` entries at the default 50% threshold (A5).

**Sweep interaction**. No new code in `enforceOwnership`: a validated pair's
endpoints satisfy `stagePathAllowed` for the executing stage, so neither the
deleted `from` nor the new `to` enters the violation list (A4) and unsanctioned
deletions keep today's restore-from-`headBefore` behavior (Options 5, #4).

**Prompts**. One sentence each in `tools/orchestration/prompts/implement.md`,
`fix.md`, `fix-tests.md`, and `test-update.md`: to move or rename a file, do not
touch the paths yourself — declare it in `moves`, and the runner performs it.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Moves validation failures burn code budget (A3) | medium | medium | In-attempt nudge loop repairs before the attempt fails; validator messages name the offending pair; prompts teach the field |
| `to`/`from` state changes between validation and apply (TOCTOU) | low | medium | `applyMoves` re-runs rules 4–7 as one guard immediately before the first rename; reverse rollback; `.pipeline.lock` single-writer invariant (runner.ts:354-383) |
| Crash between apply and commit leaves a dirty half-state | low | medium | Converges on resume by ownership: the next attempt re-captures `headBefore` and the sweep keeps or restores the pair; a rollback rename that itself fails stops at the operator as `moves-rollback-failed` |
| Sweep semantics now rest on "validator and sweep share `stagePathAllowed`" (item 4 as invariant) | low | medium | One predicate, one call site each (gates.ts:260) — a future fork of the predicate breaks Test strategy #4 rather than silently breaking production; broader reply-as-authorization trust belongs to ADR-006 (Deferred) |
| Prompt-pin drift (`fix-tests.md` content assertions) | high | low | Test strategy #7 updates `fix-trigger.test.ts` and `prompts.test.ts` pins in the same commit |
| Case-insensitive checkouts behave unexpectedly (A10) | medium | low | Rule 6's comparisons and the `.git` endpoint predicate are case-folded on every platform; rule 5's disk check adds the checkout-level guard; macOS is `yes (unverified)`, Windows not exercised |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux, case-sensitive ext4 (dev + CI) | yes | reference platform; every test runs here but the operator-run cases named below |
| macOS (APFS, case-insensitive default) | yes (unverified) | Test strategy #12 runs one move round trip there plus a `notes.md` → `readme.md` move in a repo tracking `README.md`: macOS rejects it at rule 5's disk check where Linux lands a case-variant sibling — that contrast is the A10 evidence |
| Windows (NTFS, `core.protectNTFS`) | not exercised | this repo's pipeline is never run on Windows; rule 1 rejects `\`-separated and Windows-illegal names identically on every platform, pinned on Linux in #2 |
| Linked git worktree checkout (`.git` pointer file) | yes | the `.git` segment rule covers the pointer, `.git/**` and any nested `.git/`; real gitdir is outside the writable root (A11) |
| Repository spanning mount points | partial | `rename(2)` fails `EXDEV` (A8): the pair fails at apply, the list rolls back, the stage fails with `moves-apply-failed`; #5 names the case, operator-run since mounting a tmpfs needs capability the vitest suite lacks |
| Node ≥ 22.23.4 (engines raised from ≥22.12.0, tools/orchestration/package.json:30-32) | yes | no new runtime dependency: `fs.renameSync` plus the existing git helper; 22.23.4 is the 22.x line containing the October 2026 security release (Brotli RCE CVE-2026-111497 et al.) |
| git, any version this repo runs today | yes | no new floor: the only git call this change adds is the validator's `git ls-tree` (rule 4); rename detection stays at git's default (A5) |

## Test strategy

1. Unit — reply schemas (extend `tests/pipeline/reply-contract.test.ts`): a
   missing `moves` parses to `[]`; malformed pairs (non-string, empty `from` or
   `to`), an endpoint over the length cap, and a list over the pair cap fail
   `safeParse` like any invalid reply.
2. Unit — validator rejections (new `tests/pipeline/moves.test.ts`), one case
   per rule clause: untracked `from`; `from` a tree or tracked symlink; `from`
   deleted on disk; `to` on disk, dangling symlink included; `to` tracked at
   `headBefore`; `to`'s parent missing; `from === to`;
   absolute path, `.`/`..` segments, and an endpoint with `\`, a control
   character, `:`, or a Windows reserved name — that last asserting a rule that
   throws still lands as `invalid-reply: moves`, not a crash; `STATE_FILE` and
   `LOCK_FILE` segments; `.git`, `.git/config`, `.GIT/hooks/pre-commit` and
   `src/.git/config` endpoints; `stagePathAllowed` failures in both directions
   (`test-update` out of `src/**`, `implement` into `tests/**`); duplicate
   endpoints, and a lone case-only pair (`README.md` → `readme.md`); swap `A→B,
   B→A`; chain `A→B, B→C`; containment; a symlinked endpoint escaping the repo or
   landing in `.git`.
3. Unit — validator accepts a legal pair whose `from` carries uncommitted
   content edits.
4. Unit — sweep interaction (`tests/pipeline/ownership.test.ts`): an in-ownership
   move pair (delete + untracked add) is absent from `report.reverted` and stays
   applied after `enforceOwnership`; an unsanctioned delete of the same shape is
   still restored from `headBefore`; the existing rename-reverts test
   (ownership.test.ts:148-159) still passes.
5. Unit — `applyMoves`: pairs land in order carrying dirty content; the rules 4–7
   guard refuses a list whose `from` is untracked at the `headBefore` handed in,
   and that refusal takes the `moves-apply-failed` class of #6; a pair failing
   mid-list rolls back in reverse, leaving the disk byte-identical; a pair whose
   `to` parent is on another mount (tmpfs under the worktree, operator-run and
   skipped where the mount capability is absent) fails `EXDEV`; a rollback rename
   that fails yields the escalation class of #6.
6. Unit — failure classification: `retryInSameSession('invalid-reply: moves
   …', sid)` is true; `'moves-apply-failed: …'` is false; a `'moves-rollback-failed:
   …'` outcome escalates to the operator and starts no attempt 2.
7. Unit — prompt pins: `prompts.test.ts` and `fix-trigger.test.ts` updated for
   the moves sentence in `implement.md`, `fix.md`, `fix-tests.md`,
   `test-update.md`.
8. Integration — full `executeStage` round trip with a scripted reply carrying
   `moves`: exactly one stage commit, whose `git show --name-status -M` reports an
   `R` entry per pair and which also carries the stage's content edits.
9. Integration — invalid list: attempt 1 fails with the `invalid-reply: moves`
   reason, attempt 2 resumes the same session with a repair prompt quoting the
   failure (extends reply-contract.test.ts:172-182); a second rejection escalates
   with the validator message in the reason.
10. Integration — budget: a `fix` stage whose moves list is rejected burns
    exactly one `codeBudgetUsed` (extends `budget.test.ts`).
11. Integration — apply failure: a pair that can no longer be applied between validation
    and apply produces no commit, a byte-identical worktree, and an
    un-swept preserved worktree for attempt 2 (same shape as a failed reply).
12. Platform — on macOS, one move round trip plus a `notes.md` → `readme.md` move
    in a repo tracking `README.md`: rule 5's disk check rejects it there, while
    Linux lands the pair as a case-variant sibling — that contrast is the A10
    evidence; Windows gets no observation and pins rule 1 on Linux in #2.
13. Platform — `git worktree add` tmpRepo round trip: the move stage passes and
    `.git` — the pointer file included — is untouched.

## Rollout

1. One merged branch; ADR-005's `status: proposed → accepted` flip happens on
   this run's done-gate (gates.ts:230-238), like every ADR run.
2. No migration: checkpoint `version` stays 1, `moves` is optional, defaulting
   to `[]` (A13), so in-flight and old checkpoints re-parse unchanged
   (state-store.ts:145-155).
3. No config knob or feature flag: with the default `[]`, behavior is today's
   exactly; the feature is inert until a stage emits a non-empty list.
4. Dogfood acceptance (ADR-005, Consequences): the next file-moving change run shows
   no `operatorForce`/`operatorExtension` gate entries and no `stageFailure`
   reason containing `moves`, and its moves land as `R` entries
   (`git show --name-status -M`) in one stage commit.

## Deferred / out of scope

- Directory moves as single pairs — this ADR-005 if a real change needs them;
  file pairs cover the motivating case; no follow-up yet.
- Case-only renames on case-insensitive checkouts — support, not just
  rejection, deferred; no follow-up yet (A10).
- OS/sandbox-level protection of stage shells (protected paths, argv vetting, exec
  controls, config writes landing outside the worktree or at its root with no `.git`
  segment) — the OpenCode harness owns it; Node's own `--permission` model is not an
  acceptable substitute for it; no follow-up yet.
- The checkpoint/reply trust boundary that item 4's invariant leans on — owned
  by docs/adrs/006-self-update-checkpoint-compatibility.md.
- Read-only git in stage shells — ADR-005 defers the earlier item 5 to a follow-up
  ADR: its `.git`-config attack surface is separable and non-terminating, and items
  1-4 stand without it; the same deferral covers any new read-only git for review,
  research, or doc-sync seats beyond the `git diff*` and `git log*` they carry.

## Revision log

- r0: research OQ1 (execution primitive) — Chosen design: `fs.renameSync` plus the
  existing `git add -A`; no `git mv` subprocess.
- Validator (rules 4-5): file-only = `git ls-tree` regular-file mode; `to` must be
  no dentry (`lstatSync` try/catch) — r4 replaced `cat-file`/`existsSync`, which
  admitted tracked symlinks and planted dangling links; parent-required, case
  folding and the `.git` denial are validator-local, not a `stagePathAllowed` change.
- r0/r1/r3: failure classes (Apply) — `invalid-reply:` nudge; a failed apply or guard
  refusal is `moves-apply-failed`; a rollback failure escalates to the operator.
- r0: ADR-005 item 4 — no new sweep branch; the shared `stagePathAllowed` gate is
  the sanction, pinned by #4.
- Read-only git (Deferred) — item 5 is out of this change: ADR-005 defers it to a
  follow-up ADR; the audit matcher, git-env pin and config allowlists go with it.
- Compatibility matrix — no new git floor (the validator adds one `git ls-tree`);
  Node floor raised to ≥22.23.4 (r4: the 22.x line with the October 2026 security
  release, engines included); macOS `yes (unverified)` behind #12's contrast.

Editor notes: Deferred / out of scope — "Directory moves as single pairs — this
ADR-005 if a real change needs them" is ambiguous; author to clarify.

## As-built (2026-11-10)

Record of what the run actually built, against the design above. Nothing here
changes a design decision; each entry names the drift, why it happened, and the
section it touches. `03-build-notes.md` carries the same facts with line-level
detail and the gate history.

**Drift**

1. *Reply field — chain order.* `moves` is
   `z.array(MovePairSchema).max(64).default([])`, not the proposal's
   `.default([]).max(64)`: in zod v4 `.max` does not chain after `.default`.
   Same contract — max 64 pairs, absent means `[]`, bounds still enforced at
   `safeParse`. Affects: Chosen design → Reply field.
2. *Rule 2 constant relocated.* `LOCK_FILE` moved from `runner.ts` to
   `types.ts`, beside `STATE_FILE`, so `moves.ts` can name it without importing
   `runner.ts` (an import cycle). The rule and the value (`.pipeline.lock`) are
   unchanged. Affects: Chosen design → Validator, rule 2.
3. *Failure representations pinned.* `validateMoves` returns `string | null`
   rather than throwing, and `applyMoves` returns
   `MoveApplyFailure | null` (`{ error, rollbackFailed }`); the runner also
   wraps the validator call so a rule that does throw lands as the same
   `invalid-reply: moves — <issue>`. The proposal allowed "a returned issue or
   a throw from a rule"; the driver needs the structured form to separate the
   environmental apply failure from the failed rollback. Affects: Validator,
   Apply, Wiring into retry.
4. *`finalizeStage` returns the `StageOutcome`.* It previously returned
   nothing; a moves apply failure has to reach `executeStage`'s caller for the
   retry/escalate classification to see it. Affects: Apply.
5. *Ledger backfill on a failed apply (not in the proposal).* When the apply
   fails, `finalizeStage` walks `state.results` back to the accepted attempt's
   entry — written when that reply validated, before the apply ran — and
   rewrites its `failure` to the `moves-apply-failed:` / `moves-rollback-failed:`
   text. The retry decision, the gate log and the operator-facing results all
   key on that entry, and an `ok` ledger next to a known-damaged worktree is a
   lie. No checkpoint shape change: `failure` is an existing optional field
   (A13 holds). Affects: Apply.
6. *Rule 4/5 tree lookup is exact-path.* The lookup is
   `git ls-tree <headBefore> -- ':(literal)<path>'`. A bare pathspec is
   glob-matched by git even after `--`, so `a?.ts` would pass rule 4 against a
   tracked `aX.ts` (and let an untracked directory reach `renameSync` as a
   directory move) while a legitimately named `bindings[en].json` would fail
   it. Rule 1 has already rejected `:` in every endpoint, and magic signatures
   date to git 1.9, so the matrix's "git, any version" row still holds.
   Affects: Validator rules 4 and 5; Compatibility matrix (git row unchanged).
7. *The terminal class is a named predicate.* `escalatesWithoutRetry(failure)`
   is exported from `runner.ts` beside `retryInSameSession`, and the driver
   branches on it before attempt 2 is built (one `stageFailure` gate entry,
   `finalizeEscalate`, no new session). The proposal states that class in prose
   ("escalates to the operator instead of starting attempt 2"); the driver needs
   something to branch on, and a named export is what Test strategy #6 can pin.
   Affects: Apply, Risks row 3, Test strategy #6.
8. *A nudged-then-silent session keeps the moves label.* `executeInference`
   remembers the last issue `validateMoves` returned for the attempt, and at the
   exhausted-nudge branch of the empty-reply path reports
   `invalid-reply: moves — <issue>` where that branch used to say
   `invalid-reply: not JSON`. Without it the pair the attempt died on names
   itself in no gate entry and in no repair prompt. A schema mismatch or a
   narration-only stage with no moves in the reply still ends
   `invalid-reply: not JSON`. Affects: Wiring into retry; Test strategy #10.
9. *Fail-closed guard where the proposal is silent.* A moves-bearing reply whose
   window carries no base revision is an issue
   (`"<stage> declared moves without a base revision to validate them
   against"`), not a silent pass and not an unvalidated apply. The proposal says
   only that `headBefore` is "passed only by `executeStage`". Under today's
   schemas that branch is unreachable, so it has no test pin — recorded here as
   the intentional guard the code-review testing seat (r1c2, nit) asked to have
   on the record. Affects: Wiring into retry.

**Not drift (verified against HEAD)**

- `gates.ts`, `enforceOwnership`, `stagePathAllowed` and `state-store.ts` are
  untouched: there is no move-specific sweep branch, exactly as the Sweep
  interaction section says, and checkpoint `version` is still 1.
- The prompts changed are exactly the four named (one sentence each). The
  `design-review.md` and `test-authoring.md` wording that also appears in the
  range's diff belongs to design 007 (`29087ca`), which entered through the
  `8f98292` merge of main, not to this run.
- Out-of-scope edits attempted in the first implement commit — staging on the
  failed-apply path, skipping the min-nudge floor for moves rejections, the
  reply-poll baseline change, the extra review-prompt sentence, and leniency on
  a missing `headBefore` — were all reverted (`967dd3c`), so nudge pacing, poll
  discipline and the preserved-worktree path at HEAD are the behavior the
  proposal's "behave exactly as for any invalid reply" describes.
- Deferred items stayed deferred: no directory-pair handling, no case-only
  rename support, no git write access or read-only git in stage shells.
  `docs/adrs/005-runner-executed-file-moves.md` is still `status: proposed`;
  the flip belongs to this run's done-gate, which has not run.

**Test strategy, as landed**

Items 1-13 all landed, in `moves.test.ts` (new), `moves-stage.test.ts` (new),
`moves-rollback.test.ts` (new: the driver-level pin for #5/#6's rollback-fault
class), `reply-contract.test.ts`, `budget.test.ts`, `ownership.test.ts`,
`prompts.test.ts` and `fix-trigger.test.ts`. Two are pinned slightly differently
from the wording: #5's EXDEV pair is a capability-probed
`describe.runIf(canMountTmpfs())` block, so an unprivileged host reports it
skipped (2 skipped in the suite) rather than leaving it operator-run; and #12 is
two platform-gated blocks — the Linux case-variant contrast runs here, the
`process.platform === 'darwin'` one never does on this host, so A10 remains
unverified exactly as the register says. #13's `git worktree add` round trip
landed in `moves.test.ts` and asserts the `.git` pointer file is untouched.
