# Compatibility review r1 (dissent pass c2)

verdict: request-changes

Scope: `git diff 64ae876f 89027e8a` over `02-proposal.md`, then the full current
document, against `docs/adrs/005-runner-executed-file-moves.md` and the
Revision log.

## r1 findings — status

- [closed] Rule 2 / `stagePathAllowed` `.git` case-sensitivity. The new
  first line is a case-folded `.git` segment rule declared as "one predicate,
  so validator rule 2, the apply guard and the sweep all refuse
  `.GIT/hooks/pre-commit`", rule 7 re-runs it on resolved paths, and #2/#4 pin
  `.GIT/hooks/pre-commit` and `src/.git/config` on Linux. Closures introduce a
  defect on the same predicate — see finding 3.
- [closed] Rule 6's case-fold scope. "every comparison here is case-folded on
  every platform" now covers `to`-vs-`from` and path-prefix, the conservatism
  is stated, and #2 pins a lone `README.md` → `readme.md`. The fold makes the
  macOS evidence plan inert — see finding 2.
- [closed] Mount-point row. "#5 names the case, operator-run since mounting a
  tmpfs needs capability the vitest suite lacks" plus #5's "operator-run and
  skipped where the mount capability is absent". Row keeps `partial`, not
  `unknown`; no finding.

## Findings

- [major][new] Chosen design / Read-only git — "`git()` (runner.ts:106-112)
  stops reading ambient config: `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL` at
  an empty file, `-c safe.bareRepository=explicit`". This deletes the only
  git-identity source most operator environments have, and nothing replaces it.
  `commitStage` (runner.ts:288-309) authors every stage commit with a bare
  `git(repoDir, ['commit', '-m', message])` — no `-c user.*`, no author/committer
  env — and today resolves identity from the ambient global/system config the new
  env suppresses; repo-local `.git/config` is still read (the paragraph itself
  scopes the agent denies as "advisory, scoped to the repo-local config source"),
  and neither the repo nor this design pins a repo-local identity. On the
  reference platform with the usual `~/.gitconfig` identity, every stage commit
  fails `unable to auto-detect email address`, i.e. item 5's control breaks the
  runner's committer path. The same holds for the suite: every tmpRepo fixture
  pins identity only on its own setup calls (`execFileSync('git', ['-c',
  'user.name=f', '-c', 'user.email=f@f', ...])`, stage-lifecycle.test.ts:94 and
  the same helper in reply-contract/funnel/parallel-*), so #8-#11's runner-side
  commits lose their identity too. #13's new fixture ("the runner's git calls
  ignore a hostile `~/.gitconfig`") pins exactly the behavior that removes it.
  The concern is not handed off either: this diff deletes the former Deferred
  bullet "Hardening the runner's own git subprocess env (`GIT_CONFIG_NOSYSTEM`,
  pinned `GIT_CONFIG_GLOBAL`) — no follow-up yet", and no matrix row or risk
  names the operator git-config dependency.
  Fix: in the same sentence, pin identity on the authoring calls — `git()`'s env
  gains `GIT_AUTHOR_NAME`/`GIT_AUTHOR_EMAIL`/`GIT_COMMITTER_NAME`/
  `GIT_COMMITTER_EMAIL` (or the commit calls pass `-c user.name=… -c
  user.email=…`), replacing the identity the suppressed global config used to
  supply — and extend #13 with the positive half: a stage commit succeeds in a
  tmpRepo with no global/system config at all.

- [minor][new] Compatibility matrix (macOS row) + Test strategy #12 —
  "Test strategy #12 runs one move round trip and one case-only move there and
  records the observed rejection as the A10 evidence" / "one move round trip plus
  one case-only move, recorded as the A10 evidence behind that matrix row".
  A10 is the claim that *the on-disk existence check* rejects a case-only
  endpoint on a case-insensitive checkout (rule 5's `existsSync`). Rule 6's r2
  fold now rejects a case-only pair on every platform — "rejecting … case-only
  moves (`README.md` → `readme.md`)" — before rule 5's disk check is reached, so
  the macOS observation records the same platform-independent rule-6 rejection
  that #2 already pins on Linux and can never discriminate `core.ignoreCase`.
  The row's `yes (unverified)` and A10's `unverified` therefore stay permanently
  unverifiable by the test named to verify them, and the Deferred entry
  ("Case-only renames … support, not just rejection, deferred") presumes the
  rejection is the interesting platform behavior when the design now rejects the
  pair by fold everywhere.
  Fix: change #12's second case from a case-only pair to a pair rule 6 cannot
  see: `README.md` tracked, reply moves an unrelated `notes.md` → `readme.md`.
  On macOS rule 5's `existsSync` rejects it; on Linux the same pair is accepted
  and lands as a case-variant sibling — that difference is the A10 evidence, and
  restate the macOS row's note as that contrast.

- [minor][new] Chosen design / Read-only git — "`stagePathAllowed` (gates.ts:260)
  returns `false` as its first line for any path with a case-folded `.git`
  segment or a segment ending in `.git`". The second half is a suffix rule on a
  predicate the sweep applies to *every* dirty path for *every* stage (A4), so it
  is an ownership change beyond moves: any path with a segment ending in `.git`
  (`fixtures/upstream.git/**`, a file `notes.git`) becomes unowned forever, an
  agent's edit there is silently reverted as if it were an ownership violation,
  and a validator endpoint there fails as a reply error on every platform.
  Nothing in the paragraph needs the suffix: both pinned cases
  (`.GIT/hooks/pre-commit`, `src/.git/config`) and the planted
  `<owned-dir>/.git/` gitlink have a segment *equal* to `.git` case-folded, and
  the bare-repo-named case the suffix would catch is the one the same paragraph
  assigns to `safe.bareRepository=explicit`. The agent-config mirror `**/*.git`
  carries the same over-reach.
  Fix: state the first line as case-folded segment equality — `false` for any
  path with a segment equal to `.git` ignoring case — and drop "or a segment
  ending in `.git`" (and `**/*.git`); if the suffix is wanted anyway, say which
  plant needs it and add a #4 case pinning that an edit to `<owned>/upstream.git`
  is a deliberate sweep revert.

## Checklist notes (no finding)

- Matrix: every row carries a status, no `unknown`, no empty cell.
- Node floor: nothing above 22.12 is used (`renameSync`, `existsSync`,
  `realpath`, `path.posix`, `spawnSync` env) — the row holds.
- git floor: raised 2.16 → 2.38 for `safe.bareRepository`, which git does
  introduce in 2.38 and honors from `-c` (protected configuration);
  `GIT_CONFIG_GLOBAL` (2.32) and `--no-optional-locks` (2.19) predate it. Claim
  checks out.
- Test strategy names a platform test for each supported row: macOS #12, linked
  worktree #13, mount points #5; Windows `not exercised` with rule 1 pinned on
  Linux in #2.
- Native modules: none added; no packaging story needed.

## FYI (no severity)

- The audit's `.git` matcher is described as covering "completed
  `write`/`edit`/`patch` calls and … shell command text"; if the tool-call half
  matches payload text rather than the call's path segments, a doc stage editing
  prose that quotes `.git/config` — as this document does — fails its own
  attempt. Worth pinning for the correctness/security seats.
