# Code review — correctness — r1 ratification c2

Scope: `git diff 48c80bde34da123b8dd0306343c361617d5cb227..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and ADR-020. Purpose of this
pass: confirm the post-approval revisions did not break what this seat
approved in round 1, and that the r1 finding was answered.

## Verdict: approve

## R1 finding — resolved

- **minor (r1): `finalizeEscalate` did not clear `state.restartRequired`** —
  fixed exactly as prescribed: `state.restartRequired = undefined` is now the
  first statement of `finalizeEscalate` (runner.ts:3754), mirroring the pause's
  own failed-save arm (runner.ts:3673). The r1 scenario — a surface-touching
  coder window sets the record at window end, the iteration's checkpoint save
  then fails, the run escalates — is pinned end to end by the new
  `runner.test.ts` fixture `escalates with no restart record when the save
  after the window fails` (gate-command sabotage of the checkpoint path,
  escalated result with no record, `ESCALATION.md` present, no boundary commit,
  restored committed checkpoint carries no record). A second belt stands at
  the seam: `exitCodeFor` answers `escalated → 2` before consulting the record
  (cli.ts:56-58), pinned by the new `cli.test.ts` exit-code-table block
  (`answers an escalated result with 2, record beside it or not`) and by the
  integration driver-loop fixture's `expect(exitCodeFor(result)).toBe(2)`.
  The widened result therefore cannot carry exit 3 beside a committed
  `ESCALATION.md`: the `while [ $? -eq 3 ]; do resume; done` loop can never
  resume past an escalation. `docs/workflows.md` now documents the
  escalation-outranks-pause rule, so the shipped copy matches the code.

## Ratification check — nothing approved was broken

Walking the revision hunks against the proposal:

1. **Names/signatures (item 1):** `touchesExecutableSurface`, the
   `restartRequired` companion field, `finalizeRestartBoundary`, the loop-top
   boundary check (runner.ts:2619-2622), the resume clear, and
   `exitCodeFor` all unchanged in shape. The `templateText` relocation onto
   `StageHandle` (executeInference's chain at runner.ts:1359-1360) preserves
   the §4/A8 entry pin: `entryTemplateText` is still loaded once at stage
   entry (runner.ts:2646-2650) and now rides on the handle into both attempts;
   behavior is identical to the r1 arg-passing, and Test #8's pin is now
   strictly non-vacuous (the template is rewritten on disk between attempts,
   and the identical sha can only come from the entry pin).
2. **Assumptions (item 2):** A5 (status enum and CLI mapping) holds — the
   escalated arm maps to 2, its exact A5 meaning. A7, A8, A9, A10 untouched.
3. **Deviations (item 3):** the `escalated` arm before the record arm is
   narrower than §3's canonical sentence only for an `escalated + standing
   record` result — a state the design itself makes unreachable (escalation
   clears the record). Every reachable row of §3's table (`done→0`, standing
   record→3, escalated→2, other non-done→2, thrown error→1) is honored, and
   the arm implements §3's stated rationale verbatim ("`2` keeps its exact
   current meaning ... no existing driver automates past an escalation by code
   collision"). No undeclared deviation found in the reviewed paths.
4. **Scope (item 4):** the `main`/`dispatch` split with a catch that returns
   1 on a printable message implements the table's own "thrown error → 1" row
   at the seam Test #6 pins; the gate-name constants, `surfacePathCount`, and
   the `statusLines` recent-gates filter all serve §2's bounding posture and
   §3's "carries one line" (the pause entry's twin in recent gates is hidden
   only while the standing line renders it; crossed entries keep reading).
   No un-asked-for behavior.
5. **Claims (item 5):** the gate record (green `npm run check`, fmt, clippy,
   cargo) plus the read of the new tests support the diff's asserted
   behaviors; the count tests verify 34-true-total vs 33-padded-list, matching
   `MAX_GATE_LEGS = 32`.
6. **Error paths (item 6):** the ADR-006 §3 terminal-save guard in
   `finalizeRestartBoundary` is intact (3662-3682) and now consistent with
   `finalizeEscalate`'s record clear — the escalation is the single terminal
   record on every failure arm.
7. **Revision rounds (item 7):** the r1 finding is addressed as above.

## FYI (no severity, out of specialty)

- A future proposal touch-up could restate §3's canonical mapping to name the
  escalation-before-record arm explicitly; today only `docs/workflows.md` and
  the code comment carry it.
- The `isMain` rejection handler (cli.ts:300-303) is now effectively dead for
  throws inside `main`; it is a harmless second belt.

## Verdict

**approve** — zero open blocker/major findings; the r1 minor finding is fixed,
tested at three levels, and the revisions leave the r1-approved behavior of
the boundary mechanism intact.
