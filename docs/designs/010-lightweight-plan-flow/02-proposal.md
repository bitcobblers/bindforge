# Design: A lightweight `plan → implement → code review` flow beside the full pipeline

## Context

[ADR-010](../../adrs/010-lightweight-plan-flow.md) adds a second selectable **flow** to the
pipeline: `light` — one `plan` stage, the existing coder, and a two-seat code review —
declared in a new `tools/orchestration/src/pipeline/flows.ts` and selected by `flow: light`
frontmatter on the source document. Today `decideNext` starts every run at `research` with no
branch on `source.kind` (A1), so a brief takes the same twelve-stage, five-seat path as an ADR, and
the six completed runs spent 43–96 stage sessions each (research: Performance baselines).
This design keeps one state machine, one runner, one ownership matrix, one checkpoint grammar,
and makes the **stage list, review seats, budgets, plan limits, and document-size meter**
declarable per flow. `flow` defaults to `full`, so every existing document and checkpoint
behaves exactly as it does today.

## Goals / Non-goals

- Goals:
  - A `light` flow that runs `plan → implement → code review (2 seats) → done`, with `fix`
    bounded at 2 rounds, and sharing the lock, per-stage commits, checkpoint/resume, escalation,
    transcripts, and reply contract unchanged.
  - The flow chosen by `flow: light` frontmatter, validated before the lock and before any
    inference, stamped into `.pipeline.json` so a resume needs no flag.
  - Budgets, seats, stage list, doc-size meter stages and plan limits expressed as registry
    data, resolved once from `state.flow`.
  - Light `implement`/`fix` owning test paths, so the `test-update` routing loop disappears from
    small jobs.
  - Two deterministic stops for a too-big light run: oversized plan, budget exhaustion.
  - `pipeline status` and `ESCALATION.md` naming the flow and the effective numbers.
- Non-goals:
  - No second runner, no second checkpoint grammar, no second ownership matrix.
  - No flow auto-detection, no mid-run promotion with state migration, no docs-only flow,
    no per-flow model routing (ADR-010 defers all four).
  - No guard against an operator marking a wide-consequence change `light`; the two
    deterministic stops and the operator are the control.
  - No change to the full flow's stages, seats, budgets, escalation text, or `version: 1`.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | `decideNext`'s `case ''` returns `stage('research')` unconditionally; `source.kind` is recorded in every checkpoint and read by no transition | verified | `tools/orchestration/src/pipeline/states.ts:86-99` |
