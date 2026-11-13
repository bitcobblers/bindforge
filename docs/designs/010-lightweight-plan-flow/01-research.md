# Research: A lightweight `plan → implement → code review` flow beside the full pipeline (ADR-010)

## Summary

ADR-010 adds a second selectable flow — `light`: one `plan` stage, the existing
coder, and a two-seat code review — declared in a new `flows.ts` registry and
selected by `flow: light` frontmatter on the source document, stamped into the
checkpoint so resume needs no flag (source: docs/adrs/010-lightweight-plan-flow.md).
Nothing in the current code expresses the concept: the stage graph in
`decideNext` starts every run at `research` with no branch on `source.kind`,
`DEFAULT_CONFIG` is the single budget authority, and no `flow` identifier exists
anywhere in `src/` (source: tools/orchestration/src/pipeline/states.ts, lines
86–99; tools/orchestration/src/pipeline/stages.ts, lines 75–84). The runs on
disk confirm the direction of the ADR's cost claim and not its arithmetic: runs
003–008 spent 352 stage sessions and ≈25.2 h of stage wall clock, 68% of
sessions in review seats and 56% in the design phase the light flow deletes
(source: docs/designs/00{3,4,5,6,7,8}-*/.pipeline.json). The cheap-to-add
parts are real: `runReviewPass` is parameterized by its seat list and the
funnel is pure over the seat ledger and the `02-proposal.md` blob stamp, so a
two-seat panel is data, not new pass code (source: tools/orchestration/src/pipeline/runner.ts,
lines 1900–1943; tools/orchestration/src/pipeline/panel.ts, lines 100–133). The
risk centre is `stagePathAllowed` — the ownership spine ADR-006 §1 and ADR-007
lean on — gaining a flow parameter, plus two seats being the entire safety net
for a plan nobody reviewed (source: tools/orchestration/src/pipeline/gates.ts,
lines 288–316; docs/adrs/010-lightweight-plan-flow.md).

## Findings

### What the current code forces the design to touch

- The graph is a `switch` on `state.lastStage`, and `case ''` returns
  `stage('research')` unconditionally; `state.source.kind` is recorded in every
  checkpoint (`{kind, path}`) and read by no transition, so the ADR's "every run
  starts at research" premise is exactly true (source: tools/orchestration/src/pipeline/states.ts,
  lines 86–99; docs/designs/010-lightweight-plan-flow/.pipeline.json).
- Two declarable structures already exist and the light flow needs a third
  peer of each: the stage registry `STAGE_BASES` maps stage → `agent` (a string
  or a function of the review specialty) → template (source: tools/orchestration/src/pipeline/stages.ts,
  lines 95–138), and the panels are plain arrays, `CODE_SPECIALTIES` being
  `correctness, compatibility, security, performance, testing` (source:
  tools/orchestration/src/pipeline/stages.ts, lines 10–24). A `light` registry
  entry is therefore a stage list + a seat list, and the light code panel needs
  no new agent definitions: `codereview/code-correctness` and
  `codereview/code-testing` are the two names the funnel already prompts, and
  both appear in shipped transcripts (source: tools/orchestration/tests/pipeline/prompts.test.ts,
  line 97; docs/designs/006-self-update-checkpoint-compatibility/transcripts/code-review-correctness-r1c0.md,
  line 5).
- Budgets have exactly one origin today: `DEFAULT_CONFIG` (`maxRounds: 5`,
  `maxReviews: {design: 40, code: 20}`, `maxPassesPerRound: 6`,
  `stageTimeoutMs: 30 * 60_000`, `maxParallelReviews: 1`), spread once into the
  run config at the CLI (`cfg: { ...DEFAULT_CONFIG, maxParallelReviews }`) and
  re-defaulted in `runPipeline`/`resumePipeline` (`opts.cfg ?? DEFAULT_CONFIG`)
  and in the pass loop (`cfg.maxParallelReviews ?? DEFAULT_CONFIG...`) (source:
  tools/orchestration/src/pipeline/stages.ts, lines 75–84; tools/orchestration/src/pipeline/cli.ts,
  lines 212 and 225; tools/orchestration/src/pipeline/runner.ts, lines 2068,
  2126, 1940–1943). Resolving from `state.flow` means every one of those three
  seams changes, and `PipelineConfig` carries knobs the ADR's
  `LIGHT_CONFIG` table does not list (`pollIntervalMs`,
  `requestTimeoutMs`, `maxParallelReviews`), so "override" is a spread over a
  partial record — a hazard for the designer, not an open question for the
  researcher (source: tools/orchestration/src/pipeline/stages.ts, lines 26–84;
  docs/adrs/010-lightweight-plan-flow.md).
