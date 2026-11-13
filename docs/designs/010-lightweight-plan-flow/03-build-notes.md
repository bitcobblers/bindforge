# Build notes: A lightweight `plan → implement → code review` flow

Implementation of [`02-proposal.md`](./02-proposal.md) (ADR-010). The registry,
the light stages, the ownership widening, the plan gate, the reporting and the
promotion path are in. Round 3 closed the open findings of code review round 1
(`reviews/code/correctness-r1c0.md`, `security-r1c0.md`, `performance-r1c0.md`):
the full flow's budgets are back at the design's numbers, the funnel is
unmodified, `--extend-budget` is consumed at the cap check in both flows, the
ownership widening is the designed one, the plan stop runs pre-commit on the
meter's own count, and the registry's exported names match the proposal's
contract. Round 4 is the last of the mechanism findings (deviations 16-20) and
clears `npm run typecheck`. Round 5 closes the open findings of the round-1
dissent passes (`reviews/code/correctness-r1c1.md` findings 1-6): the light
flow's `fix` loop is charged and bounded the way the full flow's is, a plan stop
reports the stage and phase that stopped it and leaves the plan on disk, the
identity rule that was shared by both flows is the light flow's only, and
`--stage plan` is forceable on a light run. Round 6 is the fix round the gate
asked for: the shared round gate stops reading an `--extend-budget` grant in
the light flow, where nothing spends it (deviation 24), and `npm run check`
exits green. Round 7 closes the one finding the round-1 correctness dissent
pass left open (`reviews/code/correctness-r1c2.md` finding A): the plan stop is
recorded on the checkpoint, so a resume cannot walk past it into `implement`,
and the record that stops the run prints only the repairs the state machine
honours — the promotion recipe and the in-flow re-plan (deviation 25). Round 8 is the fix
round the gate asked for: it closes the two red assertions on that record — a
plan-stop report must print no bare-resume prefix — and `npm run check` exits
green (deviations 27-28). Round 9 closes the one major the round-1 correctness
dissent pass left open (`reviews/code/correctness-r1c3.md` finding 1): the
plan-stop predicate keyed on the **stage**, so a light run that never reached
the plan gate read as stopped — a failing `implement`, whose escalation leaves
`lastStage` at the name the successful `plan` stage persisted, and a healthy
running light run both printed the plan stop's promotion recipe and printed no
plain `pipeline resume`, the repair their state honours (deviation 29).

