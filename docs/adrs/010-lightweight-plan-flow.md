---
status: accepted
date: 2026-11-11
---

# ADR 010: Add a lightweight plan flow for small work

We will add a second, selectable **flow** to the pipeline — `light`: a short
plan, one implementation stage, and a two-seat code review — declared in
`src/pipeline/flows.ts` and selected by `flow: light` frontmatter on the source
document. The runner stays single: lock, stage-file ownership, per-stage
commits, checkpoint/resume, escalation, transcripts, and the reply-contract
hardening are shared by both flows unchanged, and every existing document and
checkpoint keeps running the current flow (`flow` defaults to `full`).

## Context

`docs/workflows.md` advertises `pipeline run docs/prompts/<name>.md` as the
entry point for "a brief (small job)", and ADR-001 lists brief-driven runs
beside ADR-driven ones. The cost never differentiated them. `decideNext`
(`states.ts`) starts **every** run at `research` with no branch on
`source.kind`, so a brief pays the identical 13-stage path as an ADR: the
five-seat design-review funnel, the editor, a five-seat editorial confirmation
round, the separate test-authoring engineer, and the five-seat code review.

The seven runs on disk show what that buys, measured from each run's
`.pipeline.json` (stage sessions = entries in `results`, wall clock = the sum
of `durationMs`):

| Run                                    | Sessions | Wall clock | Scope of the task                     |
| -------------------------------------- | -------- | ---------- | ------------------------------------- |
| 004-mirror-src-hierarchy-in-tests      | 53       | 3.0 h      | test-directory layout chore           |
| 003-parallel-inference-lane            | 43       | 2.2 h      | one CLI flag                          |
| 007-audit-by-deeds                     | 44       | 3.3 h      | three harness defects                 |
| 006-self-update-checkpoint-compatibility | 43     | 3.9 h      | checkpoint recovery rules             |
| 008-rust-for-the-product               | 73       | 5.8 h      | language decision                     |
| 005-runner-executed-file-moves         | 96       | 7.1 h      | runner-executed moves                 |

The cheapest run in the table cost 43 sessions to add a command-line flag. The
heavy part is review breadth: run 005 spent 27 sessions on
`design-review:*` seats alone, and 004 spent 14 on `test-update` and `fix`
that a small change should not have needed at all. The multi-seat panel is the
right price for a decision that fixes a public contract; for moving files
between test directories it buys audit at a price that makes the pipeline
avoided, and the ADR chain is the thing that suffers when runs are avoided.

Two facts make a lighter path cheap to add rather than a second system. First,
`runReviewPass` (`runner.ts`) is already parameterized by its seat list, and
the funnel (`panel.ts`) is a function of the seat ledger and a revision stamp —
so a two-seat code review is the existing code with a shorter specialty list.
Second, the light run's design artifact can live at the same path
(`02-proposal.md`), which keeps the seat stamp (`git` blob id of that file),
the `docLines` size meter, the reviewers' faithfulness check, and
`reviews/code/` paths working untouched. The reuse is not incidental: the
heavy path's cost is its **stages and seats**, and those are exactly the parts
that are declarable per flow. The parts that are not shed-able — the ownership
matrix that reverts out-of-scope writes, the single-flight lock, the
checkpoint grammar ADR-006 guards, the audit-by-deeds rule of ADR-007, the
nudge/reply rules earned in the first dry runs — all live in `runner.ts`,
which is why a second runner is not on the table.

## Decision

Two flows over one state machine. `flow: 'full' | 'light'`, default `full`.
The word `lane` stays reserved for inference width (ADR-003's parallel
inference lane, `--max-parallel` as the review-lane knob).

### The flow registry (`src/pipeline/flows.ts`)

A flow declares its stage list, review panels, budgets, source gate, and
ownership deltas. It carries no inference logic: the runner drives it.

