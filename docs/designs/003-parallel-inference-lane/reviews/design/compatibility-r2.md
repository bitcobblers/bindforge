# compatibility review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (the r2 editorial language pass), confirmed
against `02-proposal.md`, `docs/adrs/003-parallel-inference-lane.md`, and the Revision
log. Previous compatibility findings: none (editorial phase is a single language pass
per round), so this is a confirmation pass: report only what the revisions broke or
made false.

## Checklist walk (against the diff hunks)

1. **Compatibility matrix** — untouched by the diff. All six rows carry a status
   (`yes`, `no`, `yes, conditional`); no `unknown`, no empty cell. The r1 closure in
   the Revision log ("macOS/Windows: `unknown` → `no`, untested") is present in the
   current text and stands.
2. **OS input APIs** — untouched. The diff adds no OS-facing behavior; the matrix
   justifies the abstraction ("no OS-specific code added"), consistent with ADR-003,
   which is a scheduling change in `runner.ts`.
3. **Node version floor** — untouched. No hunk introduces an API; A10's claim
   (`node >= 22.12.0` covers the fan-out primitives) is unchanged by the language pass.
4. **Native modules** — untouched. The diff adds no dependency; the in-process
   semaphore stays pure JS.
5. **Keyboard reality** — not engaged by this design or by any hunk.
6. **File paths** — the touched writer/CLI prose (`a range check (integer ≥ 1)`, the
   escalation paragraph) changes wording only; the tmp-file + rename behavior it
   describes (A7, state-store.ts) is identical to the pre-diff claim.
7. **Test strategy vs matrix** — the only touched test line is #9, whose "the pass
   produced one commit" → "the pass produces one commit" is a tense fix that aligns
   it with the present-tense rest of the list and with #6's "exactly one commit …
   per review pass". Linux remains the only claimed-supported platform and is
   covered by platform test #10; macOS/Windows stay `no`, so no test is owed for them.

Hunk-by-hunk semantic check: every change ("wall-clock cost", "semantics stay",
"cap stays in runner code", "has today's fallback shape", "releases its permit and
returns without prompting", "complete in-policy sibling reports", "a range check",
"produces one commit") preserves the technical meaning the ADR states (ADR-003
§Self-update commit-shape caveat, §Execution 3 escalation rule, §Configuration
strict validation). Nothing the revisions changed makes a previously approved
passage false.

## Findings

None.

## FYI (no severity)

- Chosen design / Escalate bullet: the r2 rewrite leaves an awkward hard line break
  ("then\n  `finalizeEscalate`") mid-sentence in the markdown source; it renders fine,
  but the editorial seat may want to re-wrap. Language-only, outside this specialty.
