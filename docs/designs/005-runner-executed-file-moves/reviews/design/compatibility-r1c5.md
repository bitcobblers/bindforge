# Compatibility review r1 (ratification pass c5)

verdict: approve

Scope as given: `git diff 6b21e2f4 44c52fa6` over `02-proposal.md`, then the full
current document. Checklist applied: `references/compatibility.md`. Inputs used:
the proposal, `docs/adrs/005-runner-executed-file-moves.md`, the Revision log, my
round-1 findings. No review report or transcript was opened.

Checklist state: matrix rows all carry a status (no empty cell, no `unknown`);
OS input APIs — not applicable, the design touches no input path; Node floor —
`fs.renameSync` only, far below the ≥ 22.12 floor; native modules — none, no
packaging story needed; keyboard reality — not applicable; file paths — rule 1's
`path.posix` segment semantics plus explicit `\`/reserved-name rejection, pinned
on Linux in #2; platform tests — macOS by #12, linked worktree by #13,
mount-spanning by #5 (operator-run). The one unsupported-by-test band is the
git-version row, filed below.

## Findings

- [minor][new] Compatibility matrix, git row — "floor set by `GIT_CONFIG_GLOBAL`
  (introduced in git 2.32)" — this round lowered the floor from 2.38 to 2.32 and
  in the same round put `-c core.fsmonitor=false` into the shared git env, so the
  row now claims uniform support for a band in which one of its own pins does not
  mean what the row implies. git's own config documentation for `core.fsmonitor`
  warns: "Git versions 2.35.1 and prior will not understand the boolean values and
  will consider the "true" or "false" values as hook pathnames to be invoked. Git
  versions 2.26 thru 2.35.1 default to hook protocol V2 and will fall back to no
  fsmonitor (full scan)". So inside the newly claimed floor band (2.32 through
  2.35.1) the pin is read as a hook path, not as an off switch: it still shadows a
  planted value (command-line `-c` outranks repo-local config), but it disables
  nothing by name, and #13's marker-file assertion runs against CI's git only, so
  no test exercises the band the row calls `yes`. Nothing in the row or in Test
  strategy says which version the env assertions actually ran against.
  Fix: set the floor at the version whose semantics every pin in the shared env
  actually has (git's boolean `core.fsmonitor` reading — the row then reads "floor
  set by `core.fsmonitor`'s boolean value; `GIT_CONFIG_GLOBAL` (2.32),
  `core.hooksPath` (2.9), `--no-optional-locks` and `git cat-file -t` all predate
  it"), and drop the Revision log's 2.32 line to match; or, to keep 2.32, replace
  nothing and add the one fact the row now omits — below the boolean reading the
  pin shadows the planted value as an uninvokable hook path and git falls back to
  a full index scan — and name in #13 the git version the ship gate ran on, so the
  band below it is labelled unverified the way the macOS row's is.

- [nit][re-raised] Chosen design, Read-only git — "`git()`'s env (106-112) becomes
  the shared env for every runner git call — `gitOk` (143-145) and the raw
  `merge-base` spawn (1417-1420) included — setting `GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL` at an empty file, `-c core.fsmonitor=false -c
  core.hooksPath=<empty dir>`, and a fixed pipeline author and committer replacing
  the identity that env suppresses" — my r1 finding's mechanism half landed (one
  shared env over every runner git call), its value half did not: the identity is
  still unnamed, while the matrix row still advertises it as a platform-visible
  output ("that env pins a fixed pipeline committer identity, so no operator
  `~/.gitconfig` is needed to land a stage commit") and the dogfood acceptance and
  review prompts both read those commits, so it is an externally visible constant
  no operator can configure. The sentence also now files two `-c` command-line
  flags under the word "env", and `<empty dir>` is a brand-new on-disk artifact
  named nowhere else in the document, though every platform row has to supply it.
  Fix: in that one sentence name `GIT_AUTHOR_NAME`/`GIT_COMMITTER_NAME` =
  `bindforge` and `GIT_AUTHOR_EMAIL`/`GIT_COMMITTER_EMAIL` = one fixed pipeline
  address, move the two `-c` flags into a separate "and `-c` on every call"
  clause, and name the empty hooks directory (where the runner creates it, outside
  the worktree) instead of the `<empty dir>` placeholder.

- [nit][new] Chosen design, Validator rule 7 — "rule 2's `.git` segment test then
  runs on the resolved path relative to that realpath." — the clause that used to
  follow ("so a checkout under a `project.git` ancestor stays legal") was deleted
  in this revision. The behavior is still specified, but that example was the only
  text making its purpose observable: read alone, rule 2 ("no endpoint
  `.git`-denied") plus rule 7 are satisfied by a predicate that scans absolute
  paths, which refuses every move in a repo checked out under an ancestor whose
  name ends in `.git` (a `git clone …/foo.git` layout) — a platform-layout change
  none of Test strategy #2's `.git` endpoint assertions would catch, since they
  all place the offending segment inside the repo.
  Fix: restore the deleted consequence in the same sentence — "…relative to that
  realpath, so a checkout under a `*.git` ancestor stays legal" — rather than any
  new rule text; optionally extend #2 with one legal-pair case in a tmpRepo whose
  ancestor ends in `.git`, which is the assertion that keeps the relative form
  from being implemented as an absolute one.

## FYI (no severity)

- Risks table, first row — "one shared env over every runner git call reads no
  ambient config … so nothing outside the repo executes at runner privilege" —
  `GIT_CONFIG_NOSYSTEM`/`GIT_CONFIG_GLOBAL` close the outside-in direction, but
  A6's own list includes `include.path`, and a planted repo-local `.git/config`
  can pull a file from outside the worktree inward; the sentence's absolute claim
  belongs to the security seat, and the design already defers exactly that landing
  class to the sandbox line.
- Test strategy #13 — "never discovers a bare repo planted in the worktree" —
  this round dropped the `-c safe.bareRepository=explicit` pin that used to back
  that assertion, leaving `cwd: repoDir` as the sole mechanism; consistent with
  the Chosen design's own argument, but it is the security seat's call whether the
  assertion keeps its teeth.
- ADR-005 item 3 says the runner performs the renames "through its own git
  plumbing"; the design's chosen primitive is `fs.renameSync` plus the existing
  `git add -A` (Revision log r0, research OQ1). Substance — one stage commit,
  rename detection intact — is met; wording conformance is for the correctness
  seat, not raised here.
- My r1 minor about rule 5's tracked comparison not being case-folded (macOS row +
  #12 + rule 5) sits in text this diff did not touch: rule 5 still reads "not
  tracked at `headBefore`" and #12 is unchanged, so it is neither broken nor
  closed by this revision, and per the ratification scope it is not re-filed. The
  Revision log's Validator entry records "case-folded comparisons" for rule 6 only,
  so the rule 5 / rule 6 asymmetry remains the one place a Linux stage can still
  land a tree a case-insensitive checkout cannot represent.
