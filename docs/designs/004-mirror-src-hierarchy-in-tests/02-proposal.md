# Design: Mirror the src/ hierarchy in the orchestration test tree

## Context

`tools/orchestration/tests/` is flat: 23 of its 26 test files encode the
`pipeline/` subsystem as a `pipeline-` filename prefix instead of a directory.
ADR-004 (docs/adrs/004-mirror-src-hierarchy-in-tests.md) decides to move those
23 files to `tests/pipeline/<name>.test.ts`, so the test tree mirrors the two
tiers of `src/` (package root, `src/pipeline/`). Research (01-research.md)
verified every load-bearing toolchain claim statically.

## Goals / Non-goals

- Goals:
  - `tests/pipeline/` mirrors `src/pipeline/`; the 23 prefixed files move and
    lose their prefix (ADR Decision ¶1).
  - History follows each file via `git mv` (Decision, bullet 1).
  - Content inside a moved file changes only at Chosen design ¶2's depth-sensitive sites and its one comment line.
- Non-goals:
  - Decision bullet 2 constrains the move commit's diff only: no change there
    to test logic, test names, fixture inputs, or file splits — 26 test files before,
    26 after. Rollout 1's added assertions are a separate test change outside
    ADR-004's subject: it keeps the 26-file count and adds cases (four new `it`
    blocks, 228 tests), and the tree pins it adds are red by construction until
    the move commit lands. (Wording corrected against the landed history — the
    earlier "leaving both counts unchanged" described the file count only;
    code-review r1, testing, minor.)
  - No repackaging of `src/pipeline/` (ADR Alternatives; see Deferred).

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | `tests/` is flat with exactly 26 files: 23 match `pipeline-*.test.ts`, 3 are package-root (`cli`, `opencode-runner`, `opencode-runner-timeout`) | verified | tools/orchestration/tests/ directory listing |
| A2 | The 23 names the ADR lists match the on-disk files one-for-one after prefix stripping; no two collide inside `tests/pipeline/` | verified | docs/adrs/004-mirror-src-hierarchy-in-tests.md:33-38 vs tests/ listing |
| A3 | `src/pipeline/` holds 13 modules, not the "14" the ADR Context says | verified | tools/orchestration/src/pipeline/ listing vs ADR:16-17 |
| A4 | TypeScript never compiles tests: `rootDir: "./src"`, `include: ["src/**/*.ts"]`, `exclude: [..., "tests"]` | verified | tools/orchestration/tsconfig.json:8,17,18 |
| A5 | No `vitest.config.*`/`vite.config.*` exists in the repo; vitest's default recursive `include` discovers `tests/pipeline/*.test.ts` with no change | verified | repo glob (no matches); https://vitest.dev/config/include |
| A6 | `vitest run tests/pipeline` selects every file under the new directory (positional args are path-substring filters) | verified | https://vitest.dev/guide/cli |
| A7 | `TEST_PATH_RE = /(^|\/)tests\/|\.test\.ts$/` already matches nested `tests/…/x.test.ts` paths, so path gates are unchanged | verified | tools/orchestration/src/pipeline/gates.ts:253,262 |
| A8 | Test files never import each other; all 87 `../` occurrences in the 26 files fall into four classes: 65 `from '../src/...'` imports (62 in moved files), the 2 non-import `../src/` sites at `cli.test.ts:29-30` (`vi.mock`, type-import), 16 depth-sensitive `new URL('<rel>', import.meta.url)` paths in 11 moved files, and 4 inert `'../escape'` fixture strings | verified | grep `\.\./` across tools/orchestration/tests (87 matches), decomposed by greps `from ['"]\.\./src/` (65), `new URL\(` (16), `\.\./escape` (4) |
| A9 | The lint/format gate is layout-agnostic: eslint ignores only `dist`/`node_modules`; `npm run check` chains lint, format:check, test | verified | tools/orchestration/eslint.config.js:5; tools/orchestration/package.json:23 |
| A10 | The prettier import-sort plugin's intra-file relative order is unchanged by a uniform `../src/` → `../../src/` rewrite | verified | package.json:35 (`@trivago/prettier-plugin-sort-imports`); uniform-prefix argument (A8) |
| A11 | Agent permissions already govern nested tests: the test-engineer agent edits `**/tests/**` | verified | .opencode/agents/code/test-engineer.md:9 |
| A12 | The test-authoring stage prompt is directory-generic, so it needs no edit for nesting | verified | tools/orchestration/prompts/test-authoring.md:3 |
| A13 | Two convention surfaces encode the old flat form, not one: repo-practices and test-strategy both say `tests/*.test.ts` | verified | .opencode/skills/repo-practices/SKILL.md:21; .opencode/skills/test-strategy/SKILL.md:27 |
| A14 | One real in-test comment references a moved file's old path (`tests/pipeline-reply-contract.test.ts`); other in-test path mentions are inert fixture data | verified | tools/orchestration/tests/pipeline-parallel-escalation.test.ts:45; pipeline-independence.test.ts:145; pipeline-gates.test.ts:298-303 |
| A15 | `git mv` plus ¶2's handful of changed lines keeps rename similarity near 100% (rename detection is on by default at 50% threshold) | verified | https://git-scm.com/docs/git-mv, https://git-scm.com/docs/git-diff |
| A16 | Case-insensitive checkouts (macOS/Windows) see nothing special: no case-only or case-colliding renames among the 23 names | unverified | inferred from the all-lowercase, distinct name list (ADR:33-38); not executed on those platforms |
| A17 | Acceptance (`npm run check` green, test count 26→26) has not been executed yet; research confirmed mechanisms only | unverified | 01-research.md "No verification of the acceptance gate yet"; package.json:21-23 |
| A18 | No orchestration run is mid-branch editing these test files at merge time | unverified | ADR:84-85 states intent only; must be checked at rollout |
| A19 | `TEST_PATH_RE` is the only test-path classifier in the gate and ownership code, so nesting cannot silently escape a pattern: it is defined at `gates.ts:253` and consumed by `stagePathAllowed` (`gates.ts:262`, enforced at `runner.ts:232`) and the revert check (`runner.ts:1031`); no pattern in `src/` keys on the flat `pipeline-*` filename shape | verified | static enumeration — grep for `tests`/`test.ts`/`TEST_PATH_RE`/`stagePathAllowed` over tools/orchestration/src (only the cited sites) |

