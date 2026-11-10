# Feature pipelines

The normative design is [ADR-001](adrs/001-code-driven-agent-pipeline.md).
This page is the practical guide: how work enters the pipeline, what it
produces, and what to do when it stops.

## How work enters

Every pipeline run starts from a document — never from a command-line
sentence:

```sh
pipeline run docs/adrs/007-shared-binding-format.md    # a decision to execute
pipeline run docs/prompts/add-timeout-flag.md          # a brief (small job)
```

An ADR that feeds the pipeline must have frontmatter `status: proposed` and
non-empty **Context** and **Decision** sections. The pipeline flips it to
`accepted` itself when the run completes; if a run stalls, the ADR stays
`proposed` and the reason is recorded in the run's `.pipeline.json`.

Brief-driven runs claim the next free `NNN` number. If that work later
becomes a real decision, write `docs/adrs/NNN-<slug>.md` with the same
number.

## What a run produces

Everything lives under `docs/designs/NNN-<slug>/`:

| Path                                 | What it is                                                                |
| ------------------------------------ | ------------------------------------------------------------------------- |
| `01-research.md`                     | Findings with a URL behind every claim                                    |
| `02-proposal.md`                     | The design, incl. assumption register and test strategy                   |
| `03-build-notes.md`                  | Deviations the coder had to make                                          |
| `reviews/design/<specialty>-rNcM.md` | Design review reports, per specialty, per pass of a round                 |
| `reviews/design/editorial-rN.diff`   | The editor's diff that the editorial review checks                        |
| `reviews/code/<specialty>-rNcM.md`   | Code review reports (`cM` absent for single-pass stages)                  |
| `transcripts/<stage>-rNcM.md`        | What was actually asked/answered, with model + timestamps                 |
| `.pipeline.json`                     | State, seat ledger, revision stamps, gate log (runner-owned; do not edit) |

Code is committed on a dedicated branch `pipeline/NNN-<slug>`. Merging it
back after `done` is a human decision.

## The stages

```
source gate -> research -> proposal -> design review (funneled passes)
  -> editor -> editorial review (same 5, scoped to the edit diff) -> approved
  -> test authoring -> implement -> test pre-check
  -> code review (funneled passes) -> doc sync -> done
```

The five review specialties — correctness, compatibility, security,
performance, complexity — form a standing panel; each seat holds a verdict
pinned to a revision stamp (ADR-002). A round is a **sequence of passes**,
named `rNc0`, `rNc1`, …:

- **c0 is the full panel** — all five seats review the whole document (for
  code, the branch diff).
- A revision goes back only to the **dissent pass**: the seats that rejected
  the previous revision, and only they. A seat's `approve` carrying an open
  `blocker`/`major` finding counts as dissent.
- Once no dissent is open, the seats whose approval predates the current
  revision run **one ratification pass** over the diff since they approved.
  A stale approval never re-runs while a dissent is open (late catch only).
- The phase advances when **every seat approves the current revision stamp**:
  the proposal's git blob id for design, `HEAD` for code. The next revise
  (or fix) stage receives only the last pass's dissenting reports.

Seats execute one at a time (single local inference lane); the order is
fixed for reproducibility. The editorial phase is not funneled — it is a
single language pass with its own existing gate.

### Proposed, not implemented

- **ADR-004 — parallel review lane (proposed).** A pass's seats run up to N
  at a time instead of strictly one at a time, with one commit per pass. It
  schedules whatever panel ADR-002 selects.

## Running, resuming, asking

```sh
pipeline run <source-path>        # start (or continue) a run
pipeline resume <design-slug>     # continue from the last checkpoint
pipeline status <design-slug>     # where it is and why it's stuck
pipeline resume <slug> --extend-budget 5   # buy 5 more review sessions
```

`pipeline status` explains the current state from the gate log without you
having to read reviews — including the per-seat board
(`design seats: correctness:approve security:DISSENT* …`), where `*` marks a
verdict that no longer describes the current revision.

## When a run escalates

A run escalates when review can no longer move — the session budget is spent
(`pipeline resume <slug> --extend-budget <n>` grants more, recorded in the
gate log), the per-round pass backstop is hit, the revise-round backstop is
hit, or a **deadlock stop** fires: the designer replied `changed: false`
while a seat still dissents. A stage may also fail its reply contract twice.
It writes `ESCALATION.md` at the design directory root — assembled from the
gate log, the seat board, and open findings — and exits non-zero. Read
`ESCALATION.md`, fix the cause (usually the brief or the proposal), then
`pipeline resume`.

Only one pipeline may run at a time: the runner holds `.pipeline.lock` at
the repo root and exits loudly if another run holds it. If the holder
process is dead, the lock is cleared on the next invocation with a warning.

## Tuning

Pipeline config defaults (set in `tools/orchestration/src/pipeline/stages.ts`):

- `maxReviews: { design: 40, code: 20 }` — reviewer **sessions** per phase;
  checked at every pass entry, so no pass sneaks past the cap
- `maxPassesPerRound: 6` — dissent/ratify passes inside one round; the
  backstop against a seat that will not converge
- `maxRounds: 5` — backstop on the mutating revise/fix loop only; passes
  inside a round never spend a round
- `stageTimeoutMs: 30 min` — wall-clock cap per stage attempt; stage turns
  are bounded only by this timeout, there is no model-turn/step cap
- `blockingSeverities: ["blocker", "major"]` — what makes a seat dissent

### Small-model hardening (learned in the first dry run)

The runner, not the prompts, carries these deterministic countermeasures:

- A stage reply is the **last assistant message carrying text** (models
  often end on a tool-call turn), extracted with a string-aware balanced
  JSON scan (prose- and fence-wrapped replies parse; empty `{}` objects are
  never a reply).
- Idle-without-reply (narration like "Now let me verify...") triggers up to
  **2 nudges in the same session** before the attempt fails; each nudge
  restates the full JSON schema inline.
- Verdict/severity **spelling synonyms** (`Approved`, `critical`, ...) are
  normalized deterministically before schema validation; unknown values
  still fail.
- **Zero model turns** after a prompt fails fast as
  `no model turns — agent "..." may not exist` (binding a nonexistent agent
  is silent otherwise).
- **Stage file ownership is enforced in the runner**, not just in agent
  config: agent permissions proved advisory (a model delegated `git commit`
  to a subagent to route around them). After each stage, changed paths
  outside the stage's ownership are reverted, including mid-stage commits.
