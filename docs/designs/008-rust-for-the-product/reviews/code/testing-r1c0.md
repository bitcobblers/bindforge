# Code review — testing seat, round 1, pass c0

Change under review: `git diff 881d3b48641d7dbb6114de03c5770bc12073bc45...HEAD -- ':(exclude)docs/designs'`
(32 files, +1918, all additions except `.gitignore`, `README.md`, `.opencode/skills/repo-practices/SKILL.md`).
Judged against `02-proposal.md` and `docs/adrs/008-rust-for-the-product.md`. No prior findings carried in.

Verdict: **request-changes** — 2 blockers, 2 majors.

## Gate run (checklist item 6)

The seat had no shell permission, so the gate commands were executed verbatim by a mechanical
helper subagent that was fenced to the diff's paths and to `docs/designs` exclusion; the repo was
not modified. Environment: `cargo 1.99.1` / `rustc 1.99.1` (Arch package), **no rustup installed**,
so `rust-toolchain.toml`'s pinned `1.85.0` channel was never applied anywhere in these runs.

Run at HEAD, as committed:

```
$ cargo fmt --all -- --check                          exit 1
`cargo metadata` exited with an error: error: failed to load manifest for workspace member
`/home/mike/repos/bindforge/apps/*`
referenced by workspace at `/home/mike/repos/bindforge/Cargo.toml`
Caused by:
  failed to read `/home/mike/repos/bindforge/apps/*/Cargo.toml`
Caused by:
  No such file or directory (os error 2)

$ cargo clippy --workspace --all-targets -- -D warnings   exit 101
error: failed to load manifest for workspace member `/home/mike/repos/bindforge/apps/*`
referenced by workspace at `/home/mike/repos/bindforge/Cargo.toml`
Caused by:
  failed to read `/home/mike/repos/bindforge/apps/*/Cargo.toml`
Caused by:
  No such file or directory (os error 2)

$ cargo test --workspace --locked                     exit 101
error: failed to load manifest for workspace member `/home/mike/repos/bindforge/apps/*`
referenced by workspace at `/home/mike/repos/bindforge/Cargo.toml`
Caused by:
  failed to read `/home/mike/repos/bindforge/apps/*/Cargo.toml`
Caused by:
  No such file or directory (os error 2)
```

Zero test binaries were built; zero tests ran; no `running N tests` / `test result:` line exists.

Same tree copied to a scratch dir (`/tmp/opencode/rev-c0/ws`, repo untouched) with the one line
`members = ["crates/*", "apps/*"]` narrowed to `members = ["crates/*"]`:

```
$ cargo test --workspace --locked                     exit 101
   Compiling bindforge-core v0.1.0 (/tmp/opencode/rev-c0/ws/crates/core)
error[E0599]: no method named `path` found for enum `Result<T, E>` in the current scope
   --> crates/core/tests/common/mod.rs:177:61
177 |     let mut paths: Vec<PathBuf> = entries.map(|entry| entry.path()).collect();
    |                                                             ^^^^ method not found in `Result<DirEntry, std::io::Error>`
error: could not compile `bindforge-core` (test "reach_tables") due to 1 previous error
error: could not compile `bindforge-core` (test "version_gate") due to 1 previous error
error: could not compile `bindforge-core` (test "chain_guard") due to 1 previous error
error: could not compile `bindforge-core` (test "conformance") due to 1 previous error

$ cargo clippy --workspace --all-targets -- -D warnings   exit 101
error: using `contains()` instead of `iter().any()` is more efficient
  --> crates/core/src/document.rs:91:9
   = note: `-D clippy::manual_contains` implied by `-D warnings`
error: could not compile `bindforge-core` (lib) due to 1 previous error
error: could not compile `bindforge-core` (lib test) due to 1 previous error

$ cargo fmt --all -- --check                          exit 1
Diff in .../crates/core/src/document.rs (4 hunks), src/error.rs (2), src/lib.rs (1, import order),
src/reach.rs (1, import order), src/toml.rs (2), tests/chain_guard.rs (1), tests/common/mod.rs (4),
tests/conformance.rs (2), tests/reach_tables.rs (1)
```

