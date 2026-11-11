# compatibility review r3c0 (ratification pass)

verdict: approve

Scope: the diff of `02-proposal.md` between blobs `67a6a1a3…` and `bbece027…`
(the r2 revisions), plus the full current document. Inputs per the caller:
the proposal, the Revision log inside it, `docs/adrs/008-rust-for-the-product.md`,
and my r3 findings. Ratification rule applied: report only what the revisions
broke among what I approved, or what they made false in a section I cited.

## Findings

None. The r2 revisions did not break what I approved in r3, and did not make
any section I cited newly false.

Diffs checked against the compatibility checklist:

- **Node floor handling (Gates, Rollout 4/5, TypeScript boundary, Deferred, A28)** —
  the changed text replaces "Node version pinned explicitly in a `setup-node`
  step" with a concrete floor: `setup-node` `node-version: 22.23.4`, sourced
  from `tools/orchestration`'s `engines.node` entry (A28, verified), and the
  `repo-practices` bullet plus Rollout 4/5 mirror that one value. Checklist
  item 3 stays satisfied: the CI job runs at the floor version, and A28 claims
  the repo builds and checks under it today. One micro-nuance, not raised as a
  finding: the pin `22.23.4` is the lower bound of the `>=22.23.4` range, so
  "taken verbatim from its `engines.node` entry" drops the range operator —
  pinning at the floor is the conservative reading and the sentence states it
  correctly ("the version `tools/orchestration` already builds under"). The
  "only Node floor this design sets" claim is consistent with TypeScript
  boundary ("any Node floor they bring are the desktop design's choice") and
  with the Deferred bullet adding the desktop-era Node floor to the desktop
  design's ownership; neither contradicts ADR-008's Decision or consequences.
- **Context bullet** — "client types" → "the same compiled types" now tracks
  ADR-008's Consequences verbatim ("the hub validates uploaded profiles with
  the same compiled types the client uses"). Improvement, no compatibility
  effect.
- **Workspace shape dependency-edge sentence** — rewording preserves the
  meaning (core is the depends-on-nothing root). No effect.
- **A28 register row** — verified, dated basis named. Register hygiene fine.
- **Adversarial guard / Risks / Deferred word-smoothing** — meaning preserved.
- **Matrix, OS input APIs, native modules, keyboard reality, file paths,
  platform tests (checklist items 1, 2, 4, 5, 6, 7)** — untouched by the diff;
  each remains in the state I approved in r3 (every matrix row has a definite
  status; no input-event API targeted, per Context and ADR-008's "files remain
  the product"; Test #7 names the Linux and Windows legs the matrix claims).

## Status of my open r3 findings (unchanged by r2 — not re-reported per the ratification rule)

- [minor, still open] Test strategy #5 "every command listed in the Gates
  block" vs the Gates prose naming `cargo deny check advisories` and
  `npm ci && npm run check`: Test #5 is untouched in the diff, and the r2
  rewrite of the Gates section adds no *new* command mention (both command
  strings were already in the r3 base), so the referent ambiguity has exactly
  the same scope as when I approved with it. The Revision log records the
  r2 Gates/Node changes but does not address this collision. My proposed fix
  (scope the citation to the fenced block's three commands) remains valid.
- [nit, still open] `rust-toolchain.toml` contents still name only
  `[toolchain] channel = "1.85.0"`; the `components = ["clippy", "rustfmt"]`
  gap under "nothing installed beyond the pinned toolchain" is unchanged.
- [nit, still open] cargo-deny install path ("installing `cargo-deny` at a
  pinned version") is unchanged; no new A25-analog entry was added.

## FYI (no severity)

- The r2 Node-floor closure is well-formed as an anti-drift mechanism: one
  value (A28 → Gates → Rollout 4/5 → repo-practices), with the desktop-era
  floor explicitly parked in Deferred. If the #5 citation fix is ever applied,
  it does not disturb this.
