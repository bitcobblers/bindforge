# correctness review r1 (dissent pass c4)

verdict: approve

Scope: `git diff 6b21e2f 44c52fa` on 02-proposal.md, then the full current document,
against docs/adrs/005-runner-executed-file-moves.md and the Revision log.

## Closure check of my r1 findings

- **major — escalation is not git-free over the plant.** CLOSED as prescribed. The
  paragraph now names the neutralization before any git on that path (`git()`'s env
  becomes the shared env for every runner git call, setting `-c core.fsmonitor=false
  -c core.hooksPath=<empty dir>`), acknowledges that `finalizeEscalate` → `commitStage`
  (runner.ts:2008-2013) still commits `ESCALATION.md`, and states the operator
  requirement (`nothing clears it in-process, so a pipeline resume … runs over it until
  the operator removes it`). #13's marker fixture now pins the assertion against
  "the escalation's own `ESCALATION.md` commit" instead of a bare "next git call".
  Residual over-claim reported below as [minor] — the theme is closed.
- **major — `subagent` deny vs. what the audit can assert.** CLOSED as prescribed
  (my option b): the paragraph concedes "Its input is one attempt's own session, so a
  child agent's `.git` write is audit-blind and the `subagent` deny below is the only
  cover for that route"; #13's escalate list drops the subagent-launched write and
  keeps it in the refused-by-config list; the Risks row says "child-agent writes are
  audit-blind, covered by the advisory `subagent` deny only". Internally consistent now.
- **minor — env scope wider than `git()`.** PARTIALLY closed: the text now says
  "every runner git call" and names `gitOk` (143-145) and the `merge-base` spawn
  (1417-1420), and #13 and the Compatibility/Risks rows follow. One runner git spawn
  is still unnamed — see finding 1.
- **minor — orphaned `safe.bareRepository` flag.** CLOSED (my option b): the flag is
  gone from the paragraph and the Compatibility row, and the floor is re-derived from
  `GIT_CONFIG_GLOBAL`. I verified the floor claim against git's own release notes:
  the 2.32 relnotes introduce `GIT_CONFIG_GLOBAL` ("introduce GIT_CONFIG_GLOBAL to
  override the per-user configuration in $HOME/.gitconfig"), and git(1)'s ENVIRONMENT
  section confirms "if `GIT_CONFIG_GLOBAL` is set, neither `$HOME/.gitconfig` nor
  `$XDG_CONFIG_HOME/git/config` will be read" — so it covers both A6 global sources,
  and `GIT_CONFIG_NOSYSTEM` remains honored. The retained sentence "a planted bare repo
  is inert anyway under the pinned `cwd: repoDir`" is also sound: `.git` present at
  `repoDir` ends discovery there, so no bare candidate below the root is ever
  discovered.
- **nit — rule 7's stale `project.git` reason.** CLOSED: the sentence ends at "the
  resolved path relative to that realpath" and no test now claims the `*.git` ancestor.
- **nit — Options 1 reads as rejecting the shipped read-only allowlist.** CLOSED:
  heading and closing clause now say "Agent git *write* access" / "no write-capable
  command allowlist fixes that half", matching the Goals line and ADR-005's Alternatives.

## Findings

- [minor][re-raised] Chosen design — Read-only git (item 5) — "`git()`'s env (106-112)
  becomes the shared env for every runner git call — `gitOk` (143-145) and the raw
  `merge-base` spawn (1417-1420) included" — the enumeration is still incomplete, so the
  absolute form of the claim and of #13's gate ("every runner git call, `gitOk` and
  `merge-base` included") is false against the code as it stands. There is a fourth
  `spawnSync('git', …)` with `cwd: repoDir` outside `git()`/`gitOk`: `writeEditorialDiff`
  (called from runner.ts:1507 in the review/editorial loop) spawns
  `git diff --no-index -- <snapFile> 02-proposal.md` at runner.ts:1933-1937, inheriting
  the operator's environment and reading the worktree's config like the others. Fix: make
  the sentence name all non-`git()` sites — "`gitOk` (143-145), the raw `merge-base`
  spawn (1417-1420) and `writeEditorialDiff`'s `git diff --no-index` (1933-1937) included"
  — and add the editorial-diff call to #13's "every runner git call" list; or state the
  rule as "every `spawnSync('git')` site goes through the one env helper", which makes the
  enumeration self-maintaining.
- [minor][new] Chosen design — Read-only git (item 5), same paragraph as its closing
  sentence, plus the Risks row — "So neither the config sources outside the repo (A6) nor
  a planted `.git/config` or `.git/hooks` runs at runner privilege" / "nothing outside the
  repo executes at runner privilege" — the conclusion is broader than the mechanism named
  two sentences earlier. `GIT_CONFIG_NOSYSTEM`/`GIT_CONFIG_GLOBAL` suppress only the
  system and per-user levels, and `-c core.fsmonitor=false -c core.hooksPath=<empty dir>`
  overrides exactly those two keys; the repo-local `.git/config` is still read, and A6
  (verified, cited in this same paragraph's source list) names "`include.path` from any of
  them" — a planted `.git/config` carrying `[include] path = /home/<operator>/.gitconfig`
  re-opens the outside-repo sources at runner privilege, and other exec-capable repo-local
  keys that fire in the escalation's own calls (notably `filter.<driver>.clean` under
  `git add -A`, which `commitStage` runs at runner.ts:298, driven by a planted
  `.gitattributes`, which is not a `.git` segment and so is neither config-denied nor
  audit-matched) are untouched by the two pins. So the ship gate's fsmonitor-marker
  fixture passes while the sentence's blanket guarantee is false. Fix: scope the sentence
  to what ships — "neither the system nor per-user config sources A6 names are read, and
  the two A6 mechanisms that fire in these calls — `core.fsmonitor` on an index refresh and
  hooks — are overridden; other exec-capable repo-local keys (filters, external diff) and
  an `include.path` inside a planted `.git/config` are the Deferred sandbox's" — or add the
  pins that would make the blanket form true.
- [nit][new] Chosen design — Read-only git (item 5) — closing sentences of the paragraph —
  this revision dropped the merge-blocking clause the paragraph used to carry ("If Test
  strategy #13 lets a vector land a plant, or a runner git call run over one, the change
  does not merge"). Goals still promises item 5 "conditional on the controls and ship gate
  named under Read-only git", and #13 still calls itself "Item 5's ship gate", but the
  named section no longer says what the gate does on failure, so the condition has no
  stated consequence. Fix: restore that one sentence at the end of the paragraph (it also
  carries the finding above into the gate).

## FYI (no severity)

- The `.gitattributes` + `filter.<driver>` plant named in my second finding is not a `.git`
  segment, so it is invisible to `stagePathAllowed`'s narrowed predicate, the agent-config
  denies and the audit matcher alike — containment question for the security seat.
- Options 1's heading now reads "Agent git write access, any scope" while ADR-005's
  Alternatives heading is "Agent git access (any scope)"; the proposal says the rejection is
  "inherited from ADR-005's Alternatives", so the name drift is worth one look by the
  wording/compatibility seat.
