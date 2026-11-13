# Code review — security — r1 c5 (ratification)

Scope: `git diff 96f50c2110560362e5fb61f05bd313f04bf503cb..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/010-lightweight-plan-flow.md`.
Checklist: `references/security.md` only. Ratification: later fixes must not break what
the r1 security seat approved.

**Verdict: approve.** No blocker and no major finding. My r1 finding 2 (the light flow's
unconsumed round grant) is closed, and the closure is fail-closed.

## Method note (limits of this pass)

The `shell` tool is denied in this session (`Permission denied: shell` on every
invocation, including `git status`), so the diff range could not be read hunk by hunk.
This pass therefore reviewed the **current tree** at every path the caller named
(`tools/orchestration/src/pipeline/*`, `tools/orchestration/tests/pipeline/*`,
`.opencode/agents/design/planner.md`, `tools/orchestration/package.json`) and judged
each security-relevant property against the r1 approval, using the code's own
provenance markers (deviation numbers in `03-build-notes.md`) to tell r1-era code from
later fixes. Provenance of individual lines is consequently asserted where the
build-notes deviation names the fix round, and flagged as unverified where it does not
(finding 2).

## My r1 finding, re-checked

**Finding 2 (minor, round 1) — RESOLVED.** The light flow no longer arms a round grant.

- `states.ts:78-80` — `budgetLeft` is now
  `used < cfg.maxRounds || (flowOf(state) !== 'light' && (state.extensionRounds ?? 0) > 0)`.
  A light run that ever holds a round grant no longer satisfies the gate, so the
  fix → red tests → fix loop cannot stay legal for the checkpoint's whole life.
- The light loop is charged and bounded where it is entered: `runner.ts:2754-2762` charges
  `state.codeBudgetUsed += 1` at **every** `fix` entry (the `!isLight` gate is gone), and
  `states.ts:214-222` reads that charge on both entry paths — red tests and dissent.
  `states.ts:229-233` bounds the round number itself in both flows.
- The full flow is unchanged on the shared lines: for `full`, `flowOf(state) !== 'light'`
  is true, so `budgetLeft` is the r1 expression; `if (!isLight) state.designBudgetUsed/codeBudgetUsed += 1`
  (`runner.ts:2605,2608`) keeps full's round-entry charge; the new `fix` charge is the full
  flow's existing charge. `state.codeCycle = 0` / `state.designCycle = 0` stayed **outside**
  the `!isLight` guard (`runner.ts:2603-2609`), so the light funnel's per-round pass budget
  still resets — the light round is bounded by `maxPassesPerRound: 3` and the cumulative
  `maxReviews.code: 6`, not by the skipped round gate.
- The session grant (my r1 finding 1) is still consumed at pass entry in **both** flows
  (`runner.ts:2652-2674`), decrements the state, and prints the state's own count.
  Pinned: `light-budget.test.ts:356-421` (`extensionSessions` 1 → 0, printed grant left `1`,
  one `operatorExtension` line per over-cap pass, surplus left unspent).
- Red-test path pinned end to end: `light-budget.test.ts:306-326` — exactly
  `plan, implement, fix, fix`, `codeBudgetUsed: 2`, `tests not green at the code-review
  budget limit` in the gate log and in `ESCALATION.md` with the light suffix.
- Full-flow consumption still pins unchanged: `budget.test.ts:193-261`
  (`extensionRounds` → 0, `over-cap design round entered` consent line).
- Recorded as deviation 24 (`03-build-notes.md:357-376`), which states the same shape I
  reported: the light flow has no site that spends a round grant, so reading it at the
  shared gate bought an uncharged third `fix` round with the grant undecremented.

## What the later fixes added, and how it reads on security