## Options considered

**Keep the flat `pipeline-` prefix.** Rejected (as in the ADR): the prefix is a
namespace without a hierarchy that every future `src/` subsystem extends.

**Strict one-file-per-`src/`-module**, merging behavior suites into
module-named files. Rejected (as in the ADR): most pipeline tests pin
cross-module behavior from ADR-002/ADR-003 (`funnel`, `pass-finalize`,
`parallel-fanout`); merging loses the "which behavior is pinned where" signal.

**Make `src/pipeline/` its own package with its own tests.** Out of scope; the
layout fix should not wait on a packaging decision (see Deferred).

**Amend only the repo-practices convention line** (the only one the ADR names).
Rejected: the same flat form is in `.opencode/skills/test-strategy/SKILL.md:27`
too (A13); amending one copy leaves stages writing flat names. ¶4 amends both.

**Let the cross-reference comment go stale** (strict reading of "only the
import depth"). Rejected: `pipeline-parallel-escalation.test.ts:45` names
`tests/pipeline-reply-contract.test.ts`, a path this commit destroys (A14) — a
defect the move creates, not historical record; hence ¶2's deviation.

## Chosen design

The move is one commit with no config change, in four parts; the paths it touches
are named in Rollout 2, and Test strategy #4's assertions land before it (Rollout 1).

1. **Moves (Decision ¶1, bullet 1).** `git mv` all 23 files matching
   `tests/pipeline-*.test.ts` to `tests/pipeline/<name>.test.ts`, dropping the
   prefix: `budget`, `checkpoint-writer`, `cli-max-parallel`, `concurrency`,
   `escalation`, `fix-trigger`, `funnel`, `gates`, `identity`, `independence`,
   `ownership`, `panel`, `parallel-escalation`, `parallel-fanout`,
   `parallel-integration`, `pass-finalize`, `prompts`, `proposal-size`,
   `reply-contract`, `runner`, `stage-lifecycle`, `state-store`, `states` (A2).
   `tests/cli.test.ts`, `tests/opencode-runner.test.ts`, and
   `tests/opencode-runner-timeout.test.ts` stay at the package-root tier.
   Rename detection follows every file (A15).

2. **Content edit (Decision, bullet 2 — as amended by this commit).** In each
   moved file, rewrite every depth-sensitive `../` reference A8 enumerates: the
   62 `from '../src/...'` imports to `from '../../src/...'`, and the 16
   `new URL('<rel>', import.meta.url)` paths — `../prompts/` → `../../prompts/`,
   `../../../.opencode/…` → `../../../../.opencode/…` — which resolve against
   the moved file's own location and would ENOENT one tier deeper. Import order
   is unchanged (A10). Bullet 2 permits import depth alone, so this commit
   amends it: "a relative path reference — code or comment — naming a location
   this move passes takes its new depth" (the comment site: A14).

3. **No config change (Decision, bullet 3).** Vitest discovers the nested tree
   via its default recursive `include` (A5); `tsconfig.json` excludes `tests/`
   from every compile (A4); eslint/prettier are layout-agnostic (A9);
   `gates.ts`'s `TEST_PATH_RE` already matches nested paths (A7); the
   test-engineer agent's `**/tests/**` permission and the directory-generic
   stage prompts need no edits (A11, A12). Trust boundary: nothing here parses
   untrusted or network input — it moves only trusted, repo-local files, and their
   only security-relevant consumers are that gate and that grant, unchanged in
   scope (A7, A11, A19).

4. **Convention rule (Decision, bullets 4–5), stated once here:** a directory
   under `tests/` mirrors a subsystem directory under `src/`; a module-named file
   (`state-store.test.ts`) is that module's unit test, and cross-module behavior
   suites keep descriptive names (`funnel.test.ts`). The commit amends both
   surfaces encoding the flat form — `.opencode/skills/repo-practices/SKILL.md:21`
   and `.opencode/skills/test-strategy/SKILL.md:27` — to `tests/**/*.test.ts`
   plus this rule (A13).

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A pipeline run mid-branch edits old test paths across the merge, producing a half-mirrored tree (A18) | Low | Medium | Rollout step 1 lands between runs; Test strategy #2 fails closed on any leftover flat filename |
| A second convention line is missed, leaving stages writing flat names | Medium if unmitigated | Medium | Chosen design ¶4 names both files; Test strategy #6 greps for the flat form |
| The deviation hides a broader content change | Low | High | It is A8's closed enumeration — 16 `new URL` sites plus one comment (A14); Test strategy #1 asserts the moved-file diff stays inside ¶2's allowed set |
| Acceptance gate not yet executed; 26→26 count rests on inspection (A17) | Low | Medium | Rollout step 3 runs `npm run check` and compares the vitest reporter's file/test counts to the Rollout-1 baseline |
| Fixture path strings still describe the old layout, teaching the wrong convention to future readers | Medium | Low | Accepted; fixture inputs keep gate assertions valid either way; refresh deferred (Deferred) |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux (dev/CI host) | Yes | Statically verified (A4–A9); `npm run check` execution pending (A17) |
| macOS (case-insensitive checkout) | Expected — assumption | No case-only or case-colliding renames among the 23 names (A16); not executed there |
| Windows (case-insensitive checkout) | Expected — assumption | Same basis as macOS (A16); git handles the rename identically (A15); not executed there |
| Node >= 22.12.0, vitest ^5.0.5, TypeScript ^6.0.3 | Yes | Move touches no Node APIs; discovery/compile behavior verified at these pinned majors (A4, A5; package.json:31,41,43) |

## Test strategy

No new test files: the move is proven by the existing suite being unchanged
against the pre-move baseline Rollout 1 defines. Tests must prove:

1. Unit: every previously existing test still exists and passes unmodified
   against that baseline, and the moved files' diff contains only ¶2's allowed
   set: `../../src/` import lines, depth-adjusted `new URL` lines, and its one comment line.
2. Integration: `vitest run` after the move discovers and runs exactly 26 test
   files (23 under `tests/pipeline/`, 3 at `tests/` root; A1, A5) with the
   baseline test count, and fails closed on a half-applied move: a file listing
   (`git ls-files tools/orchestration/tests/`) finds zero files named
   `pipeline-*.test.ts`; in-file fixture strings are not files (Deferred). The
   exact 26-file/228-test figures are that measurement at the move commit —
   Rollout 3's reported acceptance evidence — not a permanent suite invariant:
   the lasting assertions there are the zero flat names in the listing plus the
   23 moved and 3 root-tier paths as a *subset* of it, since new mirrored test
   files are expected from the merge on.
3. Integration: `vitest run tests/pipeline` selects exactly the 23 moved files
   and none of the 3 root-tier tests (A6). The reported selection is the proof;
   the suite's own assertion simulates that path-substring filter over the
   tracked listing rather than spawning a nested vitest run.
4. Unit: the existing `stagePathAllowed` block in the gate suite asserts the real
   post-move path `tools/orchestration/tests/pipeline/gates.test.ts` — allowed
   for `test-authoring`/`test-update`, denied to `implement`/`fix` — so nested
   classification is asserted, not inferred from inert fixture strings (A19).
5. Integration: `npm run check` (lint, format:check, test) is green after the
   commit, the ADR's acceptance gate (A9, A17).
6. Integration (static): grep over `.opencode/skills/` and
   `tools/orchestration/prompts/` finds no remaining `tests/*.test.ts`
   flat-form convention statement (A13).
7. Platform: Linux executes items 1–5; macOS/Windows rely on A16 unless a
   maintainer runs item 2 there.

## Rollout

1. Confirm no orchestration run is mid-branch against those test files (A18): the
   ADR's ordering clause requires landing between runs. Then land the test-only
   commit with item 4's assertions — `expect`s inside `pipeline-gates.test.ts`'s
   existing `stagePathAllowed` block, which `stagePathAllowed`'s purity keeps true
   on either side of the move — together with items #1–#3's tree pins, which are
   red by construction from this commit until the move commit lands. That commit
   adds cases, so the invariant it preserves is the 26-file count while the test
   count becomes 228; that 26-file/228-test pair is the pre-move baseline for Test
   strategy #1–2 and Risk 4.
2. Land the move commit: `git mv` ×23 (alphabetical), ¶2's rewrite of every
   depth-sensitive `../` reference and its one comment line, the two skill-file
   convention amendments, and the one-clause amendment to Decision bullet 2
   sanctioning those references. Commit message per AGENTS.md conventions:
   `refactor: mirror src/ hierarchy in orchestration test tree`.
3. Verify Test strategy items 1–6 locally before pushing; the reported
   file/test counts are the acceptance evidence.
4. No migration or defaults change: from the merge, downstream stages place tests in
   the mirrored layout (¶4); historical artifacts stay as written (Deferred).

## Deferred / out of scope

- Repackaging `src/pipeline/` as its own package — future packaging ADR; the
  test tree is already shaped for it (ADR Alternatives).
- Refreshing fixture path strings that mimic the old naming (e.g.
  `pipeline-lane.test.ts` at `pipeline-independence.test.ts:145`) — a later
  test-update pass may align them; they are not in Test strategy #2's listing.
- ADR-004 Context's "14 modules": left as written, with the verified count in A3.
  Only its Decision bullet 2 is edited, by the move commit (Chosen design ¶2).
- Updating historical run artifacts (run 002 postmortem, run 003 build notes
  and reviews) that cite old paths — records, not references (ADR Consequences).
- Colocating tests with source (Go-style) or any other layout alternative —
  settled by ADR-004; no follow-up.

## Revision log

- Chosen design ¶1 (r1 correctness, minor): the commit's areas are named in Rollout 2; ¶1 drops the path qualifier.
- Chosen design ¶2 (with A8, Test strategy #1, Rollout 2) (r1c2 correctness, blocker; r1
  correctness, major + minor): one amendment — bullet 2 covers every depth-sensitive `../` reference, A8's 87-site enumeration, not imports alone.
- Non-goals / Rollout 1 (r1 correctness, major): bullet 2 constrains the move commit's diff only;
  Rollout 1's added assertions are a separate test change, and that commit is the pre-move baseline.
- Test strategy #2 (r1 security, minor; r1 correctness, major): the fail-closed check is a file
  listing, not a content grep, so deferred fixture strings miss it.
- Test strategy #4 (r1 security, major): nested-path classification is asserted over a moved file's
  own path, plus A19's enumeration of the gate's path patterns.
- Chosen design ¶3 (r1 security, minor): trust boundary stated.
- Non-goals / Rollout 1 / Test strategy #2–#3 (code-review r1, testing, major + minor):
  records what landed and keeps it durable — Rollout 1's test-only commit adds cases
  (228 tests at 26 files) and its tree pins are red until the move commit; the exact
  26/228 and 23-selection counts are the move commit's acceptance measurement, not
  permanent invariants, so the permanent suite asserts the zero-flat-name listing plus
  the moved and root-tier paths as a subset, and its #3 check simulates A6's filter.

Editor notes: Chosen design ¶2 — "a relative path reference — code or comment — naming a location this move passes takes its new depth" is ambiguous; author to clarify.

## As-built (2026-11-10)

Post-implementation record of what landed in `d5092f3…HEAD`, appended after the
change set passed code review round 1 (all five seats approve stamp
`deaf37f3…`). Design decisions are unchanged; this records build drift only.

Landed-as-designed (verified against the tree at HEAD): ¶1's 23 moves to
`tests/pipeline/<name>.test.ts` with the 3 root-tier files unmoved; ¶2's full
A8 rewrite set — 62 `from '../../src/…'` imports, 16 depth-adjusted
`new URL('<rel>', import.meta.url)` sites (including the no-trailing-slash
`'../../prompts'` at `fix-trigger.test.ts:8` and the five
`'../../../../.opencode/skills/…'` reads in `prompts.test.ts`), and the one
comment at `parallel-escalation.test.ts:45`; the 4 inert `'../escape'` fixtures
and `cli.test.ts:4,29-30` untouched; ¶3's no-config-change; ¶4's two skill
amendments with the naming rule stated once in `repo-practices`; the ADR-004
Decision bullet-2 amendment clause in place. Test strategy #4 and #6 assert
green; the tree pins and simulated-selection pins live in
`tests/pipeline/gates.test.ts`.

Drift from this proposal, with reason and affected sections (full working
record in 03-build-notes.md, Deviations 1–5):

1. **Rollout 2's "one commit" landed as several** (Affects: Chosen design
   intro, Rollout 2). The `git mv` ×23, ¶2's rewrites, and the ADR clause
   landed together in the operator's commit
   `refactor: mirror src/ hierarchy in orchestration test tree`, but ¶4's two
   skill amendments landed earlier in a separate implement-stage commit
   (`docs: adopt the recursive test-tree convention in the two skill files`).
   Reason: the harness commits once per stage and no stage agent has git
   access or a rename/delete primitive; the ownership sweep would revert any
   stage-side move (Deviations 1–4). Same content, split commits; the move
   commit's diff still stays inside ¶2's allowed set.
