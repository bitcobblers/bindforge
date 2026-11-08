---
status: accepted
date: 2026-11-07
---

# ADR 001: Adopt a code-driven state machine orchestrating thin agents

We will orchestrate agentic feature work (research -> design -> review ->
test-first implementation -> review) with a TypeScript state machine in
`tools/orchestration`, driving one-shot OpenCode sessions bound to
single-purpose agents and exchanging only in-repo artifacts under `docs/`.

## Context

BindForge needs repeatable agentic workflows to grow the framework: research
before design, multi-specialty design review, implementation, and multi-
specialty code review. Two constraints shape any design:

1. **Local inference with limited resources.** The engine holds enough context
   to run exactly one task at a time. Logical concurrency of tasks is fine;
   concurrent inference is not.
2. **Agent behavior varies from model to model.** If the workflow itself
   (sequencing, fan-out, gating, retries) lives in an agent's prompt, its
   semantics drift every time the model changes. We need pipeline behavior
   that is identical across models.

Additionally, subagents run with fresh context, so any multi-agent chain must
hand off through durable artifacts rather than conversation memory.

## Decision

### Design principles

1. **Artifacts on disk, never conversation memory.** Chaining only works
   through the artifact contract below.
2. **The state machine is the orchestrator.** There is no orchestrator agent.
   Each transition spawns a *fresh top-level session* bound to the stage's
   agent via the OpenCode API (never a `subagent` tool call), removing
   cross-model ambiguity about how work gets delegated.
3. **Thin agents, deep skills.** Agent files are ~20 lines (role,
   permissions, which skill to load). Domain expertise lives in skills with
   `references/` files. Prompts for small local models are short and
   imperative; judgment lives in checklists and in code.
4. **Gates are code.** Where code can check something (test exit codes,
   verdict JSON, file presence), it does — never trust a model's word.
5. **Sequential inference lane, complete review rounds.** A stage's five
   reviewers are a logical fan-out executed as a queue (one inference at a
   time), **every specialty running every round**. Results aggregate only at
   round end, and the round passes only if **all** reviewers approve — there
   is no intra-round short-circuit. The order
   `correctness -> compatibility -> security -> performance -> complexity`
   is a deterministic execution order for reproducibility, not an
   early-exit device. New session per stage; per-stage `steps` cap (default
   15, overridable in `stages.ts`).

### Artifact contract (`docs/`)

```
docs/
├── adrs/
│   ├── template.md               # context / decision / consequences + status frontmatter
│   └── NNN-<slug>.md             # status: proposed | accepted | superseded
├── prompts/
│   └── <slug>.md                 # kickoff briefs for work that doesn't warrant an ADR
├── designs/
│   └── NNN-<slug>/
│       ├── 01-research.md        # researcher
│       ├── 02-proposal.md        # designer (+ revisions; editor final pass)
│       ├── 03-build-notes.md     # coder
│       ├── reviews/design/<specialty>-rN.md
│       ├── reviews/code/<specialty>-rN.md
│       ├── transcripts/<stage>-rN.md   # provenance header + prompt + reply + gate outcome
│       └── .pipeline.json        # state, round counters, verdicts, gate log, source
└── workflows.md                  # human-facing pipeline doc
```

- Collections are parameterized (`designs` and `prompts` now, `adrs` later):
  adding a
  `docs/*` collection is a config entry + stage definition, not a permission
  re-audit.
- Kickoff prompts are always files, never CLI arguments.
- **The active design directory is the shared hand-off location.** Because
  hand-offs are files, every pipeline agent needs write access there: the
  baseline denies `edit` under `docs/**`, and each pipeline agent re-allows
  exactly `docs/designs/**`. Separation of duties comes from the stage prompt
  naming the file to write plus the runner verifying expected outputs at
  expected paths — an agent that writes the wrong file fails its schema gate.
  `docs/adrs/**` and the runner-owned records inside the design directory
  (`.pipeline.json`, `transcripts/`) are the exceptions: only the runner
  writes them.

### Entry points: ADR-driven pipeline

```sh
pipeline run docs/adrs/007-shared-binding-format.md    # ADR-driven (decisions)
pipeline run docs/prompts/add-timeout-flag.md          # brief-driven (small jobs)
```

- The source document seeds the `research` and `proposal` stage templates;
  `.pipeline.json` records `source: { kind, path }`.
