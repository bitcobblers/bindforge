# Code review — security seat, round 1, pass c0

**Range:** `60faf8c24438ed55121757057b578182d2815083...HEAD -- ':(exclude)docs/designs'` (38 files,
+4298/−167)
**Judged against:** `02-proposal.md` (ADR-010 light flow), `docs/adrs/010-lightweight-plan-flow.md`
**Checklist:** `references/security.md` (7 items) — all seven evaluated, none unevaluated.
**Previous findings from this seat:** none (first pass).

## Verdict: request-changes (2 major, 3 minor, 2 nit)

## Checklist coverage

| # | Item | Result |
| --- | --- | --- |
| 1 | External input validated at the boundary | Pass with gaps — the frontmatter `flow:` value is validated pre-lock (`runner.ts:2078-2091`), a checkpoint's `flow` is schema-validated, not coerced (`state-store.ts:67`), and a caller's config panel is validated pre-lock (`flows.ts:113-124`, asserted `light-gates.test.ts:221-238`). Gaps: findings 3, 4. |
| 2 | Path handling | Pass — the reserved-segment rule stays first and wins in both flows (`gates.ts:396`, asserted for every stage class in both flows at `gates.test.ts:647-655`), the new `.`/`..` refusal (`gates.ts:370,399`) is asserted (`gates.test.ts:665-666`), paths reach the predicate only as git plumbing output, and a light `plan` write spelled `docs/../src/x.ts` is judged at git's real path (`light-ownership.test.ts:114-125`). |
| 3 | Injection | Pass — every git call is `spawnSync`/`execFile` with an argument array (`identity.ts`, `moves.ts`, `runner.ts:160`); no user string enters a shell. The one new regex (`/^flow:[ \t]*([^\r\n]*)/m`, `gates.ts:41`) is linear; `TEST_PATH_RE` is unchanged; no recursive re-render in `renderTemplate` (`prompts.ts:47-57`). |
| 4 | Secrets and PII | Pass — no credential, token or personal path in the diff; `fixtures/checkpoints/light-era.json` uses synthetic blob ids and session ids. |
| 5 | New dependencies | Pass — the diff touches no manifest or lockfile; the frontmatter scan stays regex-based (no YAML dependency), matching A10. |
| 6 | Filesystem writes, least privilege | Pass — no new write site; the pinned template load (`runner.ts:2279-2283`) is read-only, the pre-lock gates write nothing (asserted), and escalation goes through the existing atomic `finalizeEscalate`/`saveState`. The template-pinning fix removes a real injection vector: a stage that edits its own prompt no longer has that edit rendered into the retry attempt. |
| 7 | Clean rejection on malformed/hostile input | Pass with gaps — the pipeline CLI prints `error.message`, not a stack (`pipeline/cli.ts:239-242`); the pre-lock gates leave no lock, design directory, checkpoint, marker or client call (`light-gates.test.ts:157-238`); an unhandled throw now ends with a record instead of a split record (`runner.ts:2665-2682`). Gaps: findings 3, 4. |

The widening itself is sound: `stagePathAllowed` keys it on the exact string `'light'`, fails
closed to the narrow rules when the parameter is omitted (`gates.ts:389`), validates the flow
before any rule so an unknown flow cannot fall through to `full` (`gates.ts:392-394`), and the
light branch admits only paths under a real `tests/` directory (`gates.ts:405-411`) —
`src/foo.test.ts` stays denied. The sweep reads the flow from the checkpoint
(`runner.ts:280`), not from a flag, and `moves.ts` gates a move whose stage the flow never runs
(`moves.ts:157-162`).

## Findings

### 1. Major — the full flow's cost ceilings were raised outside the design, and the doc now contradicts the code
**Location:** `tools/orchestration/src/pipeline/stages.ts:108`, `tools/orchestration/src/pipeline/stages.ts:105`

