# complexity review r1 (ratification pass c2)

verdict: approve

Nothing in `15ba04f..0b13556` added a moving part, a dependency or an
abstraction that was not there when I approved: the two substantive edits —
`git diff -z --no-renames --name-only` through the existing `nulPaths` parser
(§1) and the `printable` / bounding sentence (§2) — both reuse in-repo helpers
and are complexity-positive. One defect comes in with the r2 bounding closure
(finding 3). Value check: the design's mechanism count (one predicate, one net
diff per window, one record, one finalize, one exit code) looks proportionate to
what ADR-020 buys; the only weight I still object to is Test #7's import-graph
walk, which is larger than the rule it guards.

## Findings

- [minor][re-raised] Chosen design §1 — "`finalizeStage` and
  `finalizeReviewPass` (runner.ts:1848–1892, 1940–1961) already receive the
  window's `headBefore` and mint its commit" — the r2 edit rewrote this
  sentence (added `-z --no-renames`, named `nulPaths`) and left both hooks in
  place, so the r1 problem survives verbatim: `finalizeReviewPass` is called
  only at pass level (runner.ts:2184, 2187) and on the single-stage pre-commit
  refusal with `commit: false` (runner.ts:1924); the only `preCommit` in the
  tree is the plan stage's (runner.ts:2816); `stagePathAllowed` confines every
  non-`implement`/`fix` stage to `docs/**` (gates.ts:559–589). Under §1's own
  "only `implement`/`fix` can commit under the two prefixes", the second hook
  can never fire, and Test #3's "a `finalizeStage` and a `finalizeReviewPass`
  window whose net diff touches a template or a src file set the record" is
  only satisfiable by fabricating a state the ownership fence forbids. Fix:
  instrument `finalizeStage` alone and say so in the sentence that already
  states the coder-window rule; drop the review-pass half of Test #3. If the
  hook is kept deliberately as defence against a future ownership-matrix
  change, add one clause saying that and mark the Test #3 review-pass case a
  rule-bypass fixture, not a live window. Same red-on-drift property either
  way; one unreachable call site and one impossible test case cheaper.

- [minor][re-raised] Test strategy #7 — "walking the import graph from the
  pipeline entry points (`src/pipeline/cli.ts`, `src/cli.ts`,
  `src/opencode-runner.ts`), every in-repo module reached is under
  `tools/orchestration/src/`" — r2 deleted §1's description of this mechanism
  ("The ADR's \"test that fails when a new executable path is added\" is Test
  strategy #7.") but left the mechanism itself, which is still the largest new
  thing in the design and still has no provider: `tools/orchestration/package.json`
  devDeps are typescript, vitest, eslint, prettier, tsx and the two plugins —
  nothing that resolves an import graph — so it means hand-written specifier
  parsing plus an entry-root resolver, and it still misses any module none of
  the three roots imports. The rule it guards is directory-scoped by §1, so
  the walk buys nothing over a flat check. Fix: replace the walk with a
  package-wide enumeration — every tracked `.ts`/`.js` under
  `tools/orchestration/` that is not under `tests/` must sit under `src/` —
  plus the `PROMPTS_DIR` root assertion the test already names (cli.ts:20).
  The `readdirSync`-based helpers the `tests/pipeline/` suites already use
  cover it: no new dependency, ~15 lines instead of a resolver, identical
  "new executable location turns the suite red" property.

- [minor][new] Chosen design §2 — "the record keeps the `boundedGateEvidence`
  bounding (first N paths plus the count)" — this r2 clause imports a bound
  the design's own record shape cannot hold. `boundedGateEvidence`
  (runner.ts:3057) is typed `GateEvidence → GateEvidence`, and its "first N
  plus the count" works only because an elided leg is a whole extra leg object
  with a `command`/`tail` to put the count in; `restartRequired?: { stage,
  paths, at }` (§2) has no slot for a count, and the only bound named in the
  tree is `MAX_GATE_LEGS` (gates.ts:310), a gate-leg cap, not a path cap. So as
  written the implementer either adds a fourth checkpoint field for the count
  or drops it silently — and dropping it makes §3's status line, "how many
  harness files it changed", print the capped length rather than the number of
  changed files. Fix: bound where it renders, not what is recorded — keep
  `paths` whole and say the gate-log line, commit body and summary are
  `brief`-bounded (`brief` is the existing one-line helper, independence.ts:448,
  right beside the `printable` the sentence already names, independence.ts:443),
  with `paths.length` as the count §3 prints. If the record itself is meant to
  be capped instead, name the constant and add `total` to the §2 shape so §3's
  count stays honest.

- [nit][re-raised] Chosen design §2 — "mints a terminal `chore: <nnn>-<slug>
  restart boundary` commit" — unaddressed by r2 and not recorded as rejected in
  the Revision log. `boundary` is already loaded in this file's own vocabulary:
  the lock records a stage named `boundary` (runner.ts:606), and "boundary
  commits", "the boundary re-assertion" and "at the next boundary" name the
  ADR-006 stage boundary (runner.ts:328, 571, 2376). §1's "at the top of
  `drive`'s `for (;;)` body" and §2's "gate-logs the boundary it is crossing"
  both lean on that second meaning in the same breath as ADR-020's pause. Fix:
  one clause in §1's heading or §2 — "'restart boundary' is ADR-020's pause;
  the runner's existing 'boundary' is the ADR-006 stage boundary" — and keep
  every new identifier and label qualified (`restartRequired`,
  `finalizeRestartBoundary`, `restart-boundary`), never a bare `boundary`.

## FYI (no severity)

- A7's rewrite is a factual correction, and the correct one: `loadState`
  returns `parsed as PipelineState` after only *validating* a separate parse
  (state-store.ts:239, 256) and `saveState` writes
  `JSON.stringify(state, null, 2)` (state-store.ts:365), so unknown keys are
  never stripped at either end and do round-trip. The old "stripped" wording
  was wrong; the Options argument against a `RunStatus` member still stands,
  because refusal comes from `safeParse` failing (state-store.ts:240–244), not
  from key handling. No complexity change — no migration ladder is implied by
  the new wording.
- §1's `git diff -z --no-renames --name-only` through `nulPaths` is real reuse,
  and the smallest available option: `nulPaths` exists at runner.ts:262 and is
  module-private, and both hooks and the diff call live in `runner.ts`, so
  nothing needs exporting; the tree has no range-level path-listing helper
  (the only one is per-commit `diff-tree` inside `enforceOwnership`,
  runner.ts:308).
- §2 stores `printable` copies in the record, so the persisted `paths` are not
  byte-exact git paths once a path carries a control byte. Harmless for the
  commit body, status line and summary; if a later round ever acts on recorded
  paths, that lossy copy is the thing to notice. Not my seat.