- The pass loop is already seat-driven: `ReviewPassArgs` carries
  `seats: readonly string[]` "exactly as `planPass` / `specialtiesFor` handed
  them", and `planPass` computes full / dissent / ratify from the seat ledger
  alone, so a 2-seat panel produces a 2-seat full pass, dissent on at most two
  seats, and ratification of stale approvals with no code change (source:
  tools/orchestration/src/pipeline/runner.ts, lines 1900–1943;
  tools/orchestration/src/pipeline/panel.ts, lines 100–133).
- The artifact-path premise holds: the funnel's revision stamp is
  `state.designStamp = blobStamp(repoDir, `${designDirRel}/02-proposal.md`)`,
  i.e. `git rev-parse HEAD:<path>`, so a plan written at the same path keeps
  staleness detection, the reviewers' faithfulness check, and `reviews/code/`
  paths untouched (source: tools/orchestration/src/pipeline/runner.ts, lines
  199–204 and 2416; docs/adrs/010-lightweight-plan-flow.md).
- The size meter is gated by a stage-name set, not by flow:
  `DESIGN_DOC_STAGES = new Set(['proposal', 'proposal-revise', 'proposal-final'])`
  guards the `proposalSize(...)` call, which reads `02-proposal.md` and reports
  `docLines` plus the HEAD delta (source: tools/orchestration/src/pipeline/runner.ts,
  lines 161–188 and 1655–1656). A `plan` stage is invisible to the meter until
  the set is widened or keyed by flow, so the ADR's "the runner measures the
  document with the existing `proposalSize` meter" names a code change, not a
  free reuse (source: docs/adrs/010-lightweight-plan-flow.md,
  tools/orchestration/src/pipeline/runner.ts, lines 161–188).
