# performance review r1 (ratification c2)

verdict: approve

Scope: `git diff fc0a1f9 0dacb96` on `02-proposal.md`, plus the full current
document. This pass checks only that the post-r1 revisions broke something the
performance seat approved or made a cited section false. No previous
performance findings were open.

## Checklist walk (against the changed text only)

1. **Input-path latency / per-event cost** — the diff's new Keyboard geometry
   row and the Product artifacts addition both *narrow* the performance
   surface rather than widen it: "`apply` and `load_policy` see only action ids
   and closed-vocabulary physical input ids (A19), never a composed character"
   and "touches no OS input API ... parsed documents in, parsed documents out"
   confirm there is no keystroke path, no synchronous IO, and no OS input
   hook in what ships. Nothing on the event path. Pass.
2. **Structure costs** — the new text's only algorithmic claim is
   "`relocate-cluster`'s contiguity and legality read the loaded grid", which
   matches the unchanged stack section and stays inside the unchanged
   `MAX_POLICY_TRANSFORMS`/`MAX_RELOCATE_TARGETS` composite. No new O(n²)
   shape. Pass.
3. **Memory growth** — no cache or table introduced or implied by the diff.
   Pass.
4. **Startup cost** — the diff adds no boot-time read or parse claim; the
   unchanged Budget and cost section's composite still covers document reads.
   Pass.
5. **Caching/invalidation** — no caching claim changed; "no clock, no
   randomness" determinism text is untouched. Pass.
6. **Research baseline vs expected numbers** — the diff introduces no numeric
   claim. The one quantitative assertion it carries ("exactly one geometry
   ships", "exactly one package" context) matches the proposal's own cited
   baseline (A20; the TF2 exec-chain lesson in Budget and cost, unchanged and
   previously approved). No gap. Pass.

## Cross-reference integrity (changed text)

- Risks row mitigation ("the Compatibility matrix's Keyboard geometry row
  states what no v1 policy may claim") now points at a row that exists; the
  old sentence's substance ("ISO/JIS legality needs new table files, not
  logic") survives verbatim in that row, so the citation does not dangle and
  the claim is still stated exactly once.
- Deferred → "ISO/JIS grid tables" re-phrasing defers to the same row; it does
  not contradict the ADR, which is silent on ISO/JIS.
- No test-strategy, rollout, or budget statement became false.

## Findings

None.

## FYI (no severity)

- The Keyboard geometry row mixes two orthogonal scopes (geometry tables vs
  IME/composition) in one cell; a wording seat may want them split, but both
  claims are accurate as written.
