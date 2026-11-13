# Code review — compatibility (round 1, pass c0)

Scope: `git diff 60faf8c24438ed55121757057b578182d2815083...HEAD -- ':(exclude)docs/designs'`.
Inputs: that diff, `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`, the
compatibility checklist. No shell was available in this review environment, so
the diff was read from the tree at HEAD and the suite could not be executed
(finding 4).

Verdict: **approve** — 0 blocker, 0 major, 4 minor.

## Checklist coverage

1. **Platform code matches the Compatibility matrix** — pass, with finding 1.
   Every "yes" row has a code path: brief + `flow: light` (`gates.ts:110-113`
   judges the frontmatter-stripped body), ADR + `flow: light` (`gates.ts:115-124`
   keeps the ADR shape rules and `flowDef` validates the flow),
   CRLF (`gates.ts:56,59` — `[\s\S]*?\r?\n---` plus `[^\r\n]*` reads `light`
   from a CRLF block), bad value pre-lock (`runner.ts:2081-2091`, `flows.ts:97-103`),
   `--stage` vs. flow at both entry points (`runner.ts:2113,2173`), legacy
   checkpoint as `full` (`flows.ts:87-89`, `state-store.ts:70`,
   `fixtures/checkpoints/light-era.json`), `--max-parallel` last
   (`flows.ts:116`, no partial sets it). `src/` adds no `process.platform`
   guard and no OS-specific branch; the only `runIf(process.platform …)` blocks
   in the suite (`tests/pipeline/moves.test.ts:754,766`) cite run 005's A10 and
   predate this change, so the matrix's "no test runs there" row holds. All
   agents the light path launches exist: `design/planner`,
   `codereview/code-correctness`, `codereview/code-testing`; `prompts/plan.md`,
   `plan-note/SKILL.md` and `plan-note/references/template.md` exist.
2. **Node floor** — pass. `engines.node` is `>=22.23.4`
   (`tools/orchestration/package.json:30-32`) and CI pins exactly that
   (`.github/workflows/check.yml:44-48`). The new code uses nothing above the
   floor: `Number.isInteger`, `Array.prototype.at`, `Object.keys`, spread,
   `readdirSync`/`readFileSync` with no options object. No `fs.globSync`,
   `util.styleText`, `Promise.withResolvers`, `Array.fromAsync`, `node:sqlite`,
   `scheduler`, `crypto.hash`.
3. **Paths** — pass. New path construction is all `join`
   (`identity.ts:75-77,94,105,127`, `runner.ts:175`, every new test file), and
   the new `flow` parameter parses no path: `gates.ts:370-372` refuses `.`/`..`
   segments in both flows, `moves.ts:110-131` rejects backslash, colon,
   absolute and control-character shapes. Case sensitivity of the ownership
   regexes is unchanged and pinned by Test #7; `runner.ts:182` and
   `identity.ts:57` keep normalizing to POSIX separators.
4. **Guarded platform imports** — pass. No platform-only module is imported
   anywhere in the new code.
5. **Locale/layout reality** — finding 2. Run ids stay ASCII
   (`identity.ts:37-44`), checkpoints are read/written UTF-8, and git's `-z`
   intake (`runner.ts:226-229,222`) never quotes non-ASCII paths, so a
   non-ASCII document path survives the ownership sweep. The new flow line's
   `→` (`flows.ts:63,74`) lands in UTF-8 output on the supported platform and
   is the string the proposal mandates.
6. **Native dependencies** — pass. No new dependency in
   `tools/orchestration/package.json:33-49` (runtime: `@opencode/client`,
   `yargs`, `zod`), no new CI job, no toolchain change.
7. **Tests run where they claim** — findings 3 and 4. The new light suites
   (`flows`, `light-budget`, `light-flow`, `light-gates`, `light-ownership`,
   `light-parallel`, `light-plan-size`, `light-promotion`) carry no platform
   guard, so none is skipped-everywhere on linux, the only CI runner.

## Findings

### 1. minor — `docs/workflows.md:235,251` document a `stageTimeoutMs` the change does not ship

`docs/workflows.md:235` (per-flow budget table, `full` column) and
`docs/workflows.md:251` (defaults list, naming `src/pipeline/stages.ts` as the
authority) both state `stageTimeoutMs: 30 min`. The diff's registry ships
`stageTimeoutMs: 90 * 60_000` (`stages.ts:108`). The same table was updated for
the sibling knob in this range — `maxPassesPerRound` reads `8` at lines 232 and
243, matching `stages.ts:105` — so the timeout row is the one the doc pass
missed, not a deliberate carve-out. `02-proposal.md`'s budget table and the
matrix row "Existing ADRs and briefs … budgets unchanged" both describe the
30-minute full flow, so an operator sizing a full run from `workflows.md`
computes a stage bound three times smaller than the runner applies. The
`light` row (12 min, `flows.ts:79`) is correct, so Rollout 6's 96-minute light
bound is unaffected.

