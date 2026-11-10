# Code review — compatibility — r1 c2 (ratification)

**Range reviewed:** `git diff 1a66bda6a543508cf60bf792107a6d8b38bbfeee..HEAD -- ':(exclude)docs/designs'`
**Inputs:** that diff, `02-proposal.md` (current), `docs/adrs/004-mirror-src-hierarchy-in-tests.md`, my own r1 verdict (no findings).
**Verdict: approve.** Zero open blocker/major findings; no compatibility property that held at r1 was broken by the later fixes.

## What the range contains

One non-docs change: `tools/orchestration/tests/pipeline/gates.test.ts`, 24 insertions / 19 deletions, all inside
the `describe('test tree mirrors src/ hierarchy')` block. No `src/`, config, tsconfig, package.json, skill or prompt
path is touched. Two `it` blocks were reworded and their assertions retargeted:

- `:385-389` — exact inventory (`sorted toEqual` over the whole listing, `toHaveLength(23)`) replaced by the
  zero-flat-name check (`:387`, kept verbatim) plus per-path subset assertions (`:388`).
- `:397-401` — the `tests/pipeline` selection pin: positive half became a subset loop (`:399`); the substring
  filter itself (`:398`) and the root-tier exclusion (`:400`) are unchanged.
- `:330-337` — comment rewording only.

No file added, removed, renamed, or split; both `it`s still present, so the run's file and test counts are
unmoved by this range.

## Ratification question, per checklist item

1. **Platform code vs the Compatibility matrix.** The diff adds no OS-specific code and removes none. The
   matrix rows (Linux executed, macOS/Windows "Expected — assumption") are unaffected: the only platform-
   dependent dependency in the block is `execFileSync('git', …)` at `:371-375`, which this range does not touch.
   Nothing in the fix needs a guard. Pass.
2. **Node floor (`engines.node >= 22.12.0`, package.json:30-32).** New constructs are `for…of`, array spread,
   `Array.prototype.filter/includes`, a regex literal, and vitest `toContain`/`not.toContain` on `string[]` —
   all far below the floor and inside the pinned `vitest ^5.0.5`. No new import. Pass.
3. **Paths.** `:388`/`:399-400` compare literals built at `:365-366` against `git ls-files --full-name` output.
   That comparison is *correctly* kept in POSIX form on both sides: git's porcelain listing is `/`-separated on
   every platform, so the hardcoded `/` in the expected strings is right, and substituting `path.join` here would
   break it on Windows. The comparison is also index-cased, not filesystem-cased, so a case-insensitive checkout
   (A16) cannot flip it — the exactness `toContain` demands is the exactness the index guarantees. Pass.
4. **Guarded platform imports.** None introduced; imports at `gates.test.ts:1-19` are unchanged by this range. Pass.
5. **Locale/layout reality.** The names in `MOVED_NAMES`/`ROOT_TESTS` are ASCII; the flat-name pattern at `:387`
   and the `'tests/pipeline'` filter at `:398` are ASCII literals. Nothing in the fix introduces an encoding or
   non-ASCII assumption. Pass.
6. **Native dependencies.** None added anywhere in the range (`git diff --stat` shows one test file). Pass.
7. **Planned tests run on the platforms they claim.** No `skip`/`skipIf` was added or removed, and the two pins
   still execute wherever vitest runs, so Test strategy #7's claim (Linux executes #1–#5, macOS/Windows rely on
   A16) is unchanged. Pass.

## Did the fix break an approved compatibility guarantee?

The property to check was fail-closed behavior on a half-applied move, since the fix deleted the only exact-count
assertions. It survives:

- The prescribed fail-closed check — zero `pipeline-*.test.ts` names in the tracked listing, `:387` — is untouched,
  which is the check Test strategy #2 names as the lasting invariant ("a file listing … finds zero files named
  `pipeline-*.test.ts`").
- All 23 moved paths and all 3 root-tier paths are still asserted present (`:388`), so a partially applied move or
  a revert fails. Verified against the live index: `git ls-files --full-name tools/orchestration/tests/` returns
  exactly those 26 paths, and `MOVED_NAMES` matches the 23 files under `tests/pipeline/` one-for-one, so the
  retargeted assertions are green at HEAD, not merely plausibly green.
- Root-tier exclusion in the selection pin (`:400`) is intact, so #3's "none of the 3 root-tier tests" half is
  still pinned.

The dropped assertions pinned `26 files / 23 nested` as a *permanent* invariant. That is a testing-seat question,
not a compatibility one, and the current proposal explicitly directs it: Test strategy #2 says the exact
26-file/228-test figures are "that measurement at the move commit — Rollout 3's reported acceptance evidence — not
a permanent suite invariant", and that the lasting assertions are the zero-flat-name listing "plus the 23 moved
and 3 root-tier paths as a *subset* of it, since new mirrored test files are expected from the merge on"; #3 says
"the suite's own assertion simulates that path-substring filter over the tracked listing". The code now matches
that wording. Compatibility-wise the change is neutral-to-positive: a future mirrored test file (ADR Decision
bullet 4, proposal ¶4) no longer turns a platform-neutral pin red on unrelated branches, and nothing that carries
a platform claim moved from asserted status into reported-only status — the cross-platform rows of the matrix were
already assumption-based (A16) and the counts were already Rollout 3 evidence.

Formatting gate note: the longest new line (`:388`, 87 columns) is inside
`tools/orchestration/.prettierrc`'s `printWidth: 100`, so `npm run check` — the ADR's acceptance gate — is not
pushed red by the edit.

## Findings

None.

## FYI (no severity, out of seat)

The permanent suite no longer asserts anywhere that vitest's *real* nested discovery works; that remains Rollout 3
reported evidence plus A5's static argument, exactly as the amended Test strategy #2/#3 word it. Flagging only
because it is the one cross-platform behavior the suite no longer touches at all; it is not a compatibility defect
in this diff.
