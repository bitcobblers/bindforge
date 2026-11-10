---
status: proposed
date: 2026-11-09
---

# ADR 002: Send a revision only to the reviewers who dissented

A revision goes back to the reviewers whose blocking findings it was written to
answer, and no further. Reviewers who already approved see it exactly once,
after the dissenters are satisfied. A review round stops being the unit of cost:
the budget counts reviewer sessions.

This amends ADR-001 principle 5's round half — "every specialty running every
round", "results aggregate only at round end" — and its `maxRounds` budget.
**All-must-approve survives**, re-expressed over revision stamps rather than over
one round's five reports. ADR-001 remains accepted otherwise.

## Context

The last attempt to design ADR-003 (run `002-run2`, stopped by the operator on 9
Nov after 8 design rounds) measured what "every specialty runs every round" buys:

- **40 design reviewer sessions produced 33 blocking/major findings.** Seat yield
  was lopsided: security 11, correctness 8, compatibility 2, performance 0,
  complexity 0 — the complexity seat approved all 8 rounds. The three
  lowest-yield seats consumed roughly 45% of design review time for 2 of the 33
  findings.
- **0 re-raised themes across rounds 1-6**, while the round gate denied 8 times
  out of 8. Repeating the full panel did not produce convergence; it produced
  new surface — the proposal grew from 401 lines to 2,696.
- **Sections no revision had touched produced zero findings; sections the last
  revision touched produced 155 of 191 citations.** Unchanged prose is
  measurably where review value is not.
- The cap counts rounds, but a round costs 1-5 sessions depending on what
  happened in the previous one. A round cap bounds neither cost nor operator
  intent, and an operator extending it cannot say what they are buying.

## Decision

### Vocabulary

- **Seat** — one specialty's reviewer for a phase (`correctness`,
  `compatibility`, `security`, `performance`, `complexity` for design; the same
  shape with `testing` for code).
- **Pass** — one invocation of a panel inside a round. Passes are numbered
  `c0, c1, …` within round `rN`.
- **Dissenter** — a seat whose latest verdict on the current revision contains
  at least one finding at a severity in `blockingSeverities`. Non-blocking
  findings are advisory: they are handed to the designer with the rest and never
  trigger a pass.
- **Revision stamp** — the git blob id of the design document at the moment a
  verdict was recorded (`git rev-parse HEAD:docs/designs/<run>/02-proposal.md`),
  which the runner already computes for its size meter.

### The loop inside one round

1. `c0`: all five seats review the whole document.
2. Any dissenter's blocking findings are aggregated and sent to the designer for
   a revision.
3. The revised document goes back to **the dissenters only**. Scope: the diff
   since the revision each of them reviewed, plus every section that diff
   touches.
4. Dissent again → revise again → back to step 3. Repeat until no seat holds an
   open blocking finding.
5. **Ratification pass**: every seat whose approval predates the current
   revision reviews the cumulative diff since the revision it approved. This
   happens once, not once per pass.
6. A ratifier that requests changes joins the dissenter set and the loop
   resumes at step 2; the other approvers wait again.
7. The round passes when all five seats' latest verdict is `approve` for the
   current revision stamp.

Nothing about the artifact contract's *content* changes: each seat still writes
one report per pass it takes, still replies with the same verdict JSON, and the
gate is still code over those verdicts.

### Late catch only

An approving seat is never pulled back mid-round, not even when the revision
touched a section that seat had cited. A regression in an uncontested dimension
is caught at ratification, one pass later. This is a deliberate trade: it keeps
panel selection a function of verdicts and stamps alone, with no parsing of
reviewer reports.

### Budget: sessions, not rounds

- `maxReviews`: **40** design-side reviewer sessions, **20** code-side.
- `--extend-budget <n>` grants **n sessions**, not n rounds, and the consent
  entry in the gate log says so.
- **Cycle backstop:** at most 6 dissent passes inside one round, so a single
  round cannot consume the phase budget.
- **Deadlock stop:** when the designer replies `changed: false` while a seat
  still dissents, the run escalates immediately, naming the open seats and the
  findings they hold. That is the run-2 failure mode — objections that revisions
  do not move — detected in one pass instead of eight rounds.

### What does not change

The stage chain (`research -> proposal -> design review -> editor -> editorial
review -> implement -> code review -> doc sync`), one session per stage, the
one-retry-then-escalate rule, artifact ownership, transcripts, `.pipeline.lock`,
and the editorial phase — an editor is a language pass, not a vote, so its
single confirmation review stays as written. Checkpoint changes are **additive
with defaults**; `version` stays 1.

### State and artifacts

Per phase, per seat: `{verdict, stamp, dissents}`. Per phase: passes used.
Reports are named `reviews/<phase>/<specialty>-r<round>c<pass>.md`; the `rN`
prefix keeps round history readable. There is **no finding ledger** — see
Alternatives.

### Validation posture

This change is expected to be implemented by the pipeline itself, so it is
proven by mock-client unit tests rather than by a live run first: panel
selection at `c0`, dissenters-only at later passes, the approver set at
ratification, and a ratifier re-joining the dissenter set; stamp-based
eligibility; budget accounting in sessions including `--extend-budget`; the
deadlock stop; resume mid-pass with a partial panel; and a checkpoint written
before these fields existing still parsing.

## Alternatives considered

**Full panel every round (the status quo):** measured above — 45% of design
review time for 2 of 33 findings, and it did not converge in 8 rounds.

**Early pull-back of approvers** (an approval expires when a revision touches a
section that seat cited): rejected. It needs the runner to parse reviewer
reports for section anchors, it spends sessions on the failure class it
prevents — 7 `regression` findings across the whole of run 2 — and ratification
already catches that class, one pass later. Revisit if ratification turns out to
catch regressions too late to be cheap.

**A finding ledger with stable keys:** deferred, not rejected. It is what makes
"the same objection in new wording" detectable, and it would bound a stubborn
seat with something better than a counter. Under this funnel a wasted pass costs
1 session instead of 5, so the ledger does not pay for its matching complexity
until re-raise churn is measured against real runs. Related gate policy lives in
ADR-004.

**Severity-weighted gates or an arbiter seat** when two seats hold incompatible
demands: deferred to ADR-004. This ADR keeps the AND over five seats.

**Keep the round cap and add the funnel:** rejected. With a panel of 1-5 seats,
a round cap bounds neither inference cost nor operator intent.

## Consequences

Becomes possible: run 2's design phase fits inside the 40-session budget with
slack — roughly 5 seats at `c0`, about 6 dissent passes, about 4 ratifiers — and
the sessions that remain are spent on text that actually changed. The designer
receives 1-2 dissenting reports per pass instead of five full reports each
round, which is part of why the last proposal grew sixfold while every round
reported closing its findings.

Accepted costs: a stubborn seat can re-raise a closed objection in new wording
and burn a pass, bounded only by the cycle backstop, the deadlock stop and the
session budget; regressions in uncontested dimensions surface at ratification,
after everything else has already been approved; one round's report set spreads
across more files; and `report.ts`, `pipeline status`, and `ESCALATION.md` must
show seats, stamps and sessions rather than rounds, or the operator loses the
ability to tell a stuck run from a busy one.

Not covered here: running a pass's seats concurrently — ADR-003, whose unit of
fan-out becomes "the panel this pass selected"; cross-version checkpoint resume
and checkpoint trust — ADR-004.

Ordering: this is the next ADR to be run. The operator has not authorized a run
against it yet, and nothing here should be executed against ADR-003 or ADR-004
first.
