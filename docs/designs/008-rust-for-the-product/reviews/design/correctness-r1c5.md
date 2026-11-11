# correctness review r1c5

verdict: approve

Scope: `git diff 796b2a63 67a6a1a3` on `02-proposal.md` (A27 row, Gates paragraph,
TypeScript boundary, Deferred bullet removal, Rollout 4, Revision log r1c3 lines),
then the full current document. Checked against `docs/adrs/008-rust-for-the-product.md`
and the Revision log. Spot-checks of the changed text's external claims: the GHSA
`w28w-mhc8-qvjv` repository advisory on `tauri-apps/tauri` exists with severity
high, CVSS 7.6, no CVE, published 2026-09-26, `tauri >= 2.0.0, <= 2.11.5`, patched
`>= 2.11.6`; the OSV crates.io/`tauri` set has 9 entries, the newest v2 one being
`GHSA-7gmj-67g7-phm9` fixing at 2.11.1 and none fixing at 2.11.6 — **A27 is
verified as written**, and no other changed text contradicts ADR-008.

## Findings

- [minor][new] Chosen design → Gates — "installing `cargo-deny` at a pinned version
  and running `cargo deny check advisories`" — the run is defined inside a checkout
  whose `rust-toolchain.toml` fixes `channel = "1.85.0"` (Workspace shape, unchanged),
  and rustup's toolchain-file override applies to every cargo invocation in that
  tree, including `cargo install`. Every current `cargo-deny` release (0.19.x–0.20.2,
  crates.io `rust_version`) declares MSRV 1.88.0, so the pinned release the job
  installs cannot build under 1.85.0 — and the design's only bump conditions for the
  channel ("a reviewed dependency upgrade or a published toolchain security advisory")
  do not authorize raising the pin for a dev tool that is not a workspace dependency.
  A25 covers the product graph only, so the new obligation's toolchain need is
  undecided anywhere. Fix: in the Gates sentence, name the job's own toolchain —
  "installs `cargo-deny` at a pinned version under a `rustup`-installed toolchain at or
  above that release's `rust-version` (the `check.yml` 1.85.0 pin is not used by this
  job)" — and add one assumption-register row for the pinned `cargo-deny` release's
  MSRV; leave `check.yml` and the workspace floor alone.

- [minor][new] Chosen design → Gates — "That same run is the trigger for the
  `rust-toolchain.toml` security-advisory bump (Workspace shape)" — the run cannot
  observe that event. `cargo deny check advisories` matches the `Cargo.lock`
  dependency graph against the RustSec advisory DB, which carries advisories for
  crates published on crates.io; the toolchain is not a `Cargo.lock` package, so a
  published *toolchain* security advisory — the second bump condition Workspace shape
  allows — never appears in its output. The same sentence concedes the DB is
  incomplete for dependencies while assuming it is complete for the toolchain. Fix:
  decouple the two, e.g. "that same weekly cadence is when the maintainer checks the
  Rust security announcements for a `rust-toolchain.toml` bump (Workspace shape); the
  run itself only reports `Cargo.lock` crate advisories."

- [minor][new] Chosen design → Gates — "a `deny.toml` `skip` names the advisory id it
  waives" — `skip` is not an advisory-waiver key. In cargo-deny the advisories check
  waives through `[advisories] ignore` (advisory id, `{ id = "RUSTSEC-…", reason =
  … }`, or `crate@version`); `skip`/`skip-tree` belong to `[bans]` and take crate
  names, and cargo-deny flags unused/unknown advisory-section config. Rollout 4
  creates the `deny.toml`, so this names the mechanism the step writes. Fix: replace
  "`skip`" with "`ignore`" in that clause (optionally the `{ id = …, reason = … }`
  form so the waiver carries its reason); no other change.

- [nit][re-raised] Test strategy #5 vs Chosen design → Gates — "every command listed
  in the Gates block succeeds on the Linux CI runner, with nothing installed beyond
  the pinned toolchain" — the referent has widened since r1: the Gates section now
  also contains `cargo deny check advisories`, which the same paragraph says "needs a
  tool outside the pinned toolchain", and `npm ci && npm run check`. Read as the
  section, #5 now demands commands the section itself excludes from that promise; the
  Revision log's claim that #5 "cannot drift from the gate" no longer holds for the
  section's added content. Fix (unchanged from r1): name the fence — "every command in
  the Gates code fence (`cargo fmt`/`clippy`/`test`) succeeds …".

- [nit][new] Revision log → "Chosen design → Gates (r1, r1c3)" — "advisory scanning
  stays out of it (unpinned tool outside the rustup toolchain, live advisory DB)" —
  the retained "unpinned tool" reason is now false in the same breath as the entry's
  r1c3 clause: the body commits to "installing `cargo-deny` at a pinned version". The
  surviving reasons (outside the rustup toolchain; live DB) stand on their own. Fix:
  drop "unpinned" from that parenthetical.

- [nit][new] Chosen design → TypeScript boundary — "checks Tauri's own repository
  advisory list — the crates.io/RustSec data the scheduled `cargo deny` scan reads
  does not carry every v2 advisory (A27)" — the new obligation is one-time (at
  desktop-design landing) and has no home after that: it is not in the weekly job
  (A27 says that job is blind to exactly these advisories), not in Rollout 5's
  `repo-practices` rules, and not in the Deferred bullets that name what the desktop
  design owns ("Webview UI framework, `apps/desktop` npm toolchain, and its exact
  workspace-members path"). Fix: one clause on that Deferred bullet — "…and the
  recurring check of Tauri's own repository advisory list — owned by the desktop
  design".

## FYI (no severity)

- Two r1 findings are untouched by this revision and remain exactly as filed: the
  toolchain file's missing `components = ["rustfmt", "clippy"]` (Workspace shape,
  `channel = "1.85.0"`, feeds Tests #5/#7), and the Compatibility-matrix Postgres
  pointer "server version pinned and tested by ADR-014's strategy, see Deferred"
  whose target ADR-014 Deferred bullet still does not list Postgres server-version
  pinning. Neither is broken by the r1c3 text, so neither is re-counted here.
- Removing the Deferred bullet for advisory scanning left no dangling reference
  anywhere in the document (Rollout 4 and the Revision log both point back at Gates) —
  that part of the revision is internally clean.
