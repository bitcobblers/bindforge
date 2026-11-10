# Correctness review r1

verdict: request-changes

Scope: whole document (panel pass c0, round 1). Inputs: 02-proposal.md,
docs/adrs/004-mirror-src-hierarchy-in-tests.md, the Revision log, my own
prior findings (none — first pass).

## Checklist evaluation

1. **Assumption register** — every `verified` row re-checked against its
   cited basis: A1 (26 files, 23 `pipeline-*` + 3 root: confirmed by listing),
   A2 (ADR :33-38 name list matches on-disk files one-for-one; no collisions
   in `tests/pipeline/`), A3 (`src/pipeline/` holds 13 modules — confirmed),
   A4 (tsconfig.json lines 8/17/18 exactly as cited), A5 (repo glob finds no
   vitest/vite config; vitest default include `**/*.{test,spec}.?(c|m)[jt]s?(x)`
   confirmed recursive), A6 (vitest CLI docs: positional filter is path
   containment), A7 (gates.ts:253 regex verbatim; matches nested paths),
   A8 (grep `from ['"]\.\.?/` returns exactly 65 matches, all `../src/...`,
   covering all 26 files), A9 (eslint.config.js:5 ignores only dist/
   node_modules; package.json:23 check chain), A11 (test-engineer.md:9
   `**/tests/**`), A12 (test-authoring.md:3 directory-generic), A13
   (repo-practices SKILL.md:21 and test-strategy SKILL.md:27 verbatim),
   A14 (a repo-wide grep for `tests/pipeline-` inside tools/orchestration
   finds exactly two hits: the escalation:45 comment and the independence:145
   fixture string — the register's "exactly one site" holds), A15 (see 2).
   All verified rows hold. The three `unverified` rows (A16–A18) are honestly
   marked and not used as fact.
2. **Anti-hallucination gate** — spot-checked 3 cited URLs with webfetch:
   https://vitest.dev/config/include (default include is recursive — supports
   A5), https://vitest.dev/guide/cli (positional args are path-containment
   filters — supports A6), https://git-scm.com/docs/git-diff ("The default
   similarity index is 50%." and `diff.renames` "Defaults to `true`" —
   supports A15's rename-detection claim). All live, all support their rows.
3. **Internal contradictions** — one found; see the finding. Everything else
   agrees across sections (23+3=26 throughout; the 23-name list matches the
   ADR list exactly; the carve-out target `tests/pipeline/reply-contract.test.ts`
   matches the naming rule; Risks ↔ Test strategy cross-references resolve).
4. **Edge cases** — name collisions (none, A2), case-insensitive checkouts
   (A16, honestly unverified but the all-lowercase distinct list supports the
   inference), concurrent modification (A18 → Rollout 1), selection-command
   ambiguity (`vitest run tests/pipeline` substring-matches all 23 nested
   files and no root-tier file). A grep over `src/` and `prompts/` finds no
   other code or prompt encoding the flat test layout, so "no config change"
   holds.
5. **Interfaces** — stable names and shapes across sections; no double
   naming found.
6. **Decides what the ADR asked** — yes, except the carve-out sanction gap
   below.
7. **Revision rounds** — n/a; revision log records no prior findings.

Deferred items re-checked against the ADR: leaving the "14 modules" Context
error unedited contradicts neither the ADR's Decision nor its requirements —
consistent deferral, not a finding.

## Findings

- [major][new] Goals (¶ Goals bullet 3) + Chosen design ¶2 — "The only
  content change inside a moved file is its import depth, plus one stated
  carve-out (Decision, bullet 2…)" / "**Content edit (Decision, bullet 2 +
  carve-out).**" — the carve-out is cited to ADR-004 Decision bullet 2, but
  that bullet as written says the opposite: "The only content change
  permitted is the import depth (`../src/...` -> `../../src/...`)." The ADR
  (status: proposed) is left unamended — the Deferred section defers only the
  "14 modules" Context fix, and nothing in the proposal tasks an ADR
  amendment — so the design as written directly contradicts its source
  document: the single Rollout-2 commit contains a content change the
  Decision forbids, and the Risks-table check ("the diff contains only
  renames, import-depth lines, and that one comment line") is enforced
  against a rule the accepted Decision does not contain. The carve-out's
  *substance* is sound (the ADR Context premise "the tests … never …
  reference each other's paths" is false — A14's comment site is verified,
  and my repo-wide grep confirms it is the only real one), but the deviation
  is unsanctioned while being presented as Decision-compliant. Fix: amend
  ADR-004 Decision bullet 2 with one clause — "the single exception: a
  path-reference comment naming a file this move relocates is updated to its
  new path in the same commit" — and keep the proposal's "(Decision, bullet
  2)" citations pointing at the amended bullet (alternatively: stop citing
  bullet 2 for the carve-out and record it as an explicit, tasked ADR
  deviation instead).

## FYI (no severity)

- A10's basis is the cited plugin dependency plus the uniform-prefix argument
  rather than an executed prettier run; the argument is sound (a common-
  prefix substitution is order-preserving when every relative specifier in
  every file shares the prefix, per A8) and `npm run check`'s format:check
  catches any violation, but a test-strategy seat may want it pinned
  empirically — not a correctness defect.
- A3's 13-vs-14 module count corrects the ADR Context; the deferral is
  consistent with the source document — a reviser must not "fix" it by
  editing the ADR, since leaving it as written is a recorded decision.
