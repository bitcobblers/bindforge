# Compatibility review r1 (pass c3, ratification)

verdict: approve

Scope: `git diff b12a4dd…bc928d9` (five hunks: §3 closing sentence, §5 item 3 and
the registry-spelling paragraph, §8 comment + closing paragraph, Test strategy #3,
Test strategy #13, Rollout #5, Revision log r1 entries), then the full current
document. My r1 verdict carried no findings, so this pass checks only whether the
revisions broke what was approved. The compatibility checklist is re-applied to the
changed text plus the sections it cites (§5 → §1 `absolute_env`, §7; §8 → #12/#13,
Rollout #5; #3 → the `WOW6432Node` matrix row).

Checklist status: (1) matrix — every row has a status, no `unknown`, no empty cell;
unchanged by this diff. (2) OS input APIs — not applicable, this design reads the
registry and the filesystem, it targets no input API. (3) Node version floor — not
applicable (no Node surface); the Rust floor is pinned (A13, `winreg` MSRV 1.60
below the 1.85.0 channel). (4) Native modules — the one dependency is
`[target.'cfg(windows)'.dependencies]` with the lock diff limited to `winreg` +
`cfg-if` (§9, A10); unchanged. (5) Keyboard reality — not applicable. (6) File
paths — per-OS defaults via env vars and `PathBuf` joins, case/separator reality
delegated to `std::fs::canonicalize` (§5, §7, A8); verified `absolute_env`
(crates/discovery/src/steam_linux.rs:338-342) and `canonical_paths`
(crates/discovery/src/steam_linux.rs:322-334) exist as §1's "moved verbatim" list
claims. (7) Platform test per supported matrix row — Linux rows: #9/#10/#11;
Windows compile: #12; Windows runtime: #13. The diff *strengthens* item 7 (#13
gains the `--steam-root` run and is named as the roots-branch gate), and §8's and
Rollout #5's cross-references to it are consistent.

Verified against the code: `apps/discovery-cli/src/main.rs:33-38` is today's
single-plugin selection, so §8's revised claim that "the Linux binary's roots
branch keeps T-001's one-plugin shape" holds, and the three CLI tests all drive
`list --steam-root` (apps/discovery-cli/tests/cli_tests.rs:40, :79, :146) —
i.e. the roots branch Linux keeps, so they stay untouched.

## Findings

- [minor][new] *Test strategy* #3 — "`Unit (`steam_windows::host_candidates`, pure, fake registry + fake env)` … `both views answering (one path from both → one root after the canonical-path rule, distinct paths → two roots and rows from two installs)`" — the newly added case asserts outcomes at a layer that cannot produce them: §5 defines `host_candidates` as "pure — registry and environment are lookups, no filesystem access", so it never canonicalizes and can only report that both values were appended in §5's order; the collapse-to-one-root and the "rows from two installs" outcome belong to `from_registry_and_roots`/`SteamCore::with_roots`. The second half additionally needs two *valid* Steam roots, while the fixture trees are `crates/discovery/tests/fixtures/steam/minimal` and `…/steam/all-broken` (the latter exists to produce the error case, #6) and #5 states "no new fixture tree is introduced". This matters to compatibility because #3 is the named evidence for the matrix row "Windows x64 whose Valve key exists only under `WOW6432Node` | Yes", which currently rests on an unrunnable assertion. Fix: in #3 keep only "both views answering → both values appended, `LocalMachineValveSteam32` before `LocalMachineValveSteam64`, not deduped"; move the "one path from both → one root" collapse to #4 (`from_registry_and_roots`, fake registry pointing both views at the same fixture root) and delete "distinct paths → two roots and rows from two installs" rather than adding a fixture tree.
- [nit][new] *Chosen design* §5 — "Registry values arrive in mixed case and either slash form (A3, A4)" — the citation pair no longer supports the sentence it cites: A3 records the opposite for the HKCU value ("stored lowercase with forward slashes") and A4 says nothing about case or slash form. The r1 A1b entry took the right step for ACF `path` values ("no spelling is assumed"); this sentence replaced one asserted spelling with a broader one instead of dropping the assertion. Behaviour is unaffected — no normalization happens there and §7's canonical-path rule absorbs either form — so this is wording only. Fix: "Registry values arrive in whatever case and slash form the installer wrote (A3 records one observed spelling; no spelling is assumed) — the canonical-path rule is what turns both into one comparable spelling (§7)."
- [nit][new] *Test strategy* #13 — "the only gate for §8's roots branch" overstates against §8's own "the Windows branch is type-checked by #12 and run only by #13", since #12's `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` compiles that `#[cfg(windows)]` branch. Fix: "the only *run* gate for §8's roots branch (#12 only type-checks it)".

## FYI (no severity)

- §8's host-split means a Linux runner can never emit `steam-windows` rows, so the task's ":65-67" `--steam-root` clause is answered only on a Windows host; that is a deliberate Revision-log decision (r1 `--steam-root`, supersedes r0: q8) and this revision improved it by pinning #13 as its run gate — not re-opened here, but the correctness seat may want the task-clause reading on record.
- #13's three pasted commands assume a `discovery-cli` binary built on the Windows machine; the Non-goals exclude a Windows build/release pipeline, which is consistent, but the build step for the manual gate is left implicit for the build notes.
