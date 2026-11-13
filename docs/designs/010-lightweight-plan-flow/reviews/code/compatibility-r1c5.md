# Code review — compatibility, round 1 pass c5 (ratification)

Scope as given: `git diff eb8b40ad413a397f71b977f6d0dcd6c903f0d17c..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/010-lightweight-plan-flow.md`.
Posture: this seat approved at round 1; this pass checks only that the revisions that landed
after that approval did not break what was approved. Inputs used: the diff's source/test paths,
the proposal, the ADR, and this seat's own round-1 findings. No other seat's report, transcript
or verdict was read.

Method note, stated up front: this seat has no shell (the `shell` tool was denied), so the diff
range could not be run and the suite could not be executed. The pass is therefore a state review
at HEAD over the paths the range covers, plus the fix sites named by my own round-1 findings.
That method limit is itself recorded as a finding (item 7 of the checklist stays unevaluated).

## Verdict

**approve** — no open blocker or major finding. Three items below: two created by the fixes that
landed after the round-1 approval (one minor, one nit), and one checklist item this seat still
cannot evaluate from the change.

## What the later fixes touched, and whether they broke anything

Compatibility surfaces the post-approval fixes reached, checked one by one:

1. **Checkpoint grammar (ADR-006 §2).** `planStop` enters `StateSchema` as
   `z.string().optional()` (`state-store.ts:75`), beside the additive-optional `flow`
   (`state-store.ts:70`); `version` stays `z.literal(1)`; nothing was removed, retyped, renamed
   or re-interpreted. `saveState` validates with the same schema, so the writer cannot mint a
   checkpoint the loader rejects. The pre-`flow` era fixtures and the new `light-era.json`
   (`tests/pipeline/fixtures/checkpoints/light-era.json`) are both `version: 1` shaped. **No
   regression** — the sticky-plan-stop feature did not widen the resume contract.
2. **State graph.** Every full-flow `case` in `states.ts` is untouched; the two new guards are
   flow-gated (`states.ts:113, 118-139, 210, 218, 238`). `budgetLeft` (`states.ts:78-80`) keeps
   the full flow's test byte-identical (`used < cfg.maxRounds || extensionRounds > 0`) and adds
   the light-only no-extension rule, which is the deviation the code names. `case 'plan'` throws
   for a non-light state (`states.ts:121-123`) — an unreachable state in the full flow, so no
   full-flow path changes. **No regression.**
