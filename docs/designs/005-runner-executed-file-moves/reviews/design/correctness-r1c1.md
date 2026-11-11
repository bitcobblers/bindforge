# correctness review r1 (dissent pass c1)

verdict: request-changes

Scope: `git diff 188d705..64ae876` of `02-proposal.md`, then the full current
document. Inputs: the proposal, `docs/adrs/005-runner-executed-file-moves.md`,
the Revision log, and my r1 findings.

## Round-1 closure check (all five closed for real, not reworded)

- **major (pinned status form) — closed.** The allow entry is now
  `git --no-optional-locks status*`, matched by the Goals bullet and the
  Revision log. Verified against git(1) SYNOPSIS (`[--no-optional-locks] …
  <command> [<args>]` — a global option, not a `git status` option) and
  git-status(1) BACKGROUND REFRESH, which itself spells "using
  `git --no-optional-locks status`" and documents the refresh write and the
  background lock conflict. The new Compatibility row "git ≥ 2.16 … floor set
  by the global `--no-optional-locks`" also checks out: the 2.16.0 release
  notes carry `(merge … jk/no-optional-locks …)`; `diff.autoRefreshIndex` and
  the 50% rename default (`-M`, "The default similarity index is 50%") predate
  it, as the row claims.
- **minor ("three of four") — closed.** Now "the same two of four, `git diff*`
  and `git log*`, code-correctness.md:27-31"; that file carries `git diff*`
  (27) and `git log*` (30) and nothing else; no agent config anywhere carries
  `git show*` or `git status*`.
- **minor (`to` parent existence) — closed.** Rule 5 now requires the parent to
  exist, rule 7 re-anchors on "`to`'s parent (rule 5)", and Test strategy #2
  lists "`to`'s parent missing".
- **nit (A13 citation) — closed.** `state-store.ts:178-184` is `saveState`'s
  `StateSchema.safeParse` re-validation gate; 145-155 remains the load side.
- **nit (A6 basis) — closed.** The row now cites git and git-status alongside
  git-log/git-diff/git-config. (One new over-reach inside that edited row is
  re-flagged below as a nit.)

## Findings

- [major][new] Chosen design — Read-only git (item 5) — "The enforcement is the
  harness-side write deny: `coder.md` and `test-engineer.md` gain `edit` denies
  on `.git` and `.git/**` … It is load-bearing, because the sweep cannot see or
  undo a config plant" — the r1 rewrite moved item 5's enforcement onto the
  permission layer and demoted the runner-side check to "Defense in depth", but
  this repo states in two places that the permission layer is not the control:
  gates.ts:256-258 ("Agent-config permissions are only advisory (a model can
  route around them via subagents), so the runner enforces this matrix
  deterministically") and independence.ts's header ("permissions alone did not
  hold, so the runner verifies after the fact"). The design's own sentence
  concedes the deterministic half is blind here (`.git/**` is never a dirty
  path), and the `stagePathAllowed` line it keeps only ever runs over move
  endpoints and over sweep output — never over an `edit` call. So as revised,
  the config plant that Options 1 cites as the reason to refuse *any* agent git
  access is left with no deterministic control, and the item-5 allowlist
  re-opens exactly that lever. Test #13's fixture ("an attempt whose agent
  edits `.git/config` is refused") only exercises the direct write the advisory
  fence already claims to stop. Fix: make the enforcement the mechanism the repo
  already uses for fences that permissions do not hold — extend the existing
  runner-side tool-call audit (`auditSessionIndependence`, independence.ts,
  called at runner.ts:839-849, "The permissions say so; this checks up on it")
  with a `.git`-segment matcher over completed write/edit tool calls that fails
  the attempt — and keep the agent-config deny as the advisory mirror, which is
  what r0 said; alternatively register "the harness `edit` deny cannot be routed
  around by the stage's own tools or subagents" as an unverified assumption and
  make #13's fixture use the route gates.ts:257 names.
