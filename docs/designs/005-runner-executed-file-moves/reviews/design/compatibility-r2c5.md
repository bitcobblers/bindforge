# Compatibility review r2 (c5 ratification pass)

verdict: approve

Scope per caller: `git diff 82b346f0 2168fac7` of `02-proposal.md` (the
revisions since the r2 approval), then the full current document. Inputs:
`02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`, the
in-document Revision log, and this seat's r2 findings. The diff's dominant
change is the removal of item 5 (read-only git in stage shells): the Read-only
git section, its goal, its risk row, its Options paragraph, A9, #13's ship
gate, the audit-matcher classification in #6, and the shared git env all leave
the change; the git compatibility row is rewritten to "no new floor"; the
`.git` endpoint denial becomes validator-local (rule 2, re-run resolved by
rule 7); the Deferred section and Revision log record the deferral.

## Prior findings — disposition

- [minor, r2] Compatibility matrix git row / `core.fsmonitor=false` band below
  the 2.32 floor — **closed by removal**. The row now reads
  "git, any version this repo runs today | yes | no new floor: the only git
  call this change adds is the validator's `git cat-file -t` (rule 4); rename
  detection stays at git's default (A5)", and the shared env that carried
  `-c core.fsmonitor=false` and `GIT_CONFIG_GLOBAL` is gone from the proposal
  (Revision log: "the audit matcher, git-env pin and config allowlists go with
  it"). The row no longer sets a floor, so no version-semantics band of that
  env is being sold. Soundness of the rewritten row checked: the repo's
  existing `gitOk` helper already issues `cat-file -e` at
  `tools/orchestration/src/pipeline/runner.ts:135,153,235`, so a
  `cat-file -t <headBefore>:<from>` spawn needs no new floor, and grep for
  `GIT_CONFIG|fsmonitor|hooksPath` in `tools/orchestration/src` matches
  nothing — the env was never shipped code, so removing it from the proposal
  creates no code-vs-proposal mismatch. The Revision log's replacement line
  ("no new git version floor (only the validator's `git cat-file -t` is
  added)") matches the row, which is exactly the second branch of my r2 fix.
  Not re-reported.
- [nit, r2] Chosen design "Read-only git" — unnamed identity vars, `-c` flags
  filed under "env", unnamed `<empty dir>` — **closed by removal**. The entire
  cited sentence went to the follow-up ADR with item 5, so the naming gaps
  (checklist item 6's on-disk artifacts, the committer-identity claim behind
  the old git row) are no longer in this change and no dogfood check in this
  document depends on a pinned pipeline identity. Fix moot.

## Checklist walk (current document)

1. Matrix status per row: Linux `yes`, macOS `yes (unverified)`, Windows
   `not exercised`, linked worktree `yes`, mount points `partial`,
   Node ≥ 22.12 `yes`, git `yes`. No `unknown`, no empty cell. The macOS and
   Windows labels are unchanged from r2 and were accepted then.
2. OS input APIs — n/a: the change moves files via `fs.renameSync`; no input
   API is introduced on any platform.
3. Node floor — unchanged row; `fs.renameSync` and the existing git helper are
   long under 22.12; no new API appears in the revised sections.
4. Native modules — none introduced; "no new runtime dependency" stands.
5. Keyboard reality — n/a.
6. File paths — rule 1's `path.posix` segment semantics with explicit
   rejection of `\`, `:`, drive-absolute, reserved names, and trailing
   `.`/space is unchanged and pinned on Linux in #2; rule 7's realpath
   containment is unchanged. The `<empty dir>`/empty-config artifacts my r2
   nit required naming are gone with item 5; no new on-disk artifact appears
   in the revised sections.
7. Test strategy vs. matrix — macOS round trip (#12) and linked-worktree
   round trip (#13) both survive the item-5 deletions intact; mount-point
   `EXDEV` (#5) survives; Linux is the reference platform running the rest.
   The removed #13 ship gate belonged to item 5, not to any matrix row this
   change claims, so checklist item 7 stays satisfied.

## Deferred section

Re-opened only against the ADR as instructed: the new deferral entry is a
faithful restatement of docs/adrs/005-runner-executed-file-moves.md's
"**Deferred — read-only git in stage shells.** ... It is deferred to a
follow-up ADR. Items 1-4 stand without it" — no contradiction; not re-opened.
The in-body claim that rule 2's `.git` denial is validator-local rather than a
`stagePathAllowed` change is also consistent with ADR item 2, which names only
`stagePathAllowed` and the runner-reserved paths.

## Findings

None. Zero open blocker/major findings.

## FYI (no severity)

- Validator rule 4's rev-spec `<headBefore>:<from>` interpolates the path into
  a git argument; rule 1 strips `:`, `/`, `..` and absolute forms first, so
  no compatibility break results, but argument-level (not shell-level) quoting
  of the spec is an implementation detail another seat may want pinned.