```
full (today, unchanged)
  research -> proposal -> design review (5 seats, funnel passes)
    -> editor -> editorial review -> test-authoring -> implement
    -> test pre-check -> code review (5 seats) -> doc-sync -> done

light (new)
  plan -> implement -> code review (2 seats, funnel passes) -> done
                       ^___ fix, max 2 rounds ___^
```

The light flow runs no research, no design-review panel, no editor, no
editorial round, no test-authoring stage, and no doc-sync. It runs the
existing funnel pass logic unmodified: full pass with both seats, revision to
the dissenters only, ratification pass for stale approvals.

`LIGHT_CONFIG` overrides `DEFAULT_CONFIG` (which remains the sole default
authority) with:

| Knob                  | `full`       | `light`                    |
| --------------------- | ------------ | -------------------------- |
| `maxRounds`           | 5            | 2                          |
| `maxReviews`          | 40 / 20      | design 0, code 6           |
| `maxPassesPerRound`   | 6            | 3                          |
| `stageTimeoutMs`      | 30 min       | 12 min                     |
| code panel            | 5 specialties | `correctness`, `testing`  |
| design panel          | 5 specialties | none (no design phase)    |

`blockingSeverities` and the serial-lane default are shared. `--max-parallel`
applies to both flows.

Budgets resolve from `state.flow` inside `runPipeline` and `resumePipeline`,
with the operator's `--max-parallel` applied last. They must not come from the
CLI's `DEFAULT_CONFIG`: a resumed light run that silently inherited
`maxRounds: 5` and `maxReviews.code: 20` looks healthy and spends 20 review
sessions on a flag change.

### Selecting the flow

`flow: light` in the source document's frontmatter — the document is the task
and the record, and the CLI never carries prose (ADR-001). `parseSource`
already runs its frontmatter scan for briefs, so the gate learns one key: a
value that is neither `full` nor `light` is a source-gate denial, before the
lock and before any inference. Absent means `full`, which keeps every existing
ADR and brief behaving exactly as it does today.

`flow` is stamped into `.pipeline.json` at the run's entry save, so a resume
needs no flag. In `StateSchema` it is added **optional**, following ADR-006 §2:
new fields are optional with a behavior-safe default, nothing is removed,
retyped, renamed, or re-interpreted, and `version` stays `1`. A checkpoint
written before the field exists resumes as `full`.

### Light stages

- **`plan`** (new stage, agent `design/planner`, template `prompts/plan.md`)
  writes `02-proposal.md` at the path the heavy flow uses, from a short
  template: problem, chosen approach, files to touch, test plan, done
  criteria. The `plan-note` skill carries the template and its length cap.
  The runner measures the document with the existing `proposalSize` meter.
- **`implement`** is the existing coder stage and template. In the light flow
  the coder **owns the test files**: there is no test engineer, so
  `stagePathAllowed(stage, path)` gains the flow, and light `implement`/`fix`
  may write test paths. The runner-reserved rule (checkpoint, escalation
  marker, lock, `saveState` tmp names) stays first, ahead of every per-stage
  rule, in both flows. `03-build-notes.md` keeps recording deviations.
- **`code-review`** uses the existing `codereview/code-correctness` and
  `codereview/code-testing` agents and the existing template; the seats are
  selected by the flow, not by new code in the pass loop.
- **`fix`** is the existing stage. There is no `test-update` stage to route to,
  because in this flow no test path belongs to anyone else.

No `01-research.md`, no `reviews/design/`, no editorial diff.
`docs/designs/NNN-<slug>/` is the artifact root for both flows, with the
`NNN` allocation shared: a light run consumes a number alongside a decision,
and the branch is `pipeline/NNN-<slug>` in both cases.

### When a light run is too big

Two deterministic stops, both naming the way out:

- **Oversized plan.** The plan stage's measured length over the template cap
  (80 lines) escalates with `plan outgrew the light flow (N lines)`. The
  meter is the one that caught run 009's proposal growth; the stage cannot
  report its own size.
