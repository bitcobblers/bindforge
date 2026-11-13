# Code review — testing seat, round 2 pass c2 (ratification)

Scope: `git diff 6a5c454..HEAD -- ':(exclude)docs/designs'`, read against the full
current `02-proposal.md`. This is the ratification pass: the c2 fixes (written
against other seats' r2c0 findings) are checked for having broken what round 2
approved, plus the fixes' own test coverage. Six findings from my r2 pass are
carried forward with their status.

## Check

- `npm run check` in `tools/orchestration` (my own run, this checkout):

  ```
   Test Files  51 passed (51)
        Tests  570 passed | 2 skipped (572)
     Start at  14:48:51
     Duration  7.65s (tests 89%, import 8%, transform 2%)
  ```

  Green, consistent with the runner's gate record (npm check, cargo fmt/clippy/
  test all green). No blocker on item 6.

## What the c2 fixes changed, from the testing side

Checkpoint fence on `.pipeline.json` (17 agent configs, `STATE_FILE_SEGMENT` in
the audit, agent-shell + independence test blocks); write-time citation rule
`citableTail`/`citableCwd` in `boundedGateEvidence`; renderer fence discipline
(`$ `/`| ` prefixes, `defuseFences`, reserved closing fence); `greenSummary`
single-clause green line; `sessionTokens` one-level flattening (`cache.read`);
`docName` on size records with the `proposalGrowth` aggregate filter; the
`boundedTail` elision equation; the `design-review.md` rewording with its test
pin updated.

Nothing I approved in r2 is broken: `docName` is optional per A6 and the golden
checkpoint fixtures still load; the strategy citations #1–#14 all still have
tests; the agent-rule matrices gained coverage rather than losing it; prompt
render tests still pin the stand-in line, the cap, failing-legs-first, and now
the line-selective build-note sentence; the `- parameters:` line is untouched.
Security findings 1–3 and correctness findings 1, 2, 5 have tests that name
them and would catch the original deed (dirty-leg integration, checkpoint
fence ×3, fence-containment + cap tests, token fixture, green-line test).

## Findings

1. **minor — `tools/orchestration/src/pipeline/report.ts:186` + `tests/pipeline/proposal-size.test.ts:303–398`**
   The aggregate fix (correctness r2c0 finding 3: research counts must not fold
   into the `proposal:` line) ships with no test. No fixture anywhere carries
   `docName: RESEARCH_DOC` (grep: `docName` appears in tests only in a comment),
   and the research-meter test (`proposal-size.test.ts:464`) asserts `docLines`
   but not the stamped `docName`. Deleting the `&& r.docName !== RESEARCH_DOC`
   clause keeps the whole suite green — the mutation that reproduces the
   accepted defect survives. **Fix:** add a `proposalGrowth` case whose results
   mix proposal-size records with one `docName: RESEARCH_DOC` record and assert
   the aggregate counts only the proposal ones; assert the research stage's
   result carries `docName`.

2. **minor — `tools/orchestration/src/pipeline/gates.ts:349–367` (boundedTail equation)**
   The elision-equation rewrite (correctness r2c0 finding 4: the marker must
   state the true elision) is not pinned: no test greps for the note text
   (`characters elided by the gate tail cap` — zero matches in the suite). The
   `big` capture fixture (`gate-evidence-capture.test.ts:45–46`) emits a known
   4 009-char stream but asserts only length ≤ 2 000 + `TAIL_END`; the *old*
   fixed-64 reservation (stating 2 071 instead of 2 056) passes every current
   assertion. **Fix:** in the `big` test, match
   `/^\[(\d+) characters elided by the gate tail cap\]/` and assert the stated
   number equals 4 009 − (tail.length − note.length), i.e. the marker's count
   and the kept span are one equation.

