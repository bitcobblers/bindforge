# Compatibility review r1 (dissent pass c3)

verdict: approve

Scope reviewed: `git diff 89027e8 6b21e2f` over `02-proposal.md`, then the full
current document, plus `docs/adrs/005-runner-executed-file-moves.md` and the
Revision log. Inputs limited to those, per the caller.

## Prior findings — status against the r3 text

- [major] git identity under `git()`'s config-free env — **closed**. "That
  suppression drops the committer identity stage commits resolve today, so
  `git()`'s env pins a fixed pipeline author and committer" answers the
  mechanism half; the git row's "that env pins a fixed pipeline committer
  identity, so no operator `~/.gitconfig` is needed to land a stage commit" and
  #13's "a stage commit lands in a tmpRepo holding no global or system config at
  all" supply the positive assertion. Nothing outstanding beyond the nit below.
- [minor] macOS row / #12 unverifiable A10 evidence — **closed** as specified
  (the `notes.md` → `readme.md` pair in a repo tracking `README.md` is invisible
  to rule 6, so rule 5's disk check is what fires on macOS). One consequence of
  that closure is the new finding 1 below.
- [minor] `stagePathAllowed`'s `.git` suffix over-reach — **closed**. The
  predicate is now "any path with a case-folded `.git` segment", `**/*.git` is
  gone from the config mirror, and rule 7's `.git` test is made relative to the
  repo realpath. The linked-worktree row's claim still holds (the pointer file
  is `.git` itself, so segment equality covers it).

## Findings

- [minor][new] Compatibility matrix (macOS row) + Test strategy #12 + Validator
  rule 5 — "macOS rejects it at rule 5's disk check where Linux lands a
  case-variant sibling — that contrast is the A10 evidence" — this now pins, as
  the reference platform's expected behavior, creating a working tree that
  tracks two paths differing only in case (`README.md` and the newly moved
  `readme.md`). Rule 5 refuses `to` only when it exists on disk (the fold that
  happens there is the filesystem's, so it fires only on a case-insensitive
  checkout) or is tracked at `headBefore`, and rule 6's fold compares only the
  paths inside one reply, so nothing on Linux refuses the pair; `git add -A`
  then commits a tree with a case-only collision. The same matrix row claims
  macOS `yes`, and Windows is where rule 1's name checks are aimed: a checkout
  of that commit on a case-insensitive filesystem cannot represent both entries,
  so a Linux stage silently lands a commit that breaks the platform the row
  calls supported — the exact "case-insensitive checkouts behave unexpectedly"
  hazard the risk table tracks. Fix: make rule 5's tracked comparison case-folded
  on every platform — `to` is refused when its case-folded form equals any path
  tracked at `headBefore` or present on disk, replacing the bare "not tracked at
  `headBefore`" — which also matches rule 6's stated conservatism ("a repo with
  case-variant filenames cannot move both in one reply"). #12's contrast survives
  unchanged as A10 evidence with the Linux half restated as "refused at rule 5's
  tracked clause" against macOS's disk clause; the validator messages already
  name the failing pair, so add the clause they name to keep that discrimination
  observable, and extend #2 with the pair (`README.md` tracked, `notes.md` →
  `readme.md`) as a Linux rejection.
- [nit][new] Chosen design / Read-only git — "`git()`'s env pins a fixed
  pipeline author and committer" — the mechanism is named but the value is not,
  and it is an externally visible output on every platform row: every stage
  commit in every run, dogfood acceptance included (`git show --name-status -M`,
  review prompts reading `git log`), carries one identity that no operator can
  configure because global and system config are suppressed. Fix: name it in the
  same sentence — e.g. `GIT_AUTHOR_NAME`/`GIT_COMMITTER_NAME` = `bindforge`,
  `GIT_AUTHOR_EMAIL`/`GIT_COMMITTER_EMAIL` = a fixed pipeline address — and
  state it as env on `git()` (all four vars) rather than `-c user.*` on the two
  authoring calls, so the value is one documented constant rather than an
  implementation choice.

## Checklist

1. Matrix — every row carries a status; no `unknown`, no empty cell. macOS's
   `yes (unverified)` is a qualified status tied to a named operator-run test
   (#12), unchanged from r1/r2.
2. OS input APIs — n/a: the design touches no input path.
3. Node floor (≥ 22.12) — nothing above it used: `fs.renameSync`, `existsSync`,
   `path.posix`, realpath.
4. Native modules — none; "no new runtime dependency" holds (git subprocess is
   already the existing helper).
5. Keyboard reality — n/a. Non-ASCII paths: rule 1's character blacklist plus
   rule 6's `toLowerCase()` fold are locale-independent for ASCII; no finding.
6. File paths — `path.posix` segment semantics plus realpath containment, no
   string concatenation; per-OS config dirs are out of scope for this design.
   Case handling is the subject of finding 1.
7. Platform tests per supported row — Linux (all), macOS (#12), linked worktree
   (#13), mount-spanning (#5, operator-run and labelled as such), Node and git
   floors pinned by the CI environment and #13's config-free tmpRepo. Windows is
   not claimed as supported and its rule 1 checks pin on Linux.

## FYI (no severity)

- The new `subagent` deny on the two seats plus A9's `subagent` clause removes a
  capability those agents have today; whether any current flow relies on it is
  the scope/correctness seat's call, not the compatibility checklist's.
- A `git-plant-detected` audit hit "ends the stage ... no attempt 2 and no
  finalize", which is a new terminal class outside the A2 retry chain the wiring
  section pins for every other reply failure — worth a correctness seat's eye.
- Out-of-worktree config landings are now explicitly Deferred, but a stage's own
  `git status`/`git log` (agent shell, ambient config) still reads a `~/.gitconfig`
  it could write; consistent with ADR-005 item 5, so not a finding here.
