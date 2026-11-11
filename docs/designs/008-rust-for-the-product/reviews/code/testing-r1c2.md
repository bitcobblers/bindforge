# Code review — testing seat, round 1 dissent pass c2

Scope: `git diff a8b3aeda622ff4da60863c46e704091e6a74707b..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md` (Test strategy 1–7, Gates, Formats contract,
Adversarial guard), `03-build-notes.md`, and `docs/adrs/008-rust-for-the-product.md`.
Checklist applied: `references/testing.md` only. Inputs: the diff, the proposal, the
ADR, and my own round-1 findings. Verdict: **request-changes** (1 blocker, 1 major open).

## How this pass was run

The shell is denied in this reviewer's environment, so the gate commands
(`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked`) could not be run here — checklist item 6 is not
green-and-pasted, it is unevaluated, and I record it as finding 2 rather than a pass.
Every other judgment below is a hand type-check and a hand assertion-check of each test
target against the shipped `schemas/` data, done file by file.

## My round-1 findings — verification against the current tree

| Round-1c1 finding | Status | Evidence |
|---|---|---|
| Blocker 1 — `BTreeMap` not in scope in `toml.rs` | **closed** | `crates/core/src/toml.rs:28` `use std::collections::BTreeMap;`; `:98` return type and `:105` `BTreeMap::new()` now resolve. |
| Blocker 2 — `class.to_string()` on `ReachClass` in `conformance.rs:119` | **closed** | `tests/conformance.rs:122` maps to `class.id.clone()`; `classes` from `string_array` is `Vec<String>`, so the `assert_eq!` at `:123` type-checks, and the comparison is against the textually-read id list, as the load contract in `schemas/reach-classes/v1/spec.md` states. |
| Blocker 3 — `slice::contains(&&str)` on `&[ReachClass]` in `reach_tables.rs` | **closed** | `tests/reach_tables.rs:112-123` compares `class.id.as_str()` id lists against `["near","mid","extended"]` / `["near","mid","cross"]`, keeping the `hand()` checks and the `assert_ne!`. Those exact id lists match `schemas/reach-classes/v1/left.toml:6` and `right.toml:6`, so the assertion holds and is stronger than the `contains` version. |
| Major — the gate-verification checklist was never executed | **open** | `03-build-notes.md:68-80` — all three bullets still unchecked; see finding 2. |
| Minor — `hand_matches_file_name` and the stream budget had no consumer | **closed** | `tests/reach_tables.rs:138-155` (truth table over `left`/`right`/wrong-case, six cases) and `tests/parse_budget.rs:131-157` (`parse_reader` over a `Cursor` of `MAX_DOCUMENT_BYTES + 1` → `DocumentTooLarge { limit }`; in-budget stream parses). Both entry points now have consumers. |

The other two "test changes the next stage must make" that the build notes listed also
landed: `tests/gate_contract.rs:298-305` now pins `cargo deny --format json check` (the
form the pinned 0.20.2 parses) and `:306-326` rejects a `--format` flag or a section
argument after `check` — the regression guard that was missing when the round-1c1
security finding made CI weekly-red behind a green suite. I checked that guard against
the shipped `.github/workflows/advisories.yml:63-65`: the only non-comment `cargo deny`
line is the `--format json` one, and the `after_subcommand` scan finds neither `--format`
nor a section word after `check`, so it fires on the old spelling and passes on the new.

I also hand-walked the rest of the corpus against the assertions that read it, and the
data backs the suite: 33 links in `chain-past-max-depth.toml` (→ `ChainTooDeep`), 17
array levels in `nesting-past-limit.toml` (→ `NestingTooDeep`), `basic-table.toml`
declaring `hand = "right"` with `classes = ["near"]` (the `load_corpus_table` scratch
directory pairs it with the shipped `left.toml`, so both hands load), all five excluded
constructs and both `unknown-entry.toml` fixtures present, every fixture name
lowercase-and-dashed, and the Node floor matching `tools/orchestration/package.json:31`
(`>=22.23.4`) against `check.yml:48`. `members = ["crates/*"]` matches a real member, and
`channel = "1.85.0"` sits on `rust-version = "1.85"`.

## Findings

