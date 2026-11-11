# security review r1 (panel pass c0, scope: whole document)

verdict: request-changes

Checklist coverage: 1 trust boundaries — evaluated (reply `moves` is treated as
untrusted at the single `safeParse` choke point, A2; consistent with ADR-005
item 1). 2 parsing — gap (F3). 3 path handling — mostly good (rules 1, 6, 7 plus
the symlink-escape test), gap on control characters and separator syntax (F4).
4 secrets/PII — nothing new leaves the machine; validator messages carry repo
paths only, acceptable. 5 dependencies — evaluated by web lookup (F6): Node
22.x advisories are fixed upstream, git advisories in the current class
(`core.fsmonitor` config sinks, e.g. the 2026 GitSpawn cluster and
GHSA-grg2-7gc6-36m6) are in *consumers of git*, not in git itself, so nothing
here is an unfixed advisory in a named dependency — but the design's lever list
misses that sink (F2). 6 least privilege — gap (F1). 7 fail closed — gaps
(F4, F7).

## Findings

- [major][new] Chosen design — "Read-only git for implement/fix/test-update
  seats (item 5)" — "Precondition, enforced: `stagePathAllowed` gains, as its
  first line, `path === '.git' || path.startsWith('.git/') → false`" — this is
  presented as the enforcement that makes item 5 safe, but the mechanism named
  cannot observe or undo the write it is meant to stop. Per A4 the sweep's whole
  input is git's dirty-path list, and `.git/**` is by construction never a
  dirty path; and "restores violations from `headBefore`" cannot restore
  `.git/config`, which has no blob at `headBefore`. So a stage that writes
  `.git/config` (A7 says it can today) is neither blocked nor reverted by
  `stagePathAllowed`: the plant survives the attempt boundary, is re-read by
  every later git call — including the runner's own (`git add -A` in
  `commitStage`, the sweep's status/diff, the new `git cat-file` in rule 4) —
  and turns config keys into execution at *runner* privilege, outside the agent
  permission model entirely. The one mechanism that can actually prevent the
  write is the agent-config `edit` deny, which the text demotes to "The
  advisory mirror goes into agent configs". Test strategy #4 only asserts the
  predicate returns false, which is true whether or not the write is
  preventable. Fix: make the harness-side control load-bearing — name the
  OpenCode `edit`/`write` deny on `.git` and `.git/**` in `coder.md` and
  `test-engineer.md` as the enforcement (delete the word "advisory"), keep the
  `stagePathAllowed` first line only as the validator's endpoint rule and
  defense-in-depth, and replace test #4's predicate assertion with an
  attempt-and-observe test that a stage write to `.git/config` is refused. If
  the harness cannot be shown to deny that path, item 5 must not ship in this
  change.

- [major][new] Chosen design — same section — "with `.git/config` unwritable,
  `diff.external`, `core.pager`, `core.hooksPath`, and `diff.*.textconv`
  drivers cannot be planted" — the closed-lever list (and A6, which grounds it)
  omits `core.fsmonitor`, the sink behind the current class of git-config
  execution advisories, which fires on an index refresh rather than on a flag.
  That matters because the design's only stated defence for `git status` is
  `git status --no-optional-locks*` "avoids the index-refresh side effect", and
  the newly allowed `git diff*`/`git show*` are exactly the calls that refresh
  the index. With argv vetting Deferred, every config-triggered sink is
  protected solely by the config write-deny of the finding above; an
  incomplete list there means an unmitigated RCE path, and the argv deny list
  (`--output*`, `--ext-diff*`, `--textconv*`) covers none of it. The Risks
  row-1 mitigation is also mismatched: "worktree landings are swept and land in
  visible commits" mitigates a rogue *write*, not a rogue *execution*, which
  leaves no trace in the worktree. Fix: add `core.fsmonitor` (and
  `filter.<driver>.clean`/`smudge`, `core.sshCommand`) to A6 and to the closed
  list, restate row 1's mitigation as the config-write deny plus a hostile
  `.git/config` fixture asserting the allowed commands do not execute it, and
  say plainly that the argv deny list is best-effort, not the control.

