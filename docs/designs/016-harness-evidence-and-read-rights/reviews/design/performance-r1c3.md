# performance review r1c3

verdict: approve

## Findings

None. This pass checked only whether the post-approval revisions broke anything
the performance seat approved in r1, or made a cited section false. They did not.

## Ratification checks (performance checklist against the given diff)

- Cited sections intact. §4's ceiling sentence — "states a ceiling: at most 25
  reads of files outside that set" — survives verbatim; its reworded follow-up
  ("machine-checking it is the Deferred entry") matches the Deferred bullet and
  does not contradict ADR-016 Decision 4, which only requires the ceiling be
  stated. §2's canonical shape — "GateEvidence = { at: number; green: boolean;
  legs: GateLeg[] }" — is unchanged, and the overwrite-at-each-gate-call
  invalidation my r1 nit relied on is unchanged.
- Bytes → chars capture (`MAX_GATE_TAIL_BYTES` → `MAX_GATE_TAIL_CHARS = 2_000`,
  decoded once with replacement): the gates.ts bullet keeps the rolling tail
  buffer per leg, so capture stays bounded; worst case ≈ 32 legs × 2,000 chars
  (≤ 8 bytes/char) ≪ the 8 MiB checkpoint cap (A6), so the "Checkpoint growth"
  risk row still holds. The A18 claim ("2,000 characters from the end
  comfortably holds cargo test's multi-line summary") is not weakened — 2,000
  decoded characters covers at least as much ASCII output as 2,000 bytes. Test
  strategy #1/#2 pin the new caps at write and render (A7). No memory-growth or
  startup cost is added; nothing new is read or parsed at boot.
- docMaxLines 600 → 900: now derived, not asserted — A24 measures the shipped
  artifacts (largest 733) and 900 stays far below the 2,696-line runaway the
  meter exists for. The old "600 — above the observed shipped artifacts" was
  the false one (733/728 exceed 600); the revision corrects it, and the
  in-flight-run compatibility row plus the Risks row cover a resumed document
  already over the cap. My r1 minor contrasted the un-derived 25 against the
  then-claimed 600 derivation; the change strengthens that grounding and does
  not make my cited passage false.
- Evidence block header line: added inside the still-pinned
  `MAX_EVIDENCE_BLOCK_CHARS = 6_000` cap, and test #7 still pins "the rendered
  block never exceeds MAX_EVIDENCE_BLOCK_CHARS" — the circular-pricing risk row
  is intact with the renamed capture budget.
- Research baseline (checklist item 6) still quoted: A19/A20 carry the
  189,788-token / 32,768-resident-KV / 2,696-line figures; §4's condensed
  "more concurrency evicts prefixes" keeps Decision 4's non-change clause. No
  new unexplained gap between the design's numbers and the baseline.
- My r1 findings (minor: 25-read ceiling lacks a derivation against the A20/
  189,788-token figures; nit: `at` unrendered / freshness rule implicit) are
  unaddressed by these revisions but also unbroken — per ratification rules
  they are not re-reported as findings.

## FYI (no severity)

- The capture change to raw decoded output plus the §2 line-selective build-note
  citation (dropping credential-bearing/absolute-path lines) moves the
  sensitive-content question into committed text — security seat's surface,
  flagged only as a cross-reference.
- `engines.node` → `>=22.23.5` in Rollout/Compatibility — compatibility seat's
  surface, no performance impact noted.