3. **Full-flow budgets.** The knobs my round-1 finding flagged are back to the design's values:
   `maxPassesPerRound: 6` and `stageTimeoutMs: 30 * 60_000` (`stages.ts:107,117`), with the
   comment stating that the light 12 minutes arrives only through `FLOWS.light.config`
   (`flows.ts:86`). Pinned by tests: `flows.test.ts:142` asserts `DEFAULT_CONFIG.stageTimeoutMs`
   is `30 * 60_000` and `:149` asserts the resolved full config is. `docs/workflows.md:244-252`
   (per-flow table) and `:256-272` (defaults list) read 20 sessions / 6 passes / 5 rounds / absent
   `planMaxLines` / 30 min for `full` and 6 sessions / 3 passes / 2 rounds / 80 lines / 12 min for
   `light` — every row now matches the shipped registry. My round-1 finding 1 is closed, by
   aligning the code to the documents (ADR-010's table pins full at 30 min / 6), not by editing
   the table. **No regression; the operator-facing budget table no longer contradicts the runner.**
4. **Reporting for a full state.** `hasPlanStop` requires `flowOf(state) === 'light'`
   (`report.ts:252-254`), so a full checkpoint gains no `plan stop:` line
   (`runner.ts:3271-3280`) and no promotion recipe; `repairSteps` returns the one standing step
   for it (`report.ts:281-289`), and `escalationMarkdown` prints that step as note + command
   (`report.ts:392-394`) — the footer text is the prose the report has always carried
   ("Fix the underlying cause (usually the brief or the proposal), then run:" + `pipeline resume
<nnn>-<slug>`), asserted for a full state at `escalation.test.ts:368` and for a non-plan light
   stop at `:199-207`. The flow line is the additive line the design asked for
   (`runner.ts:3296-3298`, `report.ts:325-327`), and `runner.test.ts:393-398` pins that an
   explicit `flow: 'full'` renders identically to an absent key. **No regression.**
5. **Escalation suffix.** The light promotion suffix is appended at the one site, gated on the
   flow and on `lastStage` ∈ {implement, fix, code-review} (`runner.ts:3160-3167`), and the
   plan-stop record's own string already names the promotion (`runner.ts:2805-2808`), so no
   record gets the recipe twice. `docs/workflows.md:149-167` states the same rule. **Consistent
   with the design's State graph delta; full flow untouched.**
6. **Entry / identity.** One pre-lock `parseSource` (`runner.ts:2174-2188`), the bad-value usage
   error before the lock (`:2182-2186`), `configFor` validated pre-lock (`:2199`, `:2272`),
   `assertStageInFlow` at both pre-lock points (`:2219`, `:2282`), the flow stamped on the entry
   save (`:2227`), and `sourceAdmissible` judging that same parse (`:2248`). The identity rule
   added for re-planning is light-only (`identity.ts:69-74`), so an ADR's or a brief's full entry
   allocates `NNN` exactly as before — the matrix's "Existing ADRs and briefs … unchanged" row
   holds. **No regression.**
7. **Ownership, meter, seats.** The plan gate runs at the meter's own count at the pre-commit
   point (`runner.ts:2775-2810`), including the "no document written" case the doc claims
   (`runner.ts:2778-2786` vs `docs/workflows.md:149-151`), and `docStages` replaces the meter's
   stage list (`runner.ts:1702`) with `full`'s list equal to the three proposal stages
   (`flows.ts:69`). Panel and funnel readers take the resolved config (`panel.ts:32-96`), and the
   pass backstop reads `cfg.maxPassesPerRound` with no extension channel (`panel.ts:133-139`), as
   `docs/workflows.md:260-267` states. **No regression.**
8. **Platform surface (checklist items 1, 2, 4, 6).** No new platform-specific code: the diff
   adds no `process.platform` branch, no native dependency (`package.json:33-49` carries only the
   pre-existing `@opencode/client`, `yargs`, `zod`), and no new shell-out — the plan meter is the
   JS line count (`runner.ts:200-202`), as the matrix's Windows row claims. The only Node API of
   note in the touched files is `structuredClone` (`state-store.ts:383`), above nothing in the
   floor: `engines.node` is `>=22.23.4` (`package.json:30-32`) and the CI leg pins that version on
   `ubuntu-latest` (`.github/workflows/check.yml:36,48,52-54`). No light test carries a
   `runIf`/`skipIf` (the only platform guards in the suite are `moves.test.ts`'s, from an earlier
   run), so the suite's linux claim is not asserted by a test that skips everywhere.
   **Matrix rows hold.**

My round-1 findings, disposition under this pass's rule (a previously-accepted minor is not a
break, so it is not re-reported as a new finding):

- **Finding 1 (budget-table/timeout divergence) — closed**, as described in item 3 above.
- **Finding 2 (BOM-authored source document reads as `full`)** — **unchanged in shape, so not a
  regression and not re-reported.** `parseSource` is still anchored `^---` with no leading
  U+FEFF strip (`gates.ts:55-58`), so a brief saved with a BOM still yields `front === null`,
  `status: 'none'` and `flow: 'full'` (`gates.ts:72`); the pre-lock read inherits that value
  (`runner.ts:2181`) and cannot raise the bad-value error. No test in `tests/` mentions a BOM
  (searched for `FEFF` and the escapes). The accepted widening from round 1 stands as it was
  approved; the fix this seat asked for (strip U+FEFF, add the `flow: light` + BOM gate case) has
  not landed.
