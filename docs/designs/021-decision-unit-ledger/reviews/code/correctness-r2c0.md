# Code review — correctness (round 2, panel seat c0)

Scope: `git diff 1bda5c34f1561afd79fc395d63855b3d1194d08d...HEAD -- ':(exclude)docs/designs'`.
Inputs used: that diff, `02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md`,
the runner's gate record (green at 2026-11-15T18:43:11Z), no prior findings.

## Verdict

request-changes — one open major (undeclared-looking scope creep in the stage
prompts). Everything else on the correctness checklist is clean.

## Checklist pass

1. **Components and interfaces (¶1).** `tools/orchestration/src/pipeline/ledger.ts`
   exports `parseLedger`, `extractHandles`, `checkLedgers`, `ledgerGate` and
   `MAX_LEDGER_FINDINGS` (ledger.ts:81,143,175,395,497); `gates.ts` now exports
   `frontmatterOf` with the BOM dropped and `parseSource` calls it
   (gates.ts, diff hunk at the old inline regex). Paths built with `node:path`
   (`join`/`basename`), read-only, no unprompted path parsing. The gate returns
   the shared `GateResult` via `allow`/`deny`. `ledgerGate(repoDir)` reason is
   `file:line code detail` lines bounded at `MAX_LEDGER_FINDINGS` with a
   `+N more` line (ledger.ts:502–508), as ¶7 requires.
2. **Hook points (¶7, A9, A10).** Entry: `runPipeline` calls
   `ledgerGate(opts.repoDir)` immediately after the `sourceAdmissible` deny
   check and before `drive()`, gate-logging `ledger-join`, denying into
   `finalizeEscalate` (runner.ts, diff hunk ~2442). Finalize: `finalizeDone`
   re-checks before the `flipAdrStatus` block (runner.ts:3606 follows the hook),
   denying into `finalizeEscalate` — the A10 posture, and test #12's green legs
   in the gate record confirm both directions.
3. **Grammar fidelity.** The anchored id regex is verbatim ¶2
   (ledger.ts:88, A15 look-behind at the Node floor). The trailing-span handle
   rule (ledger.ts:98) matches ¶4 exactly: last span only, single-space
   separators, at most one cell `|`, fence toggle on trimmed ```` ``` ````/`~~~`
   starts, frontmatter skipped, `\r?\n` split (A13). Pointer = last `→` outside
   a code span followed by non-space (ledger.ts:284–288), which is precisely the
   shape that keeps `021-C1`'s quoted-glyph case red pre-run and green post-run.
   The three owner forms with backticks stripped (ledger.ts:295–306) match ¶2
   and #5's pins. The unassigned-reservation exemption for `unit-unhandled`
   (ledger.ts:459–461) is 021-L2's single exception. The registry spans every
   ledger-bearing ADR (¶6), which is what 013's cross-attribution test pins.
   `depends-order` runs on every `NNN-*.md` carrying the key, ledger-bearing or
   not (ledger.ts:411–415), with the line number of the `depends-on` entry.
4. **Assumption register.** No row is silently changed: the corpus walk is
   `docs/adrs` + `schemas` only, `docs/designs` excluded (ledger.ts:395–399,
   header comment ¶5); `ledger: yes` is the only trigger (¶6); the template
   matches no `NNN-` and never parses as a ledger; fixtures live in
   `tmp-dir.ts` scratch, no committed markdown for prettier (Rollout 1).
5. **Tests.** #1–#8 (`tests/pipeline/ledger.test.ts`), #9/#11/#12
   (`tests/pipeline/ledger-join-gates.test.ts`), #10 (`ledger-corpus.test.ts`)
   all present and named per A21. The gate record shows `npm run check` green
   at 2026-11-15T18:43Z, which is the #10 standing-leg evidence and #12's
   021-R2 machine-verified acceptance.
6. **Error paths.** Denials at both hooks fail closed into escalation before
   any stage window opens and before the status flip; tests pin
   "no window opens" and "status stays proposed."

## Findings

### major — `tools/orchestration/prompts/test-authoring.md:8`, `tools/orchestration/prompts/test-update.md:8`

The diff rewrites both stage prompts from "format **only those files**" to
"format **the files you touched**", widening what the test stages' formatter
pass may rewrite in every future run. `02-proposal.md` never asks for this:
¶8's scope is template/021/013/spec.md, ¶1's is `ledger.ts` + the
`frontmatterOf` extraction, ¶7's is the two runner hooks, and Rollout 1
constrains the test commit to prettier-formatted `.ts` files only. Per the
correctness checklist (item 4), behavior in the diff the proposal never asked
for is a finding even if it looks useful. Fix: revert both prompt lines, or if
this was a fix-response the panel required, record it in `03-build-notes.md`
and the proposal's Rollout so the deviation is declared (checklist item 3).

## Not established with my inputs (non-blocking)

- Checklist items 3 and 5 need `03-build-notes.md`, which is under the
  caller-excluded `docs/designs/`; I could not verify whether the prompt edit
  above is declared there, nor audit build-notes claims. The code itself
  matches the proposal everywhere else, so this does not block.

## FYI (no severity)

- `docs/workflows.md`'s `ledger-join` paragraph (Rollout 5) is not in this
  diff — expected, it belongs to the later `doc-sync` stage.
