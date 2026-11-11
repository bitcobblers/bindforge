# correctness review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`) — a language-only
pass over the proposal. This seat had zero findings in r1, so the confirmation
rule applied is: report only where the revised text breaks something previously
approved or makes a cited section false. Each of the six hunks was checked
against the proposal body and `docs/adrs/008-rust-for-the-product.md`.

## Findings

None. Per-hunk verification:

- Goals — "hub validating uploads with the same compiled types": now quotes the
  ADR-008 Consequences clause verbatim ("validates uploaded profiles with the
  same compiled types the client uses") and matches the Adversarial guard's
  parenthetical; it repairs, not breaks, the earlier "client types" drift.
- Gates — "and nothing installed" → "with nothing installed": matches Test
  strategy #5's verbatim phrase "with nothing installed beyond the pinned
  toolchain"; no semantic change.
- Adversarial guard — comma before ", with typed errors": the error-typing
  clause still modifies "parsing rejects documents larger than 1 MiB or nested
  deeper than 16 table/array levels"; the decided budget, variant names, and
  depth bound are untouched.
- TypeScript boundary — "Node is only ever a prerequisite" → "Node is a
  prerequisite only for that frontend's framework tooling": this now tracks
  A15's verified claim ("Node is a prerequisite only if a JavaScript frontend
  framework is used") instead of stating it as absolute; no contradiction with
  the Compatibility matrix contributor row.
- Risks — "not silently" → "not silent": grammar only; the MSRV-drift
  mitigation still names both `rust-version` and `rust-toolchain.toml`,
  consistent with Workspace shape.
- Deferred — "never interpolating paths into shell strings" → "no
  interpolation of paths into shell strings": same obligation, still owned by
  ADR-012; nothing in ADR-008's Decision or requirements contradicts the
  deferral.
- Appended "Editor notes" lines: new text, but they flag ambiguities in
  sentences the diff did not change; they are author TODOs, not revisions, and
  introduce no decision or mechanism.

Checklist items not re-evaluated in this diff-scoped pass: the assumption
register and its cited URLs are untouched by the diff (no rows added, removed,
or re-scoped), so items 1–2 of the correctness checklist carry forward from r1
unchanged; the diff introduces no new interfaces, names, or edge-case handling
(items 3–5), decides nothing beyond what the ADR asked (item 6), and closes no
prior blocker/major finding (item 7 — none existed).

## FYI (no severity)

- Workspace shape — "Dependency edges point one way: nothing → `bindforge-core`":
  under the arrow convention the same sentence uses for "`transforms`, `adapters`,
  `scanner` → `core`" (X → Y = X depends on Y), "nothing → core" reads as "no
  crate depends on core," which the next clause contradicts; the intended meaning
  is "core depends on nothing." Unchanged by this diff and already flagged in the
  document's own Editor notes for the author to resolve — recorded here only as a
  pointer, not a new finding of this pass.
- Rollout 5 — "the Node floor": the document never sets a Node floor; Gates
  "pin[s] the Node version explicitly in a `setup-node` step." Also already
  flagged in the Editor notes; the author's fix is to align the Rollout wording
  with the pinned-version language, not to invent a floor.
