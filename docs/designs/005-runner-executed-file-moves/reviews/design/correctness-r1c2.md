# Correctness review r1 (dissent pass c2 — changed text since r1)

verdict: request-changes

Scope: `git diff` of `02-proposal.md` blobs 64ae876 → 89027e8 (the r2 revision),
then the full current document. Inputs: the proposal (incl. its Revision log),
`docs/adrs/005-runner-executed-file-moves.md`, my r1 findings. Re-verified
against repo source (`independence.ts`, `gates.ts`, `runner.ts`,
`.opencode/agents/code/coder.md`) and against the URLs the changed rows cite.

## r1 findings — closure check

- #2 (stale-`headBefore`/checkpointed-reply premise) — **closed**. The clause is
  gone; the guard is now "re-runs rules 4–7 as one guard across the
  inference→finalize window", which matches the Risks row and the code
  (`finalizeStage` has one call site after fresh inference).
- #3 (guard refusal outcome unclassified) — **closed**: "a guard refusal takes
  the same `moves-apply-failed` outcome and preserved worktree as a rename
  error", pinned by Test strategy #5.
- #4 (ship-gate fallback undecided) — **closed**: "There is no half-landing:
  item 5 ships with the controls below or the change does not merge", and the
  Goals bullet is now qualified ("item 5, conditional on the controls and ship
  gate named under Read-only git").
- #5 (case-folded distinctness unstated as conservatism) — **closed**: "the fold
  is deliberate conservatism, since a repo with case-variant filenames cannot
  move both in one reply".
- #6 (A6's `git show` sub-claim) — **closed**: `git show` dropped; the added
  multi-source config claim is supported by the cited git-config(1) FILES
  section.
- #1 (enforcement not deterministic) — **partly closed, new defect introduced**:
  enforcement did move to the runner-side audit the finding named, and A9's
  rewrite is accurate (opencode.ai/docs/permissions: whole-value wildcards,
  `*` spans `/`, "combines rules in order and uses the last match", policies
  hard-deny after rules and saved approvals — so the "appended after their
  existing rules" ordering is right, and marking A9 `verified` is fair). But the
  chosen mechanism misses two things the same paragraph asserts about it; see
  the two majors below.

## Findings

- [major][new] Chosen design — Read-only git (item 5) — "in the shape of its
  existing `transcripts` matcher, and a hit fails the attempt. It carries the
  plant the sweep cannot see or undo" — the control is detect-and-fail-attempt,
  not neutralize, and the retry it triggers re-exposes the runner to the payload.
  `auditSessionIndependence` judges only messages at or after this attempt's
  prompt (`since`, independence.ts:46-50: "A retried attempt must not be failed
  again for reads an earlier attempt already paid for"), and an
  `independence violation:` failure is not resumable, so attempt 2 runs a fresh
  session **in the same worktree** with the plant still in `.git`
  (runner.ts:836-837, 845-849, `retryInSameSession` at 1121-1126 resumes only
  `invalid-reply`). Nothing restores `.git` — that is the design's own premise
  ("`.git/**` is never dirty and has no blob at `headBefore`"). So attempt 2's
  finalize runs `git add -A` (runner.ts:298) and `git commit -m` (307, no
  `--no-verify`) under the planted `.git/config` (`core.fsmonitor`,
  `include.path`) or a planted `.git/hooks/pre-commit`, and no later attempt
  re-detects it. Test strategy #13 pins only "fails the attempt", i.e. exactly
  the half that does not protect the run. Fix: make a `.git`-segment audit hit
  terminal for the run — escalate to the operator as `moves-rollback-failed`
  does, no attempt 2 — or name the neutralization (delete/restore the detected
  `.git` path) that happens before any later attempt; and extend #13 to assert
  the *following* attempt's runner `git commit` does not execute the plant.

- [major][new] Chosen design — Read-only git (item 5) — "The enforcement is
  runner-side, because agent-config permissions are only advisory: a model can
  route around them (gates.ts:256-258, independence.ts)" — the route those lines
  name is subagents, and the new control is blind to it. gates.ts:257 reads
  "a model can route around them via subagents"; the audit scans `lastSeen`,
  this attempt's own session message list (runner.ts:759-769, 839;
  independence.ts:10-12 — "the completed tool calls and shell commands in its
  own message list"). A9's newly cited docs say "A custom subagent uses its own
  permissions, not a subset of its parent's permissions", with a base policy of
  `{action: "*", resource: "*", effect: "allow"}`, and nothing under
  `.opencode/agents` denies the subagent action for `coder.md` or
  `test-engineer.md`. So a `.git/config` write performed by a spawned subagent
  is invisible to the audit, invisible to the sweep, and ungoverned by the two
  configs' denies — the exact lever Options 1 refuses git access over. #13's
  three vectors (`edit .git/config`, `src/.git/config` plant,
  `git log --output=.git/config`) all run in the seat's own session, so the ship
  gate never exercises the route the justification names. Fix: name how child
  sessions of the attempt get audited (the mechanism that enumerates them and
  feeds their completed tool parts into the same matcher), or add a
  subagent-performed `.git` write to #13 and, if it cannot be covered, record
  the subagent route as an explicit uncovered vector rather than claiming
  runner-side enforcement over "the plant the sweep cannot see or undo".

- [minor][new] Chosen design — Validator rule 7 — "rule 2's `.git` test runs on
  those resolved paths too" — combined with the new predicate "any path with a
  case-folded `.git` segment or a segment ending in `.git`", running the test on
  *absolute resolved paths* rejects every endpoint when any ancestor of the
  checkout has such a segment. That is a real layout, not a curiosity: the
  bare-clone-with-worktrees convention (`~/src/project.git/wt-feature`) is
  precisely the linked-worktree case the Compatibility matrix marks `yes` and
  #13 exercises with `git worktree add`; there `moves` fails rule 7 for every
  pair. The symlink-via-`.git` case the sentence is there for needs only the
  repo-relative form. Fix: state that the segment test runs on the resolved path
  made relative to the repo's realpath (containment remains the absolute check);
  this loses nothing, since a symlink escaping into the real gitdir is already
  caught by the containment clause.

- [minor][new] Chosen design — Read-only git (item 5) — "`-c
  safe.bareRepository=explicit`, so the config sources outside the repo that A6
  names and a nested repo planted in the worktree never execute at runner
  privilege" — the nested-repo half is mis-attributed. `safe.bareRepository`
  (introduced in git 2.38, so the new floor row is right) governs whether git
  will operate on a bare repository it *discovers* at the invocation's cwd;
  `git()` already pins `cwd: repoDir` (runner.ts:106-112), so a nested repo in
  the worktree is never the discovered repository with or without the flag.
  Consequence for the ship gate: #13's "the runner's git calls ignore … a nested
  bare repo" fixture passes whether or not the flag ships, so a control the
  design and the Compatibility floor both rest on is pinned by a test that cannot
  fail. Fix: attribute the nested-repo half to the pinned cwd and keep the flag
  sentence to the ambient-config half, or make #13's case a runner git call whose
  cwd is inside the planted bare repo so the flag is actually exercised.

- [nit][new] Chosen design — Read-only git (item 5) — "The two agent configs
  mirror it as `edit` denies on `.git`, `.git/**`, `**/.git/**` and `**/*.git`" —
  under the whole-value wildcard matching A9 now records (`*` matches zero or
  more characters including `/`), `**/*.git` denies only paths that *end* in
  `.git`, so the mirror does not express the predicate's "a segment ending in
  `.git`" for its children (`src/mirror.git/config` is still permitted by the
  config), and the docs make path matching case-insensitive only on Windows, so
  the `.GIT/hooks/pre-commit` refusal is runner-side alone. Fix: one clause —
  the advisory mirror covers the `.git` dir/pointer and nested `.git/` dirs; the
  `*.git`-segment and case-folded halves exist only in `stagePathAllowed`.

## FYI (no severity)

- Non-goals bullet "Any agent git access, any scope (ADR-005 Alternatives; see
  Options 1)" sits four lines under a Goals bullet that now explicitly grants
  four read-only git commands per ADR-005 item 5; the pointer makes it readable,
  but the wording reads as a contradiction — clarity seat.
- "the audit's command-text matcher plus `git()`'s pinned env carry that class"
  covers only command text naming a `.git` segment; an allowed command writing
  outside the repo (`git log --output=$HOME/.bashrc`) is in that class, is
  invisible to the sweep, and the Deferred section gives it no owner — security
  seat.
