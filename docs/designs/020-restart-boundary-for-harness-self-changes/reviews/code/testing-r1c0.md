# Code review — testing seat, round 1 pass c0

Diff reviewed: `git diff 0179375b8d037833c6b6c5150312d88af0162211...HEAD -- ':(exclude)docs/designs'`.
Judged against `02-proposal.md` (Test strategy, Decisions 1–4) and ADR-020.

## Verdict

**request-changes** — 1 major, 2 minor, 1 nit. The suite is comprehensive and
the seam widening is pinned everywhere except the one place Decision 3 lives:
the exit integers themselves.

## Checklist

1. **Every numbered Test strategy item has a citing test** — formally yes:
   #1 `gates.test.ts:757,770`; #2 `state-store.test.ts:326,353,381`;
   #3 `runner.test.ts:795,820,839,871,887`; #4 `runner.test.ts:928,956,980,1006`;
   #5 `runner.test.ts:1038,1066`; #6 `cli.test.ts:226–268`;
   #7 `executable-surface.test.ts:95,118`; #8 `runner.test.ts:1092`;
   #9 `restart-boundary-integration.test.ts:323,373`;
   #10 `:408,432,450`; #11 `:474`. But the #6 pin does not pin the behavior
   its rows name — finding 1.
2. **Unit and integration layers** — both present: unit in
   `gates/state-store/runner/cli/executable-surface` tests, integration in
   the new `restart-boundary-integration.test.ts` (mock client, temp git
   repos, per the promised posture). ✓
3. **Revision rounds** — n/a: first code round, no prior rejected findings.
4. **Determinism** — all stubs are in-process clients; git runs in
   `mkdtemp` repos; no wall-clock, network, or port assertions (`at:` values
   are fixture constants, never asserted against `Date.now()`). ✓
5. **Test names describe behavior** — yes ("opens the boundary when an
   implement window adds a template", "crosses the boundary on resume and
   renders the changed template green"). ✓
6. **`npm run check` run locally, green** — tail below. The runner's gate
   record (2026-11-13T21:02:48.947Z) shows all four legs exit 0.
7. **Untestable additions** — the writer-side bounding branch for boundary
   paths is never exercised — finding 3. `PROMPTS_DIR`'s new export is
   consumed by the honesty test; no dead knobs otherwise.

```
 Test Files  54 passed (54)
      Tests  599 passed | 2 skipped (601)
   Start at  16:15:43
   Duration  9.44s (tests 71%, import 23%, transform 5%, worker 1%)
```

(The `mount: must be superuser` line is the pre-existing cap-probe's expected
environment noise, not a failure.)

## Findings

### 1. major — Decision 3's exit-code table is cited but never asserted; no test pins the integers

**Location:** `tools/orchestration/tests/pipeline/cli.test.ts:225–273`
(against `src/pipeline/cli.ts:43` `exitCodeFor`).

Test strategy #6 names four rows — `done→0`, record standing `→3`,
escalated `→2`, thrown `→1` — and the risk register's mitigation for its
top risk is "Distinct codes pinned both ways (pause→3, escalation→2)…
pinned by Test #6". The four tests cite the rows but only assert the seam
*shape*: `result.status` and `result.restartRequired`. Nothing anywhere in
the suite calls `exitCodeFor` (grep: it appears only in `cli.ts`), and the
integration driver-loop pin (#10) loops on
`result.restartRequired !== undefined`, explicitly "the standing record,
not the status" — so the integer 3 is asserted nowhere. A regression
swapping 3↔2, or pausing with exit 0, keeps all 599 tests green — exactly
the driver-misread risk (pause mistaken for escalation) the design claims
is mitigated by this pin.

**Fix:** in `cli.test.ts`, statically import `exitCodeFor` from
`../../src/pipeline/cli.js` and assert the table: `exitCodeFor({status:'done'})===0`,
`exitCodeFor({status:'running',restartRequired:record})===3`,
`exitCodeFor({status:'escalated'})===2`; pin the thrown-error row with
`await main([...bad slug args])` (or the documented rejection handler)
returning 1. Keep the existing seam tests — they pin the widened result.

### 2. minor — the #8 provenance pin is vacuous against the mechanism it names

**Location:** `tools/orchestration/tests/pipeline/runner.test.ts:1091–1137`.

Strategy #8 pins that attempt 2 "carries the identical sha — the entry pin
(runner's stage-entry template preload) is the record §4 relies on". The
test calls `executeStage` twice against an **unchanged** `implement.md`, so
each call re-loads identical text: the identical sha is true even if the
stage-entry pin is deleted or `templateText` is ignored. The test pins the
provenance *line and hash* (worth having) but cannot catch the regression
it claims to catch.

**Fix:** after the attempt-1 call, rewrite `implement.md` on disk to
different text, then run attempt 2 with the stage-entry-pinned
`templateText` exactly as the drive loop does, and assert attempt 2's
transcript sha still equals `templateHash(originalText)` — the sha of the
text loaded at entry, not the text on disk.

### 3. minor — the boundary record's writer-side posture (printable + MAX_GATE_LEGS bound) is never exercised

**Location:** `tools/orchestration/src/pipeline/runner.ts:1862–1868`
(`boundedSurfacePaths`); no citing test.

§2 (r2 security revision) requires boundary paths to keep the writer-side
posture — `printable` copies, bounded like `boundedGateEvidence`. The
`paths.length > MAX_GATE_LEGS` elision branch (32-cap, the "N further
harness path(s) elided by the record cap" line that flows into the
checkpoint, gate log, commit body and status line) is unreachable in the
suite: every fixture window touches one or two surface paths. Checklist
item 7: unexercised branch. The sibling `boundedGateEvidence` posture is
pinned in its own design's suite (`gate-evidence-capture.test.ts:187`,
`gate-evidence-render.test.ts:74`) — this copy of the posture has no
equivalent pin.

**Fix:** add a window-classification case whose implement stub writes
`MAX_GATE_LEGS + 2` surface probe files, and assert the saved record holds
32 real paths plus the elision line with the right count; include one
probe filename carrying a control byte and assert the record/gate-entry
copies are `printable`-sanitized.

### 4. nit — status-line count assertion is trivially satisfiable

**Location:** `tools/orchestration/tests/pipeline/runner.test.ts:1083`.

`expect(lines[0]).toContain('2')` passes on the fixture's own run id
(`002-x`) regardless of the file count.

**Fix:** assert the count phrase, e.g. `toContain('changed 2 harness file(s)')`.

## FYI (no severity)

The integration fixture's second-half `{{gateEvidence}}` render relies on the
merged renderer already supplying the variable — a faithful model of the
run-016 class (the fresh process renders the committed tree), noted only
because a future reader may look for a stale-renderer leg that cannot exist
in one process.