1. **blocker — `crates/core/tests/gate_contract.rs:330`.**
   `.filter(|section| deny.contains(format!("[{section}]")))` passes a `String` **by
   value** where a search pattern is required. `str::contains` takes `P: Pattern`, and
   the `Pattern` implementors are `&str`, `&&str`, `char`, `&String`, `&[char]`,
   `[char; N]`, `&[char; N]` and `F: FnMut(char) -> bool` — never `String`. The
   `gate_contract` target therefore fails to compile (E0277, rustc's "expected a
   `FnMut(char)` closure, found `String`"), so with `--all-targets` the clippy gate and
   `cargo test --workspace --locked` are still red and the whole Test #5/#6 pin suite —
   plus the new scan-command regression guard — never executes. This is the same
   round-1 blocker class for the third time in a row, in a new file, with the same root
   cause the build notes name: no toolchain has ever compiled this tree.
   Fix: `deny.contains(&format!("[{section}]"))` (or `.as_str()`); then grep the whole
   workspace for the same shape — `grep -nE '\.(contains|find|split|splitn|replace|starts_with|ends_with|trim_matches|strip_prefix|strip_suffix|matches)\([^)]*(format!|to_string\(\))' crates`
   — since the failure mode is one character wide. The only other hit today
   (`tests/common/mod.rs:316`) is a `char` pattern and is fine; keep it that way.

2. **major — `docs/designs/008-rust-for-the-product/03-build-notes.md:68-80` (and
   `:48-66`).** The three dated bullets — fresh Linux clone with no `DATABASE_URL`
   (Rollout 7 / Test #5), the Windows leg (Test #7), and one manual `workflow_dispatch`
   of `advisories.yml` — are still unchecked and unexecuted, in the fourth stage to touch
   this tree. Test strategy #5 and #7 are integration obligations that only a run can
   close, and checklist item 6 ("run the gate yourself, paste the tail") is unevaluable
   for the run *and* for this seat, so the panel has no evidence of a single green gate
   command. With finding 1 open, no such run could pass today either — the two findings
   are one problem: the fix stage keeps hand-verifying instead of running, and the
   hand-verification has now missed a compile error in each of its three rounds.
   Fix: after the `gate_contract.rs:330` fix, install rustup, run the three gate commands
   under the pinned 1.85.0 on a fresh clone with no `DATABASE_URL`, run the same three on
   a Windows contributor machine, do one manual advisories dispatch (the "produced no
   report" step must stay green), and paste the `test result:` lines and command tails
   into the three bullets. If no stage can run a toolchain, that blocker belongs in the
   verdict, not in a "not ours to edit" note.

3. **minor — `crates/core/src/error.rs:71` (`source_path`) and `:35` (`NotUtf8`).**
   Checklist item 7: public surface with no consumer. `source_path` is the documented
   channel for the `Io` path that `Display` deliberately omits — the security contract
   `03-build-notes.md`'s "Error API spelling" bullet commits to — and no test pins the
   pair: `tests/reach_tables.rs:180-183` checks only that the path is *absent* from the
   message, never that it is *present* in `source_path()`. `NotUtf8` is reachable only
   through `parse_reader`, which the revision just gave tests without a non-UTF-8 case.
   Fix: two small in-memory cases in `tests/parse_budget.rs` —
   `parse_reader(Cursor::new(&[0xff, 0xfe, 0x00, 0x00]))` expecting `NotUtf8`, and
   `parse_file(&scratch.path().join("absent.toml"))` asserting
   `err.source_path() == <that path>` while `err.to_string()` does not contain it.

## What is right

The three compile blockers are fixed exactly as the round-1 dissent asked, the two
public-API gaps I flagged are covered, the scan-command regression guard landed in the
test that had let the round-1c1 security finding through, and the corpus assertions I
re-derived by hand (chain 33/32, nesting 17/16, per-hand ids, fixture naming, Node floor,
action pins, members glob) all match the shipped data. The suite is close: one
one-character type error separates it from building, and no stage has noticed because no
stage has ever run `cargo`.

FYI (not a finding, unchanged by this diff): Rollout 2's "unit tests live in each crate's
`tests/`, mirroring `src/`" is realized as one test file per contract concern
(`conformance.rs`, `chain_guard.rs`, …) rather than one per `src/` module; the naming is
consistent and traceable, so I read it as the design's looser phrasing rather than a miss.
