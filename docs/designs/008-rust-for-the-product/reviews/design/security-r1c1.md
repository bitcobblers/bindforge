# security review r1 (dissent pass c1)

verdict: approve

Scope: `git diff 04a4ae1a2d2f75dc291f14e573660a483f51bbaa 6e4ea214d807aebc3faf5a2e28068a7b30b54411`
(blob ids of `02-proposal.md`), then the full current document, plus
`docs/adrs/008-rust-for-the-product.md` and the Revision log. Advisory checks
were re-run against the dependency set the revised text names (crates.io OSV +
GitHub global advisory DB + NVD): serde, clap, toml_edit, axum, tower, sqlx,
tokio, tauri. No named dependency carries an unfixed advisory, so the
checklist's blocker condition is not met (sqlx's only advisory, RUSTSEC-2024-0363,
is fixed at 0.8.1 and A3 states 0.9).

All eleven r1 findings are closed by the revised text, and none of the closures
re-open a Deferred item that contradicts ADR-008. The findings below are all
residuals *created by* the new text, not re-reported themes.

## Findings

- [minor][new] Chosen design → "Adversarial guard" — "parsing rejects documents
  larger than 1 MiB or nested deeper than 16 table/array levels with typed
  errors" + "This design decides that input budget, the two error variant
  names, and the depth bound" — the budget is now stated (r1 #2 closed), but the
  sentence claiming to decide "the two error variant names" names only `Cycle`
  and `ChainTooDeep`; the size and nesting violations are "typed errors" with no
  names, so the third-party conformance obligation this section exists to
  publish ("so third-party readers can conform") is unfulfilled for the two new
  guards. Nothing in the design's own tests pins them either: the same paragraph
  says "the invalid-fixture corpus is the contract for that behavior", yet
  Test strategy #3 boundary-tests only the chain depth, and no named fixture
  exercises >1 MiB or >16 levels — so the newly-added guard is the one guard in
  the section with no contract behind the claim. Fix: name the two variants
  beside `Cycle`/`ChainTooDeep` and extend Test strategy #3 into an
  input-budget test that asserts rejection one level past 16 and one byte past
  the cap (assert the size case in-test rather than as a checked-in fixture, so
  the corpus does not carry a 1 MiB file).

- [minor][new] Chosen design → "TypeScript boundary" — "the desktop design pins
  a `tauri` floor no lower than 2.11.1 (current fix for the IPC-origin
  advisories)" — the floor closes r1 #5, but the parenthetical is already stale
  and the design has no mechanism that keeps framework advisories in view. As of
  this review a later core advisory is published: CVE-2026-95626 /
  GHSA-6vxm-x265-58qf (Sept 2026) — Tauri's CSP nonce hardening gives no
  protection when the app's `script-src` includes `data:` or `blob:`, i.e.
  arbitrary script execution in the webview, which is the exact threat
  capability-based isolation is cited here for. That advisory is published only
  as a Tauri *repository* advisory: it is absent from the GitHub global
  advisory DB and from crates.io OSV, so the design's new
  `cargo deny check advisories` gate will not surface it either, and
  A13 ("2.12 line") sits above a numeric floor of 2.11.1. Fix: state the floor
  as moving — "the newest `tauri` release at the time the desktop design lands,
  no lower than 2.11.1" — and add one clause (or a Deferred bullet naming the
  desktop design) requiring the webview CSP to omit `data:`/`blob:` from
  `script-src`; that is the framework-side pair to the capability rule.

- [nit][new] Chosen design → "Gates (ADR-008 clause 4)" — "`deny.toml` is
  committed at the workspace root" — the gate is added and pinned to the
  committed lock (r1 #6 closed), but no triage rule accompanies it. The cheap way
  to keep a red advisory gate green is a `skip` entry in the committed
  `deny.toml`, which restores exactly the informal posture r1 #6 replaced, and
  that file is ordinary contributor-editable text with no review bar named. Fix:
  one clause in the same sentence — "a `deny.toml` `skip` names the advisory id
  and a tracking issue for the upgrade that removes it" — no new tooling.

- [nit][new] Chosen design → "TypeScript boundary" — "the `fs`, `shell`, and
  `dialog` plugins are not enabled" — the enumeration closes r1 #4 and does
  cover the dialog-plugin scope-expansion advisory CVE-2026-95627, but an
  enumerated ban reads as permission for the unlisted ones, and the two that a
  desktop app is most likely to reach for next are `http` and `updater`
  (the latter has its own 2026 advisories and brings the update-signing secret
  and artifact-verification posture, i.e. the "where are secrets stored"
  question this checklist tracks, which no document in the run names). Fix: keep
  the enumeration in the capability-file rule but pair it with a one-line
  default — "no other Tauri plugin is enabled without a reviewed capability
  entry" — and put the updater signing-key posture in Deferred under the desktop
  design.

## FYI (no severity)

- Test strategy #5 ("one-gate promise") still lists only `cargo fmt`,
  `cargo clippy`, `cargo test --workspace --locked` after the gate block grew a
  fourth command; either add `cargo deny check advisories` to that list or say
  why it is exempt (it is the only gate command that needs network, to clone the
  advisory DB). Correctness/test-strategy seat.
- Removing the `[workspace.lints.rust]` / `[workspace.lints.clippy]` tables also
  removed the prose's claim that members inherit lints ("Workspace members
  inherit edition, `rust-version`, and dependency versions"), so there is no
  longer a single declared place for a lint policy over the crate that parses
  untrusted files. A one-line `unsafe_code` forbid in `bindforge-core` would be
  the cheap version; ownership of lint policy is the correctness seat's call.
- Goals still read "Chain resolution hardened against adversarial profile
  files", narrower than the guard section, which now covers size, nesting,
  version, and chain depth across all entry points.

## Closures checked (no finding)

- r1 #1 untrusted-input premise: stated verbatim at the head of "Adversarial
  guard", with the invalid-fixture corpus named as the contract, entry points
  enumerated.
- r1 #2 input budget: 1 MiB / 16 levels, typed errors, mirrored in
  `schemas/README.md`, and the ADR-014 deferral no longer defers file size —
  the bound now lives in core on every path.
- r1 #3 missing version: rule now rejects "a document that declares no version";
  Test #2 and a named `fixtures/invalid/` case cover it.
- r1 #4 capability posture: plugins disabled in the boundary paragraph and in
  Rollout 5.
- r1 #6 advisory gate: `cargo deny check advisories` in the gate block,
  `deny.toml` in Rollout 1.
- r1 #7 toolchain bump for a published toolchain security advisory: allowed.
- r1 #8 runner posture: `permissions: contents: read`, action SHA pins, explicit
  Node version, repeated in Rollout 4.
- r1 #9 zero-fixture conformance: harness now fails the run; scaffold scope
  (`profile/v1`, `reach-classes/v1`) keeps the walk green at merge;
  majors↔`schemas/*/v*/` equality added.
- r1 #10 disclosure owner: ADR-009 for manifest/scanner-derived identifying
  fields, ADR-012 for path redaction when fingerprinting.
- r1 #11 path handling in the codec deferral: canonicalization, `..`/absolute
  rejection, no shell interpolation now named in the Deferred bullet.