`tools/orchestration` is unchanged by this diff, so `npm run check` is not in scope for this pass
(`check.yml`'s `orchestration` job shape is noted under Test strategy #6).

## Checklist walk

1. **Proposal Test strategy citations** — #1 (`conformance.rs:5,37,72`), #2 (`version_gate.rs:3,41,53,66,91`),
   #3 (`chain_guard.rs:7,36,49,61,76`), #4 (`reach_tables.rs:5,33,57`) are all cited. #5, #6, #7 have no
   `proves: Test strategy #N` citation anywhere — only prose in `conformance.rs:15-20`. Partial → finding 5.
2. **Unit and integration layers** — the proposal puts the harness and the unit suites in
   `crates/core/tests/` (Rollout 2) and that is what ships; the promised integration layer (#5, #6) has
   no counterpart in code. Covered by finding 5 rather than a separate one.
3. **Revision rounds** — no previously rejected blocker/major exists for this seat; nothing to re-test.
4. **Determinism** — clean. Every test reads the checked-in contract tree through
   `env!("CARGO_MANIFEST_DIR")` (`common/mod.rs:36-43`); no wall-clock, no randomness, no network, no
   ports, no Postgres. The fixtures are static files. Pass.
5. **Names describe behavior** — clean. `rejects_a_chain_one_link_deeper_than_max_chain_depth`,
   `every_invalid_fixture_is_rejected_with_a_structured_error`,
   `core_declares_exactly_the_major_versions_present_under_schemas` all read as behavior, not steps. Pass.
6. **`check` gate green** — **not green**, on any toolchain, for three independent reasons: the
   workspace members glob, the test-target compile error, and fmt/clippy. Blockers 1–2, major 3.
7. **Untestable additions / dead branches** — the parse budget, the grammar-rejection branches, and the
   loader's hand/file-name conflict branch are all behavior with no test (findings 4, 6); `deny.toml`
   carries three sections no job runs (finding 8).

## Findings

1. **blocker — `crates/core/tests/common/mod.rs:177`.** `fs::read_dir()` yields
   `io::Result<DirEntry>`, so `entries.map(|entry| entry.path())` does not compile. Because every suite
   pulls in `mod common;`, all four test targets (`conformance`, `version_gate`, `chain_guard`,
   `reach_tables`) fail to build and **zero tests execute** — the entire Test strategy #1–#4 corpus is
   dead code today. Fix: handle the `Result`
   (`.filter_map(|entry| entry.ok().map(|entry| entry.path()))`, or propagate into a `CoreError::Io`), then
   run `cargo test --workspace --locked` and paste the `test result:` lines into the build notes.
2. **blocker — `Cargo.toml:7`.** `members = ["crates/*", "apps/*"]` names a directory that has no member
   crate. Git does not track empty directories, so a fresh clone (and therefore every CI runner, per Test
   strategy #5's "fresh clone") has no `apps/` at all, and all three gate commands abort at manifest load
   with `failed to read .../apps/*/Cargo.toml` before any crate is built. The one-gate promise fails at its
   first command. Fix: `members = ["crates/*"]` now, with the comment noting `apps/*` is added by ADR-012
   when `apps/cli` lands (and `apps/desktop/src-tauri` in the desktop era) — the proposal's glob is a
   target state, not this tree's state.
3. **major — gate not demonstrably green on any toolchain: `crates/core/src/document.rs:91`, plus
   `cargo fmt` hunks across `src/` and `tests/`.** `clippy::manual_contains` fires on
   `supported_majors().iter().any(|major| *major == version)` under `-D warnings`, and `cargo fmt --all
   -- --check` reports 13 hunks (including import order in `src/lib.rs:24-28` and `src/reach.rs:16-19`).
   Neither failure was observed under the pinned `1.85.0` channel, because rustup is absent from this
   environment — which is itself the point: the pin has never been exercised, so I cannot evaluate the
   gate as green. Fix: install rustup (or otherwise honor `rust-toolchain.toml`), run `cargo fmt --all`,
   rewrite line 91 as `supported_majors().contains(&version)`, and re-run all three gate commands under
   1.85.0 with their tails recorded.
4. **major — the adversarial parse budget the design commits to the contract has no test.**
   `schemas/README.md:46-47` states the 1 MiB and 16-level limits as third-party conformance rules and
   `error.rs:18-20` carries `DocumentTooLarge`/`NestingTooDeep`, but no fixture and no test produces
   either — `grep` over `crates/core/tests` finds neither constant nor variant. `toml.rs:39-43` and
   `toml.rs:229-233` are therefore unenforced contract behavior, and they are exactly the guards the ADR's
   "the hub validates uploads with the same compiled types" consequence leans on. Fix: mirror the chain
   guard's shape — an invalid fixture under `schemas/profile/v1/fixtures/invalid/` nesting arrays to
   limit+1 (`nesting-past-limit.toml`), plus a `tests/parse_budget.rs` asserting `NestingTooDeep` at
   limit+1 and acceptance at the limit, and `DocumentTooLarge` for a `MAX_DOCUMENT_BYTES + 1` document
   built in memory (not committed).
5. **minor — Test strategy #5, #6, #7 are uncited by any test.** #1–#4 all carry
   `// proves: Test strategy #N`; #5–#7 exist only as prose in `conformance.rs:15-20` and as comments in
   `check.yml:4-6`. Checklist item 1 treats an unevaluated item as a finding. Fix: add
   `crates/core/tests/gate_contract.rs` citing #5 and #6 at the level code can pin — read
   `.github/workflows/check.yml` and assert both jobs (`rust`, `orchestration`) exist, every
   third-party action is pinned to a 40-char SHA, `permissions: contents: read` is set per job, and no
   step or service references `DATABASE_URL`/Postgres. The #5 fresh-clone leg and #7 Windows leg cannot
   be tested from the tree; record them as explicit dated rollout checks with their command output in the
   build notes (Rollout 7), so "verified once on Linux and once on Windows" is evidence rather than intent.
6. **minor — unexercised rejection branches in the loader and the grammar; one test pins a tautology.**
   `reach.rs:92-111` (a `<hand>.toml` whose `hand` entry contradicts its file name) and `reach.rs:80-83`
   (unreadable contract file) are unreachable by any test, and `tests/reach_tables.rs:81` asserts
   `!std::ptr::eq(left, right)` — two entries of the same `Vec` always have distinct addresses, so that
   assertion cannot fail on the bug it claims to guard (both hands loaded from the same data). Likewise
   `toml.rs` rejects table headers, dotted keys, multi-line arrays, floats and hex integers, and
   `schemas/README.md:36-38` declares them outside the v1 grammar, yet no invalid fixture covers any of
   them. Fix: replace the pointer check with `assert_ne!(left.classes(), right.classes())` (the shipped
   `left.toml` is `near/mid/extended`, `right.toml` is `near/mid/cross`, so it is a real assertion) plus
   exact expected class lists; cover the hand/file-name conflict through an invalid fixture
   (`hand-mismatch.toml`) or by exposing the file-name/hand agreement rule as a pure function the test can
   call; add one invalid fixture per excluded syntax (`table-header.toml`, `dotted-key.toml`,
   `multi-line-array.toml`, `float-value.toml`, `hex-integer.toml`).
7. **minor — the valid corpus never exercises a string that needs escaping, and there it breaks.**
   The only valid profile fixture is `basic-profile.toml` with `derives = []` and a plain ASCII id, so the
   re-serialization half of Test strategy #1 runs on exactly one trivial document. `document.rs:243-257`
   does not escape `\b` (0x08) or `\f` (0x0c), while `toml.rs:161-162` decodes those escapes and
   `toml.rs:175-177` rejects the raw control character — so a document containing `\b` or `\f` parses,
   serializes to text its own parser refuses. Fix: add a valid profile fixture with a non-empty `derives`
   list and an id carrying `\"`, `\\`, `\t`, `\b` (and/or a unit round-trip over a string table); the
   harness's existing round-trip assertion then catches the serializer gap instead of leaving it latent.
8. **nit — `deny.toml` carries config no job consumes.** `advisories.yml:30` runs only
   `cargo deny check advisories`, so `[bans]`, `[licenses]` and `[sources]` (`deny.toml:15-36`) are
   knobs without a consumer — the license allow-list reads like an enforced policy and never runs.
   Fix: either run `cargo deny check` in the scheduled job, or delete the three sections and say the
   scheduled scan is advisories-only.
9. **nit — the zero-fixture rule's own trigger is unexercised.** `conformance.rs:99-120` asserts the real
   tree is non-empty, but nothing can show the rule fires, because `schemas_root()`
   (`common/mod.rs:36-43`) is hard-wired. Fix: take the root as a parameter on `format_dirs()`/
   `fixtures()` and add a case pointing at a scratch tree with an empty `fixtures/valid/` asserting the
   run fails. Relatedly, `common/mod.rs:17`'s blanket `#![allow(dead_code)]` hides helpers that stop being
   used across all four targets; scope it to the items that need it.

## FYI (out of specialty, no severity)

The `orchestration` job runs `npm ci`/`npm run check` with `working-directory: tools/orchestration` and
`node-version: "22.23.4"`, matching Gates/A28; `deny.toml:4-6` deliberately spells the advisory waiver
`ignore` where the proposal says `skip` — correctness/compatibility seats may want to reconcile that wording.
