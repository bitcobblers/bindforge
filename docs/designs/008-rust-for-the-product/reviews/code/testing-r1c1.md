# Code review — testing seat, round 1 dissent pass (c1)

Scope: `git diff ea54086ca6562fb2b1bf69a25bf071479cb09b27..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md`, `03-build-notes.md`, ADR-008, and my round-1 findings.
Shell/cargo was not available in this seat's environment either, so I hand-verified
compilation of every source and test file line by line instead of running the three gate
commands; the decisive findings below are hard compile errors, which the CI runner will
confirm. The checklist's "run the check yourself" item therefore stays unevaluated by me
and is counted as a finding, not a pass.

## Round-1 finding status

| R1 finding | Status |
|---|---|
| 1 (blocker) `common/mod.rs` `read_dir` Result unhandled | **Fixed in place** (`read_dir_sorted` now unwraps each `DirEntry`), but the corpus *still does not build*: the revision introduced three new compile errors (findings 1–3 below). Net effect unchanged — zero tests execute. Blocker open. |
| 2 (blocker) `members = ["crates/*", "apps/*"]` | **Resolved.** `Cargo.toml:12` ships `members = ["crates/*"]` with the target-state comment, and `gate_contract.rs`'s `every_workspace_members_entry_names_an_existing_member` is exactly the regression test that catches this class of bug. |
| 3 (major) gate not demonstrably green | **Open.** `document.rs:108` now uses `supported_majors().contains(&version)` (the `manual_contains` site is gone) and src is fmt-shaped by hand, but no gate command has run on any toolchain — `03-build-notes.md:48-68` still carries the unchecked fresh-clone/Windows/dispatch checklist — and finding 1 means no toolchain run could pass today. |
| 4 (major) parse budget unfixture'd/untested | **Resolved.** `nesting-past-limit.toml` (17 array levels) in the corpus plus `tests/parse_budget.rs` asserting the boundary on both sides, text path and file path, against the exported constants. |
| 5 (minor) Test #5/#6/#7 uncited | **Resolved.** `tests/gate_contract.rs` pins both jobs, the three verbatim gate commands, SHA pins, per-job permissions/timeouts, no-DB, Node floor read from the package, and the scan's separate toolchain; #5's fresh-clone and #7's Windows legs are the dated Rollout 7 checks in the build notes. |
| 6 (minor) reach distinctness tautology; grammar exclusions unfixtured | **Partly resolved.** All five excluded-construct fixtures exist and the reference reader refuses each; the pointer tautology was replaced with content assertions — but the replacement does not compile (finding 3). |
| 7 (minor) trivial round-trip fixture | **Resolved.** `escaped-strings.toml` carries a non-empty `derives` and an id with `\"`, `\\`, `\t`, `\b`, `\f`; `quoted()` now escapes `\b`/`\f`/control as `\uXXXX`, closing the serializer gap the old fixture could not see. |
| 8 (nit) `deny.toml` sections without a consumer | **Resolved.** `advisories.yml:51-56` runs the full `cargo deny check`, and `gate_contract.rs` fails if the job regresses to `check advisories` while sections are configured. |
| 9 (nit) zero-fixture rule's trigger unexercised; blanket `allow(dead_code)` | **Resolved.** `format_dirs_under(root)` parameterizes the walk and `the_zero_fixture_rule_fires_on_an_empty_fixture_directory` exercises it against scratch trees (both directions). The module-level `allow(dead_code)` stays with a stated reason (each target uses a different subset); I accept that rationale. |

## Findings

### 1. Blocker — `crates/core/src/toml.rs:28-31` (use of `BTreeMap` at :97, :104): missing import, the crate does not compile

