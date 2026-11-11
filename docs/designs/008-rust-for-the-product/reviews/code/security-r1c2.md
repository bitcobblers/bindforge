# Code review — security, round 1 dissent pass c2

Scope: `git diff a8b3aeda622ff4da60863c46e704091e6a74707b..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md` (Gates, Adversarial guard, TypeScript boundary) and
ADR-008. Specialty: security checklist only. This pass checks first whether the
last revision closed my round-1 major, then anything the changed code makes true.

## Prior finding: still open

Round-1 major (the scheduled lockfile scan is a dead control) is **not closed**.
The half of the fix that moved `--format` to the root position is correct and
verified — but the report capture on the same line still produces an empty
report, so the weekly job is still red forever and the design's "only recurring
look at the lockfile" (plus its toolchain-advisory bump trigger) is still
unmet. Third mechanism, same outcome.

## Verification of the tool facts (pinned cargo-deny 0.20.2)

- `src/cargo-deny/main.rs` (tag 0.20.2): `Args::command()` defines
  `Arg::new("FORMAT").short('f').long("format")` on the **top-level** `Command`,
  without `.global(true)`; `src/cargo-deny/check.rs` `Args::cmd()` lists only
  `--graph`, `--hide-inclusion-graph`, `--audit-compatible-output`,
  `--show-stats`, `-W/-A/-D`, `--feature-depth`, and the positional `WHICH`.
  So `cargo deny --format json check` parses and
  `cargo deny check --format json` is a clap parse error. The round-1 flag-order
  defect is genuinely fixed.
- The same source shows where JSON output goes, and this is what the fix missed:
  with `--format json`, `common.rs` `DiagPrinter::new` builds
  `OutputFormat::Json(Json { stream: StdioStream::Err(std::io::stderr()) })`
  (`StdioStream` has no stdout variant — it is commented out), and
  `OutputLock::Json(..)` serializes each diagnostic pack into that stderr lock.
  `main.rs` `setup_logger(Format::Json)` chains log records to
  `std::io::stderr()` as `{"type":"log"}` objects. `stats.rs` `print_stats`
  writes the `{"type":"summary"}` record to stderr, with the comment "for JSON
  we still go to stderr since presumably computers will be looking at that
  output". The only stdout writes are `--audit-compat`'s `println!` (advisories
  only) and, for `--format sarif`, the SARIF document in `print_diagnostics`
  (`// Output to stdout for SARIF format`). `cargo metadata`/`cargo fetch`
  output is captured by the tool, not passed through.
  => With `--format json` and `> cargo-deny-report.json`, stdout carries nothing:
  the file exists at 0 bytes, `[ ! -s cargo-deny-report.json ]` is true, and the
  next step exits 1.
- `cargo-deny` 0.20.2 is the newest stable on crates.io and `rust_version` is
  `1.88.0` (so `SCAN_TOOLCHAIN: "1.88.0"` is right); OSV has no crates.io
  advisory for `cargo-deny` itself. No product dependency is added
  (`Cargo.lock` holds only `bindforge-core`).

## Checklist

1. Boundary validation of untrusted files — pass. `crates/core/src/toml.rs:54-95`
   applies `MAX_DOCUMENT_BYTES` before allocation for both file and stream
   inputs; `read_document` re-checks, nesting is capped
   (`MAX_NESTING_DEPTH`), entry sets are closed. The only `src/` change in this
   revision is the `use std::collections::BTreeMap;` import at `toml.rs:28`,
   which is what `read_document`'s signature needs under edition 2024.
2. Path handling — pass at this scope. No path from untrusted data is
   interpolated into a shell string; `toml.rs:66-71` keeps the read path on the
   variant and out of the message. Traversal/absolute-path rejection is
   Deferred to ADR-012 per the proposal, unchanged by this diff.
3. Injection — pass. The workflow interpolates only its own pinned env values,
   quoted (`advisories.yml:48-50`, `:64-65`); no user string reaches a shell,
   and no regex runs over untrusted input.
