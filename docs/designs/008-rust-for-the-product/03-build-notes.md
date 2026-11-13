# Build notes — design 008 implementation

What landed, against the Rollout list:

1. Root `Cargo.toml` (virtual workspace, `resolver = "3"`, members — see the
   deviation below — `[workspace.package]` at edition 2024 / `rust-version`
   1.85 / Apache-2.0, and `[workspace.dependencies]` as written in the
   proposal), `rust-toolchain.toml` (`channel = "1.85.0"`), committed
   `Cargo.lock`, and `/target` added to `.gitignore`.
2. `crates/core` — package `bindforge-core`, the depends-on-nothing root.
   Modules: `toml` (parse + input budget), `document` (model, version gate,
   closed per-format grammars, `serialize`), `chain` (`resolve_chain`,
   `MAX_CHAIN_DEPTH = 32`, `Cycle`/`ChainTooDeep`), `reach` (reach-table
   loader, `Hand`), `error` (`CoreError`). Public surface: `parse`,
   `parse_file`, `parse_reader`, `serialize`, `supported_majors`,
   `resolve_chain`, `MAX_CHAIN_DEPTH`, `MAX_DOCUMENT_BYTES`,
   `MAX_NESTING_DEPTH`, `load_reach_tables`, `hand_matches_file_name`,
   `Hand` (with `from_name`), `ReachClass`, `ReachTable`, `ReachTables`,
   `Document`, `Profile`, `ReachClassTable`, `CoreError` (with
   `source_path`).
3. `schemas/` — `README.md` with the canonical versioning rule, the grammar
   subset statement, and the parse budget / chain bound; `profile/v1` and
   `reach-classes/v1` with `spec.md`, table data, and valid+invalid fixture
   corpora covering the missing-version case, both chain boundary cases
   (`chain-past-max-depth.toml` holds 33 literal links — regenerate it if
   ADR-009 tunes `MAX_CHAIN_DEPTH`), the nesting boundary
   (`nesting-past-limit.toml`, 17 array levels — regenerate if the budget
   moves), the closed entry set (`unknown-entry.toml` in both formats), each
   excluded TOML construct (`table-header`, `dotted-key`, `multi-line-array`,
   `float-value`, `hex-integer`), a rejection carrying a newline in an
   untrusted value (`forged-log-line.toml`), and a valid profile whose id
   exercises the string escapes (`escaped-strings.toml`). The `conformance.rs`
   harness walks them unchanged.
4. `.github/workflows/check.yml` (`rust` + `orchestration` jobs, actions
   pinned to SHAs, `permissions: contents: read`, `setup-node`
   `node-version: "22.23.4"`, `timeout-minutes` on both jobs) and
   `.github/workflows/advisories.yml` (weekly + manual, `cargo-deny` at a
   pinned version on its own toolchain, the full `cargo deny check`, a step
   that fails if no report was produced, the report uploaded as a build
   artifact, `timeout-minutes`) plus root `deny.toml`.
5. Rust section added to `.opencode/skills/repo-practices/SKILL.md`
   (package shape and `bindforge-` naming rule, the members-glob rule, cargo
   gate commands, Node floor 22.23.4, capability-file rule,
   tests-under-each-crate, the advisory scan's own toolchain, and the
   documented grammar subset / closed entry set).
6. `README.md` gained the rustup one-liner and the Linux system-deps list.

## Gate verification status — BLOCKED: no stage in this pipeline can run a toolchain

