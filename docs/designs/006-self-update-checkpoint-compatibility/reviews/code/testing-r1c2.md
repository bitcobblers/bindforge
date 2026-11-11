# Code review — testing seat — round 1, ratification pass c2

Scope as given: `git diff 640c289bdd01611c1fcd07a504af78eb59fb3fc7..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/006-self-update-checkpoint-compatibility.md`.
Task: confirm that revisions landed after the r1c1 approval did not break what that pass approved.

## Scope verification

`git log 640c289b..HEAD` lists exactly one commit, `2d49432 docs: 006-self-update-checkpoint-compatibility
code-review stage`, and its only contents are run records under `docs/designs/` (`.pipeline.json`,
review reports, transcripts) — excluded from the diff by the caller's pathspec. Therefore:

- `git diff 640c289b..HEAD -- ':(exclude)docs/designs'` is **empty**.
- `git diff 640c289b..HEAD -- tools packages src` is empty, and `git diff HEAD -- tools packages`
  (working tree vs HEAD) is empty: the source and test tree is byte-identical to the state
  (`640c289b fix: stop committing the run lock and honor runner-authored deletes in the
  ownership sweep`) that r1c1 reviewed and approved.

FYI (no severity): the caller's stated premise that revisions landed after the r1 approval does not
hold for code — the only post-approval commit is docs-only. The r1c1 findings themselves reference
`stageAll` and the `holdsRunnerBytes` ordering introduced by `640c289b`, so that commit was already
the reviewed head of r1c1. I verified this directly rather than assuming it, and re-checked each
anchor below.

## Checklist

1. **Test strategy citations** — unchanged since the approved state; the proposal's #1–#13 pins are
   all present (e.g. `gates.test.ts:330` #1, `ownership.test.ts:237,251,265,283,298` #2,
   `checkpoint-compat.test.ts:38,51,66,79` #3, `state-store.test.ts:240,262` #4/#5,
   `checkpoint-writer.test.ts:126,140,162` #6, `runner.test.ts:306,328,360` #8/#9/#10,
   `checkpoint-integration.test.ts:208,242,280` #11/#12/#13). PASS.
2. **Unit + integration layers** — both present, unchanged. PASS.
3. **Revision-round pins** — the accepted r1 fixes (checkpoint trust, escalation marker, `'wx'`
   lock, lock unstaging, delete-outranks-write) are exactly the code of `640c289b`, which r1c1
   audited; no later commit touched them. Nothing regressed. PASS.
4. **Determinism** — no test file changed; the suite run shows the EXDEV mount probe self-skipping
   (2 skipped) rather than failing. PASS.
5. **Behavior-named tests** — no renames since the approved state. PASS.
6. **`npm run check` in `tools/orchestration`** — green. Tail:

   ```
    Test Files  33 passed (33)
         Tests  357 passed | 2 skipped (359)
      Duration  2.48s (tests 75%, import 19%, transform 5%, worker 1%)
   ```

7. **Untestable additions** — none appeared; the diff adds no source at all. PASS.

## Re-check of r1c1 findings (all unchanged, none blocking)

- Minor #1 (`runner.test.ts:408-421`): still the proxy test — pre-written live holder, refusal,
  byte-identity; no literal two-concurrent-acquisition test. Still accurate, still minor.
- Minor #2 (`runner.ts:536-574`, `runner.ts:353-356`): no test reaches any `ensureLockHeld` throw
  (grep for "lost the pipeline lock" in tests: no hits), and no fixture scans committed history for
  `.pipeline.lock`; `checkpoint-integration.test.ts:83-85` still gitignores the lock. Unchanged.
- Minor #3 (`runner.ts:2783-2792`): the finalizeDone `CheckpointWriteFailure` guard still has no
  done-boundary test; the only directory-sabotage case remains mid-run
  (`checkpoint-integration.test.ts:279-301`). Unchanged.
- Nit #4 (`ownership.test.ts:390-405`): the marker-delete unit case still does not
  `recordRunnerWrite` the marker's bytes, so the delete-outranks-write ordering remains pinned only
  through the integration second-resume leg. Unchanged.

## Verdict

The range under review changes no source or test path, so nothing the r1c1 pass approved can have
been broken; the suite is green and all r1c1 minors/nit stand exactly as reported (open, not
blocking). **approve**.