- The ADR brackets the pipeline lifecycle: `proposed` at start, flipped to
  `accepted` deterministically **by the runner** (targeted frontmatter patch,
  verified by re-parse) on the `done` transition. A model never edits the
  record. Stalled runs leave the ADR `proposed` with the reason in
  `.pipeline.json`.
- Numbering is shared: ADR-007 -> `docs/designs/007-<slug>/`. Prompt-driven
  runs take the next free NNN; if the work later crystallizes into a
  decision, a human authors the matching ADR and back-fills `source`.
- Deferred: auto-drafting ADRs from brief-driven designs (muddies the
  "what were we tasked with" provenance).

### State machine

```
run(source)
  -> sourceAdmissible gate
  -> research
  -> proposal
  -> design-review round n       (5 reviewers, each runs every round;
                                  aggregate at round end; all must approve)
       allDesignReviewsApproved -> proposal-final (editor)
       gate fail & n < MAX_ROUNDS    -> proposal-revise(n+1) -> design-review(n+1)
       gate fail & n == MAX_ROUNDS   -> escalated (written summary, non-zero exit)
  -> editorial-review            (proposal-final may shift meaning while fixing
                                  language: 5 reviewers run one extra round,
                                  scoped by the runner-produced pre/post-edit
                                  diff; numbering continues the rN sequence)
       all approve              -> approved
       fail & design budget left -> proposal-revise -> design-review (normal loop)
       fail & budget exhausted   -> escalated
  -> test-authoring              (test engineer writes tests from 02-proposal.md
                                  + research only; never sees the implementation)
  -> implement                   (coder's job: make the authored tests green)
  -> testsGreen pre-check        (runner runs npm run check before any review
                                  sessions are spent)
  -> code-review round n         (5 reviewers, each runs every round;
                                  aggregate at round end; all must approve)
       all approve + testsGreen -> doc-sync -> done
       fail & n < MAX_ROUNDS -> fix(n+1) -> [test-update if fix reply sets
                                 testsNeedUpdate] -> testsGreen re-check
                                 -> code-review(n+1)
       fail & n == MAX_ROUNDS -> escalated
```

- MAX_ROUNDS = 5 per review stage (config). The editorial-review is one
  additional confirmation round; a failure there re-enters the normal revise
  loop and consumes a round from the same design budget.
- A failed `testsGreen` pre-check spends no reviewer inference: it routes
  straight to `fix`. The pre-check itself consumes no review round; the `fix`
  it triggers draws on the code-review stage's round budget, preserving
  termination.
- **Runner commits per stage.** The pipeline works on a dedicated branch
  `pipeline/NNN-<slug>`; after each mutating stage the runner commits exactly
  that stage's owned paths — deterministic staging, no git access for any
  model. The commit message is model-generated via a `commitMessage` field on
  the stage's Zod reply, validated against the Conventional Commits format in
  AGENTS.md with a deterministic fallback, so committing can never fail.
  Merging the branch back after `done` remains a human decision.
- **One pipeline at a time.** The runner holds a repo-level lock
  (`.pipeline.lock`, gitignored; records slug, PID, stage, timestamp) for the
  duration of a run; a second invocation exits loudly citing the holder. A
  stale lock (dead PID) is cleared with a logged warning. NNN allocation
  happens under the same lock.
- Step caps: `stepsDefault = 40` (measured in the first dry run: a capable
  researcher needs more than 15 model turns before it can even write its artifact),
  overridable per stage in `stages.ts` — the
  **single source of record**; agent files must not set `steps` (the runner
  validates agent configs and refuses to launch a stage whose agent defines
  it). The runner enforces the effective cap by aborting the stage session at
  the cap.
  **Fail-loud rule**: exhausting the allowance is a hard stage failure, *not*
  a review round. The runner detects exhaustion from the session history
  (model-turn count from `message.list` >= effective cap; the forced final
  summary also fails Zod reply validation), records `steps-exhausted` in the
  `.pipeline.json` gate log, and retries the stage **once** (fresh session,
  same inputs). Exhaustion on the retry -> escalate.
- Checkpoint after **every** stage into `.pipeline.json` -> `pipeline resume`
  is first-class (local runs are slow; crashes happen).
- `pipeline status <slug>` explains where a run is stuck from the gate log,
  without re-reading reviews.

### Gates: three tiers (strongest first)

