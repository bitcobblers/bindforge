# Transcript: design-review:security r1c2

## Provenance
- stage: design-review:security (round r1c2)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:42:38.001Z
- ended: 2026-11-10T17:46:25.627Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 64ae876fb3689573e750e2d79a658f530bbd0579 89027e8abc18ce75659a7237f8544692866db24f` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design — \"Read-only git for implement/fix/test-update seats (item 5)\": \"An unwritable `.git/config` closes every config sink A6 names, `core.fsmonitor` — which fires on the index refresh these commands perform — included.\"",
    "problem": "re-raised theme, new instance (arises from the r1 closure): git's exec-capable config is not one file. Sources are repo-local `.git/config`, `$XDG_CONFIG_HOME/git/config`, `~/.gitconfig`, `/etc/gitconfig`, and `include.path` reachable from any of them. The control r1 makes load-bearing cannot reach them: OpenCode matches `edit` resources as whole-value wildcards against the normalized Location-relative path, so `.git`/`.git/**` denies nothing outside the worktree, and the harness default has no `external_directory` ask/deny (\"reads and edits outside the Location and worktree proceed without a prompt\"), while the design's Options 2 says the runner's git helper \"inherits ambient config\". A stage writing `core.fsmonitor` into `~/.gitconfig` is stopped by nothing in the changed text, is invisible to the sweep (no worktree path, never dirty, no blob at `headBefore` — the paragraph's own argument), and executes at runner privilege on the runner's next `git status`/`git add -A`, surviving the… [elided by the checkpoint cap]",
    "fix": "Put the runner-side half on the load-bearing side: have `git()` set `GIT_CONFIG_NOSYSTEM=1` and `GIT_CONFIG_GLOBAL` to an empty file, or pass `-c core.fsmonitor=false -c safe.bareRepository=explicit` on the runner's calls (the shape of the 2026 fixes for this class); or deny the global/system config paths in `coder.md`/`test-engineer.md` and add them to Test #13's fixture. Either way replace \"closes every config sink A6 names\" with \"closes the repo-local config source\", matching the Deferred clause it leans on."
  },
  {
    "severity": "major",
    "location": "Chosen design — item 5 paragraph: \"`stagePathAllowed` gains, as its first line, `path === '.git' || path.startsWith('.git/') → false`\" (and Compatibility matrix \"git ≥ 2.16 … floor set by the global `--no-optional-locks`\")",
    "problem": "new: both named mechanisms are anchored at the worktree root, while the plant vector the design and ADR name is \"the worktree\" at large. `implement`/`fix` own everything but `tests/**` (A7), so a stage can write `<owned-dir>/.git/config` (or a bare `<owned-dir>.git/`). That path matches neither the whole-value patterns `.git`/`.git/**` nor `startsWith('.git/')`; it is in-ownership, so the sweep neither reverts nor deletes it (enforceOwnership only touches violations); and `commitStage`'s `git add -A` records it as a gitlink with only a warning (\"adding embedded git repository\"), giving the plant a committed marker that persists into later attempts. Every later superproject `git status`/`git diff` — including the runner's `dirtyFiles` — then consults that nested repository's config, where `core.fsmonitor` fires on the index refresh; git's only protection, `safe.bareRepository=explicit`, is opt-in and its default does not flip until Git 3.0. The stated floor removes even that: below 2.38… [elided by the checkpoint cap]",
    "fix": "Make the rule a segment rule instead of a prefix rule: `p.split('/').includes('.git')` plus rejecting any `*.git` name in `stagePathAllowed`, and `**/.git/**` / `**/*.git` in the two agent configs. Raise the floor to `git ≥ 2.38` so the runner can set `safe.bareRepository=explicit`, and add a planted `<owned-dir>/.git/config` carrying `core.fsmonitor` to Test #13's hostile fixture as part of item 5's ship gate."
  },
  {
    "severity": "major",
    "location": "Chosen design — item 5 allowlist paragraph: \"the config-write deny above carries item 5 and argv vetting stays sandbox territory\"; Risks row 1: \"worktree landings land in visible commits\"",
    "problem": "new: the deny r1 makes load-bearing is scoped to the `edit` action's target path and cannot observe a write performed by an allowed shell command. A6 itself names `--output=<file>` on `git log`/`git diff` — a flag-only file write requiring no config — and the shipped allow patterns `git log*`/`git diff*` prefix-match any suffix, including `--output=.git/config` or an out-of-repo target; the harness warns that directory inference from shell command text is best-effort. With the argv refinements demoted to \"best-effort hygiene, not the control\", nothing in the changed text carries that class. Risks row 1's mitigation is false where the risk lands: a `.git/**` landing is, by the same paragraph's argument, never a dirty path and has no blob at `headBefore`, so it is neither swept nor visible in a commit.",
    "fix": "Split the control by class in the text: config sinks → the config-write deny; flag-only write/exec levers → the `--output`/`--ext-diff`/`--textconv` denials, stated as load-bearing for that class and expressed as OpenCode `policies` hard-denies (applied after allow rules and saved approvals) rather than order-dependent agent-config refinements. Pin them in Test #13 by running `git log --output=.git/config` and `git diff --output=<owned path>` in an attempt and asserting refusal, on the same ship-gate footing as the `.git/config` write test. Delete \"worktree landings land in visible commits\" or scope it to paths that can be dirty."
  },
  {
    "severity": "nit",
    "location": "Chosen design — item 5 allowlist paragraph: \"follow the existing prefix-pattern style\"; A9: \"OpenCode permission matching is prefix-pattern with most-specific-wins\"",
    "problem": "new: the documented model is whole-value wildcards with last-match-wins rule combination, not most-specific-wins. The `.git` deny and the argv refinements work only because they are appended after the broad allow (as coder.md happens to order them), so the mechanism description the design relies on for both the edit deny and the refinements is wrong.",
    "fix": "Restate A9 as whole-value wildcards with last-match-wins, and state the ordering requirement — deny lines appended after the allows in `coder.md` and `test-engineer.md` — instead of relying on specificity."
  },
  {
    "severity": "nit",
    "location": "Test strategy #4 — sweep interaction (r1 rewrite of the r0 list)",
    "problem": "new: the r1 rewrite dropped \"`stagePathAllowed` is false for `.git` and `.git/**` for every stage\" while demoting that rule to defense-in-depth, so the predicate's new first line has no direct assertion; #2 pins only the validator's use of it.",
    "fix": "Restore one direct predicate assertion for the first line of `stagePathAllowed` in #4."
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
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/security-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design — \"Read-only git for implement/fix/test-update seats (item 5)\": \"in the shape of its existing `transcripts` matcher, and a hit fails the attempt\"","problem":"new: the audit detects a `.git` plant but never contains it, and the shipped retry path re-exposes it. An audit hit fails the attempt; nothing removes the planted `.git/config` (the design's own A4 says the sweep can neither see nor undo `.git/**`). An `independence violation:` failure is not `invalid-reply`, so attempt 2 runs a fresh session on the same worktree (runner.ts:836-838, retryInSameSession 1121-1126), and attempt 2's audit scans only messages since its own `auditStart` (independence.ts `since`), so the attempt-1 write is never re-detected. If attempt 2 succeeds, finalize's `dirtyFiles` `git status` and `commitStage`'s `git add -A` are runner `git()` calls, and the pinned env (GIT_CONFIG_NOSYSTEM, GIT_CONFIG_GLOBAL, safe.bareRepository) does not stop git from reading the repo-local `.git/config` — the planted `core.fsmonitor` fires on the index refresh at runner privilege. If both attempts fail, the plant persists in the operator's checkout and executes on every later git command. Chain: plant in attempt 1 → attempt 1 fails audit → clean attempt 2 → the commit executes the plant at runner privilege; #13 as written passes this chain because it asserts only that the attempt fails.","fix":"Name the containment step beside \"fails the attempt\": a `.git`-matcher hit escalates the stage to the operator immediately — no attempt 2, no finalize until the plant is cleared — and extend #13's ship gate to pin containment: the hostile fixture is an fsmonitor that touches a marker file, and after the audit-failed attempt the runner's next git call must not create the marker."},{"severity":"major","location":"Chosen design — item 5 paragraph: \"agent-config permissions are only advisory: a model can route around them (gates.ts:256-258, independence.ts)\"","problem":"new: the design demotes the config denies to advisory using the repo's own rationale that the bypass route is subagents (gates.ts:256-258: \"a model can route around them via subagents\"), but the audit it promotes to carrier sees only the attempt's own session — \"the completed tool calls and shell commands in its own message list\" (independence.ts:10-11; the runner passes one session's `lastSeen`, runner.ts:757, 839). A write completed inside a subagent's child session is not in that list; the base permission policy allows `*` and neither agent config denies `subagent`. The exact bypass the design cites to justify the audit therefore slips past both the advisory deny and the carrier, and the surviving plant then executes per the first finding.","fix":"State the audit's session scope: either audit subagent sessions spawned by the attempt, or deny `subagent` launch for the two move-emitting seats (the doc's own note that a `policies` entry hard-denies after rules and saved approvals makes that enforceable), and pin the chosen route in #13 — a subagent write to `src/.git/config` must fail."},{"severity":"minor","location":"Risks row 1: \"The audit's case-folded `.git` matcher fails the attempt for the write however it was spelled\"","problem":"new: overclaimed — the matcher is a `.git` whole-segment match, so an allowed command writing a git config file outside the worktree (`git log --output=~/.gitconfig`, an XDG config path) matches nothing: the segment is `.gitconfig`, the `**/.git/**`/`**/*.git` edit denies are Location-relative whole-value patterns with no `external_directory` ask/deny in the harness default, and the sweep cannot see outside the worktree. Impact is contained — `git()`'s pinned env keeps it from executing at runner privilege and agent-privilege exec is the Deferred sandbox line — but a host-persistent config plant escapes every control this change ships while the mitigation sentence asserts it is caught.","fix":"Scope the claim: \"fails the attempt for any `.git`-segment write, however spelled; out-of-worktree config landings are contained by `git()`'s pinned env and left to the Deferred sandbox line\" — or extend the matcher with a `.gitconfig` segment case and pin it in #13."}]}
```