The package's gate is **green**: `npm run check` exits 0 — `npm run lint`
clean, `format:check` clean, vitest at 41 files, 488 passed / 2
platform-skipped; round 8 re-ran the same chain with deviation 27 in — lint
clean, `format:check` clean, 41 files, 492 passed / 2 platform-skipped, and
`npm run typecheck` exits 0. Both halves of the round-5 gap are closed. Blocked item 5,
the two unformatted test files, is the test owner's `chore: format
light-promotion test`, and the expectations listed under "Round 5: the
assertions that move with the mechanism" landed in `test: align the light-flow
pins with the round-1 correctness findings`. The one red the gate still
carried into this round was in `light-budget.test.ts`'s two
`--extend-budget` cases, and the mechanism behind them was the deviation this
round closes (24): `states.ts` alone changed, no test edited — the runner
reverts every test path it is handed, and none needed moving.

Round 7's caveat, for correctness r1c2 finding B: the implementation stage that
wrote it has no shell, so it ran neither `npm run check` nor `npx prettier
--check`. The green above is round 6's run; what round 7 asks the gate to
confirm is that the same `lint → format:check → test` chain exits 0 with
deviation 25's files in — `types.ts`, `state-store.ts`, `states.ts`,
`runner.ts`, `report.ts`, `prompts.ts`, `stages.ts`, `docs/workflows.md`. No
test path was touched, so the chain reaches vitest on the formatting the
test owner landed in round 6. Round 8 ran it: the chain exits 0 with those
files in, and the only reds it met were the two plan-stop report assertions
deviation 27 closes.

Round 9's caveat, for correctness r1c3 finding 2: this stage has no shell
either, so it ran neither `npm run check` nor `npx prettier --check` — the
finding asks for the chain's output, and only the gate can produce it. What it
changes is `report.ts` (`hasPlanStop`, `repairSteps`) and `runner.ts`
(`planStopStatusLines`, `statusLines`), written in the same prettier shape the
package keeps (`printWidth 100`, no line above it except an unbreakable string,
each statement a single template concatenation prettier wraps at its own
operators). The gate's confirmation for this round is that the same
`lint → format:check → test` chain reaches vitest and exits 0 with those two
files in; the only red the round expects to meet is the one listed under
"Round 9: the assertions that move with the mechanism", which is the test
owner's.

## What landed

- `src/pipeline/flows.ts` — `FLOWS`, `flowOf`, `configFor` (the one budget
  resolver, validation included: seat names per phase, `planMaxLines`
  absent-or-positive, panels non-empty for the phases the flow's own stages
  run), `seatsFor`, `isStageInFlow`/`assertStageInFlow`. `funnelSpecialties`
  and `specialtiesFor` delegate to `seatsFor`; the five seat readers take the
  `cfg` their callers hold.
- `PipelineConfig.reviewSeats` / `.planMaxLines` as optional fields;
  `STAGE_BASES.plan` (`kind: proposal`, `design/planner`, `plan.md`);
  `PipelineState.flow` / `StateSchema` `flow` at `version: 1`, and
  `PipelineState.planStop` / its `StateSchema` key (deviation 25).
- `decideNext`: `''` → `plan`, `plan` → `implement` (escalating on a standing
  plan stop, deviation 25), converged code panel →
  `done`, `testsNeedUpdate` routing guarded to `flowOf(state) === 'full'`.
- `prompts/plan.md`, `.opencode/agents/design/planner.md` (a verbatim copy of
  `designer.md:4-22`), `.opencode/skills/plan-note/` with its
  `references/template.md`; `{{testOwnership}}` in `implement.md`, `fix.md`,
  `fix-tests.md`.
- `stagePathAllowed(stage, path, flow)`, `moves.ts` threading the same value,
  `planOutgrewFlow`, the pre-lock `flow:` validation and `assertStageInFlow` at
  both entry points, the size meter keyed on `FLOWS[flow].docStages`,
  `escalationPhase` reporting `plan` as `code`, the one flow line in
  `statusLines`/`escalationMarkdown`, and the promotion suffix appended once in
  `finalizeEscalate`; `repairSteps`/`hasPlanStop` (`report.ts`) are the repair
  the record, the terminal and `pipeline status` print (deviation 25), keyed on
   the recorded stop — `state.planStop` — not on the stage (deviation 29).

## Deviations from the proposal

1. **`assertStageInFlow(flow, stage)`** — the code had the parameters in the
   opposite order (`(stage, flow)`) from the proposal's signature block. Fixed
   to the proposal's order; the two runner call sites follow.
2. **`renderTemplate` carries a default for `{{testOwnership}}`**
   (`prompts.ts`: `TEST_OWNERSHIP_FULL` / `TEST_OWNERSHIP_LIGHT` /
   `testOwnershipLine(flow)`). The proposal puts the value in `stageVars`
   only. A default at the renderer keeps the rule in one place for every
   caller that renders a coder template outside the drive loop (prompt-shape
   and stage round-trip tests, and any future caller): an unspecified flow
   renders the standing full-flow rule, and the runner always passes the
   flow's line, so the default is a fallback, never an authority. The strict
   "every placeholder must be filled" rule is unchanged for every other key.
3. **The code-phase revision stamp is refreshed per pass kind, not per pass
   entry** (`runner.ts`, drive loop). The proposal keeps A5's "`HEAD` is the
   code stamp"; it does not say when it moves. A review pass's own finalize
   commit (the seats' reports and transcripts) moves `HEAD`, so re-stamping at
   every pass entry ages the approvals the ratification pass is ratifying and
   buys a second ratify pass — a light run that dissents once then converges
   spent three passes and hit the 3-pass backstop instead of converging
   (Test strategy #13). Now: a `full` or `dissent` pass entry — which follows a
   mutating stage — re-stamps to `HEAD`; a `ratify` pass keeps the stamp its
   round entry set, matching the design phase, whose blob-id stamp a report
   commit cannot move. Full-flow code-phase tests are unaffected. (This rule
   is also what run 010's near-miss at 6 passes needed; see deviation 11 — the
   pass cap is not raised to buy it.)
4. **The checkpoint-identity rule is the light flow's** (`identity.ts`:
   `nnnOfExistingRun`, `deriveIdentity`'s `flow` parameter; round 5,
   correctness r1 finding 3). An earlier pass applied it to every non-ADR
   source, so a plain re-run of a brief that has a run became a clash needing
   `--force`/`--slug`/`--nnn` — a change to the **full** flow's shared entry
   path that the proposal does not ask for (A17: "`nextNnn` and `identityClash`
   need no change"; Compatibility: "every existing document and checkpoint
   behaves exactly as it does today"). It is now gated on the flow that needs
   it: a light entry re-run from the document that already drives a run
   resolves the `NNN` that run's checkpoint names, so a light run cannot mint a
   second design directory beside its own plan (the shape Test strategy #22
   pins for the light side). `sourceKind` is exported from `identity.ts` so
   `runPipeline` can parse the source's `flow:` key before the identity is
   derived, and `deriveIdentity` derives the identity's kind from the same
   rule. Consequence, in the full flow's corner: a brief promoted to
   `flow: full` keeps A17's allocation, i.e. the counter's next free number, so
   that promotion lands on a new identity beside the light plan rather than
   `--force` on the light one. Making the full flow share a brief's identity is
   a rule about the full flow's entry path and needs its own design entry; the
   brief case of Test strategy #22 is the test owner's to move (Blocked list,
   round 5).
5. **`statusLines` prints the run identity under the label `slug:`** (was
   `run:`). The proposal's Reporting section names only the flow line;
   Test strategy #10 asserts a `slug:` line in the rest of the block, and no
   test, prompt or CLI path reads the old label.
6. **Dead code removed:** the `DESIGN_DOC_STAGES` set in `runner.ts` (the meter
   keys on `FLOWS[flowOf(state)].docStages`, per A6), the unused `FlowId`
   import in `states.ts`, and a redundant initializer on `flow` in
   `runPipeline`. `eslint .` treats all three as errors, so `npm run check`
   could not reach vitest with them in place.
7. **Formatting:** `panel.ts:32`, `states.ts:183`, `runner.ts:22` and the
   `finalizeEscalate` suffix condition in `runner.ts` were committed unformed
   by the earlier feature commits; `npm run format`-shaped wrapping applied
   here, since `format:check` is part of the gate.
8. **`ParsedSource` carries `body`, and the brief gate judges it** (round 2).
   The proposal states the brief rule as "non-empty" (A10, Compatibility
   matrix) and does not name what the non-empty test reads. Before ADR-010 a
   brief had no frontmatter, so the whole document and the brief were the same
   bytes; a `flow:` key splits them, and the gate's `content.trim()` counted a
   frontmatter-only brief as a brief (`gates.test.ts:588`). `parseSource`
   already derives the frontmatter-stripped `body` for the section scan, so
   `sourceAdmissible` judges that value and `ParsedSource` exposes it — one
   frontmatter read per entry (Selecting the flow), no second regex. The
   plan-size gate keeps measuring the whole file, because that is the count
   `proposalSize` records (`runner.ts:179-180`, A6), and the plan's emptiness
   rule is untouched.
9. **`FlowDef.path`** — a registry field the proposal's `FlowDef` does not
   list. `statusLines` and `escalationMarkdown` print the flow's stage path
   (Reporting's example line), so the string lives with the flow it describes.
   `FLOWS.full.path` names the full flow's own stages: the word `plan` is a
   stage of the light flow in this release, so it never appears as a step of
   `full`. (Round 3.)
10. **`maxRounds: 2` binds the light fix loop on the record A16 states it in**
    (round 3 raised it, round 5 closed it — correctness r1 finding 1). An
    earlier pass repurposed the light flow's `codeBudgetUsed` into a session
    counter (each pass charged its seats; the round entry and each `fix`
    charged nothing) and read the flow's bound off `state.codeRound` in
    `states.ts`. Two shapes fell out, neither the design's: a light `implement`
    with red tests and no code pass yet left the counter at 0, so `states.ts`'s
    `budgetLeft` answered `fix` **forever** — no session cap gates that path
    (the pass-entry cap gates review passes), the pass backstop needs a round,
    and a `fix` that replies `ok` is no stage failure — and a light dissent
    after one 2-seat pass read 2, so its *first* fix escalated: one round where
    the Goals ask for two. The charging is A16's and `PipelineState`'s own
    docstring's: each `fix` charges one round in every flow, the full flow's
    code round entry charges one more, and the seats a pass spends are the
    `reviewSessionsUsed` count `maxReviews.code` caps. The light flow's
    mutating loop is the `fix` loop, so a light code round — opened by the
    funnel, not by a mutating stage — charges no round at its entry and reads
    no round backstop there (drive loop, round-entry branch): the light
    `codeBudgetUsed` **is** the fix charges, and at `maxRounds: 2` the light
    code phase is two `fix` rounds. The full flow's arithmetic is untouched:
    at `maxRounds: 5` its code phase is one round entry plus four fixes, and
    the knob that ends a stubborn panel inside a round is `maxPassesPerRound: 3`
    (Budget resolution: the knob that binds there). Which check binds on which
    light path: **red tests from `implement`/`fix`** → the shared `states.ts`
    `budgetLeft` on the fix charges, i.e. 2 fix rounds then `tests not green
    at the code-review budget limit` — the path that looped unboundedly
    before; **a dissent** → the same shared `budgetLeft` (the State graph
    delta's "shared" row), so a dissenter gets its **second** `fix` charge,
    and once the two charges are spent it is the funnel's pass backstop
    inside the round that prints: a panel dissenting every pass stops at 3
    passes — 6 sessions, 4 for a single dissenter — the Goals' shape; **a
    forced `--stage code-review` resume** into a spent session budget → the
    pass-entry session cap, `review session budget exhausted (6/6 sessions)`;
    and `states.ts`'s `codeRound` check guards the round number itself in
    both flows. The printed unit follows the record:
    `statusLines` and `ESCALATION.md` name `codeBudgetUsed` rounds in both
    flows, so the full-flow lines are byte-equal, and the session count they
    print is `reviewSessionsUsed(state, 'code')`, which was already there.
11. **The full flow's budgets are `DEFAULT_CONFIG`'s own values** (round 3,
    correctness r1 finding 1, security r1 finding 1, performance r1 findings
    1-2): `maxPassesPerRound: 6`, `stageTimeoutMs: 30 * 60_000`, as
    `01-research.md:48-50`, the proposal's budget table and ADR-010's table
    record. An earlier pass of this branch had raised them to 8 / 90 minutes
    from run 010's near-miss; that is out of this design's scope (Non-goals)
    and it made `docs/workflows.md` disagree with the code. The
    convergence run 010 was one pass from is deviation 3's stamp rule, which
    lands in this change. `docs/workflows.md`'s budget table and defaults list
    now read 6.
12. **The plan stop denies a plan it cannot measure** (round 3, security r1
    finding 4, correctness r1 finding 6). The proposal pins the stop's
    *position* — "the same stage-ok/pre-commit point where `proposalSize` runs"
    — and its over-length string; it says nothing about the stage that writes
    no document. That case now escalates
    `plan document not written: docs/designs/<id>/02-proposal.md` at the same
    point, before the plan is committed and before the `implement` session: a
    gate that exists to be a deterministic stop is never satisfied by the
    absence of its own input, and the light flow's two seats have nothing to
    review without the document its stamp is built from (A5).
13. **The seat-panel validator checks each phase against that phase's seats**
    (round 3, security r1 finding 3). The proposal's rule is "a resolved config
    must name only known specialties"; read per phase it is stronger and is
    what `reviewSeats` means: `code` is the set of `codereview/code-*` agents
    and `design` the set of `review/design-*` agents, so `code: ['complexity']`
    — a design seat — is a pre-lock usage error instead of a mid-run
    "no model turns — agent … may not exist" after the lock and an escalation
    record. A `reviewSeats` phase key that is not an array is the same clean
    usage error, in place of a `TypeError` from the resolver.
14. **`sourceAdmissible` takes the caller's parse as an argument** (round 3,
    correctness r1 finding 8, security r1 finding 6, performance r1 finding 5).
    The design asks for "one frontmatter read per entry" and for the post-lock
    gate to "take the same result as an argument instead of re-reading":
    `runPipeline` parses the source once, pre-lock, and hands that
    `ParsedSource` in. The parameter is optional so the gate keeps its own
    single read — and its unreadable-file denial — on any path that has no
    parse of its own.
15. **`sourceAdmissible` judges the source document's shape, not a plan's
    length** (round 3, correctness r1 finding 5, security r1 finding 7). An
    earlier pass gave it a `'plan'` kind that capped the document it was
    handed; nothing on the entry path produces that kind (`deriveIdentity`
    yields `adr | brief`), so the branch was dead in production and its line
    count — `documentLines` — was a second meter beside `proposalSize`'s. Both
    are deleted: the cap is `planOutgrewFlow`, run by the runner on the
    meter's own count (deviation 12), and `proposalSize` is the only line
    counter.

### Round 4

16. **`--extend-budget` consent is recorded where the grant is spent**
     (`runner.ts`, `resumePipeline`). The intake-time `operatorExtension` gate
     entry is removed: A27 puts the grant on the checkpoint
     (`extensionSessions` / `extensionRounds`), and the over-cap entries — the
     round entry and the pass entry — already name the grant they paid for and
     the count that remains. The intake line logged a consent the run may never
     use, so a light run that dies at the pass backstop carried a gate entry
     claiming a budget extension that bought nothing. Each consumption line now
     carries the intake's wording — `extended the review budget by 1 review
     session(s) over the caps of <design>/<code>: over-cap <phase> … entered
     under operator extension (N … left)` — so the record names the caps the
     grant moved at the point it moved them, and an unspent grant leaves no
     extension in the gate log.
17. **The plan stop leaves the refused document in the worktree** (round 4
    raised it, round 5 reversed it — correctness r1 finding 6). The proposal
    fixes the stop's *position* — the stage-ok/pre-commit point, so the stage's
    own commit window is skipped — and says nothing about deleting the stage's
    product; the ownership sweep is the design's only revert, and `docs/**` is
    `plan`'s allowed surface. An earlier pass put the path back the way the
    stage found it (`git restore` from `HEAD`, else delete), which left the
    refused plan nowhere: not committed, not on disk, while the stop's own
    record tells the operator to promote from it. The posture is unchanged —
    no plan commit, the sweep still runs on the refusal path — and the bytes
    stay unstaged where the stage wrote them. Consequence, accepted: the
    terminal escalation commit `finalizeEscalate` mints is a `git add -A` over
    the swept worktree, so it carries the refused plan into the run's branch
    history, which is where A26 says the plan for promotion lives.
18. **A review pass's ledger entries read in the order the seats were handed**
     (`runReviewPass`, `runner.ts`). Seats appended their `state.results` entry
     as they finished, so the same run at `--max-parallel 1` and at width 3 left
     a different checkpoint — and the seat order a report prints with it.
     `--max-parallel` is an operator knob; the record a run leaves cannot depend
     on which sibling's turn landed first. The pass's ledger tail is re-sorted,
     stably, into handed order after the barrier, so a seat's own
     attempt-1-then-attempt-2 pair keeps its order.
19. **Panel validation is split between the caller's base and the resolved
     config** (`validatePanelPresence`, `flows.ts`). Deviation 13's rule is about
     names; Test strategy #23 also pins a `reviewSeats` that omits a phase key
     as a usage error. The flow's partial is the last writer of `reviewSeats`
     (the registry seats win), so a base naming one key is legal and presence
     cannot be a rule about the base: the base is checked for seat names and for
     an empty panel among the keys it wrote, and every phase the flow runs must
     resolve to a panel — checked on the resolved config. Before this, the
     omitted key reached the emptiness check and was a `TypeError`, exactly the
     shape #23 refuses.
20. **`SeatPhase` is `'design' | 'code'`** (`flows.ts:29`). It was an alias of
     the state graph's `ReviewPhase`, which has an `editorial` arm, while every
     panel reader — `seatsFor`, `PANELS`, both validators — indexes a two-key
     panel with it, so `npm run typecheck` was red with four errors in
     `flows.ts` (3 × TS7053, 1 × TS2741). The alias is now
     `Exclude<ReviewPhase, 'editorial'>`, which is what its own doc comment and
     `states.ts:31`'s `phase === 'editorial' ? 'design' : phase` already say:
     the editorial round is a language pass over the design panel, not a seat
     list. `typecheck` is not part of `check`, so the gate did not see this; it
     exits 0 now.

### Round 5

21. **`--stage` is validated against the flow's registry alone** (round 5,
    correctness r1 finding 5). The entry points ran `assertForceableStage`
    before `assertStageInFlow`, and `STAGE_NAMES` excludes `plan` by
    construction (it is the full flow's twelve-stage regression guard), so
    `pipeline run <light-doc> --stage plan` and the resume shape were usage
    errors — the operator could not re-plan the run a plan stop ended, only
    resume into it. `FlowDef.stages` is the gate the proposal documents
    ("documentation + `--stage` gate"), and the flow check answers every case
    the old check did: every name in `STAGE_NAMES` and `REVIEW_STAGE_NAMES` is
    in `FLOWS.full.stages` (`flows.test.ts` pins that pair), and `plan` is in
    no full-flow path, so a full-flow `--stage plan` stays a usage error.
    `assertForceableStage` stays exported (its own test pins the message); the
    runner no longer calls it.
22. **A pre-commit gate's stop names the stage it stopped at** (round 5,
    correctness r1 finding 2). `state.lastStage` was assigned after
    `runWithRetry` returned, and a `preCommit` refusal escalates from inside
    that call, so the run reached `finalizeEscalate` with the name still
    empty: `escalationPhase` answered `design`, ESCALATION.md opened
    `Run stopped after stage `` in the **design** phase`, and `reportFindings`
    read the design phase's round — the exact mislabel A15 adds `plan` to
    `CODE_STAGES` to remove, and the unit test that pins it (#10) builds the
    state by hand, so the run path never reached it. `executeStage` records
    the stage on the refusal path, before the stop is reported.
23. **Round 5 changed no gate, no seat list and no registry number.** The
    findings it closed are the light flow's budget charging (deviation 10),
    the plan stop's reporting and posture (deviations 22 and 17), the
    identity rule's flow scope (deviation 4), and the `--stage` gate
    (deviation 21). `gates.ts`, `panel.ts`, `moves.ts`, the prompts, the
    agents, the skills, `FLOWS`'s values and `DEFAULT_CONFIG` are untouched.

### Round 6

24. **A light `--extend-budget` grant buys over-cap review sessions, never a
     third `fix` round** (`states.ts`, `budgetLeft`). Deviation 10 made the
     light `codeBudgetUsed` the flow's fix charges and `maxRounds: 2` the hard
     bound on the light `fix` loop, but left the shared gate's extension clause
     — `used < maxRounds || extensionRounds > 0` — reading in the light flow
     too. The light flow has no site that *spends* a round grant: the drive
     loop's round-entry block, where A27 consumes `extensionRounds`, is the
     full flow's, and a light code round — opened by the funnel, charging no
     round (deviation 10) — never reaches it. So a grant read at that gate
     bought an uncharged **third** `fix` round past the cap and left the grant
     undecremented: the Goals' "`fix` bounded at 2 rounds" became 3 on the one
     path where the operator passed `--extend-budget`. The light grant's
     designed consumer is the pass-entry session cap (A16, A27, deviation 16),
     which is where the light run reads the decreasing count. The clause is now
     gated on `flowOf(state) !== 'light'`. The full flow is untouched — its
     grants stay consumable at round entry, and `budget.test.ts`'s round-entry
     consumption pins pass unchanged — and no assertion moved:
     `light-budget.test.ts:372` (the grant spent per over-cap pass,
     `codeBudgetUsed` 2) and the grant-count-decrease pair at `:405` (the
     surplus left unspent) go green on the design's shape.

### Round 7

25. **The plan stop is recorded on the state, and its record prints only the
    repairs the state machine honours** (`types.ts`, `state-store.ts`,
    `states.ts`, `runner.ts`, `report.ts` — correctness r1 dissent pass c2,
    finding A). The proposal's `case 'plan'` row is `stage('implement')`, and
    deviation 22 made `state.lastStage` accurate at the plan stop, which moved
    the checkpoint's resume landing onto exactly that row. A light run the size
    gate stopped carries `lastStage: 'plan'`, `flow: 'light'`,
    `codeBudgetUsed: 0`, so the command its own record printed — `ESCALATION.md`
    ended "Fix the underlying cause … then run: `pipeline resume 002-x`" —
    walked straight past the flow's one deterministic plan stop: the 120-line
    plan the flow refused, committed by the terminal escalation commit
    (deviation 17), became the document `implement` built against and the two
    seats checked faithfulness against, with no cap downstream. The design's
    remedy for that stop is promotion, so the record was printing two
    instructions with opposite outcomes. Now the stop is data:
    `PipelineState.planStop?: string` (additive-optional, ADR-006 §2,
    `StateSchema` at `version: 1`) carries the gate's refusal text,
    `decideNext`'s `case 'plan'` escalates on that record while it stands, and
    the next `plan` pass whose gate proceeds clears it. So a plain
    `pipeline resume <id>` re-escalates on the promotion record — no
    `implement` session, no client call — and `pipeline resume <id> --stage plan`
    (the stage the flow registry makes forceable, deviation 21) is the in-flow
    repair. `repairSteps` (`report.ts`) keeps the record single-voiced: for a
    standing plan stop the `ESCALATION.md` footer, the terminal's repair lines
    and `pipeline status` print the promotion (`pipeline run <source> --force`
    after setting `flow: full`) and the re-plan, never the bare
    `pipeline resume`; for every other stop the footer is byte-equal to what it
    was, so the escalation pins that read it are unaffected. The state graph's
    row now reads: escalate on the recorded stop while it stands, else
    `implement`. `docs/workflows.md`'s escalation section says the stop is
    sticky and names the two repairs, and its command block lists
    `pipeline resume <slug> --stage plan`.
26. **The same review's two FYI notes** (`stages.ts`, `prompts.ts`):
    `assertForceableStage`'s docstring names its actual role now — the
    `STAGE_NAMES` regression guard, with `assertStageInFlow` the production
    `--stage` gate (deviation 21) — and `testOwnershipLine(flow)` takes a
    `Flow`, which is what every caller passes, so an open `string` can no
    longer render the full flow's sentence silently.

### Round 8

27. **The plan stop's record is single-voiced in its own spelling**
    (`report.ts`, `runner.ts`: `hasPlanStop`, `repairSteps`, `statusLines`).
    Deviation 25 shipped the record with two repairs, and the gate went red on
    both of them, in the two ways the design's single-voice rule reaches:
    *the predicate* keyed on `state.planStop`, a field the report is written
    from the state of a stopped run, so a checkpoint that describes the stop by
    its stage — `flow: 'light'`, `lastStage: 'plan'` — took the generic footer
    and printed the bare `pipeline resume`, the command `case 'plan'` answers
    with the same escalation (`escalation.test.ts:171`); and *the re-plan* was
    spelled `pipeline resume <id> --stage plan`, which carries the bare
    `pipeline resume <id>` as a prefix, so the record printed the forbidden
    prefix inside the repair that was meant to replace it
    (`light-plan-size.test.ts:281`, `escalation.test.ts:176`). Now:
    `hasPlanStop(state)` is `flowOf(state) === 'light' && lastStage === 'plan'`
    — the shape of the stop, the plain `boolean` the report needs, since the
    gate that stops a light run at `plan` is the flow's only plan gate; the
    state graph is untouched and keeps escalating on the recorded `planStop`
    (`states.ts:136`), because that record is what survives the checkpoint. The
    promotion note names the remedy without quoting a resume, and the re-plan
    prints as `pipeline resume --stage plan <id>`: the same invocation (the
    `resume <slug>` positional is parsed with the flag in front of it), with no
    bare-resume prefix in the bytes an operator reads.
    `statusLines` prints the recorded refusal, falling back to a standing-stop
    sentence when a checkpoint carries the shape without the record. (Round 9
     superseded the stage-keyed predicate and that fallback: deviation 29.)
    `docs/workflows.md` keeps the canonical `pipeline resume <slug> --stage
    plan` spelling for the operator typing it by hand.
28. **Round 8 touched no gate, no budget, no seat list and no test.** The
    reproduction ran the whole chain in this package (lint clean,
    `format:check` clean, 41 files / 492 passed / 2 platform-skipped,
    `npm run typecheck` clean), and `report.ts` and `runner.ts` are the only
    sources it changed; both are prettier-clean as written.

### Round 9

29. **The plan stop is reported from its record, not from the stage**
    (`report.ts`: `hasPlanStop`, `repairSteps`; `runner.ts`:
    `planStopStatusLines`, `statusLines` — correctness r1 dissent pass c3,
    finding 1). Deviation 27 picked `flowOf(state) === 'light' &&
    state.lastStage === 'plan'` as the predicate, reading the stop as a shape.
    The stage is not the stop, and two light shapes reached the report through
    it: *a failing `implement`* — `state.lastStage` is assigned only after
    `runWithRetry` returns, so every stage-failure escalation (two failed
    replies, a half-applied moves list) runs with `lastStage` still at the
    `'plan'` the successful plan stage persisted and with `planStop` already
    cleared at the gate; and *a healthy running light run* — a plan that
    clears the cap leaves the checkpoint at `lastStage: 'plan'`, no
    `planStop`, `status: 'running'`, for the whole `implement` window. Both
    printed the sticky-stop sentence and the promotion recipe for a stop that
    was never made, and neither printed the plain `pipeline resume <slug>`
    that is the repair their state answers (`states.ts:136-139` falls to
    `stage('implement')` when no stop stands). The predicate is now
    `flowOf(state) === 'light' && state.planStop !== undefined` — the field
    deviation 25 added and documented as the stop's record, "its absence is
    'no stop standing'" (`types.ts:247-248`). For a plan stop the two
    predicates agree, because the gate writes the field before the refusal's
    checkpoint save (`runner.ts:2785`, `2804-2807`, then
    `finalizeEscalate:3181`), so the record a plan stop leaves prints exactly
    the two moves it printed before — the promotion and the `--stage plan`
    re-plan, never the bare resume. `statusLines` renders its line from the
    same field, so the fallback sentence deviation 27 added for "the shape
    without the record" is gone: that shape is now the no-stop shape and
    prints no line. `repairSteps`'s contract is unchanged — single-voiced for
    a plan stop, byte-equal to the plain-resume footer for every other stop,
    which is the class this fix reaches.
30. **Round 9 changed no gate, no budget, no seat list, no state graph and no
    test.** Two source files, and the test-side consequence is the one
    assertion listed under "Round 9: the assertions that move with the
    mechanism": the report test that hands the record a
    `{ lastStage: 'plan', flow: 'light' }` state with no `planStop` hands it a
    shape the gate never writes.

## Mechanism restored to the design (round 3)

- **The funnel runs unmodified on the light seat list** (`panel.ts`): the
  pass-cap lift keyed on an unspent grant is gone, so `planPass` blocks at
  `cycle >= cfg.maxPassesPerRound` in both flows and its reason names the cap
  it applied (A16).
- **`--extend-budget` is consumed at the cap check in both flows**
  (A16, A27; correctness r1 finding 2, security r1 finding 2, performance r1
  finding 3): the over-cap pass entry spends the grant it pays for, and the
  `operatorExtension` gate log's count is the count the state carries.
  `--extend-budget 1` buys one over-cap pass; `--extend-budget 5` buys five.
- **The ownership widening is the design's** (`gates.ts`): light
  `implement`/`fix` are denied only the runner-reserved names, i.e. they own
  `TEST_PATH_RE` — A8's predicate, co-located `src/x.test.ts` included — so
  the light coder is never told it owns a file the sweep takes back
  (`TEST_OWNERSHIP_LIGHT` at `prompts.ts:22`).
- **The predicate parses no path** (A28, `02-proposal.md:302`): the
  `.`/`..`-segment refusal an earlier pass added to the spine in both flows is
  deleted, along with the two assertions that pinned it. Paths reach the
  predicate as git plumbing output — the real spelling — and `moves.ts` Rule 1
  rejects the other shapes as usage errors, which is the argument the design
  relies on; Test strategy #7 ("a POSIX repo-root-relative path gets the
  verdict of its own spelling") and #18 read accordingly.
- **The plan stop is pre-commit** (`runner.ts`): `executeStage` runs the
  stage's `preCommit` gate between the inference phase and its commit window,
  over the `docStages` measurement `executeInference` already recorded, so the
  oversized plan is never committed, the sweep still runs (the escalation-path
  posture `finalizeReviewPass(commit: false)` uses), and the document is not
  re-read or re-measured through two `git` subprocesses (performance r1
  finding 4).
- **The registry's names are the proposal's** (`flows.ts`): `Flow =
  'full' | 'light'` is what `PipelineState.flow`, `flowOf`, `configFor` and
  `assertStageInFlow` carry; the open `string` arm stays only where untrusted
  text arrives (`ParsedSource.flow`, `flowDef`'s lookup, `stagePathAllowed`
  and `moves.ts`, which validate it). `SeatPhase` is the state graph's
  `ReviewPhase` less `editorial` (deviation 20), `seatsFor` returns
  `readonly string[]`, and `configFor`'s base is `Partial<PipelineConfig>`.
- **Every panel reader takes the resolved config** (security r1 finding 5):
  `dissentingSeats(state, 'design', cfg)` in `decideNext`'s `proposal-revise`
  case, as its siblings already did.

## Blocked by the test suite (test-owner work; this stage edited no test)

1. `tests/pipeline/gates.test.ts:558` and `:586` — `ReferenceError: tempFile is
   not defined`. The helper is declared at `gates.test.ts:78`, inside
   `describe('sourceAdmissible')`, and used from
   `describe('parseSource and sourceAdmissible learn the flow key')`. Hoisting
   the helper to module scope makes both tests pass against the implementation.
2. `tests/pipeline/gates.test.ts:746` (`keeps the ADR gate shape-only in the
   light flow`) — its 120-line ADR fixture has no `## Context` and no
   `## Decision`, and `sourceAdmissible` denies it for exactly the reason the
   proposal pins: "the ADR gate is unchanged: `status: proposed` + Context +
   Decision" (Compatibility matrix, A10). The fixture needs the two sections to
   test its actual point (that the plan cap does not reach ADRs).
3. `tests/pipeline/panel.test.ts:274` and `:349` — both assert the code panel
   ends in `complexity`. `CODE_SPECIALTIES` is `correctness, compatibility,
   security, performance, testing` — the five `codereview/code-*` agents;
   `complexity` is a design-only seat. The design-phase uses of `SEATS` in that
   file are right; the code-phase ones are not.
4. `tests/pipeline/light-plan-size.test.ts:192` (`runs on when the plan is
   exactly at the cap`) — the stub answers **every** session with the plan
   reply `{"summary": "planned", "commitMessage": "docs: plan"}`, which fails
   the implement schema, so the run sits in the nudge lane and the test times
   out. The stub needs an implement-shaped reply. (Fixed by the test owner in
   round 2; listed here for the record.)
5. Two test files are not prettier-formatted, so `npm run check` stops at
   `format:check` before vitest: `gates.test.ts:706-708` wraps an `expect(`
   argument that fits on one line — prettier reflowed it to
   `expect(stagePathAllowed(…)).toBe(` with `false,` hanging — and
   `runner.test.ts:390` keeps `linesOf(state).filter(…).join('\n')` as a flat
   member chain where prettier breaks each link onto its own line. Both were
   committed unformatted by the `test-update` stage and are unmodified since;
   the `light-*` files' trailing blank lines from the round-1 list are gone.
   One command clears what is left —
   `npx prettier --write tests/pipeline/gates.test.ts tests/pipeline/runner.test.ts`
   — and `src/` is clean. This stage cannot land that: those are test files,
   and the runner reverts every test path it is handed. (Closed by the test
    owner: `format:check` is clean, so the `&&` chain reaches vitest.)

### Round 3: the assertions that move with the mechanism

These are the test expectations that pinned the pre-round-3 behavior; each is
the test owner's to move, and the implementation is at the proposal's reading.

1. `gates.test.ts:664-670` — the dot-segment assertions (`plan` on
   `docs/../src/x.ts`, light `implement` on `src/../tests/x.test.ts`) and the
   `Tests/`-case pair for the light coder. Under the design's rules a light
   `implement`/`fix` is denied only the reserved names, and a spelling is
   judged as spelled (A28, `moves.ts` Rule 1): those four expectations are
   `true`, `true`, `true`, `true`. The case-sensitivity point survives on
   `plan` (`Docs/designs/...` → `false`) and on the test-engineer stages.
2. `gates.test.ts:692-762` — the suite that caps a **source** document through
   `sourceAdmissible(path, 'plan', flow)`. That kind is gone (deviation 15);
   the gate is `planOutgrewFlow`, pinned by `light-plan-size.test.ts` driving
   a real run. The `sourceAdmissible(path, 'adr' | 'brief', flow)` cases in the
   file are unaffected.
3. `light-budget.test.ts:270-284` — the extension case pins 8 code sessions
   and an untouched grant. With the grant consumed (Mechanism section): a
   light run with `--extend-budget 1` spends its first code round's 3 passes
   (6 sessions) and stops on the pass backstop, grant 1 unconsumed because
   that entry never happened; the grant binds at a **later** code round's
   over-cap entry, where the sessions are 6 → 8 and the grant reads 0.
   The seat that asked for this also wants a second over-cap entry escalating
   and the printed grant count decreasing.
4. `flows.test.ts` — the seat-name cases stay (typo in `code` throws); add
   `code: ['complexity']` and `design: ['testing']` (each panel names only its
   own seats) and a `reviewSeats` with one phase key (a usage error, not a
   `TypeError`). Add the pin the reviews asked for: `DEFAULT_CONFIG`'s full-flow
   values — `maxPassesPerRound: 6`, `stageTimeoutMs: 30 * 60_000` — asserted
   against the constant, so the next silent edit of a cost ceiling is a test
   failure (`configFor('full', base)` against a literal base is tautological).
5. `light-plan-size.test.ts` — add the missing-document case
   (`plan document not written: …`, escalation, no `implement` session) and a
   zero-line plan (a file that exists with no bytes: `plan outgrew` does not
   fire, since 0 ≤ 80; the run goes on).
6. `states.test.ts` — a caller-supplied `cfg` design panel drives the
   dissenter list on the `proposal-revise` transition (deviation:
   `dissentingSeats(state, 'design', cfg)`).

## Round 2 status of the blocked list

Items 1-4 were closed by the test owner (the hoisted `tempFile` helper, the
ADR fixture's `## Context`/`## Decision`, the code-panel seat lists, the
implement-shaped stub reply), and `gates.test.ts:588` — the brief that is only
frontmatter — was closed here as deviation 8. Item 5, the unformatted test
files, is what `format:check` still stops on.

## Round 4 status of the blocked list

The `test-update` stage authored what the round-3 list asked for, and vitest
passes against this implementation: 41 files, 473 passed, 2 platform-skipped.
Of the blocked list only item 5 is open — two test files whose formatting is
one `npx prettier --write` away, with no assertion in it. `npm run check` is
therefore red on landing: `format:check` runs before vitest in the `&&` chain,
so the gate reports the formatting, not the suite. Nothing in round 5 touches
it; the test owner's one command closes it.

## Round 5: the assertions that move with the mechanism

Round 5 changed the mechanism the findings named, so these expectations now read
against the old behavior. Each is the test owner's to move.

1. `light-budget.test.ts:212, 244, 312, 347, 384` — `codeBudgetUsed` is the
   light flow's **fix charges** (deviation 10), not the sessions a pass spent:
   each of these runs leaves `2`, the whole light fix budget — two `fix`
   rounds. The stop records the tests pair with those numbers hold as
   written: the all-dissent run at 6 sessions on the pass backstop (`:213`),
   the single dissenter at 4 (`:245`), the forced resume at `review session
   budget exhausted (6/6 sessions)` (`:268`), the unspent-grant case on the
   pass backstop (`:286`) — a dissenter gets its **second** `fix` charge
   before the third pass entry, so the funnel's cap is the terminal record.
   The two grant-consumption tests move in shape, not only in number: their
   second resume enters a forced code round, spends its grant at the over-cap
   pass entry (sessions 6 → 8, `extensionSessions` 0, the consent line — all
   as written), and its dissent then finds the light fix budget spent by the
   first run, so the run stops there: 2 sessions on the second resume (not
   4), the terminal record `… dissent and the fix-round backstop is exhausted
   (2/2)` (not `review session budget exhausted (8/6 / 10/6 sessions)`), and
   the second over-cap entry the `--extend-budget 2` case pins is never
   entered. The session count these tests can read is
   `reviewSessionsUsed(state, 'code')` — the `reviews:` count on the status
   line — which `codeBudgetUsed` no longer is.
2. New pins correctness r1 finding 1 asked for: a light run whose tests stay
   red from `implement` **escalates** on `tests not green at the code-review
   budget limit` after two `fix` rounds (it looped unboundedly before, with no
   session cap on that path), and a light dissent gets its **second** `fix`
   round before an escalation.
3. `runner.test.ts:394-397` — the light budget line says `rounds`, as the
   full-flow line always did: the same fixture reads `code 6 rounds`, and the
   session count rides on the line's `reviews:` count, where
   `maxReviews.code` caps it (deviation 10).
4. `light-plan-size.test.ts:191-192` — the refused plan is no longer discarded
   (deviation 17): the bytes stay in the worktree for the operator to promote
   from, and the terminal escalation commit tracks them, so `ls-files` names
   the path and `existsSync` is true. The stage's own commit is still skipped.
   Extend the record assertion for finding 2: `ESCALATION.md` reads
   `Run stopped after stage \`plan\` in the **code** phase` (it said
   `` stage `` in the **design** phase on the run path).
5. `light-promotion.test.ts` — the checkpoint-identity rule is gated on
   `flow: light` (deviation 4), so a **brief** promoted to `flow: full` keeps
   A17's allocation: the counter's next free number, i.e. `003-<slug>`, a new
   identity beside the light plan. The three cases here read the light design
   directory, so they need the owner's decision: pin A17's allocation for the
   brief, or move the promotion story to an ADR source, whose file name carries
   the `NNN` and where `--force` on the same identity is what A19/A26 describe.
6. The full-flow ratify-stamp pin correctness r1 finding 7 asked for: one
   full-flow pass sequence (full → dissent → `fix` → ratify) asserting the
   ratify pass does not move `codeStamp`. The rule (deviation 3) is shared by
   both flows and has no full-flow pin today — a pin request, not a mechanism
   request.
7. The `--stage plan` acceptance (deviation 21): a light source with
   `forceStage: 'plan'`, and a light checkpoint resumed with it, starts the
   plan stage; the full-flow pair stays pinned by
   `assertStageInFlow('full', 'plan')` (`flows.test.ts:238`).

## Round 6 status of the blocked list

All five blocked items are closed: 1-4 by the test owner (round 2), item 5 by
the test owner's formatting commit (round 6), and the round-5 assertion list
landed in `test: align the light-flow pins with the round-1 correctness
findings`. The gate is green — `npm run check` exits 0, 41 files, 488 passed,
2 platform-skipped — and this round moved no assertion: the two
`--extend-budget` cases in `light-budget.test.ts` were authored against the
design's shape (the grant buys over-cap sessions, the fix budget stays at 2)
and the implementation, not the tests, is what changed (deviation 24).

## Round 7: the assertions that move with the mechanism

Round 7's mechanism (deviation 25) has no pin yet. Each of these is the test
owner's; no test file was edited here.

1. The shape finding A is about — `light-plan-size.test.ts`: resume a light run
   a plan stop ended (`lastStage: 'plan'`, `planStop` set, the 120-line plan on
   disk) and assert the run ends `escalated` with **no** session created past
   `plan` — the refused plan never reaches `implement`. Its pair: resuming the
   same checkpoint with `forceStage: 'plan'` re-plans, and a plan within the cap
   clears the stop (`planStop` absent from the checkpoint, the `plan-size` gate
   log names the clear) and the run goes on to `implement` and `done`.
2. `light-plan-size.test.ts:187-208` — the footer shape: a plan-stop record
   prints `pipeline run <source> --force` and
   `pipeline resume 002-x --stage plan`, and **not** the bare
   `` `pipeline resume 002-x` `` line. The STOP string, the promotion string, the
   phase label and the refused-plan assertions stay as written.
3. `states.test.ts` — the `case 'plan'` pair: with `planStop` set, `decideNext`
   escalates with the stored record verbatim; with it absent, `implement` (the
   round-5 pins at `:338-343` and `:416-419` are unchanged).
4. `state-store.test.ts` / `checkpoint-compat.test.ts` — `planStop` round-trips
   through `saveState`/`loadState` at `version: 1`, and every pre-`planStop`
   fixture loads and resolves as it did.
5. `escalation.test.ts` — the terminal's repair lines: a plan-stop escalation
   prints the promotion and the re-plan commands; every other stop prints the
   one line it printed before (`retry after a fix: pipeline resume <id>`, the
   pin at `:279`).
6. `runner.test.ts` — `statusLines` gains the plan-stop line only while a stop
   stands; the full-flow block stays byte-equal (the `:370-379` pins are states
   with no stop, so they are unchanged).

## Round 9: the assertions that move with the mechanism

Round 9's predicate (deviation 29) is what one existing fixture was written
against the old way. Each item is the test owner's; no test file was edited
here.

1. `escalation.test.ts:171-176` (`repairs a plan stop with promotion, not with
   a plain resume`) — its state is `{ lastStage: 'plan', flow: 'light' }` with
   **no** `planStop`, a shape the gate never writes (it records the refusal
   before the escalation's checkpoint save, and clears it when a plan passes).
   Under the record predicate that state is a plan that cleared the gate, so
   the footer is the plain resume and `not.toContain('pipeline resume 002-x')`
   is red. Hand it `planStop: planReason` — the shape the stop leaves — and its
   two assertions hold as written; the `code-review` pair at `:178-182` keeps
   naming the bare resume, unchanged. The recipe-singularity case at `:158-164`
   reads the same fixture; giving it `planStop` too is the honest shape, and
   both of its assertions hold either way.
2. New pin the finding asks for (a): a light run escalated with
   `lastStage: 'plan'` and **no** `planStop` — the failing-`implement` shape —
   prints the plain `pipeline resume 002-x` and no plan-stop claim: no
   "The plan stop is sticky", no `pipeline run <source> --force`, no
   `pipeline resume --stage plan`. Its pair is the state-graph half the finding
   cites: with no stop, `decideNext` answers `stage('implement')`, so the
   record's footer and the transition agree.
3. New pin the finding asks for (b): a **running** light state at
   `lastStage: 'plan'` with no `planStop` prints no `plan stop:` line in
   `statusLines`, and its repair is the plain resume — the healthy run mid-
   `implement` must not read as stopped.
4. The review's out-of-specialty note: `statusLines`' plan-stop line has no
   assertion on its text anywhere in the suite. Pin it while the shape is
   here — a light state with `planStop` set prints one line that starts
   `plan stop: `, quotes the recorded refusal verbatim, and names both
   commands `repairSteps` prints (`pipeline run <source> --force`,
   `pipeline resume --stage plan 002-x`) with no bare `pipeline resume 002-x`.

## Housekeeping

`docs/designs/010-lightweight-plan-flow/ESCALATION.md` was deleted in the
working tree by an earlier attempt (a prior run's marker, committed at HEAD);
restored to its committed bytes, so this change does not delete a run record.
Round 6's reproduction escalated the run, which minted the marker again, and a
resume cleared it; it is restored to its committed bytes once more, so the
diff this round lands is `states.ts` and these notes.

`docs/designs/010-lightweight-plan-flow/.pipeline.json` moved in the round-5
diff because the runner writes the checkpoint it is driving: it carried the
`test-update` stage's result entry, the gate line for the red `testsGreen`
check, and `lastStage`/`testsNeedUpdate` with it. Nothing in round 6 edited it
by hand, and `transcripts/` is untouched.

## As-built closure — 2026-11-12

This pass records the shipped change set against the proposal; it changes no
mechanism, budget, seat list, ownership rule, state graph, or test. The
implementation lands the design's core contract: `flows.ts` and the two
optional config fields; the light registry and budgets; the `plan`,
`planner`, `plan.md`, and `plan-note` surfaces; the light transitions;
flow-threaded ownership and move validation; the 80-line plan gate at the
pre-commit meter; flow-scoped status/escalation reporting; and the promotion
path.

### Status of the round-9 assertions

The round-9 list is closed. The plan-stop tests now hand the report and status
the shape the gate writes: `escalation.test.ts:173-209` covers the recorded
promotion repair and the no-stop plain-resume repair,
`escalation.test.ts:233-265` pins `hasPlanStop` on the record, and
`runner.test.ts:438-487` pins the `plan stop:` line only for a state carrying
`planStop`. The sticky stop and its re-plan are pinned end to end in
`light-plan-size.test.ts:284-331`. The review's r1c5 ratification confirms the
same pins.

### Latest review disposition

The ratified code-review record at this point is `correctness-r1c4` and the
`c5` ratification passes for `security`, `compatibility`, `performance`, and
`testing`. No seat leaves a blocker or major open. The open non-blocking gaps
recorded in the proposal's As-built note are:

- the converged code-review gate-log reason hard-codes "all five seats" even
  for a two-seat light panel (`runner.ts:2740`);
- the plan-stop promotion command prints `state.source.path` unquoted, so a
  source path needing shell quoting makes the printed repair unusable
  (`report.ts:290-306`, `runner.ts:3277-3278`);
- the shared repair refactor labels the full flow's terminal footer, and that
  label is not pinned by the full-flow escalation test
  (`runner.ts:3231-3237`, `report.ts:279-309`);
- a light `--extend-budget` grant still increments `extensionRounds`, which
  light has no site to spend (`runner.ts:2231-2234`, `2294-2303`);
- the shared `codeRound > maxRounds` guard is not grant-aware, so a full
  over-cap code round can stop after one pass (`states.ts:229-233`);
- a BOM before frontmatter is not stripped, so a BOM-authored `flow: light`
  document parses as `full` (`gates.ts:54-74`);
- reserved-name coverage omits three document stages, planner agent permissions
  are not read as a test fixture, and `planMaxLines` non-integer/`NaN`
  validation is not fully covered (`gates.test.ts:661-672`,
  `prompts.test.ts:467-474`, `flows.test.ts:323-327`).

These are record and edge gaps; none changes the flow's stage sequence,
panel width, plan cap, ownership widening, budgets, or full-flow behavior.

### Gate evidence

The last executed package gate recorded in this file is round 8's:
`npm run check` exits 0, lint clean, `format:check` clean, vitest at 41 files,
492 passed / 2 platform-skipped, and `npm run typecheck` exits 0. Round 9's
source files and the round-9 test pins landed after that datum. This
documentation pass has no shell (`Permission denied: shell`), so it did not
re-run `npm run check`; the orchestration CI job on linux at Node 22.23.4 is
the standing proof for Test strategy #21. Do not restate the suite as green
from this pass alone — that claim requires the command's exit status and
vitest's counts.

## As-built verification — 2026-11-12 (second pass)

This pass completes the as-built record; it changes no mechanism, budget, seat
list, ownership rule, state graph, prompt, agent, skill or test, and it edits
only these two documents. The proposal's As-built section carried nine drift
rows and a seven-item gap list; this pass re-derived every one of them from the
shipped tree and found each at the site its row names, so none is restated here.

### The one drift the first pass had not recorded

31. **A pre-commit gate's refusal records the stage it stopped, and the plan
    gate writes a paired record** (`runner.ts`: `executeStage`, `recordPlanStop`,
    `clearPlanStop`; `gateLog` gate `plan-size`). The proposal fixes the stop's
    *position* (the stage-ok/pre-commit point) and its escalation string, and
    says nothing about the record the stop leaves. `state.lastStage` is assigned
    only after `runWithRetry` returns, so a refusal raised from inside the stage
    reached `finalizeEscalate` with an empty name: `escalationPhase` answered
    `design` and ESCALATION.md opened "Run stopped after stage `` in the
    **design** phase" — the mislabel A15 adds `plan` to `CODE_STAGES` to remove.
    `executeStage:1881` records the stage on that path, and the gate writes
    `plan-size` denials at `runner.ts:2785` and `2793-2796` and the
    clearing entry at `2381-2387`. This is round 5's deviation 22 stated as
    shipped behavior; it lands in the proposal's As-built table as the tenth
    row, affecting Reporting, The `plan` stage and the State graph delta.

### Verification of the earlier rows and of the round-9 pins

The shipped tree reads as the rows say: `flows.ts:16-163` (the registry,
`SeatPhase`, `FlowDef.path`, base-then-resolved validation, phase-scoped seat
names, `phasesRun`-scoped emptiness); `stages.ts:89-120` (`DEFAULT_CONFIG` at
5 / 40-20 / 6 / 30 min, `reviewSeats` and `planMaxLines` optional) and
`stages.ts:143` (`STAGE_BASES.plan`); `state-store.ts:60-75` (`version: 1`,
`flow` an optional enum, `planStop` an optional string); `gates.ts:28-124`
(`body`-judged brief gate, the caller's parse as an argument,
`planOutgrewFlow`), `gates.ts:371-402` (reserved names first, light
`implement`/`fix` owning `TEST_PATH_RE`, `plan` on `docs/**`);
`moves.ts:157-171` (`isStageInFlow` before the ownership predicate);
`identity.ts:51-119` (the light-only `nnnOfExistingRun`, `sourceKind`
exported); `states.ts:78-80` (`budgetLeft` reading no round grant for light),
`states.ts:118-139` (`case 'plan'` on the record) and `states.ts:229-233` (the
shared round guard); `report.ts:252-309` (`hasPlanStop` on `state.planStop`,
`repairSteps`' two repairs) and `report.ts:36-47` (`plan` in `CODE_STAGES`);
`runner.ts:2124-2133` (the pass ledger in handed order),
`runner.ts:2620-2627` (a ratify pass keeps its stamp), `runner.ts:3160-3167`
(the suffix at one site, on `implement`/`fix`/`code-review`),
`runner.ts:3289-3335` (`slug:`, the flow line, the plan-stop line);
`prompts.ts:21-66` (`testOwnershipLine(flow: Flow)`, the renderer's
full-flow fallback) and `runner.ts:2926-2934` (`testOwnership`, `planMaxLines`
in `stageVars`); `panel.ts` with every reader taking the caller's `cfg`.

The round-9 assertion list is closed, and its pins are where the closure says
they are: `escalation.test.ts:173-189` (the recorded stop prints the promotion
and the `--stage plan` re-plan, no bare resume; a `code-review` stop still
prints the bare resume), `escalation.test.ts:199-209` (the failing-`implement`
shape prints the plain resume and claims no stop), `escalation.test.ts:233-264`
(`hasPlanStop` on the record, the stage alone as no stop, nothing for a full
run), `runner.test.ts:438-487` (the `plan stop:` line only for a state carrying
`planStop`, quoting the recorded refusal and both commands — the review's
out-of-specialty note, round-9 item 4, is pinned), and
`light-plan-size.test.ts:290-331` (a resume spends no `implement` session on the
refused plan; a `--stage plan` re-plan under the cap clears the stop and ends
`done`). The recipe-singularity case at `escalation.test.ts:158-164` keeps the
no-`planStop` fixture; both of its assertions hold under the record predicate,
which is what the round-9 list predicted.

### Status of this pass

The seven non-blocking gaps in the proposal's As-built note are unchanged and
each was re-found at its cited site (see that note's verification section for
the list with lines). The gate datum is unchanged from round 8: this pass has no
shell (`Permission denied: shell`), so it ran no `npm run check`, no
`npx prettier --check` and no vitest, and claims nothing about the suite beyond
the recorded datum. The orchestration CI job on linux at the `engines` Node
floor remains the standing proof for Test strategy #21.

For independence: this pass read no run record — not `transcripts/`, not
`.pipeline.json`, not the `ESCALATION.md` marker — and wrote no file outside
`02-proposal.md` and this one. Its evidence is the source tree
(`src/pipeline/`, `prompts/`, `tests/pipeline/`), the agents and skills, and
`docs/workflows.md`.

