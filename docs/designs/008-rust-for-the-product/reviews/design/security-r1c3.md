# security review r1c3

verdict: request-changes

Ratification pass c3. Inputs read: the named diff (`6e4ea21` → `796b2a6`, blob
ids of `02-proposal.md`), the full current `02-proposal.md`,
`docs/adrs/008-rust-for-the-product.md`, the Revision log, and my own r1
findings. No review report or transcript was opened. Advisory state for every
dependency named in the design was re-checked (OSV/crates.io for `serde`,
`clap`, `toml_edit`, `axum`, `sqlx`, `tauri`, `tower`; Tauri repository
advisory list): no dependency in the design's stated version range carries a
known **unfixed** advisory, so checklist item 5 is not a blocker on its own
(`sqlx` RUSTSEC-2024-0363 is fixed at 0.8.1, design takes 0.9; Tauri
CVE-2026-42184 / GHSA-7gmj-67g7-phm9 is fixed at 2.11.1, matching the floor).
The untrusted-input, path-handling, secrets/PII and fail-closed items were not
touched by this diff; the changed text does not contradict what r1 approved
there.

## Findings

- [major][regression] Chosen design → "Gates (ADR-008 clause 4)" — "Per-PR
  dependency-advisory scanning is deliberately excluded from the gate: it needs
  a tool outside the pinned toolchain and an advisory database that is live
  network state" (and Deferred: "Dependency-advisory scanning (`cargo deny`, a
  root `deny.toml`) — no follow-up yet") — the text I approved at r1 carried a
  standing dependency-advisory control (`cargo deny check advisories` run
  against the committed `Cargo.lock`, with `deny.toml` created in Rollout 1);
  this pass deletes it from the gate, from Rollout 1, and from the toolchain
  narrative, and hands the obligation to an entry that names no owner and no
  cadence. Every other Deferred bullet names the document that inherits its
  concern (ADR-009/012/014, the desktop design); this one says "no follow-up
  yet", so the run now contains no mechanism and no party that ever looks at
  advisories for a lockfile this design commits and freezes with `--locked`.
  The design also still assumes advisory knowledge exists: `rust-toolchain.toml`
  is bumped when "a published toolchain security advisory requires it" (Workspace
  shape), and the TypeScript boundary's numeric floor is justified as the
  "current fix for the IPC-origin advisories". That justification is already
  stale on the framework ADR-008 fixes: Tauri GHSA-w28w-mhc8-qvjv ("Improper
  Tauri IPC Access Control for Fetch Command", high, CVSS 7.6, published
  2026-09-26) covers `>= 2.0.0, <= 2.11.5` and is patched only at **2.11.6**,
  above the 2.11.1 floor this design publishes, and it exists only as a
  Tauri-repository advisory — absent from crates.io OSV/RustSec, so the deleted
  `cargo deny` gate could not have surfaced it either. Net effect of the
  closure: the automated half of the r1 dependency-advisory posture is removed,
  the human half was never added, and the r1 closure of the floor question was
  left resting on a numeric constant that has since moved. Label: regression —
  closed in the r1 text I approved, gone in changed text. Fix: the objection to
  the *placement* is sound, so keep advisory scanning out of the three-command
  per-PR gate, but replace "no follow-up yet" with an owner: rewrite the
  Deferred bullet to name a scheduled (non-PR-blocking) workflow running
  `cargo deny check advisories` against the committed lock plus a root
  `deny.toml`, and add one clause to TypeScript boundary — "the desktop design
  pins the newest `tauri` release available when it lands and checks the
  framework's own security-advisory list, not only crates.io". No new tooling;
  if the scheduled job is rejected, the bullet must instead name who checks and
  on what trigger — the sentence replaces the unowned deferral rather than
  adding to it.

## FYI (no severity)

- Gates / Test strategy #5: the new phrase "nothing installed beyond the pinned
  toolchain (A11)" plus "every command listed in the Gates block succeeds"
  makes the gate's self-containment claim depend on `rust-toolchain.toml`
  (currently `channel = "1.85.0"`, no `components` line) supplying `clippy` and
  `rustfmt`; a contributor or runner on a minimal rustup profile would fail the
  gate for a non-security reason. Correctness/repo-practices seat's call.
- Compatibility matrix: the Postgres row now points at "ADR-014's strategy" for
  where the server version is pinned and tested — consistent with the ADR
  (Postgres is hub-only); no security obligation moved out of the run there.
- Unchanged since r1 and therefore not re-filed here: the `Cycle` /
  `ChainTooDeep` naming gap for the 1 MiB / 16-level parse budget (Adversarial
  guard), the enumerated-only Tauri plugin ban, and the `tauri` floor wording.
  The staleness evidence for the floor is folded into the finding above because
  the gate removal is what left it unpoliced.
