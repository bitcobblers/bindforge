# correctness review r2

verdict: approve

Scope: the editorial round-2 diff (`reviews/design/editorial-r2.diff`, i.e. the
changes landed by `docs: apply final language pass to ADR-020 restart-boundary
proposal`). Ratification pass: findings only where a revision broke something
previously sound or made a cited section false. Previous correctness findings:
none.

## Findings

None. The diff is language-only against the correctness checklist:

1. Assumption register — untouched by the diff; I re-checked the `verified`
   rows the diff's changed sentences lean on, and their cited basis holds:
   A1 (`prompts.ts:81–83` `loadTemplate` reads at stage time; `cli.ts:20`
   `PROMPTS_DIR = join(fileURLToPath(new URL('../../prompts/', import.meta.url)))`
   → `tools/orchestration/prompts/`; `package.json:4` `"type": "module"`, so the
   runner's modules are ES modules cached for the process lifetime);
   A2 (`finalizeStage` runner.ts:1848, `finalizeReviewPass` :1940, revert
   commits inside `enforceOwnership` :292–379; `docs/workflows.md:109–123`
   "The runner commits each stage's work (models have no git access)" and
   "one commit lands per pass"); A3 (`gates.ts:559–589`);
   A5 (`types.ts:202`, `state-store.ts:97`, `cli.ts:17` "Exit codes: 0 done,
   2 escalated, 1 error", `cli.ts:214,227` `status === 'done' ? 0 : 2` — and no
   existing handler returns 3, so exit 3 is free); A6 (`state-store.ts:58–79`);
   A7 (`state-store.ts:80` plain `z.object` — zod 4 strips unrecognized keys by
   default — but `:256` returns `parsed as PipelineState`, so the raw parse, and
   with it an unknown key, survives and round-trips); A9 (`:2263`
   `acquireLock`, `:2548–2554` `ensureLockHeld` re-assert); A10
   (`types.ts:290–301`).
2. Cited sources — the two URLs named in the proposal are live and support
   what they are cited for: `nodejs.org/api/modules.html#caching` ("Modules are
   cached after the first time they are loaded"; its eviction lever is
   `require.cache`, i.e. CommonJS-only, which is exactly the negative half of
   A1's claim) and `zod.dev/api` ("By default, unrecognized keys are *stripped*
   from the parsed result").
3. Internal contradictions — the one claim-bearing edit, "any fresh process
   started from the run's worktree already imports and renders the changed tree
   (A1)", replaces an ungrammatical "has imported and renders" with A1's own
   verified phrasing; its extension is unchanged and stays consistent with the
   risk row about resuming from an older checkout and the matrix row about a
   pre-020 runner.
4. Edge cases — unchanged by the diff and still covered (non-surface windows,
   swept/reverted attempts, forced `--stage` resume, pre-020 readers, consecutive
   boundaries).
5. Interfaces — the diff-touched §3 bullet still names real, mutually
   consistent things: `planStopStatusLines` at runner.ts:3580–3589 (the exact
   cited range) and `statusLines` at :3598, which Test #5 names;
   `finalizeRestartBoundary`, `touchesExecutableSurface`, `restartRequired`,
   `restart-boundary` are used by one name each across §1–§3, the risks, the
   matrix and Tests #1–#11. §1's `for (;;)`/`decideNext` anchor
   (runner.ts:2440–2441) and `finalizeDone`'s terminal pattern with the
   save-before-commit guard (:3377–3425, comment at :3395–3396) are accurate, as
   is the resume split-record posture (:2345–2378) the §2 clear mirrors.
6. Source-document coverage — unchanged: ADR-020 Decisions 1–4 are all decided
   (surface rule + window net diff; clean pause with no attempt/round/session
   charge and no `ESCALATION.md`; exit 3 distinct from 2, recorded and printed;
   template-hash provenance pinned by Test #8), and the "Not decided here" items
   stay deferred consistently with the ADR.
7. Revision-round check — no earlier blocker/major exists in this seat, and the
   diff reopens none; the Revision log's r2 entries (`printable`/
   `boundedGateEvidence` posture, one net-diff check per window) are untouched.

## FYI (no severity)

- The editorial pass normalized `ADR 020` → `ADR-020` throughout (consistent
  with the document's other ADR references), but the spelling it introduced in
  Deferred — "per-session re-read behaviour" — now sits next to A13's "their
  staleness behavior is unknown" (A6's "behavior-safe defaults" is a verbatim
  quote of `state-store.ts:63` and must stay as-is). For the language seat:
  pick one variety in prose and leave quoted code comments alone.