- **Sticky plan stop (deviation 25).** New checkpoint field `planStop`
  (`state-store.ts:75`) is typed, not coerced, and optional per ADR-006 §2; its only writer
  is `recordPlanStop` (`runner.ts:2365-2368`) with runner-authored text (a line count and a
  path it built), so no model-authored string reaches it. `hasPlanStop` (`report.ts:252-254`)
  requires `flowOf(state) === 'light'`, so a forged full-flow record prints nothing
  (`escalation.test.ts:241-262`). `decideNext`'s `case 'plan'` (`states.ts:136-139`)
  escalates on the record, so a plain resume spends **zero** sessions on the refused plan
  (`light-plan-size.test.ts:290-312`) — the stop cannot be walked past, and it cannot loop
  into a session burn. The repair footer prints only the two moves the state machine
  honours; they are printed as text, never executed.
- **Ownership spine (the r1 spine) intact.** `stagePathAllowed` validates the flow first
  (`flowDef`, `gates.ts:371-374`), so an unknown flow is a refusal, never a widening to
  `full`'s rules (`gates.test.ts:726-728`); `namesReservedSegment` stays first and wins in
  both flows (`gates.ts:382-384`), asserted for ten stage classes × seven reserved
  spellings × both flows (`gates.test.ts:660-689`). The widening is exactly
  `implement`/`fix` (`gates.ts:387-395`), pinned at the sweep (`light-ownership.test.ts`),
  and the light `plan` keeps the document-stage rule with the accepted `docs/**` widening
  pinned as allowed, not claimed as a confinement. `moves.ts:157-171` adds
  `isStageInFlow` ahead of the shared predicate, so a stage the flow never runs cannot
  declare moves; Rules 1 and 4-7 (shape, `.`/`..`, absolute, realpath, gitdir) are
  untouched and the sweep's intake stays git plumbing output (A28), asserted with the
  `docs/../src` spelling (`light-ownership.test.ts:132-143`).
- **Budgets/seats as untrusted config.** `configFor` validates the caller's base *and* the
  merged result (`flows.ts:120-134`), pre-lock in both entry paths
  (`runner.ts:2199`, `runner.ts:2272`), with `validatePanelPresence` scoped to the phases
  the flow runs; empty code panel, unknown seat, non-integer `planMaxLines` are usage
  errors, and `--max-parallel` survives by spread order. Pinned pre-lock with zero side
  effects at `light-gates.test.ts:221-279` (no lock, no design dir, no client call).
- **Checklist walk.**
  1. External input: `flow:` frontmatter is carried verbatim to `flowDef`, the single gate
     (`gates.ts:39-74`, `flows.ts:104-110`), validated pre-lock before lock/dir/session
     (`runner.ts:2174-2188`), and a checkpoint's `flow` is an enum, not coerced —
     `flow: "sideways"` refuses to load (`checkpoint-compat.test.ts:143-159`).
  2. Path handling: no traversal accepted (moves Rule 1), no absolute path from data, no
     interpolated shell string anywhere in the diff's paths.
  3. Injection: no new shell invocation; the cargo/npm gates take fixed argv arrays. New
     regexes are linear (`/^flow:[ \t]*([^\r\n]*)/m`, `{{\w+}}`). Model-authored text into
     reports stays behind `printable`/`boundedField` (`report.ts:363-386`,
     `runner.ts:3290-3324`, `finalizeEscalate` `runner.ts:3155`).
  4. Secrets/PII: none in the new source, agents, prompts or fixtures; log lines carry
     counts, slugs, paths.
  5. New dependencies: none — `tools/orchestration/package.json` deps are the pre-existing
     `@opencode/client`, `yargs`, `zod`; the frontmatter scan is a regex, deliberately no
     YAML parser (A10/A20). Nothing to check against advisories.
  6. Filesystem writes: checkpoint and escalation marker stay tmp+rename with the recorded
     bytes fed to the authorship record (`state-store.ts:325-358`, `runner.ts:3206-3224`);
     the plan stop's refusal writes nothing new (the sweep is the only revert, documented).
  7. Clean rejection: the pre-lock refusals and the checkpoint refusals surface as single
     messages with named repairs (`cli.ts:236-243` prints `error.message`, not a stack);
     `loadState` size-caps before parsing.

## Findings

