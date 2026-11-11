# Code review — correctness seat, round 1c5 (ratification)

Scope as given: `git diff 42d019b292b97c70c49f656a0b8d5304bf9b5008..HEAD -- ':(exclude)docs/designs'`
(5 files: `.github/workflows/advisories.yml`, `.opencode/skills/repo-practices/SKILL.md`,
`crates/core/tests/common/mod.rs`, `crates/core/tests/gate_contract.rs`,
`crates/core/tests/parse_budget.rs`), read against the full current `02-proposal.md` and
`docs/adrs/008-rust-for-the-product.md`. Checklist: `references/correctness.md`.
Question for this pass only: did the revisions that landed after the round-1 approval break
what was approved?

Verdict: **approve**. No blocker and no major. The three tool facts this round's fix hangs on
are true of the pinned tool, the pins it adds are real, and the new test logic is satisfied by
the shipped files. Findings left open are one minor and two nits.

## What I checked, and what it came out as

1. **The SARIF rewrite is factually right about the pinned cargo-deny 0.20.2** (the whole point
   of the c2 fix, so it had to be re-verified independently, not trusted). Upstream 0.20.2
   `main.rs` parses `--format` with the `Format` enum (`human`/`json`/`sarif`), so
   `cargo deny --format sarif check` is a *valid* command line (SARIF landed in 0.18.5, PR#790 —
   the flag position stays root-level, which the round-1c1 fix pinned). `common.rs`'s
   `DiagPrinter::new` wires the JSON output to `StdioStream::Err(std::io::stderr())` and the
   `Format::Json` logger chains to stderr, so the c1 spelling really did capture nothing on
   stdout; the SARIF branch writes the document to stdout. `sarif/collector.rs`'s
   `generate_sarif` always returns `SarifLog { runs: vec![Run { … }] }`, so the guard's
   `"runs"` marker is present even with zero findings, and with `Format::Sarif` the logger is
   capped at `Error` on stderr — so a scan that died loading `deny.toml` or the advisory DB
   leaves stdout empty and the guard fires. `advisories.yml:26-34,71-83,84-100` and the
   repo-practices bullet at `SKILL.md:74-84` state exactly this. Checklist item 5 passes on
   these claims.
2. **The pins the fix adds are real.** `actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9`
   is the commit the `v7.0.2` tag points at (checked against the GitHub refs API), 40 hex chars,
   so `gate_contract.rs:249-289` and the new `:457-498` assertion are satisfied and the design's
   "pin every third-party action to a commit SHA" holds.
