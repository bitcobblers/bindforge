# Security review — ADR-004 test-tree mirror (code, round 1, pass c0)

Seat: security. Inputs: `git diff d5092f39755c4a0903dfd19207e4951dfc1cd33a...HEAD
-- ':(exclude)docs/designs'`, 02-proposal.md, docs/adrs/004-mirror-src-hierarchy-in-tests.md.
First pass for this seat; no prior findings to re-check.

## Change surface (from the diff stat)

26 files: 23 renames `tests/pipeline-<name>.test.ts` → `tests/pipeline/<name>.test.ts`
(similarity 91–99%; gates 71% and prompts 80% because the Rollout-1 test-update
commits add new suites), plus two `.opencode/skills` convention lines and the
one-clause ADR-004 Decision bullet-2 amendment. No `src/` file, no config file,
no `package.json`/lockfile, no prompts touched.

## Checklist

1. **External input at the boundary** — PASS. Nothing in the diff parses shared
   binding files or network input. The only new parsing is of `git ls-files`
   output and repo-local markdown/skill files — all trusted, repo-local
   (proposal ¶3 trust boundary holds as written).

2. **Path handling** — PASS. All depth rewrites (`../src/` → `../../src/`,
   `../prompts/` → `../../prompts/`, `../../../.opencode/…` →
   `../../../../.opencode/…`) are compile-time constants in test code; no `..`
   traversal introduced beyond the pre-existing relative-import depth the move
   requires, no absolute paths built from data. New test code uses
   `execFileSync('git', ['rev-parse'|'ls-files', …])` with fixed argv — no
   shell, no interpolation — and `join(gitRoot(), …)` over trusted directory
   names.

3. **Injection** — PASS. No user-controlled string reaches a shell, a file
   path, or a regex. The one new regex, `/(^|\/)pipeline-[^/]*\.test\.ts$/`
   (gates.test.ts:382), is linear — no nested quantifiers, no backtracking
   exposure.

4. **Secrets / PII** — PASS. Added lines contain only synthetic fixtures
   (`slug '002-x'`, `user.email=f@f`); no tokens, paths, or PII. Nothing in the
   diff exports or logs sensitive data.

5. **New dependencies** — PASS (none). No manifest or lockfile changes in the
   diff, so no advisories to check.

6. **Filesystem writes** — PASS. No production code changed. The two new test
   suites are read-only (git listings, `readFile` of skills/prompts); existing
   writes remain confined to `mkdtemp` temp dirs, unchanged.

7. **Clean rejection** — PASS. Gate behavior (`TEST_PATH_RE`,
   `stagePathAllowed`) is untouched in `src/`; the diff only *strengthens* the
   security-relevant coverage: gates.test.ts:299–314 now asserts the real
   post-move nested path (`tools/orchestration/tests/pipeline/gates.test.ts`)
   is denied to `implement`/`fix` and allowed to `test-authoring`/`test-update`
   — exactly Test strategy #4, closing the earlier design-review security
   finding that nested classification was only inferred from inert fixtures.
   The fail-closed flat-name check (#2) is a tracked-file listing, not a
   content grep, so the deferred fixture strings cannot mask a half-applied
   move; I verified the post-move tree on disk: 23 files under
   `tests/pipeline/`, 3 at the root, zero `pipeline-*.test.ts`.

## Consistency with the design

- The ADR bullet-2 amendment in the diff matches proposal Rollout 2's
  "one-clause amendment … sanctioning those references"; the only comment-line
  content change is the single sanctioned site at
  `tests/pipeline/parallel-escalation.test.ts:45` (A14). No security-relevant
  logic, names, or fixtures changed in any moved file.
- Skill-file amendments match proposal ¶4 on both A13 surfaces;
  `tests/**/*.test.ts` does not contain the flat substring the new grep test
  hunts, so it cannot false-positive on its own fix.

## Verdict

**approve** — zero open blocker or major security findings.

Note: the acceptance gate (`npm run check` execution) was not run in this
review seat (shell restricted); security items above are all evaluable
statically and were. Confirming the suite's greenness is the testing/correctness
seats' evidence.

## FYI (out of specialty, no severity)

- Rollout 1/2 composition deviates from the proposal's letter: the
  tree-mirror and convention suites were added as separate `test:` commits
  (some asserting post-move state before the move commit exists, so
  `npm run check` would be red at those intermediate commits), and the skill/ADR
  amendments land outside the move commit de51504. The cumulative end state at
  HEAD is consistent with the design; per-commit bisection hygiene is for the
  correctness/testing seats.