4. Secrets/PII — pass; none in the diff.
5. New dependencies — pass on the dependency itself (pinned, `--locked`, newest
   stable, no known advisory), **fail on the control that consults advisories**
   → finding 1.
6. Filesystem writes — pass. The report is a single redirect in job scratch;
   no partial-state risk.
7. Clean rejection on hostile input — pass. Typed variants only, `one_line`
   (`error.rs:141-155`) bounds and control-character-escapes every untrusted
   fragment, and `Io.path` stays off the message.

## Findings

1. **major** — `.github/workflows/advisories.yml:63-65` (with `:66-74`).
   `cargo deny --format json check > cargo-deny-report.json || true` writes an
   empty file: in cargo-deny 0.20.2 all `--format json` output (diagnostics, log
   records, and the summary) goes to stderr, not stdout (see verification
   above). `[ ! -s cargo-deny-report.json ]` therefore fires every Monday, the
   job is permanently red, no report artifact ever exists, and the design's
   only recurring look at the lockfile — advisories, bans, licenses, sources —
   still yields nothing the maintainer triages; the toolchain security-advisory
   bump trigger stays unmet. It is also now actively misleading: the
   `::error::` text says "the scheduled lockfile scan did not run" when the scan
   did run and its findings were thrown away on stderr. The step comment at
   `:51-62` claims the opposite ("the report still has to exist"), so the file
   argues for a behavior the pinned tool does not have.
   Fix: capture the stream the pinned tool actually writes the report to, and
   check the report's content rather than only its size. Cleanest, because it
   keeps "empty stdout ⇒ the scan never ran" exact:
   `RUSTUP_TOOLCHAIN="$SCAN_TOOLCHAIN" cargo deny --format sarif check > cargo-deny-report.json || true`
   (0.20.2 writes SARIF to stdout), and the guard step fails unless the file
   holds a SARIF document, e.g. `grep -q '"runs"' cargo-deny-report.json`.
   If JSON is kept, redirect stderr instead (`2> cargo-deny-report.json`) and
   require the `{"type":"summary"}` record so a run that died loading
   `deny.toml` or the advisory DB — whose error is itself a stderr line — is not
   counted as a report. Update the step comment and header note to state which
   stream cargo-deny 0.20 writes its report to.
2. **minor** — `crates/core/tests/gate_contract.rs:298-313`. The strengthened
   pin asserts the literal `cargo deny --format json check` and forbids
   `--format` after `check`; the still-broken command satisfies both, so the
   guard the round-1 fix asked for passes while the control stays dead — and
   the substring pin would fail a `--format sarif` fix. Nothing pins where the
   report comes from. Fix: assert the capture as well — the scan line must
   redirect the stream cargo-deny writes its report to (`2>` alongside
   `--format json`, or a stdout redirect alongside `--format sarif`), and that
   the guard step tests a content marker (`"type": "summary"` / `"runs"`)
   rather than mere non-emptiness. Spell the assertion to accept either
   format so the correct fix does not break the suite.
3. **minor** — `.github/workflows/advisories.yml:75` (and the absence of any
   upload step). The week's triage record is neither durable nor complete: the
   only view of the report is a 4 000-byte log preview, which truncates any
   real finding set, and the report file dies with the runner, so "the
   maintainer triages what it reports within the week" (Gates) has nothing to
   triage from later. Fix: upload the report with a SHA-pinned
   `actions/upload-artifact` step and keep `head` as a log preview only.

## FYI (no severity, out of specialty)

- The proposal's Gates prose still says a `deny.toml` `skip` names the waived
  advisory; cargo-deny spells that waiver `ignore` under `[advisories]`, and
  both `deny.toml:9-10` and `repo-practices/SKILL.md:70-72` say so. The code is
  right and the design sentence is the one that needs the doc-sync correction.
- `03-build-notes.md:252-296` records that `tests/conformance.rs:119` and
  `tests/reach_tables.rs:111-125` still fail to compile, so
  `cargo test --workspace --locked` — and with it every `gate_contract.rs`
  assertion, including the scan pin above — does not run yet. That belongs to
  the testing seat and is already listed there; noting it only because it
  bounds what any test-side fix in this area can currently prove.