- **Finding 3 (`run:` → `slug:` label rename in the full flow's status block)** — **unchanged, not
  a regression.** The label is `slug:` (`runner.ts:3304`) and the rename stands as the pinned
  contract; `runner.test.ts:376` now asserts a line carries that label, and `:393-398` pins
  explicit-full against absent-key. The exact label column of the full block is still not
  asserted, which is the residual my round-1 fix named.
- **Finding 4 (`npm run check` green on linux at the engines floor)** — **still unevaluated**,
  restated below as the open item it is.

## Findings

### 1. minor — the plan-stop repair prints the source path unquoted, so the recipe breaks for a
path the shell splits

- **Location:** `tools/orchestration/src/pipeline/report.ts:290,298` (with
  `tools/orchestration/src/pipeline/runner.ts:3277-3278`)
- **Problem:** `repairSteps` builds the promotion command by interpolating the checkpoint's
  `state.source.path` directly: `` `pipeline run ${source} --force` ``. That path is the
  document's real repo-relative path (`runner.ts:2224`), which is **not** sanitized — only the
  identity's slug is (`identity.ts:34-45`, `sanitizeSlug`). A brief whose name contains a space,
  an apostrophe, or any shell metacharacter (`docs/prompts/mirror tests, src.md`) is a legal
  entry path for the light flow — `sourceKind` classifies by prefix (`identity.ts:90-92`),
  `slugFromSource` folds the illegal characters out of the slug, and the counter allocates
  `NNN` (`identity.ts:69-73`) — and the repair this pass added then prints
  `pipeline run docs/prompts/mirror tests, src.md --force`, which yargs reads as a source plus a
  stray positional (`cli.ts:109-131`: `run <source>`). `statusLines` compounds it:
  `planStopStatusLines` joins the two commands with `   or   ` (`runner.ts:3277-3278`), so a
  split path also loses the separator's meaning. This is the operator's only route out of a
  sticky plan stop (a plain resume re-escalates by design, `states.ts:136-138`), so the record
  points at a command that does not run. The compatibility matrix's non-ASCII/odd-spelling
  posture is that IDs survive them — the identity does, the printed command does not.
- **Fix:** quote the path in the printed command when it is not already shell-safe — e.g.
  `const arg = /^[A-Za-z0-9._/-]+$/.test(source) ? source : \`'${source.replace(/'/g, `'\\''`)}'\``
  — and use `arg` in both steps; add a `light-promotion` case for a brief whose path needs
  quoting, asserting the printed `pipeline run …` line round-trips through the CLI's parser
  (yargs `run <source>`), and assert the same line `status` prints.

### 2. nit — the repair refactor put a label column on the full flow's escalation terminal text,
and the full-flow footer is unasserted

- **Location:** `tools/orchestration/src/pipeline/runner.ts:3231-3237` (with
  `tools/orchestration/src/pipeline/report.ts:228-235,279-309`)
- **Problem:** to make the plan stop print two moves, the footer moved into `RepairStep`, and the
  terminal print now renders `` `  ${step.label}: ${step.command}` `` — so a **full** run's
  escalation terminal gained a label column (`  retry after a fix: pipeline resume 002-x`). The
  proposal's Non-goals list "No change to the full flow's … escalation text", and this seat
  cannot verify from the change alone that the label is the pre-branch string: this pass had no
  shell, so the range's before-image of that line is unreadable, and no test pins the label —
  `escalation.test.ts:366-368` asserts only that the terminal print contains `pipeline ESCALATED`
  and the resume command. `ESCALATION.md` itself is safe (its footer is note + command,
  `report.ts:392-394`, asserted at `:188` and `:204`).
- **Fix:** confirm `retry after a fix` is the label the full flow's terminal printed before this
  branch, and keep it if so; if it is new, restore the pre-branch label for the non-plan-stop
  step. Either way, extend the escalation test to assert the full-flow footer block
  (label + command) so the Non-goals promise is pinned and a later shared-shape refactor cannot
  move it silently.

### 3. minor — checklist item 7 ("`npm run check` stays green on linux at the engines floor")
remains unevaluated by this seat

- **Location:** `tools/orchestration/package.json:23,30-32`
  (`.github/workflows/check.yml:36,48,52-54`)
- **Problem:** this seat was denied the shell, so `npm run check` (lint, `format:check`, vitest)
  was never executed and the suite's pass count is unknown to this review. The per the skill an
  item a seat cannot evaluate is a finding, not a pass. The static half of the claim checks out
  and nothing in the post-approval fixes undermines it: the orchestration CI job runs
  `npm run check` on `ubuntu-latest` with `node-version: "22.23.4"`, which is exactly
  `engines.node`'s floor; no new test carries a `runIf`/`skipIf`/platform guard, so no test
  claims a platform it skips; the new light suites use only the suite's existing
  `mkdtempSync(tmpdir(), …)` pattern; and the touched files add no API above the floor. The
  change still carries no executed datum.
- **Fix:** run `npm run check` in `tools/orchestration` at Node 22.23.4 on linux and record the
  exit status and vitest suite/test counts in the run's build notes, so the compatibility row
  rests on a datum; the CI job is the standing proof afterwards.

## Out of specialty (FYI, no severity)

The light entry's checkpoint scan (`identity.ts:119-141`) reads every design directory's
`.pipeline.json` with an uncapped `readFileSync` + `JSON.parse`, where `loadState` sizes its read
against `MAX_CHECKPOINT_BYTES` (`state-store.ts:150-165`); the light path is new, the full path is
untouched, and the file is runner-authored — an FYI for the seat that owns untrusted-input sizing.
