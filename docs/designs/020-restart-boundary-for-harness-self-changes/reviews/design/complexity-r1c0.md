# complexity review r1

verdict: approve

Scope: whole document (`docs/designs/020-restart-boundary-for-harness-self-changes/02-proposal.md`),
against `docs/adrs/020-restart-boundary-for-harness-self-changes.md`. First pass for this
seat: no earlier complexity findings, so nothing is re-raised or regressed. The Revision log
holds five r1 decisions (companion field, one net-diff check per window, exit 3, no new
provenance record, three deferrals); none of the findings below contradicts a recorded
rebuttal.

Basis for the code claims I checked: direct reads of every cited anchor —
`gates.ts:559–590` (`stagePathAllowed`), `runner.ts:294–377` (the sweep's git plumbing),
`runner.ts:1899–1928`/`1940–1947` (the two finalize windows and their `headBefore`),
`runner.ts:2440–2441` (loop top), `runner.ts:2345–2378` (resume split-record posture),
`runner.ts:3377–3425` (`finalizeDone` terminal save + save-before-commit),
`runner.ts:3580–3626` (`planStopStatusLines` in `statusLines`), `runner.ts:2816` (the only
`preCommit`), `runner.ts:1344` (the `tools/orchestration/prompts/...` provenance spelling),
`cli.ts:17`, `cli.ts:20`, `cli.ts:201–228`, `types.ts:248–261`, `types.ts:301`,
`tools/orchestration/package.json`, `tests/pipeline/fixtures/checkpoints/`. The npm and cargo
legs could not be run from this session (no installed `node_modules`, and some shell forms were
denied), so no claim below rests on a command result — each rests on the source above.

## Findings

- [minor][new] Chosen design §1 — "Detection is the window's **net** diff. `finalizeStage` and
  `finalizeReviewPass` (runner.ts:1848–1892, 1940–1961)" — the second hook is dead under the
  design's own A3, which §1 restates: "only `implement`/`fix` can commit under the two
  prefixes, so the boundary opens only after a coder window". `finalizeReviewPass` runs only
  for pass-level review stages (`runner.ts:2184–2187`) and for the single-stage pre-commit
  refusal path (`runner.ts:1924`), and the only `preCommit` in the tree is the `plan` stage's
  (`runner.ts:2816`) — all document stages, which `stagePathAllowed` confines to `docs/**`
  (gates.ts:588–589); on the refusal path no commit is minted (`commit: false`), so the
  `headBefore..HEAD` range is empty. So the hook can never fire, and Test #3's "a
  `finalizeStage` and a `finalizeReviewPass` window whose net diff touches a template or a src
  file set the record" can only be satisfied by fabricating a state the ownership fence
  forbids. Fix: instrument `finalizeStage` alone and say so in §1 in the sentence that already
  concludes the coder-window rule — one hook, because every other window is surface-free by
  A3 — and drop the review-pass half of Test #3. If the author wants the hook kept as defence
  against a future ownership-matrix change, §1 needs one clause saying that, and Test #3's
  review-pass case has to be labelled a rule-bypass fixture rather than a live window. Either
  way the same red-on-drift property is kept; the saving is one unreachable call site and one
  impossible test case.

- [minor][new] Test strategy #7 — "walking the import graph from the pipeline entry points
  (`src/pipeline/cli.ts`, `src/cli.ts`, `src/opencode-runner.ts`)" — the graph walk is the
  largest single new mechanism in the design, and `tools/orchestration` has no dependency that
  provides one (devDeps are typescript, vitest, eslint, prettier, tsx), so "walking the import
  graph" means hand-written specifier parsing plus an entry-root resolver, and it still misses
  any module none of the three roots imports. The rule it guards is directory-scoped, so the
  walk buys nothing over a flat check: every module under `tools/orchestration/src/` is already
  surface by §1, so the only drift that matters is a module file living *outside* `src/`, or a
  prompt read root outside `prompts/`. Fix: replace the walk with a package-wide enumeration —
  every tracked `.ts`/`.js` under `tools/orchestration/` that is not under `tests/` must sit
  under `src/`, plus the `PROMPTS_DIR` assertion §7 already makes (cli.ts:20, and the same
  spelling the provenance line already uses at runner.ts:1344). That is roughly fifteen lines
  with the `fs`/git helpers the pipeline tests already use, no new dependency, and the same
  "new executable location turns the suite red" property the ADR asks for.

- [nit][new] Chosen design §2 — "mints a terminal `chore: <nnn>-<slug> restart boundary`
  commit" — `boundary` is already a loaded word in the file the new code lands in: the lock
  records `stage: 'boundary'` (runner.ts:606), and "boundary commits", "the boundary
  re-assertion" and "at the next boundary" all name the ADR-006 *stage* boundary
  (runner.ts:328, 571, 2376). The ADR's term is fixed, so the concept keeps its name, but bare
  "boundary" in new comments, status text or identifiers will read as the wrong thing. Fix: one
  clause in §2 (or the §1 heading) — "'restart boundary' is ADR-020's pause; the runner's
  existing 'boundary' is the ADR-006 stage boundary" — and keep every new identifier and label
  qualified: `restartRequired`, `finalizeRestartBoundary`, `restart-boundary`, never a bare
  `boundary`.

## Value check (checklist 7)

Proportionate. The production change is about seventy lines over seven touchpoints — a predicate
beside `stagePathAllowed`, one net-diff call reusing the sweep's existing git plumbing
(runner.ts:302–308), one optional companion field in the shape `planStop` already sets
(types.ts:301), a finalize that mirrors `finalizeDone`'s terminal-save and save-before-commit
posture rather than inventing one (runner.ts:3401–3410), a three-line loop-top guard, a resume
clear mirroring the split-record posture (runner.ts:2345–2378), and a status line built like
`planStopStatusLines` (runner.ts:3580). No new package, no new interface with one implementation,
no configuration knob (`promptsDir` is not a CLI option — cli.ts:20/205/220 fix it), no new
dependency, and the checkpoint grammar is untouched: `version` stays 1, one additive optional
field, which is the cheapest move ADR-006 §2 allows. The record's `at` matches the repo's record
convention (types.ts:87, 174, 199, 375) rather than being an unused extra. Nothing pre-existing
in `tools/` covers this — no `restart` concept exists in `src/` — so the reuse checks come back
clean. On deferral (checklist 5): the design cannot be halved safely, because a pause that still
exits 2 is signed as an escalation, so §1–§3 must land together; §4 adds no mechanism at all.
The only real shrink candidates are the dead review-pass hook and the graph-walk test, both
named above; with them fixed the design is close to the smallest thing that satisfies ADR-020
Decisions 1–3.

## FYI (no severity)

- Compatibility matrix / Test strategy Platform basis is factually wrong: "the repo has no CI
  workflow (same basis as ADR-006's matrix)", the Windows row's "no CI workflow exists", and
  "no CI workflow exists in this repo, so the suite is run locally" are contradicted by
  `.github/workflows/check.yml`, which runs `npm run check` in `tools/orchestration` on
  `ubuntu-latest` with node-version `22.23.5`. The Linux and Windows rows' reasoning and the
  "not exercised in this run" posture both need re-deriving on the true basis (CI exists, but
  only on Linux). Also relevant to the drift-guard argument: Test #7 does have a PR-time
  enforcement point. Correctness/compatibility seats' call, not this one.
- §2's record `restartRequired = { stage, paths, at }` and §3's "the stage that opened the
  boundary" name the *finished* stage, while ADR-020 Decision 2 asks the marker to name "the
  stage that will run next". Which one the field holds should be stated once; §4's "the §2
  record already names the stage" inherits the ambiguity.
- The `chore:` prefix on the boundary commit differs from the two existing terminal commits,
  which use `docs:` (runner.ts:3422, 3563); it fits AGENTS.md's semantic-commit types, so this
  is only worth a deliberate word in §2 if consistency matters.
