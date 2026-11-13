# Code review — testing seat, round 1 dissent pass c1

Scope: `git diff 48c80bde34da123b8dd0306343c361617d5cb227..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md` and ADR-020. Specialty: testing checklist only.
This pass checks the revision written to answer the testing seat's round-1
findings first; new findings only where the changed code makes them true.

## Round-1 findings — resolution check

### #1 (major) exit-code integers unpinned — RESOLVED

- `cli.test.ts:10` now statically imports `exitCodeFor`; the
  `describe('the exit-code table')` block (`cli.test.ts:293-326`) pins every
  row of §3's table with explicit `// proves: Test strategy #6` cites:
  `done→0` record-beside-it-or-not (`:303-304`), record standing `→3`
  (`:310`), running-no-record `→2` (`:316`), escalated `→2` with or without a
  record (`:324-325`), and the thrown row by calling `main()` on a
  resume-into-nothing invocation (`cli.test.ts:334-384`, observing the integer
  on whichever channel the CLI answers — returned code, `process.exit`, or
  `process.exitCode`; the channel fallbacks are non-vacuous because
  `process.exitCode` is zeroed at `:354` before the call).
- The #10 driver-loop pin now turns on the integer, not the field:
  `restart-boundary-integration.test.ts:418-433` loops on
  `exitCodeFor(result) === 3` and asserts the terminal code is 0.
  Integer pins also added inside the run-016 integration legs (`:342` pause→3,
  `:393` green→0), the escalation-never-resumed leg (`:454` →2), the red-gate
  pause leg (`:476` →3), and consecutive boundaries (`:507/:513` →3,
  `:525` →0).
- A swap of 3 and 2, a pause exiting 0, or an escalated run exiting 3 now
  reddens the suite — including the escalated-with-record arm
  (`exitCodeFor` gained the `escalated→2` priority arm at `cli.ts:57-59`,
  mirrored by `finalizeEscalate`'s record clear at `runner.ts:3754`, pinned
  end-to-end by the new `runner.test.ts:1161` escalate-after-boundary test,
  which plants the checkpoint-as-directory sabotage through the gate script so
  the save-after-window failure lands with the record standing).
- The old gap ("`exitCodeFor` is called by no test") is closed: it is called
  12 times across the two test files.

### #2 (minor) Test #8 provenance pin vacuous — RESOLVED

`runner.test.ts:1344-1373`: the template file is rewritten to different text
between the two `executeStage` calls, attempt 2 carries the entry-pinned
`templateText` on the handle exactly as the drive loop does at each attempt
(`runner.ts:2665-2672`, `:2708-2716`, loaded once at stage entry at
`runner.ts:2646-2650`), and the test additionally asserts
`templateHash(rewritten) !== expected` (`:1373`) so the identical sha can only
come from the pin. Removing the pin or ignoring `templateText` re-hashes the
new text and fails. The mechanism under test now matches the drive-loop
mechanism (text on the handle, per-attempt re-read suppressed by
`runner.ts:1360`'s `templateText ?? handle.templateText ?? loadTemplate`
precedence).

### #3 (minor) `boundedSurfacePaths` elision + sanitization unexercised — RESOLVED