Fix: write `90 min` in `docs/workflows.md:235` and `:251`, and note there that
`light` overrides it to 12 min through `configFor`; carry the same correction
into the run's record so A25's "12 minutes is enough" is compared against the
shipped full default, not the 30-minute one.

### 2. minor — `src/pipeline/gates.ts:56,59` (`runner.ts:2083`) — a UTF-8 BOM silently reads `flow: full`

The frontmatter scan is anchored `^---`, so a byte-order mark at byte 0 makes
`front` null. `parseSource` then returns `status: 'none'` and
`flow: 'full'` (`gates.ts:57,73`), and the pre-lock check at `runner.ts:2083-2091`
sees the same `full` the scan invented, so it passes. For an ADR that is
visible (`status: "none"` denial); for a **brief** the body is non-empty, so
`sourceAdmissible` allows it (`gates.ts:110-113`) and a document that says
`flow: light` runs the twelve-stage, five-seat path. That is the exact failure
the design rules out — matrix row "`flow: Light` / `flow: sideways` … no:
pre-lock usage error", Test strategy #6 "never silently `full`" — reached
through an encoding spelling the matrix never considered: a BOM is what
Windows editors and `iconv -t utf-8-bom` write, and the matrix's CRLF row shows
the scan was designed for exactly that class of authored file.

Fix: strip the mark once at the parse seam — make the first line of
`parseSource` strip a leading `U+FEFF` from `content` and scan that value — which also
fixes `status:` for BOM-authored ADRs, and add one case to
`tests/pipeline/light-gates.test.ts`: a `flow: light` brief with a leading BOM
resolves as `light`, not `full`.

### 3. minor — `src/pipeline/runner.ts:3086` — the full flow's status label changed `run:` → `slug:`

`statusLines` now prints `slug:    ${nnn}-${slug}`. The proposal's Test
strategy #10 requires that "for a full state, every existing line is
byte-equal", and the matrix's rows for existing checkpoints, ADRs and briefs
claim today's behavior; this line is in the full flow's output, so a full run's
`pipeline status` header changes in a run whose flow is not selected. Deviation
5 records the rename and notes nothing reads the old label — that is the
problem: no test asserts the label set (nothing in `tests/` matches `run:` or
`slug:`), so the byte-equality #10 promises is unasserted and a future rename
of the same shape passes. The flow line itself (`runner.ts:3082-3084`) is the
additive part the design asked for; the rename is not.

Fix: print the identity under the label the full flow already used, keeping the
flow line additive, and assert the exact full-flow status block (or at least
its label column) in the `statusLines` test, which is what #10 asks for and
would have caught this.

### 4. minor — Test strategy #21's "`npm run check` stays green" could not be evaluated from the tree

Checklist item 7 asks that the suite run on the platform it claims. The
platform half is verifiable and true: `.github/workflows/check.yml:35-54` runs
`npm run check` for `tools/orchestration` on `ubuntu-latest` with
`node-version: "22.23.4"`, exactly the `engines` floor, and the change adds no
`runIf`/`skipIf` on any new test. The green half is not verifiable from a tree
read: this review had no shell, so the `lint`, `format:check` and vitest legs
of `tools/orchestration/package.json:23` were not executed. The named
failures recorded mid-run look closed at HEAD — the seven `light-*.test.ts`
files end at their final `});` with no trailing blank line, and
`gates.test.ts:738-751` is in prettier's call/array shape — so the claim is
plausible, but the matrix's linux row and #21 are not evidenced by the change.
This is recorded as a finding because the item could not be evaluated, not
because it reads as broken.

Fix: run `npm run check` in `tools/orchestration` at Node 22.23.4 on linux and
record the exit status (and the vitest count) in the run's build notes, so the
matrix row is backed by a datum; the CI job is the standing proof afterwards.

## Out-of-specialty FYI (no severity)

- `runner.ts:2616-2633`: the plan-size stop measures `size?.docLines ?? 0`, so
  a `plan` stage that produces no `02-proposal.md` measures 0 lines, passes the
  cap, and the run goes on to `implement`, whose prompt names a document that
  does not exist. The full flow's analogue (`runner.ts:2605-2607`) throws and
  escalates.
- `stages.ts:181-183` excludes `plan` from `STAGE_NAMES`, so
  `assertForceableStage` (`stages.ts:194-201`) rejects `--stage plan` before
  `assertStageInFlow` can accept it: a light run cannot be forced back to its
  plan stage, while the proposal's `--stage`-vs-flow rule reads as if it can.
- `identity.ts:93-115` reads every `docs/designs/*/.pipeline.json` on every run
  entry, pre-lock; bounded by the design-directory count, and joins/case are
  correct.
