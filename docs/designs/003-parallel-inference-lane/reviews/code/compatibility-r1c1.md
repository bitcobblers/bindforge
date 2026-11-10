# Code review — compatibility — 003-parallel-inference-lane, round 1 dissent pass (c1)

Verdict: **approve**

Inputs: the branch change (diff range `91dfbea..HEAD` minus `docs/designs/`; reconstructed by
reading the current tree, as shell access was refused in this seat — every r1c0 line reference
was re-located in the current files, so each of my six findings is checked against the exact
fix code), `02-proposal.md`, `docs/adrs/003-parallel-inference-lane.md`, and my own r1c0
findings. No other review report or transcript was read.

## Re-check of my r1c0 findings

### 1. major — operator-facing compatibility story for the knob absent — **resolved**
- `docs/workflows.md:87-95` now reads "A pass runs up to N seats at a time (`--max-parallel`,
  default 1 — the serial lane). Launch order is fixed … one commit lands per pass … Keep N at
  or below the inference slots your server actually exposes: the runner cannot verify the slot
  count, an oversized N queues seats, and each seat's timeout starts at its own prompt send and
  counts that queue waiting." — the rewritten sentence plus the `N ≤ slots` and
  per-seat-deadline/queueing notes, as Rollout step 3 and the Chosen design require.
