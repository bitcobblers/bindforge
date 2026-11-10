# correctness review r1c3

verdict: approve

Scope: the diff `bedc938..1d1e68a` of `02-proposal.md` (Goals/Non-goals, A8, A15,
Chosen design ¶2, Risks row 3, Test strategy #1, Rollout 2, Revision log),
read against the full current document, ADR-004, and the repository.

## Closure check of my r1c2 blocker

Closed for real, not reworded. Re-verified against the tree, not the prose:

- `grep \.\./` over `tools/orchestration/tests/` = **87** matching lines; they
  decompose exactly as A8 now states: 65 `from '../src/...'` imports (3 of them
  in the three root-tier files, so 62 in moved files), the 2 non-import
  `../src/` sites at `cli.test.ts:29-30` (`vi.mock`, `importOriginal<…>`),
  **16** `new URL('<rel>', import.meta.url)` path strings in **11** moved files
  (11 `../prompts`/`../prompts/` + 5 `../../../.opencode/skills/…` in
  `pipeline-prompts.test.ts`, one of which is a multi-line call at 260-263),
  and 4 `'../escape'` fixture strings. 65+2+16+4 = 87. (My r1 figure of "12
  moved files" was my own miscount; the proposal's 11 is right.)
- No other depth-sensitive form exists: `grep ['"`]\./` finds nothing in the
  26 files, and `import.meta.url|__dirname|process.cwd` matches only those 16
  sites, so nothing resolves against a moved file outside A8's set.
- The `'../escape'` class is genuinely inert: `pipeline-state-store.test.ts:156`
  writes the string into a `mkdtemp` checkpoint; `pipeline-identity.test.ts:65,68,103`
  feed it to `sanitizeSlug`/`deriveIdentity`. Never resolved against the file.
- ¶2's rewrite set, Test strategy #1's allowed diff set, and Rollout 2's
  move-commit contents all now carry the 16 URL sites; the false completeness
  clause is gone; A8's basis is the checked enumeration.

Checklist otherwise: spot-checked 3 register URLs — `vitest.dev/config/include`
(default include `['**/*.{test,spec}.?(c|m)[jt]s?(x)']`, recursive: supports A5),
`vitest.dev/guide/cli` (positional args are path-substring filters, not globs:
supports A6), `git-scm.com/docs/git-diff` ("The default similarity index is
50%", `diff.renames … Defaults to true`: supports A15 as revised). No config
file matches `**/{vitest,vite}.config.*` (supports A5). Deferred section
re-checked against ADR-004: the four deferrals track its Alternatives,
Consequences and Context; none contradicts a Decision clause.

## Findings

- [minor][new] Chosen design ¶2 — "`../prompts/` → `../../prompts/`" — the
  mapping as written covers 15 of the 16 URL sites: `pipeline-fix-trigger.test.ts:8`
  uses `new URL('../prompts', import.meta.url)` with no trailing slash, so a
  rewrite keyed to the quoted pattern leaves one site one tier shallow.
  Test strategy #1 only asserts the diff stays *inside* the allowed set, so it
  cannot catch the missed site — only the acceptance run (#5) would.
  Fix: key the rewrite to the call rather than the string literal — "each
  `new URL('<rel>', import.meta.url)` in a moved file gains one `../`" — with
  the two mappings kept as examples.
- [minor][new] Chosen design ¶2 (the clause the commit writes into ADR bullet 2)
  — "a relative path reference — code or comment — naming a location this move
  passes takes its new depth" — at the edge this revision just closed, the
  clause reads wide enough to cover A8's fourth class: the 4 `'../escape'`
  strings are relative path references in code, and the Deferred section
  deliberately keeps other in-test path strings stale. The exclusion currently
  lives only in A8's word "inert", so the durable ADR sentence and the
  proposal's closed enumeration can be read against each other at implementation
  time. Fix: qualify the clause by resolution, not by label — "a path reference
  that the moved file resolves against its own location — code or comment —
  takes its new location" — which excludes the escape strings and A14's inert
  fixtures by definition.
- [minor][new] Chosen design ¶2 — "takes its new depth" plus "(the comment
  site: A14)" — the comment A14 names is
  `tests/pipeline-reply-contract.test.ts` at `pipeline-parallel-escalation.test.ts:45`
  (verified verbatim). It carries no `../`, so it is not inside A8's 87-site
  enumeration, and its edit is a directory insertion plus prefix drop, not a
  depth change. r1's clause ("takes its new path") sanctioned that edit; the
  replacement clause's "new depth" does not describe it, leaving the comment
  line covered only by the parenthetical while the ADR sentence is what a
  reviewer of the commit will check. Fix: one word — "takes its new location" —
  and in ¶2 name the actual edit ("the comment's path gains the `pipeline/`
  tier and drops the prefix: A14").
- [minor][new] Goals / Non-goals — "no change there to test logic, test names,
  fixture inputs, or file splits" — restates Decision bullet 2, which says
  "No test logic, test names, **fixtures**, or file splits" (ADR:42-44).
  Rollout 2 adds exactly one clause to bullet 2, so this word narrowing exists
  only in the proposal and is not recorded in the Revision log — while the move
  commit now rewrites 16 path constants that feed `readFile`/`loadTemplate`.
  Fix: quote bullet 2's word ("fixtures") and say that ¶2's clause is what
  licenses those path-constant changes, or fold the word change into the
  Revision log's ¶2 entry; either replaces the silent paraphrase instead of
  adding another caveat.
- [nit][new] Assumption register A8 — "all 87 `../` occurrences in the 26
  files" — the cited grep counts 87 matching *lines*; the 5
  `../../../.opencode/…` lines each contain three `../`, so literal occurrences
  are 97. Fix: "all 87 `../`-bearing lines", so the row's arithmetic
  (65+2+16+4=87) matches its basis verbatim.

## FYI (no severity)

- The Revision log's ¶2 entry names A8, Test strategy #1 and Rollout 2 but not
  A15, Goals bullet 3 or Risks row 3, which this revision also rewrote — the
  log's coverage of the same amendment is incomplete (editorial/process, not
  correctness).