| Tier | Gate | Evaluated by |
|---|---|---|
| Deterministic | `sourceAdmissible` | Zod over source file: ADR parses, `status: proposed`, non-empty Context/Decision; brief is non-empty markdown. Fails before any inference. |
| Deterministic | `testsGreen` | Runner executes `npm run check` itself **before every code-review round** (initial and after every fix). Red pre-check routes to `fix` without reviewer sessions; still re-verified at aggregation. Exit code is the verdict, independent of reviewer claims. |
| Deterministic | `lintClean` | Same mechanism as `testsGreen`; folded into `npm run check`. |
| Deterministic | `stepsExhausted` | Runner counts model turns per stage session from `message.list`; at cap -> hard failure, log `steps-exhausted`; one stage retry, escalate on second exhaustion. |
| Aggregated verdicts | `allDesignReviewsApproved`, `allCodeReviewsApproved` | TS over Zod-validated JSON replies collected at **round end**. Every required specialty present **and** approve; missing != passed; partial rounds never advance the state machine. |
| Schema/shape | `researchCited`, `proposalSectionsComplete` | Zod over stage reply + presence checks on artifact files. |

```ts
// pipeline/gates.ts
export type Gate = (s: PipelineState) => { proceed: boolean; reason: string }

export const allDesignReviewsApproved: Gate = (s) => {
  const verdicts = s.stageResults.filter(isDesignReviewRound(s.round))
  if (verdicts.length < REQUIRED_REVIEWS.length)
    return deny(`missing reviews: ${missing}`)   // missing != passed
  const open = verdicts.flatMap(v => v.findings)
    .filter(f => config.blockingSeverities.includes(f.severity))
  return open.length ? deny(`${open.length} blocking findings open`) : allow()
}
```

- Verdict replies are Zod-validated JSON: `{ verdict: "approve" |
  "request-changes", findings: [{ severity, ... }] }`; severities
  `blocker | major | minor | nit`. Markdown review files are the human
  record; JSON replies are the control signal.
- Config `blockingSeverities = ["blocker", "major"]` (default); `minor`/`nit`
  recorded and surfaced to humans, non-blocking.
- The transition table references gates by name, so the state graph stays
  declarative. Every gate denial is logged with its reason.

### Prompts: on-disk templates + transcripts

- **Agent file** = durable role (persona, skill to load, permissions).
- **Stage template** = per-run instantiation, deliberately small:

```md
<!-- tools/orchestration/prompts/design-review.md -->
Review the design at {{designDir}}/02-proposal.md.
Apply only this change set: {{scope}}.
Load the `design-review` skill and apply the {{specialty}} checklist.
Write your report to {{designDir}}/reviews/design/{{specialty}}-r{{round}}.md.
Reply with the JSON verdict object only.
```

- Placeholders: `{{designDir}} {{specialty}} {{round}} {{sourcePath}}
  {{reviewFiles}} {{scope}}`. `{{scope}}` is the whole artifact for design and
  code rounds and the runner-produced editor diff for the editorial round.
  The CLI never carries prose.
- The runner writes the editorial diff to `reviews/design/editorial-rN.diff`.
  The escalation summary lands in `ESCALATION.md` at the design-directory
  root, assembled by the runner from the gate log and open findings — no
  inference spent on it.
- After every stage, the runner archives to `transcripts/<stage>-rN.md`: a
  **provenance header** (stage + round, agent id, template path + content
  hash, model as `provider/model#variant`, start/end timestamps, and the
  inference parameters exposed by the endpoint), followed by the rendered
  prompt, the raw reply, and the gate outcome. Transcripts are **committed**:
  runs stay reproducible, debuggable on small models, and behavior is
  diffable across model changes.
- Research anti-hallucination rule: every claim in `01-research.md` carries
  a URL; `review/design-correctness`'s first checklist item spot-checks a
  sample of those URLs via `webfetch`.

### Agent roster (16 files, all `mode: primary`)

