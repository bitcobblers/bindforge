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

| Path | What it is |
| --- | --- |
| `01-research.md` | Findings with a URL behind every claim |
| `02-proposal.md` | The design, incl. assumption register and test strategy |
| `03-build-notes.md` | Deviations the coder had to make |
| `reviews/design/<specialty>-rN.md` | Design review reports, per specialty, per round |
| `reviews/code/<specialty>-rN.md` | Code review reports |
| `transcripts/<stage>-rN.md` | What was actually asked/answered, with model + timestamps |
| `.pipeline.json` | State, round counters, verdicts, gate log (runner-owned; do not edit) |

Code is committed on a dedicated branch `pipeline/NNN-<slug>`. Merging it
back after `done` is a human decision.

## The stages

```
source gate -> research -> proposal -> design review (5 reviewers x rounds)
  -> editor -> editorial review (same 5, scoped to the edit diff) -> approved
  -> test authoring -> implement -> test pre-check
  -> code review (5 reviewers x rounds) -> doc sync -> done
```

The five review specialties — correctness, compatibility, security,
performance, complexity — **all run every round**; the round passes only if
all five approve with no `blocker`/`major` findings open. They execute one
at a time (single local inference lane); the order is fixed for
reproducibility.

## Running, resuming, asking

```sh
pipeline run <source-path>        # start (or continue) a run
pipeline resume <design-slug>     # continue from the last checkpoint
pipeline status <design-slug>     # where it is and why it's stuck
```

`pipeline status` explains the current state from the gate log without you
having to read reviews.

## When a run escalates

A run escalates when a review stage exhausts its 5-round budget or a stage
burns its step allowance twice. It writes `ESCALATION.md` at the design
directory root — assembled from the gate log and open findings — and exits
non-zero. Read `ESCALATION.md`, fix the cause (usually the brief or the
proposal), then `pipeline resume`.

Only one pipeline may run at a time: the runner holds `.pipeline.lock` at
the repo root and exits loudly if another run holds it. If the holder
process is dead, the lock is cleared on the next invocation with a warning.

## Tuning

Pipeline config defaults (set in `tools/orchestration/src/pipeline/stages.ts`):

- `maxRounds: 5` — revision rounds per review stage
- `stepsDefault: 40` — model-turn cap per stage, overridable per stage
  (agent files must not set `steps` themselves)
- `blockingSeverities: ["blocker", "major"]` — what blocks a round

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
