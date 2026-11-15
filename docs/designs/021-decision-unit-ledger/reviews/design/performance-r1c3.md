# Performance review r1 (pass c3 — ratification)

verdict: approve

Scope: the `git diff 67e7b57 e66564c` revision of `02-proposal.md` plus the
full current document, checked only for regressions against what the
performance seat approved in r1. Inputs used: the proposal, the diff, the
Revision log, ADR-021, and read-only verification of the code claims the
revised text makes (`MAX_GATE_LEGS` in `tools/orchestration/src/pipeline/gates.ts:310`;
`.github/workflows/check.yml` — `npm run check` on `pull_request`, node
`22.23.5`; `tools/orchestration/tests/pipeline/tmp-dir.ts` present;
`tools/orchestration/package.json:23` chains lint/format:check/test).
No run record other than this report was opened.

## Findings

None. No revised section breaks an approved performance property, and no
section r1 cited has become false.

- Closed (r1 nit, not re-raised): Chosen design ¶4 — the r1 finding flagged
  the "maximal trailing run of backticked unit ids … one or more code spans"
  grammar as inviting a nested-quantifier regex with ambiguous adjacent space
  separators. The revision replaces it with "A handle is one trailing code
  span on a source line whose content is unit ids separated by single spaces,
  followed only by spaces and at most one table-cell `|` before the line
  terminator" — one span per line, ids delimited by a mandatory token, so the
  multi-span separator ambiguity that made the polynomial case is gone. The
  requested "deterministic single pass" sentence was not added, but the
  residual (`spaces, optional `|`, spaces`) is at most quadratic in one line's
  length on a corpus of short lines (A5, A23), and Test #2's accept/refuse
  fixtures — the ones r1 cited, unchanged — pin the behavior a right-to-left
  scan must reproduce. Closed as resolved-by-simplification; re-raising it
  would violate this pass's bar.

Checklist walk over the changed text:

1. Event-path latency — n/a to a change cost: the gate still runs at run
   entry and at `finalizeDone` (¶7 sentence "runPipeline calls it immediately
   after sourceAdmissible" unchanged); nothing lands on a per-keystroke path.
   The new ¶2 pointer rule ("the last `→` outside a code span") and the
   leading `\uFEFF` drop (¶1) are O(line)/O(1) per line. No violation.
2. O(n²) — the handle matcher got cheaper (single span); `checkLedgers`
   structure ("walks every ledger-bearing ADR and every corpus file") is
   unchanged. Dropping the four `amends-*` checks (¶3, ¶7: "findings in
   twelve codes") only removes work; "unit ids are never reused" (021-L4)
   stays structurally enforced by ¶2's `NNN`-equals-file-number plus per-file
   uniqueness, so the removed `unit-id-reuse` code leaves no unbounded scan.
3. Memory bounds — the denial bound survives the rewrite: "denies with at
   most `MAX_LEDGER_FINDINGS` findings — an exported constant, the way
   `MAX_GATE_LEGS` bounds a gate record" — and the cited analogue exists
   (`gates.ts:310`, exported). Test #9 keeps the pin ("lists at most
   `MAX_LEDGER_FINDINGS` findings and states the elided count"). Bounded.
4. Startup cost — A23's bounded-scan basis (28 repo-local markdown files) is
   untouched by the diff.
5. Caching — no cache in the design; n/a.
6. Baseline gap — `01-research.md` is not an input to this pass (n/a here);
   the ADR's stated "the join check is a standing CI cost" (Consequences)
   matches Test #10 and the Risks row, and A24's promotion from unverified to
   verified matches the shipped workflow checked above — strictly stronger
   than what r1 approved.

Renumbering coherence (test #11→#10 etc.) checked at every changed cite:
Risks rows, matrix "#1–#12", Test #13 ("Linux executes #1–#12"), Rollout 1
"#1–#8 / #9, #11, #12 / #10" and 4, 5 — all consistent; Test #2 (r1's cited
fixture) is unchanged. Revision log covers each change; no rejected-and-
unsound finding, and no Deferred item contradicts ADR-021 (the deferred
`amends` symmetry is not one of 021-R2's four named check classes).

## FYI (no severity)

- ¶7: "denies with at most `MAX_LEDGER_FINDINGS` findings … each as a
  `file:line code detail` line plus `+N more`" reads as if every finding line
  carries the `+N more` suffix; the intended single trailing elision line is
  a language-seat phrasing nit, not a bound problem.
