# security review r1 (dissent pass c1)

verdict: request-changes

Scope: `git diff 188d705… 64ae876…` over `02-proposal.md`, then the full current
document. Inputs: the proposal, its Revision log, ADR-005, my r1 findings.

## r1 findings — closure check

1. (major) `.git` deny must be load-bearing — **closed**. "The enforcement is the
   harness-side write deny: `coder.md` and `test-engineer.md` gain `edit` denies
   on `.git` and `.git/**`"; the word "advisory" is gone; `stagePathAllowed`'s
   first line is now labeled "Defense in depth"; "If Test strategy #13 cannot show
   the harness refusing a `.git/config` write, item 5 does not ship in this
   change" is the ship gate I asked for. What the closure newly depends on is the
   deny's *completeness* — findings 1 and 2 below attack that, not the demotion.
2. (major) closed-lever list omitted `core.fsmonitor`/`core.sshCommand`/
   `filter.<driver>.clean`+`smudge` — **closed** in A6 and in the item-5
   paragraph; Risks row 1 now names the config deny and #13's hostile fixture,
   and the argv list is explicitly "best-effort hygiene, not the control". The
   demotion is what makes finding 3 below load-bearing.
3. (minor) unbounded `moves` list — **closed**: `.max(200)` per endpoint,
   `.max(64)` per list, failing at `safeParse`, pinned in Test #1.
4. (minor) rule 1 had no character class, and a throwing rule escaped the nudge
   path — **closed**: rule 1's allow-pattern (`\`, NUL, control characters, `:`,
   reserved names, trailing `.`/space) plus "a failure — a returned issue or a
   throw from a rule — takes the existing schema-failure branch", pinned in #2.
5. (minor) apply did not re-check state-dependent rules — **closed**:
   `applyMoves(repoDir, headBefore, stageName, moves)` "re-runs validator rules
   4–7 as one guard" before the first rename; pinned in #5.
6. (minor) rollback failure unspecified — **closed**: `moves-rollback-failed:
   <error>` escalates instead of starting attempt 2; pinned in #5 and #6.
7. (minor) runtime floors and Node's permission model — **closed**: the Node row
   names 22.23.4 (a real security release, 2026-10-21) and the Deferred adds
   "Node's own `--permission` model is not an acceptable substitute". The new
   `git ≥ 2.16` row is right about `--no-optional-locks` (it merged into 2.16)
   but introduces the floor problem in finding 2.

## Findings

- [major][new] Chosen design, "Read-only git for implement/fix/test-update seats
  (item 5)" — "An unwritable `.git/config` closes every config sink A6 names,
  `core.fsmonitor` — which fires on the index refresh these commands perform —
  included." — git's exec-capable config does not live in one file. The sources
  are repo-local `.git/config`, `$XDG_CONFIG_HOME/git/config` and `~/.gitconfig`,
  `/etc/gitconfig`, and `include.path` reachable from any of them; the ADR's own
  reasoning (Options 1 quotes it: "a stage that can write the worktree can plant
  config or hooks that any permitted git command then executes") is about the
  *plant*, not about one filename. The named control cannot reach them: OpenCode
  matches `edit` resources as whole-value wildcards against the normalized,
  Location-relative path, so `.git`/`.git/**` denies nothing outside the
  worktree, and the harness default has no `external_directory` ask/deny ("reads
  and edits outside the Location and worktree proceed without a prompt"), while
  the design's own Options-2 words say the runner's git helper "inherits ambient
  config". A stage that writes `core.fsmonitor` (or `include.path` pointing at an
  in-ownership `.gitattributes`-reachable file it can also write) into
  `~/.gitconfig` is stopped by nothing in the changed text, is invisible to the
  sweep (no worktree path, never dirty, no blob at `headBefore` — the very
  argument the paragraph uses to make the deny load-bearing), and executes at
  runner privilege on the runner's next `git status`/`git add -A`, surviving the
  attempt boundary. The Deferred section states the gap without closing it:
  "Hardening the runner's own git subprocess env (`GIT_CONFIG_NOSYSTEM`, pinned
  `GIT_CONFIG_GLOBAL`) — no follow-up yet; the `.git` write-deny closes the
  repo-local half." Fix: move the runner-side half onto the load-bearing side —
  have `git()` set `GIT_CONFIG_NOSYSTEM=1` and `GIT_CONFIG_GLOBAL` to an empty
  file, or pass `-c core.fsmonitor=false -c safe.bareRepository=explicit` on the
  runner's calls (the shape every 2026 fix for this class took) — or deny the
  global/system config paths in the two agent configs and add them to #13's
  hostile fixture. In either case shrink the sentence to what the mechanism does:
  "closes the repo-local config source", matching the Deferred clause it leans
  on.

