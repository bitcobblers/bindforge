# security review r3c0

Ratification pass. Inputs read: `docs/designs/008-rust-for-the-product/02-proposal.md`
(blob `bbece02`, full document), the diff `67a6a1a..bbece02` for that blob,
`docs/adrs/008-rust-for-the-product.md`, the Revision log, and my own r2 findings.
Checklist applied: security only.

verdict: approve

## Findings

- [nit][new] Chosen design → Gates (ADR-008 clause 4) — "whose `setup-node`
  `node-version` is pinned to `22.23.4` — the Node floor, i.e. the version
  `tools/orchestration` already builds under, taken verbatim from its
  `engines.node` entry (A28)" — the r2 revision turns the previously generic
  "Node version pinned explicitly in a `setup-node` step" into a concrete exact
  pin, but the paragraph's only refresh mechanism is scoped to Rust: "That same
  run is the trigger for the `rust-toolchain.toml` security-advisory bump
  (Workspace shape)". Every other frozen artifact in the design has a named bump
  path (`rust-toolchain.toml` on a published toolchain advisory; `Cargo.lock`
  through reviewed upgrade PRs; Risks row 3 for MSRV); the new Node pin does not,
  so an exact-version CI pin sits outside both the weekly advisory run and the
  bump trigger, and nothing tells the maintainer to move it when the next Node
  22 security release lands. Verified live: `22.23.4` is the current patched
  Node 22 release (nodejs.org release blog, 2026-10-21, "This is a security
  release"), so the pin is correct as of this pass — the finding is the missing
  refresh path, not a stale number. Not a blocker or major: CI Node only runs
  `npm ci && npm run check` for internal pipeline tooling under
  `permissions: contents: read`. Fix: widen the existing sentence rather than
  adding a mechanism — replace "the trigger for the `rust-toolchain.toml`
  security-advisory bump (Workspace shape)" with "the trigger for the
  `rust-toolchain.toml` security-advisory bump and for moving the `setup-node`
  `node-version` to the newest patched release of the pinned Node line
  (Workspace shape)".

## Ratification check — what the diff touched, and whether it broke anything

Checked hunk by hunk; none of the following introduces or falsifies a security
claim:

- Goals — "the hub validating uploads with the same compiled types": now matches
  ADR-008 Consequences ("validates uploaded profiles with the same compiled
  types the client uses"). Closer alignment, no change to the guard.
- A28 added (Node floor `>=22.23.4`) and the Gates pin to `22.23.4`: single value
  mirrored in Rollout 4/5 and Deferred; consistent internally and with the ADR,
  which fixes no Node version. Only gap is the nit above.
- Workspace shape — "Dependency edges point one way: `bindforge-core` depends on
  nothing": removes the earlier ambiguity (the old "nothing → `bindforge-core`"
  read as *nothing depends on core*, contradicting "`transforms`, `adapters`,
  `scanner` → `core`"). The dependency direction that keeps `core`
  dependency-free is now stated correctly, which is what the security review
  wants; no regression.
- Adversarial guard — comma insertion only ("nested deeper than 16 table/array
  levels, with typed errors"). Untrusted-input classification, 1 MiB size
  budget, 16-level nesting budget, `Cycle`/`ChainTooDeep`, and the
  fail-closed invalid-fixture contract are all intact (checklist items 1, 2, 7).
- TypeScript boundary — only the Node sentence changed ("Node is a prerequisite
  only if that frontend uses a JavaScript framework (A15)"), which matches A15
  and ADR clause 2's deferral of the webview framework. The capability rule, the
  disabled `fs`/`shell`/`dialog` plugins, the "never below 2.11.1" floor, and
  the A27 reference are byte-identical to what I reviewed.
- Risks row 3 ("not silent"), Deferred wording ("no interpolation of paths into
  shell strings"), Revision log rewrites of the Gates and TypeScript-boundary
  entries: no change to the controls they describe.

## FYI (no severity)

- My four r2 findings (2.11.1 floor below the 2.11.6 patch named by A27's
  GHSA-w28w-mhc8-qvjv; "crates.io/RustSec data the scheduled `cargo deny` scan
  reads" overstating cargo-deny's `db-urls` RustSec source; `deny.toml` `skip`
  instead of `[advisories] ignore`; `cargo install cargo-deny` without
  `--locked`) sit in text this diff left byte-identical. They are neither
  broken nor falsified by the revision, so per this pass's scope I do not
  re-report them; they remain open as written. Editor note: the Revision log
  carries no disposition for them — the log's TypeScript-boundary and Gates
  entries restate the old decisions ("with 2.11.1 only as a floor", the
  unchanged advisory-source clause) rather than answering them.
- Checklist item 3 (path handling: canonicalization, `..`/absolute rejection,
  no shell interpolation) is fully handed to ADR-012 by Deferred. That is
  consistent with ADR-008 clause 2, which assigns `crates/adapters` "config path
  discovery, write policy", so it is not a finding; it is the item to watch when
  the ADR-012 design lands.
