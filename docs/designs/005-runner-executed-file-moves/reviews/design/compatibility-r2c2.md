# Compatibility review r2 (ratification pass c2)

verdict: approve

Scope: `git diff 44c52fa 82b346f` over `02-proposal.md` (Read-only git paragraph,
Risks row 1, Test strategy #13, Deferred bullet 3, Revision log), then the full
current document. Checklist applied: `references/compatibility.md`.

Checklist walk, changed text only:
- Item 1 (matrix rows have a status): matrix untouched by this diff; every row
  carries a status, no empty cell, no `unknown`. Pass.
- Item 3 (Node floor): no new runtime API named; Node row unchanged. Pass.
- Item 4 (native modules): unchanged — `fs.renameSync` plus the existing git
  helper, no new dependency. Pass.
- Item 6 (file paths): the shared env's two on-disk artifacts are still unnamed
  placeholders — see the nit below. Open, not newly created.
- Item 7 (a test per claimed platform): #13 still names the `git worktree add`
  tmpRepo round trip (linked-worktree row) and still asserts config suppression
  plus a stage commit with no global or system config (git row). Nothing in the
  matrix lost its test; the payload swap (`core.fsmonitor` → `alias.status` /
  `filter.<driver>.clean`) actually removes the test's dependence on
  version-sensitive fsmonitor semantics. Pass, with the band caveat below.
- Items 2 and 5 (input APIs, keyboard reality): nothing in this design touches
  them. Not applicable.

Nothing in the revised text falsifies a matrix row or a platform pin I approved
in r2. The two open items below are carried unchanged in substance; both are
non-blocking.

## Findings

- [minor][re-raised] Compatibility matrix, git row — "git ≥ 2.32 | yes | floor
  set by `GIT_CONFIG_GLOBAL` (introduced in git 2.32)" — the row and the shared
  env are both unchanged by this revision, so the r2 gap stands: the env still
  pins `-c core.fsmonitor=false`, and git's own config documentation says
  "Git versions 2.35.1 and prior will not understand the boolean values and will
  consider the \"true\" or \"false\" values as hook pathnames to be invoked. Git
  versions 2.26 thru 2.35.1 default to hook protocol V2 and will fall back to no
  fsmonitor (full scan)" — so across part of the band the row calls `yes`, that
  pin shadows a planted value by name but disables nothing as an off switch. The
  only movement in this round is in Test strategy #13, whose hostile config
  changed from a `core.fsmonitor` marker to `alias.status='!…'` /
  `filter.evil.clean='!…'`; that payload is version-stable (good), but it means
  no assertion anywhere now exercises the fsmonitor pin at all, and the band
  below CI's git stays unverified and unlabeled, unlike the macOS row's
  `yes (unverified)`. Not newly created; not blocking. Fix: one clause in the
  row — "git's boolean `core.fsmonitor` reading lands later than the floor;
  below it `-c core.fsmonitor=false` is read as an uninvokable hook pathname and
  git falls back to a full index scan, which is why #13's ship gate carries no
  fsmonitor assertion and the band below the CI version is unverified" — or
  raise the floor to the version where every pin in the shared env means what the
  row implies, and update the Revision log's "git floor 2.32 from
  `GIT_CONFIG_GLOBAL`" line to match either choice.

- [nit][re-raised] Chosen design, Read-only git — "setting `GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL` at an empty file, `-c core.fsmonitor=false
  -c core.hooksPath=<empty dir>`, and a fixed pipeline author and committer
  replacing the identity those sources supply" — this exact sentence was
  rewritten this round and all three naming gaps from r2 survive it: the
  identity variables are still unnamed while the git row still sells that
  identity as the reason "no operator `~/.gitconfig` is needed to land a stage
  commit", and it is an output every dogfood check (`git show --name-status -M`)
  and review prompt reads, unconfigurable by any operator because global and
  system config are suppressed; the two `-c` command-line flags are still filed
  under the word "env"; and `<empty dir>` remains a placeholder for an on-disk
  artifact named nowhere else, even though checklist item 6 wants every path the
  design touches named and cross-platform (an empty dir plus an empty config file
  both need a runner-chosen location, off the worktree, on Linux and macOS). Fix:
  name `GIT_AUTHOR_NAME`/`GIT_COMMITTER_NAME` and
  `GIT_AUTHOR_EMAIL`/`GIT_COMMITTER_EMAIL` values in that sentence; split the
  flags into an "and `-c` on every call" clause; and name the empty hooks
  directory and the empty config file, and where the runner creates them.

## FYI (no severity)

- New text, Read-only git + Risks row 1 + #13 — the r2 sentence that admitted
  "a `pipeline resume` (runner.ts:1376) runs over it until the operator removes
  it" was replaced with "removing it is the operator's remediation step before
  the worktree sees another git call" and "no runner git touches the plant".
  Within the halted stage that holds (no commit), but nothing in the design stops
  a later resume's `gitOk`/`merge-base` from running over a plant the audit's
  `since` window never re-detects, and the section now says a planted
  repo-local `.git/config` "is read by every runner git call" while the only
  repo-local pins are `core.fsmonitor` and `core.hooksPath` — the new #13 payload
  (`alias.status`, `filter.evil.clean`) is precisely what those two keys do not
  cover, and #13 asserts only the no-commit path. Security/correctness lane.
- Scope vs the source ADR: docs/adrs/005-runner-executed-file-moves.md says of
  read-only git "It is deferred to a follow-up ADR. Items 1-4 stand without it",
  while this proposal's Context and Goals land item 5, and the Deferred bullet
  this round widened (now including "the review seats' child-session `.git`
  route the `subagent` deny does not reach") still ends "no follow-up yet". The
  ADR's flip in Rollout and these bullets need one consistent story. Scope lane.
- #13 — "hostile `alias.status='!…'` and `filter.evil.clean='!…'` plus a planted
  `.gitattributes`, both touching a marker file": "both" counts two of the three
  artifacts listed, and only the two config values can touch the marker.
  Language lane.
