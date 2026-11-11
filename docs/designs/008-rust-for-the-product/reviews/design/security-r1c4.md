# security review r1c4

verdict: approve

Scope: `git diff 796b2a63..67a6a1a3` of `02-proposal.md` (assumption register A27, Gates,
TypeScript boundary, Rollout 4, Deferred, Revision log), then the full current document.
Inputs: the proposal, `docs/adrs/008-rust-for-the-product.md`, the Revision log, my r1
findings. Checklist applied: `references/security.md` only.

## Closure check (my r1 major)

Closed as specified. The unowned `Deferred` bullet ("no follow-up yet") is gone; the
obligation is now owned inside Gates — `.github/workflows/advisories.yml`, "runs weekly and
on manual dispatch, never as a PR check, installing `cargo-deny` at a pinned version and
running `cargo deny check advisories` against the committed `Cargo.lock` and a root
`deny.toml`; the repo maintainer triages what it reports within the week" — with a landing
step (Rollout 4), an owner, a cadence, and a waiver trail. The TypeScript boundary now says
the desktop design "pins the newest `tauri` release available when it lands ... and checks
Tauri's own repository advisory list", and A27 records the crates.io/OSV gap with the
GHSA-w28w-mhc8-qvjv evidence. I re-verified that evidence: the advisory is real (High, CVSS
7.6, no CVE, published 2026-09-26, `>= 2.0.0, <= 2.11.5`, patched `>= 2.11.6`), the
crates.io/OSV record set for `tauri` contains no entry for it, and the crates.io record for
the older one (GHSA-7gmj-67g7-phm9, CVE-2026-42184) fixes at 2.11.1. A27 stands. Not
re-raised. The three findings below are defects in the text that closed it, not re-openings.

## Findings

- [minor][new] Chosen design → TypeScript boundary (clause 1; canonical) — "the desktop
  design pins the newest `tauri` release available when it lands in
  `[workspace.dependencies]` (never below 2.11.1)" — the closure dropped the stale
  *justification* but kept the stale *number* as the floor. A27 in the same document says
  GHSA-w28w-mhc8-qvjv is patched only at 2.11.6, so the floor explicitly permits
  2.11.1–2.11.5, and that advisory's mechanism is exactly the control this paragraph
  assumes: `plugin:__TAURI_CHANNEL__|fetch` is carved out of Tauri's ACL rejection branch,
  so a renderer can pull queued IPC payloads belonging to another webview by guessing the
  old monotonic channel id — an app-wide queue, no capability check, no webview binding. A
  floor exists to catch downgrades, and this one sits below the patch for the one High-severity
  IPC advisory the paragraph cites. Separately, "`the newest tauri release available`" is
  ambiguous on the day it is read: the newest published `tauri` on crates.io right now is
  `3.0.0-alpha.6`, while the newest stable v2 is `2.12.5`; ADR-008 clause 2 fixes "Tauri v2",
  so a literal reading pins an unreleased major's alpha. Fix: change the parenthetical to
  "(never below 2.11.6, the patch A27 names)" and qualify the pin as "the newest **stable
  v2** release available when it lands"; this replaces the 2.11.1 floor rather than adding a
  second constraint.

- [minor][new] Chosen design → Gates (ADR-008 clause 4) — "its crates.io/RustSec data is not
  every dependency's only advisory source (A27, TypeScript boundary)" — this overstates what
  the new control reads. cargo-deny's advisory source is one or more advisory-database git
  repos (`[advisories] db-urls`, default `RustSec/advisory-db`); it does not read crates.io
  or OSV data at all, which is the comparison A27 is built on. Verified: the RustSec DB's
  `crates/tauri` directory holds only `RUSTSEC-2022-0088` and `RUSTSEC-2022-0091`, i.e. no
  v2 advisory — neither GHSA-w28w-mhc8-qvjv nor the 2.11.1 fix GHSA-7gmj-67g7-phm9. So the
  weekly job is blind to the entire v2 tauri advisory class, including the advisory whose
  version number the paragraph above quotes; the design's conclusion (check Tauri's own list)
  is right, but it is justified by data the named control never sees. Fix: in that sentence
  name the source — "the RustSec advisory DB it clones (`db-urls`) carries no `tauri` v2
  advisory at all, so framework advisories come from Tauri's own list" — so no reader
  believes the scheduled job enforces a tauri version floor.

- [minor][new] Chosen design → Gates — "a `deny.toml` `skip` names the advisory id it waives"
  — `skip` is not cargo-deny's advisory-waiver key: waivers are `[advisories] ignore`
  entries (`{ id = "...", reason = "..." }`, optionally `expiry = "P90D"`, plus
  `unused-ignored-advisory` to retire stale waivers); `skip` is the `[bans]` key. An
  implementer who writes `skip` under `[advisories]` gets a config error rather than a
  waiver, and the "auditable waiver" property the sentence promises is not what the named
  key provides. Fix: one rename plus the two fields that make a waiver self-limiting —
  "a `deny.toml` `[advisories] ignore` entry names the advisory id it waives, with a
  `reason` and an `expiry`".

- [nit][new] Chosen design → Gates — "installing `cargo-deny` at a pinned version" — the
  version is pinned, but `cargo install` without `--locked` re-resolves the tool's own
  transitive dependencies on every weekly run, so this workflow is the one place in CI that
  pulls freshly resolved registry code that never appears in the repo's `Cargo.lock` — the
  exact artifact the rest of the design freezes with `--locked` (Workspace shape, Risks row
  1). Fix: say "`cargo install cargo-deny --locked --version X.Y.Z`", replacing the bare
  "at a pinned version" wording.

## FYI (no severity)

- Checklist item 5, verified for every dependency the changed text names or newly implies:
  `cargo-deny`, `serde`, `serde_json`, `clap`, `axum`, `toml_edit`, `wry`, `tauri-runtime` —
  no advisories in the OSV crates.io index. `sqlx`: one advisory (RUSTSEC-2024-0363 /
  GHSA-xmrp-424f-vfpx) fixed at 0.8.1, and the design's A3 floor is 0.9 — clear. `tauri`:
  every record is version-bounded and patched, and the newest stable v2 (2.12.5) is above
  every patch level, so no known-unfixed advisory blocks the design.
- Residual, not a finding because it is a deferral and does not contradict ADR-008: after
  the desktop crate lands, nothing recurring checks Tauri's own advisory list — the desktop
  design checks it once, at pinning time — and the design has now established that this
  class is invisible to its only automated control. If the reviser wants a recurring hook,
  the cheapest is one line in the desktop design's obligations, not a new mechanism here.
- The scheduled job has an owner and a cadence but no delivery path for its output: a weekly
  workflow that goes red is silent unless someone opens the Actions tab. Worth a sentence in
  the desktop-era or hub-era CI work; flagging for the operations/maintainability seat, not
  as a security gate.
- Rollout ordering: `Cargo.lock` is committed at Rollout 1 and `advisories.yml` arrives at
  Rollout 4, so the first three steps have no advisory coverage. Immaterial here — the
  dependency set at that point is `serde` alone and no product code exists — noting only so
  a reshuffled rollout keeps the ordering.