| Agent | Writes | Key permissions |
|---|---|---|
| `research/researcher` | `01-research.md` | edit `docs/designs/**`; `websearch`/`webfetch` allow |
| `design/designer` | `02-proposal.md` + revisions | edit `docs/designs/**`; read repo freely |
| `design/editor` | final language pass on `02-proposal.md` | edit `docs/designs/**` |
| `review/design-correctness` | `reviews/design/correctness-rN.md` | edit `docs/designs/**` (report only); shell deny; `webfetch` allow |
| `review/design-complexity` | `reviews/design/complexity-rN.md` | same as design-correctness |
| `review/design-security` | `reviews/design/security-rN.md` | same, plus `webfetch` for CVE checks |
| `review/design-performance` | `reviews/design/performance-rN.md` | same as design-correctness |
| `review/design-compatibility` | `reviews/design/compatibility-rN.md` | same as design-correctness |
| `code/coder` | source; `03-build-notes.md` | edit all repo paths **except** `docs/adrs/**` and test files owned by the test engineer; shell allow `npm run check` |
| `code/test-engineer` | test files only (`test-authoring`, `test-update`) | edit test paths + `docs/designs/**`; receives `02-proposal.md` + research only — never the implementation, so tests validate the design, not the coder's assumptions |
| `codereview/code-correctness` | `reviews/code/correctness-rN.md` | edit `docs/designs/**` (report only); shell allow build/test/lint; faithfulness to `02-proposal.md` incl. assumption register |
| `codereview/code-complexity` | `reviews/code/...` | same as code-correctness |
| `codereview/code-security` | `reviews/code/...` | same as code-correctness |
| `codereview/code-performance` | `reviews/code/...` | same as code-correctness |
| `codereview/code-testing` | `reviews/code/...` | same, plus: `approve` only if `npm test` ran green (runner verifies independently) |
| `ops/doc-sync` | drift notes on `02-proposal.md` / `03-build-notes.md` | edit `docs/designs/**`; ADR status flips are done by the runner, not this agent |

- Every pipeline agent can write to the shared hand-off location
  (`docs/designs/**`) and nothing under `docs/` beyond it.
- Runner-owned records inside the design directory — `.pipeline.json` and
  `transcripts/**` — are denied in every agent's permissions (deny rules
  after the allow); agents can influence the pipeline only through their
  stage's artifact files and their Zod-validated reply.
- **Test ownership boundary:** test files belong to `code/test-engineer`.
  Implementation and fix diffs that touch owned test paths fail a
  deterministic runner diff-check: the runner reverts the offending paths to
  the last stage commit, then queues `test-update` if the fix reply set
  `testsNeedUpdate`.

Nested IDs: `.opencode/agents/review/design-correctness.md` ->
`review/design-correctness`. Representative agent file:

```md
---
description: Reviews a design proposal for correctness — unverified assumptions, contradictions, missing edge cases
mode: primary
permissions:
  - action: edit
    resource: "*"
    effect: deny
  - action: edit
    resource: "docs/designs/**"
    effect: allow
  - action: edit
    resource: "docs/designs/**/.pipeline.json"
    effect: deny
  - action: edit
    resource: "docs/designs/**/transcripts/**"
    effect: deny
  - action: shell
    resource: "*"
    effect: deny
  - action: webfetch
    resource: "*"
    effect: allow
---

Load the `design-review` skill and apply the correctness checklist to the
proposal path given by the caller. Write your report to the reviews/design/
file named by the caller and reply with the JSON verdict object only.
```

Rules are ordered so the last match wins: blanket deny, then the scoped allow
for the hand-off location, then the runner-owned records stay off-limits.
Agents are `mode: primary` because the runner binds them to top-level
sessions; none is ever launched through the `subagent` tool. Baseline
permissions in `.opencode/opencode.jsonc`: deny `edit` under `docs/**`
globally, then each agent re-allows the shared hand-off location as
above. Interactive TUI use is humans invoking stages with the built-in
`build` / `plan` agents + the skills.

### Skills (7)

```
.opencode/skills/
├── research/SKILL.md               # + references/lenses.md (prior art, security
│                                   #   landscape, perf baselines, platform notes)
├── design-proposal/SKILL.md        # + references/template.md: context, goals/
│                                   #   non-goals, options, chosen design, assumption
│                                   #   register (verified/unverified), risks,
│                                   #   compatibility matrix, test strategy
├── design-review/SKILL.md          # + references/{correctness,complexity,security,
│                                   #   performance,compatibility}.md — shared severity
│                                   #   model; compatibility checklist covers OS-specific
│                                   #   input APIs (X11/Wayland/Windows/macOS HID), Node
│                                   #   floors, native modules
├── proposal-editing/SKILL.md       # editorial style guide, language-only pass
├── repo-practices/SKILL.md         # TS strictness, ESLint/Prettier/Vitest, new
│                                   #   tools/* layout, semantic commits (AGENTS.md)
├── code-review/SKILL.md            # + references/{security,compatibility,performance,
│                                   #   correctness,testing}.md; review unit =
│                                   #   git diff main...HEAD
└── test-strategy/SKILL.md          # unit vs integration expectations, coverage bar
```

