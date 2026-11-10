# Build notes: 004 — mirror the src/ hierarchy in the orchestration test tree

Stage: implement (round 0), working from `02-proposal.md`.

## What landed here

Chosen design ¶4 (Decision, bullets 4–5) — the convention surfaces:

- `.opencode/skills/repo-practices/SKILL.md`: the Commands line now reads
  `tests/**/*.test.ts` (was `tests/*.test.ts`), and the naming rule is
  stated once in the existing `## Tests` section — a directory under
  `tests/` mirrors a subsystem directory under `src/`; a module-named file
  (`state-store.test.ts`) is that module's unit test, cross-module behavior
  suites keep descriptive names (`funnel.test.ts`).
- `.opencode/skills/test-strategy/SKILL.md`: rule 5's glob only,
  `tests/*.test.ts` → `tests/**/*.test.ts`. The rule itself is not copied
  into this file, per the r1c4 complexity finding ("state the rule once…
  amend only the glob test-strategy:27 already restates").

That is the whole of the design that this stage is allowed to execute. The
two `describe('test-tree convention')` assertions in
`tests/pipeline-prompts.test.ts` (Test strategy #6) are satisfied by this
change: both skill files carry the recursive glob, and no file under
`.opencode/skills/` or `tools/orchestration/prompts/` retains the flat
`tests/*.test.ts` statement (checked by grep over both trees).

## Deviation 1 (blocking): the move itself is not implement-stage work

Rollout 2's payload — the 23 `git mv`s (¶1) and the rewrite of every
depth-sensitive `../` reference (¶2) — lives entirely in paths matched by
`TEST_PATH_RE` (`tools/orchestration/src/pipeline/gates.ts:253`).

- `stagePathAllowed('implement', …)` and `stagePathAllowed('fix', …)`
  return false for every such path (`gates.ts:260-266`). The suite pins
  exactly that for the post-move path at
  `tools/orchestration/tests/pipeline-gates.test.ts:301-309`.
- The runner enforces that matrix deterministically after the stage: it
  restores every out-of-ownership path and deletes paths that did not exist
  at `headBefore` (`runner.ts:204-265`), and when the reverted paths are
  test paths it sets `state.testsNeedUpdate = true` (`runner.ts:1029-1034`).
  So any move performed in this stage is undone before the stage commits.
- No agent has git access ("the runner is the only committer; models never
  touch git", `runner.ts:104`); this stage's shell permission is deny-all
  except `npm run check*` and `npm run format*`
  (`.opencode/agents/code/test-engineer`'s counterpart,
  `.opencode/agents/code/coder.md:26-34`). There is therefore no `git mv`,
  no `git rm`, and no delete primitive in the stage's tool set. A
  copy-to-nested layout would leave both forms on disk — precisely the
  half-mirrored tree Risks row 1 names — and
  `tests/pipeline-gates.test.ts:374-381` reads `git ls-files`, so it fails
  at 49 tracked test paths whether or not the copies are correct.

Consequence: the four assertions that pin the post-move tree
(`pipeline-gates.test.ts:329-390`, Test strategy #1/#2/#3) stay red after
this stage and cannot be closed by any implement/fix-stage edit. The reply
sets `testsNeedUpdate: true`, which routes `implement → test-update`
(`tools/orchestration/src/pipeline/states.ts:161-168`) — the stage that
owns `**/tests/**` (`.opencode/agents/code/test-engineer.md:9`), the only
stage whose edits to those 26 files survive ownership enforcement.

### Exact scope the test-update pass must land (¶1–¶2, unchanged from the proposal)

- 23 renames: `tests/pipeline-<name>.test.ts` → `tests/pipeline/<name>.test.ts`
  for the ADR's list (budget … states), verified against the directory: 26
  test files today, 23 flat `pipeline-*` plus the three root-tier files
  (`cli`, `opencode-runner`, `opencode-runner-timeout`).
- 62 `from '../src/…'` specifiers in the moved files → `from '../../src/…'`
  (A8: 65 across all 26 files; the 3 in `cli.test.ts` stay — it is
  root-tier, and its `vi.mock`/type-import `../src/` sites at
  `cli.test.ts:29-30` are not moved).
- Each of the 16 `new URL('<rel>', import.meta.url)` sites gains one `../`
  (verified by grep: 16 matching lines in 11 moved files). Sites:
  `pipeline-fix-trigger.test.ts:8` (`'../prompts'`, no trailing slash),
  `pipeline-prompts.test.ts:22`, `pipeline-budget.test.ts:85`,
  `pipeline-reply-contract.test.ts:83`, `pipeline-stage-lifecycle.test.ts:108`,
  `pipeline-funnel.test.ts:187`, `pipeline-proposal-size.test.ts:191`,
  `pipeline-parallel-integration.test.ts:191`,
  `pipeline-pass-finalize.test.ts:192`, `pipeline-parallel-fanout.test.ts:199`,
  `pipeline-parallel-escalation.test.ts:203` (all `'../prompts/'`), and the
  five `'../../../.opencode/skills/…'` reads at
  `pipeline-prompts.test.ts:214, 252, 261-262, 275, 298`.
- One comment line: `tests/pipeline-reply-contract.test.ts` →
  `tests/pipeline/reply-contract.test.ts` at
  `pipeline-parallel-escalation.test.ts:45` (A14).
- Untouched: the 4 inert `'../escape'` fixture strings
  (`pipeline-identity.test.ts:65,68,103`; `pipeline-state-store.test.ts:156`)
  and the other in-test fixture paths (Deferred).

## Deviation 2: the ADR-004 Decision bullet 2 amendment is not landed

Rollout 2 tasks one clause into `docs/adrs/004-mirror-src-hierarchy-in-tests.md`
bullet 2 (sanctioning the ¶2 path references and the comment line). The
implement agent's edit permission denies `docs/**`
(`.opencode/opencode.jsonc:9-11`, `.opencode/agents/code/coder.md:9-16`);
`docs/adrs/**` is runner-written. The runner's ownership matrix would allow
it (`stagePathAllowed('implement', 'docs/…')` is true), but the tool layer
will not. It needs the operator or the runner.

## Deviation 3: the design's "one commit" cannot hold in this harness

The proposal plans ¶1–¶4 plus the ADR clause as a single move commit
(Rollout 2). The runner commits once per stage (`runner.ts:287-309`), so
the ¶4 convention amendment landed here and the move lands in the
test-update stage. Same content, three commits, and the move commit's diff
still stays inside ¶2's allowed set.

## Gate state observed

`npm run check` (the only gate command this stage may run) fails before
vitest: `eslint .` passes, `prettier --check .` reports
`tests/pipeline-prompts.test.ts`. The fix is `npm run format`, which
rewrites a test file — out of this stage's ownership, so it is listed as a
test-update item rather than executed here; running it would also be a test
edit this stage is told never to make. The vitest phase therefore did not
execute in this stage and its totals are unverified (A17). By inspection
the only failing assertions are the layout ones listed above; the 23 flat
suites are untouched by this stage and still resolve `../src/…` from the
root tier.

## Fix round r0: gate reproduced, cause confirmed to be the unexecuted move

Re-ran `npm run check` in `tools/orchestration` (the only gate command this
stage may run; the shell is otherwise deny-all — `git status` itself was
rejected). Result, superseding the "Gate state observed" section above:

- `eslint .` green. `prettier --check .` green — the
  `tests/pipeline-prompts.test.ts` formatting failure recorded above was
  closed by the test-update-r0 pass (`test: 004-… test-update stage`).
- `vitest run`: 26 test files, 228 tests, **2 failed | 226 passed**. Both
  failures are the layout assertions in `tests/pipeline-gates.test.ts`
  `describe('test tree mirrors src/ hierarchy')` (Test strategy #1/#2/#3,
  lines 374–390): the `git ls-files` listing still shows 23 flat
  `pipeline-*.test.ts` names, so the flat-name check and the nested-path
  equality both fail.

Nothing in `src/`, config, or the skill files is red, and the proposal
mandates no change to any of them (¶3). The red gate's cause lives entirely
in test-owned paths — Deviation 1, unchanged — so this stage made no edit
outside this build note: every `tests/**` path it touched would be restored
or deleted by the ownership sweep (`runner.ts:204-265`), which also sets
`testsNeedUpdate` itself (`runner.ts:1029-1034`). The reply sets
`testsNeedUpdate: true`, routing `fix → test-update` (`states.ts:173-174`).

### Deviation 4: the renames themselves need a primitive no stage agent has

The test-update-r0 pass closed the format failure but did not land the
renames, and a repeat test-update pass cannot: neither stage agent has git
access or a delete primitive — coder shell is `npm run check*`/`format*`
(`coder.md:26-34`), test-engineer shell adds only `npm test*`
(`test-engineer.md:23-34`), and both agents' tool sets can write files but
never unlink them. The failing assertions read `git ls-files` — the index —
which moves only through a commit, and only the runner commits, staging
what a stage left dirty. A copy-to-nested pass would leave the
half-mirrored tree Risks row 1 forbids and fail the same listing at 49
tracked paths. So Rollout 2's `git mv` ×23 is, exactly as the proposal
words it, "land the move commit" — operator-level work (ADR Ordering: the
change is small enough to land directly); the harness cannot execute it
from inside a stage. The exact scope the test-update pass must land, if the
harness gains a rename path, is itemized in Deviation 1 above and stands
unchanged.

## Fix round r1: gate re-reproduced; state identical, no stage-side fix exists

Re-ran `npm run check` in `tools/orchestration` (again the only shell this
stage has: every non-`npm run check*`/`format*` command, `git status` and
`ls` included, is denied — `coder.md:26-34`). Result matches the r0
reproduction exactly:

- `eslint .` green; `prettier --check .` green.
- `vitest run`: 26 test files, 228 tests, **2 failed | 226 passed**. Both
  failures are `tests/pipeline-gates.test.ts`
  `describe('test tree mirrors src/ hierarchy')` — the flat-name check and
  the nested-path listing equality, reading `git ls-files` over the still
  flat tree (23 `pipeline-*.test.ts` tracked at the `tests/` root).

No formatting failure exists, so `npm run format` has nothing to do and was
not run. Nothing in `src/`, config, or the skill files is red, and ¶3
mandates no change to them. The red gate's cause remains the unexecuted
Rollout 2 move — Deviations 1 and 4, unchanged: the payload lives entirely
in `**/tests/**` paths this stage neither owns nor can rename (no delete
or git primitive; the ownership sweep at `runner.ts:204-265` would revert
any such edit anyway). The two failing assertions are Test strategy
#1/#2/#3's correct expectations for the post-move tree, not miswritten
tests; they go green when the move lands in the test-owned paths.

This stage therefore edited nothing outside this build note and replies
`testsNeedUpdate: true`, routing `fix → test-update` (`states.ts:173-174`).
If the test-update pass again cannot land the renames, the blocker is the
harness's missing rename primitive (Deviation 4), i.e. operator-level
landing of Rollout 2 — repeating this fix round cannot change that.

## Test-update round r1: the suite already pins the approved design

Prompt named five round-0 code-review reports
(`reviews/code/{correctness,compatibility,security,performance,testing}-r0.md`).
None exist: `docs/designs/004-mirror-src-hierarchy-in-tests/reviews/code/` is
not a directory, and `.pipeline.json` carries no `code-review` stage entry and
no `seats.code` verdicts. So no accepted code-review finding exists to
consume, and `02-proposal.md` is byte-unchanged since `proposal-final` r0 —
i.e. since before `test-authoring` r0, which wrote its assertions against the
current Test strategy section. Both conditions for a test edit (rule 6: the
proposal changed, or an accepted finding changed) are false, so this pass
changed no test: no assertion, test name, fixture input, or file was added,
weakened, or deleted (rule 3).

Item-by-item re-verification of Test strategy #1–#6 against the suite:

| Item | Pinned where | State |
|---|---|---|
| #1 | `pipeline-gates.test.ts:337-387` — all 26 post-move paths are named and compared against the tracked listing | red (listing still flat); its diff-review half ("moved-file diff stays inside ¶2's allowed set") is a commit-diff review at Rollout 3, not a suite assertion — adding one now would move the 228-test baseline Risk 4's count comparison uses |
| #2 | same block: zero `pipeline-*.test.ts` in `git ls-files` (fail-closed listing, per the accepted r1 security minor + correctness major), 23 nested + 3 root | red |
| #3 | `pipeline-gates.test.ts:392-396` — substring filter over the same listing | red |
| #4 | `pipeline-gates.test.ts:295-318` — nested path allowed for `test-authoring`/`test-update`, denied to `implement`/`fix` | green |
| #5 | the gate itself (`npm run check`) | red on those two assertions |
| #6 | `pipeline-prompts.test.ts:306-339` — both skills state `tests/**/*.test.ts`; no flat form left in `.opencode/skills/` or `tools/orchestration/prompts/` | green |

Gate state, re-run this stage: `npm test` in `tools/orchestration` → 26 test
files / 228 tests, **2 failed | 226 passed**, both failures in
`describe('test tree mirrors src/ hierarchy')` — identical to fix r0/r1.
Counts are unchanged from the Rollout-1 baseline, as Rollout 1 requires.

Primitive re-confirmed from inside this stage: `npm test` is permitted and
runs; every `git` invocation (`git status`, `git ls-files`) is rejected by the
stage's shell permissions (`.opencode/agents/code/test-engineer.md:23-34`),
and the tool set has no unlink/rename. A nested copy of the 23 files would
leave 49 tracked test paths, the half-mirrored tree Risks row 1 forbids, and
#2's listing would still fail — so no test-owned edit can close #1–#3.

Operator action (Rollout 2, unchanged): `git mv` ×23 `tests/pipeline-<name>.test.ts`
→ `tests/pipeline/<name>.test.ts`, then ¶2's rewrites exactly as itemized in
Deviation 1 (62 `from '../src/…'` → `../../src/…`; one `../` added at each of
the 16 `new URL('<rel>', import.meta.url)` sites, including the no-trailing-slash
site at `pipeline-fix-trigger.test.ts:8`; the comment at
`pipeline-parallel-escalation.test.ts:45` →
`tests/pipeline/reply-contract.test.ts`), plus the one-clause amendment to
ADR-004 Decision bullet 2, which no stage may write (`docs/adrs/**` is
runner-written — Deviation 2). `npm run check` then closes items #1–#3 and #5.

## Fix round r2: gate re-reproduced byte-identically; no stage-side fix exists

Re-ran `npm run check` in `tools/orchestration` (again the only shell this
stage has; every other command, `git status` included, is denied):

- `eslint .` green; `prettier --check .` green ("All matched files use
  Prettier code style"), so `npm run format` has nothing to do and was not
  run.
- `vitest run`: 26 test files, 228 tests, **2 failed | 226 passed** —
  byte-identical to fix r0/r1 and to the test-update r1 counts (the
  Rollout-1 baseline). Both failures remain
  `tests/pipeline-gates.test.ts` `describe('test tree mirrors src/
  hierarchy')`: the flat-name check receives the 23 tracked
  `tools/orchestration/tests/pipeline-*.test.ts` paths where it expects
  none, and the nested-path listing equality receives the flat paths where
  it expects the 23 `tests/pipeline/<name>.test.ts` paths.

Re-verified this stage's ownership surface is not red and ¶3 mandates no
change to any of it: the two ¶4 convention surfaces are landed and hold
(recursive `tests/**/*.test.ts` at `.opencode/skills/repo-practices/SKILL.md:21`
with the naming rule stated once in its `## Tests` section, and the glob-only
amendment at `.opencode/skills/test-strategy/SKILL.md:27`), which Test
strategy #6's `pipeline-prompts.test.ts` suite confirms green.

Conclusion unchanged from Deviations 1 and 4: the red gate's cause lives
entirely in `**/tests/**` paths — the unexecuted Rollout 2 move. This fix
stage neither owns those paths (denied by `stagePathAllowed`, and the
runner's ownership sweep would revert or delete any such edit) nor has a
rename/delete/git primitive to land it, and nested copies would create the
49-path half-mirrored tree Risks row 1 forbids that the same listing fails
closed on. The two failing assertions are Test strategy #1/#2/#3's correct
expectations for the post-move tree, not miswritten tests; nothing in them
needs changing. This stage therefore edited nothing outside this build note
and replies `testsNeedUpdate: true`, routing `fix → test-update`
(`states.ts:173-174`). As recorded in Deviation 4 and the test-update r1
pass, if that pass again cannot land the renames, closing the gate requires
operator-level execution of Rollout 2; repeating fix or test-update rounds
cannot change that.

## Test-update round r2: re-verified, no test change is licensed

The prompt again named five code-review reports
(`reviews/code/{correctness,compatibility,security,performance,testing}-r0.md`)
as the source of round-1 changes. Re-checked from inside this stage, none
exists: `docs/designs/004-mirror-src-hierarchy-in-tests/reviews/code/` is not a
directory anywhere in the worktree or the other worktrees, `.pipeline.json`
carries no `code-review` entry in `stages`, and `seats` holds only the `design`
seat set — no code-seat verdict exists to accept a finding from. The other
condition for a test edit is also false: `02-proposal.md` still carries the
revision stamp `1d1e68a6…` that all five design seats approved at round 1 and
that editorial round 2 passed with no blocking findings, and its last write
(`proposal-final` r0, then the editorial pass) predates `test-authoring` r0,
which wrote its assertions against that same Test strategy section. Fix rounds
r0–r2 edited nothing outside this file. So both rule-6 triggers are absent and
this pass changed no test: no assertion, test name, fixture input, or file was
added, changed, weakened, or deleted.

Item-by-item re-verification against the current tree:

| Item | Pinned where | State |
|---|---|---|
| #1 | `pipeline-gates.test.ts:337-387` — all 26 post-move paths named against the tracked listing; its diff-review half belongs to the Rollout 3 commit review | red (listing still flat) |
| #2 | same block: `git ls-files` listing check (zero `pipeline-*.test.ts`), 23 nested + 3 root | red |
| #3 | `pipeline-gates.test.ts:392-396` — `tests/pipeline` substring selection | red |
| #4 | `pipeline-gates.test.ts:295-318` — nested path allowed to `test-authoring`/`test-update`, denied to `implement`/`fix` | green |
| #5 | the gate itself (`npm run check`) | red on those two assertions |
| #6 | `pipeline-prompts.test.ts:306-339` — recursive glob in both skills, no flat form left in `.opencode/skills/` or `tools/orchestration/prompts/` (repo-wide grep for `tests/*.test.ts`: zero hits) | green |
| #7 | platform coverage: Linux executes #1–#5 | unchanged |

The A8/A1 pre-move enumeration also still holds exactly, so no earlier pass
half-applied the move: 26 test files (23 flat `pipeline-*` plus the three
root-tier files), 65 `from '../src/…'` lines (62 of them in moved files), the 16
`new URL('<rel>', import.meta.url)` depth sites (15 `new URL('../…` lines plus
the `template.md` argument at `pipeline-prompts.test.ts:262`), the one comment
at `pipeline-parallel-escalation.test.ts:45`, and the 4 inert `'../escape'`
fixture strings untouched. Outside those five `.opencode/skills` reads, no `../`
in the tree sits at post-move depth.

Gate state could not be re-executed in this pass: unlike the prior stages, this
one's shell tool is denied entirely — every invocation, `npm test` and
`npm run check` included, is rejected — so no vitest totals are measured here.
The last executed state on this branch is fix round r2's: 26 test files / 228
tests, 2 failed | 226 passed, both failures in
`describe('test tree mirrors src/ hierarchy')` — the Rollout-1 baseline, which
this pass leaves unchanged because no test file changed. Both failures stay red
by design; only operator-level Rollout 2 (¶1's `git mv` ×23 plus ¶2's rewrites
as itemized in Deviation 1, and the ADR bullet-2 clause from Deviation 2) closes
them.

## Fix round r3: gate executed green-through-to-vitest; cause re-confirmed, unchanged

Unlike the prior stages, this one's shell permitted exactly the gate commands
(`npm run check` ran; every other invocation, `git log` included, was denied).
Executed `npm run check` in `tools/orchestration`:

- `eslint .` green; `prettier --check .` green ("All matched files use
  Prettier code style!"), so `npm run format` has nothing to do and was not
  run.
- `vitest run`: 26 test files, 228 tests, **2 failed | 226 passed** —
  identical to fix r0/r1/r2 (the Rollout-1 baseline). Both failures are
  `tests/pipeline-gates.test.ts` `describe('test tree mirrors src/
  hierarchy')`: the fail-closed flat-name check receives the 23 tracked
  `tools/orchestration/tests/pipeline-*.test.ts` paths where it expects
  `[]`, and the selection equality receives those flat paths where it
  expects the 23 `tools/orchestration/tests/pipeline/<name>.test.ts` paths.

Verified from the read side that the two failing assertions are Test strategy
#1/#2/#3's correct post-move expectations, not tests asserting the wrong
thing: `MOVED_NAMES` at `pipeline-gates.test.ts:337-361` matches the ¶1 list
of 23 names exactly, the root tier matches A1's three files, and the check is
the `git ls-files` listing the proposal prescribes (Test strategy #2's
fail-closed form). The only edit that turns them green is the Rollout 2 move
itself, which lives entirely in `**/tests/**` — denied to `fix` by
`stagePathAllowed` (pinned at `pipeline-gates.test.ts:303-304`), reverted by
the runner's ownership sweep, and beyond every stage's tool set for lack of a
rename/delete primitive (Deviation 4).

This stage's ownership surface re-verified green: ¶4's convention amendment
holds on both surfaces — recursive `tests/**/*.test.ts` at
`.opencode/skills/repo-practices/SKILL.md:21` and
`.opencode/skills/test-strategy/SKILL.md:27`, zero flat-form hits under
`.opencode/` — which `pipeline-prompts.test.ts` (#6) confirms passing.
¶3 mandates no change to `src/`, config, or gates; none was made.

Conclusion unchanged from Deviations 1, 2, and 4: the red gate's cause is the
unexecuted Rollout 2 move, entirely in test-owned paths; no implement/fix- or
test-update-stage edit can close it. This stage edited nothing outside this
build note and replies `testsNeedUpdate: true`, routing `fix → test-update`
(`states.ts:173-174`). Closing the gate requires the operator to land Rollout
2 (`git mv` ×23 plus ¶2's rewrites per Deviation 1, and the ADR bullet-2
clause per Deviation 2); repeating fix or test-update rounds cannot change
that.

## Test-update round r3: re-verified against the approved design; no test change is licensed

The prompt again named five round-0 code-review reports
(`reviews/code/{correctness,compatibility,security,performance,testing}-r0.md`)
as the source of the round-1 changes to consume. Re-checked from inside this
stage, none exists: `docs/designs/004-mirror-src-hierarchy-in-tests/reviews/`
holds only `design/` (the `*-r1c*.md` and `editorial-review-*-r2.md` reports), a
repo-wide glob for `**/reviews/code/**` matches only run 003's files, and
`docs/designs/004-…/.pipeline.json` records `codeRound: 0` with no `code-review`
entry anywhere in `results` and a `seats` map holding only the `design` seat set.
There is therefore no accepted code-review finding to consume. The other rule-6
trigger is also absent: the last write to `02-proposal.md` was `proposal-final` r0
(207 lines, revision stamp `1d1e68a6…`, the stamp all five design seats approved
at round 1 and that editorial round 2 passed with no blocking findings), which
predates `test-authoring` r0 — so the suite was written against the currently
approved Test strategy section. Fix rounds r0–r3 changed nothing outside this
file. With both triggers absent, this pass changed no test: no assertion, test
name, fixture input, or file was added, changed, weakened, or deleted (rule 3).

Item-by-item re-verification against the current tree:

| Item | Pinned where | State |
|---|---|---|
| #1 | `pipeline-gates.test.ts:337-387` — `MOVED_NAMES` matches ¶1's 23 names one-for-one, all 26 post-move paths compared against the tracked listing | red (listing still flat); its diff-review half ("moved-file diff stays inside ¶2's allowed set") is the Rollout 3 commit review, not a suite assertion — adding one now would move the 228-test baseline Risk 4's count comparison uses |
| #2 | same block: zero `pipeline-*.test.ts` in the `git ls-files` listing (the fail-closed listing form the accepted r1 correctness major + security minor asked for), 23 nested + 3 root | red |
| #3 | `pipeline-gates.test.ts:392-396` — `tests/pipeline` substring selection | red |
| #4 | `pipeline-gates.test.ts:295-318` — nested path allowed to `test-authoring`/`test-update`, denied to `implement`/`fix` | green |
| #5 | the gate itself (`npm run check`) | red on those two assertions only |
| #6 | `pipeline-prompts.test.ts:306-339` — recursive glob in both skills, no flat form left; re-grepped this pass: zero `tests/*.test.ts` hits under `.opencode/` or `tools/orchestration/prompts/` | green |
| #7 | platform coverage: Linux executes #1–#5 | unchanged |

The A1/A8 pre-move enumeration still holds exactly, so no earlier pass
half-applied the move: 26 test files (23 flat `pipeline-*` plus the three
root-tier files), 65 `from '../src/…'` lines (62 of them in the files ¶1 moves),
the 16 `new URL('<rel>', import.meta.url)` depth lines (15 `'../…'` lines plus the
`template.md` argument at `pipeline-prompts.test.ts:262`), the one comment at
`pipeline-parallel-escalation.test.ts:45`, and the 4 inert `'../escape'` fixture
strings. That comment and those 16 sites are ¶2 content of the move commit
(Rollout 2), not test-update work: rewriting them here would strand references to
paths that do not exist until the `git mv` lands, and would put content outside
the move commit that Test strategy #1 bounds to ¶2's allowed set.

Gate state, executed in this pass (`npm test` in `tools/orchestration`): 26 test
files / 228 tests, **2 failed | 226 passed** — identical to fix r0–r3 and to the
test-update r1/r2 counts, i.e. the Rollout-1 baseline. Both failures are in
`describe('test tree mirrors src/ hierarchy')`: the fail-closed flat-name check
receives the 23 tracked `tools/orchestration/tests/pipeline-*.test.ts` paths where
it expects `[]`, and the selection equality receives those flat paths where it
expects the 23 `tools/orchestration/tests/pipeline/<name>.test.ts` paths. They are
Test strategy #1/#2/#3's correct post-move expectations and stay red by design;
the only edit that turns them green is Rollout 2 itself, which needs the
operator's `git mv` ×23 (Deviation 4: no stage agent has git access or a
rename/delete primitive, and nested copies would leave the 49-path half-mirrored
tree Risks row 1 forbids).

## Fix round r4: gate re-executed, byte-identical failures; no implementation-side fix exists

Executed `npm run check` in `tools/orchestration` (this stage's shell again
denies everything else — `git status` rejected on arrival):

- `eslint .` green; `prettier --check .` green ("All matched files use
  Prettier code style!"), so there is no formatting/lint complaint for
  `npm run format` to fix and it was not run.
- `vitest run`: 26 test files, 228 tests, **2 failed | 226 passed** —
  identical to fix r0–r3 and test-update r1–r3 (the Rollout-1 baseline).
  The only failures are `tests/pipeline-gates.test.ts`
  `describe('test tree mirrors src/ hierarchy')`: line 382 receives the 23
  tracked `tools/orchestration/tests/pipeline-*.test.ts` paths where it
  expects `[]`, and line 394 receives those flat paths where it expects the
  23 `tools/orchestration/tests/pipeline/<name>.test.ts` paths.

Re-verified from the read side this round, not by citation: the disk tree is
still the 26 flat files (23 `pipeline-*` + the 3 root-tier files, A1); the
failing block (`pipeline-gates.test.ts:329-397`) compares the
`git ls-files tools/orchestration/tests/` index listing against `MOVED_NAMES`
(¶1's 23 names, one-for-one) plus the A1 root tier — Test strategy #2/#3's
prescribed fail-closed checks, correctly written for the post-move tree, not
tests asserting the wrong thing. `stagePathAllowed` (`gates.ts:253,260-266`)
denies every `tests/**` path to `fix`, and the runner's ownership sweep
reverts any such edit anyway, so this stage cannot land or fake the move: a
nested copy would be invisible to `git ls-files` (untracked) while vitest
discovered 49 files, failing the same assertions worse. Nothing in `src/`,
config, or the skill files is red, and ¶3 mandates no change to any of them.

The red gate's cause therefore remains exactly Deviations 1, 2, and 4: the
unexecuted Rollout 2 move, entirely in test-owned paths, needing a rename/git
primitive no stage agent has. This stage edited nothing outside this build
note and replies `testsNeedUpdate: true`, routing `fix → test-update`
(`states.ts:174`). As fix r3 recorded, if the test-update pass again cannot
land the renames, closing the gate requires operator-level execution of
Rollout 2 (`git mv` ×23 plus ¶2's rewrites per Deviation 1, and the ADR
bullet-2 clause per Deviation 2); further fix or test-update rounds cannot
change that.

## Test-update round r4: re-verified against the approved design; no test change is licensed

The prompt again named five round-0 code-review reports
(`reviews/code/{correctness,compatibility,security,performance,testing}-r0.md`)
as the source of the round-1 changes to consume. Re-checked from inside this
stage, none exists: `docs/designs/004-mirror-src-hierarchy-in-tests/reviews/`
contains only `design/`, a glob for `**/reviews/code/**` across the repo matches
only run 003's reports, and `.pipeline.json` records `codeRound: 0` with no
`code-review` entry anywhere in `results` and a `seats` map holding only the
`design` seat set. So no accepted code-review finding exists. The other rule-6
trigger is also absent: `02-proposal.md` on disk is still the 207-line
`proposal-final` r0 write carrying design stamp `1d1e68a6…` (all five design
seats approved it at round 1; editorial round 2 approved with no blocking
findings), and that write predates `test-authoring` r0 — the suite was written
against the currently approved Test strategy section. Fix rounds r0–r4 changed
nothing outside this file. With both triggers absent, this pass changed no test:
no assertion, test name, fixture input or file was added, changed, weakened, or
deleted (rule 3).

Re-verified from the read side this round, not by citation:

| Item | Pinned where | State |
|---|---|---|
| #1 | `pipeline-gates.test.ts:337-387` — `MOVED_NAMES` matches ¶1's 23 names one-for-one, all 26 post-move paths compared against the tracked listing; the comment at :330-335 names the Rollout-1 baseline once | red (listing still flat); its diff-review half is the Rollout 3 commit review, not a suite assertion |
| #2 | same block — `trackedTestFiles()` (:368-375) is the `git ls-files` listing the accepted r1 correctness major + security minor asked for, then `:382` fails closed on any `pipeline-*.test.ts` name, `:383` the 23 nested + 3 root equality, `:384-386` the 23 count | red |
| #3 | `pipeline-gates.test.ts:392-396` — `tests/pipeline` substring selection over the same listing, root tier excluded | red |
| #4 | `pipeline-gates.test.ts:295-318` — the real post-move path `tools/orchestration/tests/pipeline/gates.test.ts` allowed to `test-authoring`/`test-update`, denied to `implement`/`fix` | green |
| #5 | the gate itself (`npm run check`) | red on those two assertions only |
| #6 | `pipeline-prompts.test.ts:306-339` — re-grepped this pass: `tests/**/*.test.ts` at `.opencode/skills/repo-practices/SKILL.md:21` and `.opencode/skills/test-strategy/SKILL.md:27`, zero flat-form statements under `.opencode/skills/`, and `tools/orchestration/prompts/` stays directory-generic (its only `tests/` mention is `test-authoring.md:3`) | green |
| #7 | platform coverage: Linux executes #1–#5 | unchanged |

The A1/A8 pre-move enumeration still holds exactly, so no earlier pass
half-applied the move: 26 test files (23 flat `pipeline-*` plus `cli`,
`opencode-runner`, `opencode-runner-timeout`), 65 `from '../src/…'` lines, the 16
depth-sensitive `new URL('<rel>', import.meta.url)` lines (15 single-line sites
plus the multi-line `template.md` argument at `pipeline-prompts.test.ts:261-262`),
the one comment at `pipeline-parallel-escalation.test.ts:45` naming
`tests/pipeline-reply-contract.test.ts`, and the 4 inert `'../escape'` fixture
strings (`pipeline-identity.test.ts:65,68,103`; `pipeline-state-store.test.ts:156`).
Those ¶1/¶2 edits belong to the move commit (Rollout 2): rewriting them here would
strand references to paths that do not exist until the `git mv` lands and would put
content outside the move commit that Test strategy #1 bounds to ¶2's allowed set.

Gate state could not be re-measured in this pass: the shell tool was denied outright
— `npm test` was rejected on arrival — so no vitest totals were produced here. The
last executed run on this branch is fix round r4's: 26 test files / 228 tests,
2 failed | 226 passed, both failures in `describe('test tree mirrors src/
hierarchy')` — the Rollout-1 baseline, which this pass leaves unchanged because no
test file changed. Those two failures are Test strategy #1/#2/#3's correct
post-move expectations and stay red by design (rule 3); closing them needs the
operator's Rollout 2 — `git mv` ×23 plus ¶2's rewrites itemized in Deviation 1 and
the ADR bullet-2 clause from Deviation 2 — since no stage agent has git access or a
rename/delete primitive (Deviation 4) and nested copies would leave the 49-path
half-mirrored tree Risks row 1 forbids.

## Test-update round r1 (post-move): the gate is green; no test change is licensed

Rollout 2 has landed as the operator's commit
`refactor: mirror src/ hierarchy in orchestration test tree`: the tree on disk is
now 23 files at `tests/pipeline/<name>.test.ts` (¶1's list, one-for-one) plus the
three root-tier files (`cli`, `opencode-runner`, `opencode-runner-timeout`), and
Deviation 2's ADR clause landed too — `docs/adrs/004-mirror-src-hierarchy-in-tests.md:42-44`
now reads "The only content changes permitted are depth-sensitive path references:
a … passes takes its new depth." The escalation recorded in `.pipeline.json` has
therefore lost its cause; nothing in this pass depends on it.

Both rule-6 triggers for a test edit are still absent, so this pass changed no
test — no assertion, test name, fixture input or file was added, changed, weakened
or deleted (rule 3):

- No accepted code-review finding exists to consume. The prompt again named five
  round-0 reports (`reviews/code/{correctness,compatibility,security,performance,testing}-r0.md`);
  re-checked from inside this stage: `docs/designs/004-…/reviews/` holds only
  `design/`, a repo-wide glob for `**/reviews/code/**` matches only run 003's
  reports, and neither this worktree nor `mighty-squid`, `swift-star`,
  `tidy-cactus` nor the `/home/mike/repos/bindforge` checkout (which has no 004
  design directory at all) carries run 004 code reviews. `.pipeline.json` still
  records `codeRound: 0`, no `code-review` entry in `results`, and a `seats` map
  holding only the design seats.
- `02-proposal.md` is unchanged since `proposal-final` r0 — the same 207-line
  write the five design seats approved at round 1 (stamp `1d1e68a6…`) and
  editorial round 2 passed — which predates `test-authoring` r0. The Test strategy
  section the suite was written against is the approved one, unchanged by the move.

Post-move re-verification, read side, against the ¶2 allowed set (Test strategy
#1's diff half, which the earlier rounds deferred to the commit review): grep over
`tests/pipeline/` finds exactly 82 `../` sites — 62 `from '../../src/…'` imports,
16 `new URL('<rel>', import.meta.url)` sites (10 `'../../prompts/'`, the
no-trailing-slash `'../../prompts'` at `fix-trigger.test.ts:8`, and the five
`'../../../../.opencode/skills/…'` reads at `prompts.test.ts:215, 255, 266, 280,
305`), and the 4 inert `'../escape'` fixtures (`identity.test.ts:65, 68, 103`;
`state-store.test.ts:156`) — i.e. 62 + 16 + 4 = A8's closed enumeration, no
depth-sensitive site left un-rewritten and nothing outside it touched. The one
comment (A14) reads `tests/pipeline/reply-contract.test.ts` at
`parallel-escalation.test.ts:45`. The only remaining `../src/` specifiers are the
three root-tier files' own (`cli.test.ts:4`, `opencode-runner.test.ts:7`,
`opencode-runner-timeout.test.ts:5`), which ¶1 does not move. Fixture strings that
still mimic the old naming (`independence.test.ts:145`) stay as written — Deferred.

Item by item, now that the tree matches the design:

| Item | Pinned where | State |
|---|---|---|
| #1 | `pipeline/gates.test.ts:337-387` — 26 tracked paths named one-for-one; diff half verified statically above | green |
| #2 | same block: `git ls-files` listing, zero `pipeline-*.test.ts`, 23 nested + 3 root | green |
| #3 | `pipeline/gates.test.ts:392-396`, and executed directly: `npm test -- tests/pipeline` selects 23 files / 190 tests, none of the root tier | green |
| #4 | `pipeline/gates.test.ts:295-318` — `tools/orchestration/tests/pipeline/gates.test.ts` allowed to `test-authoring`/`test-update`, denied to `implement`/`fix`; that path is now this file's real location | green |
| #5 | the gate itself (`npm run check`) | green (executed below) |
| #6 | `pipeline/prompts.test.ts:314-348` — recursive glob in both skills, no flat form left; re-grepped: `tests/**/*.test.ts` at `.opencode/skills/repo-practices/SKILL.md:21` and `.opencode/skills/test-strategy/SKILL.md:27`, zero flat-form statements under `.opencode/`, `tools/orchestration/prompts/` stays directory-generic (`test-authoring.md:3` only) | green |
| #7 | platform coverage: Linux executes #1–#5; macOS/Windows rest on A16 | unchanged |

Gate state, executed in this pass (the stage's shell allowed `npm test` and
`npm run check` only, as recorded for earlier stages): `npm run check` in
`tools/orchestration` → `eslint .` green, `prettier --check .` green ("All matched
files use Prettier code style!"), `vitest run` → **26 test files / 228 tests, all
228 passed**. Counts are exactly the Rollout-1 baseline, which is Risk 4's and
Rollout 3's acceptance evidence. `npm test -- tests/pipeline` → 23 files / 190
tests. The two assertions fix rounds r0–r4 recorded as red
(`describe('test tree mirrors src/ hierarchy')`) went green because Rollout 2
landed, not because anything in the suite was relaxed — the same assertions that
were red are green now.

Rollout 3's remaining verification is therefore satisfied locally: items #1–#6
hold and the reported file/test counts (26/228; 23/190 for the nested selection)
are the acceptance evidence. Only Rollout 2's push/merge and the historical
artifacts (Deferred) are left, and this stage writes neither.

## Fix round r5 (code-review round 1): the testing seat's three findings

Code round 1 has one dissent — the testing seat
(`reviews/code/testing-r1c0.md`, verdict recorded at `.pipeline.json:850-881`):
1 major, 2 minors, all three about `tools/orchestration/tests/pipeline/gates.test.ts`.
The other four seats approved (compatibility and security with no findings;
correctness with 1 minor + 1 nit; performance with 1 nit). Nothing in `src/`,
config, or the skill files is implicated by any of them, and ¶3 forbids touching
those anyway, so this stage changed no implementation file.

This stage's shell tool was denied outright (every invocation, `npm run check` and
`npm test` included), so no gate numbers are re-measured here; the last executed
run on the branch stays the post-move test-update pass's green 26 files / 228 tests
(section above). All three findings were verified from the read side.

### Finding 1 (major) — test-owned, cannot land in a fix stage; Deviation 5

`gates.test.ts:383` (sorted set-equality against the 26 post-move paths),
`:384-386` (`toHaveLength(23)`) and `:394` (set-equality again) pin Rollout 3's
acceptance *measurement* as a permanent invariant. The ADR's forward rule ("New
subsystems follow the same shape"; from the merge, test stages place new tests in
the mirrored layout) guarantees a further `tests/**/*.test.ts`, which would turn
these three assertions red on every later branch and force a `MOVED_NAMES` edit in
an unrelated suite. Confirmed on disk: all three are the only places the exact
inventory is asserted; the permanent fail-closed invariant is `:382`'s
`git ls-files` zero-flat-name check, which the proposal's Test strategy #2
prescribes and which the seat keeps.

The fix is inside `**/tests/**`: `stagePathAllowed('fix', …)` returns false for it
(`gates.ts:263-265`, pinned at `gates.test.ts:303-304`), the runner's ownership
sweep reverts it, and this stage's prompt forbids test edits — so the change is
routed to test-update with `testsNeedUpdate: true`, which the accepted finding
licenses ("Change tests only where the design or an accepted review finding
changed", `prompts/test-update.md:4`).

### Finding 2 (minor) — test-owned, and the subprocess option is unsafe here

`gates.test.ts:392-396` is named for vitest's selection but applies
`includes('tests/pipeline')` to the tracked listing, i.e. it pins A6's substring
rule rather than vitest's behaviour. The seat allows either spawning the real
command or renaming the test to say it simulates the A6 filter. Recommendation:
**take the rename branch.** Spawning a nested run is not free of hazard: this file
lives at `tests/pipeline/gates.test.ts`, so `vitest run tests/pipeline` launched
from it selects that file itself, which spawns another run — unbounded recursion
unless an env-var guard is added — plus a second reporter, a second module graph
and the vitest API port inside the suite's own determinism budget (checklist 4 of
that same review: the new tests "spawn only local git"). The real selection stays
execution evidence, as it already is: `npm test -- tests/pipeline` → 23 files /
190 tests, recorded above and in Rollout 3. Test strategy #3's wording now says so
explicitly (see the proposal amendment below), so the renamed test no longer
over-claims and the design no longer expects the suite to spawn vitest.

### Finding 3 (minor) — landed here: the run's rollout wording now matches the history

Amended in `02-proposal.md`, language only, no requirement changed:

- Non-goals bullet 1: replaces "leaving both counts unchanged" with what the
  Rollout-1 commit actually did — 26 files preserved, four new `it` blocks, 228
  tests, tree pins red by construction until the move commit.
- Rollout 1: the pre-move commit lands item 4's expects *and* items #1–#3's tree
  pins, is red-until-move by construction, and its 26-file / 228-test pair is the
  baseline Test strategy #1–2 and Risk 4 compare against.
- Test strategy #2/#3: the exact 26/228 and 23-selection figures are labelled as
  the move commit's acceptance evidence, not permanent invariants; #2 names the
  two lasting assertions (zero flat names, moved+root paths as a subset) and #3
  states that the suite simulates the A6 filter while the run's reported list is
  the proof.
- Revision log: one entry citing the accepted finding.

### Deviation 5: this fix stage edited the design-approved `02-proposal.md`

The finding's own fix tasks the proposal's "test and file counts unchanged"
clause with alignment, and the ownership matrix permits it
(`stagePathAllowed('fix', 'docs/…')` is true, `gates.ts:261`; the agent's edit
allow covers `docs/designs/**`), so the clause was amended rather than left for
the operator. Consequence to note for the operator: the `02-proposal.md` blob id
moves off the `1d1e68a6…` stamp the five design seats approved at design round 1 —
the text changes no design requirement (it records landed history), and design
review is closed, but the recorded design verdicts now describe an earlier blob.
The proposal's own "Editor notes:" line at the foot is untouched.

### Prescribed test-update edit for findings 1 and 2 (gates.test.ts:329-397)

Same two `it` blocks, same file count (26) and test count (228) — nothing added,
weakened or deleted; only exact-inventory comparisons become subset ones:

```
  // Baseline: Rollout 1's test-only commit (the move commit's parent) — 26 test
  // files, 228 tests, the four new it blocks included. The pins below are red
  // from that commit until the move commit lands and are satisfied by it; the
  // exact 26/228 and 23-selection counts are Rollout 3's acceptance evidence at
  // that commit, not invariants of this suite — new mirrored test files are
  // expected from the merge on. What stays permanent is #2's fail-closed
  // listing: zero flat pipeline-*.test.ts names, and the moved and root-tier
  // paths as a subset of what git tracks.
  it('fails closed on a leftover flat name and tracks every moved and root-tier test', () => {
    const files = trackedTestFiles();
    expect(files.filter((f) => /(^|\/)pipeline-[^/]*\.test\.ts$/.test(f))).toEqual([]);
    for (const path of [...nested, ...rootTier]) expect(files).toContain(path);
  });

  // proves: Test strategy #3's rule, simulated — the A6 path-substring filter
  // over the tracked listing. The real selection is execution evidence from
  // `npm test -- tests/pipeline` (23 files / 190 tests), recorded in Rollout 3.
  it('the A6 path-substring filter selects the moved tier and none of the root tier', () => {
    const selected = trackedTestFiles().filter((f) => f.includes('tests/pipeline'));
    for (const path of nested) expect(selected).toContain(path);
    for (const path of rootTier) expect(selected).not.toContain(path);
  });
```

`MOVED_NAMES`, `ROOT_TESTS`, `nested`, `rootTier` and `trackedTestFiles()` all
stay; the test *names* are local to this file (grep: no other reference), so the
renames break nothing. The performance seat's accepted nit may ride the same
edit — memoize the listing at describe scope instead of re-spawning `git
rev-parse` + `git ls-files` per assertion (and share one repo root in
`prompts.test.ts`).

### The other seats' accepted findings

- correctness minor (paste the vitest reporter tail verbatim at the merge commit):
  needs a gate run; this stage's shell is denied, so it stays operator/next-stage
  evidence. The counts it should show are already recorded above (26/228 green,
  23/190 nested).
- correctness nit (name the prettier re-wraps): landed here as this clause — the
  six over-100-column ¶2 rewrites (`proposal-size.test.ts:10-14` and the five
  `fileURLToPath` blocks in `prompts.test.ts`) are prettier's formatting of the
  allowed statements, so ¶2's allowed set reads as "those statements, in whatever
  shape prettier requires".
- testing FYI (whether both skill surfaces should carry the naming rule): left as
  is — the rule sits once in `repo-practices/SKILL.md`, the glob alone in
  `test-strategy/SKILL.md:27`, which is what ADR bullet 5 and the r1c4 complexity
  finding asked for; the seat itself calls the ambiguity the correctness seat's.

Net: one doc-side fix landed (finding 3), two test-side fixes prescribed and routed
(findings 1, 2) → `testsNeedUpdate: true`, routing `fix → test-update`
(`states.ts:173-174`), the stage that owns `**/tests/**`.

## Test-update round r1 (post-code-review): prescribed findings 1–2 landed; code round 1 closed

The routed test-update pass landed the exact edit prescribed above, as commit
`test(orchestration): keep tree-mirror pins as subset invariants, drop
transient counts`. Verified from the read side at the resulting HEAD
(`tests/pipeline/gates.test.ts:329-402`):

- Finding 1: the exact-inventory pins are gone. What remains is the permanent
  fail-closed form — zero `pipeline-*.test.ts` names in `git ls-files`
  (:387) plus `toContain` subset checks over the 23 nested and 3 root-tier
  paths (:388) — with the baseline comment naming 26/228 as Rollout 3's
  acceptance measurement, deliberately not pinned (:330-337).
- Finding 2: the #3 check states in its name and comment that it *simulates*
  A6's path-substring filter over the tracked listing (:397-401); the nested
  vitest spawn was not taken, per the fix-r5 rejection rationale (self-
  recursion). The two `it` names landed slightly reworded from the snippet
  above ('tracks every moved and root-tier test file, and fails closed on
  leftover flat names' / 'simulated tests/pipeline selection matches every
  moved file and no root-tier test'); substance identical. Same two `it`
  blocks, so the file stays 26 files / 228 tests.
- `MOVED_NAMES`, `ROOT_TESTS`, `nested`, `rootTier`, `trackedTestFiles()` and
  the `stagePathAllowed` #4 block (:295-318) are unchanged.
- The performance seat's nit (memoize the git listing at describe scope;
  hoist `gitRoot()` in `prompts.test.ts`) was **not** taken — the helpers
  still spawn per call. The seat carried it approved-but-open at c2, so this
  is a recorded non-action, not a dissent.

Gate state: the runner re-verified `npm run check` green after this commit
(`.pipeline.json` gateLog: `testsGreen → proceed: true`, "npm run check
green"). The last stage-executed totals remain the post-move pass's 26 files /
228 tests green and 23 files / 190 tests for `npm test -- tests/pipeline`.

Code round 1 then closed: testing ratified at c1 (no findings); at c2 all five
seats approve revision stamp `deaf37f3…` (compatibility and security clean;
correctness carries its 1 minor; performance carries its 1 nit; testing adds
1 nit that its seat could not execute the gate itself). No further fix round
was routed — `testsNeedUpdate` false, `testsGreen` true.

## As-built pass (2026-11-10): whole-change-set comparison against the proposal

Read-side comparison of `d5092f3…HEAD` against `02-proposal.md` (the shell is
denied outright in this pass — `git diff`/`git log` rejected — so the change
set is verified from the tree, the transcripts, and `.pipeline.json`, not
from a diff render). Re-verified at HEAD: 23 files at
`tests/pipeline/<name>.test.ts` + the 3 root-tier files; exactly 62
`from '../../src/…'` imports, 16 `new URL('<rel>', import.meta.url)` sites,
4 inert `'../escape'` fixtures, and the updated comment at
`parallel-escalation.test.ts:45` — A8's closed enumeration, nothing outside
it; `cli.test.ts:4,29-30` still root-tier; both skill files carry
`tests/**/*.test.ts` with the naming rule stated once in `repo-practices`
(:21, :27-29); ADR-004 Decision bullet 2 carries the amended clause
(`docs/adrs/004-…:42-45`).

No As-built note existed in `02-proposal.md`; one dated 2026-11-10 is appended
there, consolidating the drift Deviations 1–5 already recorded (multi-commit
landing of Rollout 2, Rollout 1's four added `it` blocks, the subset/simulated
form of Test strategy #2/#3, #1's diff half as a review check) plus the two
open residues: the verbatim vitest reporter tail at the merge commit
(accepted correctness minor — still not attached; this pass could execute no
gate command), and the performance nit carried unaddressed.

## As-built verification pass (2026-11-10, second doc-sync pass)

The doc-sync pass that appended the "As-built pass" section above and the
As-built note in `02-proposal.md` **failed the runner's independence check**
— `.pipeline.json` `stageFailures."doc-sync": 1`, gateLog `stageFailure`:
"independence violation: read accessed run records:
docs/designs/004-mirror-src-hierarchy-in-tests/transcripts" — so its writes
are uncommitted working-tree content and `stageCommits` ends at `22be5986…`.
This pass re-verified that record without opening any transcript: the change
set was read from the working tree, `git log`/`.pipeline.json.stageCommits`,
and the source files. This pass' shell is denied outright as the prior one's
(every invocation, `git diff`/`npm run check` included, is rejected).

### Change-set inventory (`d5092f3…` → HEAD `22be5986…`)

33 commits in the range: 31 runner stage commits (`.pipeline.json.stageCommits`)
plus two operator-side commits — the escalation record `43b4c4ca…` and the move
`de51504a…`. They land the design in five groups:

- research, proposal, design-review rounds, editorial: `e2cd3b49…` …
  `1e903389…`.
- **Rollout 1** — `677b4c48…` "test: pin the mirrored test tree and
  nested-path gates before the ADR-004 move": Test strategy #4's
  `stagePathAllowed` expects plus the #1–#3 tree pins, written then as
  **exact-inventory** assertions.
- **Chosen design ¶4** — `35dc2a2b…` "docs: adopt the recursive test-tree
  convention in the two skill files": its own commit, ahead of the move
  (As-built drift 1).
- fix/test-update record commits `00c66a64…` … `5337a692…`, the run's
  escalation commit `43b4c4ca…`, then **Rollout 2 operator-side** —
  `de51504a…` "refactor: mirror src/ hierarchy in orchestration test tree"
  (¶1's 23 `git mv`s + ¶2's rewrites + the ADR bullet-2 clause), followed by
  gateLog `operatorForce` ("operator forced the next step: test-update") and
  `operatorExtension` (+3 review rounds).
- post-move verification `1a66bda6…`; code round 1 `32290c99…`; fix r5 design
  wording `5b32468d…`; the subset-pin rewrite `93032c60…` "test(orchestration):
  keep tree-mirror pins as subset invariants, drop transient counts"; code
  round 1 closed at `deaf37f3…` and `22be5986…`.

### Re-verification at HEAD (read side)

| Claim | Check at HEAD | Result |
|---|---|---|
| ¶1 | glob `tools/orchestration/tests/**/*.ts` | 26 files: 23 under `tests/pipeline/` (¶1's names one-for-one) + `cli`, `opencode-runner`, `opencode-runner-timeout` at the root tier |
| ¶2 | grep `from '../../src/` → **62**; grep `new URL(` → **16** (10 `'../../prompts/'`, the no-slash `'../../prompts'` at `fix-trigger.test.ts:8`, and `'../../../../.opencode/skills/…'` at `prompts.test.ts:215, 255, 265, 280, 305`); grep `../escape` → **4** (`identity.test.ts:65, 68, 103`; `state-store.test.ts:156`) | A8's closed enumeration exactly; nothing outside it |
| ¶2's comment | `parallel-escalation.test.ts:45` | reads `tests/pipeline/reply-contract.test.ts` |
| ¶1 root tier | grep `'\.\./src/` | only `cli.test.ts:4, 29-30`, `opencode-runner.test.ts:7`, `opencode-runner-timeout.test.ts:5` |
| ¶3 | `gates.ts:253, 262`; `tsconfig.json:8, 17, 18`; glob `**/vitest.config.*` | `TEST_PATH_RE` / `stagePathAllowed` exactly as A7/A19 describe; `rootDir: "./src"`, `include: ["src/**/*.ts"]`, `exclude: [..., "tests"]` as A4 describes; still no vitest config file (A5) — no config touched |
| ¶4 | skill files | `tests/**/*.test.ts` at `repo-practices/SKILL.md:21`, rule stated once at `:27-30`; glob-only at `test-strategy/SKILL.md:27` |
| ADR clause | `docs/adrs/004-…:42-45` | "a relative path reference - code or comment - naming a location this move passes takes its new depth" |
| #4 | `pipeline/gates.test.ts:295-318` | nested path denied to `implement`/`fix`, allowed to `test-authoring`/`test-update` |
| #1/#2/#3 | `pipeline/gates.test.ts:329-402` | zero-flat-name listing check (:387) + `toContain` subset (:388) + simulated A6 filter (:397-401); no exact 26/228 pin |
| #6 | `pipeline/prompts.test.ts:314-348` | `describe('test-tree convention')` reads both skill files |
| performance nit | `gates.test.ts:370-377, 386, 398`; `prompts.test.ts:315-317, 324, 337` | helpers still spawn `git rev-parse`/`ls-files` per call — carried nit unaddressed, as recorded |

### Gate evidence reachable without a shell

`.pipeline.json.gateLog` records `testsGreen → proceed: true` ("npm run check
green") at `at: 1794323193163` (after the post-move test-update `1a66bda6…`),
`1794324397965` (after fix r5 `5b32468d…`) and `1794324534912` (after the
subset-pin rewrite `93032c60…`). That answers the correctness minor's premise
that "no gate execution at the rewritten HEAD is recorded anywhere": the
runner did verify the gate at that commit. The counts themselves are still not
attached, so the minor's prescribed fix — the verbatim vitest reporter tail —
remains open. The testing seat's c2 nit ("optionally attach the runner's
`npm run check` output") is partially served by this paragraph.

### Drift beyond the As-built note (appended to `02-proposal.md` as items 6–8)

1. The lasting #2/#3 assertions were **rewritten after the move** in
   `93032c60…`; Rollout 1's `677b4c48…` landed them as exact-inventory pins,
   so the test side of the change set is three commits, not one.
2. Rollout 2 landed **mid-run at the escalation**, through `escalate` →
   `operatorForce` → `operatorExtension`, not between runs as Rollout 1's
   ordering clause (A18) assumed.
3. Both As-built artifacts (proposal note + the section above) are
   **uncommitted** — doc-sync r1's independence failure left them in the
   working tree; this pass' additions are, too.

No design decision was changed by this pass, and nothing outside these two
design-directory documents was edited.

## Remaining operator work (updated by this pass)

- Paste the vitest reporter tail (26 files / 228 tests; 23 / 190 nested;
  eslint/prettier pass lines) into this file at the merge commit — the
  accepted correctness minor's fix and Rollout 3's final evidence.
- Commit the As-built records: the note in `02-proposal.md` (items 1–8) and
  the two As-built sections in this file are working-tree content left by the
  failed doc-sync r1 pass, so they are not yet in any commit on the branch.
- Push/merge the branch; historical run artifacts stay as written (Deferred).
