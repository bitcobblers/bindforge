# Code review — compatibility — r1 c2 (ratification)

## Scope and method

Inputs: `git diff 3745f3c6d0d0855bf2b77ea2f400a9d16c0b6a6a..HEAD -- ':(exclude)docs/designs'`,
02-proposal.md, docs/adrs/003-parallel-inference-lane.md, and my own r1 findings.
Ratification mandate: check only that revisions landed after my r1c1 approval did not
break what I approved.

**The in-scope diff is empty.** Verified:
- `HEAD` = `ec6130a`; base `3745f3c` is an ancestor.
- `git log 3745f3c..HEAD` = one commit, `ec6130a docs: 003-parallel-inference-lane
  code-review stage`, whose stat shows it touches only files under
  `docs/designs/003-parallel-inference-lane/` (checkpoint update, r1c1 review reports,
  transcripts) — all excluded from this pass by the pathspec.
- `git diff --name-only 3745f3c..HEAD -- ':(exclude)docs/designs'` → no output.
- The test-update stage commit (`3745f3c test: …`) is the base itself, i.e. already the
  state I approved at r1c1.

Therefore every source and test path is byte-identical to the r1c1-approved state, and
the compatibility checklist items were re-checked as "unchanged from approved state",
with spot-checks of the one compatibility-relevant area my r1c1 finding touched.

## Checklist

1. **Platform code vs Compatibility matrix** — no change in scope. Spot-check: `docs/
   workflows.md` still carries the shipped knob docs (`--max-parallel`,
   `$OPENCODE_MAX_PARALLEL`, lines 87, 104–108, 143–144) matching the matrix's
   opt-in N>1 row. Unchanged.
2. **Node floor (≥ 22.12.0)** — no code touched; nothing new above the floor. Unchanged.
3. **Path joins** — no code touched. Unchanged.
4. **Guarded platform imports** — no imports added. Unchanged.
5. **Locale/layout reality** — no format/ID code touched. Unchanged.
6. **Native dependencies** — the in-scope diff adds no dependencies of any kind. Pass.
7. **Tests run on claimed platforms** — the test-update stage (`3745f3c`) is the base;
   no test changes landed after my r1c1 approval. Unchanged.

## Spot-check of carried r1 finding (was: nit)

Gate-label situation is exactly as I verified and approved at r1c1: `states.ts:35,37`
still writes the aggregated-verdict gate entries under `design-review` / `code-review`;
`docs/adrs/001-code-driven-agent-pipeline.md:121,190,197` still names
`allDesignReviewsApproved`/`allCodeReviewsApproved` in its gate table with no amendment
pointer, and `docs/workflows.md:3` still declares ADR-001 normative. Since ADR-001 and
workflows.md are untouched by the in-scope diff, this remains a documentation-only nit —
it did not grow, and nothing in this pass makes it worse.

## Findings

- **nit** — `docs/adrs/001-code-driven-agent-pipeline.md:121,190,197` — carried from
  r1c1, unchanged (ADR-001 is not touched by this pass's diff): the gate table still
  names `allDesignReviewsApproved`/`allCodeReviewsApproved` while the runner logs these
  gates under `<phase>-review`; an operator keyed on ADR-001 alone targets a name the
  code no longer writes. Fix (unchanged): amendment note in ADR-001's gate table, or a
  statement that ADR-001's table is historical and `docs/workflows.md` is the live label
  authority.

## Verdict

approve
