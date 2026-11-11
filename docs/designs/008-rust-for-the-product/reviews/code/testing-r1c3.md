# Code review — testing seat, dissent pass r1c3

Diff under review: `42d019b292b97c70c49f656a0b8d5304bf9b5008..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md` (Test strategy, Gates, Rollout) and `docs/adrs/008-rust-for-the-product.md`.
Checklist: `.opencode/skills/code-review/references/testing.md` (testing seat).

Dissent scope: my round-1 findings were the fix target of the last stage. I checked those
first, then re-verified the whole test suite statically, because the shell is permission-denied
in this seat too — no `cargo` could be run here, and that fact itself is finding 2.

## Prior findings — status

1. **Blocker 1 (`crates/core/tests/gate_contract.rs:330`, `deny.contains(format!(..))` E0277) — CLOSED.**
   The pin suite was rewritten and now reads `deny.contains(&format!("[{section}]"))` at
   `gate_contract.rs:408`. Re-running my round-1 grep over the workspace
   (`.contains|find|split|replace|starts_with|trim_matches|strip_prefix|strip_suffix` with a
   `format!`/`to_string()` argument) leaves `tests/common/mod.rs:316` as the only hit —
   `trim_matches('"')` is a `char` pattern, correct as before. I additionally static-checked
   every test target's imports, variant shapes, and constant types against
   `src/lib.rs`/`error.rs`/`toml.rs`/`document.rs`/`chain.rs`/`reach.rs`
   (`MAX_DOCUMENT_BYTES`/`MAX_NESTING_DEPTH`/`MAX_CHAIN_DEPTH` all `usize`, matching the
   `assert_eq!`s; `NotUtf8` unit variant matches `NotUtf8 { .. }`; `source_path() -> Option<&str>`
   matches `unwrap_or_default()` + `assert_eq!` against `&str`; `Cursor<&[u8]>: Read`). The
   suite builds.

