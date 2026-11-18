# Complexity review r1 (c0, full panel)

verdict: request-changes

Scope: whole document (`docs/designs/tasks/002-steam-windows-discovery/02-proposal.md`).
Inputs: the proposal, `docs/tasks/T-002-steam-windows-discovery.md`, the Revision
log (r0 entries only — first pass for this seat). No prior findings of mine
were injected.

## Checklist walk-through

1. **Reuse vs. existing repo code** — pass. The design moves, not rewrites:
   `keyvalues`, `normalize_slug`, `library_paths`, `libraries_in_root_with_skips`,
   `scan_library`, `read_text_file`, `canonical_paths`, `discovered_game`,
   `absolute_env` and the single error path all exist in
   `crates/discovery/src/steam_linux.rs` (verified against the source) and are
   reused by the core. No `steam_common`-like module exists today, and nothing
   in `tools/` (a Node orchestration CLI) is reusable for Rust discovery.
2. **Moving parts trace to Goals** — pass. `SteamCore<T>`/`SteamTarget` → the
   task's no-duplication Done-when; the `&dyn Fn` registry seam + `RegistryKey`
   → the task's injectable-seam clause; the cross-`cargo check` CI step → the
   Done-when Windows type-check clause; two-plugin CLI registration → the
   wiring clause. Nothing is untraceable YAGNI.
3. **30–50 % smaller design?** — considered. The smaller shape (plain
   `steam_common` functions + `platform`/`bool` args) is in *Options
   considered* and rejected with a stated reason; the task itself names "a
   platform-parameterized core that both plugins call" as an accepted shape
   (T-002:32-34), so the chosen size is within the task's own envelope.
4. **New dependencies** — pass. `winreg` is the only one, target-gated, and
   the lock-reuse claim checks out: `Cargo.lock` already pins `windows-sys
   0.61.2` and `windows-link 0.2.1`, `cfg-if` is absent (A10 verified). The
   stdlib has no registry access; the stdlib-ish alternatives (hand-rolled FFI,
   `reg.exe`) were considered and rejected with concrete reasons.
5. **Defer the second half** — pass. *Rollout* lands the refactor alone first,
   then the plugin, then the CLI, then CI: the first merged slice is thin.
6. **Knobs with no consumers / single-implementation abstractions** —
   `SteamTarget` has two implementations, `RegistryKey` has exactly the two
   keys §5 reads, no new config knobs; one public-surface wrinkle is Finding 2.
7. **Value check** — the complexity looks proportionate: one small
   target-gated dependency, one sealed trait with two impls, one private core
   module and one public plugin module, for a feature the task mandates be
   shared rather than forked. The registry seam deliberately reuses the
   `&dyn Fn` shape the Linux probe already has instead of a new trait — the
   right direction.

## Findings

- [major][new] Chosen design §8 + Deferred — "engine.register_plugin(Box::new(SteamLinuxPlugin::from_roots(steam_root.iter().cloned())));" and "*Deferred*: An explicit 'these roots are Windows Steam roots' CLI seam that would emit `steam-windows` rows from any host (§8): no follow-up yet." — this deferral contradicts the task's Done-when: "The `steam-windows` plugin is wired into `apps/discovery-cli` … alongside `steam-linux`, and `--steam-root` keeps working for it" (docs/tasks/T-002-steam-windows-discovery.md:65-67); the sentence's subject is the steam-windows plugin, so the flag must produce rows from it. Under §8, `--steam-root` never registers `SteamWindowsPlugin`, so no `steam-windows` row can ever come from the flag. The proposal's own rebuttal is unsound on its own terms: the assertions it protects by freezing the flag live in `apps/discovery-cli/tests`, while the only tests the task freezes are the Linux plugin's (T-002:34-35), and "a platform tag that is a fiction on a Linux runner" is contradicted by the design's own rule that "the row's platform tag is the plugin's" — a user-supplied root scanned by the Windows plugin means exactly that. Fix: register both plugins on the roots branch — add `engine.register_plugin(Box::new(SteamWindowsPlugin::from_roots(steam_root.iter().cloned())));` beside the Linux one — and update the row-count assertions in `apps/discovery-cli/tests/cli_tests.rs` to the doubled expectation; delete the Deferred bullet. If the author instead reads the clause as "the flag keeps working on Linux", that is a deviation from the task text and belongs in front of the runner as an explicit question, not inside this document's Deferred list.
- [minor][new] Test strategy #2 — "`steam_windows::usable_install_dir` and `steam_linux::usable_install_dir` differ on no other case." — this test needs `steam_linux::usable_install_dir` to be public; it is private today (crates/discovery/src/steam_linux.rs:275), it is not in A15's preserved-surface list, and Non-goals freeze "Any change to `SteamLinuxPlugin`'s … `pub` items". So either §1's preservation claim breaks or the test cannot be written — and the added public item would have no consumer but a differential test, which is public surface the design grows without need. Fix: test the one-rule difference solely through the already-public `parse_manifest` on both sides (the first half of #2, backslash `installdir` → `None` vs `Some`, already establishes it) and delete the `usable_install_dir` clause from #2; keep `usable_install_dir` `pub(crate)` in `steam_common`.

## FYI (no severity)

- Correctness seat: §1's sketch declares `pub fn parse_manifest(text: &str) -> Option<AppManifest>` in the public `steam_windows` module while `AppManifest` lives in the private `steam_common`; the sketch spells the `pub use` re-export only for `steam_linux`. Without a re-export (or a `pub use` in `steam_windows` too), the return type is unnameable outside the crate and the `private_interfaces` lint is a plausible `-D warnings` tripwire.
- Compatibility/correctness seats: the `\\?\` prefix posture (§7) is stated consistently in Chosen design, Compatibility matrix, Risks and Deferred, with A8 as its basis — internally coherent; the judgment call itself is not this seat's.
- Registry-deferral check (step 5): the other Deferred bullets (second VDF location, `REG_EXPAND_SZ`, extra registry keys, `installdir` naming hardening, AppID/state filtering, Deck/macOS) are each consistent with the task's stated requirements and were not re-opened.