`stageTimeoutMs` went `30 * 60_000` → `90 * 60_000` and `maxPassesPerRound` 6 → 8 for `full`.
The design's Non-goals say "No change to the full flow's stages, seats, budgets", and ADR-010's
table pins `full` at 30 min / 6. `docs/workflows.md` — updated in this same diff — records
`maxPassesPerRound: 8` (lines 232, 243) but still states `stageTimeoutMs: 30 min` at lines 235
and 251, so the operator-facing budget table and the code disagree about the one knob the doc
itself calls load-bearing: "stage turns are bounded only by this timeout, there is no
model-turn/step cap" (line 251-252; `stages.ts:6-7` says the same). With two attempts per stage
(`runner.ts:2284`, `runner.ts:2320`), the per-stage wall-clock ceiling goes from 1 h to 3 h and
the pass budget per round rises by a third, for **every** existing run, not just light ones.
No test pins either value: `flows.test.ts:118-133` asserts `configFor('full', base)` equals a
literal `base` (tautological against `DEFAULT_CONFIG`), and the only timeout assertion is the
light 12-min pin at `flows.test.ts:92`.

**Fix:** restore `30 * 60_000` and `6`, or move the two knob changes out of this design's scope
with an amendment to the proposal/ADR that carries the run-010 evidence, update
`docs/workflows.md:235,251` to match the code, and pin the full-flow values with an assertion
against `DEFAULT_CONFIG` so the next silent edit is a test failure.

### 2. Major — a light run's `--extend-budget` grant is never consumed, so the flow's budget-exhaustion stop stops binding
**Location:** `tools/orchestration/src/pipeline/runner.ts:2506-2513` with `tools/orchestration/src/pipeline/panel.ts:126-131`

At the over-cap pass entry, `if (!isLight) state.extensionSessions = left - 1;` leaves
`state.extensionSessions` intact for a light run, and `planPass` lifts the pass cap by one "while
a grant is held". The grant therefore never depletes: every pass entry above
`maxReviews.code: 6` is admitted for as long as the checkpoint holds any grant, bounded only by
`maxPassesPerRound + 1` per round and by `maxRounds: 2` across rounds — up to 16 code sessions
against a cap of 6. `light-budget.test.ts:280` pins 8 sessions for the round where the cap first
binds, so the multi-round case is untested. Consequences: one of ADR-010's two deterministic
stops ("budget exhaustion") is disabled by a single grant; `--extend-budget 5` behaves exactly
like `--extend-budget 1`; and the gate log prints a constant `1 session grant(s) left` on every
over-cap entry, so the audit record the operator resumes from overstates what remains. The
design's A16/A27 is explicit that the extension is "consumed at the pass-entry cap check as an
over-cap allowance".

**Fix:** consume the grant in both flows (drop the `!isLight` guard) and key the light
pass-cap lift on the grant that is left, then assert in `light-budget.test.ts` that a second
over-cap entry escalates and that the printed grant count decreases.

### 3. Minor — seat names are validated against the union of both panels, so a panel naming a seat no agent backs still launches
**Location:** `tools/orchestration/src/pipeline/flows.ts:164` with `tools/orchestration/src/pipeline/flows.ts:25`

`validateSeatNames` tests membership in `SEAT_NAMES` (design ∪ code). `complexity` is
design-only, so `reviewSeats.code: ['complexity']` passes pre-lock validation; the code panel
then resolves `codereview/code-complexity` (`stages.ts:166`) and `.opencode/agents/codereview/`
has no such agent (correctness, compatibility, security, performance, testing). The failure
surfaces mid-run as a failed stage attempt — "no model turns — agent … may not exist"
(`runner.ts:1435`) — after the lock, the entry checkpoint and an escalation record are live,
burning an attempt at `stageTimeoutMs`. Symmetrically, `testing` in the design panel resolves
`review/design-testing`. This is the failure the design's Budget resolution section names as the
reason the resolver validates at all ("`['corectness']` launches a seat no agent backs"), and
`flows.test.ts:214-225` catches only the typo shape. The same intake also reaches a
`TypeError: undefined is not iterable` when a JSON-shaped config omits one phase key
(`flows.ts:160` iterates `seats[phase]` unguarded) — an opaque message, not the usage error.

**Fix:** validate `reviewSeats.code` against `CODE_SPECIALTIES` and `reviewSeats.design` against
`DESIGN_SPECIALTIES` per phase, guard a missing/non-array phase key with the same usage-error
message, and add `flows.test.ts` cases for `code: ['complexity']`, `design: ['testing']` and a
one-key `reviewSeats`.