- [minor][new] Chosen design — Apply — "so a resume re-entering finalize on a
  checkpointed reply never applies against a stale `headBefore` or stale disk" —
  no such path exists. `finalizeStage` is called only from `executeStage`,
  immediately after a fresh `executeInference` (runner.ts:1070-1081), and
  resume re-runs the stage from inference (`resumePipeline` → `drive` →
  `runWithRetry` → `executeStage`, runner.ts:1453-1484); the checkpoint stores
  no replayable reply, and `finalizeStage` has no other call site. The real
  window the guard closes is validate-at-inference → apply-at-finalize, which
  the Risks row already names correctly. Worse, the stated premise is
  self-defeating: were finalize ever re-entered on an applied list, rules 4–7
  would reject it forever (rule 4 wants `from` on disk, rule 5 wants `to`
  absent), contradicting "A crash between apply and commit converges on resume".
  Fix: restate the guard's rationale as the inference→finalize window and delete
  the checkpointed-reply clause.
- [minor][new] Chosen design — Apply — "Immediately before the first rename it
  re-runs validator rules 4–7 as one guard" — the guard's failure class is the
  only exit in that paragraph left unclassified: rename errors get
  `moves-apply-failed` (attempt 2, fresh session, worktree preserved) and
  rollback errors get `moves-rollback-failed` (stops at the operator), while a
  guard refusal names no outcome, no preserved-worktree state, and no test
  class (#5 pins that the guard refuses; #6 enumerates only the other three).
  The refusal also happens after the reply was accepted, where the
  `invalid-reply:` nudge path is no longer available. Fix: one clause — a guard
  refusal takes the same `moves-apply-failed` outcome and preserved-worktree
  state as a rename error — and pin it in #5/#6.
- [minor][new] Chosen design — Read-only git, vs Options considered — "If Test
  strategy #13 cannot show the harness refusing a `.git/config` write, item 5
  does not ship in this change." vs "**Land `moves` without item 5.** Rejected:
  the ADR decided item 5" — the gate's fallback is precisely the option the
  Options section rejects, and ADR-005 item 5 is decided, so the design leaves
  the ship-failure outcome undecided (does the whole change hold, or do `moves`
  land with ADR-005 item 5 unimplemented?). The Goals bullet "The coder and
  test-engineer seats can run `git --no-optional-locks status` …" states the
  conditional outcome as unconditional. Fix: name the consequence — the change
  does not merge with item 5 unproven, or item 5 moves to a named follow-up
  with the ADR's item left open — and qualify the Goals bullet to match.
- [nit][new] Chosen design — Validator rule 6 — "all `from`s distinct and all
  `to`s distinct compared case-folded" — on the case-sensitive reference
  platform two tracked files differing only by case (`src/a.ts`, `src/A.ts`)
  are two distinct blobs at `headBefore` (rule 4 proves it) and a two-pair list
  over them is legal input that the validator now rejects as a reply error,
  burning a code-budget unit (A3). That is safe, so it is not unsound — but as
  written an implementer cannot tell whether rejecting that list on Linux is
  intended. Fix: one clause stating the conservatism ("case-folded distinctness
  is enforced on every platform; a repo with case-variant filenames cannot move
  both in one reply"), or gate the fold on the checkout's `core.ignoreCase`.
- [nit][new] Assumption register A6 — "`core.fsmonitor` fires on an index
  refresh — which `git status`, `git diff`, and `git show` all perform by
  default" — the row's own sources support two of the three: git-status(1)
  BACKGROUND REFRESH for `git status`, and `diff.autoRefreshIndex` ("defaults to
  true … affects only git diff Porcelain") for `git diff`; nothing in the cited
  pages documents an index refresh for `git show`, which by default compares
  commits/trees rather than the worktree. The row's conclusion is unaffected
  (the `.git/config` deny covers `core.fsmonitor` either way). Fix: drop
  "`git show`" from that clause, or add the basis that shows it.

## FYI (no severity)

- Rule 7 checks that `to`'s parent's realpath stays inside the repo's realpath,
  and the apply guard re-runs rules 4–7 — but the `.git` endpoint denial lives
  in rule 2, matched on the literal path. A worktree symlink planted between
  validation and apply (an agent can reach `npm run check`, and package.json is
  in its edit scope) resolves *inside* the repo, so `to` = `<link>/config`
  passes rule 7 and the guard. Security seat's call.
- The Reply-field bounds sentence attributes the capped subprocess work to
  finalize; the per-pair `git cat-file` calls are spent at validation
  (`executeInference`) as well as by the apply guard.