2. **Rollout 1's pre-move commit added cases, not only `stagePathAllowed`
   expects** (Affects: Rollout 1, Non-goals bullet 1). It landed as four new
   `it` blocks (tree pins in `pipeline-gates.test.ts`, convention checks in
   `pipeline-prompts.test.ts`), red by construction until the move commit;
   the 26-file/228-test pair was preserved and is the verified baseline. This
   was folded into the proposal text during code round 1 (see the Revision
   log's "code-review r1, testing" entry), so current ¶-text and history
   agree; the drift was against the design-round-1 stamp `1d1e68a6…`.
3. **Test strategy #2/#3's exact-count checks landed as subset assertions and
   a simulated A6 filter** (Affects: Test strategy #2, #3). Per the accepted
   testing-seat major/minor, the permanent suite pins only the zero-flat-name
   `git ls-files` check plus the 23 moved and 3 root paths as a subset, and
   #3's in-suite check simulates the path-substring filter (spawning a nested
   vitest run was rejected as self-recursive). The proposal was amended to
   match in the same round; the exact 26/228 and 23/190 figures remain the
   rollout's reported acceptance evidence, not suite invariants.
4. **Test strategy #1's diff half is a commit-review check, not a suite
   assertion** (Affects: Test strategy #1). Enforced statically at review:
   the moved-file diff is exactly the 62 + 16 + 1 allowed sites (plus
   prettier re-wraps of six over-width rewrites, named in the build notes).
5. **Acceptance-evidence residue open at merge** (Affects: Rollout 3).
   `npm run check` green — 26 files / 228 tests, 23 files / 190 tests for the
   nested selection — is recorded as prose in 03-build-notes.md; the accepted
   correctness minor asks that the verbatim vitest reporter tail be attached
   at the merge commit, which no stage or this pass could produce (all shells
   denied). The performance seat's memoize-the-listing nit was carried
   approved-but-unaddressed.

### As-built addendum (2026-11-10, verification pass)

Items 1–5 above stand; nothing here restates them. This pass re-checked them
against the tree at HEAD (`22be5986…`, `pipeline/004-mirror-src-hierarchy-in-tests`)
from the tree, `git log`/`.pipeline.json.stageCommits`, and the source files —
the shell is denied here, so there is no diff render and no gate run — and
found all of them accurate: 23 files at `tests/pipeline/<name>.test.ts` plus
the 3 root-tier files; 62 `from '../../src/…'` specifiers, 16
`new URL('<rel>', import.meta.url)` sites, 4 inert `'../escape'` fixtures, and
the updated comment at `parallel-escalation.test.ts:45`; `gates.ts:253,262`
unchanged (A7, A19); `tests/**/*.test.ts` at
`.opencode/skills/repo-practices/SKILL.md:21` with the rule stated once at
`:27-30` and the glob-only edit at `test-strategy/SKILL.md:27`; the ADR-004
Decision bullet-2 clause at
`docs/adrs/004-mirror-src-hierarchy-in-tests.md:42-45`; all five code seats
approve stamp `deaf37f3…`. Verification table: 03-build-notes.md, "As-built
verification pass".

Drift not recorded above, with reason and affected sections:

6. **The lasting Test strategy #2/#3 assertions were rewritten after the move,
   in a third test-side commit** (Affects: Rollout 1, Test strategy #2, #3).
   Rollout 1's commit `677b4c48…` landed the tree pins as exact-inventory
   checks (equality against the 26 post-move paths, `toHaveLength(23)`); the
   zero-flat-name listing plus subset plus simulated-A6 form that #2/#3 now
   describe landed only after the move, in `93032c60…`. Reason: the testing
   seat raised the exact-count major/minor at code round 1, after the move had
   already landed. Ordering drift — the amended text describes the end state,
   which HEAD matches.
7. **Rollout 2 landed mid-run, not between runs** (Affects: Rollout 1, Rollout
   2, A18). Fix rounds r0–r4 could not land the `git mv` ×23 (no stage agent
   has git access or a rename/delete primitive), so the run reached `escalate`
   at the code-review budget limit; the operator then committed the move
   (`de51504a…`) and forced the next stage (`operatorForce` → `test-update`,
   `operatorExtension` +3 rounds). Rollout 1's "no run mid-branch" precondition
   was met by operator intervention inside the run, not by landing between runs.
8. **These As-built records are working-tree state until the operator commits
   them.** The doc-sync pass that appended the note above failed the runner's
   independence check (it read `transcripts/` — `.pipeline.json`
   `stageFailures."doc-sync"`, gateLog `stageFailure`), so no commit in
   `d5092f3…22be5986` contains it. Item 5's two residues stay open — the
   verbatim reporter tail is unattached (shells denied again) and the
   performance nit is unaddressed (the helpers still spawn `git` per call) —
   narrowed by one fact: the runner's gateLog does record `npm run check` green
   after the subset-pin rewrite, so what is missing is the counts, not any gate
   execution.