### 4. Minor — the plan-size stop fails open when the plan document cannot be measured
**Location:** `tools/orchestration/src/pipeline/runner.ts:2621`

`planOutgrewFlow(size?.docLines ?? 0, cfg)` reads "no measurement" as "within cap".
`proposalSize` returns `undefined` when `docs/designs/<run>/02-proposal.md` is absent
(`runner.ts:176-177`), so a light `plan` stage that writes nothing — or writes to any other name
— passes the flow's one deterministic plan stop, spends the `implement` session, and hands the
two seats a run with no design document, which is the artifact their faithfulness check and the
seat stamp are built from (`02-proposal.md`'s blob id, A5). A gate that exists to be a
deterministic stop must not be satisfied by the absence of its input.

**Fix:** deny when the measurement is missing — escalate `plan document not written:
<designDir>/02-proposal.md` when `proposalSize` returns `undefined` — and add the missing-file
and zero-line cases to `light-plan-size.test.ts`, which today covers only 79/80/81-line plans.

### 5. Minor — one panel reader drops the resolved config and re-derives the standing five seats
**Location:** `tools/orchestration/src/pipeline/states.ts:145`

`dissentingSeats(state, 'design')` omits the `cfg` argument, so it resolves through
`panel.ts:59-65`'s `DEFAULT_CONFIG` default — the standing five — while the caller
(`decideNext`) holds the resolved config and every sibling site threads it
(`states.ts:87`; `panel.ts:123-124`). No registry flow sets a design panel today
(`FLOWS.full.config` is empty), so the divergence is latent, but it is exactly the residual the
design's Risks row 4 records, and the design's Budget resolution requires all five panel readers
to take the `cfg` their callers already hold. With a caller-supplied `opts.cfg` design panel, the
dissenter list routed into the `proposal-revise` prompt is computed from a panel the run is not
using, so a dissent from a seat outside the standing five is dropped from the revision input
while the funnel keeps its record.

**Fix:** pass the config the caller has — `dissentingSeats(state, 'design', cfg)` — and assert in
`states.test.ts` that a caller-supplied design panel drives the dissenter list.

### 6. Nit — the source document is read and parsed twice per entry, against the design's one-read rule
**Location:** `tools/orchestration/src/pipeline/runner.ts:2079-2083` with `tools/orchestration/src/pipeline/gates.ts:96-101`

The design's Selecting the flow requires "one frontmatter read per entry, so the value that
cleared the pre-lock check is the value stamped and printed". The code passes the *flow value*
into the post-lock gate (so the stamped value is the pre-lock one — correct), but
`sourceAdmissible` re-reads and re-parses the file, so the Context/Decision/status verdict is
judged on a second read of a file that the first read already parsed.

**Fix:** thread the pre-lock `ParsedSource` (and its content) into `sourceAdmissible` as an
argument and delete the second read.

### 7. Nit — the pre-lock plan gate is unreachable in production, and it duplicates the line count
**Location:** `tools/orchestration/src/pipeline/gates.ts:88-110,131-136`

`sourceAdmissible(sourcePath, 'plan', flow)` is never called: `deriveIdentity` produces
`kind: 'adr' | 'brief'` only (`identity.ts:58`) and the runner passes `id.kind`
(`runner.ts:2142`), so the plan branch — its empty-document denial and its pre-lock line cap —
is reachable only from `gates.test.ts:761`. Net effect on the light path: nothing bounds the
**source** document's size (the source is operator-supplied, so this is not an exposure), and
`documentLines` (`gates.ts:131-136`) is a second implementation of the meter's count
(`runner.ts:181`) — the design's own "one authority" argument applies to it.

**Fix:** call the gate on the light entry path (kind `plan` when `flowOf(state) === 'light'`),
or delete the branch and reuse `proposalSize`'s count for the gate.

## FYI (out of specialty, no severity)

The coder-class write surface includes `tools/orchestration/prompts/**` and
`.opencode/agents/**` in both flows, since the sweep denies only test paths and reserved names;
the template-pinning change (`runner.ts:2270-2283`) removes the in-run consequence, and the
surface predates this diff.
