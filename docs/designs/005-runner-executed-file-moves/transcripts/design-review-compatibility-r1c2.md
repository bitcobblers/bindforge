# Transcript: design-review:compatibility r1c2

## Provenance
- stage: design-review:compatibility (round r1c2)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:38:40.323Z
- ended: 2026-11-10T17:42:37.954Z
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
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "Chosen design / Validator rule 2 + Read-only git — \"no endpoint `.git`-denied (see Read-only git — the same `stagePathAllowed` rule)\" and \"`stagePathAllowed` gains, as its first line, `path === '.git' || path.startsWith('.git/') → false` (gates.ts:260)\"",
    "problem": "The `.git` guard is a case-sensitive string test while the row that makes it load-bearing is now a supported row (macOS `yes (unverified)`). On a case-insensitive checkout `to = '.GIT/hooks/pre-commit'` clears rule 1 (clean POSIX shape), rule 2 (`.GIT` ≠ `.git` as a string), rule 3 (`stagePathAllowed` is true for implement/fix; the new first line does not match), rule 5 (`existsSync` false for an absent hook, no blob at headBefore, and parent `.GIT/hooks` resolves to the real `.git/hooks`), and rules 6–7. `applyMoves` then renames a worktree file into the real git directory — invisible to the sweep (A4: `.git/**` is never a dirty path) and executed by the runner's own next `git commit`. The r1 revision sharpens this: item 5's enforcement moved to the harness-side `edit` deny, which constrains agent writes only and cannot constrain a runner-side rename, so rule 2 is the sole control on this path; and A10's new claim \"nothing else in the design depends on the checkout being case-sensitiv… [elided by the checkpoint cap]",
    "fix": "Make rule 2's test and the `stagePathAllowed` first line case-folded — `path.toLowerCase() === '.git' || path.toLowerCase().startsWith('.git/')` — replacing the exact-case comparison; it cannot reject a legitimate name, since a segment equal to `.git` ignoring case cannot coexist with `.git`. Add `.GIT/hooks/pre-commit` to Test strategy #2's rejection list, pinned on Linux."
  },
  {
    "severity": "minor",
    "location": "Chosen design / Validator rule 6 — \"all `from`s distinct and all `to`s distinct compared case-folded, no `to` equals any `from`, and no listed path is a path-prefix of another — which rejects … case-only collisions\"",
    "problem": "The case-folded qualifier grammatically binds only the two distinctness clauses; the `to`-vs-`from` and path-prefix clauses remain string-exact. A single case-only pair (`README.md` → `readme.md`) is rejected nowhere on the reference platform: rule 5's `existsSync` guard — the rule it calls \"the case-insensitive-checkout guard, A10\" — is inert on case-sensitive ext4, rule 1's `from !== to` is exact, and `applyMoves` lands it as an `R` entry. That makes the Risks mitigation \"Rule 5's disk check rejects case-only endpoints and rule 6 rejects case-folded collisions on every platform\" false for the reference platform, sits under the Deferred entry that presumes case-only renames are currently rejected, and leaves \"apply order is irrelevant\" unproven for a list with `A→B` and `a→C`. Test strategy #2's \"duplicate endpoints including case-only collisions\" pins duplicates within a list, not the lone case-only pair. (re-raised: my r1 rule-6 finding, partially closed)",
    "fix": "Scope it in one clause — \"every comparison in this rule is case-folded\" — replacing the clause-attached qualifier, and add one Test strategy #2 case: a lone `README.md` → `readme.md` pair rejected on Linux."
  },
  {
    "severity": "nit",
    "location": "Test strategy #5 / Compatibility matrix \"Repository spanning mount points | partial\" — \"a pair whose `to` parent is on another mount (tmpfs under the worktree) fails `EXDEV`\"",
    "problem": "The `partial` row now names an observation, but creating a tmpfs requires root/`CAP_SYS_ADMIN`, which the vitest tmpRepo suite cannot assume; if CI cannot mount, the row keeps an evidence-less status under a new name. (new)",
    "fix": "Name the runner of that case the way #12 does for macOS (\"operator-run, mount required\"), or gate it on mount capability so it skips visibly instead of disappearing."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/compatibility-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design / Read-only git — \"`git()` (runner.ts:106-112) stops reading ambient config: `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL` at an empty file, `-c safe.bareRepository=explicit`\"","problem":"new: this removes the only git-identity source most operator environments have and nothing replaces it. `commitStage` (runner.ts:288-309) authors every stage commit with a bare `git(repoDir, ['commit','-m',message])` — no `-c user.*`, no author/committer env — and today resolves identity from the ambient global/system config the new env suppresses; repo-local `.git/config` is still read (the paragraph itself scopes the agent denies as \"advisory, scoped to the repo-local config source\"), and no repo-local identity is pinned anywhere. On the reference platform with the usual `~/.gitconfig` identity, every stage commit fails `unable to auto-detect email address`: item 5's control breaks the runner's committer path. The same applies to the suite — every tmpRepo fixture pins identity only on its own setup calls (`execFileSync('git',['-c','user.name=f','-c','user.email=f@f',...])`, stage-lifecycle.test.ts:94 and the identical helper in reply-contract/funnel/parallel-*), so #8–#11's runner-side commits lose identity too, and #13's new fixture (\"the runner's git calls ignore a hostile `~/.gitconfig`\") pins exactly that behavior. The concern is not handed off: this diff deletes the former Deferred bullet on hardening the runner's git env, and no matrix or Risks row names the operator git-config dependency.","fix":"In the same sentence, pin identity on the authoring calls — `git()`'s env gains `GIT_AUTHOR_NAME`/`GIT_AUTHOR_EMAIL`/`GIT_COMMITTER_NAME`/`GIT_COMMITTER_EMAIL` (or the commit calls pass `-c user.name=… -c user.email=…`), replacing what the suppressed global config supplied — and extend #13 with the positive half: a stage commit succeeds in a tmpRepo with no global or system config present."},{"severity":"minor","location":"Compatibility matrix (macOS row) + Test strategy #12 — \"Test strategy #12 runs one move round trip and one case-only move there and records the observed rejection as the A10 evidence\"","problem":"new (arises from the r2 rule-6 closure): A10 is the claim that the on-disk existence check (rule 5's `existsSync`) rejects a case-only endpoint on a case-insensitive checkout. Rule 6 now states \"every comparison here is case-folded on every platform … rejecting … case-only moves (`README.md` → `readme.md`)\", so a case-only pair is rejected before rule 5's disk check is reached, identically on Linux where #2 already pins it. The macOS observation can never discriminate `core.ignoreCase`, leaving the row's `yes (unverified)` and A10's `unverified` permanently unverifiable by the test named to verify them.","fix":"Change #12's second case to a pair rule 6 cannot see: `README.md` tracked, reply moves an unrelated `notes.md` → `readme.md`. On macOS rule 5's `existsSync` rejects it; on Linux the same pair is accepted and lands as a case-variant sibling — restate the macOS row's note as that contrast, which is the actual A10 evidence."},{"severity":"minor","location":"Chosen design / Read-only git — \"`stagePathAllowed` (gates.ts:260) returns `false` as its first line for any path with a case-folded `.git` segment or a segment ending in `.git`\"","problem":"new: the second half is a suffix rule on the predicate the sweep applies to every dirty path for every stage (A4), so it is an ownership change beyond moves: any path with a segment ending in `.git` (`fixtures/upstream.git/**`, a file `notes.git`) becomes permanently unowned, an agent's edit there is silently reverted as if it were an ownership violation, and a validator endpoint there fails as a reply error on every platform. Nothing in the paragraph needs the suffix — both pinned cases (`.GIT/hooks/pre-commit`, `src/.git/config`) and the planted `<owned-dir>/.git/` gitlink have a segment equal to `.git` case-folded, and the bare-repo-named case the suffix would catch is the one the same paragraph assigns to `safe.bareRepository=explicit`. The config mirror `**/*.git` carries the same over-reach.","fix":"State the first line as case-folded segment equality — `false` for any path with a segment equal to `.git` ignoring case — and drop \"or a segment ending in `.git`\" together with the `**/*.git` deny; if the suffix is kept, name the plant that needs it and add a #4 case pinning that an edit to `<owned>/upstream.git` is a deliberate sweep revert."}]}
```
