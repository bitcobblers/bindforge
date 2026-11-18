# complexity review r1 (dissent pass c1)

verdict: request-changes

Scope: `git diff d5bafff8 b12a4dd7` of `02-proposal.md` (Assumption register A1/A1b + A19,
Options gating + roots-registration bullets, §1 shared list/`name()`, §3–§5 registry reader and
`RegistryKey`, §8 CLI wiring, Compatibility matrix rows, Test strategy #2/#8/#12, Rollout #4,
Deferred, Revision log) plus the full current document. Inputs used: the proposal,
`docs/tasks/T-002-steam-windows-discovery.md`, the Revision log, my quoted r1 findings, and
read-only code checks (`crates/discovery/src/steam_linux.rs`, `apps/discovery-cli/src/main.rs`,
`apps/discovery-cli/tests/cli_tests.rs`).

Status of my round-1 findings:
- minor (test #2 requiring a public `steam_linux::usable_install_dir`) — **closed**. Test #2 now
  pins the difference through `parse_manifest` on both sides, and `usable_install_dir` stays
  private at `crates/discovery/src/steam_linux.rs:275`; moving `absolute_env` into
  `steam_common` is likewise surface-neutral (private today at `steam_linux.rs:338`).
- major (deferral contradicted the task's `--steam-root` Done-when) — **not re-raised as such**:
  the Deferred bullet is gone and §8 now feeds the flag to `SteamWindowsPlugin` on Windows. The
  rebuttal's factual core checks out: all three CLI tests pass `--steam-root` at
  `crates/discovery/tests/fixtures/steam/minimal` and assert exactly four rows
  (`cli_tests.rs:68-72`, `:91`) plus `"SteamLinux"` (`:124`), so registering both plugins on that
  branch really does double every assertion the task freezes at docs/tasks/T-002:34-35. What the
  revision left behind is a different defect, reported as new below.

## Findings

- [major][new] §8 — "`#[cfg(windows)] engine.register_plugin(Box::new(SteamWindowsPlugin::from_roots(steam_root.iter().cloned())));`" — the branch that the Options bullet now cites as where "the task's ':65-67' clause is met" (docs/tasks/T-002-steam-windows-discovery.md:65-67) is executed by no gate this design names. On Linux it is compiled out, so *Test strategy* #9/#10 cannot reach it; #12 only type-checks it (`cargo check --all-targets`, which compiles the bin but runs nothing); and #13's manual Windows check enumerates only "the default install, any secondary library, the `\\?\` prefix in `install_path`, and a non-default install path from the registry" — no `--steam-root` invocation — and *Rollout* #5 points at the same list. So the revision replaced a Deferred bullet with a Done-when clause that rests on one statement of unexercised code on a host with no runner: nothing anywhere demonstrates the flag reaching the Windows plugin, which is the exact behavior the round was written to secure. The document's own convention is to name the lone gate for each such row (*Compatibility matrix* gnu row: "*Rollout* #4 runs the `cargo check` for it once"); §8 makes the opposite claim without one. Fix: add "`discovery list --steam-root <that install>`" to *Test strategy* #13's enumerated checks and name #13 in *Rollout* #5 as the lone gate for §8's Windows roots branch, then keep §8's "keeps working for it" sentence pointed at that gate rather than at #12.

- [minor][new] §5 — "`HKLM\SOFTWARE\Valve\Steam` → `InstallPath` as two ordered candidates: `LocalMachineValveSteam32` then `LocalMachineValveSteam64` (A4); values are appended in that order" — the r1 seam change made both HKLM views unconditional candidates, but *Test strategy* #3 was not updated: it still specifies the list for "both keys present" as "HKCU `SteamPath` first, HKLM `InstallPath` second, then the two env candidates" — four entries, where the revised rule yields five when both views answer. The two new consequences of appending both views are also unpinned: same path from both views (the §5 "collapses both views reporting one path" claim, which does hold given the `BTreeSet` in `canonical_paths`, `steam_linux.rs:322-333`, but is asserted only in prose — #3's dedupe clause covers registry-vs-env equality only), and distinct paths from the two views (two roots scanned, so rows from two different installs, where the pre-revision short-circuit scanned one). Fix: restate #3's "both keys present" expectation as the five-entry ordered list and add one enumerated case — both HKLM views reporting, same path vs distinct paths — with the expected candidate list for each.

- [nit][new] §3 — "Which HKLM view to ask first is `host_candidates`' policy, not the reader's (§5)" — "ask first" carries over the superseded fallback wording (r0 §5 asked the 64-bit view "if the first was `None`"); under §5's always-append rule there is no first/second branch, only an order, and #8 asserts "exactly the three `RegistryKey` variants" are queried. Fix: say "which HKLM views to ask, and in what order", so the §3 sentence cannot be read as license for a short-circuit that would contradict #8.

## FYI (no severity)

- For compatibility/correctness: §8 now gives `--steam-root` a build-target-dependent meaning — the Linux binary can no longer produce `steam-windows` rows from an explicit root (a mounted Windows Steam library scanned from Linux yields nothing), and the Windows binary's roots branch no longer registers the Linux plugin. That is the cost of the doubling rejection, not a defect I can anchor to the task text; the panel's other seats may want the reading of ":65-67" on the record.
- §8's "The Linux binary's roots branch is byte-for-byte T-001's" is true behaviorally (one Linux plugin from roots) but not literally: `main.rs:33-38`'s single-`if/else`-one-plugin shape becomes two `cfg`-attributed statements plus a doubled probe branch.
- Value check (checklist #7): proportionate. The r1 changes mostly moved complexity out of the untestable path — `RegistryKey`'s third variant pushes the (key, view) choice into `host_candidates`, which is pure and Linux-tested, instead of into `#[cfg(windows)]` code, and #2's shrink removed a public item the crate does not need. `absolute_env` moving to `steam_common` and `name()` delegating to `self.core.name()` both remove, not add, duplication. No new dependency, knob, or abstraction layer appeared in the diff.
