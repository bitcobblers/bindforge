# performance review r2

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r2.diff`, read against
`02-proposal.md` (including its Revision log) and
`docs/adrs/014-profiles-as-chains.md`. Previous-round findings: none
(editorial phase is one language pass per round). This is the confirmation
pass: I checked only whether the editorial revisions broke a performance
property or made a performance statement false.

## Findings

None. Every hunk in the diff is language-only (comma insertion, `refused` →
`refuse`, `chosen` → `selected`, `needs` → `needed`, article and clause
repairs, line reflow). No hunk changes a bound, a cost claim, a cap, or an
algorithmic statement.

## Checklist evaluation (performance)

1. **Input-path latency / per-event cost** — no change in the diff.
   Resolution stays a load-time operation, not a keystroke-path one; the
   "one per-resolution lookup of the `MAX_MANIFEST_ROWS`-capped action table"
   sentence is untouched.
2. **O(n²) over unbounded n** — untouched. The composite element bound
   (`MAX_CHAIN_DEPTH × 3 × MAX_PATCH_DELTA_ELEMENTS`) and the
   `MAX_MANIFEST_ROWS`-capped action join stand as approved.
3. **Memory growth** — the only cost-bearing hunk (Resolution, the composite
   cost paragraph) changes "at most `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES`
   (32 MiB)" to "**totalling** at most `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES`
   (32 MiB)". This *clarifies* the bound (a total across opened documents,
   consistent with A21's 32 × 1 MiB product and with the Risks table's
   "≤ 32 documents / 32 MiB"), it does not weaken it. Arithmetic checks out.
4. **Startup cost** — unchanged: bounded documents per `load_chain`, byte
   budget per document, unchanged.
5. **Caching / invalidation** — no caching introduced or altered; the
   pin-equality load refusal (the staleness answer) is untouched.
6. **Research baseline gap** — `01-research.md` is outside this pass's named
   inputs; evaluated via the proposal's A21 instead. The stated composite
   (32 documents / 32 MiB) matches A21's baseline product, and the editorial
   hunk is precisely the sentence that states it — no unexplained gap.

Cross-check of the reflowed hunks: "An absent file is `ChainLinkMissing`
naming the id", the `patch-empty-deltas` shell clause, the `diff`/`merge`
pin and refusal clauses, and the semicolon split in Test strategy #5 all
restate the approved text verbatim in substance; none contradicts the ADR's
Decision or requirements.

## FYI (no severity)

- Chain operations (`merge` bullet): the reflow leaves one line notably
  longer than the document's wrapping style
  ("...over every action either touches — exactly, not heuristically — and").
  Formatting seat's business, not performance.
