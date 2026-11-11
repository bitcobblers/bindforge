# Research: Runner-executed file moves (ADR-005)

> In-repo claims are cited as `path:lines`; everything else carries a URL.
> Lenses applied: Prior art (in Prior art), Security landscape (in Risks &
> unknowns), Platform notes (in Platform notes).

## Summary

ADR-005 proposes that the `implement`, `fix` and `test-update` replies grow a
`moves: { from, to }[]` field, that the runner validates the whole list before
touching anything, and that the runner — still the only committer — executes the
renames with its own git plumbing between the ownership sweep and `commitStage`
(docs/adrs/005-runner-executed-file-moves.md:36-59). Its diagnosis reproduces
line by line against run 004's artifacts: the reply schemas carry no move
primitive, the ownership sweep reverts every out-of-ownership delete, the coder
and test-engineer shells allow only `npm` commands, and the run escalated with
`tests not green at the code-review budget limit` until the operator landed
`git mv` ×23 by hand. Item 5's second half — read-only git inside stage shells —
is the part with an external track record, and that record says the four named
commands are not actually write-free, so the design must pin how they are
invoked, not only which ones are named. The naming shape `{from, to}` matches
Terraform's `moved` block; the "moves are data the harness applies" shape
matches OpenAI Codex's `apply_patch` (`*** Move to:`).

## Findings

- Nothing in today's reply contract can express a move: `test-update` is
  `{ commitMessage }`, `implement` and `fix` are `{ testsNeedUpdate,
  commitMessage }` (source: tools/orchestration/src/pipeline/types.ts:44-46).
- Reply validation already happens at one choke point, so `moves` needs no new
  mechanism: the raw text is JSON-extracted and synonym-normalized, then
  `schema.safeParse`; a failure produces `failure: 'invalid-reply: <issue>'`
  (source: tools/orchestration/src/pipeline/runner.ts:873, 912-916, 947-950).
  The same schema is rendered into the prompt as JSON Schema, so a new field is
  self-documenting to the model (source: tools/orchestration/src/pipeline/runner.ts:709-710, 447-457).
- "Routes the stage to retry, exactly like an invalid reply today" is a real,
  narrow path: only a failure string starting `invalid-reply` reuses the same
  session for attempt 2, and a second failure escalates (source:
  tools/orchestration/src/pipeline/runner.ts:1121-1126, 1453-1497; pinned at
  tools/orchestration/tests/pipeline/reply-contract.test.ts:172-182). Note the
  cost: every `fix` entry burns one unit of the code budget (source:
  tools/orchestration/src/pipeline/runner.ts:1712-1714), so a rejected `moves`
  list is a budget event, not a free correction.
- Finalize order is fixed and single-commit: `headBefore` is captured before the
  prompt (source: tools/orchestration/src/pipeline/runner.ts:1077), then
  `finalizeStage` runs `enforceOwnership` and afterwards `commitStage`
  (`git add -A` → `git diff --cached --quiet` → `git commit -m`) (source:
  tools/orchestration/src/pipeline/runner.ts:1038-1063, 288-309). Item 3's
  "before the ownership sweep and commit" is therefore a specific insertion
  point between those two calls, not a general capability (source:
  tools/orchestration/src/pipeline/runner.ts:1049-1053).
- The sweep is what deletes an agent's move, and it sees a move as two
  unrelated paths: the dirty list comes from `git status --porcelain=v1 -z
  --untracked-files=all --no-renames`, deliberately `--no-renames` so a rename
  is never collapsed into one `old -> new` pseudo-path (source:
  tools/orchestration/src/pipeline/runner.ts:159-182). An out-of-ownership path
  that existed at `headBefore` is restored with `git restore --source=<headBefore>
  --staged --worktree`; one that did not exist is `git rm -f` plus an
  `rmSync` (source: tools/orchestration/src/pipeline/runner.ts:234-247). The
  current behaviour for a renamed tracked file is pinned by a test that expects
  the move to be reverted (source: tools/orchestration/tests/pipeline/ownership.test.ts:148-159).