- **Budget exhaustion.** `maxRounds: 2` spent in the code phase escalates as
  today, and the reason names the flow.

Promotion is an operator action, not a mid-run migration: set `flow: full` in
the source document and re-run it with `--force` on the same identity. The
full run's `proposal` stage rewrites `02-proposal.md`; the light plan survives
in the branch's git history.

### Status

`pipeline status` names the flow and, for a light run, shows the two-seat
board. An escalated light run must read as "this was too big for the light
flow", not as an unexplained stall.

### Test plan

Mirrored per ADR-004: `tests/pipeline/flows.test.ts` for the registry; light
transitions in `states.test.ts`; frontmatter parsing, the oversized-plan gate,
and light test-path ownership in `gates.test.ts`; a pre-`flow` checkpoint
resuming as `full` plus a golden light checkpoint in the
`fixtures/checkpoints/` era set; a light run driven end to end against the
fake client; and one test that resumes a light checkpoint and asserts the
**effective** config, because budget drift on resume is the failure this
design can hide.

## Alternatives considered

**A second runner for small jobs.** Rejected: it forks the ownership matrix,
the lock, the checkpoint grammar, the escalation records, the transcripts, and
the reply-contract hardening — every lesson runs 001–009 paid for in escalations.
Every future ADR-006/007-class fix would then land twice, and the copy is the
one nobody reviews.

**Config-only variants of the current flow.** Rejected: with no new stages, a
small job still runs `research`, `test-authoring`, `proposal-final`, and
`doc-sync`, and the review seats still fan out to five. Dropping the seats and
rounds is a fifth of the win; the stages it has no input for remain.

**No plan artifact at all (`implement -> review -> done`).** Rejected: the
code reviewers check faithfulness to `02-proposal.md`, the seat stamp is that
file's blob id, and the run record would then contain no statement of what the
change was meant to do. A capped page of plan is the cheapest artifact that
keeps the audit chain intact.

**Keep the test-authoring engineer in the light flow.** Rejected: ADR-001's
separation of duties (tests written from the design, never from the
implementation) is the reason the test/ownership boundary exists, and it is
worth its price in the heavy flow. In a flow aimed at a flag or a file move it
costs a stage, the `test-update` routing loop, and the ownership revert path —
and run 004 shows that loop is where a small job's sessions actually go.

**`lane` as the field name.** Rejected: ADR-003 and `docs/workflows.md`
already use "lane" for inference width, and one word carrying two meanings in
an escalation message is how an operator misreads a stop.

## Consequences

Becomes possible: a small, reviewable change enters the pipeline at a cost
that makes using it rational, so the design record covers routine work instead
of only decisions; the flow is recorded in the source document and the
checkpoint, so a resumed run needs no flag and a transcript shows which rules
governed it; the review panel, budgets, and stage list become declarable data,
so the next flow (a docs-only flow, a read-only audit flow) is a registry
entry rather than a new state machine.

Costs and risks accepted: `stagePathAllowed` — the security spine ADR-006 §1
and ADR-007 lean on — gains a flow parameter, widening the coder's write
surface in one flow, so its tests must cover the reserved-name rule in both
flows; light runs consume `NNN` numbers, so the shared numbering space grows
faster and the ADR/design directory pair is no longer one-to-one in meaning;
promotion overwrites the light plan, leaving it only in branch history; the
`plan` stage inherits every small-model hazard of the stages it replaces
without the design panel's five pairs of eyes, so the two code seats are the
whole safety net for a wrong plan; and a task that is small in lines but wide
in consequences (a codec change, a public schema edit) is exactly the case
where nothing in the tool stops an operator from picking `light` and
under-reviewing it.

Deferred: auto-detecting the flow from the source document (the operator
chooses, and the oversized-plan gate catches the mismatch); mid-run promotion
with state migration; a docs-only flow; per-flow model routing.
