# Code review — correctness — round 1, pass c0 (full panel)

Scope: `git diff d5092f39755c4a0903dfd19207e4951dfc1cd33a...HEAD -- ':(exclude)docs/designs'`.
Inputs: that diff, `02-proposal.md`, `docs/adrs/004-mirror-src-hierarchy-in-tests.md`,
`03-build-notes.md`. First pass for this seat: no prior findings to re-check.

Method note (no shell in this seat): the diff was verified against the pre-move tree
state read out of the `main` checkout (`/home/mike/repos/bindforge`, `0b4c81d`, whose
`tools/orchestration/tests/` is byte-for-byte the pre-move baseline for this range), plus
per-file line counts, per-file `describe`/`it` inventories, and grep enumerations over
both trees. Every static claim below is reproducible that way.

## Checklist

**1. Components/interfaces named in the proposal exist and match — PASS.**
- ¶1: all 23 files are at `tools/orchestration/tests/pipeline/<name>.test.ts`, the names
  one-for-one with ¶1/ADR:33-38 (`budget` … `states`); `cli`, `opencode-runner`,
  `opencode-runner-timeout` are still at the root tier; no `pipeline-*.test.ts` file
  remains anywhere under `tools/orchestration/tests/` (glob over the whole tree).
- Rollout 1's assertions exist where Rollout 1 put them: `pipeline/gates.test.ts:295-327`
  (Test strategy #4: the real post-move path `tools/orchestration/tests/pipeline/gates.test.ts`
  denied to `implement`/`fix`, allowed to `test-authoring`/`test-update`) and
  `pipeline/gates.test.ts:329-397` (#1/#2/#3: `MOVED_NAMES` = ¶1's 23 names, the
  `git ls-files` listing check, the 23-nested + 3-root equality, the `tests/pipeline`
  substring selection). `pipeline/prompts.test.ts:314-348` carries #6.
- ¶4: `.opencode/skills/repo-practices/SKILL.md:21` now reads `tests/**/*.test.ts` with the
  naming rule stated once in `## Tests` (27-30); `.opencode/skills/test-strategy/SKILL.md:27`
  carries the glob only.
- Rollout 2's ADR clause is in place: `docs/adrs/004-…:42-45` carries exactly the clause
  ¶2 quotes ("a relative path reference - code or comment - naming a location this move
  passes takes its new depth"). ADR Context's "14 modules" is untouched, as Deferred says.

**2. Assumption register honored — PASS.**
A4 (tsconfig unchanged, byte-identical to the pre-move file: `rootDir: "./src"`,
`include: ["src/**/*.ts"]`, `exclude: [..., "tests"]`), A5 (no `vitest.config.*` /
`vite.config.*` in the repo), A7/A19 (`gates.ts:253` `TEST_PATH_RE`, `:260`
`stagePathAllowed`, `:262` consumption — identical lines and text in both trees; no `src/`
pattern keys on the flat `pipeline-*` shape: grep for `pipeline-…\.test\.ts` over
`tools/orchestration` hits only the deferred fixture string), A9 (`package.json` scripts
byte-identical; `check` still chains lint → format:check → test), A11
(`.opencode/agents/code/test-engineer.md:9` = `**/tests/**`), A12
(`prompts/test-authoring.md:3` still directory-generic), A8's closed enumeration holds on
both sides: pre-move 65 `from '../…'` + 2 `cli.test.ts` non-import sites + 16 `new URL` +
4 `'../escape'` = 87; post-move 62 + 16 + 4 = 82 inside `tests/pipeline/`, plus the three
root-tier files' own `../src/` (`cli.test.ts:4,29,30`, `opencode-runner.test.ts:7`,
`opencode-runner-timeout.test.ts:5`) which ¶1 does not move. A16/A17/A18 are unverified in
the proposal and stayed unverified — not silently upgraded.

**3. Deviations are declared — PASS.** Deviations 1–4 in `03-build-notes.md` (move is
test-owned work, ADR clause is runner-written, one commit became three, no rename primitive
in any stage) match the landed history: the Rollout-1 assertion commit
(`test: pin the mirrored test tree…`) precedes the skill-file commit
(`docs: adopt the recursive test-tree convention…`) and the operator's
`refactor: mirror src/ hierarchy in orchestration test tree`. No undeclared gap found —
see the nit below for the one place the declaration is thinner than the diff.

**4. Scope creep — none found.** Nothing in `src/`, `tsconfig.json`, `package.json`,
`eslint.config.js`, the prompts, or the agent grants changed (compared against the
pre-move tree). No behavior beyond ¶1/¶2/¶4 + the sanctioned ADR clause.

