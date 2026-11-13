# correctness review r1

verdict: approve

Verification performed (this seat): every `verified` assumption-register row was
checked against its cited basis by reading the cited code. All match:
A1 (prompts.ts:81–83 `loadTemplate` reads from disk; cli.ts:20 `PROMPTS_DIR`
resolves off the module URL; nodejs.org "Caching" fetched — supports the
module-cache claim, and the cited CJS page's eviction lever `require.cache` does
not apply to ESM, as A1 states); A2 (runner.ts:1848 finalizeStage, 1940
finalizeReviewPass, 292/363–377 enforceOwnership's revert commit, inside the
same window; workflows.md:109 "models have no git access"); A3 (gates.ts:559–589
exactly as described); A4 (revert commit at 363–377 precedes the stage commit at
1881–1890, so the tree-to-tree window diff nets it); A5 (types.ts:202;
state-store.ts:97; cli.ts:17, 214, 227, plus thrown→1 at cli.ts:241);
A6 (state-store.ts:58–79 grammar text matches the register verbatim in meaning);
A7 (zod.dev/api Objects section fetched — "By default, unrecognized keys are
stripped"; StateSchema is plain `z.object` at state-store.ts:80; package.json
pins zod ^4.6.7, consistent with the fetched docs); A8 (prompts.ts:85–87 sha12;
runner.ts:1343–1345, prompts.ts:208 emits `- template: <file> @ <sha>`;
runner.ts:2453–2467 entry pin); A9 (2263/2298, 2325/2392 lock+finally,
2548–2554 ensureLockHeld at the stage boundary); A10 (types.ts:290–301,
states.ts:136–138, runner.ts:2406–2429); A12 (stages.ts:126 30-min default with
the light 12 in the comment; ADR:116–117). A11/A12/A13 are honestly labeled
unverified. The `for (;;)` / `decideNext` anchor at runner.ts:2440–2441,
`finalizeDone` at 3377–3425 with save-before-commit at 3395–3396 and the
terminal-save guard at 3401–3410, `planStopStatusLines` at 3580–3589, the
split-record resume posture at 2345–2378, and the engine floor
package.json:30–31 (`>=22.23.5`) all check out. Shell check legs were denied by
the runner this session, so claims that only `npm run check` could confirm
(suite green on Linux, no CI workflow) were taken as cited; every code-level
claim above was confirmed by direct read. The two external URLs the proposal
cites (the only two in the document) are both live and both support their rows.

## Findings
- [minor][new] Chosen design §1 (and Options: "Per-commit hooks") — "runs
  `git diff --name-only <headBefore>..HEAD`" — git enables rename detection in
  `diff` by default, and `--name-only` prints only the destination path for a
  detected rename. A self-change window that *removes* a surface file by
  renaming it out of the two prefixes (e.g. a template moved from
  `tools/orchestration/prompts/` to `docs/`) can therefore show only the
  non-surface destination and slip through the predicate — the design's
  "Directory scope… the moment it exists" argument covers additions, not this
  removal shape. The failure is loud (stage-entry load throws), not silent
  staleness, hence minor. Fix: specify `git diff --name-only --no-renames
  <headBefore>..HEAD` in §1 (renames then appear as delete+add, both sides
  filtered by the predicate); the Options section's citation of the same command
  inherits the fix.
- [minor][new] Chosen design §2 — "`restartRequired?: { stage: string; paths:
  string[]; at: number }`" — ADR-020 Decision 2 asks the pause to stop "with a
  `restart-required` marker naming the changed paths and the stage that will run
  next", but every artifact the design names (§2 record, `restart-boundary`
  gate entry, §3 status line "the stage that opened the boundary", the printed
  summary) names the stage that *completed*, not the next stage. The next stage
  is derivable from `lastStage` plus the flow, so nothing is lost, but the
  design never says where the ADR's "stage that will run next" lives. Fix: one
  sentence in §2 stating that the next stage is carried by the checkpoint's
  `lastStage` + flow and surfaced by the status line's resume posture — or add
  the next stage to the printed summary's content list.

## FYI (no severity)
- The exit-code table's `3` choice and the "record, not the integer, is the
  durable contract" posture are compatibility-seat territory; they read as
  consistent with ADR-020 Decision 3 from this seat.
- Revision log: all five r1 entries are decisions against research open
  questions, not closures of review findings; nothing re-raised applies to this
  first pass. Deferred items (self-restart, boundary cap, `.opencode/agents/**`)
  each quote ADR-020's "Not decided here" faithfully — no contradiction.