- The ownership matrix is a path predicate per stage, with no move rule and no
  reserved-path rule: `implement`/`fix` may touch anything but tests,
  `test-authoring`/`test-update` may touch docs and tests, everyone else docs
  only (source: tools/orchestration/src/pipeline/gates.ts:253, 260-273). Its
  header states the reason the runner, not agent config, is the enforcement
  point: "Agent-config permissions are only advisory (a model can route around
  them via subagents)" (source: tools/orchestration/src/pipeline/gates.ts:255-259).
- The runner-reserved paths ADR-005 item 2 names exist as constants but are not
  consulted by the ownership code today: `STATE_FILE = '.pipeline.json'`
  (source: tools/orchestration/src/pipeline/types.ts:143) is kept away from
  agents by an edit-deny rule in every agent definition (source:
  .opencode/agents/code/coder.md:17-19), and `LOCK_FILE = '.pipeline.lock'`
  (source: tools/orchestration/src/pipeline/runner.ts:102, 354-383) is invisible
  to the sweep because it is gitignored (source: .gitignore:3). So the two
  reserved-path checks the ADR wants are genuinely new surface.
- Checkpoint growth is bounded by an explicit convention the new field must
  honour: `.pipeline.json` is `version: 1`, `loadState` refuses any other
  version with no migration path, every post-run-1 field is optional, and
  `saveState` re-validates before the atomic tmp+rename write (source:
  tools/orchestration/src/pipeline/state-store.ts:42, 145-148, 178-189; rule at
  docs/adrs/003-parallel-inference-lane.md:115-116).
- Today's shell allowlists make the ADR's context sentence exact: the coder's
  shell is deny-all except `npm run check*` and `npm run format*`, the test
  engineer adds `npm test*`, and the designer and editor agents deny shell with
  no allow at all (source: .opencode/agents/code/coder.md:26-34,
  .opencode/agents/code/test-engineer.md:23-34,
  .opencode/agents/design/designer.md:20-22, .opencode/agents/design/editor.md:20-22).
  Read-only git is not new to the
  harness — review and doc-sync seats already carry `git diff*` and the code
  seats `git log*` (source: .opencode/agents/codereview/code-correctness.md:27-36,
  .opencode/agents/review/design-security.md:30, .opencode/agents/ops/doc-sync.md:24),
  so item 5 extends an existing pattern rather than inventing one.
- Run 004 is evidence, not anecdote. Its build notes record that Rollout 2's
  "payload — the 23 `git mv`s … lives entirely in paths matched by
  `TEST_PATH_RE`", that "no agent has git access … no `git mv`, no `git rm`, and
  no delete primitive in the stage's tool set", and that a copy-to-nested pass
  would fail the `git ls-files` assertion at 49 tracked paths (source:
  docs/designs/004-mirror-src-hierarchy-in-tests/03-build-notes.md:27-59).
  Deviation 4 states the same from the test-update side and concludes the move
  is "operator-level work" (source:
  docs/designs/004-mirror-src-hierarchy-in-tests/03-build-notes.md:144-160).
- The escalation and its cost are recorded mechanically: gateLog entries
  `"reason": "tests not green at the code-review budget limit"`, followed by
  `operatorForce` ("operator forced the next step: test-update") and
  `operatorExtension` (+3 review sessions over the caps of 40/20), with 5 `fix`
  and 6 `test-update` stageResults carrying earlier timestamps (source: the
  stageResults at docs/designs/004-mirror-src-hierarchy-in-tests/.pipeline.json:
  619-766, and the gateLog at :1181-1201). The
  escalation string is produced by the state machine's budget backstop (source:
  tools/orchestration/src/pipeline/states.ts:161-178). The operator's move commit
  `de51504a` ("refactor: mirror src/ hierarchy in orchestration test tree")
  landed 23 renames plus the content rewrites plus an ADR clause no stage could
  write (source: docs/designs/004-mirror-src-hierarchy-in-tests/03-build-notes.md:229-237, 807-825).
