# correctness review r1 (ratification pass c3)

verdict: approve

Scope: `git diff 6e4ea214d807aebc3faf5a2e28068a7b30b54411 796b2a63f1997540de4eb8a6c4288cacd77ce505`
(Gates, Compatibility matrix Postgres row, Test strategy #5/#7, Rollout 1/7, Deferred,
Revision log), then the full current document. Inputs: `02-proposal.md`,
`docs/adrs/008-rust-for-the-product.md`, the Revision log, my r1 findings.

## Checklist pass

1. **Assumption register** — no register row changed in this diff. The one new
   inference drawn in changed text ("nothing installed beyond the pinned toolchain")
   leans on A11, whose basis (rustup toolchain-file page) covers pinning but not
   component availability; checked against that page directly (finding 1).
2. **Source-document fidelity** — the removal of `cargo deny check advisories`
   does not contradict ADR-008: clause 4 fixes the gate as "`cargo clippy` +
   `cargo fmt` as the `check` gate ... tests under each crate", and no ADR clause
   requires advisory scanning. The new Deferred entry ("Dependency-advisory
   scanning (`cargo deny`, a root `deny.toml`) — no follow-up yet; excluded from
   the per-PR cargo gate (see Gates)") is a consistent deferral, not a
   re-openable one.
3. **Internal contradictions across the revised text** — deny removal is clean:
   Rollout 1 no longer creates `deny.toml`, the gate code fence holds exactly the
   "three gate commands" the Gates prose counts, Test #5 cites the block instead
   of restating it, and no other section still references `cargo deny`.
   Test #7 ↔ Rollout 7 cross-reference is mutual and consistent (Windows leg
   scheduled; Linux leg is the fresh-clone check). Postgres row: finding 2.
4. **Regression check on my r1 findings** — none of the three locations moved:
   Test #1 and the harness carve-out sentence are byte-identical to what I
   approved, the Non-goals bullet is unchanged, and Rollout 5's "the Node floor"
   is unchanged. Nothing in this revision broke or falsified them, so per the
   ratification rule they are not re-reported here; they remain open exactly as
   recorded in r1.

## Findings

- [minor][new] Chosen design → Gates / Test strategy #5 — Gates: "`rust` (the
  three gate commands above, and nothing installed beyond the pinned toolchain
  (A11))"; #5: "every command listed in the Gates block succeeds on the Linux CI
  runner, with nothing installed beyond the pinned toolchain" — the new
  "nothing beyond the pinned toolchain" promise is only true if the pinned
  toolchain actually carries `rustfmt` and `clippy`, and the toolchain file this
  design fixes (Workspace shape: "`rust-toolchain.toml` with
  `[toolchain] channel = \"1.85.0\"`", which Rollout 1 copies verbatim) requests
  no components. rustup installs a toolchain-file toolchain with the machine's
  configured profile — the rustup book states the toolchain file's `profile`
  setting falls back to a possibly-changed default and its profiles page
  explicitly recommends the `minimal` profile "in CI" (and for Windows);
  `minimal` ships only `rustc`, `rust-std`, `cargo`, so `cargo fmt` and
  `cargo clippy` fail with "no such command" on such a clone. That makes #5's
  one-gate promise, and #7's Windows/contributor verification leg, depend on an
  environment setting the design never decides — and it is precisely the
  configuration the tool's own docs recommend. Fix: make the components explicit
  in the Workspace shape toolchain file — `[toolchain] channel = "1.85.0"
  components = ["rustfmt", "clippy"]` — which the `components` setting installs
  additively regardless of profile; no other text needs to change.
- [nit][new] Compatibility matrix (Postgres row) vs Deferred — "server version
  pinned and tested by ADR-014's strategy, see Deferred" — the sentence now
  points at `## Deferred / out of scope`, but that section's ADR-014 bullet
  enumerates only "Hub DB schema, migrations, upload policy, auth, voting, plus
  the sqlx offline-cache mechanism and the hub-path `cargo sqlx prepare --check`
  job"; pinning and testing the Postgres server version is not among the
  obligations handed over, so the new pointer does not resolve. The previous
  row asserted the same ownership without invoking Deferred, so the dangling
  cross-reference is created by this revision. Fix: add "Postgres server version
  pinning and its test" to the ADR-014 Deferred bullet (one clause), keeping the
  matrix wording as is.
- [nit][new] Test strategy #5 vs Chosen design → Gates — "every command listed in
  the Gates block succeeds on the Linux CI runner, with nothing installed beyond
  the pinned toolchain" — "the Gates block" is new wording (the r1 text spelled
  the three commands out). The Gates *section* also lists `npm ci && npm run
  check` for the `orchestration` job, so read as the section, #5 requires the npm
  commands to run "with nothing installed beyond the pinned toolchain", which the
  same Gates paragraph contradicts by pinning a Node version in a `setup-node`
  step. Fix: name the fence, not the section — "every command in the Gates code
  fence (`cargo fmt`/`clippy`/`test`) succeeds …".

## FYI (no severity)

- Rollout 5 still asks the `repo-practices` Rust section to record "the Node
  floor" while this design decides only "Node version pinned explicitly in a
  `setup-node` step" — my r1 nit, untouched by this revision, so not re-filed.
- The revision-log entry for the Gates change reads as a decision record rather
  than a finding closure; the removal itself is ADR-consistent, so no rebuttal to
  audit.