- [minor][new] Chosen design — "Reply field" — "`moves: z.array(MovePairSchema)
  .default([])`" — the untrusted list has no bound: no maximum pair count and no
  maximum path length, while rule 4 spends a `git cat-file` subprocess plus an
  `existsSync`/`realpath` per pair and rule 6's "no listed path is a path-prefix
  of another" is quadratic in the list size. A model reply steered by injected
  content can make finalize spend unbounded subprocess and CPU work before any
  rejection. Fix: `z.array(MovePairSchema).default([]).max(N)` plus a per-string
  `.max(len)` on `from`/`to`, so the oversize list fails at `safeParse` on the
  same invalid-reply path, and extend test #1 with the over-limit case.

- [minor][new] Chosen design — "Validator" rule 1 — "Shape: repo-relative POSIX
  paths, normalized, no absolute path, no `.` or `..` segments, `from !== to`" —
  shape is checked but the character class is not: a NUL byte, a control
  character, or a backslash survives rule 1 and reaches `existsSync`,
  `git cat-file`, and `realpath`, where `spawn`/`fs` throw
  `ERR_INVALID_ARG_VALUE`. The design routes only a validator-returned issue to
  "`invalid-reply: moves — <issue>`", so a throw escapes the nudge/retry path
  and crashes finalize instead of failing the reply — precisely the fail-open
  case the goal forbids. The backslash case is also platform-dependent:
  splitting on `/` only means `..\..\x` is a single legal segment on Windows,
  with rule 7's realpath as the sole backstop. Fix: tighten rule 1 to one
  allow-pattern (reject NUL, control characters, and `\`) so no exotic string
  reaches a syscall, and wrap the `validateMoves` call in a catch that converts
  any throw into the same `invalid-reply: moves` branch; add both cases to test
  #2.

- [minor][new] Chosen design — "Apply" — "`moves.applyMoves(repoDir, moves)`,
  called from `finalizeStage`" — the only apply-time check is "re-check `to`
  absent", and rules 4–7 (tracked-at-`headBefore`, on-disk existence,
  containment) are not re-checked there because `applyMoves` is not even given
  `headBefore`. Since `moves` is persisted in the checkpoint (A13; Rollout 2's
  "in-flight and old checkpoints re-parse unchanged"), a resume that re-enters
  finalize with a previously accepted reply applies a list whose state-dependent
  rules were evaluated against a stale `headBefore` and a stale disk — the
  design reasons about a crash between apply and commit but not between reply
  acceptance and apply. Fix: pass `headBefore` into `applyMoves` and re-run the
  state-dependent rules (4–7) as a single guard immediately before the first
  rename, so there is one authoritative validation call site rather than a
  parse-time-only one.

- [minor][new] Chosen design — "Apply" — "completed renames are rolled back in
  reverse — exact because pairs are disjoint" — the rollback itself can fail
  (`rename(2)` fails `EISDIR`/`EXDEV`/`EINVAL`, A8, or `EACCES`/`ENOSPC`), and
  the design says nothing about that case. The result is exactly the
  half-applied list the Goals promise to prevent, and the documented recovery
  launders it: "the next attempt re-captures `headBefore`" makes the damaged
  state the baseline, so a partially applied move set can be committed by a
  later stage. Fix: on a rollback failure emit a distinct
  `moves-rollback-failed` outcome, escalate to the operator instead of starting
  attempt 2 on a known-damaged worktree, and add a test #5 case where a
  rollback rename fails.

- [minor][new] Compatibility matrix — "Node ≥ 22.12
  (tools/orchestration/package.json:30-32) | yes" — the runtime floor predates
  published Node 22.x security releases, including the permission-model
  path-traversal and `fs.realpathSync` checks fixed in the 2026 releases
  (22.22.0, 22.23.4). Nothing here is an unfixed advisory, so this is not a
  blocker, but rule 7's containment leans on `realpath` semantics and the
  Deferred hand-off to "OS/sandbox-level protection" should not assume Node's
  `--permission` model is sound. Fix: state a patched minimum in that row
  (e.g. Node 22.23.4) and add a line to the sandbox Deferred noting that Node's
  permission model is not an acceptable substitute for harness or OS
  enforcement; also state the minimum `git` version the design relies on, which
  is currently unpinned.

## FYI (no severity)

- Moves add no leverage over what `implement`/`fix` may already write (A7), but
  note for another seat that `.opencode/agents/**` and `tools/orchestration/
  prompts/**` sit inside those seats' write ownership, so an agent can rewrite
  the very config this design leans on for denies; pre-existing, worth its own
  ADR rather than a change here.
- A validated move relocates a file a review verdict cited by path; the
  stamp/verdict consequence is the review-protocol question the design hands to
  ADR-006 consistently with ADR-005:84-85, so it is not a finding here.
