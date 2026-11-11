# Compatibility review r4

verdict: approve

Scope: `reviews/design/editorial-r4.diff` — 4 hunks (Goals bullet, Gates
paragraph, Risks mitigation cell, Rollout 5 bullet). Inputs: `02-proposal.md`,
`docs/adrs/008-rust-for-the-product.md`, the Revision log, and this seat's
prior findings (none — editorial is one language pass per round).

## Checklist walk-through

1. **Compatibility matrix** — untouched by the diff. Every row still carries a
   status (`yes`, `yes (target)`, or `no (v1)`); no `unknown`, no empty cell.
   Pass.
2. **OS input APIs** — untouched. Context still names no X11/Wayland/Raw
   Input/HID target: "it never reads or injects input events, so no
   X11/Wayland/Raw Input/HID API is targeted", and pushes any future capture
   component to its own matrix. Pass.
3. **Node version floor** — the changed Gates sentence leaves the floor intact:
   `22.23.4`, "taken verbatim from its `engines.node` entry (A28)", "this is
   the only Node floor this design sets". Rollout 5 keeps the single mirrored
   value ("the Node floor (the `engines.node` value cited in Gates)"). Nothing
   in the changed text reaches for a Node API above that floor. Pass.
4. **Native modules** — unchanged. The `rust` job installs "nothing … beyond
   the pinned toolchain (A11)"; `cargo-deny` enters pinned, scheduled, and
   non-blocking; no Node native module (no node-gyp/prebuilds) is introduced
   anywhere in the diff. Pass.
5. **Keyboard reality** — untouched; the ASCII/`-`-separated fixture-name
   constraint and the non-ASCII question stay where they were. Pass.
6. **File paths** — untouched. Path canonicalization and `..`/absolute
   rejection stay deferred to ADR-012, which matches ADR-008 clause 2 placing
   "config path discovery, write policy" in `crates/adapters`. No deferral in
   the diff contradicts the ADR. Pass.
7. **Test strategy per supported platform** — untouched. Test #7 plus Rollout 7
   still name the Linux CI runner and a Windows contributor machine, which are
   exactly the platforms the matrix marks supported for this design; macOS and
   SteamOS are `no (v1)`, so no platform test is owed for them. Pass.

Cross-check against the ADR: the rewritten Goals bullet ("CI running both
toolchains' gates, and a Rust section in `repo-practices`") still states ADR-008's
"CI must run both" and clause 4; the rewritten Risks mitigation ("Codec
conformance fixtures pin `toml_edit`'s known non-preservation gaps (A21) when
`crates/adapters` lands (ADR-012 takes the obligation)") keeps the A21 basis and
the ADR-012 hand-off intact. No revision broke a previously approved
compatibility position.

## Findings

(none)

## FYI (no severity)

- Gates — "so it commits to the only recurring look at the lockfile": the
  rewrite drops the object qualifier "it commits", so the lockfile is now
  identified only later in the same sentence ("the committed `Cargo.lock`").
  Compatibility is unaffected — the scheduled job still owns the recurring
  advisory look the ADR's CI consequence needs — flagging only as language-seat
  territory.
