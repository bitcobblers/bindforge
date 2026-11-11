# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:24:36.554Z
- ended: 2026-11-10T17:30:49.765Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 188d705ce82601480eb1b3318daec99a7595b254 64ae876fb3689573e750e2d79a658f530bbd0579` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Chosen design — \"Read-only git for implement/fix/test-update seats (item 5)\": \"Precondition, enforced: `stagePathAllowed` gains, as its first line, `path === '.git' || path.startsWith('.git/') → false`\"",
    "problem": "The mechanism named as enforcement of item 5's precondition cannot observe or undo the write it is meant to stop. Per A4 the sweep's entire input is git's dirty-path list, and `.git/**` is by construction never a dirty path; and \"restores violations from `headBefore`\" cannot restore `.git/config`, which has no blob at `headBefore`. A stage that writes `.git/config` (A7 says it can today) is neither blocked nor reverted: the plant survives the attempt boundary and is re-read by every later git call including the runner's own (`git add -A` in `commitStage`, the sweep's status/diff, the new rule-4 `git cat-file`), turning config keys into execution at runner privilege, outside the agent permission model. The only mechanism that can actually prevent the write — the agent-config `edit` deny — is demoted in the text to \"The advisory mirror\". Test strategy #4 only asserts the predicate returns false, which holds whether or not the write is preventable.",
    "fix": "Make the harness-side control load-bearing: name the OpenCode `edit`/`write` deny on `.git` and `.git/**` in `coder.md` and `test-engineer.md` as the enforcement and delete the word \"advisory\"; keep the `stagePathAllowed` first line as the validator endpoint rule and defense-in-depth only; replace test #4's predicate assertion with an attempt-and-observe test that a stage write to `.git/config` is refused. If the harness cannot be shown to deny that path, item 5 does not ship in this change."
  },
  {
    "severity": "major",
    "location": "Chosen design — item 5 paragraph: \"with `.git/config` unwritable, `diff.external`, `core.pager`, `core.hooksPath`, and `diff.*.textconv` drivers cannot be planted\" (and A6, which grounds it)",
    "problem": "The closed-lever list omits `core.fsmonitor`, the config sink behind the current class of git-config execution advisories, which fires on an index refresh rather than on a flag — and the design's only stated defence for `git status` is that `--no-optional-locks` \"avoids the index-refresh side effect\", while the newly allowed `git diff*`/`git show*` are exactly the calls that refresh the index. With argv vetting Deferred, every config-triggered sink is protected solely by the config write-deny of the finding above, so an incomplete list there is an unmitigated execution path, and the argv deny list (`--output*`, `--ext-diff*`, `--textconv*`) covers none of it. Risks row 1's mitigation is also mismatched: \"worktree landings are swept and land in visible commits\" mitigates a rogue write, not a rogue execution, which leaves nothing in the worktree to sweep.",
    "fix": "Add `core.fsmonitor` (and `filter.<driver>.clean`/`smudge`, `core.sshCommand`) to A6 and to the closed-lever list; restate Risks row 1's mitigation as the config-write deny plus a hostile `.git/config` fixture asserting the newly allowed commands do not execute it; state explicitly that the argv deny list is best-effort, not the control."
  },
  {
    "severity": "minor",
    "location": "Chosen design — \"Reply field\": \"`moves: z.array(MovePairSchema).default([])`\"",
    "problem": "The untrusted list carries no bound — no maximum pair count and no maximum path length — while rule 4 spends a `git cat-file` subprocess plus `existsSync`/`realpath` per pair and rule 6's \"no listed path is a path-prefix of another\" is quadratic in list size. A reply steered by injected content makes finalize burn unbounded subprocess and CPU work before any rejection (security checklist: size limits on untrusted data).",
    "fix": "`z.array(MovePairSchema).default([]).max(N)` plus `.max(len)` on `from`/`to`, so an oversize list fails at `safeParse` on the same invalid-reply path; extend Test strategy #1 with the over-limit case."
  },
  {
    "severity": "minor",
    "location": "Chosen design — \"Validator\" rule 1: \"Shape: repo-relative POSIX paths, normalized, no absolute path, no `.` or `..` segments, `from !== to`\"",
    "problem": "Shape is checked but the character class is not: NUL, control characters, or a backslash survive rule 1 and reach `existsSync`, `git cat-file`, and `realpath`, where spawn/fs throw `ERR_INVALID_ARG_VALUE`. The design routes only a validator-returned issue to \"`invalid-reply: moves — <issue>`\", so a throw escapes the nudge/retry path and crashes finalize instead of failing the reply — the fail-open case the Goals forbid. Backslash is also platform-dependent: splitting on `/` alone makes `..\\..\\x` one legal segment on Windows, with rule 7's realpath as the only backstop.",
    "fix": "Tighten rule 1 to a single allow-pattern that rejects NUL, control characters, and `\\`, so no exotic string reaches a syscall, and wrap the `validateMoves` call in a catch that converts any throw into the same `invalid-reply: moves` branch; add both cases to Test strategy #2."
  },
  {
    "severity": "minor",
    "location": "Chosen design — \"Apply\": \"`moves.applyMoves(repoDir, moves)`, called from `finalizeStage`\" with only \"re-check `to` absent\"",
    "problem": "Rules 4–7 (tracked at `headBefore`, on-disk existence, containment) are not re-checked at apply time — `applyMoves` is not even passed `headBefore` — while `moves` is persisted into the checkpoint (A13; Rollout 2's \"in-flight and old checkpoints re-parse unchanged\"). A resume that re-enters finalize with an already-accepted reply therefore applies a list whose state-dependent rules were evaluated against a stale `headBefore` and stale disk state; the design reasons about a crash between apply and commit but not between reply acceptance and apply.",
    "fix": "Pass `headBefore` into `applyMoves` and re-run the state-dependent rules (4–7) as a single guard immediately before the first rename, making apply the one authoritative validation call site instead of parse-time only."
  },
  {
    "severity": "minor",
    "location": "Chosen design — \"Apply\": \"completed renames are rolled back in reverse — exact because pairs are disjoint\"",
    "problem": "The rollback itself can fail (`rename(2)` fails `EISDIR`/`EXDEV`/`EINVAL` per A8, or `EACCES`/`ENOSPC`), and the design specifies nothing for that case. The result is exactly the half-applied list the Goals promise to prevent, and the documented recovery launders it: \"the next attempt re-captures `headBefore`\" makes the damaged state the baseline, so a partially applied move set can be committed by a later stage.",
    "fix": "Emit a distinct `moves-rollback-failed` outcome that escalates to the operator instead of starting attempt 2 on a known-damaged worktree, and add a Test strategy #5 case where a rollback rename fails."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix — \"Node ≥ 22.12 (tools/orchestration/package.json:30-32) | yes\"",
    "problem": "The runtime floor predates published Node 22.x security releases, including the permission-model path-traversal and `fs.realpathSync` permission-check fixes (22.22.0, 22.23.4). No unfixed advisory applies to a dependency named here, so this is not a blocker, but rule 7's containment leans on realpath semantics and the Deferred hand-off to \"OS/sandbox-level protection\" should not assume Node's `--permission` model is sound. No minimum `git` version is pinned anywhere despite the design relying on `git cat-file`, `-M` rename detection, and config-sink behavior.",
    "fix": "State a patched minimum Node (e.g. 22.23.4) and a minimum `git` version in that row, and add one line to the sandbox Deferred noting that Node's permission model is not an acceptable substitute for harness or OS enforcement."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/005-runner-executed-file-moves.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/security-r1c1.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "Chosen design — \"Read-only git for implement/fix/test-update seats (item 5)\": \"An unwritable `.git/config` closes every config sink A6 names, `core.fsmonitor` — which fires on the index refresh these commands perform — included.\"", "problem": "re-raised theme, new instance (arises from the r1 closure): git's exec-capable config is not one file. Sources are repo-local `.git/config`, `$XDG_CONFIG_HOME/git/config`, `~/.gitconfig`, `/etc/gitconfig`, and `include.path` reachable from any of them. The control r1 makes load-bearing cannot reach them: OpenCode matches `edit` resources as whole-value wildcards against the normalized Location-relative path, so `.git`/`.git/**` denies nothing outside the worktree, and the harness default has no `external_directory` ask/deny (\"reads and edits outside the Location and worktree proceed without a prompt\"), while the design's Options 2 says the runner's git helper \"inherits ambient config\". A stage writing `core.fsmonitor` into `~/.gitconfig` is stopped by nothing in the changed text, is invisible to the sweep (no worktree path, never dirty, no blob at `headBefore` — the paragraph's own argument), and executes at runner privilege on the runner's next `git status`/`git add -A`, surviving the attempt boundary. The Deferred item states the gap without closing it: \"Hardening the runner's own git subprocess env (`GIT_CONFIG_NOSYSTEM`, pinned `GIT_CONFIG_GLOBAL`) — no follow-up yet; the `.git` write-deny closes the repo-local half.\"", "fix": "Put the runner-side half on the load-bearing side: have `git()` set `GIT_CONFIG_NOSYSTEM=1` and `GIT_CONFIG_GLOBAL` to an empty file, or pass `-c core.fsmonitor=false -c safe.bareRepository=explicit` on the runner's calls (the shape of the 2026 fixes for this class); or deny the global/system config paths in `coder.md`/`test-engineer.md` and add them to Test #13's fixture. Either way replace \"closes every config sink A6 names\" with \"closes the repo-local config source\", matching the Deferred clause it leans on."}, {"severity": "major", "location": "Chosen design — item 5 paragraph: \"`stagePathAllowed` gains, as its first line, `path === '.git' || path.startsWith('.git/') → false`\" (and Compatibility matrix \"git ≥ 2.16 … floor set by the global `--no-optional-locks`\")", "problem": "new: both named mechanisms are anchored at the worktree root, while the plant vector the design and ADR name is \"the worktree\" at large. `implement`/`fix` own everything but `tests/**` (A7), so a stage can write `<owned-dir>/.git/config` (or a bare `<owned-dir>.git/`). That path matches neither the whole-value patterns `.git`/`.git/**` nor `startsWith('.git/')`; it is in-ownership, so the sweep neither reverts nor deletes it (enforceOwnership only touches violations); and `commitStage`'s `git add -A` records it as a gitlink with only a warning (\"adding embedded git repository\"), giving the plant a committed marker that persists into later attempts. Every later superproject `git status`/`git diff` — including the runner's `dirtyFiles` — then consults that nested repository's config, where `core.fsmonitor` fires on the index refresh; git's only protection, `safe.bareRepository=explicit`, is opt-in and its default does not flip until Git 3.0. The stated floor removes even that: below 2.38 `safe.bareRepository` does not exist.", "fix": "Make the rule a segment rule instead of a prefix rule: `p.split('/').includes('.git')` plus rejecting any `*.git` name in `stagePathAllowed`, and `**/.git/**` / `**/*.git` in the two agent configs. Raise the floor to `git ≥ 2.38` so the runner can set `safe.bareRepository=explicit`, and add a planted `<owned-dir>/.git/config` carrying `core.fsmonitor` to Test #13's hostile fixture as part of item 5's ship gate."}, {"severity": "major", "location": "Chosen design — item 5 allowlist paragraph: \"the config-write deny above carries item 5 and argv vetting stays sandbox territory\"; Risks row 1: \"worktree landings land in visible commits\"", "problem": "new: the deny r1 makes load-bearing is scoped to the `edit` action's target path and cannot observe a write performed by an allowed shell command. A6 itself names `--output=<file>` on `git log`/`git diff` — a flag-only file write requiring no config — and the shipped allow patterns `git log*`/`git diff*` prefix-match any suffix, including `--output=.git/config` or an out-of-repo target; the harness warns that directory inference from shell command text is best-effort. With the argv refinements demoted to \"best-effort hygiene, not the control\", nothing in the changed text carries that class. Risks row 1's mitigation is false where the risk lands: a `.git/**` landing is, by the same paragraph's argument, never a dirty path and has no blob at `headBefore`, so it is neither swept nor visible in a commit.", "fix": "Split the control by class in the text: config sinks → the config-write deny; flag-only write/exec levers → the `--output`/`--ext-diff`/`--textconv` denials, stated as load-bearing for that class and expressed as OpenCode `policies` hard-denies (applied after allow rules and saved approvals) rather than order-dependent agent-config refinements. Pin them in Test #13 by running `git log --output=.git/config` and `git diff --output=<owned path>` in an attempt and asserting refusal, on the same ship-gate footing as the `.git/config` write test. Delete \"worktree landings land in visible commits\" or scope it to paths that can be dirty."}, {"severity": "nit", "location": "Chosen design — item 5 allowlist paragraph: \"follow the existing prefix-pattern style\"; A9: \"OpenCode permission matching is prefix-pattern with most-specific-wins\"", "problem": "new: the documented model is whole-value wildcards with last-match-wins rule combination, not most-specific-wins. The `.git` deny and the argv refinements work only because they are appended after the broad allow (as coder.md happens to order them), so the mechanism description the design relies on for both the edit deny and the refinements is wrong.", "fix": "Restate A9 as whole-value wildcards with last-match-wins, and state the ordering requirement — deny lines appended after the allows in `coder.md` and `test-engineer.md` — instead of relying on specificity."}, {"severity": "nit", "location": "Test strategy #4 — sweep interaction (r1 rewrite of the r0 list)", "problem": "new: the r1 rewrite dropped \"`stagePathAllowed` is false for `.git` and `.git/**` for every stage\" while demoting that rule to defense-in-depth, so the predicate's new first line has no direct assertion; #2 pins only the validator's use of it.", "fix": "Restore one direct predicate assertion for the first line of `stagePathAllowed` in #4."}]}
```
