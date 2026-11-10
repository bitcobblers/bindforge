# Correctness review r1 (dissent pass c1)

verdict: request-changes

Scope: `git diff 1f3a83c 5996360` over `02-proposal.md`, then the full current
document. Checklist: `references/correctness.md` items 1–7.

## Did the r1 finding close?

Yes — closed for real, not reworded. Chosen design ¶2 now says "Bullet 2 permits
import depth alone, so the one comment update is an explicit deviation that this
commit amends bullet 2 to sanction", Rollout 2 tasks "the one-clause amendment to
Decision bullet 2 sanctioning that comment", Deferred says "Only its Decision
bullet 2 is edited, by the move commit", and Goals bullet 3 cites Chosen design
¶2 rather than the Decision. I re-read ADR-004:42-44 and every proposal citation
now matches it. The amended clause is the one my r1 `fix` proposed. Not
re-raised.

## Findings

- [major][new] Test strategy (intro + item 1) vs Rollout 1 — "the move is proven
  by the existing suite being unchanged, plus the one added assertion block in
  item 4" / "every previously existing test still exists and passes unmodified —
  assertion counts per file are identical before and after the move, and the
  moved files' diff contains only ¶2's allowed set". Rollout 1 now lands a
  test-only commit *before* the move that adds assertions to the gate suite —
  `tests/pipeline-gates.test.ts`, one of the 23 moved files (the existing
  `describe('stagePathAllowed')` block, verified at
  `tools/orchestration/tests/pipeline-gates.test.ts:294-313`). Measured against
  the branch's pre-work state, item 1's three invariants are all false for that
  one file: the suite is not unchanged (the intro admits it), its assertion count
  differs, and its diff across the two commits contains an assertion block that
  is neither an import line nor the comment line. Risks row 4 ("compares the
  vitest reporter's file/test counts to a pre-move baseline") inherits the same
  undefined baseline, and item 2's "with the pre-move test count" fails outright
  if the added block is a new case rather than extra `expect`s. Fix: name the
  baseline once — "before the move" = the move commit's parent, i.e. after the
  Rollout-1 commit — and record the one expected delta: `pipeline-gates.test.ts`
  gains assertions inside the existing `stagePathAllowed` block, its test count
  and the 26-file count are unchanged.

- [major][new] Goals/Non-goals bullet 2 + Chosen design ¶2 vs Rollout 1 —
  "Inside the move commit: no change to test logic, test names, fixtures, or file
  splits — 26 test files before, 26 after (Decision, bullet 2)". The revision
  adopts the strict reading of bullet 2 to justify amending it for one comment
  line, then silently narrows the same bullet to "inside the move commit" to
  admit a test-logic change (item 4's added assertions) that bullet 2's own text
  forbids ("The only content change permitted is the import depth… No test logic,
  test names, fixtures, or file splits", ADR:42-44) and that ADR Consequences:78
  turns into an acceptance gate ("gate is `npm run check` green with an unchanged
  test count"). One of the two readings has to be stated, not both: the
  comment-line deviation is declared and amended, this one is neither. Fix: one
  sentence in Rollout 1 (or Deferred) stating that bullet 2 constrains the move
  commit's diff only, that the Rollout-1 commit is a separate test change outside
  ADR-004's subject, and that it leaves the test count and file count unchanged —
  or fold it into the same one-clause amendment.

- [major][new] Test strategy item 2 + Risks row 1 — "fails closed on a
  half-applied move: a grep over `tools/orchestration/tests/` finds zero
  leftover `pipeline-*.test.ts` paths". Read as written (a content grep — the
  same verb in item 6 means a content grep), this check cannot pass:
  `tools/orchestration/tests/pipeline-independence.test.ts:145` carries the
  fixture string `tools/orchestration/tests/pipeline-lane.test.ts`, which matches
  `pipeline-*.test.ts` and which the document deliberately keeps ("Refreshing
  fixture path strings that mimic the old naming … no follow-up yet"). Verified:
  that line and `pipeline-parallel-escalation.test.ts:45` are the only two
  occurrences of that shape in `tools/orchestration/tests/`. As written the check
  either fails spuriously or pushes the implementer to edit the fixture the
  Deferred section defers — and Risks row 1's mitigation depends on it. The
  deferral itself is consistent with ADR-004 and is not re-opened; the new check
  contradicts it. Fix: make it a filename-level check — "a file listing over
  `tools/orchestration/tests/` (e.g. `git ls-files`) finds zero files named
  `pipeline-*.test.ts`" — and note that in-file fixture strings are excluded by
  Deferred.

- [minor][new] Chosen design ¶1 — "The move is one commit in
  `tools/orchestration/` with no config change, in four parts". The commit this
  section tasks is not confined to `tools/orchestration/`: Rollout 2 also commits
  the two skill-file amendments (`.opencode/skills/repo-practices/SKILL.md:21`,
  `.opencode/skills/test-strategy/SKILL.md:27`) and, newly, the ADR-004 Decision
  bullet 2 amendment in `docs/adrs/`. Fix: drop the path qualifier or name the
  three areas the commit touches ("one commit touching `tools/orchestration/`,
  the two skill files, and ADR-004 bullet 2").

- [minor][new] Chosen design ¶2 — "never a content-wide `../src/` substring
  substitution, which would also rewrite the fixture path strings fed to the gate
  (A14)". The hazard is not in the repo: the gate's fixture strings are
  `tools/orchestration/tests/…` and `docs/…` paths (`pipeline-gates.test.ts:296-313`,
  `pipeline-independence.test.ts:145`), none contains `../src/`. The actual hole
  in a `from '../src/`-keyed rewrite is unmentioned: non-import relative
  specifiers — `vi.mock('../src/opencode-runner.js', …)` and
  `importOriginal<typeof import('../src/opencode-runner.js')>()` at
  `tools/orchestration/tests/cli.test.ts:29-30`, the only two `../src/` sites
  beyond A8's 65 — which that pattern skips. Today that is harmless only because
  `cli.test.ts` is root-tier and does not move, a fact the design never states.
  Fix: replace the wrong clause with the checked fact — "no moved file contains a
  relative specifier outside A8's 65; the only such sites are the `vi.mock` and
  type-import in `cli.test.ts`, which stays at the package-root tier".

## Verification performed

- Item 1 (assumption register): A19 re-verified — `TEST_PATH_RE` at
  `tools/orchestration/src/pipeline/gates.ts:253`, consumed by
  `stagePathAllowed` at `gates.ts:262`, enforced at `runner.ts:232`, revert check
  at `runner.ts:1031`; grep for `pipeline-` over `tools/orchestration/src` returns
  nothing, so "no pattern keys on the flat shape" holds. Item 4's expected
  behavior matches `gates.ts:260-273` (nested path allowed for
  `test-authoring`/`test-update`, denied to `implement`/`fix`). A8's count of 65
  `from ['"]\.\.?/` matches re-verified; A14's two sites re-verified.
- Item 2 (anti-hallucination): https://vitest.dev/config/include (default
  `['**/*.{test,spec}.?(c|m)[jt]s?(x)']` — supports A5) and
  https://vitest.dev/guide/cli (positional args are path-substring filters, not
  globs — supports A6) are both live and support the rows that cite them.
- Items 3/5/7 are the findings above; item 6: the design does decide what ADR-004
  asked, and the r1 ADR-consistency defect is closed.

## FYI (no severity)

- The single move commit spans paths that no pipeline stage may write in one
  pass (test files are denied to `implement`/`fix`; `.opencode/` is denied to
  every stage per `gates.ts:260-273`). Harmless only because ADR-004's Ordering
  clause has it landed by hand between runs — worth a sentence if another
  specialty cares about who lands it.
