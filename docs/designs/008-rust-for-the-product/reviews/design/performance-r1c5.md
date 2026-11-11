# performance review r1c5

verdict: approve

Scope: the diff `796b2a6..67a6a1a` of `02-proposal.md` (Assumption register row
A27, `Chosen design → Gates`, `Chosen design → TypeScript boundary`, Rollout 4,
the removed `Deferred` line, and the Revision log), read against the full
current document and `docs/adrs/008-rust-for-the-product.md`. Prior performance
verdict held no findings, so this pass checks only for regressions introduced by
the r1c3 revisions.

## Findings

None. The revisions did not break anything the performance seat approved.

## Checklist evaluation (changed text only)

1. **Input-path latency / per-event cost** — untouched and still clean. The
   Context section's "BindForge reads and writes binding *files* only ... it
   never reads or injects input events" is unchanged, so no keystroke/event path
   exists for the revisions to add cost to. Nothing in the r1c3 text adds
   allocation, table rebuild, or synchronous IO to any event path.
2. **Structure costs (O(n²) over bindings × devices × platforms)** — unchanged
   by this diff; no new structure or traversal is introduced.
3. **Memory growth / bounded tables and caches** — unchanged. The parse budget
   ("rejects documents larger than 1 MiB or nested deeper than 16 table/array
   levels") and `MAX_CHAIN_DEPTH = 32` are byte-identical to what r1 approved.
4. **Startup cost (file reads, schema parsing at boot)** — unchanged. The new
   `.github/workflows/advisories.yml` and root `deny.toml` are CI-side
   artifacts; they add no product binary startup work.
5. **Caching and invalidation** — the diff touches only the *advisory* data
   plane, and it handles invalidation soundly: the scan compares the committed
   `Cargo.lock` against a database the design itself names as "live network
   state", which is exactly why it stays out of the per-PR gate ("it can turn an
   unchanged PR red on any day the database moves") and why the new job is
   weekly/manual-dispatch, "never as a PR check". The PR gate is still exactly
   the three `cargo fmt` / `clippy` / `test` commands, so the revisions add zero
   contributor-loop latency and no nondeterminism to the blocking path.
   No cache is added whose staleness could be hidden.
6. **Research baseline vs expected numbers** — the changed sections carry no
   performance numbers or expected timings, so there is no figure in the revised
   text that could diverge from the research baseline. (Baseline itself was not
   re-read: `01-research.md` is outside the inputs this pass was given.)

## Regression check on the revised passages

- **Gates** — "That obligation is not deferred — this design is the document
  that creates the repo's CI, so it keeps the only recurring look at the
  lockfile it commits". Performance-positive, not a regression: the recurring
  look is scheduled off the PR path and pinned to a versioned tool, so no new
  blocking work or unpinned install enters the gate. Consistent with ADR-008
  clause 4, which fixes only `cargo clippy` + `cargo fmt` as the check gate and
  requires CI to run both toolchains' gates — a non-blocking advisory job
  neither contradicts the Decision nor moves work onto the per-PR path.
- **TypeScript boundary** — "pins the newest `tauri` release available when it
  lands in `[workspace.dependencies]` (never below 2.11.1)" replaces a fixed
  floor with a land-time pin. No performance consequence: the committed
  `Cargo.lock` plus `--locked` (A8, A11) keeps the resolved graph
  deterministic either way, and the Risks row "MSRV drift ... floor bump is a
  deliberate workspace change" already owns the cost of moving a version.
- **Assumption register A27** — a new advisory-coverage fact. It is not a
  workspace-shape, dependency-version, or gate-cost statement, so it changes no
  performance assumption; it is also consistent with the ADR, which fixes the
  framework choice but not the patch level.
- **Rollout 4** — adds `advisories.yml` and `deny.toml` to a step that already
  lands the CI config; no additional gate command, so Test strategy #5's "every
  command listed in the Gates block succeeds ... with nothing installed beyond
  the pinned toolchain" still describes the three-command `sh` block and stays
  satisfiable.
- **Deferred** — removing the unowned `cargo deny` line and taking the
  obligation here narrows the deferral list; the remaining entries are
  unchanged and none contradicts ADR-008's Decision.

## FYI (no severity)

- Gates / Test strategy #5 — #5's "nothing installed beyond the pinned
  toolchain" and #6's gate-independence test both describe the three-command
  gate; the `advisories.yml` job deliberately installs `cargo-deny`, so the two
  are only reconcilable if #5 is read as scoped to the `sh` block. Correctness /
  test-strategy seat's call whether to pin that wording.
- Gates — weekly cadence against a lockfile whose churn the Risks table rates
  `Med` leaves up to a week of unscanned dependency upgrades, and A27 says the
  scanned OSV data is not every dependency's only advisory source. Detection
  latency, not runtime cost — security seat's call.
- Gates / Revision log — the maintainer triage obligation ("triages what it
  reports within the week") is recurring human process; complexity seat may want
  to confirm it is bounded.