`read_document` returns `Result<BTreeMap<String, Value>, CoreError>` (line 97) and
`read_document` builds `BTreeMap::new()` (line 104), but the module's imports are only
`std::fs`, `std::io::Read`, `std::path::Path`, `crate::error::CoreError`. `BTreeMap` is
not in scope (no std prelude entry) — `error[E0412]`/`[E0433]` at line 97. This is in
**library source**, not a test: `cargo clippy --workspace --all-targets` and
`cargo test --workspace --locked` both fail at the first member, so every gate command
is red and zero tests execute — the same end-to-end effect as round-1 blocker 1, just
moved from `tests/common/mod.rs` to `src/toml.rs`. This is precisely why "hand
verification" (build notes §Gate verification status) cannot substitute for the gate.

Fix: add `use std::collections::BTreeMap;` to `crates/core/src/toml.rs`, then run all
three gate commands under the pinned 1.85.0 toolchain and paste the `test result:` lines
into the build-notes checklist.

### 2. Blocker — `crates/core/tests/conformance.rs:119`: `class.to_string()` on `&ReachClass` — no `Display`, test target fails to build

`load_corpus_table` maps `table.classes().iter()` through `|class| class.to_string()`.
`ReachClass` (reach.rs:61-64) derives only `Clone, Debug, PartialEq, Eq` — no `Display`,
so `ToString` is not implemented: `error[E0599] no method named to_string`. The
conformance target (the flagship Test #1 suite) fails to compile.

Fix: `class.id.clone()` (or `class.id.to_string()`), or implement `Display` for
`ReachClass` if the contract wants an id-shaped string; either way the comparison
against the textually-read `classes` list then holds.

### 3. Blocker — `crates/core/tests/reach_tables.rs:114-121`: `contains(&"near")` against `&[ReachClass]` — type error, Test #4 suite fails to build

`left.classes()` returns `&[ReachClass]`; `slice::contains` takes `&ReachClass`
(`T = ReachClass`), and `&"near"` is `&&str` — `error[E0308]` six times over. The
rewritten Test #4 assertions replace the round-1 pointer tautology (good direction) but
do not compile, so the suite that is supposed to prove the round-1 fix is itself the
build failure.

Fix: compare ids directly and exactly, which also lands the "exact expected class
lists" half of my round-1 fix that the `contains` version only approximates:
`assert_eq!(left.classes().iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["near", "mid", "extended"]);`
and `["near", "mid", "cross"]` for the right hand, keeping the `hand()` checks and the
`assert_ne!`.

### 4. Major — gate verification checklist (`03-build-notes.md:59-68`) still entirely unexecuted

Finding 1 proves the gate is red on every toolchain until fixed, and no run of the three
commands — fresh clone, Windows leg, or the `workflow_dispatch` advisories check — has
ever been recorded anywhere. Checklist item 6 of my own seat ("run the gate yourself")
remained unevaluable in both implementation and test stages; per my checklist an item I
cannot evaluate is a finding. After findings 1-3 are fixed, the dated evidence in
Rollout 7's three bullets is what closes this.

### 5. minor — `crates/core/src/lib.rs:34,31`: `hand_matches_file_name` and `parse_reader` are exported public API that no test consumes

The build notes' own test-stage item 2 asked `parse_budget.rs` to cover
`hand_matches_file_name` ("reachable without a filesystem") — the shipped file does not,
and no other suite calls it; likewise the stream half of the input budget
(`parse_reader`/`read_reader`, the hub-upload entry point the Adversarial guard section
commits to) is never exercised. Under checklist 7 these are config knobs without a
consumer: an unused public fn and an untested budget path. (The closed-entry-set half
of that item *was* covered, by `grammar_closure.rs`.)

Fix: one unit test each — `hand_matches_file_name(Hand::Left, "left"/"right"/"Left")`
truth table, and `parse_reader(Cursor::new(bytes_of_exactly(budget + 1)))` refusing with
`DocumentTooLarge` while an in-budget stream parses.

## Verdict

request-changes — three compile errors (one in `src/`, two in the rewritten test
assertions) mean the test corpus the revision was written to complete still executes
zero tests, re-opening round-1 blocker 1 in new locations. The rest of the round-1 fix
list is genuinely and verifiably addressed.
