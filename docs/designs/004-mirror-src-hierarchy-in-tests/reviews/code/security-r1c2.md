# Code review — security — r1 ratification c2

**Scope:** `git diff 1a66bda6a543508cf60bf792107a6d8b38bbfeee..HEAD -- ':(exclude)docs/designs'`
read against the full current `02-proposal.md` and
`docs/adrs/004-mirror-src-hierarchy-in-tests.md`.

**Prior seat state:** r1 verdict held no open security findings. This pass checks
only that the revisions landed since (the fix stage and the subset-invariant test
commit) did not break what was approved.

**Change under review:** one file,
`tools/orchestration/tests/pipeline/gates.test.ts` (+24/−19), all inside the
`test tree mirrors src/ hierarchy` describe block: the tree-pin comment reworded
to the amended Rollout 1 baseline, the exact 26-file equality pins relaxed to
subset-containment plus the zero-flat-name check, and the selection test renamed
to a simulated path-substring filter. No source, config, or dependency paths in
the diff (range stat shows this single file).

## Checklist walk

1. **External input at the boundary** — no new untrusted-input handling; the
   suite's only input is the repo's own tracked file listing via git. Pass.
2. **Path handling** — `trackedTestFiles()` (gates.test.ts:370–377) invokes
   `execFileSync('git', ['ls-files', '--full-name', 'tools/orchestration/tests/'])`:
   fixed argv array, no shell string interpolation, no traversal, no absolute
   paths from data. Pass.
3. **Injection** — both patterns are static literals:
   `/(^|\/)pipeline-[^/]*\.test\.ts$/` (linear, anchored, no backtracking risk)
   and `f.includes('tests/pipeline')`. No user-controlled strings into shell,
   regex, or paths. Pass.
4. **Secrets / PII** — none in the diff's comments, names, or data. Pass.
5. **New dependencies** — none; the range touches no manifest. Pass.
6. **Filesystem writes** — none. Pass.
7. **Fail-closed behavior preserved** — the security-bearing pieces r1 relied on
   are untouched by the revisions:
   - the nested-path `stagePathAllowed` assertions over the real post-move path
     (`nestedTest` at gates.test.ts:299, deny-to-coder / allow-to-test-stages at
     :301–318) — the fix for the r1 security major — are outside the diff and
     still present;
   - the fail-closed zero-flat-name check on the tracked listing survives the
     rewrite (gates.test.ts:387), matching amended Test strategy #2's
     "listing, not content grep" wording;
   - the move from `toEqual` on the full sorted listing to the containment loop
     (gates.test.ts:388) still fails on a half-applied move: a leftover
     `pipeline-*.test.ts` trips the line-387 regex at any depth, and a missing
     moved or root-tier file trips its containment assertion. It intentionally
     stops failing on *added* mirrored test files, which the amended proposal
     (Test strategy #2, Rollout note) requires post-merge. No gate or grant
     scope changed (¶3's trust boundary still holds: A7/A11/A19 sites untouched).

## Verdict

No regressions introduced by the post-r1 revisions. Nothing broken that this
seat approved.

**FYI (no severity, outside security specialty):** the root-tier exclusion
assertion at gates.test.ts:400 (`expect(selected).not.toContain(rootFile)`) is
structurally near-tautological given how `selected` is filtered, but it does
guard against a future broadening of the filter constant, so it retains
mutation-guard value — testing-seat nuance, no action requested.

## Findings

None.
