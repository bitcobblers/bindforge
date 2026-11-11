# compatibility review r2

verdict: request-changes

Scope: `reviews/design/editorial-r2.diff` (single editorial language pass, round 2).
My previous-round findings for this seat: none, so there is nothing for the
revisions to have broken or made false. The six body hunks are semantically
neutral or improvements: "client types" → "the same compiled types" now quotes
ADR-008's Consequences clause ("the hub validates uploaded profiles with the
same compiled types the client uses") exactly, matching the Adversarial guard's
existing quote; the remaining hunks are a moved comma, "not silently" →
"not silent", "Node is only ever a prerequisite for" → "Node is a prerequisite
only for", and "never interpolating" → "no interpolation of" — none changes
platform, toolchain, path or input-handling meaning. The one open compatibility
item the diff leaves in the document is the appended editor note on the Node
floor.

## Findings

- [major][new] Revision log (appended editor note) + Gates + Rollout —
  `Editor notes: Rollout — "the Node floor" is ambiguous; author to clarify.`
  sitting against Rollout 5's `the cargo gate commands, the Node floor, the
  capability-file rule` and Gates' `Node version pinned explicitly in a
  setup-node step`. Compatibility checklist item 3 (Node version floor) cannot
  be evaluated: the design requires a floor — CI pins a Node version, and the
  `repo-practices` Rust section must document "the Node floor" — but never
  states the floor's value or where it comes from, and the document has two
  different Node consumers with different floors: the `orchestration` gate
  (this design) and the desktop webview's framework tooling (deferred to the
  desktop design). With no stated floor, `setup-node` in Rollout 4 and the
  repo-practices bullet in Rollout 5 can be filled in differently by two
  implementers, and "any API used above the floor" is uncheckable. Fix: state
  the floor once in the Gates paragraph as the Node version
  `tools/orchestration` already builds under (its `engines` entry), say the
  `setup-node` `node-version` mirrors that same value, and in Rollout 5 replace
  the bare `the Node floor` with that reference; the desktop-era Node
  requirement stays deferred with the desktop design, so name the deferred
  bullet there as the second floor's home rather than leaving one phrase
  covering both.

## FYI (no severity)

- Chosen design → Workspace shape — the other appended editor note, `nothing →
  `bindforge-core``, reads as a contradiction of the same sentence's "`transforms`,
  `adapters`, `scanner` → `core`" unless "nothing →" means "core depends on nothing";
  dependency-direction clarity, not a compatibility matter.
- TypeScript boundary — "Node is a prerequisite only for that frontend's
  framework tooling (A15)" states the prerequisite flatly while the framework
  that would require it is itself deferred; A15's "only if a JavaScript
  frontend framework is used" condition is what the Compatibility matrix's
  contributor-dev row and Rollout 6 quietly rely on (rustup only). Wording is
  unchanged in substance by this diff, so not a finding here.
- Compatibility matrix, Test strategy, Deferred bullets are untouched by this
  diff; on my checklist they still hold — every matrix row carries a status, no
  input-event API is targeted and the Context requires a future capturing
  component to name its API per platform, config path discovery and
  path-canonicalization obligations are deferred consistently with ADR-008's
  `crates/adapters` clause (no contradiction with the ADR's Decision), and
  Test strategy #7 names a Linux CI leg plus the Windows contributor leg for the
  platforms the matrix claims.