New case `runner.test.ts:938` ('bounds a record-cap-sized window and keeps
every path copy printable', cites Test strategy #3): an implement stub writes
34 surface probes including one whose filename carries a `0x07` control byte;
the test pins the stored record at 32 real paths + one elision line
(`MAX_RECORD_PATHS=32` cross-checked against the `gate-evidence-capture`
bound), asserts no control byte survives in the checkpoint record, the
`restart-boundary` gate entry, the status line, or the printed pause summary,
and asserts every "N harness file(s)" render shows the window's true total
(34) and never the marker-padded length (33) — the `surfacePathCount`
(`runner.ts:1906`) posture the new count sites take their N from.

### #4 (nit) `toContain('2')` trivially satisfied by the run id — RESOLVED

`runner.test.ts:1258` now asserts `changed 2 harness file(s)`.

## New findings (changed code only)

### 1. nit — unreachable `Number.isNaN` arm in `surfacePathCount` (checklist item 7)

`tools/orchestration/src/pipeline/runner.ts:1911-1916`: `SURFACE_ELISION_RE`
captured group is `(\d{1,9})`, so `hit[1]` is always 1–9 digits and
`Number(hit[1])` can never be NaN — the ternary's `paths.length` arm at
`:1916` is dead code no test can reach. It also papers over a writer/reader
asymmetry: `elisionMarker` (`:1890-1892`) writes an unbounded-count integer,
while the reader regex only matches up to 9 digits, so a (practically
unreachable) 10-digit elision would silently fall back to the padded length.
Fix: widen the regex to `(\d+)` and drop the NaN arm, or derive the count from
a non-textual signal. Checklist item 7 counts dead branches as findings, not
passes; behavior risk is nil, hence nit.

### 2. nit — the status-lines dedup filter's crossed-side claim is unpinned

The new `statusLines` filter (`runner.ts:3955-3961`) drops the
`restart-boundary` gate entry from the recent-gates block only while the
record stands, and its comment claims a boundary already crossed keeps its
entries in that block (citing Test strategy #5). The standing side is pinned
(`runner.test.ts:938` asserts exactly one `/restart/` line with the entry in
the gate log), but no test ever renders `statusLines` for a run whose boundary
was crossed: the only crossed-entry assertion checks the checkpoint's
`gateLog` (`runner.test.ts:1288`), not the status output. A filter widened to
drop restart entries unconditionally keeps the suite green while the recent-
gates block silently loses the crossing record. Fix: in the
`renders a planted record…` test (`runner.test.ts:1270`), after the crossing,
assert `statusLines(saved, …)`'s recent-gates block contains the
`restart-boundary-crossed` entry.

## Checklist walk

1. **Test-strategy citations** — every leg touched by this diff carries its
   `// proves: Test strategy #N` cite (#3 cap test, #4 escalate-after-boundary,
   #5 planted record + one-line status, #6 exit table, #8 provenance, #10
   driver loop). No promise left uncited by this change.
2. **Both layers** — unit (`cli.test.ts`, `runner.test.ts`) plus integration
   (`restart-boundary-integration.test.ts`); the integer contract is pinned at
   both ends.
3. **Revision rounds** — my r1 major (#1) now has the exit-table block and
   integer-driven driver-loop pin that would have caught the 3/2 swap; the
   escalated-with-record and save-failure paths each have a new pin.
4. **Determinism** — no wall-clock, network, or ports in the new tests; the
   cap test's ordering claim rests on git's deterministic bytewise sort
   (`0x07` and `'0'` both precede `'p'`); the `main()` test restores cwd,
   spies, and `process.exitCode` in `finally`.
5. **Behavior-named tests** — new names read as behavior
   ('answers a standing restart record with 3', 'escalates with no restart
   record when the save after the window fails', 'carries the same template
   sha across attempts').
6. **`npm run check`** — see tail below. The runner's gate at
   2026-11-13T21:39:26Z recorded the leg green (exit 0). My full-suite runs
   on this shared machine (the review panel is hammering it concurrently)
   failed with **only** `Error: Test timed out in 5000ms` — never an
   assertion failure — in counts that swing run to run (16, 2, 13, 17, 29, 41
   failing tests) and land on files this diff does not touch
   (`light-promotion`, `checkpoint-integration`, `escalation`, …). Running
   the three changed test files alone is stable green in 3s. I read the red
   full-suite legs as load-induced timeout flake, not code; noting it per the
   checklist rather than treating it as the change's posture.
7. **Untestable additions** — finding 1 above (dead `Number.isNaN` arm); the
   rest of the new code (escalated arm, `printable` re-render at the crossing,
   record clear in `finalizeEscalate`, status filter standing-side, handle-
   carried template pin) is exercised by the new tests.

## Item 6 — output tails

Changed files in isolation (green):

```text
 ❯ tests/pipeline/runner.test.ts  ✓
 Test Files  3 passed (3)
      Tests  49 passed (49)
   Duration  2.99s
```

Full suite under concurrent panel load (flake pattern, tail):

```text
⎯⎯⎯⎯ Failed Tests 41 ⎯⎯⎯⎯
Error: Test timed out in 5000ms.   (×41; zero AssertionError lines in any run)
 Test Files  18 failed | 36 passed (54)
      Tests  29 failed | 578 passed | 2 skipped (609)   (counts varied across five runs)
```

Runner gate record (cited, captured by the runner from the code under review):
`npm run check` exit 0 — green; cargo legs green.

## Verdict

All four round-1 findings resolved; the revision's new pins are non-vacuous and
directly target the regressions they name. Two nits open; none blocks.

**approve**
