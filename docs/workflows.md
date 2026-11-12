# Feature pipelines

The normative design is [ADR-001](adrs/001-code-driven-agent-pipeline.md).
This page is the practical guide: how work enters the pipeline, what it
produces, and what to do when it stops.

## How work enters

Every pipeline run starts from a document — never from a command-line
sentence:

```sh
pipeline run docs/adrs/007-shared-binding-format.md    # a decision to execute
pipeline run docs/prompts/<name>.md                    # a brief (small job)
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
- **Seats review independently.** A reviewer sees the current artifact, the
  source document, and its own previous findings — the runner re-injects
  those from the seat ledger, so no reviewer reads a report file. Reports
  and transcripts are the run's record, not review input: every stage agent
  is denied *reading* `transcripts/`, review seats are additionally denied
  `reviews/**`, and code-review diff scopes exclude `docs/designs/`.
- **The runner commits each stage's work** (models have no git access),
  reverting paths outside the stage's ownership. The runner never runs the
  formatter itself: `node_modules/.bin` is git-ignored, and executing a
  binary a stage could have planted in the worktree is not a trade the
  runner makes. Hygiene is handled by ownership instead: a red
  `npm run check` summons the `fix` stage, which is told the build is red,
  may run `npm run format` on the files it owns, and replies
  `testsNeedUpdate: true` when the failing files are test-owned — then the
  work routes to the test engineer, so a format failure inside test files
  never deadlocks the fix loop. A fix summoned this way is never handed
  review reports that do not exist yet.

A pass runs up to N seats at a time (`--max-parallel`, default 1 — the
serial lane). Launch order is fixed for reproducibility, and one commit
lands per pass: at the default the schedule is unchanged, but the commit
shape changed with the knob — one per pass, not one per seat. Keep N at or
below the inference slots your server actually exposes: the runner cannot
verify the slot count, an oversized N queues seats, and each seat's timeout
starts at its own prompt send and counts that queue waiting. The editorial
phase is not funneled — it is a single language pass with its own existing
gate.

## Running, resuming, asking

```sh
pipeline run <source-path>        # start (or continue) a run
pipeline resume <design-slug>     # continue from the last checkpoint
pipeline status <design-slug>     # where it is and why it's stuck
pipeline resume <slug> --extend-budget 5   # buy 5 more review sessions
pipeline run <source> --max-parallel 3     # review-lane width ($OPENCODE_MAX_PARALLEL also works)
```

`--max-parallel <n>` (on `run` and `resume`, default 1) is the only review
lane knob; `$OPENCODE_MAX_PARALLEL` is its env fallback and a malformed value
in either source fails the command before any inference is spent. The
guidance on picking `n` is in the stages section above.

`pipeline status` explains the current state from the gate log without you
having to read reviews — including the per-seat board
(`design seats: correctness:approve security:DISSENT* …`), where `*` marks a
verdict that no longer describes the current revision. The aggregated review
gates log under `design-review` / `editorial-review` / `code-review`; a run
checkpointed before those labels landed keeps its historical
`all*ReviewsApproved` entries, so one log can show both vocabularies —
nothing keys on the old names.

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
The lock is also a runner-reserved name: a stage that writes it is an
ownership violation, and the runner re-asserts it at every stage boundary —
if another live process has taken it, the run refuses to continue rather
than work beside a second writer.

## Self-update checkpoint recovery

A stage that writes anywhere inside another design's `docs/designs/<id>/`
directory is rejected, and the write is reverted. The pipeline's own record
— the state file, its temporary files, and the escalation marker — is
reserved the same way: a stage that writes one is rejected, and the
committed bytes are restored.

### A replaced or unreadable checkpoint

If the checkpoint is replaced with something the pipeline cannot read (a
directory, for example), the run escalates and says which command restores
the committed copy:

```sh
rm -r docs/designs/002-example/.pipeline.json && git checkout HEAD -- docs/designs/002-example/.pipeline.json
```

The checkpoint is committed at every stage boundary, so the last boundary is
restored exactly. Then `pipeline resume` as usual.

### A split record

An escalation marker can survive when its state save also failed; the record
then says the run is still running while the marker says it stopped.
`pipeline status` says so in one line, and resuming records that the
operator accepted the split under the previous run's failure.

### A state file from another pipeline version

A state file written by an unsupported pipeline version is never resumed,
and it is never rewritten: the run refuses, and the message names the
unsupported version, the version this runner reads, and the two operator
recovery options — restore the committed checkpoint (`git checkout HEAD --
docs/designs/<id>/.pipeline.json`), or start a fresh run.

## Tuning

Pipeline config defaults (set in `tools/orchestration/src/pipeline/stages.ts`):

- `maxReviews: { design: 40, code: 20 }` — reviewer **sessions** per phase;
  checked at every pass entry, so no pass sneaks past the cap
- `maxParallelReviews: 1` — review-lane width per pass; the `--max-parallel`
  flag and `$OPENCODE_MAX_PARALLEL` override it (see the stages section)
- `maxPassesPerRound: 8` — dissent/ratify passes inside one round; the
  backstop against a seat that will not converge. It is the one budget
  `--extend-budget` cannot buy — sessions and rounds extend, passes do not —
  so a run stopped here is resumed by raising this default, not by buying
  budget (run 010 stopped at 6 passes with all five seats reading `approve`,
  four of them stale).
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