- Rounds were spent without even being able to see state: the notes repeatedly
  record `git status`/`git log` "rejected" inside fix and test-update rounds
  (source: docs/designs/004-mirror-src-hierarchy-in-tests/03-build-notes.md:122-124, 223-225, 330-331), and one
  transcript logs `{"error":{"type":"permission.rejected","message":"Permission
  denied: shell"}}` (source:
  docs/designs/004-mirror-src-hierarchy-in-tests/reviews/code/testing-r1c2.md:66-69).
- Rename detection is a diff-time computation over a delete/add pair, with
  defaults the ADR can rely on: `diff.renames` defaults to `true` and affects
  porcelain diff/log commands, the default similarity index is 50%, `-M<n>` sets
  the threshold, `diff.renameLimit` defaults to 1000 (0 means unlimited),
  `--no-renames` overrides config, and the similarity index is "the percentage
  of unchanged lines" (source: https://git-scm.com/docs/git-diff). The runner's
  own sweep intentionally opts out of it (source:
  tools/orchestration/src/pipeline/runner.ts:159-182).
- The 91-99% similarity figure ADR-005:49-50 quotes from run 004 was measured
  on a move *plus* content rewrites, and two files scored 71% and 80% there
  because earlier commits had added suites (source:
  docs/designs/004-mirror-src-hierarchy-in-tests/reviews/code/security-r1c0.md:9-12).
  The proposal's own A15 wording is "near 100% (rename detection is on by
  default at 50% threshold)" (source:
  docs/designs/004-mirror-src-hierarchy-in-tests/02-proposal.md:48). A pure
  runner-side rename with no content change scores highest; the threshold is
  what protects a "moved" pair from being displayed as delete+add.
