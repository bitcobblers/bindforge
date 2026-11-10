# Code review — correctness seat, round 1, ratification pass c2

Scope: `git diff 1a66bda6a543508cf60bf792107a6d8b38bbfeee..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and ADR-004. Ratification only:
did the post-approval revisions break what c0 approved? No shell was granted,
so verification is static (reads + greps); the verdict notes what could not be re-executed.

## Previous-seat findings — status

- **nit (prettier re-wraps vs the allowed set): resolved.** The clause landed at
  `03-build-notes.md:717-721` ("those statements, in whatever shape prettier
  requires"), and the code shapes match it: `proposal-size.test.ts:10-14` is the
  five-line `report.js` import; the five `.opencode/skills` reads are
  three-line `fileURLToPath` blocks at `prompts.test.ts:214-216, 254-256,
  264-268, 279-281, 304-306`. No content outside those statements changed.
- **minor (reporter tail as artifact): still open, unchanged by the revisions.**
  The build notes acknowledge it (`03-build-notes.md:713-716`: "stays
  operator/next-stage evidence"). It is worse-adjacent but not broken: see the
  carried finding below.

## What the post-approval diff contains (source/test paths)

The only in-scope payload is the test-update rewrite of the
`describe('test tree mirrors src/ hierarchy')` block in
`tools/orchestration/tests/pipeline/gates.test.ts:329-402` — exact-inventory
comparisons replaced by the fix r5-prescribed subset comparisons (#1 below).

## Checklist

1. **Named components exist and are wired.** Tree shape is intact: exactly 23
   files under `tests/pipeline/` matching ¶1's names one-for-one, plus the 3
   root-tier files; the `test-tree convention` block sits at
   `pipeline/prompts.test.ts:314-348`; both skill surfaces carry
   `tests/**/*.test.ts` (`repo-practices/SKILL.md:21`,
   `test-strategy/SKILL.md:27`).
2. **Assumption register honored.** A5 re-checked at HEAD: no
   `vitest.config.*`/`vite.config.*` in `tools/orchestration`. A7:
   `TEST_PATH_RE` unchanged at `gates.ts:253`, consumed at `gates.ts:262` and
   `runner.ts:1031`. A8's enumeration re-grepped at HEAD: 62
   `from '../../src/` imports in moved files, exactly 16 `new URL(` sites all
   at the new depth (10 `'../../prompts/'`, the no-trailing-slash
   `'../../prompts'` at `fix-trigger.test.ts:8`, the five
   `'../../../../.opencode/skills/…'` reads at `prompts.test.ts:215, 255, 266,
   280, 305`), 4 inert `'../escape'` fixtures (`identity.test.ts:65,68,103`;
   `state-store.test.ts:156`), and the 5 root-tier `'../src/` sites untouched
   (`cli.test.ts:4,29,30`; `opencode-runner.test.ts:7`;
   `opencode-runner-timeout.test.ts:5`) — 62+16+4, the build notes' post-move
   "82 sites" claim holds. Nothing outside ¶2's allowed set carries a `../`.
3. **Deviations declared.** Deviations 1–5 are all recorded in
   `03-build-notes.md`; the fix-stage proposal edit is Deviation 5. No
   undeclared deviation found in the in-scope paths.
4. **No scope creep.** The `gates.test.ts` rewrite is licensed by the accepted
   testing-seat major + minor and lands verbatim-in-substance the fix r5
   prescription (`03-build-notes.md:679-702`): same two `it` blocks (test count
   stays 228), zero-flat-name fail-closed check kept (`:387`, regex
   `/(^|\/)pipeline-[^/]*\.test\.ts$/` — fails closed on a half-applied move,
   matches nothing in the new tree), all 26 post-move paths still named
   (`:388`), selection assertion renamed to say "simulated" (`:397`) per the
   amended Test strategy #3 — no over-claim, no nested vitest spawn. src/,
   config, and skill surfaces show no changes in scope.
5. **Build-notes claims true (read side).** The post-move enumeration, the
   prettier clause, and the prescribed-edit match all verified above. The
   green-gate totals remain prose, not an attached reporter tail — finding 1.
6. **Error paths fail closed.** `gates.test.ts:387` still fails closed on any
   leftover flat name; `trackedTestFiles()` throws rather than returns empty if
   git fails, so the subset loops cannot go vacuous.
7. **Previous round's findings addressed.** My c0 findings: nit — addressed;
   minor — explicitly deferred, not broken. No c0 blocker/major existed for
   this seat.

## What could not be evaluated

No shell in this seat: `npm run check` and the 26/228 and 23/190 counts could
not be re-executed at the current HEAD. That gap is exactly the carried
finding below, not a new one.

## Findings

1. **minor (carried from c0, re-scoped).** `03-build-notes.md:577-591` records
   the gate as prose ("26 test files / 228 tests, all 228 passed"; "23 files /
   190 tests") and fix r5 confirms its own shell was denied
   (`:603-606`). Since that last recorded green run, the tree-pin block itself
   was rewritten (`gates.test.ts:329-402`) with no later gate run recorded
   anywhere. The acceptance evidence Rollout 3 makes load-bearing is therefore
   a recollection of the pre-rewrite commit, and the rewritten pins' green
   status at HEAD is asserted only by this session's static reads. Fix: paste
   the vitest reporter tail verbatim (26 files / 228 tests all passed, plus the
   `npm test -- tests/pipeline` 23/190 run and the eslint/prettier lines) into
   `03-build-notes.md` at the merge commit, after the rewrite.

## FYI (no severity)

- `gates.test.ts:338` cites "ADR-004 Decision ¶1"; the ADR's Decision has no ¶
  numbering — the source of the 23-name list is the ADR Decision paragraph /
  proposal ¶1. Comment-only, harmless.

## Verdict

No new blocker or major; the one open minor is unchanged in kind from what c0
approved with. **approve**.
