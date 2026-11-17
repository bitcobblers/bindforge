# Feature pipelines

The normative design is [ADR-001](adrs/001-code-driven-agent-pipeline.md).
This page is the practical guide: how work enters the pipeline, what it
produces, and what to do when it stops.

## How work enters

Every pipeline run starts from a document — never from a command-line
sentence:

```sh
pipeline run docs/adrs/007-shared-binding-format.md    # a decision to execute
pipeline run docs/tasks/T-001-add-auth-service.md      # a task (a unit of work beneath an ADR)
pipeline run docs/prompts/<name>.md                    # a brief (small job)
```

A run is launched from a dedicated worktree, never from the main checkout
while it sits on the default branch: `ensureBranch` checks the run branch
out *in the launch directory*, so a run started from the main checkout
would move that checkout off `main` for the run's lifetime. Create a
worktree, launch there, and issue `resume` from the same worktree — the
restart boundary (ADR-020) names the worktree as the place to relaunch.
The main checkout stays on `main`.

An ADR that feeds the pipeline must have frontmatter `status: proposed` and
non-empty **Context** and **Decision** sections. The pipeline flips it to
`accepted` itself when the run completes; if a run stalls, the ADR stays
`proposed` and the reason is recorded in the run's `.pipeline.json`.

A **task** (ADR-023) is a smaller, concrete unit of work — often one of
several required by an ADR. Tasks live in `docs/tasks/T-NNN-slug.md`, count
in their own number series from 001 (separate from ADRs), and carry their
artifacts to the segregated root `docs/designs/tasks/NNN-slug/`. A task must
have frontmatter `doc_type: task`, `status: pending`, `type`, and
`functional_area`, plus a non-empty body; `associated_adr` (the ADR it
serves) and `depends_on` (other tasks, as `T-NNN-slug` names) are optional.
The pipeline flips `status` to `completed` itself when the run completes;
`disposition` is yours to manage. A `depends_on` entry is checked at entry:
the named task must exist, and its run must be `done`, or the run refuses to
start. Every source document carries a `doc_type` key — `adr` or `task` —
and a `doc_type` that disagrees with the document's path is a usage error.

Brief-driven runs claim the next free `NNN` number. If that work later
becomes a real decision, write `docs/adrs/NNN-<slug>.md` with the same
number. `resume` and `status` search both design roots —
`docs/designs/NNN-slug` and `docs/designs/tasks/NNN-slug` — and refuse a
slug that exists in both.

A source document can select the **light flow** with frontmatter
`flow: light` (ADR-010): one `plan` stage instead of research and the design
panel, the same coder, and a two-seat code review. The key is
case-sensitive and validated before the lock — `flow: Light` or
`flow: sideways` is a usage error that starts nothing (no lock, no design
directory, no session). Without the key, every run is the full flow.

## What a run produces

Everything lives under `docs/designs/NNN-<slug>/`:

| Path                                 | What it is                                                                |
| ------------------------------------ | ------------------------------------------------------------------------- |
| `01-research.md`                     | Findings with a URL behind every claim                                    |
| `02-proposal.md`                     | The design, incl. assumption register and test strategy                   |
| `03-build-notes.md`                  | Deviations the coder had to make                                          |
| `03-build-notes.md` § Gate posture   | The runner's own gate record: each leg, exit, bounded tail (ADR-016)      |
| `reviews/design/<specialty>-rNcM.md` | Design review reports, per specialty, per pass of a round                 |
| `reviews/design/editorial-rN.diff`   | The editor's diff that the editorial review checks                        |
| `reviews/code/<specialty>-rNcM.md`   | Code review reports (`cM` absent for single-pass stages)                  |
| `transcripts/<stage>-rNcM.md`        | What was actually asked/answered, with model + timestamps                 |
| `.pipeline.json`                     | State, seat ledger, revision stamps, gate log (runner-owned; do not edit) |

Code is committed on a dedicated branch `pipeline/NNN-<slug>`. After
`done`, the operator merges it back into `main` as a single squash commit
(`git merge --squash`), then removes the run branch and its worktree —
merge-back is a human decision, and the runner never merges.

## The stages

```
source gate -> research -> proposal -> design review (funneled passes)
  -> editor -> editorial review (same 5, scoped to the edit diff) -> approved
  -> test authoring -> implement -> test pre-check
  -> code review (funneled passes) -> doc sync -> done
```

The **light flow** (ADR-010) is the short path beside it:

```
source gate -> plan (five sections, 80-line cap) -> implement
  -> code review (2 seats: correctness, testing; funneled passes) -> done
```

