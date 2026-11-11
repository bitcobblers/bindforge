# Code review — testing seat — r1c2 (ratification)

## Scope and method

Inputs per the caller: `git diff 107c1672b6f33b0cae9117cdc751abb77f75bc68..HEAD
-- ':(exclude)docs/designs'`, read against the full current `02-proposal.md`,
ADR-005, and my own r1c1 findings. Checklist: `references/testing.md`.

**The scope diff is content-empty for source and test paths.** The only commit
in the range is `b1c66ae docs: 005-runner-executed-file-moves code-review
stage`, which touches only `docs/designs/005-runner-executed-file-moves/`
(pipeline.json, r1c1 reports, transcripts). `git diff 107c167..HEAD --
':(exclude)docs/designs'` produces no output. The only other working-tree
delta is an uncommitted `.pipeline.json` (a run record, out of scope by
instruction). So the tree I approved at r1c1 (base `107c167`) is byte-identical
to HEAD for every source and test file under review; nothing later landed that
could have broken it. The r1c1 line citations in my own findings
(runner.ts:1411-1417, lastMovesIssue at 1191 and 1436-1438, the fail-closed
guard at 1068-1069) all still land where I wrote them, confirming my r1c1 read
was against this exact tree.

## Checklist

1. **Proposal Test strategy #1-#13 citations** — unchanged since r1c1: every
   numbered item carries its `// proves: Test strategy #N` pins across
   `reply-contract.test.ts`, `moves.test.ts`, `moves-stage.test.ts`,
   `moves-rollback.test.ts`, `ownership.test.ts`, `budget.test.ts`,
   `prompts.test.ts`, `fix-trigger.test.ts`. Nothing to re-adjudicate; the
   diff adds no new strategy item and removes no test.
2. **Unit + integration layers** — both present and untouched since approval.
3. **Revision rounds** — no blocker/major from my seat was previously
   rejected, and no new code landed that would need a regression pin. No
   previously rejected blocker/major exists in my thread for a test to answer.
4. **Determinism** — unchanged: stub servers, temp repos, no wall-clock or
   network. The `mount: ... must be superuser` line in the check output below
   is the #5 EXDEV probe self-skipping for lack of mount capability, exactly
   the operator-run-skipped case the compatibility matrix and #5 promise.
5. **Test names describe behavior** — unchanged since r1c1.
6. **`npm run check` in `tools/orchestration`, run myself** — green. Tail:

   ```
   All matched files use Prettier code style!
    Test Files  31 passed (31)
         Tests  316 passed | 2 skipped (318)
      Duration  3.48s (tests 73%, import 21%, transform 1%)
   ```

7. **Untestable additions** — my two r1c1 items remain exactly as filed
   (verified against HEAD, not from memory): the exhausted-nudge
   `lastMovesIssue` relabel at runner.ts:1411-1417 is still exercised by no
   test (the only `invalid-reply: moves — ` assertions are pure
   `retryInSameSession`/escalation classification; every sequenceStubServer
   exhaustion stub answers all prompts with parseable replies, so the
   not-JSON-exhaustion branch never carries a moves issue), and the
   no-base-revision fail-closed guard at runner.ts:1068-1069 is still
   unreachable under today's schemas and still untested.

## Verdict

**approve.** The ratification scope contains no source or test change, so
nothing that I approved at r1c1 was broken by later revisions, and the
package's check suite is green under my own run. The two open items below are
carried forward unchanged from r1c1 — both non-blocking — and remain open
because no fix addressed them.

## Open findings (carried from r1c1, unchanged)

1. **minor** — `tools/orchestration/src/pipeline/runner.ts:1411-1417`
   (lastMovesIssue at 1191, 1436-1438): the exhausted-nudge fallback that
   relabels `invalid-reply: not JSON` as `invalid-reply: moves —
   <lastMovesIssue>` is still executed by no test; every moves-rejection stub
   answers each prompt with parseable moves JSON, so exhaustion always exits
   through the parse/schema path. Fix unchanged: a sequenceStubServer case
   with texts `[moves-invalid reply, narration × N]` asserting the attempt
   fails with `invalid-reply: moves — <issue naming the pair>`, not
   `invalid-reply: not JSON`.
2. **nit** — `tools/orchestration/src/pipeline/runner.ts:1068-1069`: the
   fail-closed issue for a moves-bearing reply without a base revision is
   still unreachable under today's schemas and still has no pin (checklist
   item 7). Fix unchanged: pin it at the `movesValidationIssue` boundary or
   record it in 03-build-notes as an intentional fail-closed guard.

FYI (no severity): the c2 scope range as given contains no source/test commit;
if the run intended c2 to review the post-r1c1 fix commits, note those
(`967dd3c`, `0b16fd0`, `17710f6`, …) sit below the range base `107c167` and
were inside the tree my r1c1 already reviewed and approved.
