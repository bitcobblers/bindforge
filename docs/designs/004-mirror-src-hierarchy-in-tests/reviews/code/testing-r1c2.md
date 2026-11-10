# Code review — testing seat — round 1, pass c2 (ratification)

Scope as given: `git diff 93032c6029dd5d1fbc723270b59cf78dbe1fa20d..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and ADR-004. Ratification check only:
whether fixes landing after the c1 approval broke what was approved. Quoted prior
findings for this seat: none.

## The scoped diff is empty

    $ git diff --name-only 93032c6029dd5d1fbc723270b59cf78dbe1fa20d..HEAD
    docs/designs/004-mirror-src-hierarchy-in-tests/.pipeline.json
    docs/designs/004-mirror-src-hierarchy-in-tests/reviews/code/testing-r1c1.md
    docs/designs/004-mirror-src-hierarchy-in-tests/transcripts/code-review-testing-r1c1.md

    $ git diff --stat 93032c6029dd5d1fbc723270b59cf78dbe1fa20d..HEAD -- ':(exclude)docs/designs'
    (no output)

The only commit in the range is `deaf37f docs: 004-mirror-src-hierarchy-in-tests
code-review stage`, and it touches only `docs/designs/` run records. **Zero source
or test paths changed since the approved baseline `93032c6`.** There is therefore no
post-approval change that could break the approved state.

## Sanity pass: current tree still matches the current document

Since the pass is to be read against the full current document, I confirmed the
document's test-strategy claims still hold at HEAD (this is the state c1 approved;
`93032c6` itself, the "keep tree-mirror pins as subset invariants, drop transient
counts" fix, matches the revision-log line "code-review r1, testing, major + minor"
and the amended #2/#3 wording verbatim):

- Layout (¶1, A1/A2): 23 files under `tools/orchestration/tests/pipeline/` with the
  prefix dropped, 3 root-tier (`cli`, `opencode-runner`, `opencode-runner-timeout`);
  no flat `pipeline-*.test.ts` files remain (glob of the test tree).
- Test strategy #1/#2: `gates.test.ts:379` `// proves: Test strategy #1 ... and #2's
  lasting assertions` — zero-flat-name listing check kept; the 23 moved + 3 root-tier
  paths asserted as `toContain` subset; the exact 26-file equality and the hard-coded
  `toHaveLength(23)` are gone, matching the amended "not a permanent suite invariant"
  wording.
- Test strategy #3: `gates.test.ts:391` — simulated path-substring selection over the
  tracked listing (no nested vitest spawn), asserting every moved file selected and no
  root-tier file — matching the amended "#3 check simulates A6's filter".
- Test strategy #4: `gates.test.ts:299-312` — `stagePathAllowed` asserted on the real
  post-move path `tools/orchestration/tests/pipeline/gates.test.ts`, allowed for
  test stages, denied to coder stages.
- Test strategy #6: `pipeline/prompts.test.ts:319,330` assert both convention
  surfaces; an independent grep finds no remaining `tests/*.test.ts` flat form in
  `.opencode/skills/`.

## Testing checklist walk

1. **Numbered strategy items cited** — #1, #2, #3, #4, #6 carry `// proves: Test
   strategy #N` at the sites above; #5 is the `npm run check` gate (see item 6 below)
   and #7 is a platform statement, neither is a test-citable assertion. Pass.
2. **Unit + integration layers as promised** — unit (`stagePathAllowed` classification,
   depth-rewrite behavior implied by the passing moved suites) and integration (tracked
   listing, simulated selection, static convention grep) both present. Pass.
3. **Revision rounds** — the seat's r1 major+minor fix (subset pins, dropped transient
   counts) is itself a change to the very tests that would have caught it; it is inside
   this pass's baseline and consistent with the current document. No further rejected
   findings quoted for this seat. Pass.
4. **Determinism** — the touched block uses `git ls-files` over the tracked tree and
   pure-function assertions; no wall-clock, network, live server, or ports. Pass.
5. **Names describe behavior** — `tracks every moved and root-tier test file, and fails
   closed on leftover flat names`; `simulated tests/pipeline selection matches every
   moved file and no root-tier test`. Pass.
6. **`npm run check` executed by this seat** — **could not run.** This seat's sandbox
   rejects all commands except `git log`/`git diff`; both attempts were denied:
   `npm run check` → `{"error":{"type":"permission.rejected","message":"Permission
   denied: shell"}}` (likewise `npm --prefix tools/orchestration run check`). Per the
   checklist this un-evaluated item is a finding, not a pass; recorded below as a nit
   because the diff contains zero code changes and the checklist itself notes "the
   runner verifies independently."
7. **Untestable additions** — none: no source, config, or dead-branch additions in the
   scoped diff (it is empty). Pass.

## Findings

- **[nit] Checklist item 6 — gate not executed in this seat (sandbox limitation, not a
  code defect).** Problem: this c2 seat could not run `npm run check` (tool permission
  denied), so green-gate evidence at HEAD is not this seat's own observation. Fix:
  rely on the pipeline runner's independent gate reproduction (the checklist's stated
  backstop); no builder action.

## FYI (out of specialty, no severity)

The zero-flat-name regex `/(^|\/)pipeline-[^/]*\.test\.ts$/` (gates.test.ts, unchanged
in this pass's baseline) would also flag a future legitimately nested file whose name
begins `pipeline-` (e.g. `tests/pipeline/pipeline-x.test.ts`) as a leftover. Not made
true by this pass's diff.

## Verdict

approve — the scoped diff contains no source or test changes after the approved
baseline; nothing the c1 pass approved has been broken.