3. **nit (new) — `tools/orchestration/src/pipeline/runner.ts:3188–3197` (greenSummary) vs `tests/pipeline/gate-evidence-integration.test.ts:47`**
   The "one citable line" fix is pinned to the *last* `test result:` summary —
   and every fixture (unit and the strategy-#10 integration) emits exactly one
   summary per leg. The repo's real `cargo test --workspace` tail ends with the
   doc-tests summary (`test result: ok. 0 passed` — visible in the runner's own
   gate record), so the line a seat actually gets cited for this very change is
   the least informative one. The single-summary fixture cannot tell "the
   summary behind the behavior claim" from "the last summary in the flatten".
   **Fix:** make the integration green script emit several per-target summaries
   ending with a 0-passed doc-test line (as real cargo does), and assert the
   chosen citable clause — either pin the last-summary choice knowingly, or
   prefer the last summary with nonzero counts.

4. **nit (new) — `tools/orchestration/src/pipeline/runner.ts:3113–3144, 3193–3196`**
   Three new helper branches have no test: `citableTail` all-lines-dropped
   (marker-only return), `citableCwd` `(cwd withheld)` (exported behavior, never
   asserted), `greenSummary`'s fixed-window fallback when ` finished in ` is
   absent. Checklist item 7: untested branches are findings. **Fix:** three
   one-line unit cases in `gate-evidence-render.test.ts`.

5. **minor (carried, unchanged by c2) — `tools/orchestration/tests/pipeline/gate-evidence-capture.test.ts:203` (Test strategy #14)**
   The strategy sentence still reads "the cargo legs run against the real
   workspace in the integration tests that need a green gate"; no test runs a
   real cargo binary (cargo is PATH-stubbed; the integration legs are node
   scripts). The c2 diff did not add the probe-guarded case nor a fixture
   comment recording where the real legs actually run (CI rust job, runner gate).

6. **minor (carried, unchanged by c2) — `tools/orchestration/src/pipeline/gates.ts:401–404` (MAX_TAIL_BYTES eviction)**
   The rolling-buffer eviction loop is still never exercised: the largest
   fixture leg is the 4 KiB `big` mode, far under the 64 KiB window. A
   wrong-end drop or miscounted kept bytes would corrupt exactly the tails
   behavior claims cite and pass the suite.

7. **minor (carried, partially mitigated by c2) — `tools/orchestration/tests/pipeline/agent-shell.test.ts:340–351`**
   c2 pinned the *checkpoint* read/glob denies across all 17 shell-holding
   agents (`agent-shell.test.ts:427–434`) — but the `*transcripts*`/`*reviews*`
   glob/grep deny loop still covers only `DOC_FAMILIES + DESIGN_SEATS` (10 of
   17); the two CODE_AGENTS and five codereview seats' glob/grep run-record
   denies remain unpinned and could be dropped silently.

8. **nit (carried, unchanged by c2) — `tools/orchestration/src/pipeline/gates.ts:431–447`**
   The signal-killed `exit: null` tail path (`killed by signal …`) still has no
   generator mode that self-kills; only the spawn-failure null case is tested.

9. **nit (carried, unchanged by c2) — `tools/orchestration/src/pipeline/independence.ts:273`**
   The `tokens.length === 0 ||` guard in `shellReachesReviews` is still
   provably dead and still uncommented as defense-against-divergence.

10. **nit (carried, unchanged by c2) — `tools/orchestration/tests/pipeline/gate-evidence-render.test.ts:91–211`**
    Test strategy #2's render-time array re-bound is still unasserted: the
    largest `gateEvidenceText` fixtures use 24 legs (< MAX_GATE_LEGS); only the
    write-time bound (`boundedGateEvidence` direct, 40 legs) is pinned.

## Checklist

1. Strategy items #1–#14: all cited (`proves:` tags present); #14's claim
   overstays the suite — finding 5.
2. Unit + integration layers: both present.
3. Revision rounds: security 1–3 and correctness 1, 2, 5 have caught-it tests;
   correctness 3 and 4 fixes do not — findings 1, 2.
4. Determinism: new fixtures are node scripts and string asserts; no clock,
   network, or port. Pass.
5. Names describe behavior. Pass.
6. `npm run check` green (tail pasted above). Pass.
7. Untestable/dead additions: findings 4, 8, 9 (and the eviction loop, 6).

FYI (no severity, not testing): the `.pipeline.json` fence and the write-time
citation rule now shipped are described in `docs/workflows.md` but not in
`02-proposal.md` §2/§3 or its revision log (that dir is excluded from this
diff) — an editorial alignment the run record may want.

## Verdict

No blocker or major. The c2 fixes did not break what round 2 approved; two of
the fixes landed without pinning tests (both minor), and the six r2 minors/
nits stand where they stood. **approve**.