3. **The rewritten scan assertions are satisfiable by the shipped workflow** (static walk
   through the suite's own helpers, since no stage can run a toolchain): `code_lines` drops the
   comment lines that name the forbidden spellings; `report_capture` classifies
   `cargo deny --format sarif check > cargo-deny-report.json || true` as `sarif`;
   `split_once("check")` leaves `> cargo-deny-report.json || true`, which carries no
   `--format` and no bare section word, so the root-position and whole-check assertions pass;
   `"runs"` appears at `advisories.yml:95`; no non-comment line contains `[ ! -s`;
   `redirect_target` yields `cargo-deny-report.json`, which the upload step's `path:` repeats,
   and the upload step's `uses:` line has no `- ` prefix so `action_references` is not
   polluted — the new test checks that SHA itself. `deny.toml`, `check.yml`,
   `rust-toolchain.toml` and `tools/orchestration/package.json` pins are untouched by this diff.
4. **The new `parse_budget.rs` tests compile against the real API and assert real behavior.**
   `parse_reader(impl Read)` takes `Cursor<&[u8;4]>`; `toml.rs:93` refuses non-UTF-8 with
   `CoreError::NotUtf8` (no lossy path); `toml.rs:66-71` puts the path only on
   `CoreError::Io`, `error.rs:71-76` exposes it through `source_path`, and `error.rs:128-130`
   keeps it out of `Display`, so `an_io_refusal_names_its_file_only_through_source_path`
   matches the implementation. `deny.contains(&format!(…))` at `gate_contract.rs:408` is the
   one-character fix for the E0277 the c3 pass named (`&String` is a `str::contains` pattern,
   `String` is not), and it is the only hit of that shape left in the diff.
5. **`ScratchDir::write` (`common/mod.rs:203-211`) is the fix the c4 notes prescribed, verbatim,
   and it repairs the whole class rather than one call site.** `conformance.rs:185-186,205-206`
   are the only nested writes in the suite, and with the parent created first the zero-fixture
   trigger test lays out `empty-valid`/`empty-invalid`/`complete-corpus` as its assertions
   assume. No test depends on a write failing. `#![allow(dead_code)]` at `common/mod.rs:33`
   means the helper is not dead-code-warned in the targets that don't use it.
6. **Scope of this diff against the proposal**: the three gate commands, the two PR jobs, the
   Node floor, the toolchain pin, the `schemas/` contract tree and the adversarial guard are
   untouched. The two things this diff adds beyond the proposal's Gates wording — the report
   upload step and the job's second permission scope — are both recorded in
   `03-build-notes.md:279-282,384-389`, so they are declared deviations, not undeclared ones
   (checklist items 3 and 4: no blocker).
7. **My round-1 finding stayed closed** in the current notes: `03-build-notes.md:214-218` now
   describes the load assertion as landed in `tests/conformance.rs:122` instead of "does not
   compile yet", and the round-1c1 test items at `:340-361` are recorded as landed rather than
   pending.

## Findings

- **minor — `.github/workflows/advisories.yml:54-58` (and the same claim in
  `03-build-notes.md:281`)**: the added scope's justification is a tool fact that does not
  hold. The comment says `actions/upload-artifact` cannot run without `actions: read`.
  Uploading an artifact produced by the same run needs no `actions` scope at all; the
  `actions` namespace is what cross-run/cross-repo artifact *reads* need (`actions: read`)
  and what deletes/overwrites need (`actions: write`) — which is why the ecosystem convention
  is `actions: read` on downloaders and `actions: write` on uploaders. So the scope is inert
  here, and if the maintainers do want an explicit artifact scope for the uploader, the
  required level would be `write`, not `read` — meaning the comment points a future editor at
  the wrong knob exactly when the `workflow_dispatch` checklist bullet (`03-build-notes.md:110-120`)
  reports a 403 at the upload step. It also takes the job's permissions one line beyond the
  proposal's "Both workflows declare `permissions: contents: read` at the job level" (Gates).
  Fix: drop the causal claim, or make the intent explicit — either delete the `actions: read`
  line (the upload works on `contents: read`) and let the dispatch run prove it, or set
  `actions: write` with a comment naming it as the upload scope and let the design owner
  mirror that in the Gates paragraph. No behavior of the scan changes either way.

- **nit — `.github/workflows/advisories.yml:83,95,108`**: the captured file is named
  `cargo-deny-report.json` but from this round on it holds a SARIF 2.1.0 document, which is
  what the guard greps and what the build notes' checklist bullet expects ("a
  `cargo-deny-report` artifact whose file holds a SARIF document"). A reader who opens the
  artifact with JSON tooling gets a SARIF file with a `.json` name. Fix: name it
  `cargo-deny-report.sarif` in the scan step's redirect, the guard's `grep`/`head`, and the
  upload step's `path:` — the same string in all three places, which
  `gate_contract.rs:457-498` already requires by construction.

- **nit — `crates/core/tests/parse_budget.rs:238`**: `matches!(err, CoreError::NotUtf8 { .. })`
  matches a unit variant with a struct pattern. It compiles on the pinned 1.85.0 toolchain, but
  the same line is a default-on clippy warning (`clippy::unneeded_struct_pattern`, "struct
  pattern is not needed for a unit variant") on clippy 1.86 and newer, and the gate is
  `cargo clippy --workspace --all-targets -- -D warnings`. That turns the design's own
  anticipated floor bump (`Risks`: "floor bump is a deliberate workspace change updating
  `rust-version` and `rust-toolchain.toml` together") into a red gate on brand-new test code.
  Fix: write `matches!(err, CoreError::NotUtf8)`; every other variant match in the suite
  already uses braces only where fields exist.

## FYI (no severity)

`03-build-notes.md:75-82` and `:449-490` still present the `ScratchDir::write` parent fix as
the one test edit a later stage has to make, and `:122-136` audits it as if the guard were
already there; this diff lands it, so those sections are stale against HEAD in the same way my
round-1 finding described. `docs/designs/` is outside this pass's diff, so it is noted here
rather than filed.