- Today's policy for that meter is explicitly warn-not-gate: `proposalGrowthLine`
  warns when `netPositiveStreak >= 3` and the code comment records the reason —
  "a hard size limit buys compressed prose, which is harder to review than long
  prose" (source: tools/orchestration/src/pipeline/report.ts, lines 196–216).
  The ADR's 80-line cap that *escalates* with `plan outgrew the light flow
  (N lines)` is therefore new policy that reverses a stated decision, and the
  string does not exist in the code (source: docs/adrs/010-lightweight-plan-flow.md,
  tools/orchestration/src/pipeline/report.ts, lines 196–216).
- `stagePathAllowed(stage, path)` takes no flow: `namesReservedSegment(path)`
  returns false for checkpoint/lock/escalation/`saveState`-tmp names and runs
  before every per-stage rule, then `implement`/`fix` may touch anything
  *except* `TEST_PATH_RE = /(^|\/)tests\/|\.test\.ts$/`, while
  `test-authoring`/`test-update` own tests plus `docs/**` (source:
  tools/orchestration/src/pipeline/gates.ts, lines 283–316). Light coder
  ownership of tests is a change to the function's parameters and its test
  matrix, and the reserved-name rule must keep precedence in both flows
  (source: tools/orchestration/src/pipeline/gates.ts, lines 288–316).
- The source gate is a regex scan with no YAML dependency: `parseSource`
  extracts `status` from a `^---\r?\n...\r?\n---` block and, for briefs,
  `sourceAdmissible` allows anything non-empty (`deny('brief is empty')` is the
  only brief rule) (source: tools/orchestration/src/pipeline/gates.ts, lines
  33–78). Adding one `flow` key is a one-regex change plus a denial, and the
  ADR's "denial before the lock and before any inference" matches the existing
  fail-fast position of `sourceAdmissible` (source: tools/orchestration/src/pipeline/gates.ts,
  lines 49–78; docs/adrs/010-lightweight-plan-flow.md).
- Checkpoint grammar supports the field exactly as the ADR claims: `StateSchema`
  pins `version: z.literal(1)` and the stated evolution rule is "new fields are
  added optional with behavior-safe defaults; no field is ever removed,
  retyped, renamed, or re-interpreted", with the golden-fixture-per-era
  precedent at `tests/pipeline/fixtures/checkpoints/` (source:
  tools/orchestration/src/pipeline/state-store.ts, lines 37–64). A `flow` key
  is present in no checkpoint on disk, including the newest one, whose 22
  top-level keys include `source`, `baseRef` and an empty `results` (source:
  docs/designs/010-lightweight-plan-flow/.pipeline.json).
- Promotion machinery already exists: `run` takes `--force` ("Start even if the
  run identity is already taken"), plus `--nnn`/`--slug` pinning and
  `--extend-budget` (source: tools/orchestration/src/pipeline/cli.ts, lines
  118–134). Identity is derived from the document path — `kind` is
  `docs/adrs/**` vs everything else, briefs claim `nextNnn` over `docs/adrs`
  *and* `docs/designs`, and the branch is `pipeline/<nnn>-<slug>` (source:
  tools/orchestration/src/pipeline/identity.ts, lines 57–72;
  tools/orchestration/src/pipeline/gates.ts, lines 273–281).
- Naming is clean: `lane` is already the review-width word in the operator docs
  and the CLI (`--max-parallel 3  # review-lane width`), and `flow` appears
  nowhere in the pipeline source as a domain identifier — the six `flow` hits
  in `src/` and `tests/` are prose about control flow (source: docs/workflows.md,
  lines 100–108 and 177–185; tools/orchestration/src/pipeline/runner.ts, lines
  104 and 1921–1927; grep for `\bflow\b` over tools/orchestration/src and
  tests/pipeline). The registry path in the ADR (`src/pipeline/flows.ts`) is
  also shorthand: the package root is `tools/orchestration/` (source:
  docs/adrs/010-lightweight-plan-flow.md; tools/orchestration/src/pipeline/).
- The artifacts the light flow names do not exist yet: `tools/orchestration/prompts/`
  holds 13 templates (`research`, `proposal`, `proposal-revise`,
  `proposal-final`, `design-review`, `code-review`, `test-authoring`,
  `test-update`, `implement`, `fix`, `fix-tests`, `doc-sync`) and there is no
  `plan.md`; `plan` is in no stage registry, and the names `design/planner` and
  the `plan-note` skill appear nowhere in the repo outside the ADR itself
  (source: tools/orchestration/prompts;
  tools/orchestration/src/pipeline/stages.ts, lines 101–138; grep for
  `plan-note` and `design/planner` over the repo — only hits are
  docs/adrs/010-lightweight-plan-flow.md, lines 127–130).

### Performance baselines (lens)

- Measurement rule is the ADR's own: sessions = entries in `results`, wall clock
  = sum of `durationMs` (source: docs/adrs/010-lightweight-plan-flow.md).
  Counting that way over the six completed runs gives 43 sessions / 2.21 h
  (003), 53 / 3.01 h (004), 96 / 7.05 h (005), 43 / 3.88 h (006), 44 / 3.27 h
  (007), 73 / 5.78 h (008) — 352 sessions and 25.19 h in total (source:
  docs/designs/003-parallel-inference-lane/.pipeline.json,
  docs/designs/004-mirror-src-hierarchy-in-tests/.pipeline.json,
  docs/designs/005-runner-executed-file-moves/.pipeline.json,
  docs/designs/006-self-update-checkpoint-compatibility/.pipeline.json,
  docs/designs/007-audit-by-deeds/.pipeline.json,
  docs/designs/008-rust-for-the-product/.pipeline.json).
- Review breadth is the dominant cost, as the ADR argues: seat-named stages
  account for 239 of 352 sessions (68%) — 104 `design-review:*`, 89
  `code-review:*`, 46 `editorial-review:*` (source: the six `.pipeline.json`
  files above, counting `"stage"` values with those prefixes).
- Phase split gives the size of the prize: the design phase (research,
  proposal, proposal-revise, proposal-final, design-review seats,
  editorial-review seats) is 198 of 352 sessions (56%), the code phase
  (implement, code-review seats, test-authoring, test-update, fix) is 154
  (44%), and `doc-sync` is 7. Run 005 is the extreme: 66 of 96 sessions (69%)
  in the design phase, 37 of them `design-review:*` seats (source: as above).
- Where a small job's sessions actually go: run 004, the test-directory layout
  chore, spent 14 of its 53 sessions (26%) in the `test-update`/`fix` loop — 8
  `test-update` and 6 `fix` entries — which is the loop the light flow removes
  by giving the coder the test paths (source: docs/designs/004-mirror-src-hierarchy-in-tests/.pipeline.json;
  docs/adrs/010-lightweight-plan-flow.md). The ADR's figure of "14 on
  `test-update` and `fix`" matches that aggregate.
- The ADR's table is directionally right and arithmetically loose: it credits
  run 005 with "27 sessions on `design-review:*` seats", where the checkpoint
  records 37 (source: docs/adrs/010-lightweight-plan-flow.md;
  docs/designs/005-runner-executed-file-moves/.pipeline.json). The proposal
  should restate the table from a single reproducible counting rule rather than
  inherit it.
- The light flow's own budget table is internally tight and should be checked
  before it ships: 2 rounds × `maxPassesPerRound: 3` passes × 2 seats is 12
  seats against `maxReviews.code: 6` (source: docs/adrs/010-lightweight-plan-flow.md).
  `maxReviews` is documented as the unit an operator buys with
  `--extend-budget`, and `maxPassesPerRound` is the funnel's own backstop, so on
  the current semantics the 6-session cap binds first and the 3-pass cap is
  unreachable in a 2-seat funnel (source: tools/orchestration/src/pipeline/stages.ts,
  lines 26–40; tools/orchestration/src/pipeline/panel.ts, lines 113–120).
- External baselines for "less review is not less quality": the Cisco/SmartBear
  study reports that a lightweight review takes **less than 20% of the time of
  a formal inspection and finds just as many bugs**, that review beyond 200–400
  LOC per sitting degrades defect discovery, and that spot-checking 20–33% of
  changes lowered defect density with minimal time — the measured analogue of
  2 seats out of 5 (source: https://smartbear.com/learn/code-review/best-practices-for-peer-code-review/).
  Google's own practice is one reviewer and one LGTM for most changes, with
  "small" around 200 lines, because "the most important LGTM is the first one,
  and subsequent ones don't add as much as you might think" (source:
  https://abseil.io/resources/swe-book/html/ch09.html). The published
  counter-cost of multi-agent fan-out is a 15× token multiplier over chat, with
  coding named as a domain that "involves fewer truly parallelizable tasks than
  research" (source: https://www.anthropic.com/engineering/built-multi-agent-research-system).
- The saving is not automatic in model terms: pairing a strong reasoning model
  as architect with a cheaper editor is what produced best-known scores at
  85% and then 64% "at 14× less cost" on aider's benchmarks, i.e. cost falls
  when the *role* is matched to a model, and the ADR defers per-flow model
  routing (source: https://aider.chat/2024-09-26/architect.html,
  https://aider.chat/2025-01-24/r1-sonnet.html; docs/adrs/010-lightweight-plan-flow.md).
  A light run on the default routing therefore buys stages, not cheaper stages.

## Prior art (lens)

- Tiered proposal weight, three weights, is a settled convention. Rust
  documents RFC as "a heavy-weight proposal mechanism, reserved for significant
  changes", Major Change Proposal as medium, and a plain PR as "a light-weight
  proposal mechanism… preferred when the proposal is accompanied by a small
  patchset", with per-case tables such as "Adding a compiler option for
  internal-use only — Propose using: PR, Approve using: r+" (source:
  https://forge.rust-lang.org/compiler/proposals-and-stabilization.html). The
  same page shows the FCP being *skipped* when a change is easily reversed —
  the same shape as ADR-010's shrinking of review, not of process (source:
  https://forge.rust-lang.org/compiler/proposals-and-stabilization.html).
- Kubernetes tiers review by declared maturity, and declares the tier in
  machine-read metadata: `kep.yaml` carries `stage`, `latest-milestone` and the
  per-stage `milestone` struct, the enhancements CI enforces a
  `prod-readiness/<sig>/<KEP number>.yaml` approval per stage, and the template
  labels each questionnaire section "must be completed when targeting alpha /
  beta" and "encouraged" for alpha (source:
  https://kubernetes.dev/resources/keps/1194,
  https://github.com/kubernetes/community/blob/main/sig-architecture/production-readiness.md,
  https://github.com/kubernetes/enhancements/blob/master/keps/NNNN-kep-template/README.md).
  Its "at a level appropriate to the features' maturity levels" is the explicit
  rationale ADR-010 needs for `LIGHT_CONFIG` (source:
  https://kubernetes.dev/resources/keps/1194). Two transferable cautions: one
  KEP covers a feature "for its whole lifecycle… you do not need a new KEP to
  move from beta to GA", i.e. promotion edits the record rather than minting a
  second one, and the PRR team exists precisely to add a second, less familiar
  pair of eyes for production-facing work — the class of work ADR-010 warns an
  operator can wrongly mark `light` (source:
  https://github.com/kubernetes/enhancements/blob/master/keps/NNNN-kep-template/README.md,
  https://kubernetes.dev/resources/keps/1194; docs/adrs/010-lightweight-plan-flow.md).
- Python's rule for what does *not* need a design record is the closest prior
  art for the existence of a light flow: "Most enhancements and bug fixes don't
  need a PEP and can be submitted directly to the Python issue tracker", and the
  PEP editors reject proposals that are "too unfocused or too broad" (source:
  https://peps.python.org/pep-0001/). Number assignment is the relevant
  mechanism detail: the author picks `pep-NNNN.rst` as "the next available PEP
  number not used by a published or in-PR PEP" and an editor confirms it —
  allocation from the union of live and in-flight records, which is what
  BindForge's `nextNnn` does over `docs/adrs` + `docs/designs` (source:
  https://peps.python.org/pep-0001/; tools/orchestration/src/pipeline/gates.ts,
  lines 273–281). ADR-010's shared counter means light runs consume decision
  numbers; Python has no precedent that a light record should stop consuming
  them (source: docs/adrs/010-lightweight-plan-flow.md).
- Size caps that *stop* work are conventional in review practice: Google's
  guidance is that reviewers "have discretion to reject your change outright
  for the sole reason of it being too large", that 100 lines is usually a
  reasonable CL and 1000 lines too large, and that a change's size grows with
  the number of files it spans (source:
  https://google.github.io/eng-practices/review/developer/small-cls.html). That
  last point matters for the plan cap: an 80-line plan and a 20-file plan are
  not the same size signal (source: https://google.github.io/eng-practices/review/developer/small-cls.html).
- A capped plan page has a named precedent: Google's "mini design doc" of 1–3
  pages, "especially helpful for incremental improvements or sub tasks… you
  still do all the same steps as for a longer doc, just keep things more terse
  and focused on a limited problem set", together with the explicit statement
  that design docs are overhead whose value depends on design ambiguity (source:
  https://industrialempathy.com/posts/design-docs-at-google/). This is the
  strongest support for the ADR's rejected-alternative "no plan artifact at
  all" and for the plan's template sections (source: docs/adrs/010-lightweight-plan-flow.md).
- Machine-readable frontmatter selecting a heavier/lighter path is standard
  tooling, and the fail-closed pattern is standard too: Jekyll treats any
  triple-dashed YAML block as metadata and exposes arbitrary custom keys
  (source: https://jekyllrb.com/docs/front-matter/), and GitHub Actions routes
  work off `on.push.paths` / `paths-ignore`, where a malformed filter is a
  workflow-configuration error rather than a silent skip (source:
  https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions).
  GitHub's branch protection likewise makes review width a declared integer
  policy — `required_approving_review_count` accepts 1–6 — which is the
  external shape of a two-seat panel (source:
  https://docs.github.com/en/rest/branches/branch-protection).

## Platform notes (lens)

- No new platform surface: the light flow adds a stage, a seat list and a
  config record, and the Node floor is untouched (`engines: { node: ">=22.23.4" }`)
  (source: tools/orchestration/package.json, lines 30–32).
- The plan's 80-line cap is a line count computed by the runner
  (`text.split('\n')`, matching `wc -l`), measured after inference and before
  the commit, so the stage cannot self-report its size — the property the ADR
  relies on (source: tools/orchestration/src/pipeline/runner.ts, lines 161–188;
  docs/adrs/010-lightweight-plan-flow.md).
- Frontmatter parsing is CRLF-tolerant (`^---\r?\n([\s\S]*?)\r?\n---`) and
  value-tolerant (`/status:\s*(\w+)/`, defaulting to `'none'`), so an
  unquoted `flow: light` on a CRLF brief parses cleanly, and a typo such as
  `flow: Light` reaches the gate as a value, not as absence — the gate must
  compare case-sensitively against `full`/`light` and deny, since "absent means
  full" makes a silently-unread key indistinguishable from a full run (source:
  tools/orchestration/src/pipeline/gates.ts, lines 33–47;
  docs/adrs/010-lightweight-plan-flow.md).
- Both new gates sit before any inference is bought, consistent with
  `sourceAdmissible`'s fail-fast position, and the oversized-plan gate sits
  where `proposalSize` is already called (stage ok, pre-commit) (source:
  tools/orchestration/src/pipeline/gates.ts, lines 49–78;
  tools/orchestration/src/pipeline/runner.ts, lines 1655–1656).
- The light flow's session economy interacts with ADR-003's lane knob: with
  `maxParallelReviews: 1` the two seats are serial, so a light code-review pass
  costs 2 × the per-seat wall clock; `--max-parallel` applies to both flows and
  `MAX_PARALLEL` env parsing is already strict (source:
  docs/adrs/010-lightweight-plan-flow.md; tools/orchestration/src/pipeline/cli.ts,
  lines 72 and 135–138; docs/workflows.md, lines 97–108).
- Operator-facing surfaces gain a dimension: `docs/workflows.md` documents one
  13-stage path and one budget table, and the escalation report prints phase,
  rounds and budgets without naming a flow, so the ADR's "an escalated light run
  must read as 'too big for the light flow'" is work in `report.ts` and
  `docs/workflows.md`, not a free property (source: docs/workflows.md, lines
  44–108 and 177–193; tools/orchestration/src/pipeline/report.ts, lines
  218–239; docs/adrs/010-lightweight-plan-flow.md).
- The test plan's five named homes all exist: `tests/pipeline/` (with
  `states.test.ts`, `gates.test.ts`, prompt/parallel suites) and the
  `tests/pipeline/fixtures/checkpoints/` era set referenced by the ADR-006
  contract (source: tools/orchestration/tests/pipeline/,
  tools/orchestration/src/pipeline/state-store.ts, lines 49–51).

## Risks & unknowns

- **The two seats are the whole safety net.** `plan` inherits the hazards of
  the stages it replaces with no design panel, no editor and no editorial
  confirmation round; the ADR accepts this, and nothing in the runner can
  detect a plan that is short, clean and wrong. External practice converges on
  exactly this trade for small work, and equally explicitly warns that
  small-in-lines, wide-in-consequences changes are the case where the
  lightweight path is chosen wrongly (source: docs/adrs/010-lightweight-plan-flow.md,
  https://abseil.io/resources/swe-book/html/ch09.html,
  https://google.github.io/eng-practices/review/developer/small-cls.html).
- **Ownership spine widening.** `stagePathAllowed` is the deterministic
  enforcement that reverts out-of-scope writes (agent-config permissions are
  documented as advisory because "a model can route around them via
  subagents"), and the reserved-name rule is what keeps the checkpoint, lock,
  escalation marker and `saveState` tmp files runner-authored. Adding a flow
  parameter widens the coder's write surface in one flow and makes the rule
  order — reserved names first, in both flows — a testable contract that must
  be written down (source: tools/orchestration/src/pipeline/gates.ts, lines
  283–316; docs/adrs/010-lightweight-plan-flow.md).
- **Budget drift on resume** is the failure the ADR calls out and the code
  makes easy: budgets resolve from `opts.cfg ?? DEFAULT_CONFIG` at two seams, so
  a resumed light run on the CLI's spread config gets `maxRounds: 5` and
  `maxReviews.code: 20`, looks healthy, and spends the full budget. The ADR
  demands a test on the *effective* config after resume (source:
  tools/orchestration/src/pipeline/runner.ts, lines 2068 and 2126;
  tools/orchestration/src/pipeline/cli.ts, lines 212 and 225;
  docs/adrs/010-lightweight-plan-flow.md).
- **Flow recorded in two places can disagree.** `flow` lives in the source
  document and in the checkpoint; `--force` promotion re-reads the document
  while keeping the identity, and the full `proposal` stage overwrites
  `02-proposal.md`, so the light plan survives only in branch history and the
  seat stamp changes meaning mid-branch (source: tools/orchestration/src/pipeline/cli.ts,
  lines 126–130; tools/orchestration/src/pipeline/runner.ts, lines 2416;
  docs/adrs/010-lightweight-plan-flow.md).
- **Shared numbering space.** Briefs already claim `nextNnn` over `docs/adrs` +
  `docs/designs`, and the documented upgrade path is "write
  `docs/adrs/NNN-<slug>.md` with the same number" — run 010 is itself a live
  example of a design directory numbered from a decision. With light runs in the
  same counter, `NNN` stops encoding "this is a decision", and the ADR/design
  pair gains a third meaning (source: tools/orchestration/src/pipeline/identity.ts,
  lines 57–72; docs/workflows.md, lines 9–28; docs/designs/010-lightweight-plan-flow/.pipeline.json;
  docs/adrs/010-lightweight-plan-flow.md).
- **No flow auto-detection is a policy, not a safety property.** The only
  deterministic stops are the oversized plan and budget exhaustion, and a task
  that is 5 lines across 20 files (a codec change, a public schema edit) passes
  a line-count gate with room to spare (source: docs/adrs/010-lightweight-plan-flow.md,
  https://google.github.io/eng-practices/review/developer/small-cls.html).
- **The plan-size cap reverses a documented lesson.** The repo's existing size
  policy warns because "a hard size limit buys compressed prose, which is
  harder to review than long prose"; the light flow's cap escalates. Both can be
  right — the plan is a fixed template, not a design doc — but the proposal
  should say why the lesson does not transfer (source:
  tools/orchestration/src/pipeline/report.ts, lines 196–216;
  docs/adrs/010-lightweight-plan-flow.md).
- **`maxRounds` semantics in a 2-round flow.** `maxRounds` is documented as the
  backstop on the mutating revise/fix loop, entered per design round *or* fix
  stage, not as a review cost cap; with no design phase, `maxRounds: 2` in
  `LIGHT_CONFIG` bounds only fix stages, so a light run can keep re-reviewing
  (dissent and ratify passes) without spending a round, up to the review-session
  cap (source: tools/orchestration/src/pipeline/stages.ts, lines 26–40;
  tools/orchestration/src/pipeline/panel.ts, lines 100–133;
  docs/adrs/010-lightweight-plan-flow.md).
- **Cost claims in the ADR should be recomputed, not inherited.** Run 010's
  checkpoint exists with `status: "running"`, `lastStage: ""` and an empty
  `results` array, so the run for this ADR has consumed no recorded sessions;
  the evidence base for the light flow is the six earlier runs (source:
  docs/designs/010-lightweight-plan-flow/.pipeline.json).

## Open questions (for the designer)

1. Which knob binds in the light code phase — `maxReviews.code: 6` (sessions)
   or `maxPassesPerRound: 3` × `maxRounds: 2` (passes)? With 2 seats the
   6-session cap is reached at 3 passes, so is the intended shape 6 sessions
   (2 rounds × 1 full pass + one dissent), or is `maxReviews.code` meant to be
   12? (source: docs/adrs/010-lightweight-plan-flow.md;
   tools/orchestration/src/pipeline/panel.ts, lines 113–120)
2. How does `LIGHT_CONFIG` express the `PipelineConfig` fields it does not
   list (`pollIntervalMs`, `requestTimeoutMs`, `blockingSeverities`,
   `maxParallelReviews`) — a full record, or a documented spread over
   `DEFAULT_CONFIG` with the shared knobs named? The runner's "DEFAULT_CONFIG is
   the sole default authority" comment is the invariant to preserve (source:
   tools/orchestration/src/pipeline/stages.ts, lines 26–84;
   tools/orchestration/src/pipeline/runner.ts, lines 1940–1943).
3. How does the `plan` stage get on the size meter — added to
   `DESIGN_DOC_STAGES`, or is the meter keyed by the flow's document-producing
   stages? The same question decides whether `docLines` appears in the status
   report for light runs (source: tools/orchestration/src/pipeline/runner.ts,
   lines 161–188 and 1655–1656).
4. What exactly does the oversized-plan gate measure: lines of `02-proposal.md`
   only, or also the files-to-touch list (an 80-line plan naming 20 files is the
   case Google's size guidance treats as large)? Where does the 80-line cap
   live — the `plan-note` skill, the flow registry, or `LIGHT_CONFIG` (source:
   https://google.github.io/eng-practices/review/developer/small-cls.html;
   docs/adrs/010-lightweight-plan-flow.md)?
5. Should `flow` be recorded per *stage* in `results` (the transcript already
   names the agent, e.g. `- agent: codereview/code-testing`) so a resumed or
   promoted run can show which rules governed each stage, and should
   `pipeline status` print the effective budget numbers, not just the flow name
   (source: docs/designs/006-self-update-checkpoint-compatibility/transcripts/code-review-testing-r1c0.md,
   line 5; tools/orchestration/src/pipeline/report.ts, lines 218–239;
   docs/adrs/010-lightweight-plan-flow.md)?
6. Is a brief-driven light run's `NNN` claim and `docs/adrs/NNN-<slug>.md`
   promotion path still desirable now that light runs consume the shared
   counter faster, and does the `NNN` collision check need to treat a light
   design directory as taken (source: docs/workflows.md, lines 22–28;
   tools/orchestration/src/pipeline/identity.ts, lines 57–72)?
7. Does `--force` promotion need a guard, given the full `proposal` stage
   overwrites the artifact that carries the seat stamp and the light plan
   survives only in git history — is that recorded in `ESCALATION.md` or the
   branch, and does the light run's `03-build-notes.md` survive (source:
   tools/orchestration/src/pipeline/cli.ts, lines 126–130;
   docs/adrs/010-lightweight-plan-flow.md; docs/workflows.md, lines 121–140)?
8. Where does the flow key get validated for the `resume` path — the checkpoint
   schema (`flow: z.enum(['full','light']).optional()`, defaulting to `full`)
   plus a backfill at resume in the `baseRef` precedent, and which golden
   pre-`flow` fixture pins that a legacy checkpoint resumes as `full` (source:
   tools/orchestration/src/pipeline/state-store.ts, lines 37–64)?
9. Is `light` allowed on an ADR source at all, and if so what does
   `sourceAdmissible` require of it (`status: proposed` plus Context/Decision, as
   for `full`)? The ADR names briefs as the light source but the gate is on the
   frontmatter key, not on `source.kind` (source: tools/orchestration/src/pipeline/gates.ts,
   lines 54–78; docs/adrs/010-lightweight-plan-flow.md).
10. Does the funnel's deadlock detection still fire at 2 seats — the
    `reviseNoChange` rule escalates when a revision changes nothing while named
    seats dissent, and with `maxRounds: 2` there is one fewer revision to break
    the impasse (source: tools/orchestration/src/pipeline/panel.ts, lines
    121–131)?