A light run has no research, no design panel, no editor, no test engineer,
and no doc-sync commit. Its coder owns the test files the full flow keeps for
the test engineer, and its plan lands at the full flow's
`02-proposal.md` path, so the revision stamps, the size meter and the
`reviews/code/` reports all hang off it unchanged.

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
  `reviews/**`, and code-review diff scopes exclude `docs/designs/`. Since
  ADR-016 those two paths and the run's checkpoint (`.pipeline.json` — every
  seat's verdict and findings and the gate record, a file only the runner reads
  or writes) are denied in the **permission layer** of every shell-holding
  stage agent (`shell *transcripts*`/`*reviews*`/`*pipeline.json*`, plus the
  matching `read`/`glob`/`grep` denies), not only by the post-hoc deed audit —
  knowingly coarse, with the audit still the backstop. Each review seat
  re-opens exactly its own last report through `git diff`, so a seat can
  re-read what it already wrote. Every review and implementation prompt also
  carries one bounded block of the runner's own gate record — the legs it ran,
  their exits and capped tails, credential-bearing and absolute-path lines
  already dropped where the record is written — as evidence the seat may cite;
  it is never an instruction to run anything, and a run with no record yet
  renders an explicit stand-in line.
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

A pass runs up to N seats at a time (`--max-parallel`, default 2; N = 1 is
the serial lane). Launch order is fixed for reproducibility, and one commit
lands per pass whatever the width — one per pass, not one per seat. Keep N at or
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
pipeline resume <slug> --stage plan  # light flow: re-plan a run a plan stop ended
pipeline run <source> --max-parallel 3     # review-lane width ($OPENCODE_MAX_PARALLEL also works)
```

`--max-parallel <n>` (on `run` and `resume`, default 2) is the only review
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
A light run has one more stop: a plan over the 80-line cap — or a plan stage
that wrote no document at the proposal path — escalates at the plan stage,
before the plan is committed and before the `implement` session is spent:
over length is a scope signal, and the record names the promotion. That stop is
**sticky**: it is recorded on the checkpoint, so a plain `pipeline resume`
answers with the same record instead of handing the refused plan to
`implement` — the refused plan is on the branch (the terminal escalation
commit tracks it), and the flow has no stage that should build on it.
Every escalation writes `ESCALATION.md` at the design directory root —
assembled from the gate log, the seat board, and open findings — and exits
non-zero. Read `ESCALATION.md` and run the repair it prints: for the stops
above that is fixing the cause (usually the brief or the proposal) and
`pipeline resume`; for a plan stop it is promoting the run (below) or
re-planning with `pipeline resume <slug> --stage plan`, which clears the stop
once the new plan passes the gate.

Light-flow escalations from `implement`, `fix` or `code-review` carry the
promotion recipe in the record: the run is on the light flow, and the way
out is to promote it (below).

### Promoting a light run to the full flow

Promotion is an operator action with no state migration (ADR-010): set
`flow: full` in the source document and re-run with `--force` on the same
identity —

```sh
# in the source document: flow: light  ->  flow: full
pipeline run <source-path> --force
```

The light run's branch and design directory are already "taken" by the
clash check, so the re-run needs `--force`. The full `proposal` stage
rewrites `02-proposal.md`, and the light plan stays recoverable in the
branch's git history — the run's record, not a migration.

The identity rule behind that is A17's: an ADR source names its own
`NNN` from its file name, so the promoted run lands on the light run's
identity and the clash check is what asks for `--force`. A brief carries
no number in its name, so the shared counter hands the promoted run the
next free `NNN` and it lands beside the light run; pin `--nnn`/`--slug`
to promote onto the light run's identity.

Only one pipeline may run at a time: the runner holds `.pipeline.lock` at
the repo root and exits loudly if another run holds it. If the holder
process is dead, the lock is cleared on the next invocation with a warning.
The lock is also a runner-reserved name: a stage that writes it is an
ownership violation, and the runner re-asserts it at every stage boundary —
if another live process has taken it, the run refuses to continue rather
than work beside a second writer.

## When a run pauses at a restart boundary (ADR-020)

A self-change run edits the harness itself. After each runner-minted stage
window the runner diffs the window's net change against the harness's
**executable surface** — what that process executes or renders, the
`tools/orchestration/src/` modules it imported at start and the
`tools/orchestration/prompts/` templates it renders at stage time. Only an
`implement` or `fix` window can touch either, so ordinary product runs never
see a boundary. When a window does move the surface, the run **pauses**
instead of opening the next stage window against code that is now stale. The
pause is not a failure: no attempt, round, session or budget is charged, no
`ESCALATION.md` is written, and the checkpoint carries a `restartRequired`
record naming the stage and the changed files. The pause is run history:
a `restart-boundary` gate-log entry and a terminal
`chore: <nnn>-<slug> restart boundary` commit whose body names the files.

The `run` and `resume` commands exit with:

| Exit | Meaning                                                |
| ---- | ------------------------------------------------------ |
| 0    | done                                                   |
| 3    | paused at a restart boundary — the record stands       |
| 2    | escalated — read `ESCALATION.md`                       |
| 1    | error                                                  |

`2` keeps its exact pre-020 meaning, so no existing driver mistakes a pause
for an escalation. An escalation always outranks a pause: a run that ends
escalated exits 2 and its checkpoint carries no `restartRequired` record, even
when a stage window had just opened a boundary in the same process. While the
record stands, `pipeline status` carries one line:

```text
restart boundary: stage implement changed 2 harness file(s) this runner executes or renders — not a failure, nothing was charged for it; resume from this worktree and the fresh process carries the run on: pipeline resume 020-example
```

Resume **is** the restart: relaunch `pipeline resume <slug>` from the run's
worktree, where the fresh process has already imported and renders the
committed tree. The resume gate-logs the boundary it crosses, clears the
record, and carries the run on — the next stage renders the changed template
with the current code. The driver-loop posture for self-change runs:

```sh
pipeline run docs/adrs/020-example.md
while [ $? -eq 3 ]; do pipeline resume 020-example; done
```

v1 counts boundaries only in the gate log; a per-run cap is deferred
(ADR-020, "Not decided here").

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

### Per-flow budgets (ADR-010)

The flow selects a budget partial over the defaults below
(`tools/orchestration/src/pipeline/flows.ts`); `pipeline status` and
`ESCALATION.md` print the effective numbers for the run's flow:

| Knob                  | `full`                                             | `light`                                        |
| --------------------- | -------------------------------------------------- | ---------------------------------------------- |
| Stage path            | the twelve stages above                            | plan, implement, fix, code review              |
| Code review seats     | 5: correctness, compatibility, security, performance, testing | 2: correctness, testing |
| `maxReviews.code`     | 20 sessions                                        | 6 sessions                                     |
| `maxPassesPerRound`   | 6                                                  | 3 — the binding cap: 2 seats × 3 passes = 6 sessions |
| `maxRounds`           | 5                                                  | 2 (bounds the mutating fix loop)               |
| `planMaxLines`        | absent — the size meter warns, it does not gate    | 80 — a plan over it stops the run              |
| `docMaxLines`         | 900 — a metered document (`01-research.md`, `02-proposal.md`) over it fails that stage attempt, and the normal retry-then-escalate budget decides | absent — the plan stays under `planMaxLines`   |
| `stageTimeoutMs`      | 30 min                                             | 12 min                                         |

Pipeline config defaults (set in `tools/orchestration/src/pipeline/stages.ts`):

- `maxReviews: { design: 40, code: 20 }` — reviewer **sessions** per phase;
  checked at every pass entry, so no pass sneaks past the cap
- `maxParallelReviews: 4` — review-lane width per pass; the `--max-parallel`
  flag and `$OPENCODE_MAX_PARALLEL` override it (see the stages section)
- `maxPassesPerRound: 6` — dissent/ratify passes inside one round; the
  backstop against a seat that will not converge. It is the one budget
  `--extend-budget` cannot buy — sessions and rounds extend, passes do not —
  so a run stopped here is resumed by raising this default, not by buying
  budget. Run 010 stopped at 6 passes with all five seats reading `approve`,
  four of them stale: a ratify pass that re-stamped to `HEAD` aged the very
  approvals it was ratifying. The fix is that stamp rule (a ratify pass keeps
  the stamp its round entry set), not a higher price for every run.
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

### Stage shell rights (learned in runs 008 and 011)

Because the runner enforces ownership by reverting out-of-ownership paths,
agent permissions are a guardrail, not the enforcement — so the stage agents
are given the shell they need to close their own loop:

- `.opencode/agents/{code,codereview}/*` deny `shell *` and then allow the
  toolchain (`cargo *`, `rustfmt *`, `npm run *`, `npm test *`, `npx vitest *`)
  and read-only inspection (`cd`, `git status|diff|log|show`, `ls`, `cat`,
  `head`, `tail`, `wc`, `grep`, `rg`, `find`). Since ADR-016 the five design
  seats carry the same check block — read-only gate legs allowed, every
  tree-writing leg denied — and the five document families
  (`research/`, `design/`, `ops/`) carry the read-only inspection set in the
  same star form, with no toolchain allow: their leading `shell *` deny stays
  authoritative, so `cargo` and `npm` never reach a document stage.
- Denied last, because the last matching rule wins: commits, branch state,
  pushing, `rm`/`mv` (the runner owns moves — ADR-005), `tee`/`sed -i`/`dd`
  (writes that bypass the ownership audit), and publishing. Review seats
  additionally lose the tree-writing legs (`cargo fmt` without `--check`,
  `cargo fix`, `npm run format`) while keeping `cargo fmt --all -- --check`.

Two failure modes this posture exists to avoid:

- **Compound legs.** OpenCode scans a shell command into subcommands and every
  leg must pass, so an allowlist of narrow prefixes denies `cd crates/core &&
  cargo test` on its `cd` leg. Prefixes must therefore cover the helper legs,
  not just the interesting command.
- **Self-propagating blockers.** Run 008 wrote "no stage can invoke a toolchain"
  into its build notes; ADR-011's proposal inherited it as a `verified`
  assumption (A16) and its stages stopped trying, spending 4 of 5 code rounds
  on formatter complaints nothing had run. A claim that a tool is unavailable
  is a claim about the current agent config — verify it before recording it as
  an assumption.

`tools/orchestration/tests/pipeline/agent-shell.test.ts` pins the gate
commands against these rule sets so a future tightening cannot silently
re-blind a stage.
