# performance review r3 (ratification pass c0)

verdict: approve

Scope: `git diff 67a6a1a380395c0cc041d0478f3213660b33681e bbece027b5eac36f3907aff1df1dd96c51c9f1b8` on
`02-proposal.md`, then the full current document. Inputs were exactly the named
paths; the previous performance verdict held no open findings, so this pass
checks only that the r2 revisions did not break what was approved.

## Findings

None. Every changed passage was walked against the performance checklist:

- **Adversarial guard (Formats contract)** — the only edit is a comma
  ("nested deeper than 16 table/array levels, with typed errors"). The parse
  budget that bounds memory growth (1 MiB documents, 16 nesting levels) and the
  `MAX_CHAIN_DEPTH = 32` bound that caps chain resolution (checklist items 2
  and 3) are intact, unchanged, and still the constraints ADR-009 must adopt.
- **Gates / Rollout 4–5 / new A28 (Node floor `22.23.4`)** — pinning
  `setup-node` `node-version` to a single exact value taken verbatim from
  `tools/orchestration`'s `engines.node` makes the CI orchestration job
  deterministic, matching the committed-`Cargo.lock`/`--locked` posture for the
  Rust side. It changes no cargo gate command and adds no step to the per-PR
  cargo gate, so the three-command gate and the one-gate promise (Test #5) are
  unaffected.
- **TypeScript boundary (Node sentence)** — making Node conditional on a JS
  framework and deferring any desktop-era Node floor to the desktop design
  removes an unstated future commitment; it introduces no startup, memory, or
  caching claim and stays consistent with ADR-008 (the "no Node runtime to
  install" rationale concerns end users; a build-time tooling prerequisite is
  permitted by A15).
- **Deferred (desktop item)** — "the desktop-era Node floor that toolchain may
  need" moved to the desktop design is consistent with the ADR's Decision; it
  does not contradict any clause and does not re-open a deferral.
- **Context bullet, dependency-edge sentence, Risks row, ADR-012 Deferred
  rewordings** — wording-only ("the same compiled types" now matches the ADR's
  Consequences verbatim; "nothing → core" → "core depends on nothing"). No
  structural or performance semantics change; the acyclic dependency shape and
  single declared floor remain as approved.
- **Checklist item 6 (research baseline)** — the diff introduces no expected
  performance numbers, so no gap against the research baseline is created by
  the revised text; my round-3 baseline check stands.

## FYI (no severity)

- The CI orchestration job now runs on exactly the Node floor rather than "an
  explicitly pinned" version; that is a coverage choice (oldest-supported
  rather than newest) worth a glance from the test-strategy owners, not a
  performance concern.
