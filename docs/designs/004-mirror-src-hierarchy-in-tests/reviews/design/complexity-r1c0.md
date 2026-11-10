# complexity review r1

verdict: approve

Checked against the complexity checklist (items 1–7). Verified in-repo before
judging: `tools/orchestration/tests/` holds exactly 26 files, 23 matching
`pipeline-*.test.ts` (A1 confirmed by directory listing); no
`vitest.config.*`/`vite.config.*` exists anywhere in the repo (A5 confirmed);
the flat-form convention appears in exactly two live surfaces,
`repo-practices/SKILL.md:21` and `test-strategy/SKILL.md:27` (A13 confirmed,
no third surface found by repo-wide grep).

Item-by-item:

1. Reuse: the design introduces **zero** new packages, modules, interfaces,
   scripts, or config. Nothing to reuse; nothing to flag.
2. Traceability: all four moving parts trace to stated Goals — moves
   (Goals 1–2), uniform import rewrite (Goals 3–4), one-line carve-out
   (Goal 3's stated carve-out), two convention-line amendments (Goal 5, ADR
   Decision bullets 4–5). No orphan mechanism; no YAGNI found.
3. Smaller design: the irreducible core is the 23 renames + the import-depth
   rewrite; everything else is one comment line and two skill-file lines.
   No 30–50% reduction exists without leaving a defect (stale reference or
   stale convention). One micro-duplication flagged as a nit below.
4. Dependencies: none added; A10's prettier plugin is pre-existing.
5. Deferrable slice: no — splitting moves from the convention amendments
   leaves agents writing flat names after merge, contradicting the ADR's
   "Conventions follow the code in the same change" clause. Single commit is
   the thin slice.
6. Knobs/abstractions: no configuration knobs, no abstraction layers.
7. Value check: complexity is proportionate — arguably the most proportionate
   shape possible: mechanical renames only, all "machinery" is audit
   documentation (assumption register, matrix), not runtime or process
   surface.

## Findings

- [nit][new] Chosen design ¶4 — "amends both convention surfaces that encode
  the old flat form ... to the recursive form `tests/**/*.test.ts` plus this
  rule" — read literally, the full naming rule (module-named unit test vs
  descriptive cross-module suites) is written into *both* skill files, so
  after landing two live instruction surfaces carry a copy of the rule (plus
  the ADR record), inviting drift. Fix: state the rule once — put it in
  `repo-practices`' existing "## Tests" section (its natural home), and have
  the `test-strategy` amendment change only the glob it restates
  (`tests/*.test.ts` → `tests/**/*.test.ts`), matching that line's existing
  pointer style "Follow `repo-practices` conventions:". This makes the
  document's own "stated once" claim true system-wide and shrinks ¶4's diff,
  while still removing the flat form from both files (Test strategy #6's grep
  stays green because the flat form — not the rule — is what it targets).

## FYI (no severity)

- Chosen design ¶2's carve-out ("a comment path-reference to a file moved by
  this same commit is updated in the same commit") deviates from the ADR's
  literal bullet "The only content change permitted is the import depth"
  (ADR:42–43); the rationale is sound (A14) but the wording tension belongs
  to the compatibility/correctness seats, not this one.