- Git offers both the high-level and plumbing shapes for the runner's execute
  step: `git mv` renames a file/directory/symlink, updates the index, "the
  change must still be committed", refuses an existing destination unless `-f`,
  skips erroring pairs under `-k`, and can be dry-run with `-n` (source:
  https://git-scm.com/docs/git-mv). Index-level alternatives are
  `git update-index --add --remove` / `--force-remove` / `--replace` (source:
  https://git-scm.com/docs/git-update-index).
- The machine-readable shape of a rename already exists in git's own script
  output: short/porcelain status prints `<xy> <orig-path> -> <path>` and quotes
  unusual names as C string literals unless `-z` is used (source:
  https://git-scm.com/docs/git-status). BindForge already parses the `-z` form
  (source: tools/orchestration/src/pipeline/runner.ts:166-171), so a `moves`
  reply is compatible with how it reads paths everywhere else.

## Prior art

- **Moves as data in a model's reply, applied by the harness** — OpenAI Codex's
  `apply_patch` is a grammar-constrained patch whose update hunk may carry a
  move: `change_move: "*** Move to: " filename LF`, alongside `Add File` and
  `Delete File` hunks. The model emits text; the tool applies it. That is
  structurally ADR-005 item 1 (source:
  https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide).
- **Moves as declared configuration, executed by the tool** — Terraform's
  `moved` block is literally `moved { from = <old address> to = <new address> }`
  and is used to "programmatically change the address of a resource" during
  state refactoring, i.e. the user declares the move and the CLI performs it.
  The ADR's `{ from, to }` field name matches this precedent (source:
  https://developer.hashicorp.com/terraform/language/moved).
- **Moves as an operation in a patch format** — RFC 6902 JSON Patch defines
  `{"op":"move","from":"/a/b/c","path":"/a/b/d"}`: `from` MUST exist, the
  operation is equivalent to a `remove` followed immediately by an `add`, and
  "the `from` location MUST NOT be a proper prefix of the `path` location;
  i.e., a location cannot be moved into its own contents" — a validation rule
  ADR-005 item 2 does not list but a directory move needs (source:
  https://datatracker.ietf.org/doc/html/rfc6902). RFC 6902 also applies
  operations "sequentially in the order they appear", stopping at the first
  error (source: https://datatracker.ietf.org/doc/html/rfc6902), which is a
  different contract from "validate all pairs before touching anything".
- **Validate-then-apply and path containment in a patch tool** — `git apply
  --check` "instead of applying the patch, see if the patch is applicable to
  the current working tree and/or the index file and detects errors" (source:
  https://git-scm.com/docs/git-apply); and `git apply` by default rejects a
  patch touching outside the working area, "rejected as a mistake (or a
  mischief)", requiring `--unsafe-paths` to override (source:
  https://git-scm.com/docs/git-apply). Both are direct precedents for item 2's
  all-or-nothing preflight and for repo-relative containment.
- **Command allowlists for agent shells** — Claude Code matches shell rules
  against whole command text with prefix patterns, splits compound commands on
  `&&`, `||`, `;`, `|`, `|&`, `&` and newlines and requires *every* subcommand
  to be allowed, applies deny/ask inside subshells and command substitutions,
  ignores a bypassable rule shape such as `Bash(command:rm *)` outright, and
  applies a project's committed `permissions.allow` rules only after a
  workspace-trust acceptance (source: https://code.claude.com/docs/en/permissions.md).
  It also documents that read-only-looking commands are re-checked when they
  carry write-capable or exec-capable flags, naming `git` explicitly among them
  (source: https://code.claude.com/docs/en/permissions.md).
- **Sandbox boundaries around agent command execution** — Codex separates
  `sandbox_mode = read-only | workspace-write | danger-full-access` from the
  approval policy, and in `workspace-write` it protects
  `<writable_root>/.git` as read-only "whether it appears as a directory or
  file", including the resolved Git directory when `.git` is a `gitdir:` pointer
  file, recursively; an untrusted project also disables project-local
  configuration (source: https://learn.chatgpt.com/docs/agent-approvals-security,
  https://learn.chatgpt.com/docs/config-file/config-reference). BindForge has no
  equivalent: no agent definition in `.opencode/agents/` mentions `.git` at all
  (source: .opencode/agents/code/coder.md:5-34).
- **Advisory-then-enforced split** — BindForge already states the principle the
  wider ecosystem uses (permissions advisory, runner enforces deterministically)
  in code (source: tools/orchestration/src/pipeline/gates.ts:255-259), and
  Claude Code's docs state the same defence-in-depth rationale — sandbox
  restrictions "still apply even if a prompt injection bypasses Claude's
  decision-making" (source: https://code.claude.com/docs/en/permissions.md).

## Platform notes

- Toolchain floor is Node ≥ 22.12 for this package (source:
  tools/orchestration/package.json:30-32); the runner shells out to git with
  `spawnSync` today and has no other git client (source:
  tools/orchestration/src/pipeline/runner.ts:106-112, 143-145).
- `fs.rename`/`fs.renameSync` are thin wrappers over POSIX `rename(2)` — Node's
  docs defer semantics to it, and `fs.rename` raises when `newPath` is an
  existing directory (source: https://nodejs.org/api/fs.html). `rename(2)`
  replaces an existing `newpath` atomically, fails `EXDEV` across mount points,
  `EISDIR` when `newpath` is a directory and `oldpath` is not, and `EINVAL` when
  the new pathname is a path prefix of the old (a directory moved into itself);
  opt-in `RENAME_NOREPLACE` exists where the filesystem supports it — ext4 since
  Linux 3.15, xfs since 4.0, many others since 4.9 (source:
  https://man7.org/linux/man-pages/man2/rename.2.html). ADR item 2's "every
  `to` must not exist" is what makes `RENAME_NOREPLACE` unnecessary but is also
  what makes a TOCTOU window worth naming.
- Case-only renames are platform-ambiguous: git keeps `core.ignoreCase` as an
  internal workaround for case-insensitive filesystems (APFS, HFS+, FAT, NTFS),
  set true by `clone`/`init` when appropriate, and warns that modifying the
  value "may result in unexpected behavior"; `core.protectNTFS` defaults to true
  on Windows (source: https://git-scm.com/docs/git-config). A runner-executed
  case-only move plus a case-insensitive checkout is a distinct case the ADR's
  "every `to` must not exist" check does not cover on such filesystems. Run 004
  had already met the question and parked it as acceptance criterion A16 —
  "Case-insensitive checkouts (macOS/Windows) see nothing special: no case-only
  or case-colliding renames among the 23 names", marked `unverified` (source:
  docs/designs/004-mirror-src-hierarchy-in-tests/02-proposal.md:48-49).
- Read-only git commands are not side-effect-free by default: `git status`
  refreshes the index as an optional side operation, and `GIT_OPTIONAL_LOCKS`
  (equivalently `git --no-optional-locks`) defaults to 1 and "will prevent
  `git status` from refreshing the index as a side effect" when disabled —
  documented precisely to avoid "lock contention with other operations on the
  repository" (source: https://git-scm.com/docs/git). The runner runs its own
  git commands continuously (source:
  tools/orchestration/src/pipeline/runner.ts:106-112, 178-182), so a stage's
  `git status` inside a running pipeline is a lock-contention partner.
- A checkout can be a linked git worktree — this run's own working directory
  sits under `.local/share/opencode/worktree/` — in which case `.git` is a
  `gitdir:` pointer file and the real git directory, where hooks and local
  config live, is elsewhere: a protection written only for a `.git` directory
  misses it, which is why Codex protects the pointer file *and* the resolved
  Git directory (source: https://learn.chatgpt.com/docs/agent-approvals-security).
  Hooks are read from `$GIT_DIR/hooks` and can be relocated by `core.hooksPath`,
  a config value rather than a worktree file (source:
  https://git-scm.com/docs/git-config). The runner's own invariant is one
  pipeline process per repository, guarded by `.pipeline.lock` (source:
  docs/adrs/003-parallel-inference-lane.md:101-102,
  tools/orchestration/src/pipeline/runner.ts:354-383).

## Risks & unknowns

### Security landscape (lens)

- **Repo-local config is the actual lever, and the harness does not block it
  today.** Git scopes are system, global, local (`$GIT_DIR/config`), worktree
  (`$GIT_DIR/config.worktree`) and command (`-c`, `GIT_CONFIG_*`); some options
  are "only respected when they are specified in protected configuration", and
  git treats those scopes as user/administrator-controlled because "an attacker
  who controls these scopes can do substantial harm without using Git" (source:
  https://git-scm.com/docs/git-config). The coder agent's edit permission is
  `resource: "*"` allow with only `docs/**` carve-outs, and nothing in
  `.opencode/agents/**` names `.git` (source: .opencode/agents/code/coder.md:5-13;
  grep over .opencode shows no `.git` rule). ADR-005's claim that a stage "can
  plant hooks or repo-local config" (source:
  docs/adrs/005-runner-executed-file-moves.md:27-29) is therefore about a
  currently-open surface, and closing it (deny writes to `.git/**`, as Codex
  does for `.git` inside writable roots — source:
  https://learn.chatgpt.com/docs/agent-approvals-security) is a smaller fix than
  any command-level argument and applies whether or not item 5 lands.
- **Git's own RCE history is hook-from-repo-content execution.** CVE-2024-32002
  is the case where recursive clones on case-insensitive filesystems that
  support symlinks let a repository get Git to "execute a hook from the `.git/`
  directory during a clone operation, leading to Remote Code Execution" (source:
  https://github.blog/open-source/git/securing-git-addressing-5-new-vulnerabilities/,
  https://github.com/git/git/security/advisories/GHSA-8h77-4q3w-gfgv). Git's
  standing defence for untrusted repositories is ownership-based: it "will
  refuse to even parse a Git config of a repository owned by someone else, let
  alone run its hooks" unless the directory is in `safe.directory` (source:
  https://git-scm.com/docs/git-config). A BindForge stage agent runs as the same
  user in the operator's own checkout, so that protection does not apply —
  which is a second reason the ADR's reasoning survives review but its stated
  mechanism ("any permitted `git` invocation then executes") needs to be paired
  with a write-deny on `.git/**` and, for the runner's own git calls, with
  ignoring ambient config: `GIT_CONFIG_NOSYSTEM` and `GIT_CONFIG_SYSTEM`/
  `GIT_CONFIG_GLOBAL` set to `/dev/null` are documented for exactly building "a
  predictable environment" (source: https://git-scm.com/docs/git).
- **The four named read-only commands carry write-capable and exec-capable
  flags.** `git log --output=<file>` and `git diff --output=<file>` write to a
  caller-chosen path (source: https://git-scm.com/docs/git-log,
  https://git-scm.com/docs/git-diff). `--ext-diff` "allow[s] an external diff
  helper to be executed" and `--textconv` allows external text-conversion
  filters, with the drivers defined in `$GIT_DIR/config` or `$HOME/.gitconfig`
  and selected per-path from `.gitattributes`, which *is* a worktree file
  (source: https://git-scm.com/docs/git-log, https://git-scm.com/docs/gitattributes).
  `core.pager`'s value "is meant to be interpreted by the shell", with
  `$GIT_PAGER` taking precedence (source: https://git-scm.com/docs/git-config),
  and Claude Code's rule engine independently treats `git` as a command with
  write/exec-capable flags worth prompting over (source:
  https://code.claude.com/docs/en/permissions.md). Prefix-allowlisting
  `git status`/`git log`/`git diff`/`git show` is therefore a *subcommand*
  restriction, not a capability restriction: the ADR's wording "plumbing that
  cannot write refs, the index, or HEAD" (source:
  docs/adrs/005-runner-executed-file-moves.md:54-59) is true of the subcommand
  names but not of the argument space, and `git status` writes the index by
  default (source: https://git-scm.com/docs/git).
- **Move endpoints are a path-traversal surface.** The endpoints are model
  text; `git apply`'s default rejection of paths outside the working area
  exists as a "mischief" guard (source: https://git-scm.com/docs/git-apply),
  and a `to` path that escapes the repo, or a symlinked endpoint planted by a
  stage that can already write the worktree, turns a rename into a write
  outside ownership. RFC 6902's own move rules (must exist; not into its own
  contents) and `rename(2)`'s `EINVAL`/`EISDIR` cases are the minimal invariant
  set (source: https://datatracker.ietf.org/doc/html/rfc6902,
  https://man7.org/linux/man-pages/man2/rename.2.html).
- **A half-applied move list is a real state.** 23 renames cannot be applied
  atomically: `rename(2)` is per-path, and git's own `git mv -k` exists because
  some pairs may legitimately fail while others proceed (source:
  https://git-scm.com/docs/git-mv, https://man7.org/linux/man-pages/man2/rename.2.html).
  The runner's current failure posture is revert-and-warn rather than rollback
  (source: tools/orchestration/src/pipeline/runner.ts:238-247), and
  `commitStage` only commits what a stage left dirty (source:
  tools/orchestration/src/pipeline/runner.ts:295-308).
- **The runner already refuses to execute worktree-provided binaries.** A prior
  revision ran each touched package's `node_modules/.bin/prettier` before every
  commit; the code comment records why that was removed — `node_modules` is
  git-ignored, so a planted `node_modules/.bin/<tool>` is invisible to
  `dirtyFiles` and `enforceOwnership`, never appears in a human-reviewed commit,
  and would still be exec'd by the process that owns the checkpoint, the git
  authorship and the lock (source:
  tools/orchestration/src/pipeline/runner.ts:267-277). This constrains the
  ADR-005 execution primitive: the runner must apply moves in-process or via a
  fixed `git` invocation, never a tool resolved from the model-writable worktree.
- **Sanctioned deletions weaken the sweep's default.** Today a deleted tracked
  path outside ownership is simply restored from `headBefore` (source:
  tools/orchestration/src/pipeline/runner.ts:234-236); item 4 replaces a
  blanket rule with one keyed on reply content, i.e. the sweep starts trusting
  a validated reply as authorization data — a scope ADR-006 explicitly owns as
  unresolved for the checkpoint ("trusting an in-worktree file that every review
  stage may write", source: docs/adrs/006-self-update-checkpoint-compatibility.md:19-24, 33-42).
- **Prompt-side risk:** among the stages that would emit `moves`, only the `fix`
  templates teach a reply field in prose: `fix.md:5` and `fix-tests.md:9,15` tell
  the coder to set `testsNeedUpdate`, while `implement.md:1-7` and
  `test-update.md:1-4` name no reply field at all, so for those stages the reply
  contract reaches the model only as the JSON Schema `replyFormat` appends to
  every prompt — which is why a new `moves` field is self-documenting without
  prose (source: tools/orchestration/prompts/fix.md:5,
  tools/orchestration/prompts/fix-tests.md:9-15,
  tools/orchestration/prompts/implement.md:1-7,
  tools/orchestration/prompts/test-update.md:1-4,
  tools/orchestration/src/pipeline/runner.ts:710). Where template prose does
  change, it is pinned: `fix-tests.md` by content assertions (source:
  tools/orchestration/tests/pipeline/fix-trigger.test.ts:58-67), the template
  file each stage loads by the stage registry (source:
  tools/orchestration/tests/pipeline/prompts.test.ts:84-93), and the template
  name and hash recorded in transcripts (source:
  tools/orchestration/tests/pipeline/prompts.test.ts:75, 114-117).

## Open questions

1. Which execution primitive does the runner use — `git mv` per pair, or
   `update-index --add --remove` plus an `fs.rename`? `git mv` updates the index
   and still requires a commit, refuses an existing destination without `-f`, and
   has `-n`/`-k` semantics; if the runner runs `git mv`, does it inherit the
   ambient config problem the ADR cites against stage shells, and should the
   runner itself pass `--no-optional-locks` and neutral pager/hook config (source:
   https://git-scm.com/docs/git-mv, https://git-scm.com/docs/git)?
2. Does the contract allow moving directories, or only files? `git mv` moves
   files, directories and symlinks (source: https://git-scm.com/docs/git-mv);
   RFC 6902 forbids moving a location into its own contents and `rename(2)`
   returns `EINVAL` for that case — must item 2's validator add the
   prefix/self-containment check, and what happens to a move whose `to` is
   inside a directory another pair in the same list moves?
3. What are the ordering and cycle semantics of a `moves` list? RFC 6902 is
   sequential-with-first-error (source:
   https://datatracker.ietf.org/doc/html/rfc6902), the ADR wants validate-all-
   then-apply; must `A→B, B→A` or `A→B, B→C` be rejected up front, and does
   rename ordering change the resulting rename detection?
4. Is a partially executed list rolled back, and by whom? Nothing in the runner
   today undoes a partial filesystem mutation (source:
   tools/orchestration/src/pipeline/runner.ts:238-247), and a crash between
   "renames applied" and "stage committed" leaves a dirty worktree that the next
   resume's sweep will treat as unsanctioned deletions.
5. How does `from` "tracked at `headBefore`" interact with moves already landed
   in this run — e.g. a test-authoring stage that added a file in an earlier
   round, or a second `moves` entry for a path a previous round already moved?
6. Is a rejected `moves` list a `invalid-reply` retry (and therefore a budget
   event through the same counters that livelocked run 004 — source:
   tools/orchestration/src/pipeline/runner.ts:1712-1714,
   tools/orchestration/src/pipeline/states.ts:161-178), or a distinct failure
   class with its own retry shape?
7. Should item 5 land at all, and if so with what invocation policy — pinned
   `--no-optional-locks`, `GIT_PAGER=cat`, no `-c`, and does `git show`/`git log`
   get blocked when they carry `--output`/`--ext-diff`/`--textconv` (source:
   https://git-scm.com/docs/git-log, https://git-scm.com/docs/git)?
8. Should `.git/**` be explicitly denied to every stage agent regardless of the
   `moves` decision, mirroring Codex's protected-`.git` rule (source:
   https://learn.chatgpt.com/docs/agent-approvals-security)? Nothing denies it
   today (source: .opencode/agents/code/coder.md:5-13).
9. How are case-only renames specified and validated, given `core.ignoreCase`
   on APFS/NTFS (source: https://git-scm.com/docs/git-config), and are they
   allowed at all?
10. What is the acceptance measurement for the ADR's "dogfood acceptance: a
    later run of any file-moving change, green without a single escalation"
    (source: docs/adrs/005-runner-executed-file-moves.md:87-88) — which
    in-repo change is the first candidate, and which existing tests
    (ownership, pass-finalize, reply-contract, prompts) get the new pins?
