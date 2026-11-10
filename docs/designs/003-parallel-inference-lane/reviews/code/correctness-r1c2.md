# Code review — correctness — round 1, cycle 2 (ratification)

**Verdict: approve.**

## Scope and method

Caller-given scope: `git diff 3745f3c6d0d0855bf2b77ea2f400a9d16c0b6a6a..HEAD
-- ':(exclude)docs/designs'`, read against the full current
`02-proposal.md` and `docs/adrs/003-parallel-inference-lane.md`.

That diff is **empty for source and tests**: the only commit in the range is
`ec6130a docs: 003-parallel-inference-lane code-review stage`, which touches
only files under `docs/designs` (verified: `git log --oneline` over the range
shows exactly one commit; the filtered `git diff` outputs 0 bytes). The
working tree adds no code change either (only `M docs/designs/.../.pipeline.json`,
the pipeline's own bookkeeping). The code and tests at HEAD are therefore
exactly the state I reviewed and approved in c1 (base `3745f3c`, whose own
changes — formatter-stub test deletion, segment-matching independence tests,
the `updatedAt` ordering assertion in the integration test — were in front of
me in c1, as my quoted findings show by citing the 3745f3c-era line numbers).

Ratification check performed anyway, independently:

1. **Nothing regressed**, because nothing outside `docs/designs` changed.
   Confirmed by re-reading the current files at my c1 finding sites:
   `src/pipeline/independence.ts:104–155` (write fence at 143–149, raw
   `part.name` at 135) and `src/pipeline/runner.ts:839–849`
   (`ownReportPath` wiring, `console.warn` at 846) are unchanged.
2. **Previous-round findings (checklist 7):** my c1 findings were one minor
   and one nit — no blocker/major was open, so c1 approval was not
   conditioned on a fix. Both remain **open and unchanged** (re-verified:
   `ownReportPath` appears only in `runner.ts:842` and `independence.ts:61,111`
   — no test passes it, so the write fence's flagging branch is still
   unexercised; `part.name` is still interpolated un-`printable()` at
   `independence.ts:135`). Carrying them forward verbatim, unchanged severity;
   they do not gate approval.
3. **Build-notes claims at current HEAD (checklist 5):** re-ran the gate —
   `npm run check` in `tools/orchestration` exits 0: eslint clean, prettier
   clean, vitest 224 tests / 26 files green. This matches the third-fix-round
   claim verbatim ("224 tests across 26 files"). The remaining printable()
   wording nit (finding 2 below) is unchanged from c1.
4. **Deviations / assumptions / scope (checklist 2–4, 6):** unchanged from
   c1 — deviations 1–9 in `03-build-notes.md` still declare every gap between
   proposal and code, including the independence-audit subsystem and the
   additive `SeatRecord.findings` field (deviation 7's honest contradiction of
   the "no `.pipeline.json` schema field" sentence), and the diff introduces
   no new undeclared behavior because it introduces no code at all.

## Findings

| # | Severity | Location | Status |
|---|---|---|---|
| 1 | minor | tools/orchestration/src/pipeline/independence.ts:143–149 (tests/pipeline-independence.test.ts) | open, unchanged from c1 |
| 2 | nit | tools/orchestration/src/pipeline/independence.ts:135, 142, 148; runner.ts:846 | open, unchanged from c1 |

### 1. minor — write-fence flagging branch still has no positive test

No test anywhere passes `ownReportPath`, so every write case exercises only
the `own === undefined` skip (`independence.ts:143`) and the
`key !== own → "wrote a review report"` branch (147–148) is unexercised.
Deviation 7's hardening claim is implemented in code but pinned by no test.
Unchanged from c1 — the test-update commit `3745f3c` added the
segment-matching cases, not a fence case.
**Fix:** add a case passing `ownReportPath` with a write/edit/patch part aimed
at a sibling report expecting one violation, plus a negative case writing the
handed path with `ownReportPath` set expecting none.

### 2. nit — `part.name` reaches the terminal un-sanitized

The build-notes claim that every model- and server-authored string reaching
the terminal, gate log, or `ESCALATION.md` passes through `printable()` is
still not quite true for `part.name` (server-supplied tool name interpolated
raw into the violation text at `independence.ts:135`, printed via
`console.warn` at `runner.ts:846`); paths and command text go through
`brief()`/`printable()`. Unchanged from c1.
**Fix:** wrap `part.name` in `printable()` at the violation-construction
sites, or narrow the build-notes wording.

## Checklist result

1. Components named in the proposal exist and are wired — pass (unchanged).
2. Assumption register honored — pass (unchanged).
3. Deviations declared in `03-build-notes.md` — pass (unchanged).
4. Scope creep — none in this diff (diff is docs-only).
5. Build-notes claims true at HEAD — pass (gate re-run green, 224/26).
6. Error paths fail closed — pass (unchanged).
7. Prior blocker/major findings addressed — n/a for this seat (none open);
   minor/nit above remain open by choice, unchanged.

**Verdict: approve.**