| A2 | Funnel seat lists come from `funnelSpecialties(phase)` (5 per phase) and are used by `dissentingSeats`, `staleSeats`, `seatsConverged`, `planPass`, `seatSummary`; the pass loop itself takes seats from `planPass` | verified | `panel.ts:31,55-80,100-133`; `runner.ts:1914,2445-2457` |
| A3 | `PipelineConfig` grows additively by optional fields, with the default in `DEFAULT_CONFIG` ("Optional so existing configs keep compiling") | verified | `stages.ts:44-64,75-84` |
| A4 | Budgets have three resolution seams: CLI spread, `opts.cfg ?? DEFAULT_CONFIG` in `runPipeline`/`resumePipeline`, and the pass loop's `?? DEFAULT_CONFIG` | verified | `cli.ts:212,225`; `runner.ts:2068,2126,1940-1943` |
| A5 | The design-phase revision stamp is the `02-proposal.md` blob id; the code-phase stamp is `HEAD` | verified | `runner.ts:2416,2419,199-204` |
| A6 | The size meter runs only for stages in `DESIGN_DOC_STAGES`, at stage-ok/pre-commit, and reads `02-proposal.md` | verified | `runner.ts:161-188,1655-1656` |
| A7 | Today's size policy warns and deliberately does not gate: "a hard size limit buys compressed prose, which is harder to review than long prose" | verified | `report.ts:196-216` |
| A8 | `stagePathAllowed(stage, path)` takes no flow; `namesReservedSegment` runs first and denies checkpoint/lock/escalation/`saveState`-tmp names; `implement`/`fix` are denied `TEST_PATH_RE` | verified | `gates.ts:283-316` |
| A9 | Move validation checks both endpoints through the same ownership predicate, so a flow parameter reaches `moves.ts` too | verified | `moves.ts:149-162` |
| A10 | Frontmatter parsing is a CRLF-tolerant regex scan (no YAML); a brief's only rule is "non-empty" | verified | `gates.ts:33-78` |
| A11 | The checkpoint grammar is `version: z.literal(1)` with "new fields are added optional with behavior-safe defaults", backfill at resume only where absence changes behavior, and golden fixtures per era | verified | `state-store.ts:37-64`; `tests/pipeline/fixtures/checkpoints/` |
| A12 | `plan` is in no stage registry; `codereview/code-correctness` and `codereview/code-testing` agents and the `code-review.md` template already exist, as do both review-skill references; all four document-stage primary agents — `researcher`, `designer`, `editor` (`proposal-final`), `doc-sync` — deny `.pipeline.json`, `transcripts/**` (read and edit) and `shell: "*"`; `doc-sync.md:23-25` is the one exception, an allow for `git diff*` | verified | `stages.ts:101-150`; `.opencode/agents/codereview/`; `.opencode/skills/code-review/references/`; `.opencode/agents/research/researcher.md:4-22`; `.opencode/agents/design/designer.md:4-22`; `.opencode/agents/design/editor.md:4-22`; `.opencode/agents/ops/doc-sync.md:20-25` |
| A13 | The coder's agent config allows `edit "*"` and never denies `tests/**`; the "never edit test files" rule is prompt prose plus the runner's ownership sweep | verified | `.opencode/agents/code/coder.md:5-9,49-51` |
| A14 | Three templates hard-code "Never edit test files" with a `testsNeedUpdate` escape routed to the test-authoring stage, so test ownership needs one rendered variable to stop the prompt contradicting the light matrix | verified | `prompts/implement.md:4`; `prompts/fix.md:4-5`; `prompts/fix-tests.md:8-10,14` — the last selected by `stageTemplate` for any `fix` entered with `testsGreen: false`, with no flow branch (`runner.ts:1200-1208`) |
| A15 | `escalationPhase` maps any stage not in `CODE_STAGES` to the **design** phase, so a light run stopped at `plan` would be reported as a design-phase failure it cannot have | verified | `report.ts:35-57,218-241` |
| A16 | The review-session cap is checked at pass entry, before `planPass`'s `maxPassesPerRound` backstop; `maxRounds` counts mutating design-revise/fix rounds, not review cost | verified | `runner.ts:2354-2413`; `stages.ts:27-40`; `panel.ts:113-120` |
| A17 | `NNN` is allocated from the union of `docs/adrs` and `docs/designs`; `identityClash` treats an existing design dir, its checkpoint, or the branch as taken, before the lock | verified | `identity.ts:57-72,94-123`; `runner.ts:2072-2081` |
| A18 | `sourceAdmissible` runs **after** the lock and after the entry checkpoint save, so "denied before the lock" needs a new pre-lock validation point | verified | `runner.ts:2089-2117` |
| A19 | `run` already takes `--force` ("Start even if the run identity is already taken"); no promotion machinery exists | verified | `cli.ts:126-130` |
| A20 | No new platform surface: the flow adds a stage, a seat list and a config record; the Node floor is `>=22.23.4` | verified | `tools/orchestration/package.json:30-32` |
| A21 | Fewer review seats is not less quality for small work: a lightweight review takes under 20% of a formal inspection's time and finds as many bugs; Google's practice is one reviewer, one LGTM, "small" ≈ 200 lines | verified | https://smartbear.com/learn/code-review/best-practices-for-peer-code-review/ ; https://abseil.io/resources/swe-book/html/ch09.html |
| A22 | A capped plan page is a known shape for incremental work (Google's 1–3 page "mini design doc"), and a change's size grows with the number of files it spans | verified | https://industrialempathy.com/posts/design-docs-at-google/ ; https://google.github.io/eng-practices/review/developer/small-cls.html |
| A23 | 80 plan lines is enough for the light flow's target class (one CLI flag, a test-directory layout chore) | unverified | run 004's scope (`docs/designs/004-mirror-src-hierarchy-in-tests/.pipeline.json`) + A22; measured after the first light runs |
| A24 | A files-spanned ceiling on the plan needs a number this repo does not have | unverified | A22 treats files-spanned as the size signal; no repo datum measures it — Rollout 6 is the data step (Deferred) |
| A25 | A 12-minute `stageTimeoutMs` is enough for the light flow's `plan`/`implement` stages | unverified | per-stage durations were never aggregated (research: Performance baselines counts runs, not stages); the knob is a registry value, adjustable without a code change |
| A26 | After `--force` promotion the seat stamp keeps meaning: the full `proposal` stage rewrites `02-proposal.md`, so the light plan survives only in branch history | unverified | `runner.ts:2416`; `docs/adrs/010-lightweight-plan-flow.md` Consequences |
| A27 | `--extend-budget` is state-side, not config-side: it writes `state.extensionSessions`/`extensionRounds`, consumed at the pass-entry cap check as an over-cap allowance, and reaches no `PipelineConfig` key | verified | `runner.ts:2095-2097,2149-2151,2357-2371`; `state-store.ts:78-79` |
| A28 | The predicate's only intake is `enforceOwnership`'s path set — `nulPaths(diff-tree --name-only -z)` plus `dirtyFiles` (`status --porcelain=v1 -z`), and `moves.ts`'s declared pair: repo-root-relative with POSIX separators, so a path with a `.`/`..` segment or an absolute spelling cannot reach it; git reports the real path (`docs/../src/x.ts` arrives as `src/x.ts`), and declared moves reject absolute, backslash, `.`/`..` shapes as usage errors | verified | `runner.ts:221-228,261-279`; `moves.ts:107-131,155-162` |

## Options considered

**Seats in `PipelineConfig` vs. threading a seat list through the panel.** A seat list threaded into
the funnel from outside needs a new parameter at each of the six `funnelSpecialties` call sites (A2),
leaves the panel width outside the effective-config check the ADR demands, and leaves a stale
`CODE_SPECIALTIES` import in a light run. The adopted option is an optional
`reviewSeats: { design, code }` (A3's additive shape), read through the one resolver and threaded to
its readers (Budget resolution), so the seats and the budgets cannot disagree after a resume. A panel
that lives in config is untrusted data, so the resolver that reads it validates it.

**Plan limits in the flow registry vs. the `plan-note` skill.** The gate is code — agent
permissions and skill text are advisory (A13) — so the numbers live in the registry's config
partial and the prompt interpolates them. A cap in the skill alone is a wish; a cap in both is two
authorities that drift. The skill carries the template and no number.

**The ADR's rejected alternatives, adopted unchanged here.** ADR-010's Rejected section owns that
record — config-only variants, no plan artifact, keeping the test-authoring engineer, a second
runner, `lane` as the field name — each rejected there on the ownership-matrix and session-count
grounds; this design adopts the ADR's choice without re-arguing it.

## Chosen design

Two flows over one state machine. `flow: 'full' | 'light'`, default `full`; the word `lane`
stays reserved for inference width (`--max-parallel`).

### Flow registry — `tools/orchestration/src/pipeline/flows.ts` (new)

```ts
export type Flow = 'full' | 'light';

export interface FlowDef {
  name: Flow;
  /** Stage names this flow may run, in declaration order (documentation + `--stage` gate). */
  stages: readonly string[];
  /** Stages whose product `proposalSize` measures (replaces `DESIGN_DOC_STAGES`, A6). */
  docStages: readonly string[];
  /** The budget/seat partial applied over `DEFAULT_CONFIG`. Empty for `full`. */
  config: Partial<PipelineConfig>;
}

export const FLOWS: Record<Flow, FlowDef>;

/** Absent key reads `full`; the single read seam for `state.flow` (ADR-006 §2). */
export function flowOf(state: Pick<PipelineState, 'flow'>): Flow;

/** The one budget-resolution function — see "Budget resolution" for the rule. */
export function configFor(flow: Flow, base?: Partial<PipelineConfig>): PipelineConfig;

/** The one seat-panel resolver; `funnelSpecialties` and `specialtiesFor` delegate to it. */
export function seatsFor(phase: ReviewPhase, cfg: PipelineConfig): readonly string[];

/** Rejects a `--stage` value the flow does not declare; a usage error, before the lock. */
export function assertStageInFlow(flow: Flow, stage: string): void;
```

`light` declares `stages: ['plan', 'implement', 'fix', 'code-review']`,
`docStages: ['plan']`, and the config partial below; `full` declares today's twelve stages,
`docStages: ['proposal', 'proposal-revise', 'proposal-final']`, and an empty partial, so the full
flow's behavior is untouched (`configFor('full', base)` is deep-equal to a **complete** `base`, #3).
The stage/agent/template rows stay in `STAGE_BASES` (`stages.ts:101-138`); `plan` joins it as
`{ kind: 'proposal', agent: 'design/planner', template: 'plan.md' }`, reusing the `proposal` reply
schema — no new `StageKind`, no new reply contract.

### Budget resolution — one canonical rule

`configFor` is the **only** place flow budgets are computed, used by `runPipeline`,
`resumePipeline` and `pipeline status`:

```ts
configFor(flow, base) = validated({
  ...DEFAULT_CONFIG,                        // remains the sole default authority (A4)
  ...base,                                  // CLI: { ...DEFAULT_CONFIG, maxParallelReviews }
  ...FLOWS[flow].config,                     // flow overrides; empty for full
}, flow)                                    // the seat rule is scoped to FLOWS[flow].stages
```

No flow partial sets `maxParallelReviews`, so the spread order alone applies the operator's
`--max-parallel` last — the ADR's requirement, pinned by #2.

`validated` is part of the contract: the seat panel is caller data (A4's `opts.cfg` seam), so a
resolved config must name only known specialties, must keep `planMaxLines` **absent or** a positive
integer (the full flow's config keeps it absent), and must keep `reviewSeats` non-empty **for every
phase the flow's `stages` runs** — `code` for every flow, since every flow runs a code review, and
`design` under `full`, whose `editorial` round reads it as `panel.ts:31-32` does today. A flow with
no design stage resolves the registry's `design: []` as written — registry data, not a tunable (#23).
A violation throws as a usage error, and a caller-supplied `opts.cfg` is validated **pre-lock** —
`validated` runs on it in `runPipeline`/`resumePipeline` once the flow is known, before `acquireLock`
and `freshState`, the posture of `assertForceableStage` (Selecting the flow, #17). Unvalidated,
`reviewSeats.code: []` makes `seatsConverged` vacuously true (A2): `code-review` "converges" with
zero sessions and a board that prints no dissent, and `['corectness']` launches a seat no agent backs.

`--extend-budget` never reaches this function: it writes `state.extensionSessions` /
`extensionRounds`, consumed at the cap checks (A16, A27), so no last-writer rule is needed here.

The `light` partial is a documented partial record, so unlisted knobs are shared by construction:

| Knob | `full` (from `DEFAULT_CONFIG`) | `light` (partial) |
| --- | --- | --- |
| `maxRounds` | 5 | 2 |
| `maxReviews` | `{ design: 40, code: 20 }` | `{ design: 0, code: 6 }` |
| `maxPassesPerRound` | 6 | 3 |
| `stageTimeoutMs` | 30 min | 12 min |
| `reviewSeats.design` | the 5 design specialties | `[]` (no design phase) |
| `reviewSeats.code` | the 5 code specialties | `['correctness', 'testing']` |
| `planMaxLines` | absent (no gate) | 80 |
| `blockingSeverities`, `pollIntervalMs`, `requestTimeoutMs`, `minNudgeIntervalMs` | `DEFAULT_CONFIG` | shared, not overridden |
| `maxParallelReviews` | 1 | shared; `--max-parallel` applied last |

`reviewSeats` and `planMaxLines` are new **optional** `PipelineConfig` fields (A3), so every
existing config literal keeps compiling. One resolver reads the panel — `seatsFor(phase, cfg)` in
`flows.ts` — and `funnelSpecialties` (`panel.ts:31-32`) and `states.ts`'s byte-identical
`specialtiesFor` (`states.ts:25-26`) delegate to it, so the `reviewSeats` read and the five-seat
fallback live once, in one test. Its five seat-list readers take the `cfg` their callers already hold:
`dissentingSeats`, `staleSeats`, `seatsConverged`, `seatSummary` (`panel.ts:55,65,74,181`) gain the
parameter; `planPass` (`panel.ts:100`) passes the `cfg` its signature already carries to its two
sites (`panel.ts:103,104`), so a light flow's full pass launches the flow's two seats and no call site
keeps a `*_SPECIALTIES` import. `states.ts:152` keeps `DESIGN_SPECIALTIES` (editorial only, Risks row 4).

**Which knob binds in the light code phase (the ADR's budget-table arithmetic).** A round's passes
are the funnel's own (State graph delta), and a dissent keeps the **same** round (`states.ts:83`),
so with two seats a non-converging round is at most `maxPassesPerRound` passes of one or two
sessions — **4–6 sessions** — and the last pass a round can enter starts with 4 sessions spent.
`maxPassesPerRound: 3` is the knob that binds, and it prints the same record on **every** shape of
the round it starts: the pass-backstop string at 6 sessions (both seats dissenting), 4 (one dissenter)
(`panel.ts:113-119`).
The session cap is checked at **pass entry**, ahead of that backstop (A16), so inside the first code
round the backstop prints first. The cap is phase-**cumulative** — `reviewSessionsUsed` counts every
`code-review:` result in the run (`panel.ts:50-51`) — so it binds at the entry of a *later* code
round: a `--stage code-review` force run after a round that died to the backstop at 6 spent sessions
escalates at `runner.ts:2362-2368` with the light suffix (State graph delta). `maxRounds: 2` bounds
the mutating `fix` loop, on the runner's revise-round record. `maxReviews.code: 12` is not chosen:
every round is bounded in the 4–6 band, and 12 buys a fourth pass in a flow whose point is the fifth
seat's absence.

### Selecting the flow

- `parseSource` learns one key, `flow` (A10): a CRLF-tolerant, **case-sensitive** match against
  `full`/`light`. Absent means `full`.
- The value is validated **before the lock and before any inference** (the ADR's requirement,
  which the existing gate position cannot meet — A18). `runPipeline` parses the source text once,
  right after `deriveIdentity`, and carries the parsed frontmatter on the `deriveIdentity` result;
  a `flow:` key with an unreadable value throws
  `invalid flow "<value>" for <source>: expected full or light` — the pre-lock posture of
  `assertForceableStage` (a typo is a usage error, never a half-started run: no lock, no design
  directory, no session). Unreadable files keep their existing `sourceAdmissible` denial.
- `state.flow` is set from **that one parsed value** immediately after `freshState`, so the entry
  save carries it (`runner.ts:2102`), and the post-lock `sourceAdmissible` gate takes the same
  result as an argument instead of re-reading: one frontmatter read per entry, so the value that
  cleared the pre-lock check is the value stamped and printed. A resume reads it through `flowOf`;
  no flag, no backfill — an absent key is `full` at the read seam (A11).
- `--stage` is checked against the flow: `assertStageInFlow(flowOf(state), forceStage)` at both
  pre-lock validation points, so `pipeline resume <light-slug> --stage research` is a usage
  error rather than a `01-research.md` inside a light run.
- `StateSchema` gains `flow: z.enum(['full', 'light']).optional()`; `version` stays `1`, nothing
  is removed, retyped, renamed, or re-interpreted. `PipelineState` gains `flow?: Flow`.
- `light` is legal on either source kind. The gate keys on the frontmatter value, not
  `source.kind`, so an ADR with `flow: light` still needs `status: proposed` and non-empty
  Context/Decision, unchanged (A10).

### State graph delta — `states.ts`

Three touch points; every full-flow `case` is untouched, because a light run can only ever land
on `plan`, `implement`, `fix` and `code-review`.

| Transition | `full` | `light` |
| --- | --- | --- |
| `case ''` | `stage('research')` | `stage('plan')` |
| `case 'plan'` (new) | unreachable | `stage('implement')` |
| `case 'implement'`/`'fix'` with `testsNeedUpdate` | `stage('test-update')` | guard: flow must be `full`, so it falls to `testsGreen`/`fix` — no test engineer exists in light, and no test path belongs to anyone else |
| `case 'code-review'` converged | `stage('doc-sync')` | `{ kind: 'done' }` |
| `case 'code-review'` dissent, budget left | `stage('fix')` | `stage('fix')` (shared) |
| `case 'code-review'` backstop exhausted | escalate, naming the fix-round backstop | same message **plus** `the light flow (flow: light); promote with flow: full and re-run --force` |

The funnel (`panel.ts`) runs unmodified on the light seat list: full pass with both seats,
revision to the dissenters only, ratification for stale approvals. Its deadlock stop
(`reviseNoChange` with named dissenters, A2) is seat-count-agnostic and fires at 2 seats; with
`maxRounds: 2` it is likelier to be the terminal record, which is the reading the ADR asks for.
The light suffix is appended at **one** site — `finalizeEscalate` (`runner.ts:2845`) — gated on
`flowOf(state) === 'light'` and on `state.lastStage` in `implement`/`fix`/`code-review`: one append
covers every escalation those stages reach (the pass-backstop and fix-round strings as `next.reason`
at `runner.ts:2229`, the round-entry record `runner.ts:2387`, the session-budget escalation
`runner.ts:2362`). The **stage** is the discriminator, not `escalationPhase(state)`, which reports a
light `plan` stop as `code` after the Reporting change. No `states.ts` string changes (#8).

### The `plan` stage

- Agent `.opencode/agents/design/planner.md` (new): `design/designer`'s permission list verbatim
  (`designer.md:4-22`, shape in A12) — copied, not re-derived, and `ops/doc-sync`'s `git diff*`
  allow is not part of it.
- Template `tools/orchestration/prompts/plan.md` (new) is a three-line pointer in the shape of
  `prompts/proposal.md`, naming the output `{{designDir}}/02-proposal.md` — the full flow's path,
  which keeps the design stamp, the `docLines` meter, the reviewers' faithfulness check and the
  `reviews/code/` paths working (A5).
- Skill `.opencode/skills/plan-note/` (new) carries the five sections **once**, in
  `plan-note/references/template.md`: problem, chosen approach, files to touch, test plan, done
  criteria. It states no number — `{{planMaxLines}}` is interpolated into the prompt from the
  registry, superseding ADR-010's "the `plan-note` skill carries the template and its length cap".
- `stageVars` gains `planMaxLines` and `testOwnership`, the latter rendered from the flow into
  `implement.md:4`, `fix.md:4-5` and `fix-tests.md:8-10,14` in place of the hard-coded "Never edit
  test files" (A14): light says the coder owns the test files and that no test engineer exists in
  this flow, full repeats the standing rule and the `testsNeedUpdate` escape — one variable, one
  source, covering the `fix` entered from a red `testsGreen` gate too (#11).
- Size meter: `DESIGN_DOC_STAGES` becomes `FLOWS[flowOf(state)].docStages` at the call site
  (A6), so `plan` produces `docLines`/`docLinesAdded`/`docLinesRemoved` and light runs show a
  proposal-size line in `status`.
- Oversized-plan gate, in `gates.ts`, at the same stage-ok/pre-commit point where `proposalSize`
  runs (the stage cannot report its own size): `planOutgrewFlow(lines, cfg)` denies when the plan's
  line count — the JS count at `runner.ts:179-180` (A6) — exceeds `cfg.planMaxLines`, escalating
  `plan outgrew the light flow (N lines, cap 80): set flow: full and re-run with --force`. That is
  the ADR's one plan stop; the files-spanned stop is Deferred (A24).
- Why the cap gates here and warns in the full flow (A7 reverses, deliberately): that lesson is about
  a **design document that grows across revise rounds**, where a hard limit buys compressed prose;
  the plan is a fixed five-section template written once, before any code, judged by two seats that
  are not design reviewers — over length is a scope signal, so the response is a stop naming the
  full flow, and the full flow's policy is untouched.

### Ownership — the security spine, widened in one flow

`stagePathAllowed(stage, path, flow = 'full')` (A8): `namesReservedSegment(path)` **stays first
and wins in both flows** — checkpoint, `saveState` tmp, escalation marker, lock — then light
`implement`/`fix` may write test paths, full keeps `!inTests`, and test-engineer stages and
document stages are unchanged; `moves.ts`'s `ownershipIssue(stage, pair, flow)` and the sweep's call site
(`runner.ts:279`) thread the same value (A9, #7). The widening covers exactly `implement`/`fix`.
The new name `plan` is governed by the
**document-stage** rule unchanged — `return inDocs`, `inDocs = /^docs\//` (`gates.ts:304,315`) — so
its code surface is the `docs/**` every document stage already has, **not** the run's directory: no
run-scoped branch, no new predicate. A light `plan` that rewrites `docs/adrs/**` (the decision
record) or a sibling run's design directory is therefore **not** reverted — that is an accepted
widening (Risks, Ownership row), pinned as **allowed** in #7, not a confinement this design claims.
Paths reach the predicate from the intake site of A28 — git plumbing output: repo-root-relative,
POSIX separators, no absolute path and no `.`/`..` segment, shapes `moves.ts` Rule 1 rejects outright —
so the `flow` parameter parses no path; case is **not** normalized — as today.

### Reporting

- `escalationPhase`: `plan` joins `CODE_STAGES` in `report.ts` **for reporting only** (A15) — the
  phase answers which budget and which report; the gate answers which paths, and Ownership governs
  the same name by the document-stage rule. A light run has one review phase, and `plan` is not a
  design-stage failure — without this, ESCALATION.md reads "in the **design**
  phase" for a run that has no design phase.
- `statusLines` and `escalationMarkdown` print one line from `configFor(flowOf(state))` —
  `flow:    light (plan → implement → code review, seats: correctness, testing; rounds 2, code reviews 6)`
  — and `seatSummary(state, phase, cfg)` prints exactly the flow's seats: two rows in a light board,
  not three `pending` ones. The **effective** numbers come from the same `configFor` the runner uses
  (Budget resolution).

### Promotion

Operator action, no state migration (ADR-010): set `flow: full` in the source document and
re-run with `--force` on the same identity (A19). `identityClash` reports the light run's
checkpoint and branch, `--force` proceeds, the full `proposal` stage rewrites `02-proposal.md`,
and the light plan survives in the branch's git history (A26 — accepted, see Risks). Light runs
consume `NNN` from the same counter as decisions (A17), and `nextNnn` and `identityClash` need no
change: a light design directory is already "taken" by both checks.

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Two seats are the whole net for a plan nobody reviewed (no design panel, no editor, no editorial round) | medium | high | Seats are `correctness` + `testing`; the plan gates are deterministic; `docs/workflows.md` names the class that must not go light; promotion is one frontmatter edit |
| Operator marks a wide-consequence change `light` (a codec change, a public schema edit) | medium | high | The 80-line plan gate stops a plan that outgrew the flow; the 2-round budget stops a long fight early; nothing in the tool can prove intent — the ADR's accepted cost (Deferred) |
| Ownership spine gains a flow parameter and the reserved-name precedence regresses; and `plan`'s code gate is the unchanged document-stage rule, so a light `plan` can rewrite `docs/adrs/**`, `docs/prompts/**` or a sibling run's design directory unreverted | medium | high | Precedence is the first rule in the function and the first test in the ownership matrix, asserted for every stage class in **both** flows (Test strategy #7); the `docs/**` write surface is pinned as **allowed** there and recorded here, not claimed as a confinement no code performs; ADR-010's Consequences records the widened surfaces |
| Budget drift on resume: a resumed light run inherits `maxRounds: 5` / `code: 20` and looks healthy (A4) | medium | high | Budgets resolve in one function, `configFor`, used by run, resume **and** status; the effective-config resume test is Test strategy #15. Residual: a light call site handed a literal that is not `configFor` output falls back to the five-seat panel, so #15 asserts the seat list, not only the numbers (Budget resolution) |
| Flow recorded in two places (document, checkpoint) can disagree after `--force` promotion; the seat stamp changes meaning mid-branch (A26) | low | medium | The checkpoint is authoritative for a running run (`flowOf`); promotion is a new full-flow entry save; the light plan stays recoverable from git history, and the run's transcripts name the agent each stage used |
| The plan cap gates where the repo's stated policy warns, so an operator reads a light stop as size-policing | medium | low | The escalation strings name the full flow, and the design states why the lesson does not transfer (The `plan` stage) |
| 12-minute stage timeout is too tight for a light `implement` (A25) | medium | low | One registry value, no code change to raise; the escalation names the flow, so an operator reads it as a light-flow limit |
| Shared `NNN` counter grows faster; `NNN` stops encoding "this is a decision" | high | low | Accepted by ADR-010; `docs/workflows.md` records that a design directory can be a plan, not only a decision |
| Light coder writes tests with no test engineer, so a plan's weak test story ships unreviewed by a test specialist | medium | medium | The `testing` seat reviews the tests; the plan's `Test plan` section is what that seat checks faithfulness against |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
| --- | --- | --- |
| Node ≥ 22.23.4 (package `engines`), linux | yes | No new platform surface: a stage, a seat list, a config record (A20) |
| Node older than the `engines` floor | no | Unchanged policy: the runner's floor is the same as today's |
| Existing checkpoints with no `flow` key (`version: 1`) | yes | `flowOf` reads `full`; golden era fixtures pin it (Test strategy #16) |
| Existing ADRs and briefs (no `flow` frontmatter) | yes | `configFor('full', base)` is deep-equal to `base`; the twelve `STAGE_BASES` stages, 5 seats, budgets unchanged |
| `flow: light` on a brief (`docs/prompts/**`) | yes | Brief gate stays "non-empty" (A10); claims `NNN` from the shared counter (A17) |
| `flow: light` on an ADR (`docs/adrs/**`) | yes | ADR gate unchanged: `status: proposed` + Context + Decision; the frontmatter key selects the flow, not `source.kind` |
| `flow: Light` / `flow: sideways` (bad value); a caller's `reviewSeats.code: []` panel | no | Pre-lock usage error: no lock, no design directory, no session (Test strategy #17) |
| CRLF source document with `flow: light` | yes | The existing regex scan is CRLF-tolerant; parsing is covered by Test strategy #6 |
| `--max-parallel 1` (default, serial lane) | yes | Two seats = 2 × per-seat wall clock; the knob applies to both flows |
| `--max-parallel 2` on a light code pass | yes | Existing ADR-003 semaphore, 2-seat list; Test strategy #20 |
| `--max-parallel > 2` on a light code pass | yes (no-op) | Seats launch through the semaphore in handed order; 2 seats never fill 3 lanes |
| Windows / macOS | no | Both are outside the repo's tested surface, which is linux; this design adds no platform-specific path, claims no support there, and no test runs there (Test strategy #21). The plan's size meter is the JS line count at `runner.ts:179-180`, not a shell-out, so nothing here adds a Unix dependency |

## Test strategy

1. Unit — registry shape (`tests/pipeline/flows.test.ts`, new, per ADR-004's mirrored tree):
   `FLOWS.light.stages` is `plan, implement, fix, code-review`, `docStages` is `plan`,
   `reviewSeats.code` is `correctness, testing`, `reviewSeats.design` is empty, and
   `FLOWS.full.stages`/`docStages` list today's stages exactly (a regression guard on `full`).
2. Unit — `configFor('light', { ...DEFAULT_CONFIG, maxParallelReviews: 1 })` yields
   `maxRounds: 2`, `maxReviews: { design: 0, code: 6 }`, `maxPassesPerRound: 3`,
   `stageTimeoutMs: 12 * 60_000`, `planMaxLines: 80`, `reviewSeats.code` exactly
   `['correctness', 'testing']`, the shared knobs (`blockingSeverities`, `pollIntervalMs`,
   `requestTimeoutMs`, `minNudgeIntervalMs`) equal to `DEFAULT_CONFIG`'s, and the operator's
   `maxParallelReviews` intact — by spread order alone, with no key written after the flow
   partial (Budget resolution).
3. Unit — `configFor('full', base)` is deep-equal to a **complete** `PipelineConfig` literal
   `base`, and a complete literal without the new optional fields keeps compiling and resolves to
   the five seats (the additive-shape guarantee of A3).
4. Unit — `flowOf`: absent key → `full`, `light` → `light`; `StateSchema` accepts a checkpoint
   with no `flow` key and rejects `flow: "sideways"`; `version` stays `1`.
5. Unit — plan gate: 79/80/81-line plans, the exact escalation string with the cap number, and no
   gate at all when `planMaxLines` is absent (the full flow's config).
6. Unit — `parseSource`/`sourceAdmissible` learn `flow`: absent → `full`; `flow: light` on a CRLF
   document parses; `flow: Light` and `flow: sideways` reach the **pre-lock usage error** —
   case-sensitive, never silently `full` — and the `sourceAdmissible` denial stays for unreadable
   files.
7. Unit — ownership matrix (the security contract, A8/A9): light `implement`/`fix` allow
   `tests/x.test.ts`; the same calls under `full` deny it; every reserved name —
   `.pipeline.json`, the lock, `ESCALATION.md`, a `saveState` tmp — is denied for `implement`,
   `fix`, `plan`, `test-authoring`, `test-update` and document stages **in both flows**; `plan`
   allows **every** path under `docs/` — `docs/designs/**`, `docs/adrs/**`, `docs/workflows.md`, a
   sibling run's directory — the accepted widening, pinned here as allowed (Ownership, Risks) — and
   denies `src/**` in both flows; a POSIX repo-root-relative path gets the verdict of its own
   spelling, and case variants of `tests/`/`docs/` keep today's case-sensitive regex verdict;
   `moves.ts` accepts a test→test pair for light `implement` and rejects it for full.
8. Unit — `decideNext` light transitions from a `flow: 'light'` state: `''` → `plan`, `plan` →
   `implement`, green `implement` → code review round 1, converged → `done`, dissent with budget
   → `fix`, exhausted backstop → the fix-round string as `decideNext` writes it (the light suffix is
   appended at `finalizeEscalate`, so it is asserted on the record — #14), and
   `testsNeedUpdate: true` never routing to `test-update` in light while full's routing is unchanged.
9. Unit — funnel on a 2-seat list: a full pass names both seats, a one-seat dissent returns only
   that seat, a stale approval yields a ratify pass, `seatsConverged` is true when both approve
   the current stamp, and the deadlock rule fires with a 2-seat ledger and `reviseNoChange`.
10. Unit — reporting: `escalationPhase` is `code` for a light run at `lastStage: 'plan'`;
    `statusLines`/`escalationMarkdown` print the flow line and the effective light numbers, and
    `seatSummary` prints exactly two seats; for a full state, every existing line is byte-equal.
11. Unit — `prompts/plan.md` renders with no unresolved placeholder from `stageVars`, and its
    interpolated `planMaxLines` equals the registry value; `implement.md`, `fix.md` **and
    `fix-tests.md`** render the light ownership line — light says no test engineer exists in this
    flow — and the full ownership line with the `testsNeedUpdate` escape.
12. Integration — a light run end to end against a stub client: `plan → implement → code-review
    r1c0 → done` on branch `pipeline/NNN-<slug>`, with `02-proposal.md`, `03-build-notes.md` and
    `reviews/code/<specialty>-r1c0.md` written, and **no** `01-research.md`, no `reviews/design/`,
    no editorial diff, no `doc-sync` commit; total sessions ≤ 2 + seats × 3.
13. Integration — the 2-seat funnel: one seat dissents, `fix` runs, the dissent pass prompts only
    that seat, and the stale approver runs one ratification pass on the current code stamp before
    convergence.
14. Integration — budget binding, three shapes: (a) both seats dissent every pass — escalation on the
    **pass-backstop** record at 6 sessions (not 20), that record carrying the light suffix from
    `finalizeEscalate`, the session-budget string not printing in that round (Budget resolution), and
    a `--extend-budget` resume landing as `extensionSessions` on the state, not in the config
    (A16, A27); (b) one seat dissents every pass — the same record at 4 sessions; (c) resuming the
    (a) run with `--stage code-review` escalates on the **session-budget** record at 6/6 sessions,
    that record carrying the suffix too.
15. Integration — the failure this design can hide: run a light run, checkpoint it, and
    `resumePipeline` with the CLI-shaped `{ ...DEFAULT_CONFIG, maxParallelReviews: 1 }` must give an
    **effective** `maxRounds: 2`, `maxReviews.code: 6` and `seatsFor('code', cfg)` exactly the two
    registry seats — the numbers and panel the drive loop used, not the flow name.
16. Integration — checkpoint compatibility: each pre-`flow` golden fixture in
    `tests/pipeline/fixtures/checkpoints/` loads, resolves as `full`, and its `decideNext` path
    is unchanged; a new light-era golden fixture (with `flow: "light"` and a 2-seat code ledger)
    round-trips through `saveState`/`loadState` at `version: 1`.
17. Integration — pre-lock gates: `flow: sideways`, `--stage research` on a light checkpoint, and a
    caller's `opts.cfg` with `reviewSeats.code: []` fail as a usage error — no lock file, no design
    directory, no entry checkpoint, no `ESCALATION.md`, **zero** client calls (Budget resolution).
18. Integration — ownership in a real sweep: a light `implement` that writes `src/**` and
    `tests/**` keeps both paths; the same stage under `full` has its test paths reverted; a light
    `plan` that writes `src/**` is reverted to its document paths, and one that writes through
    `docs/../src/x.ts` reaches the sweep as git's real path `src/x.ts` and is reverted there (A28 —
    the predicate never sees an un-normalized spelling to match its `docs/` prefix); and a light
    `plan` write to `docs/adrs/**` is **kept** — the widened rule of #7, asserted not reverted.
19. Integration — oversized plan: a stub `plan` writing 120 lines escalates with
    `plan outgrew the light flow (120 lines, cap 80)`, `ESCALATION.md` carries the promotion
    line, the checkpoint is `escalated`, and no `implement` session is spent.
20. Integration — lane knob with 2 seats: at `--max-parallel 2` both seats launch in the handed
    order, one commit lands per pass, and at `--max-parallel 3` the schedule is unchanged.
21. Platform — the suite runs on linux at the `engines` Node floor and `npm run check` stays
    green with no new platform-specific code path (A20); no macOS or Windows job exists, so both
    compatibility rows read `no` and no test claims them.
22. Integration — promotion: a full run with `--force` on a light run's identity completes,
    `02-proposal.md` is the full proposal, and the light plan is recoverable from the branch's
    git history (A26).
23. Unit — `configFor` validation: `reviewSeats.code: []` throws for **every** flow, an empty
    `reviewSeats.design` throws under `full` and **resolves** under `light` (the registry's own `[]`,
    as #1 pins — the check is scoped to the phases the flow's `stages` runs), an unknown seat name
    (`'corectness'`) throws, `planMaxLines: 0` throws where its absence passes, and `flow: 'light'`
    with a `base.reviewSeats` keeps the registry's two seats — partial wins.

## Rollout

1. Land the registry and the resolution seam first: `flows.ts`, the two optional
   `PipelineConfig` fields, `configFor` with its validation contract, `seatsFor(phase, cfg)` and
   the `funnelSpecialties`/`specialtiesFor` delegates to it, `state.flow` in
   `PipelineState`/`StateSchema`, and `flowOf`. With `FLOWS.light` not yet selectable, this is a no-op
   for every run (#3).
2. Add the light stages: `STAGE_BASES.plan`, `prompts/plan.md`,
   `.opencode/agents/design/planner.md`, `.opencode/skills/plan-note/` with its
   `references/template.md`, the `decideNext` light transitions, the `stageVars` additions, and the
   `{{testOwnership}}` line in `implement.md`, `fix.md` and `fix-tests.md`.
3. Add the light-specific enforcement: `stagePathAllowed(stage, path, flow)`, the `moves.ts`
   threading, `planOutgrewFlow`, `docStages` for the meter, the pre-lock `flow` validation,
   `assertStageInFlow`, and `escalationPhase`/status/escalation flow lines.
4. Docs: `docs/workflows.md` gains the two stage paths, the per-flow budget table, the
   `flow: light` frontmatter line, and the promotion recipe. ADR-001's ownership table is not
   rewritten: ADR-010's Consequences records the coder's widened surface.
5. No migration: `version` stays `1`, no checkpoint is rewritten, no document is touched, and
   `flow` defaults to `full`. Light is opt-in, one frontmatter key at a time.
6. Operate: run the first light runs on real small jobs (a flag, a test-layout chore), record
   sessions and wall clock with the ADR's counting rule, and check them against the
   **converging**-run bound — #12's `2 + seats × 3` = ≤ 8 sessions, ≤ 96 min at the 12-min stage
   bound in the serial default — and against run 003's 43 sessions / 2.2 h; a run that escalates
   legitimately sits above it, at ≈ 9 sessions — plan 1, implement 1, the fix rounds, and at most the
   code phase's 6 cumulative (Budget resolution) — plus 1–2 sessions for each `--extend-budget` grant
   the operator buys. The same data settles A23/A25 and the Deferred files-spanned ceiling (A24)
   — all registry values.

## Deferred / out of scope

- Flow auto-detection from the source document — the operator chooses; ADR-010 defers it.
- Mid-run promotion with state migration — ADR-010 defers it; promotion is `--force` re-run.
- A guardrail for "small in lines, wide in consequences" changes — ADR-010 names it operator
  policy and accepts the cost; the line-count stop and the 2-round budget are the gates here.
- A files-spanned ceiling on the plan (`planMaxFiles`) — the ADR fixes two deterministic stops, A24
  carries no datum, and the only count at that point is the planner's prose; Rollout 6 decides it, and
  if it lands it needs a **new** path-token extractor (the ownership predicate extracts nothing).
- Recording the flow per stage in `results` (transcripts already name the agent) — ADR-010's
  Decision does not ask for it; no follow-up yet.
- A docs-only flow and a read-only audit flow — ADR-010 defers them to the registry.
- Per-flow model routing — ADR-010 defers it; light buys stages, not cheaper stages.
- Making the warn-not-gate proposal-size policy of the full flow consistent with the light cap —
  the full flow's policy is deliberately untouched here; no follow-up yet.
- A Windows or macOS CI job to move those compatibility rows from `no` to `yes` — no follow-up
  yet.

## Revision log

- r1: research OQ1/OQ2 + r1c1 correctness (which knob binds; partial vs. full config) — Budget
  resolution: `configFor` is the single resolver over a documented partial; a non-converging round is
  4–6 sessions at 2 seats and prints the pass-backstop record; 12 not chosen.
- r1 + r1c3 + r1c4: complexity + security (the seat panel is unvalidated config data; the flat
  non-emptiness rule made `light` unresolvable; the scoped rule sat in prose, not the canonical
  block) — Budget resolution: `validated(merged, flow)` checks seat names, `planMaxLines`
  absent-or-positive, and `reviewSeats` for the phases the flow's `stages` runs (`code` always,
  `design` under `full`), so the registry's `design: []` resolves (#23). r3 security: a caller's
  `opts.cfg` is validated pre-lock, not at the post-lock read (#17). The cap is phase-cumulative, so
  it binds at a later code round's entry; Rollout 6 measures #12's bound against run 003; #14.
- r1 + r1c3 + r1c4: complexity (the seat rule implemented twice; no route from the readers to the
  panel; the enumeration missed the reader that launches the seats) — Budget resolution:
  `seatsFor(phase, cfg)` in `flows.ts`, its five readers take the `cfg` their callers already hold —
  `planPass` among them, so a light full pass launches the flow's two seats (#9); Options keeps the
  rejected design (a seat list threaded in from outside).
- r1: complexity review (the applied-last knob; sections in two files) — `maxParallelReviews` deleted
  from `configFor`, spread order applies it last; `prompts/plan.md` is a pointer, the sections live
  once in `plan-note/references/template.md`.
- r1 + r1c3: security (Ownership: `plan`'s confinement, the claim vs. the gate, the intake site,
  case variants) — Ownership: the code gate is the unchanged document-stage rule `docs/**`, so a
  light `plan` rewriting `docs/adrs/**` is an accepted widening (Risks), pinned as **allowed** in
  #7/#18, not a confinement asserted without enforcement; A28 names the intake (git plumbing
  output, `moves.ts` Rule 1, asserted in #18) and case stays sensitive.
- r1c1 + r1c3 + r1c4: correctness + complexity (the string enumeration missed a writer; the cited
  discriminator cannot skip the plan stop) — State graph delta: the suffix appends once at
  `finalizeEscalate`, gated on flow and the escalating **stage**, not `escalationPhase` (a light
  `plan` stop reads `code` after the Reporting change); #8, #14.
- r1: research OQ3/OQ4 + r1 review + risk (what the cap measures and where; the meter path; A7) — The
  `plan` stage: the registry owns 80 lines and the prompt interpolates them, the skill carries the
  template and no number (superseding ADR-010's wording), the meter keys on `docStages`, over length
  is a scope signal on a template written once; the 10-file gate is deleted, files-spanned deferred.
- r2 + r3: all four seats + correctness/security (a language pass inverted the deny list into grants;
  the register's document-stage universal met its `doc-sync` exception) — The `plan` stage:
  `planner.md` copies `designer.md:4-22` verbatim; A12 is the one shape authority, exceptions named.
- r1: research risk + r1 review (the light coder owns tests while the prompts forbid it, A13/A14) —
  every template carrying the rule renders `{{testOwnership}}`, `fix-tests.md` included.
- r1: research OQ5/OQ8/OQ9/OQ10 + the pre-lock gate (A18) — Selecting the flow: one pre-lock parse
  threaded to `state.flow` and `sourceAdmissible`, `flowOf` the read seam, no backfill, golden
  fixtures pin legacy-as-full, `light` on an ADR allowed; Reporting resolves through the same
  `configFor`, `plan` joins `CODE_STAGES`, the stage count is only the twelve `STAGE_BASES` rows.
- r1: correctness review (`--extend-budget` lost to the flow partial) — no conflict: the extension is
  state-side (A27), consumed at the pass-entry cap check; #14.
- r1: research OQ6/OQ7 + compatibility review (shared `NNN`; the `--force` guard; the Windows row) —
  deferred, no code change: `nextNnn`/`identityClash` already treat a light directory as taken (A17),
  promotion overwrites the plan (A26), and Windows/macOS read `no` — linux is the tested surface.

Editor notes: Budget resolution — "`states.ts:152` keeps `DESIGN_SPECIALTIES` (editorial only, Risks row 4)" is ambiguous; row 4 of the Risks table is the resume budget-drift row, while the editorial-round exposure is row 1 — author to clarify.
Editor notes: Budget resolution — "`maxRounds: 2` bounds the mutating `fix` loop, on the runner's revise-round record" is ambiguous; author to clarify.

## As-built — 2026-11-12

The approved design remains the normative record; this note records the shipped
change set (`60faf8c24438ed55121757057b578182d2815083...HEAD`) where it goes
beyond, sharpens, or locally departs from the design. The numbers named in the
registry remain the design's: `light` runs `plan → implement → fix →
code-review`, uses the `correctness`/`testing` code panel, caps the plan at 80
lines, and leaves the full flow's twelve stages, five seats, 40/20 sessions,
six passes, five rounds, and 30-minute stage timeout intact.

### Drift, reason, and affected design sections

| As-built behavior | Reason it shipped | Affected proposal sections |
| --- | --- | --- |
| `FlowDef` carries a `path`, and the registry exposes `FlowId`, `flowDef`, `isStageInFlow`, and `SeatPhase = Exclude<ReviewPhase, 'editorial'>`; `configFor` validates both the caller's base and the resolved config, scopes seat names to the phase that names them, and separately requires every phase the flow runs to resolve to a panel. | The operator-facing flow line needs one authoritative path string; untrusted flow text and caller data need one registry gate; an omitted phase key must be filled by the flow partial, not fail with a `TypeError`; `code` and `design` seat namespaces are not interchangeable. | Flow registry; Budget resolution; Test strategy #23; Compatibility matrix. |
| The plan gate refuses a missing plan document (`plan document not written: …/02-proposal.md`), records the refusal as `PipelineState.planStop`, makes `case 'plan'` escalate while that field stands, clears it only when a later `plan` pass passes the gate, and reports/statuses from the record — not from `lastStage: 'plan'`. The refused bytes remain unstaged in the worktree and are tracked by the terminal escalation commit. | A plain `pipeline resume` on the design's `plan → implement` row would build the plan the flow refused; the stage name alone is also true for a healthy run entering `implement` and for a failed `implement` that never reached the plan gate. The design has no mechanism that deletes the stage's document, and promotion needs the refused plan available. | The `plan` stage; State graph delta — `states.ts`; Promotion; Reporting; Compatibility matrix. |
| Light budget accounting charges one mutating round at every `fix` entry in both flows, skips the light code-round entry charge, reads sessions from `reviewSessionsUsed`, ignores `extensionRounds` at `budgetLeft` for `light`, and keeps the shared `codeRound > maxRounds` guard. | This makes the two `fix` rounds the design names reachable on both the red-tests and dissent paths, bounds the light `fix` loop without letting a round grant buy an uncharged third round, and preserves the full flow's round-entry charge. The code review left open that the shared round-number guard can truncate a full-flow over-cap code round; that is a residual departure from the "no change to the full flow" non-goal, not an intended design change. | Budget resolution; State graph delta — `states.ts`; Risks (budget drift); Non-goals. |
| Source intake carries the frontmatter-stripped `body`, the brief gate judges that body, `sourceAdmissible` accepts the caller's `ParsedSource`, and identity derivation takes a flow-scoped light-only `nnnOfExistingRun`; `--stage` is validated against `FlowDef.stages` alone, with `assertForceableStage` retained as a regression guard. | One frontmatter read must be the value validated, stamped, and judged; a frontmatter-only brief is empty; a light source must not mint a second directory beside its own plan; the full-flow stage list excludes `plan`, so the flow registry is the only gate that can accept a light re-plan. | Selecting the flow; Promotion; Compatibility matrix; Test strategy #6/#21. |
| Moves validation first asks `isStageInFlow(stage, flow)` before the ownership predicate; light `implement`/`fix` own the whole `TEST_PATH_RE` after reserved names win, including co-located tests, while light `plan` keeps the accepted `docs/**` document-stage surface. | A stage that a flow does not run cannot declare moves, and the prompt tells the light coder it owns tests; the accepted document-scope widening is not a confinement. | Ownership — the security spine, widened in one flow; Test strategy #7/#18. |
| `renderTemplate` has a fallback `testOwnership` value, and `testOwnershipLine` takes a `Flow`. | Prompt-shape tests and future callers that render coder templates outside the drive loop need the standing full-flow sentence as a fallback; an open `string` can silently render the wrong flow's rule. | The `plan` stage; Rollout 2. |
| Reporting prints `slug:` rather than `run:`, uses `FlowDef.path`, renders plan-stop repairs through `repairSteps`, spells the in-flow repair as `pipeline resume --stage plan <id>`, appends the light promotion suffix only for `implement`, `fix`, and `code-review`, and labels the terminal repair lines. | The flow line and the plan-stop record must print only the moves the state machine honors; a bare `pipeline resume <id>` prefix would invite the operator to re-escalate on the plan stop. The shared repair shape also puts labels on full-flow terminal repair text and leaves the plan-stop promotion path unquoted, both recorded in code review as compatibility gaps. | Reporting; Promotion; Risks; Test strategy #10/#19. |
| The code stamp refreshes on full and dissent passes and keeps the ratify pass's stamp; review-pass ledger entries are sorted into the handed seat order after the barrier. | The design fixes `HEAD` as the code stamp but does not say when it moves. Re-stamping after a report commit ages the approvals a ratify pass is ratifying; a seat's completion order must not make the checkpoint depend on lane width. | Budget resolution; Chosen design (funnel behavior on the light seat list); Test strategy #13/#20. |
| A UTF-8 byte-order mark before a `flow:` frontmatter block is not stripped, so such a document parses as `flow: full`. | This is an unimplemented edge case in the frontmatter scanner, not a design change; it leaves a BOM-authored light brief on the full flow, contrary to the spirit of the bad-value and CRLF compatibility rows. | Selecting the flow; Compatibility matrix; Test strategy #6. |
| A pre-commit gate's refusal writes the stage name to the state (`state.lastStage = stageName` in `executeStage`, `runner.ts:1881`) before the stop is reported, and the plan-size stop writes a `plan-size` gate-log entry for the refusal (`runner.ts:2785`, `2793-2796`) and a second one for the re-plan that clears it (`runner.ts:2381-2387`). | The design fixes the stop's position (the stage-ok/pre-commit point) and its string, and says nothing about the record the stop leaves. `state.lastStage` is otherwise assigned only after `runWithRetry` returns, so a refusal raised from inside the stage reached `finalizeEscalate` with an empty name: `escalationPhase` answered `design` and ESCALATION.md opened "Run stopped after stage `` in the **design** phase" — the exact mislabel A15 adds `plan` to `CODE_STAGES` to remove. The gate log is the only place a reader sees the refusal beside the run's commits, and the clear beside the re-plan's. | Reporting; The `plan` stage (the oversized-plan gate); State graph delta — `states.ts`; Test strategy #10/#19. |

### Open as-built gaps

The latest ratified review set has no open blocker or major finding. It leaves
these non-blocking record and edge gaps: the converged gate-log line says
"all five seats" on a two-seat light run; the plan-stop promotion command
interpolates an unquoted source path; the full terminal footer gained a
repair-step label; a light `--extend-budget` increments `extensionRounds`
that light has no consumer to spend; a full over-cap code round can be
truncated by the shared round-number guard; and the reserved-name and planner
permission tests do not cover every stage/agent surface named by #7.
The package's last executed gate datum recorded in the build notes is 41 files
with 492 passed and 2 platform-skipped; later doc passes in this environment
cannot re-run `npm run check`, so CI remains the standing proof for #21.

### Verification against the shipped tree — 2026-11-12 (second pass)

This pass re-read the change set at `HEAD` against the design and against the
rows above. It changes no design decision, no number and no rule; it adds the
one drift row the first pass had not recorded (the pre-commit stage record and
the `plan-size` gate-log pair) and confirms the rest.

Each row's mechanism was found where the row says it lives:

- Flow registry and budget resolution — `flows.ts`: `Flow = 'full' | 'light'`,
  `FlowId`, `SeatPhase = Exclude<ReviewPhase, 'editorial'>`, `FlowDef.path`,
  `flowDef` as the single registry gate, `configFor` validating the caller's
  base, then the merged config, then panel presence
  (`validateSeatNames`/`validatePlanMaxLines`/`validatePanels`/
  `validatePanelPresence`, scoped by `phasesRun` over
  `FlowDef.stages`), `seatsFor` as the one five-seat fallback, and
  `isStageInFlow`/`assertStageInFlow` in the proposal's parameter order.
  `FLOWS.light` reads `plan, implement, fix, code-review` / `docStages: ['plan']`
  / `maxRounds: 2`, `maxReviews: { design: 0, code: 6 }`, `maxPassesPerRound: 3`,
  `stageTimeoutMs: 12 * 60_000`, `planMaxLines: 80` and the two-seat code panel;
  `FLOWS.full.config` is empty and `DEFAULT_CONFIG` keeps 5 / 40-20 / 6 / 30 min
  (`stages.ts:89-120`).
- Flow selection — `gates.ts:54-124` and `runner.ts:2161-2199`, `2261-2283`: one
  pre-lock parse, `invalid flow "…" for …: expected full or light` before
  `acquireLock`, `state.flow` on the entry save, `sourceAdmissible` taking the
  caller's parse, `configFor` as the only resolver at run, resume and status,
  and `StateSchema` carrying `flow` as an optional enum at `version: 1`
  (`state-store.ts:60-75`).
- State graph — `states.ts:110-258`: `'' → plan` for light, `case 'plan'`
  escalating on the recorded `planStop` and falling to `implement` without it,
  the `testsNeedUpdate` route guarded to `flowOf(state) === 'full'`, a converged
  light panel ending `done`, `budgetLeft` ignoring `extensionRounds` for light,
  and the shared `codeRound > maxRounds` guard.
- Ownership — `gates.ts:371-402` (reserved names win first, light
  `implement`/`fix` own all of `TEST_PATH_RE`, `plan` on the unchanged
  `docs/**` rule) and `moves.ts:157-171` (`isStageInFlow` first); the sweep
  threads `flowOf(state)` at `runner.ts:303`.
- Reporting — `report.ts:36-47` (`plan` in `CODE_STAGES`), `report.ts:252-309`
  (`hasPlanStop` on the record, `repairSteps`' two single-voiced repairs),
  `runner.ts:3160-3167` (the suffix at one site, gated on the flow and on
  `implement`/`fix`/`code-review`), `runner.ts:3289-3335` (the flow line from
  `FlowDef.path`, the `slug:` label, the plan-stop status line).
- The `plan` stage — `STAGE_BASES.plan` (`kind: proposal`, `design/planner`,
  `plan.md`), `prompts/plan.md` naming `{{designDir}}/02-proposal.md` and
  interpolating `{{planMaxLines}}`, `.opencode/agents/design/planner.md`,
  `.opencode/skills/plan-note/references/template.md` carrying the five sections
  and no number, the meter keyed on `docStages` (`runner.ts:1702`), and
  `planOutgrewFlow` at the pre-commit point (`runner.ts:2775-2810`).
- Coder prompts — `{{testOwnership}}` in `implement.md`, `fix.md` and
  `fix-tests.md`, with `TEST_OWNERSHIP_LIGHT` naming the light coder's ownership
  and `renderTemplate`'s full-flow fallback (`prompts.ts:21-44`).
- Funnel — `panel.ts` unchanged in shape, every reader taking the `cfg` its
  caller holds, `planPass` blocking at `cycle >= cfg.maxPassesPerRound` in both
  flows; the pass ledger re-sorted into handed order at `runner.ts:2124-2133`,
  and the ratify pass keeping its stamp at `runner.ts:2620-2627`.
- Docs — `docs/workflows.md` carries the two stage paths, the per-flow budget
  table (6 passes, 30 min), the `flow: light` frontmatter line, the
  `--stage plan` repair and the promotion recipe (Rollout 4).

The seven gaps listed under "Open as-built gaps" all stand, each re-found at
its cited site: the "all five seats" converged reason (`runner.ts:2740`), the
unquoted `state.source.path` in the promotion command (`report.ts:290`,
`298`), the labelled full-flow footer (`runner.ts:3231-3237`), the
light `--extend-budget` incrementing `extensionRounds` light cannot spend
(`runner.ts:2231-2234`, `2294-2303` with `states.ts:78-80`), the non-grant-aware
round guard (`states.ts:229-233`), the BOM case (`gates.ts:55-58`), and the
three coverage gaps (`gates.test.ts:661-672` omits `proposal-revise`,
`proposal-final` and `editorial-review`; `prompts.test.ts:467-474` pins the
planner wiring, not the agent's permission list; `flows.test.ts:323-327` pins
`planMaxLines: 0` and its absence, not a non-integer). No full-flow stage,
seat, budget, escalation string, checkpoint version or ownership rule was found
changed away from the design.
