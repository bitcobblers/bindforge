# compatibility review r5c0 (ratification)

verdict: approve

Scope: the diff of 02-proposal.md blobs 2168fac..967d5e8 (the r4 revisions), then the
full current document. Inputs: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log, prior findings (none). Checking only that the r4 revisions did not
break what round 5 approved.

## Checklist walk

1. Matrix: every row carries a status (yes / yes (unverified) / not exercised / partial);
   no empty cells, no `unknown`. The macOS `yes (unverified)` row was unchanged by the
   diff and is pinned to #12 — not re-opened.
2. OS input APIs: not applicable — the design touches no input surface.
3. Node floor: the r4 revision raises it from ≥22.12.0 to ≥22.23.4. Every API the design
   uses (`fs.renameSync`, `lstatSync`, `path.posix`, zod, git CLI) predates Node 22 by
   years, so nothing is used above the floor. The raise itself is a user-visible
   compatibility change — see the one finding below, it is about scoping, not the number.
4. Native modules: "no new runtime dependency" — pure JS plus the git CLI. Unchanged by
   the diff, consistent.
5. Keyboard reality: not applicable.
6. File paths: rule 1 keeps POSIX segment comparison via `path.posix`, rejects `\` and
   Windows-illegal names on every platform; the rule 4/5 switch from `cat-file`/
   `existsSync` to `ls-tree`/`lstatSync` is path-mechanics-neutral for separators and
   case; A10's basis text was updated in step with rules 4–5 and #12, consistently.
7. Platform tests: Linux (every test), macOS (#12), linked worktree (#13), mount points
   (#5, operator-run); Windows is not claimed supported. No support claim lost its test.

Internal-consistency check of the r4 edits: rules 4/5, A10, matrix git row, #2, #12, and
the Revision log validator entry all moved together (`cat-file`→`ls-tree`,
`existsSync`→disk dentry/`lstatSync`) with no leftover reference to the old mechanism.
The new dangling-symlink and tracked-symlink test cases in #2 match the new rule text.

## Findings

- [minor][new] Context — "ADR-005 defers its earlier item 5, read-only git in stage
  shells, to a follow-up ADR, so this change ships items 1-4 only." — the r4 revision
  raises the Node engines floor (Compatibility matrix: "Node ≥ 22.23.4 (engines raised
  from ≥22.12.0, tools/orchestration/package.json:30-32)"; Revision log: "Node floor
  raised to ≥22.23.4 (r4: ... engines included)"), a package.json change outside ADR-005's
  items 1-4 that no other section anchors: Goals, Chosen design (which enumerates the
  touched files — types.ts, moves.ts, runner.ts, prompts), Rollout, and Test strategy
  never mention it. The floor itself is compatibility-safe, but as written the matrix
  promises a floor the implementation plan never commits to set, and the Context sentence
  is now literally false. Fix: add one Rollout line — "`tools/orchestration/package.json`
  `engines.node` is raised to `>=22.23.4`" — which also makes the Context's "items 1-4
  only" read as scoped to the move feature rather than to the whole diff.

## FYI (no severity)

- Validator rule 4 — "`git ls-tree <headBefore> -- <from>`" — the r4 switch away from
  `git cat-file -t <headBefore>:<from>` makes git pathspec glob magic (`*`, `?`, `[`)
  significant for the first time: rule 1's character blacklist does not reject them, and
  a glob-matched or multi-line ls-tree output could mis-drive the "mode
  100644/100755" parse. Correctness seat's territory (path handling is rule 1's, which
  this diff left unchanged), noted without severity.
