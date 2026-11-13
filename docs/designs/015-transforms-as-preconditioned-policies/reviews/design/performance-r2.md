# performance review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial diff) against
`02-proposal.md` and `docs/adrs/015-transforms-as-preconditioned-policies.md`.
Ratification mode: checked whether any changed hunk broke a performance
property the section it sits in carries.

## Findings

None. Every hunk is language-only and touches no cost-bearing text:

- Goals — "Precondition outcomes are **verdict data** on `PlacementCheck`'s
  channel, never new `CoreError` variants" (`as` → `are`): grammatical fix; the
  verdict-vs-refusal split that keeps a no-fit row on the structured-data path
  instead of an error path is unchanged.
- Non-goals and Deferred — "(Options)" → "(Options considered)": the pointer now
  resolves to the real `## Options considered` heading, so the deferral of
  precondition composition and a fourth kind to `policy/v2` stays traceable. The
  deferral does not contradict the ADR's Decision or Consequences.
- Options → Precondition carriage — "mapped one-to-one to a named verdict":
  wording; the fixed one-to-one check↔verdict mapping (bounded per-row work,
  ≤ 64 rows × ≤ 16 moves) is intact.
- Options → Row encoding — "…fixtures, as `set`/`swap`/manifest rows already
  do": dropping "exactly" removes no guarantee; the wrong-field-count refusal
  and its fixtures are still required, so parse-level rejection of oversized
  rows (the bound that keeps row work linear in `MAX_RELOCATE_TARGETS`) stands.
- The stack — "exactly the answer a bool cannot give": the `PlacementCheck`
  evidence payload is unchanged; the `Option<PlacementCheck>` in `NotApplicable`
  is still filled only for `placement-illegal`, so no per-row evidence
  allocation is introduced.

No edit touches Budget and cost, the conformance walk, `load_policy`'s
single-file read under `MAX_DOCUMENT_BYTES`, or any caching claim — and the
design ships no cache, so invalidation is not a live concern here.

## FYI (no severity)

- The stack — "the `PlacementCheck` as the evidence, exactly the answer a bool
  cannot give" — the editorial deletion of "why" loosens the A4 claim ("a bool
  cannot say *why* a placement is illegal"): a bool already answers *whether*;
  the check supplies the *why*. Language/traceability, for the correctness seat.
- Revision log — "Editor notes: Test strategy — 'grammar_closure.rs refuses a
  policy document aliasing `transforms_`' is ambiguous; author to clarify" — an
  open editorial item now sits inside the deliverable rather than resolved in
  Test strategy #1; process/clarity, for the editorial/correctness seats.
- Research baseline (checklist item 6) could not be re-checked against
  `01-research.md` — that path is not among this seat's named inputs for this
  pass; nothing in the diff changes a number, so no gap was introduced.
