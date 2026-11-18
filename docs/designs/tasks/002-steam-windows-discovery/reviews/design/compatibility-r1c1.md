# compatibility review r1 (c1 dissent pass)

verdict: approve

Scope: `git diff d5bafff8 b12a4dd7` on `02-proposal.md`, then the full current
document. Checklist: `references/compatibility.md`. Inputs: the proposal, the
task text, the Revision log, my r1 findings.

## Findings

- [resolved][re-raised → closed] Compatibility matrix — "Windows, 32-bit (x86)
  host | Partial" / "Windows, ARM64 host | Partial" — my r1 major asked for
  `unknown` → `Partial` with the caveat moved into the row text; the revision
  adopts exactly that ("HKCU candidate and the scan work (A5); the HKLM
  64-bit-view query (A20) and the absent `%ProgramFiles(x86)%` fallback are
  unexercised" / "Same code path as x64 (A4's model); not exercised in this
  run"). No `unknown` cell remains in the matrix; checklist rule 1 satisfied.
  Closed.
- [resolved][re-raised → closed] §8 CLI wiring — "host-aware on the roots
  branch only" — my r1 major asked for a target-aware roots branch with
  `#[cfg(windows)]` feeding `steam_root` to `SteamWindowsPlugin::from_roots` on
  Windows while Linux keeps T-001's branch, and for deleting the
  "host-agnostic — no `#[cfg]` in `main.rs`" sentence. The revision does all of
  it; the stale sentence is gone, the Options entry now rejects only the
  every-host variant, the Deferred entry that handed off the task's
  "`--steam-root` keeps working for it" clause (task :65-67) is deleted, and
  the Revision log records the r1 decision. Verified against the repo:
  `apps/discovery-cli` is a workspace member (`members = ["crates/*", "apps/*"]`,
  Cargo.toml:12), so *Test strategy* #12's `cargo check --target
  x86_64-pc-windows-msvc --all-targets` really type-checks the new Windows
  branch, and main.rs:33-38's current roots branch is what the
  `#[cfg(not(windows))]` arm preserves byte-for-byte for the three CLI tests.
  Closed.
- [resolved][nit → closed] Cross-compile row — "Partial (local only)" — both
  remedies I offered were taken: the row is `Partial (local only)` and Rollout
  #4 names a one-time local `cargo check --target x86_64-pc-windows-gnu` as
  that row's lone gate, cross-referenced from #12. Closed.

## Changed-text regression scan (new material since r1)

- Three-variant `RegistryKey` (one (key, view) per reader call, ordering moved
  into pure `host_candidates`): matrix row "HKLM is queried with the 32-bit
  view first, the 64-bit view second (§3, §5)" stays true; both views always
  queried is safe because the canonical-path rule collapses equal paths (§5),
  and the 32-bit-host A20 exposure is now explicitly stated in that row's
  Partial caveat. Test strategy #3 and #8 were updated to the three variants in
  lockstep with §5's named order. No break.
- A1 split into A1/A1b: the "assumes no spelling" claim is verified against
  code — the reader keeps a lone `\` verbatim (crates/discovery/src/keyvalues.rs
  :197-200) and unescapes `\\`/`\"` (:186-195), so both spellings arrive
  usable either way. No break.
- §1's move list gains `absolute_env` and §5 cites it "shared per §1": the
  helper exists at crates/discovery/src/steam_linux.rs:338-342. Consistent.

No open blocker or major findings. Checklist rules 2-6: unchanged in scope and
previously satisfied (registry APIs named per platform; no Node/native/keyboard
surface; paths via `PathBuf`/`canonicalize`/`absolute_env`). Rule 7: the
platforms claimed `Yes` (Windows x64 variants, Linux, CI) still have named
gates (#12, #13, #9, #11); the downgraded rows (`Partial`) no longer claim
support that lacks a gate — which was the point of the gnu fix.

## FYI (no severity)

- Matrix 32-bit row's "(caveat in *Risks*)" pointer is loosest-match — the
  Risks table's "no Windows runner" row names A17/A18, not A20; the caveat is
  nonetheless fully stated in the row itself and in the A20 register entry, so
  nothing is lost. Another seat may care about cross-reference tidiness.
