# performance review r1 (c2 ratification)

verdict: approve

Scope as directed: the blob diff `2efd6ad..4461833` of `02-proposal.md` plus
the full current document, checked only against what this seat approved in
round 1. The ADR and the Revision log were read for deferral/consistency
context. `01-research.md` is not an allowed input this pass.

## Findings

None. No blocker, major, minor, or nit is introduced by the r1 revision.

## Carry-over from this seat's r1 findings (not re-reported per pass rules)

- The r1 **minor** (no scan-level wall-time budget statement beside the
  per-request timeout, nor a Risks row) is neither addressed nor recorded as
  rejected in the Revision log. The r1 revision left every premise of that
  finding exactly as approved: "One GET per distinct unresolved slug — the
  ADR's singular call shape; batching is `025-R1`'s" (catalog.rs bullet),
  "TcpStream with a 2 s connect and 5 s read timeout", and scan step 5
  "For the distinct unresolved slugs ... one `candidates()` call each" are
  all untouched, and no new text makes the sequential scan cheaper or the
  sections cited false. The revisions did not break what was approved, so
  under the ratification reporting rule the finding stands as-is, unchanged
  severity, not gate-blocking.
- The r1 **nit** (resolve the authority once at `HttpCatalog` construction)
  is likewise unadopted and unchanged. The revised `probe()` now resolves
  and loopback-checks addresses once per run (a probe-time cost, and probe
  is fail-fast on refused connections); nothing in the change makes the
  per-request resolution story worse than approved.

## Ratification checks performed

- Changed text audited for new per-event cost, unbounded O(n²), or unbounded
  memory: the store gains a `65_536`-octet read budget and per-record
  spelling checks; the response parse gains case-insensitive header matching,
  exactly-one-numeric `Content-Length`, a header-block budget under the same
  `65_536`, and a `256`-candidate cap — all caps applied before allocation,
  so checklist items 2–3 are improved, not broken. `parse_endpoint` runs once
  at construction; `escape_and_truncate` on two render cells per row and on
  refusal echoes is trivial per-row CPU, not on any event path (checklist 1).
- Store model change ("one record per distinct slug", confirmation writes the
  one record for the slug from the first sharing row in the already-shipped
  `(slug, install_path)` order) keeps store size bounded by distinct confirmed
  slugs and adds no new lookup cost; caching/invalidation semantics
  (`025-L4` re-trust, no expiry) are untouched (checklist 5).
- Test-harness migration (Rollout 3: subprocess suites behind a loopback
  `TcpListener` fake) and the added pin cases affect test runtime only,
  negligibly; no product-path cost.
- `Deferred` re-check: "Batching candidate queries, and any concurrency in
  the client — ADR-026 (`025-R1`)" stays consistent with ADR-025 `025-C4`'s
  singular deliberately-narrow query shape; not re-opened.
- Code claims the diff touches were spot-checked read-only:
  `apps/discovery-cli/Cargo.toml:6-13` matches the revised A10 dep list, and
  `escape_and_truncate` exists and is exported (`crates/core/src/error.rs:190`,
  `crates/core/src/lib.rs:110`), so the render bullet's new reference is real.
- Checklist 6 (research baseline gap): not applicable — the revised text
  asserts no expected numbers, and the research document is outside this
  pass's allowed inputs.

## FYI (no severity)

- The revised response-parse bullet is meaningfully hostile-response hardening
  (security seat's theme); from performance it is a net gain in boundedness.
