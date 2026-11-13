# Compatibility review r1

verdict: approve

Scope: whole document (full-panel pass c0, round 1). Inputs: 02-proposal.md,
docs/adrs/020-restart-boundary-for-harness-self-changes.md, the Revision log.
Code claims verified read-only (shell was denied this session; the read tool
was used on the named source files instead): package.json engines
`>=22.23.5` matches the matrix row (30–31); cli.ts exit mapping
`done→0 / other→2 / throw→1` and the header comment match A5 exactly
(cli.ts:17, 214, 227); `StateSchema` is a plain `z.object` under the ADR-006 §2
grammar comment (state-store.ts:58–79, 80); `stagePathAllowed`'s coder-window
confinement matches A3 (gates.ts:559–589); `finalizeStage`/
`finalizeReviewPass` receive `headBefore` and mint the window's commit
(runner.ts:1848–1892, 1940–1961); `finalizeDone`'s terminal-save guard and
save-before-commit posture match §2's citations (runner.ts:3377–3425,
3395–3396); the resume split-record posture §2 mirrors is at 2345–2378; the
drive-loop insertion point is real (2440–2441); `planStopStatusLines` sits at
3580; `loadTemplate`/`templateHash` back A1/A8 (prompts.ts:81–87) and
`PROMPTS_DIR` resolves relative to the CLI module (cli.ts:20). No CI workflow
exists (matrix basis verified); the era-fixture dir Test #2 names exists.

Checklist walk:
1. Matrix: all five rows carry a status (yes / partial / no-unverified); no
   `unknown`, no empty cell. Pass.
2. OS input APIs: the design touches none (git plumbing + fs + zod only); the
   abstraction is justified in the matrix's macOS/Windows notes. Pass (n/a).
3. Node floor: engine floor claim verified against package.json; no primitive
   above it is introduced ("No new primitive" claim holds — spawnSync git,
   zod optional field). Pass.
4. Native modules: no new dependencies; git is an inherited CLI dependency,
   documented in the matrix notes. Pass.
5. Keyboard reality: n/a — no input surface in this design. Pass (n/a).
6. File paths: the gates.ts predicate contract ("paths reach it only as git
   plumbing output — repo-root-relative, POSIX separators, the real
   spelling", gates.ts:554–557) is exactly what §1's command form breaks.
   Finding 2.
7. Platform test per supported row: the only "yes" platform row (Linux) is
   covered by the named Linux suite run in the Platform paragraph; macOS
   partial / Windows no are recorded as not exercised. Pass.

## Findings

- [minor][new] Assumption register A7 + Compatibility matrix "A pre-020
  runner resumed onto a paused checkpoint" row + Test strategy #2 —
  "a payload with an unrecognized sibling key parses with it stripped (A7's
  posture)" — `loadState` validates with `StateSchema.safeParse` but then
  returns `parsed as PipelineState`, the raw document (state-store.ts:239–256):
  unrecognized keys are *not* stripped from what the loader hands back, and an
  older runner's next `saveState` writes them straight back (its JSON.stringify
  is of the raw state, not of the parse output). The behavior the design leans
  on survives — validation accepts the new key, `version` stays 1, an old
  reader's code never reads it — but "stripped" states a mechanism the loader
  does not implement, and Test #2 as written asserts stripping of the loader's
  return, which fails against the current code. Fix: reword A7 and the matrix
  row to "the loader's validation ignores the unknown key (it returns the raw
  parsed document, so the record harmlessly round-trips through older
  runners)", and change Test #2's clause to assert the load succeeds with the
  sibling key present and behaviorally inert (or assert stripping against
  `StateSchema.safeParse` output, not the loader's return).

- [minor][new] Chosen design §1 — "runs `git diff --name-only
  <headBefore>..HEAD`, filters it through the predicate" — the command form
  drops the pins every other name-list plumbing call in the runner uses:
  `status --porcelain=v1 -z` (runner.ts:268) and `diff-tree -r
  --no-commit-id -z --name-only` (runner.ts:308). Without `-z`, git's default
  core.quotepath emits non-ASCII paths quoted and octal-escaped — the line
  starts with `"` and the leading-slash prefix match silently misses, exactly
  the silent-miss class this design exists to end; with `diff.renames`
  default-on, a rename crossing either prefix collapses to one shown line and
  one side can go unseen. Fix: name the canonical form in §1 as
  `git diff --name-only -z --no-renames <headBefore>..HEAD`, split on NUL, and
  feed each path to the predicate — the shape the ownership sweep already
  uses.

## FYI (no severity)

- ADR-020 Decision 2 says the marker names "the stage that will run next";
  §2's record (`restartRequired = { stage, paths, at }`) and §3's status line
  ("the stage that opened the boundary") name the *completed* stage. ADR
  fidelity is not on this seat's checklist — flagging for the correctness
  seat to reconcile one of the two.
- The §3 seam widening (`runPipeline`/`resumePipeline` returning
  `{ status, restartRequired? }`) breaks roughly fifteen existing test files
  that compare the return with `.toBe('escalated')`/`.toBe('done')`; Rollout
  #1's "Callers of runPipeline/resumePipeline adapt to the widened result at
  the seam" covers it, but the seam spans the whole suite — churn size is the
  complexity seat's business.
- Test #1's expected-false list (`tools/orchestration/package.json`,
  `tools/orchestration/tests/**`) sits just outside §1's two prefixes; the
  honesty test (#7) is what keeps that boundary honest over time — no action,
  just noting the interlock reads sound.
