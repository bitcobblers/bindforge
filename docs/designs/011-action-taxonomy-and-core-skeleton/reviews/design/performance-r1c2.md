# Performance review r1c2

verdict: approve

Ratification pass c2: only the r1 revisions (diff `118d4d2..f23e3e4` of
`02-proposal.md`, then the full current document) are in scope, checked against
the round-1 performance approval. Per the pass rules, a finding is reported only
if a revision broke something previously approved or made a cited passage false.

## Checklist evaluation

1. **Event-path latency** — nothing in the revisions touches the event path. The
   new version-equality check in `load_action_registry` is load-time (one string
   comparison against the directory name), and the escaped echo of an offending
   id runs only on rejection paths. No per-event allocation or IO added.
2. **Structure costs** — no O(n²) introduced. The new source-classification
   clause ("an `action-ids` document from elsewhere is data — never loaded as
   the registry, never consulted for `axis_kind`") actively narrows the lookup
   set to one shipped copy, which is performance-positive.
3. **Memory growth** — strengthened, not broken: the entry table now states the
   axis arrays are "length bounded by the shared parse budget", bounding the id
   count before the file is read. Consistent with the ADR (the parse budget is
   the ADR-008 shipped guard ADR-011 keeps; no contradiction).
4. **Startup cost** — unchanged: still one bounded `registry.toml` read through
   `parse_file` with the size budget binding first.
5. **Caching / invalidation** — no new cache. The revision adds an invalidation
   rule instead: only the `registry.toml` in the shipped versioned directory is
   the registry, so there is exactly one copy to invalidate. The v2-directory
   posture for grammar widening keeps the version gate as the sole refusal
   mechanism — no dual-registry resolution cost appears in v1.
6. **Research baseline** — the changed text introduces no new expected numbers,
   so there is no new gap against the baseline to flag; the r1 baseline check
   stands.

## Findings

None. No revision broke an approved section or made a cited passage false.

The r1 cited passage — `ActionRegistry` — "`version` plus the ids in document
order, with `ids()`, `contains(&ActionId)`, and `axis_kind(&ActionId) ->
Option<AxisKind>`" — is untouched by the diff, so the r1 nit is neither closed
nor invalidated.

## FYI (no severity)

- r1 nit (lookup representation behind `contains`/`axis_kind`): still open —
  neither adopted nor recorded in the Revision log — and still not gating; the
  new "length bounded by the shared parse budget" clause further bounds n. Not
  re-reported as a finding per the ratification-pass rule.
- Other specialties: the entry table's "`version` ... equal to the version
  segment of the directory holding the file" is enforced in
  `load_action_registry` against the shipped directory, but the wording could be
  read as applying to fixtures under `fixtures/valid/` whose immediate parent
  has no version segment; correctness/compatibility may want to confirm the
  contract wording means the versioned directory root.