2. **Major 2 (no gate command ever run; Test #5/#7 legs unevidenced) — STILL OPEN.** See finding 2.

3. **Minor 3 (`source_path`/`NotUtf8` public surface without consumers) — CLOSED.**
   `tests/parse_budget.rs:233-241` (`refuses_a_stream_that_is_not_utf8`,
   `parse_reader(Cursor::new(&[0xff,0xfe,0x00,0x00]))` → `NotUtf8`) and
   `tests/parse_budget.rs:252-273` (`an_io_refusal_names_its_file_only_through_source_path`:
   `source_path() == <absent path>` while `to_string()` omits it) are exactly the two cases I
   asked for, and both hold against the `src` implementation (`read_reader` refuses non-UTF-8
   before decoding; `io_error` keeps the path on the variant only; Linux io::Error Display
   carries no path, so the absence assert holds). Checklist item 7 satisfied for this surface.

The round-1c2 stage's own required test changes also landed and are correct as far as they can
be checked statically: the scan-command pin now accepts both sanctioned spellings via
`report_capture` (`gate_contract.rs:142-152`), demands a content marker (`"runs"` for SARIF —
present at `advisories.yml:95`), bans `[ ! -s` (none present in non-comment lines), and pins
the SHA-pinned `actions/upload-artifact` upload whose `path:` matches the captured redirect
target (`advisories.yml:83,105-108`). Every `gate_contract` workflow assertion was checked
line-by-line against `check.yml`, `advisories.yml`, `Cargo.toml`, `deny.toml`,
`rust-toolchain.toml`, and `tools/orchestration/package.json` — all pass on paper.

## Findings

### 1. Blocker — `tests/conformance.rs:180-211`: the zero-fixture rule's own trigger test panics before it can fire; `cargo test --workspace --locked` is still red

`the_zero_fixture_rule_fires_on_an_empty_fixture_directory` builds its scratch tree with
`ScratchDir::make_dir` + `ScratchDir::write`. `write` (`tests/common/mod.rs:199-203`) is a bare
`fs::write`, which creates the file but not its parent directory. The sequence:

- `conformance.rs:184` `make_dir("empty-valid/v1/fixtures/valid")` — creates
  `empty-valid/v1/fixtures/valid` (and its parents), not `.../fixtures/invalid`;
- `conformance.rs:185` `write("empty-valid/v1/fixtures/invalid/document.toml", …)` — the
  parent `empty-valid/v1/fixtures/invalid` does not exist → `ENOENT` → the helper panics
  (`panic!("writing {}: {err}")`);
- even if it passed, `:186` writes under `empty-invalid/v1/fixtures/valid/` before that tree
  exists at all (`make_dir` for it comes at `:187`), and `:205-206` repeats the pattern for the
  `complete` scratch tree (`write("profile/v1/fixtures/valid/document.toml", …)` with no parent).

So the test fails deterministically on every platform, and with it the third gate command:
`cargo test --workspace --locked` is red, the `gate_contract`/conformance pin suite still never
completes, and the Test #1 zero-fixture rule — the assertion the proposal calls out twice
("a `schemas/*/v*/` directory yielding zero valid or zero invalid fixtures fails the run") —
has a trigger test that cannot fire. Same net outcome as my round-1 blocker class (the gate
cannot go green, so the dated evidence cannot be recorded), one layer down: last round the
suite did not compile; now it compiles and one test panics at runtime. Every other test in the
suite static-checks as passing (fixture corpora against `toml.rs`/`document.rs`, boundary
arithmetic at 1 MiB/16/32, the workflow pins above), so this is the only remaining break.

**Fix:** have `ScratchDir::write` create the parent first — `if let Some(parent) = path.parent()
{ fs::create_dir_all(parent)… }` before `fs::write` — or `make_dir` each fixture directory
before writing into it in `conformance.rs:184-187,205-206` (the second ordering also needs
`empty-invalid`'s dirs created before `:186`'s write, so fixing the helper is the smaller,
order-proof change). Then run the suite and confirm the `test result: ok` line for
`conformance`.

### 2. Major — checklist item 6 unevaluable a fifth stage: no gate command has ever been run against this tree; Test #5/#7 legs and the advisories dispatch stay unchecked, and until finding 1 lands no run could be green anyway

`docs/designs/008-rust-for-the-product/03-build-notes.md:70-87` — all three dated bullets (fresh
Linux clone / Test #5, Windows leg / Test #7, manual `workflow_dispatch` of `advisories.yml`)
remain unchecked with no `test result:` lines pasted. This seat again could not run
`cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets -- -D warnings` /
`cargo test --workspace --locked` (shell permission-denied), so per the skill an unevaluable
checklist item is a finding, not a pass, and "paste the tail" cannot be satisfied from here.
The compile blocker named last round is now fixed, so the pattern of hand-verification in place
of an executed gate has moved past the suite-building hurdle — but it has still never produced a
run, and this round it again missed a defect that only a run surfaces (finding 1 is a runtime
panic; static compile review found it only because it was looked for by hand). With finding 1
open, even a fresh toolchain run fails at `conformance`, so Test strategy #5's one-gate promise,
#6's green-rust-leg premise, and #7's platform legs have still had zero machine evidence across
all five stages.

**Fix:** after finding 1 lands: on a fresh clone with no `DATABASE_URL`, under the pinned 1.85.0,
run the three gate commands and paste the `test result:` lines and command tails into the three
bullets; run the same three once on a Windows contributor machine (Test #7); do one manual
`advisories.yml` dispatch with the `"runs"`-marker guard step green and the
`cargo-deny-report` artifact uploaded. If no pipeline stage can run a toolchain, that is itself
the blocker to name, not a note.

## Checklist notes

1. Test strategy citations: #1 (`conformance.rs:5,44,129,152,173`), #2 (`version_gate.rs:3`),
   #3 (`chain_guard.rs:7`), #4 (`reach_tables.rs:5`, `conformance.rs:80`), #5 (`gate_contract.rs:177,500,542`),
   #6 (`gate_contract.rs:220`) all carry `proves:`-citing tests. #7's platform legs are dated
   rollout checks no test can execute; the suite cites them by delegation
   (`gate_contract.rs:179-181`). The unmet half of #5/#6/#7 is the missing machine evidence —
   finding 2, not a citation gap.
2. Layers: `tests/`-per-crate as Rollout 2 promises; in-memory unit-style guards plus
   filesystem integration (`ScratchDir`) both present.
3. Previously rejected blockers/majors: the dead-scan-control class is now pinned by
   `gate_contract.rs:327-448` + `456-498` (capture-stream spelling, content marker, artifact
   upload) — good; the "suite can't execute" class is re-opened by finding 1.
4. Determinism: no wall-clock-dependent assertions, no network, no ports; scratch dirs are
   unique by pid+nonce+nanos with `Drop` cleanup.
5. Names describe behavior throughout (`refuses_a_stream_that_is_not_utf8`, not
   `test_parse_reader_utf8`).
6. Could not run — finding 2.
7. Item clean this round: the two unowned public surface bits (`source_path`, `NotUtf8`) gained
   consumers; the remaining unexercised surface (`serialize` for reach-classes via harness
   exception, `Hand::from_name` via `hand_matches_file_name`) has spec-documented reasons.

## Verdict

request-changes — 1 blocker (finding 1), 1 major (finding 2).