- `docs/workflows.md:104-110`: the CLI block gains `pipeline run <source> --max-parallel 3`,
  the `$OPENCODE_MAX_PARALLEL` fallback, the default 1, and "a malformed value in either source
  fails the command before any inference is spent" — matching `resolveMaxParallel`
  (`src/pipeline/cli.ts:66-101`) and its tests (`tests/pipeline-cli-max-parallel.test.ts`,
  strategy #8). `docs/workflows.md:143` lists `maxParallelReviews: 1` in the Tuning set, which
  matches `stages.ts:83`.
- The "Proposed, not implemented" ADR-003 entry is gone: the string no longer occurs anywhere in
  `docs/workflows.md` (the section is absent from the current 173-line file).
- `tools/orchestration/README.md:24-28` took the second branch of my fix: the flag table is
  scoped to the connection-test CLI, and the README states the pipeline CLI's knobs — naming
  `--max-parallel` / `$OPENCODE_MAX_PARALLEL` — are documented with their operational guidance
  in `docs/workflows.md`. The matrix row "Run with N > 1 — yes, conditional on the server
  exposing ≥ N slots" now has a discoverable support story.

### 2. minor — hard-coded `node_modules/.bin/prettier` resolution + silent skip + Windows `.cmd` — **resolved by removal**
`formatTouchedFiles` is gone. `src/pipeline/runner.ts:267-285` now states the replacement
posture: the runner never executes a formatter (or anything else) out of a model-writable,
git-ignored worktree path, and the comment preserves the rationale. Hygiene routes through
ownership instead: a red `npm run check` (lint + `format:check` + tests, `package.json:23`)
summons `fix`, `prompts/fix-tests.md:6-12` tells the fixer to run `npm run format` (a committed
`package.json` script — not a worktree-path binary) and to set `testsNeedUpdate` for test-owned
files, which routes to `test-update` (`runner.ts:1732-1736`, `states.ts:168-178`). The OS-specific
`.cmd` concern disappears with the step. `docs/workflows.md:75-85` carries the same story for
operators, and `tests/pipeline-fix-trigger.test.ts:15-20` records why the formatter-stub block
was deleted.

### 3. minor — deleted paths and glob metacharacters fed to `prettier --write` — **resolved by removal**
No `prettier` spawn remains anywhere under `src/` (grep). `dirtyFiles`/`porcelainPaths` are used
only for ownership accounting and the commit-is-empty check, where deleted paths and unusual
names are handled by git NUL plumbing, not by a formatter CLI. The latent failure mode cannot
return.

### 4. minor — gate-label value vocabulary changed without a record — **resolved (kept + documented), one nit left below**
The rename stands — `states.ts:34-38` — but is now recorded operator-side exactly as my fix
offered: `docs/workflows.md:116-119` ("The aggregated review gates log under `design-review` /
`editorial-review` / `code-review`; a run checkpointed before those labels landed keeps its
historical `all*ReviewsApproved` entries, so one log can show both vocabularies — nothing keys
on the old names."). I verified the last clause in the code: `gateLog` names are written
(`runner.ts:1144-1161`, `1622-1629`, `1693-1705`, `1546`) and only ever *displayed*
(`runner.ts:2033-2035`, `report.ts:225,280`); no control-flow reader compares gate-log names —
`reviewPhaseOf` (`states.ts:45-50`) keys on `--stage` targets, not the gate log — and
`gate: z.string()` (`state-store.ts:103`) parses old entries unchanged, so the matrix row
"checkpoints written before this change — yes — no checkpoint schema change" stays true. The
old-name residue is confined to `docs/adrs/001` and is reported as a nit below.

### 5. minor — `updatedAt` lost its writer — **resolved (variant shape, functionally equivalent to the fix asked)**
`saveState` deliberately stays un-stamped with the reasoning written at the definition
(`state-store.ts:159-174`); the stamp moved to *both snapshot points*: `checkpoint()` stamps
`state.updatedAt = Date.now()` immediately before every sequential write
(`runner.ts:417-431`), and `runReviewPass` stamps inside `enqueueSnapshot` right before each
`writer.enqueue` (`runner.ts:1208-1216`) — with no await between stamp and the writer's
synchronous `structuredClone` (`state-store.ts:212-216`), so enqueue order and timestamp order
are the same ordering, which is the property the writer ordering needs. I verified every
`saveState` call site in `runner.ts` goes through `checkpoint()` (single call, line 430) and
every in-pass write goes through `enqueueSnapshot`; `freshState` still seeds both fields
(`runner.ts:390-414`). The fix's *intent* — "when this snapshot was taken" on both paths, so a
checkpoint after this change is never frozen at `startedAt` — is met; the fix's exact location
(stamp inside `enqueue`) was one option, and the writer-purity variant is the cleaner choice
because it keeps the writer a pure queue over caller bytes. Tests: the writer suite now asserts
the *ordering* semantics the design claims (`pipeline-checkpoint-writer.test.ts:52-84`,
including "first snapshot authoritative until the next lands"), and the end-to-end test asserts
a real pass stamps a live time — `loaded.updatedAt > startedAt` over the run's checkpoint on
disk (`pipeline-parallel-integration.test.ts:230-234`). Hand-set fixture values remain in the
writer unit test, but there they are the *input being preserved*, which is precisely what that
test must pin.

### 6. nit — `relative()` separator in the package-prefix matcher — **resolved by removal**
The matcher went with `formatTouchedFiles`; no `relative()`-derived prefix comparison against
git porcelain paths remains. No OS-specific code exists in the new source (`src/` grep finds no
`process.platform`, `os.platform`, or path-separator special-casing; `independence.ts:164`
normalizes `\` to `/` for *matching text*, which is platform-neutral), so the matrix's
"no OS-specific code added" note is now accurate on its face.

## Compatibility checklist against the current change

1. **Platform code vs matrix** — pass. No OS-specific code; macOS/Windows rows say `no`;
   Linux-only CI claim holds (all new tests are mock-client + temp-git-repo:
   `pipeline-parallel-{fanout,escalation,integration}`, `pipeline-concurrency`,
   `pipeline-checkpoint-writer`, `pipeline-pass-finalize`, `pipeline-cli-max-parallel`).
2. **Node floor (`>= 22.12.0`, `package.json:30-32`)** — pass. New primitives are
   `structuredClone`, `Promise.allSettled`, plain promises/`??=` — all far below the floor.
3. **Paths** — pass. All new path handling is `join`/`resolve` (`state-store.ts:13,186`,
   `runner.ts:1205/1931-1943`); no concatenation. The writer's tmp name is
   `${target}.${4-byte-hex}.tmp`, excluded from the design dir by the drain-before-finalize
   barrier (tested, `pipeline-checkpoint-writer.test.ts:89-100`).
4. **Guarded platform imports** — pass; no new platform-only module.
5. **Locale/layout reality** — pass. `printable()` (`independence.ts:200-202`) strips only
   `\p{Cc}` (control/escape), so non-ASCII report text, findings, and paths survive the gate
   log, seat board, and checkpoint; git plumbing uses `-z` NUL separation (`runner.ts:159-182`).
6. **Native dependencies** — pass; nothing added (`concurrency.ts` is dependency-free;
   `package.json` deps unchanged).
7. **Tests run where claimed** — pass; no skips or platform guards in `tests/`; the suite runs
   under `vitest run` inside `npm run check` on Linux CI (strategy #10).

## Open items

| # | Severity | Location | Problem | Fix |
|---|---|---|---|---|
| 1 | nit | docs/adrs/001-code-driven-agent-pipeline.md:121,190,197 | The gate-label rename stands, and `docs/workflows.md:116-119` records it — but the document `workflows.md:3` calls normative still names the aggregated-verdict gates `allDesignReviewsApproved`/`allCodeReviewsApproved` in its gate table, with no pointer to the shipped labels or to the coexistence rule. This is the residual half of my r1c0 finding 4's "record the labels" fix; an operator consulting ADR-001 to key a script on a gate-log name is pointed at a name this code stopped writing. | One line in ADR-001's gate table (or an amendment note in the style ADR-003 uses for ADR-001) recording that the runner logs these gates under `<phase>-review` since ADR-003 — or an explicit statement that ADR-001's table is the historical record and `docs/workflows.md` is the live label authority. |

FYI (no severity, outside this seat's checklist): `docs/workflows.md` no longer has any
"Proposed, not implemented" section; ADR-004 (frontmatter `status: proposed`, not implemented)
therefore has no pointer from the practical guide — its status now lives only in its own
frontmatter. Also note for the record: this seat's shell access was refused, so the diff range
was reconstructed from the current tree plus my r1c0 evidence; every r1c0 line reference was
located and re-checked in the current files.

## Dissent-pass conclusion

All six r1c0 findings are closed: the docs item by landing the knob story end-to-end; the three
formatter items and the separator nit by removing the runner-executed formatter wholesale (which
also hardened the security posture the step had created); the `updatedAt` item by stamping at
both snapshot points with the semantics documented at each site and pinned by an end-to-end
test. Nothing in the revised code opens a new compatibility issue at blocker/major level.
**Verdict: approve**, with one nit.
