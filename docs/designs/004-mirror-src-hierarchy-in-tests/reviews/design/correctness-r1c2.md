# correctness review r1c2 (dissent pass — re-check of my r1 findings against the revision)

verdict: request-changes

Closures first: all five of my r1 findings are closed for real, not just reworded —
verified against the repo, not just the prose.
(1) Baseline: Rollout 1 now names the test-only commit as "the pre-move baseline for Test
strategy #1–2 and Risks 4"; its added `expect`s sit inside `pipeline-gates.test.ts`'s existing
`stagePathAllowed` describe (pipeline-gates.test.ts:294-315), so vitest's file/test counts are
genuinely unchanged; item 1's deleted "assertion counts per file" clause is gone; Risks 4 says
"the Rollout-1 baseline". Closed.
(2) Bullet-2 double reading: Non-goals now states one reading ("Decision bullet 2 constrains the
move commit's diff only… Rollout 1's added assertions are a separate test change… leaving both
counts unchanged"); the ADR's acceptance gate (Consequences: "unchanged test count") still holds
across both commits. Closed.
(3) Fail-closed check: item 2 is now filename-level (`git ls-files tools/orchestration/tests/`),
so the deliberately kept fixture string at pipeline-independence.test.ts:145 no longer trips it;
Deferred says so explicitly; Risks 1 says "leftover flat filename". Closed.
(4) ¶1 path qualifier: dropped; Rollout 2 names the areas. Closed.
(5) ¶2 hazard clause: replaced with checked facts; both A8 greps verified (65 from-imports, all
`../src/...`; 67 `../src/` total; the extra 2 are cli.test.ts:29-30 `vi.mock` / `importOriginal`
type-import). Closed — but the replacement text introduced a new false claim, below.

## Findings

- [blocker][new] Chosen design ¶2 (with A8, Test strategy #1, Rollout 2) — "no moved file holds a
  relative specifier outside those 65, and the only other `../src/` sites are the `vi.mock` and
  type-import in `cli.test.ts:29-30`, which stays at the package-root tier" — this new claim is
  false, and it hides the one site class the move actually breaks. `grep \.\./ ` over
  `tools/orchestration/tests/` finds 87 matches, not 67: the 65 `from` imports + 2 `cli.test.ts`
  `../src/` sites + 16 `new URL('<rel>', import.meta.url)` specifiers in 12 moved files
  (`'../prompts'`/`'../prompts/'` at pipeline-fix-trigger.test.ts:8, pipeline-prompts.test.ts:21,
  pipeline-budget.test.ts:85, pipeline-reply-contract.test.ts:83, pipeline-stage-lifecycle.test.ts:108,
  pipeline-funnel.test.ts:187, pipeline-proposal-size.test.ts:191, pipeline-parallel-integration.test.ts:191,
  pipeline-pass-finalize.test.ts:192, pipeline-parallel-fanout.test.ts:199, pipeline-parallel-escalation.test.ts:203;
  `'../../../.opencode/skills/…'` at pipeline-prompts.test.ts:213,251,261,274,297) + 4 inert
  `'../escape'` fixture strings (pipeline-identity.test.ts:65,68,103; pipeline-state-store.test.ts:156).
  `new URL` resolves against the moved file's own location, so one tier deeper turns
  `../prompts` into `tests/prompts` (absent) and `../../../.opencode/…` into `tools/.opencode/…`
  (absent), and these are live disk-read paths: `loadTemplate` is `readFile(join(promptsDir, name))`
  (prompts.ts:26-27); pipeline-fix-trigger.test.ts:60 reads `fix-tests.md` through its
  `PROMPTS_DIR`; pipeline-prompts.test.ts:212-215 reads the skill files directly; the other nine
  suites feed that `promptsDir` into `executeStage`/`resumePipeline`, which render stage templates
  from disk. As written the design rewrites only A8's 65 imports plus one comment line, and Test
  strategy #1 bounds the moved-file diff to exactly that allowed set — so the ENOENT breakage is
  neither tasked nor permitted, `npm run check` cannot be green after the move, and ADR bullet 2
  ("the only content change permitted is the import depth (`../src/...` -> `../../src/...`)")
  forbids the repair. The A8 row itself is narrowly true (it speaks only of imports and `../src/`
  occurrences), but its two cited greps structurally cannot see the URL sites, so it cannot
  support ¶2's completeness claim; note also the 65 span all 26 files (62 sit in moved files),
  so "the 65" is not the in-moved-file rewrite count either. Fix: one coherent amendment —
  extend ¶2's rewrite set, Test strategy #1's allowed diff set, and Rollout 2's move-commit
  contents to the 16 depth-sensitive `new URL(..., import.meta.url)` sites (`'../prompts'` →
  `'../../prompts'`, `'../../../.opencode/...'` → `'../../../../.opencode/...'`), recite them in
  the bullet-2 amendment alongside the comment clause (e.g. "a relative path reference — code or
  comment — naming a location this move passes takes its new depth"), and replace ¶2's clause
  and A8's basis with the checked enumeration: grep `\.\./` = 87 sites = 65 from-imports (62 in
  moved files) + 2 cli.test.ts non-import `../src/` sites + 16 `new URL` sites + 4 inert
  `'../escape'` fixture strings.

## FYI (no severity)

- ADR-004's own Context premise — "the tests never import each other or reference each other's
  paths. The move is a rename plus the import depth" (ADR:27-28) — has the same blind spot as the
  proposal's A8: it never contemplated `import.meta.url`-relative references. Whoever drafts the
  move commit's one-clause ADR amendment may want its premise line corrected too; that is the
  ADR's text, not this proposal's, so it is not counted as a finding here.