### tools/orchestration extension

```
tools/orchestration/
├── prompts/
│   ├── research.md  proposal.md  proposal-revise.md  proposal-final.md
│   ├── design-review.md            # also serves editorial-review via {{scope}}
│   ├── test-authoring.md  test-update.md  implement.md  fix.md
│   └── code-review.md
└── src/pipeline/
    ├── states.ts                   # transition table; guards by gate name
    ├── gates.ts                    # pure guards + round-end verdict aggregation
    ├── prompts.ts                  # template load/render + transcript archiving
    ├── stages.ts                   # stage -> agent id -> template -> reply schema
    │                               #   (+ per-stage `steps` override)
    ├── runner.ts                   # sequential lane; fresh session per stage;
    │                               #   one session at a time; testsGreen pre-check;
    │                               #   per-stage commits; run lock; ADR flip
    └── state-store.ts              # .pipeline.json checkpoint + resume
```

CLI:

```sh
pipeline run <source-path> [--stage <name>]   # run/continue a pipeline
pipeline resume <design-slug>                 # resume from checkpoint
pipeline status <design-slug>                 # where it is and why it's stuck
```

`--stage <name>` reruns that stage against the current artifacts for
human-directed retries; ordinary continuation is `resume`.

Exit codes are scripting-friendly; non-zero on `escalated`. Built on the
existing `opencode-runner.ts` primitives (agent-bound sessions,
Zod-validated JSON replies).

### Settled config defaults

- `blockingSeverities: ["blocker", "major"]`
- `maxRounds: 5` per review stage
- `stepsDefault: 40`, overridable per stage in `stages.ts`; exhaustion is a
  hard failure (fail-loud rule): one stage retry, then escalate
- Transcripts committed to the repo (accepted churn in design PRs)
- Deterministic review order within a round (for reproducibility, not
  short-circuiting): correctness -> compatibility -> security -> performance
  -> complexity

## Alternatives considered

**Agent-driven orchestration** (a primary orchestrator agent launching
`reviewer`/`worker` subagents): rejected because pipeline semantics — round
budgets, review completeness, test gating — would depend on model
compliance and drift on every model swap, which the local-inference
constraint makes frequent.

**Parallel subagent fan-out** for the five reviews per stage: rejected
outright; the inference engine has capacity for exactly one task at a time,
so fan-out only adds failure modes.

**Intra-round short-circuiting** (stop the review queue at the first
rejecting specialist): rejected — reviewers evaluate orthogonal
specialties, and a partial round starves the designer of the complete
change set, so every specialty runs every round and the round verdict is
only computed at round end.

**Conversation-memory hand-offs** (stage results returned as subagent
replies): rejected because subagents start with fresh context, and because
in-repo artifacts are the auditable record the ADR chain requires.

**Design artifacts outside the repo**: rejected — the design history must be
versioned alongside the code it describes.

## Consequences

Becomes possible: identical pipeline behavior across model swaps; full
audit trail per feature (ADR -> design -> reviews -> transcripts -> code),
each transcript anchored to model, agent, timestamps, and inference
parameters; crash-safe resume on slow local runs; `docs/adrs/` grows into a
trustworthy record of what the pipelines were tasked with and what they
decided; tests encode the design's intent before any implementation exists,
so reviewers check the code against the design rather than against the
coder's assumptions.

Costs and risks accepted: every reviewer runs every round, so a stubborn
review stage can burn up to 5 rounds x 5 reviewers = 25 reviewer sessions
(the design stage can reach 30 with the editorial confirmation round) on
the local engine; test-first ordering lengthens the design-to-merge path and
can produce failing tests that sit red until `implement`; 16 agents, 7
skills, and 10 stage templates become first-class surfaces that need review
like code; small-model output variance is absorbed by checklists and the
retry/escalate budget rather than solved; transcripts add churn to PRs that
touch designs.

Deferred (not in v1): release/triage agents, ADR auto-drafting, per-stage
model routing (`stages.ts` keeps the stage->agent mapping so a stage can
point at a different endpoint later without other changes).