1. **minor — `tools/orchestration/src/pipeline/states.ts:224-233` (read with
   `runner.ts:2576-2610`, `states.ts:78-80`).** The over-cap round guard is ungated, so it
   truncates the **full** flow's `--extend-budget` on the code phase. `state.codeRound >
   cfg.maxRounds` escalates at the head of `case 'code-review'`, ahead of the grant-aware
   `budgetLeft` and ahead of `planPass`. In the full flow a granted over-cap code round is
   legal by construction — `runner.ts:2577-2601` consumes an `extensionRounds` grant and
   charges the round — which leaves `codeRound == maxRounds + 1`. The first time
   `decideNext` runs after that round's first pass, the guard fires: the round runs one
   pass, its dissent never reaches the `fix` the grant paid for, a pending ratify pass
   cannot run, and the remaining grants are dead. The run then escalates with
   `code review: the fix-round backstop is exhausted (round N)`, a string that carries no
   `--extend-budget` hint, and its footer offers a plain `pipeline resume`, which
   re-escalates on the same line. That is the code-phase mirror of the clause deviation 10
   deliberately kept live for design rounds (`states.ts:156`/`194` have no round-number
   guard, which is why `budget.test.ts:193-261` stays green: it pins the design phase
   only), and it erodes the ADR/proposal Non-goal "no change to the full flow's budgets,
   escalation text". I could not diff the range (no shell), so I cannot prove this line
   moved in the c5 window; it is described as part of deviation 10's charging work
   (`03-build-notes.md:213-214`: "`states.ts`'s `codeRound` check guards the round number
   itself in both flows"), which lands it in this change set. Nothing is over-spent — the
   failure is closed, and the light flow's bound is the intended one.
   **Fix:** make the guard mean what the grant means, in the flow the design bounds: gate
   it (`flowOf(state) === 'light' && state.codeRound > cfg.maxRounds`) or make it
   grant-aware (`state.codeRound > cfg.maxRounds && (state.extensionRounds ?? 0) <= 0`) so
   the full flow's granted over-cap round can run its passes and answer its dissent while
   the light flow keeps the hard cap. Then pin it in `budget.test.ts` with a full-flow
   `--stage code-review --extend-budget 1` case that asserts the granted round runs more
   than one pass and spends its grant with an `operatorExtension` line, beside
   `light-budget.test.ts:280-297`, which pins the light reading.

2. **nit — `tools/orchestration/src/pipeline/runner.ts:2231-2234` and `2294-2303`
   (consumer: `states.ts:78-80`).** A light `--extend-budget` increments
   `state.extensionRounds` that the light flow never consumes. After the fix the grant is
   inert (fail-closed, so not the bypass I reported in round 1), but it is not nothing in
   the record: the checkpoint accumulates a number with no reader, no `operatorExtension`
   line ever names it, and `statusLines`/`escalationMarkdown` print no grant at all, so an
   operator reading a light checkpoint sees a standing "round extension granted" count
   that buys nothing — the same record-honesty shape as r1 finding 2, on the harmless side.
   Promotion cannot rescue it: `runPipeline` builds `freshState`, so a `--force` full entry
   carries no light grants into the full flow's live clause.
   **Fix:** charge only the half the flow spends —
   `state.extensionSessions = (state.extensionSessions ?? 0) + opts.extendBudget` alone when
   `flowOf(state) === 'light'` (both entry sites) — or keep the increment and gate-log it
   once at grant time with the reason the light flow sets no round cap to extend, so the
   record states what the grant did not buy. `light-budget.test.ts` then asserts
   `extensionRounds` is absent (or logged) on a light grant.

## FYI (no severity, outside the seat)

`identity.ts:119-141` (`nnnOfExistingRun`) `JSON.parse`es every
`docs/designs/<nnn>-<slug>/.pipeline.json` whose slug matches, with no size ceiling — the
8 MiB cap is `state-store.ts:150`, not here. Not reachable as a memory attack: the
reserved-name rule denies every stage the checkpoint path, and every read error is
swallowed. Mentioning it only so the seat that owns size ceilings knows the second reader
exists.