- [major][new] Chosen design, item 5 paragraph — "`stagePathAllowed` gains, as
  its first line, `path === '.git' || path.startsWith('.git/') → false`" — both
  named mechanisms are anchored at the worktree root, but the plant vector the
  design and the ADR name is "the worktree" at large. `implement`/`fix` own
  everything except `tests/**` (A7), so a stage can `write`
  `<owned-dir>/.git/config` (or a bare `<owned-dir>.git/`). That path matches
  neither the whole-value patterns `.git` / `.git/**` nor the `startsWith('.git/')`
  test, so neither the enforcement nor the defense-in-depth rule sees it; it is
  in-ownership, so the sweep neither reverts nor deletes it (enforceOwnership
  only deletes *violations*); and `commitStage`'s `git add -A` records it as a
  gitlink with a warning ("adding embedded git repository"), giving the plant a
  committed marker that persists into every later attempt. From then on, the
  superproject's own `git status`/`git diff` — the runner's `dirtyFiles` call,
  every finalize — consults that nested repository's config, and `core.fsmonitor`
  fires on that refresh (the buried-bare/embedded-repo class; git's only
  protection, `safe.bareRepository=explicit`, is opt-in and its default does not
  flip until Git 3.0). The Compatibility row makes the escape unavailable by
  fiat: "git ≥ 2.16 … floor set by the global `--no-optional-locks`" — below
  2.38 `safe.bareRepository` does not exist. Fix: make the rule a segment rule
  rather than a prefix rule — `p.split('/').includes('.git')` plus rejecting any
  `*.git` name in `stagePathAllowed`, and `**/.git/**` / `**/*.git` in the two
  agent configs — raise the floor to `git ≥ 2.38` so the runner can set
  `safe.bareRepository=explicit`, and add a planted `<owned-dir>/.git/config`
  carrying `core.fsmonitor` to Test strategy #13's fixture as part of item 5's
  ship gate.

- [major][new] Chosen design, item 5 allowlist paragraph, and Risks row 1 — "the
  config-write deny above carries item 5 and argv vetting stays sandbox
  territory" / "worktree landings land in visible commits" — the deny the
  paragraph makes load-bearing is scoped to the `edit` action's target path; it
  cannot observe a write performed *by* an allowed shell command. The design's
  own A6 names `--output=<file>` on `git log`/`git diff` — a flag-only file
  write that needs no config at all — and the shipped patterns are `git log*`
  and `git diff*`, which prefix-match any suffix, including
  `--output=.git/config` or an out-of-repo target. With those refinements now
  demoted to "best-effort hygiene, not the control", no mechanism in the changed
  text carries that class, and the harness explicitly warns that directory
  inference from shell command text is best-effort ("prefer a narrow shell
  allowlist instead of patterns intended to recognize every dangerous command").
  Risks row 1's mitigation is false exactly where the risk lands: a `.git/**`
  landing is, by the same paragraph's argument, never a dirty path and has no
  blob at `headBefore`, so it is neither swept nor visible in a commit —
  "worktree landings land in visible commits" mitigates nothing for it. Fix:
  split the control by class in the text — config sinks → the write deny;
  flag-only write/exec levers → the `--output`/`--ext-diff`/`--textconv`
  denials, stated as load-bearing and expressed as OpenCode `policies` hard-denies
  (which apply after allow rules and saved approvals, unlike agent-config
  refinements whose ordering A9 mis-states) — and pin them in #13 by running
  `git log --output=.git/config` and `git diff --output=<owned path>` in an
  attempt and asserting refusal, on the same ship-gate footing as the
  `.git/config` write test. Either delete "worktree landings land in visible
  commits" or scope it to paths that can actually be dirty.

- [nit][new] Chosen design, item 5 allowlist paragraph + A9 — "follow the
  existing prefix-pattern style" / "OpenCode permission matching is
  prefix-pattern with most-specific-wins" — the harness's documented model is
  whole-value wildcards with **last-match-wins** rule combination, not
  most-specific-wins. The `.git` deny and the argv refinements work only because
  they are written after the broad allow, which is what coder.md happens to do
  (`edit "*": allow` first). Fix: restate A9 as whole-value wildcards with
  last-match-wins, and state the ordering requirement ("deny lines appended after
  the allows in both configs") instead of relying on specificity.

- [nit][new] Test strategy #4 — the r1 rewrite dropped "`stagePathAllowed` is
  false for `.git` and `.git/**` for every stage" while moving that rule to
  defense-in-depth, so the first line of the predicate now has no direct
  assertion; #2 pins only the validator's use of it. Fix: keep one direct
  predicate assertion for that line in #4.

## FYI (no severity)

- "Immediately before the first rename it re-runs validator rules 4–7 as one
  guard": a guard failure at apply time is a *reply*-class problem but is
  classified as environmental (`moves-apply-failed`, fresh session, budget cost);
  also `stageName` is now passed to `applyMoves` although the re-run set excludes
  rules 2–3, which are the ones that use it. Ownership/correctness seat.
- "review seats already run the same two of four, `git diff*` and `git log*`"
  checks out against `.opencode/agents/codereview/*.md:27,30`; r0's "three of
  four" was the error and r1 fixed it.
- The Node row's "22.23.4 at time of writing" is a real release (2026-10-21), but
  the Node project has another 22.x security release dated 2026-11-11 — one day
  after this proposal's date — so the named floor is already stale by
  construction; wording like "the current patched 22.x" would age better.
