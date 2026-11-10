# Code review — testing seat, r1 pass c0

Scope: `git diff d5092f39755c4a0903dfd19207e4951dfc1cd33a...HEAD -- ':(exclude)docs/designs'`
Inputs: that diff, `02-proposal.md`, `docs/adrs/004-mirror-src-hierarchy-in-tests.md`. First pass — no prior findings to re-check.

## Verdict: request-changes (1 major, 2 minor)

## Checklist walk-through

**1. Every Test strategy item has a citing test.** Citations verified in the
final tree:
- #1/#2 — `tests/pipeline/gates.test.ts:377` (`tracks exactly 26 test files`), including #2's prescribed fail-closed `git ls-files` check (`:382`, zero `pipeline-*.test.ts` names) — listing, not content grep, as the revision log demands.
- #3 — `tests/pipeline/gates.test.ts:389/392` (see finding 2 on what it actually exercises).
- #4 — `tests/pipeline/gates.test.ts:302,312`: `stagePathAllowed` asserted on `tools/orchestration/tests/pipeline/gates.test.ts` — this file's own real post-move path — allowed for `test-authoring`/`test-update`, denied to `implement`/`fix`, exactly as #4 prescribes.
- #6 — `tests/pipeline/prompts.test.ts:319,330`: recursive glob present in both skill files, and a recursive scan of `.opencode/skills/` + `tools/orchestration/prompts/` finds no flat `tests/*.test.ts` statement (repo grep corroborates: remaining hits are the ADR's descriptive line, proposal/record files, and the checking test itself — not convention statements).
- #5 and #7 are execution-evidence items, not citable assertions: #5 is the `npm run check` gate (I ran it — green, tail below), #7's Linux half is items 1–5, executed here. macOS/Windows stay A16-dependent per the proposal. Treated as proven by execution, not counted as citation gaps.

**2. Unit and integration layers.** The #4 assertions are unit (pure
`stagePathAllowed`); discovery/selection counts are integration-evidence from
the suite run itself: `vitest run` → 26 files / 228 tests; `vitest run
tests/pipeline` → 23 files / 190 tests (root tier excluded), both reproduced
locally on Linux.

**3. Revision-round coverage.** The previously rejected items now have
catching tests: r1-correctness blocker (¶2 must cover all depth-sensitive
sites) — every moved suite resolves its own `new URL`/import paths, so any
missed site ENOENTs the suite red; I verified the move commit's content edits
match A8's closed enumeration exactly: 62 `../../src/` import rewrites, 16
`new URL` rewrites across exactly 11 files, and the one A14 comment line in
`parallel-escalation.test.ts` — nothing else inside moved files. r1-security
major (nested classification asserted on a real moved path) — gates.test.ts
#4 block. r1-security minor / r1-correctness major (fail-closed via file
listing) — `trackedTestFiles()` + zero-flat-name regex.

**4. Determinism.** New tests spawn only local `git rev-parse`/`git ls-files`;
no wall-clock, network, live server, or ports. No config change anywhere in
the diff (no tsconfig/vitest/eslint edits), consistent with Decision bullet 3.

**5. Names vs implementation.** One mismatch — finding 2.

**6. Gate run (mine, at HEAD, tools/orchestration):**

```
npm run check
...
All matched files use Prettier code style!
 Test Files  26 passed (26)
      Tests  228 passed (228)
   Duration  2.80s
```

**7. Untestable additions.** None: the skill-file glob is consumed by the #6
tests; the ADR bullet-2 amendment is documentation; no knobs or dead branches.

## Findings

### Major — the permanent suite hard-codes the transient acceptance inventory
`tools/orchestration/tests/pipeline/gates.test.ts:383,384,394`
The two new pins assert exact set equality against `MOVED_NAMES`/`ROOT_TESTS`
(`:383` sorted-equality, `:384` `toHaveLength(23)`, `:394` equality again).
"26 exactly" is Test strategy #2/Rollout 3's **acceptance measurement at the
move commit**, not a lasting invariant; the ADR's own forward rule ("New
subsystems follow the same shape"; "from merge, the test-authoring and
test-update stages place new tests in the mirrored layout") guarantees a
future `tests/<subsystem>/*.test.ts` or new pipeline test file — and these
tests then go red on every unrelated branch that adds one, forcing a
`MOVED_NAMES` edit in an unrelated suite through the pipeline's test-update
stage. The comment at `:330-336` even asserts the expectations "hold only
from that commit on" — they in fact hold only until the next test file lands.
Fix: keep the two real invariants permanent — the zero-flat-name listing
check (`:382`) and subset assertions (each of the 23 named moved files and
each of the 3 root files is tracked) — and drop the exact set-equality and
`toHaveLength(23)`; the "exactly 26 / 228" figures stay as the rollout's
reported acceptance evidence.

### Minor — the #3 test emulates vitest selection instead of exercising it
`tools/orchestration/tests/pipeline/gates.test.ts:392-399`
The test is named "vitest run tests/pipeline selects exactly the 23 moved
files" but never invokes vitest; it applies `includes('tests/pipeline')` to
`git ls-files` output, pinning the substring emulation and the untested A6
premise rather than vitest's real selection. (I confirmed the real behavior
holds at HEAD: `npm test -- tests/pipeline` → 23 files / 190 tests, root
tier excluded — so no false claim ships today, but the test would stay green
if vitest's filter semantics changed.) Fix: run the real command in a
subprocess and compare the reported file list, or rename the test to state
it simulates the path-substring filter per A6.

### Minor — Rollout 1's "test and file counts unchanged" clause is not what landed
Pre-move test commit (within this range) added four new `it` blocks (gates +
prompts `describe`s), and the `stagePathAllowed` #4 expects landed in a later
test commit; the listing/#3 pins are red by construction from that commit
until the move commit (their own comment at `gates.test.ts:330-336` says the
expectations "hold only from that commit on"), so the suite was red across
13 intermediate commits. The proposal's Non-goals/Rollout 1 wording says the
pre-move assertions sit "inside `pipeline-gates.test.ts`'s existing
`stagePathAllowed` block, so test and file counts are unchanged". The added
tests are the right call — the Test strategy's "tests must prove #1–#3, #6"
cannot be met otherwise, and the move-commit diff itself stays inside ¶2's
allowed set — but the design text and the landed history disagree. Fix:
amend the proposal/rollout note to record the deviation (extra test cases
beyond the #4 expects; pins deliberately land red-before-move), so the
228/26 baseline definition in `gates.test.ts:330-336` matches the design's
stated baseline.

## FYI (no severity)

The naming rule landed once in `repo-practices/SKILL.md` while
`test-strategy/SKILL.md:27` only flips the glob; that fits the ADR's bullet 5
(naming only repo-practices) and ¶4's "stated once" reading, but ¶4's
"to `tests/**/*.test.ts` plus this rule" is ambiguous on whether both
surfaces should carry the rule — correctness seat's call, not testing.
