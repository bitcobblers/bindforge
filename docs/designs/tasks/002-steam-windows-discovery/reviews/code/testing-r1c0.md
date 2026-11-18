# Code review — testing seat — r1c0

- Design: docs/designs/tasks/002-steam-windows-discovery/02-proposal.md
- Task: docs/tasks/T-002-steam-windows-discovery.md
- Scope: `git diff d4242107d82c9cf2ef3750615158165b4260f397...HEAD -- ':(exclude)docs/designs'`
- Pass: c0 (full panel), round 1. First pass for this seat; no prior findings.
- Checklist: `references/testing.md` (code testing), applied alone.

## Evidence

Supplied (runner gate record, 2026-11-18T06:26:23Z, green): `npm run check`
(exit 0), `cargo fmt --all -- --check` (exit 0),
`cargo clippy --workspace --all-targets -- -D warnings` (exit 0),
`cargo test --workspace --locked` (exit 0). The record's lone quoted
`test result: ok. 0 passed; ...` line is one zero-test binary's line, not a
workspace total — resolved by the executed check below.

Executed by this reviewer:

- `cargo test --workspace --locked` — green. Totals for the change under
  review: `steam_windows_tests` **26 passed**, `cli_dual_plugin_tests` **2
  passed**, plus the untouched regression suites green
  (`steam_linux_tests` 23, `plugin_tests` 1, `cli_tests` 3). This resolves
  the "0 passed" ambiguity in the supplied record: the new tests exist and
  run.
- `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` —
  **could not run**: E0463 "can't find crate for `core`/`std`; the
  x86_64-pc-windows-msvc target may not be installed". The reviewer has no
  `rustup target add` right, so Test strategy #12's property is not
  establishable in this review environment (see Finding 2 — missing
  evidence, not a failure).
- Read-only greps over the diff and the touched files (fixture layout,
  `// proves:` citation inventory, `\0` coverage search).

## Checklist walk

1. **Strategy items cited by tests.** #1–#8 are cited by
   `// proves: Test strategy #N` comments in
   `crates/discovery/tests/steam_windows_tests.rs` (lines 176, 262, 285, 310,
   351, 367, 383, 399, 422, 454, 499, 512, 535, 548, 560, 576, 606, 655, 684,
   714, 726, 737, 780, 801, 820, 840); #10 by
   `apps/discovery-cli/tests/cli_dual_plugin_tests.rs` (lines 23, 55). #9 is
   the untouched-suite gate: the diff modifies no existing test file
   (diffstat confirms), and those suites pass unmodified (executed run). #11
   is the cargo gate trio — green in the supplied record — with the
   `Cargo.lock` diff limited to `winreg` + `cfg-if` as #11 requires
   (`windows-sys` reused, not duplicated). #12: **missing evidence** —
   Finding 2. #13: **not applicable here** — a manual Windows-machine gate
   the proposal defers to *Rollout* #5 / `03-build-notes.md`; no Linux runner
   can execute it, and build notes are outside this seat's permitted inputs.
   One enumerated sub-case of #1 (NUL) is not covered — Finding 1.
2. **Layers.** Unit (`steam_windows_tests.rs`: rule parity, pure
   `host_candidates`, seam-shape recording fake) and integration
   (`cli_dual_plugin_tests.rs` subprocess + in-process engine, plus the
   unchanged suites) both exist, as the proposal promises.
3. **Revision rounds.** First pass; nothing previously rejected.
4. **Determinism.** No wall-clock, network, ports, or real registry/env:
   fakes are closures over `HashMap`s; the subprocess CLI test uses
   `env_clear()` + `HOME=/bindforge-no-such-home`, so the host-probe branch
   is hermetic on any build host. Fixtures are read-only; comparisons go
   through `std::fs::canonicalize` on both sides.
5. **Names describe behavior** (e.g.
   `parse_manifest_refuses_installdir_containing_backslash`,
   `host_candidates_does_not_dedup_a_registry_value_equal_to_an_env_default`).
6. **Gate assessment.** Supplied gates green; the executed `cargo test`
   confirms the new tests run; the executed cross-target check was not
   runnable (Finding 2). No failing required leg observed.
7. **Untestable additions.** None found beyond the two findings: the
   non-Windows `from_host()` branch is exercised by the CLI subprocess test;
   `WindowsTarget`'s rules, `RegistryKey` and the seam are all exercised;
   `registry_string_value` is `#[cfg(windows)]` by design, its compile
   coverage being exactly Finding 2's evidence gap.

## Findings

### 1. minor — the shared NUL refusal is exercised by no test (crates/discovery/tests/steam_windows_tests.rs:215)

Test strategy #1 enumerates the shared reject set as
`""`/`.`/`..`/`/abs/path`/`sub/dir`/**NUL**, but the parity suite's reject
list (`steam_windows_tests.rs:215`) and the Windows rule-set list (`:290`)
both stop at `sub/dir`, and no test anywhere in `crates/discovery/tests` or
`apps/discovery-cli/tests` feeds an interior NUL `installdir` (grep for
`\0`/NUL cases returns nothing). The clause
`!install_dir.contains('\0')` (`crates/discovery/src/steam_common.rs:268`)
is therefore untested on either platform — deleting it would keep the whole
suite green. (The `keyvalues` parser carries no control-character rule, so
a NUL reaches `parse_manifest` and the clause is the only thing that
decides the case.)

Fix: add `"foo\0bar"` to the parity reject list at
`steam_windows_tests.rs:215` and to the refuse list at `:290`; both sides
must yield `None` on the shared clause.

### 2. minor — Test strategy #12's Windows compile gate has no captured evidence (check.yml:29; gate record)

The runner's gate record captured only the four commands (npm check, fmt,
clippy, test); the cross-target `cargo check` that #12 and the task's
Done-when require — the sole verification of `registry_string_value`,
`from_host()`'s Windows branch and §8's `#[cfg(windows)]` roots branch —
is not among them. This reviewer's attempt to establish it failed on the
missing `x86_64-pc-windows-msvc` target (E0463), and installing it is
outside reviewer rights, so the property currently rests on the new
"Windows compile check" CI step (` .github/workflows/check.yml:29`) that
runs only on push. This is missing evidence, not a failure: the property is
establishable and CI now enforces it, so it does not block proceeding.

Fix: capture a green
`rustup target add x86_64-pc-windows-msvc && cargo check --workspace
--all-targets --target x86_64-pc-windows-msvc` transcript in the run record
(or point at the green CI leg) before the change merges.

## Verdict

approve — zero blocker/major findings; two minors above.