**5. Build-notes claims true (statically checkable ones).** The post-move re-verification
(`03-build-notes.md:550-575`) checks out: 82 `../` sites in `tests/pipeline/` with the
exact split it names — 62 `from '../../src/…'`; 16 `new URL` sites (10 `'../../prompts/'`,
the no-trailing-slash `'../../prompts'` at `fix-trigger.test.ts:8`, and five
`'../../../../.opencode/skills/…'` reads at `prompts.test.ts:215, 255, 266, 280, 305`, the
last being the multi-line `template.md` argument); 4 `'../escape'` fixtures at
`identity.test.ts:65,68,103` and `state-store.test.ts:156`. The A14 comment reads
`tests/pipeline/reply-contract.test.ts` at `parallel-escalation.test.ts:45`. The deferred
fixture string `tests/pipeline-lane.test.ts` at `independence.test.ts:145` is untouched.
`states.ts:173-174` does route `fix` → `test-update` on `testsNeedUpdate`, as cited.
The one claim this seat could not evaluate is the executed gate — see finding 1.

**6. Error paths fail closed — PASS.** #2's check is a tracked-file listing, not a content
grep: `gates.test.ts:382` expects `[]` for `/(^|\/)pipeline-[^/]*\.test\.ts$/`, so a
half-applied move (flat file still tracked, or a nested copy alongside it) fails rather
than passes; `:383` pins the full 26-path inventory and `:384-386` the 23 nested count.
The pre-existing `stagePathAllowed` expectations are preserved verbatim (compared against
the pre-move block at `pipeline-gates.test.ts:294-315`): nothing was relaxed to make the
suite green.

**7. Revision rounds — n/a** (first code-review pass for this seat; the round-1 design
findings it implements are present: listing-based fail-closed check #2, real-path
classification #4, both convention surfaces ¶4).

## Content-fidelity evidence (Test strategy #1's diff half, which the build notes defer
to this review)

- Line counts, pre vs post, are identical for 20 of the 23 moved files. The three deltas
  are fully explained: `proposal-size.test.ts` +4 (the 100-column import
  `import { escalationMarkdown, proposalGrowth, proposalGrowthLine } from '../../src/…'`
  re-wrapped into `proposal-size.test.ts:10-14`), `gates.test.ts` +82 and
  `prompts.test.ts` +45 (Rollout 1's additions; the +1 offset in `prompts.test.ts` is its
  new `execFileSync` import). Root-tier files: unchanged counts.
- The `describe`/`it` inventory is identical name-for-name across the moved set: 233
  matches pre-move in the 23 files vs 239 post-move, the +6 being exactly Rollout 1's
  two new `it`s in `pipeline/prompts.test.ts`, two in the new `describe('test tree mirrors
  src/ hierarchy')`, and those two describes. No test name, count, or fixture was added,
  renamed, or weakened by the move.
- Byte-level spot check: `pipeline-stage-lifecycle.test.ts` (pre) vs
  `pipeline/stage-lifecycle.test.ts` (post) differ only at lines 10-12 (`../../src/`) and
  108 (`../../prompts/`).

## Findings

1. **minor** — `docs/designs/004-mirror-src-hierarchy-in-tests/03-build-notes.md:577-586`.
   The acceptance evidence the proposal makes load-bearing (Rollout 3, Risk 4: `npm run
   check` green, 26 files / 228 tests, 23/190 for the nested selection) is recorded as
   prose from the test-update stage and cannot be reproduced by any review seat — no shell
   is granted here. Everything that would make it red was checked statically and is clean,
   but the numbers themselves are unverified at the merge commit.
   Fix: land the vitest reporter tail (`Test Files 26 passed`, `Tests 228 passed`, and the
   `npm test -- tests/pipeline` 23/190 line) verbatim into the build notes at the merge
   commit, so Rollout 3's acceptance evidence is an artifact rather than a recollection.

2. **nit** — `tools/orchestration/tests/pipeline/proposal-size.test.ts:10-14` and
   `tools/orchestration/tests/pipeline/prompts.test.ts:214-216, 254-256, 265-268, 279-281,
   304-306`; record at `03-build-notes.md:550-563`.
   Six of ¶2's rewritten sites exceeded `printWidth: 100` after gaining `../`, so prettier
   re-wrapped them: the import statement became five lines and the five `.opencode/skills`
   reads became three-line `fileURLToPath( new URL(…) )` blocks. The behavior is forced by
   the `format:check` half of the gate and is semantically inert (verified: no other line in
   those files differs), but Test strategy #1 words the allowed set line-per-site and the
   build notes' post-move enumeration says only "nothing outside it touched", so a later
   diff review counting changed lines will see more than 62+16+1.
   Fix: one clause in the build notes naming the re-wrapped sites as prettier output of the
   ¶2 rewrite, so the allowed set reads as "these statements, formatted however prettier
   requires".

## Verdict

approve — no open blocker or major finding.

## FYI (not scored, not this seat)

`gates.test.ts:393` models `vitest run tests/pipeline` as a substring filter over the
tracked listing; a leftover flat `tests/pipeline-gates.test.ts` would also contain
`tests/pipeline`, so that assertion alone does not discriminate the failure mode — `:382`'s
regex does. The build notes also record the real `npm test -- tests/pipeline` run
(23 files / 190 tests) alongside it, so the intent is covered.