> **Update 2026-11-13 — the blocker is closed; do not carry it forward as an
> assumption.** It was two things: the stage agents' shell allowlist was a set
> of narrow prefixes that a compound command fails leg-by-leg (`cd <dir> &&
> cargo test` is denied on the `cd`), and this paragraph was quoted into later
> runs as a `verified` assumption (ADR-011's A16), so stages stopped trying.
> The agents now allow `cargo *`, `npm run *`, `rustfmt *` and read-only
> inspection, and deny only what the runner owns (commits, branch state, file
> moves) plus publishing. Run 011 paid 4 of its 5 code rounds for the old
> posture. See `docs/workflows.md`.

**This is a blocker, and it is named as one rather than footnoted** (round-1
testing major 2, open since the first rollout check and re-opened by the
round-1c3 dissent pass as its finding 2). Every stage that has touched this tree
— both implementation stages, the test pass, round-1c1, round-1c2, and this
round-1c4 stage — has had its shell tool permission-denied; this stage
re-confirmed it (`cargo --version` and an `echo` are both refused before they
run, so nothing here can invoke a toolchain). Consequence:
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings` and
`cargo test --workspace --locked` have **never been run against this tree**, no
line in this file is `cargo` output, and every claim about the tree building,
formatting, or clippy-ing cleanly is hand verification. A stage that can only
read and write files cannot close the checklist below; the missing thing is an
environment with the pinned toolchain, and that is the blocker.

What has blocked a run has moved three times, and the current state should not
be misread from the older sections of this file:

- The `src/` compile error (missing `BTreeMap` import in `crates/core/src/toml.rs`)
  — fixed in round-1c1. It was the only one found in `src/`.
- The test-target compile error `deny.contains(format!(..))` at
  `tests/gate_contract.rs:330` (E0277) — fixed in the test pass that followed
  round-1c2, and re-verified by the round-1c3 dissent pass, which static-checked
  every test target's imports, variant shapes, and constant types and found the
  suite builds.
- What keeps a run red today is **one runtime panic, not a compile error**:
  `ScratchDir::write` (`crates/core/tests/common/mod.rs:199-203`) is a bare
  `fs::write` that does not create the parent directory, so the zero-fixture
  rule's trigger test panics at ENOENT before it can assert anything
  (`tests/conformance.rs:185-186,205-206`). That is round-1c3 finding 1, and its
  fix is a test-path edit this stage must not make — it is spelled out in
  "Test changes the next stage must make" below. Nothing in `src/` is known to
  be broken.
  **Update (2026-11-11): that edit has since landed.** The test-update stage
  applied the prescribed block verbatim — `common/mod.rs:198-211` now runs
  `fs::create_dir_all(parent)` before `fs::write` — and the round-1c5
  ratification pass re-verified it at HEAD. This bullet describes a closed
  defect; no known runtime or compile break remains, and the only thing
  keeping the checklist below open is that no stage can invoke a toolchain
  (see "Round-1c5 outcome and current state").

Two facts that make the checklist closable once that one test edit lands, so no
later stage has to re-derive them:

1. The Test #5 Linux leg already has a machine that runs it: `check.yml`'s
   `rust` job runs exactly the three gate commands, on a fresh clone of the PR
   branch, with no `DATABASE_URL`, no Postgres service, and rustup honouring
   `rust-toolchain.toml` (1.85.0) — which is the precondition
   `gate_contract.rs::the_rust_gate_runs_exactly_the_three_gate_commands_with_no_database`
   pins. The PR carrying the test edit *is* that leg; its `rust` job output
   belongs in bullet 1 below. Bullet 3 is a manual `workflow_dispatch` in the
   Actions UI, which also needs no local toolchain.
2. Until bullet 1 holds pasted `test result:` lines, Test strategy #5's
   one-gate promise, #6's green-`rust`-leg premise, and #7's platform legs have
   zero machine evidence, and Rollout 7 stays unticked. Landing a merge without
   those lines is the failure mode this finding names, not a ship state.

Outstanding dated checks (paste each run's `test result:` lines and command
tail into its bullet; do not mark any of them done from reading the code):

- [ ] Rollout 7 / Test #5, fresh Linux clone, no `DATABASE_URL`, rustup
  honouring `rust-toolchain.toml` (1.85.0):
  `cargo fmt --all -- --check` (expect exit 0),
  `cargo clippy --workspace --all-targets -- -D warnings` (expect exit 0),
  `cargo test --workspace --locked` (paste the `test result:` lines here).
- [ ] Test #7 Windows leg, same three commands on a Windows contributor
  machine, plus the fixture-name case-collision suite passing there.
- [ ] `advisories.yml` green on a manual `workflow_dispatch` run, with the
  "scan produced no report" step *not* firing and a `cargo-deny-report`
  artifact uploaded whose file holds a SARIF document (a `"runs"` key) —
  that step-plus-artifact pair is the control that proves the scheduled scan
  is alive. It doubles as the check that the scan step's command line parses
  on the pinned cargo-deny and captures the stream the pinned version writes
  its report to: the round-1c1 security finding was a flag position the
  pinned version rejects at parse time, and the round-1c2 finding was that
  with `--format json` the pinned version reports on stderr, so a stdout
  redirect captured an empty file and this step went red every week although
  the scan had run.

## Runtime-break audit (round-1c4)

Round-1c3's point is that a defect of this class only a *run* surfaces, and this
stage has no run available. So instead of hand-waving, this stage swept the
whole suite for the same class, to say what a green run is expected to look like
and to make sure the next gate run does not fail on a second landmine after the
first is fixed:

- **Every `ScratchDir` call site in the suite** (29 matches: `conformance.rs`,
  `parse_budget.rs`, `reach_tables.rs`, `common/mod.rs`). Only
  `conformance.rs:185-186,205-206` writes a path with an intermediate directory
  that no `make_dir` created. Every other `write` puts one file directly in the
  scratch root (`conformance.rs:105-106`, `parse_budget.rs:107,128`,
  `reach_tables.rs:167-168,198-199`), which `fs::write` can do. So the parent
  guard in `write` fixes the whole class, and no other test carries it.
- **Every `fs::read_to_string`/`fs::read_dir` a suite performs** resolves through
  `repo_root()` (`env!("CARGO_MANIFEST_DIR")`) or a fixture path derived from it,
  and each named file exists in the tree: `.github/workflows/check.yml`,
  `.github/workflows/advisories.yml`, `Cargo.toml`, `rust-toolchain.toml`,
  `deny.toml`, `tools/orchestration/package.json`,
  `schemas/reach-classes/v1/{left,right}.toml` — read by the harness for the other hand.
  Nothing reads a path absent from a fresh clone. The one intentional read of a
  missing file is `parse_budget.rs:257` (`absent.toml`), which asserts the refusal.
- **Corpus against `src/`.** Both valid profile fixtures parse and round-trip:
  `document.rs::quoted` escapes `"`, `\`, `\n`, `\r`, `\t`, `\b`, `\f` and other
  control characters as `\uXXXX`, so `escaped-strings.toml` re-parses to an equal
  `Document`. The one valid reach-classes fixture keeps `hand` and `classes` on
  single lines, which is what `conformance.rs::load_corpus_table`'s textual read
  and the load assertion need, and it declares `hand = "right"` against the
  shipped `left.toml` — distinct hands, so the scratch directory is a complete
  pair. Every invalid fixture is refused without panicking: `table-header` (line
  starts with `[`), `dotted-key` ("expected `=` after the key"),
  `multi-line-array` (unterminated array), `float-value` and `hex-integer`
  (`finish()` sees the leftover `.`/`x`), `missing-version`
  (`UnsupportedVersion { declared: None }`, and `declares_version` is false for
  it), both `unknown-entry` fixtures and `unknown-hand` (`InvalidValue` naming the
  key), `self-deriving-chain` (`Cycle`), `chain-past-max-depth` (`ChainTooDeep`),
  `nesting-past-limit` (`NestingTooDeep`), `forged-log-line` (one line, escaped).
- **Boundary arithmetic.** `parse_budget.rs::document_of_exactly` pads to exactly
  `MAX_DOCUMENT_BYTES` (and `+1` for the refusals), `reach_tables.rs:193-194`
  pads to `budget + 1`, the nesting fixture holds 17 array levels against the 16
  bound, and the chain fixture 33 links against `MAX_CHAIN_DEPTH = 32`. No
  subtraction in that arithmetic can underflow at these values.
- **The `gate_contract.rs` pins against the shipped files**, re-walked through
  that suite's own parsing helpers: `job_blocks` yields `["rust",
  "orchestration"]` in that order; the `rust` block contains the three gate
  commands verbatim and no `services:`/`DATABASE_URL`/`postgres` on an
  executable line; `report_capture` classifies `cargo deny --format sarif check
  > cargo-deny-report.json` as `sarif`; nothing puts `--format` after `check`;
  the `"runs"` marker is present at `advisories.yml:95`; no non-comment line
  holds `[ ! -s`; the redirect target `cargo-deny-report.json` matches the
  upload step's `path:`; every `uses:` reference is 40 hex characters;
  `node-version: "22.23.4"` equals `tools/orchestration/package.json`'s
  `">=22.23.4"`; `channel = "1.85.0"` sits on `rust-version = "1.85"`; and
  `members = ["crates/*"]` matches `crates/core/Cargo.toml`.

## Deviations from the proposal

- **Workspace members.** The proposal's snippet says
  `members = ["crates/*", "apps/*"]`. Shipped: `members = ["crates/*"]`, with
  a comment stating when `apps/*` joins. A glob that matches no member makes
  cargo abort every gate command at manifest load, and git tracks no empty
  `apps/` directory, so the proposal's glob is the target state, not this
  tree's state (round-1 testing blocker 2).
- **Parser crate.** The proposal names no TOML parser for `bindforge-core`
  and reserves `toml_edit` for ADR-012. To keep the committed `Cargo.lock`
  dependency-free and the pinned toolchain fully hermetic (Test strategy #5's
  "nothing installed beyond the pinned toolchain", taken literally: no
  crates.io fetch needed), parsing is an in-tree minimal reader
  (`crates/core/src/toml.rs`). **The accepted grammar is a documented subset
  of TOML v1.1.0, not all of it** (round-1 correctness finding 2): every
  document it admits is legal TOML, and it refuses table headers, dotted
  keys, multi-line arrays, floats, and hex integers. That narrowing is stated
  in the contract itself — `schemas/README.md` and both `spec.md` grammars,
  which now enumerate keys, escapes, and integer rules — so a third-party
  implementer reads the same boundary, and each excluded construct has an
  invalid fixture. It is a gap against the proposal's
  "Formats are TOML over TOML v1.1.0" phrasing, which the contract tree now
  spells out more precisely than the proposal does. It partly narrows and
  partly widens toward TOML: `\uXXXX`/`\UXXXXXXXX` escapes are now decoded
  (TOML-legal, previously refused), because a serializer that cannot write a
  control character as an escape produces text its own reader refuses.
- **Reach-classes fixtures: parse and load.** The proposal says reach-class
  table fixtures are "asserted by load + distinctness (Test strategy #4)".
  What this scaffold asserts: the harness walk (`conformance.rs`) parses and
  grammar-checks every corpus fixture, and for a reach-classes table it also
  runs the load half — `load_corpus_table` stands the fixture in for its
  declared hand inside a scratch directory, completes the directory with this
  version directory's own table for the other hand, and asserts the loaded
  classes come back filed under the declared hand in table order (round-1
  correctness finding 3, closed by the test pass). Per-hand distinctness of the
  version directory's own `left.toml`/`right.toml` stays `reach_tables.rs`.
  `schemas/reach-classes/v1/spec.md` states both halves. The load assertion
  itself landed in the test pass (`tests/conformance.rs:122` now maps classes
  through `class.id.clone()` against the textually-read id list, per round-1's
  dissent pass), so it builds pending the first gate run — machine evidence
  still belongs to the checklist above.
- **Input budget enforcement point.** `MAX_DOCUMENT_BYTES = 1 MiB` is checked
  on the raw text *and* — for file and stream inputs — before the read:
  `parse_file` metadata-checks the length, `parse_reader` takes at most
  `MAX_DOCUMENT_BYTES + 1` bytes, and the reach loader goes through
  `parse_file`, so no in-tree reader allocates an over-budget document. Both
  budget constants are re-exported beside `MAX_CHAIN_DEPTH`, and
  `schemas/README.md` carries the pre-read requirement (and the hub upload
  policy's `Content-Length` check, ADR-014). The 16-level nesting guard
  counts array levels because the v1 grammars admit no inline tables or table
  headers (those are rejected outright), so "16 table/array levels" is
  realized as "16 array levels plus rejection of everything table-shaped".
- **Closed entry sets.** `spec.md` already said "Any other entry is rejected";
  the reader now does (typed `CoreError::InvalidValue`, key named). Chosen
  over a dedicated `UnknownEntry` variant, which the review allowed as the
  alternative.
- **Error API spelling.** The proposal fixes `Cycle`, `ChainTooDeep`,
  `MAX_CHAIN_DEPTH`, and the test file fixes `CoreError::UnsupportedVersion`;
  the remaining `CoreError` variant names (`Malformed`, `NotUtf8`,
  `MissingEntry`, `UnknownFormat`, `InvalidValue`, `DocumentTooLarge`,
  `NestingTooDeep`, `Io`) and `resolve_chain`'s `Ok` payload (`Vec<String>`,
  profile-first order) are implementation choices in the space the proposal
  leaves open. `Display` renders every untrusted string through one
  truncating, control-character-escaping helper (bound: 80 characters), and
  `Io`'s `path` is reachable only through `source_path`, never through the
  message: rejection text is a log line the uploader controls otherwise.
- **Reach-table file layout.** The proposal fixes "tables as TOML data" in
  `schemas/reach-classes/v1/` but not file names. Chosen: `left.toml` and
  `right.toml`, each a one-hand document whose `hand` entry must agree with
  its file name; the loader rejects disagreement, naming the file and the
  wrong hand but never a local path. The rule is also exposed as
  `hand_matches_file_name`, a pure function, because the conflict cannot be
  pinned by a corpus fixture: a fixture's own file name is never the hand the
  loader is looking for, and a `hand-mismatch.toml` under `fixtures/invalid/`
  would parse as a perfectly valid stand-alone table and fail the harness.
  Class ids in the shipped tables are explicit placeholders pending ADR-009's
  taxonomy (stated in `spec.md` and the files' comments).
- **`deny.toml` waiver key.** The proposal's prose says "a `deny.toml` `skip`
  names the advisory id it waives"; cargo-deny spells the advisory waiver
  `ignore` under `[advisories]` (`skip` is its `[licenses]` key). `deny.toml`
  uses `ignore` and says so in a comment.
- **CI pins and the scan's toolchain.** `actions/checkout` pinned at
  `11bd7190…` (v4.2.2), `actions/setup-node` at `49933ea5…` (v4.4.0),
  `actions/upload-artifact` at `cf430e03…` (v7.0.2), `cargo-deny` at 0.20.2,
  advisories cron Mondays 03:17 UTC. cargo-deny
  0.18.4+ declares MSRV 1.88.0, above the product pin, so the scan installs
  its own `1.88.0` toolchain and runs the tool under `RUSTUP_TOOLCHAIN` — the
  product pin stays 1.85.0, and the scanner's toolchain is deliberately
  independent of it (A25's MSRV check applied to the scanner itself). The job
  runs the full `cargo deny check` so every `deny.toml` section has a
  consumer, with the shared `--format` option in the root position the pinned
  0.20.2 accepts, spelled `--format sarif` because that is the format the
  pinned tool writes to stdout — with `--format json` every record
  (diagnostics, log records, summary) goes to stderr, so a stdout redirect
  captures nothing (round-1c2 security finding; both stream facts verified
  against the 0.20.2 sources — `common.rs` wires the JSON stream to stderr
  with the stdout variant commented out, `check.rs` prints SARIF under the
  comment "Output to stdout for SARIF format", and the SARIF document always
  serializes a `"runs"` key, zero findings included). The report is captured
  from that stdout redirect, content-guarded on `"runs"` (non-emptiness is
  not the bar: a run that dies loading `deny.toml` or the advisory database
  still emits an error line), and uploaded as a `cargo-deny-report` artifact
  so the week's triage record outlives the runner (round-1c2 security
  finding 3; the job adds `actions: read` to its `contents: read` for the
  upload). The proposal's `cargo deny check advisories` phrasing is narrowed
  both ways — the whole check, not only the advisories section, and the
  shared option before the subcommand. `check.yml` triggers on
  `pull_request` only — the design's "two independent jobs on every PR".
- **`Cargo.lock` provenance.** Hand-authored (lock v3, the single path
  member, no external deps because the workspace has none); first-party
  toolchain runs regenerate/confirm it on the first dependency-adding PR.
  `rust-toolchain.toml` lists only `channel` (rustfmt/clippy come with the
  default rustup profile).
- **Rustfmt/clippy not machine-verified here** (no cargo/network access in
  these stages). Round 1's reported `src/` drift is fixed by hand:
  `supported_majors().contains(&version)` for `clippy::manual_contains`, the
  hand-wrapped calls in `document.rs`/`toml.rs` collapsed to the width rustfmt
  produces, `use` trees sorted, and no format argument left un-inlined. The
  round-1c1 `use std::collections::BTreeMap;` joins `toml.rs`'s std group in
  rustfmt's sorted position (`collections` before `fs`), matching
  `document.rs`. Any residual drift will surface on the first CI
  `cargo fmt --all -- --check`; applying that command's own output is part of
  the outstanding checklist above.

## Round-1 fixes applied

- Correctness F1 / security 2 — closed entry sets for both formats, typed
  refusal naming the offending key, plus `unknown-entry.toml` in both
  corpora.
- Correctness F2 — the subset is now stated in `schemas/README.md` ("a
  documented subset of TOML v1.1.0") and enumerated in both `spec.md` files;
  recorded above.
- Correctness F3 — `schemas/reach-classes/v1/spec.md` states what the
  scaffold asserts by parse/grammar and what it asserts by load; recorded
  above.
- Correctness F4 / security 5 / testing 4 — `nesting-past-limit.toml` puts
  the nesting bound into the language-neutral corpus; the size bound cannot
  be a committed fixture and needs the in-memory unit suite listed below.
- Correctness F5 — `clippy::manual_contains` fixed; fmt-shaped `src/`; the
  test-file half (`conformance.rs` un-inlining `{kind}`) landed in the test
  pass.
- Correctness F6 / testing 6 — `parse_integer` now follows TOML's decimal
  rule (no trailing or doubled `_`, no leading zero, no radix prefixes);
  `parse_key`'s bare-key rule and quoted keys are documented as TOML's own,
  so the module doc no longer claims more than it does.
- Correctness F7 / security 4 — both budget constants re-exported;
  `parse_file`/`parse_reader` added; the reach loader uses them.
- Security 1 — advisories scan runs on its own pinned toolchain, with a
  header note and a step that fails loudly when the scan produced no report.
- Security 3 — one truncating/escaping helper for every untrusted string in a
  rejection; `Io.path` kept out of messages; `forged-log-line.toml` in the
  corpus.
- Security 6 / 7, testing 8 — `timeout-minutes` on all three jobs; the scan
  runs advisories + bans + licenses + sources, matching `deny.toml`.
- Testing 2 — `members = ["crates/*"]`.
- Testing 7 — `quoted()` escapes `\b`, `\f`, and any other control character
  (as `\uXXXX`), and `escaped-strings.toml` exercises a non-empty `derives`
  with `\"`, `\\`, `\t`, `\b`, `\f` in the id.
- Testing 6 — invalid fixtures added for every excluded TOML construct
  (`table-header`, `dotted-key`, `multi-line-array`, `float-value`,
  `hex-integer`); the loader's conflict rule is now a pure function.

## Round-1c1 fixes applied (round-1c1 stage)

- Correctness blocker 1 / testing blocker 1 — `crates/core/src/toml.rs` imports
  `std::collections::BTreeMap`. `read_document`'s return type and its
  `BTreeMap::new()` had no binding behind them once the F1/F7 refactor moved
  that function out of `document.rs` (which imports it); `BTreeMap` is not in
  the 2024-edition prelude, so the lib target failed E0412 and every target
  downstream with it. This was the only compile error in `src/`.
- Security major (round-1 finding 1, still open by a new mechanism) — the scan
  step now runs `cargo deny --format json check`. In cargo-deny 0.20.x the
  shared options are root-level arguments: `--format` is defined on the
  top-level command, not on `check`, so the old flag order was a clap parse
  error. clap exited before any check ran, stdout was empty, the redirect left
  `cargo-deny-report.json` at 0 bytes, and the "produced no report" step made
  the weekly job red forever with nothing ever scanned — the design's "only
  recurring look at the lockfile" and its toolchain-advisory trigger both
  unmet. A comment at the command records that in 0.20 the shared options sit
  before the subcommand, so the next edit cannot re-introduce the old order.
- Correctness nit 4 — `schemas/reach-classes/v1/spec.md`'s Fixtures section now
  states that the harness also loads each valid corpus table and files its
  classes under the hand the document declares, and this file's deviation
  entry describes the shipped harness rather than the pre-fix one.

## Round-1c2 fixes applied (this stage)

- Security major (round-1 finding 1, third mechanism, same outcome) — the
  scan step now runs `cargo deny --format sarif check` with a stdout
  redirect, and the guard step requires the captured file to hold a SARIF
  run document (`grep -q '"runs"'`) instead of merely being non-empty.
  Verified against the pinned 0.20.2 sources, not just the review: with
  `--format json`, `common.rs`'s `DiagPrinter::new` wires the JSON stream to
  `StdioStream::Err(stderr)` — its stdout variant is commented out — and
  `stats.rs` writes the `{"type":"summary"}` record to stderr too, so the
  previous line left `cargo-deny-report.json` at 0 bytes every week and the
  guard called a scan that *had* run (with its findings thrown away on
  stderr) a scan that never ran. SARIF is the format 0.20.2 writes to stdout
  (`check.rs` prints it under the comment "Output to stdout for SARIF
  format"), its serializer always writes the `"runs"` key — zero findings
  included — and stdout carries nothing else, so the guard's marker means
  exactly "the checks ran" and a run that died loading `deny.toml` or the
  advisory database is still refused. The header note and step comments now
  state which stream each format reports on.
  `.opencode/skills/repo-practices/SKILL.md`'s scan bullet, which spelled
  the old `--format json` command, was updated in lockstep.
- Security minor 3 (no durable triage record) — the report is uploaded as a
  `cargo-deny-report` artifact via SHA-pinned `actions/upload-artifact` at
  `cf430e03…` (v7.0.2, newest stable on the GitHub tags API at fix time),
  the job's `permissions` gain `actions: read` (the artifact-service scope
  the upload needs), and the `head -c 4000` output stays only as a log
  preview.
- Testing blocker 1 — the `deny.contains(format!(..))` at
  `tests/gate_contract.rs:330` was reproduced from the source as described
  (`String` is not a `str::contains` `Pattern`, hence E0277 and a
  permanently unbuildable `gate_contract` target). Re-running the
  reviewer's grep over the whole workspace finds it as the single remaining
  hit of that shape; `tests/common/mod.rs:316`'s `char` pattern stays as
  is. The fix is one character (`&format!`) but lives on the test path —
  listed below for the test pass, not edited here.
- Testing major 2 — unchanged by this stage and honestly outstanding: the
  shell is permission-denied here too, so no gate command has been run and
  the three dated bullets below stay unchecked. With item 1 below unfixed,
  no such run could pass today either; that is the same single problem the
  dissent pass names.

## Round-1c4 fixes applied (this stage)

Round-1c3's verdict carried one blocker and one major, and both live outside
what an implementation stage may edit or execute. This stage's diff is this
file plus no source change; that is deliberate, and recorded here so the next
seat does not read it as a no-op pass:

- **Testing blocker 1 (round-1c3 finding 1) — confirmed from the sources, fix
  specified verbatim, flagged for the test pass.** `ScratchDir::write`
  (`crates/core/tests/common/mod.rs:199-203`) is a bare `fs::write`: it creates
  the file, never the parent. `tests/conformance.rs:184` creates only
  `empty-valid/v1/fixtures/valid`, so `:185`'s write into
  `empty-valid/v1/fixtures/invalid/` hits ENOENT and the helper's own
  `panic!` fires; `:186` writes into `empty-invalid/v1/fixtures/valid/` before
  its `make_dir` at `:187`, and `:205-206` writes into
  `profile/v1/fixtures/{valid,invalid}/` with neither parent existing. So
  `cargo test --workspace --locked` fails deterministically, the `conformance`
  target never completes, and Test strategy #1's zero-fixture rule has a trigger
  test that cannot fire. Nothing in `src/` can fix this: `ScratchDir` is a
  test-only helper, so the change belongs to the test pass and is written out
  in full in "Test changes the next stage must make" below. The stage verdict
  sets `testsNeedUpdate` for that reason.
- **Testing major 2 (round-1c3 finding 2) — named as the blocker it is.** The
  "Gate verification status" section is now that blocker, stated as one: this
  stage's shell tool is permission-denied too (confirmed by attempting it), so
  no gate command has ever run against this tree and none of the three dated
  bullets can be closed from inside a stage. The section also identifies, for
  each bullet, the environment that *can* produce the evidence — the PR's
  `check.yml` `rust` job for the Linux leg, the Actions UI for the dispatch, a
  Windows contributor for the second leg — so the blocker has an owner instead
  of being a note about tooling.
- **Same-class sweep, so the next gate run does not trip a second runtime
  break.** Recorded in "Runtime-break audit": all 29 `ScratchDir` call sites
  (only the four `conformance.rs` writes are nested), every filesystem read a
  suite performs (all resolve to files present in a fresh clone), both corpora
  against the reader and serializer, the boundary arithmetic, and every
  `gate_contract.rs` workflow pin re-walked against the shipped
  `check.yml`/`advisories.yml`/`Cargo.toml`/`deny.toml`/`rust-toolchain.toml`/
  `tools/orchestration/package.json`. No second defect of that class found, and
  no source, contract, or workflow change was warranted by it.
- **No deviation from the proposal was introduced.** The proposal's Gates block,
  the Test strategy's #1/#5/#6/#7 obligations, and Rollout 2/3/4/7 stand as
  implemented; the outstanding items are the test edit and the dated evidence,
  both named above.

## Test changes the next stage must make (tests are not ours to edit)

**Status (2026-11-11): item 1 below has landed and this section is kept as
the record of what was prescribed.** The test-update stage applied the
`ScratchDir::write` fix verbatim (`common/mod.rs:198-211`), and the
round-1c5 ratification pass confirmed it at HEAD — the `conformance` target's
`test result:` line still belongs to the Test #5 bullet of "Gate
verification status", which no stage can fill. The optional
`gate_contract.rs` narrowing at the end of this section was left untouched;
security-r1c5 carries it as an open nit, to be taken "on the next touch" of
that file.

Round-1c1's four test changes and round-1c2's three all landed and were
verified closed by the round-1c3 dissent pass. One remains, and it gates every
`cargo test --workspace --locked` run:

1. `crates/core/tests/common/mod.rs:198-203` — **blocker**: `ScratchDir::write`
   must create the parent directory before writing, or
   `tests/conformance.rs::the_zero_fixture_rule_fires_on_an_empty_fixture_directory`
   panics at ENOENT (round-1c3 finding 1). Apply this, leaving the existing
   `fs::write` line unchanged:

   ```rust
   /// Writes one file into this scratch tree and returns its path. The parent
   /// is created first, so a suite can lay out a nested fixture corpus in any
   /// order; a bare `fs::write` fails at ENOENT on a directory the suite has
   /// not stood up yet, which is how the zero-fixture rule's trigger test
   /// panicked before it could assert anything.
   pub fn write(&self, name: &str, text: &str) -> PathBuf {
       let path = self.path.join(name);
       if let Some(parent) = path.parent() {
           fs::create_dir_all(parent)
               .unwrap_or_else(|err| panic!("creating {}: {err}", parent.display()));
       }
       fs::write(&path, text).unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
       path
   }
   ```

   `fs` is already imported at the top of the module, and the block above is
   rustfmt-shaped, so `cargo fmt --all` should not move it. No `clippy` allow is
   needed for it.
   - The alternative the reviewer allowed — `make_dir` each fixture directory
     before writing in `conformance.rs:184-187,205-206` — also works but has to
     reorder `empty-invalid` (its dirs must exist before `:186`) and leaves the
     helper order-dependent for the next suite that uses it. Fix the helper.
   - Do not change the assertions in that test: they are Test strategy #1's
     zero-fixture trigger, and with the helper fixed they need no other edit.
   - After the edit, run the gate and paste the `conformance` target's
     `test result: ok` line (and the other targets') into the Test #5 bullet of
     "Gate verification status". That bullet is the evidence the two open
     testing majors ask for; until it is filled in, Rollout 7 stays unticked.

Optional, not required by any open finding — the security seat's r1c3 review
approved with one nit against `tests/gate_contract.rs`: the guard-of-guards
marker assertion is file-wide (`scan.contains("\"runs\"")`), and the forbidden
size-test spelling is the literal `[ ! -s`, so a future `[ -s ]`, `wc -c`, or
`stat -c%s` regression inside the guard step would not be pinned out. It lives
on the test path, the shipped workflow is correct today, and the seat called it
a nit — the test pass may narrow it while it is in that file, or leave it.

## Round-1c5 outcome and current state (as-built pass, 2026-11-11)

What moved in the tree since the round-1c4 section, and the record this stage
completes (it is also the stage that wrote the dated "As-built" note in
`02-proposal.md`, the design-side twin of "Deviations from the proposal"
above):

- **The one open test edit landed.** The `test: 008-rust-for-the-product
  test-update stage` commit is the sole source/test change since the c4
  notes: `ScratchDir::write` now creates the parent, verbatim the block
  prescribed above. The zero-fixture trigger test
  (`conformance.rs::the_zero_fixture_rule_fires_on_an_empty_fixture_directory`)
  now reaches its assertions on every path. Sections above that present the
  fix as still owed (the third bullet of the run-blocker list, and the
  "Test changes" section) carry dated status notes; read this section for
  the current state.
- **Round-1c5 ratification reviewed HEAD: five seats, five approves, zero
  blockers, zero majors** (correctness, security, compatibility,
  performance, testing). Nothing round 1 approved was broken by the
  intervening commits; the c5 reports are the authority for the
  re-verification detail.
- **The gate-evidence blocker is unchanged, including for this stage.**
  This as-built stage's shell allowed only `git diff` retrieval — `cargo`,
  `rustc`, and every other command were permission-denied before executing —
  so the three gate commands still have never run against this tree, and the
  three bullets under "Gate verification status" stay unchecked. Their
  generators are unchanged: the merge PR's `check.yml` `rust` job (Test #5
  Linux leg), a Windows contributor machine (#7), one manual
  `workflow_dispatch` of `advisories.yml`.
- **Open findings at HEAD, carried from the c5 reviews — none is design
  drift, none was fixed by this record stage.** Each names its own next
  touch:
  - correctness minor — `advisories.yml:54-58`: the `actions: read`
    justification is a tool fact that does not hold (an upload produced by
    the same run needs no `actions` scope; if an explicit scope is wanted it
    is `actions: write`). The scope line is inert; the comment is what the
    next touch fixes.
  - correctness nits — the SARIF report is captured as
    `cargo-deny-report.json` although it holds a SARIF document (rename to
    `.sarif` in the redirect, guard, and upload `path:` together);
    `parse_budget.rs:238` matches the unit variant `CoreError::NotUtf8`
    with a struct pattern, a default-on clippy lint on 1.86+ that would
    turn the anticipated MSRV floor bump into a red gate.
  - security nits — `ScratchDir`'s join has no confinement check against an
    absolute or `..`-carrying `name` (hardening for the next touch of the
    helper); the `gate_contract.rs` marker-assertion scoping nit carries
    unchanged from round 1.
  - compatibility minor — the escape family the contract now promises
    (`\uXXXX`/`\UXXXXXXXX`, surrogate and past-`U+10FFFF` refusals, the
    writer's `\uXXXX` branch) has no corpus fixture or test, so a
    third-party implementer can get it wrong and still pass conformance.
    Nits — `load_reach_tables` never passes the on-disk file-name stem
    through `hand_matches_file_name` (the `Left.toml` case loads on Windows
    and fails on Linux); no `.gitattributes`, and no contract sentence on
    CRLF tolerance.
  - performance minor/nits — `toml.rs`'s per-line `Vec<char>` allocation
    and the small per-integer/per-entry allocations (all bounded by the
    1 MiB budget; explicitly non-blocking).
- **Design drift is now recorded on both sides**: `02-proposal.md`'s dated
  As-built section (members glob, hermetic subset parser, budget
  pre-read point, reach-table file names, error-variant spelling, scan
  scope/format/toolchain, `ignore` vs `skip`, the extra permission scope,
  Rollout 7 outstanding) and this file's "Deviations from the proposal"
  above. No new deviation entered with the test-update commit or with this
  record.


