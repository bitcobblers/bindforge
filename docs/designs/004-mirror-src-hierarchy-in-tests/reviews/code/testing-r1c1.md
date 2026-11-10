# Code review — testing seat — dissent pass c1 (round 1 dissent)

Scope: `git diff 1a66bda6a543508cf60bf792107a6d8b38bbfeee..HEAD -- ':(exclude)docs/designs'`
read against the current `02-proposal.md` and ADR-004. The diff touches one
source/test path: `tools/orchestration/tests/pipeline/gates.test.ts`
(+24/−19), landed by `93032c6 test(orchestration): keep tree-mirror pins as
subset invariants, drop transient counts`; the fix-stage commit `5b32468`
changed only `docs/designs/` (excluded, but its proposal/rollout wording is
review input and I read it).

## Prior findings — checked first, per the dissent pass

1. **major — `gates.test.ts:383-384,394` (exact 26-file set equality + `toHaveLength(23)`)** — **RESOLVED.**
   The sorted full-set `toEqual`, the `toHaveLength(23)`, and the second
   exact-set `toEqual` are gone. The test
   (`tracks every moved and root-tier test file, and fails closed on leftover flat names`, :385-389)
   now keeps the fail-closed zero-flat-name listing check (:387, unchanged) and
   asserts the 23 moved + 3 root-tier paths as `toContain` subset members
   (:388). A future `tests/<newsubsystem>/x.test.ts` stays green; a leftover
   flat file or a lost moved file still fails. The proposal was amended in
   lockstep: Test strategy #2 (lines 149-156) now names the zero-flat-name
   listing plus the *subset* as the lasting assertions and marks exact
   26/228 as Rollout 3's reported acceptance evidence; revision log line
   220-225 records the change. This is exactly the prescribed fix.

2. **minor — `gates.test.ts:392-399` (test named as vitest selection never invoked vitest)** — **RESOLVED** via the rename option, which the proposal then sanctioned. The test is renamed `simulated tests/pipeline selection matches every moved file and no root-tier test` (:397), its comment states the simulation explicitly (:391-396), and proposal Test strategy #3 (lines 157-160) now says the reported selection is the proof while the suite's assertion simulates the A6 filter. The simulation's own assertions are also subset-based now (`selected ⊇ nested`, root-tier excluded), so it stays true when new mirrored files arrive — and remains faithful to A6: a future `tests/pipeline-*`-nested dir would in fact be substring-selected by vitest. I re-verified the real command at HEAD: `npm test -- tests/pipeline` selects 23 files / 190 tests. No false claim remains under either name.

3. **minor — Rollout 1 wording vs landed history (test-only commit added cases; pins red until the move)** — **RESOLVED** in both directions. Proposal Non-goals (lines 20-27) and Rollout 1 (lines 176-183) now record what landed: the test-only commit keeps the 26-file count, adds cases (228 tests), and its tree pins are red by construction until the move commit; the "test and file counts unchanged" clause is corrected. The `gates.test.ts:330-337` baseline comment now words the same thing ("adds cases… acceptance measurement… not a lasting suite invariant… what below stays permanent: the zero-flat-name listing check plus the subset assertions"), replacing the divergent "test count unchanged" baseline.

## Testing checklist pass over the changed code

1. **Test strategy citations:** #1/#2 at `gates.test.ts:379-384`, #3 at
   `:391-396`, #4 at `:302/:312` (unchanged block), #6 at
   `prompts.test.ts:319/:330` (outside this diff, still present). #5/#7 are
   run-level (check + platform), not suite assertions. OK.
2. **Layers:** unit (subset tracking) and integration-simulation (#3) both
   present as amended; promised integration runs (#2/#3/#5 real commands) are
   reported acceptance evidence per the amended proposal. OK.
3. **Revision round:** the previously rejected major's lasting-invariant form
   (subset + zero-flat fail-close) is now the test itself; nothing else in the
   tree pins transient counts (grep: `228`/`26` appear only in comments). OK.
4. **Determinism:** `git rev-parse --show-toplevel` + `git ls-files`
   (`:370-377`); no clock, network, ports. Same as r1. OK.
5. **Names describe behavior:** both renamed tests read as behavior
   statements, and the "simulated" name no longer overclaims. OK.
6. **`npm run check` executed at HEAD — green.** Tail:

   ```
   Checking formatting...
   All matched files use Prettier code style!
    Test Files  26 passed (26)
         Tests  228 passed (228)
      Start at  10:30:04
      Duration  3.03s (tests 51%, import 37%, transform 11%, worker 1%)
   ```

7. **Untestable additions:** none — the diff only relaxes pins and rewords
   comments; no new knobs or dead branches.

## Verdict

All three round-1 findings are resolved by `93032c6` plus the proposal's
recorded amendments; the changed code introduces nothing new. No open
blocker or major findings.

**Verdict: approve** (0 findings).
