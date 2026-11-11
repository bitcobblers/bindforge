# Security review — design 008, code, round 1 dissent pass c3

- Seat: security
- Range under review: `42d019b292b97c70c49f656a0b8d5304bf9b5008..HEAD -- ':(exclude)docs/designs'`
  (`.github/workflows/advisories.yml`, `.opencode/skills/repo-practices/SKILL.md`,
  `crates/core/tests/gate_contract.rs`, `crates/core/tests/parse_budget.rs`)
- Design: `docs/designs/008-rust-for-the-product/02-proposal.md`, `docs/adrs/008-rust-for-the-product.md`
- Verdict: **approve** (0 blocker, 0 major, 0 minor, 1 nit open)

## Re-check of my round-1 findings

### 1. major — dead control: `--format json` captured from the wrong stream — RESOLVED

Verified against the pinned tool's own source (cargo-deny 0.20.2 tag, raw files):

- `src/cargo-deny/main.rs`: the root `Arg::new("FORMAT")` exists with a `FormatParser`
  accepting `"sarif"`, and `check::Args::cmd()` defines no `--format`, so
  `cargo deny --format sarif check` parses.
- `src/cargo-deny/main.rs` `setup_logger`: for `Format::Sarif` the logger is
  Error-level-only, chained to **stderr** ("suppress regular logs to stderr to avoid
  mixing with SARIF JSON").
- `src/cargo-deny/check.rs` `print_diagnostics`: for `Format::Sarif` the collector
  drains the diagnostic channel and writes the document with
  `std::io::stdout().write_all(...)` — SARIF on **stdout**, unconditionally, after the
  checks complete.
- `src/sarif/collector.rs` + `src/sarif/model.rs`: `generate_sarif` always returns
  `SarifLog { runs: vec![Run { … }] }` — exactly one run even with zero findings — and
  `SarifLog`'s manual `Serialize` always writes the `"runs"` entry (no
  `skip_serializing_if`). So the guard marker `"runs"` is present on every completed
  scan, including a clean one.
- `src/cargo-deny/stats.rs`: for `Format::Sarif` no separate stats record is emitted
  at all (JSON's `{"type":"summary"}` still goes to stderr). stdout therefore carries
  only the SARIF document, exactly as the corrected header/step comments now state.

The workflow (advisories.yml:82-83) now runs
`cargo deny --format sarif check > cargo-deny-report.json || true`, the guard
(:95-98) fails unless the file contains `"runs"`, and the header (:26-34) and step
comments state the stream facts correctly. On any failure path (deny.toml load error,
advisory-DB load error, clap parse error) stdout stays empty — cargo-deny errors go to
stderr and exit before the SARIF write — so red means "the scan really did not run",
not tool misuse. With findings the exit code is a non-zero bitmask, absorbed by
`|| true` after the SARIF document has already been written. Control restored; closed.

### 2. minor — pin accepted the still-broken command — RESOLVED (one nit open)

`gate_contract.rs` now pins the capture itself: `report_capture` (142-152) requires
either `--format sarif` with a stdout redirect or `--format json` with `2>`, and a
scan line matching neither panics the test (345-357); the guard must test a content
marker **matched to the format the scan line emits** — `"runs"` for sarif,
`"summary"` for json (387-397); the `[ ! -s` spelling is forbidden outright
(398-404). Either sanctioned fix passes; the broken stdout-redirect-of-JSON spelling
fails. This is exactly the both-spellings pin my round-1 fix asked for.

Remaining robustness gap — nit, not blocking (see findings): the marker assertion is
file-wide (`scan.contains("\"runs\"")`), so a comment naming the marker could satisfy
it while the guard step itself regressed to a different size test; the forbidden
spelling is the literal `[ ! -s`, so `[ -s ]`-inverted, `stat -c%s`, or `wc -c`
regressions are not pinned out. The current code and tests are correct; this only
narrows how future regressions can slip through this guard of guards.

### 3. minor — report not durable — RESOLVED

`advisories.yml:101-109` uploads `cargo-deny-report.json` with
`actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9`. I verified the SHA
against the GitHub API: it is the cryptographically-verified release commit
"prepare v7.0.2 release" (2026-10-07), matching the `# v7.0.2` comment — a genuine
full-SHA pin, not a floating tag. `actions: read` was added to job permissions
(least privilege — required by upload-artifact, no scope beyond what it needs),
`if-no-files-found: error` makes a vanished report loud, and `head -c 4000` remains as
a log preview only. `gate_contract.rs:456-498` pins the upload (SHA length + hexdigit
shape, and the uploaded path must be the file the scan redirected to). Closed.

## Findings (open at c3)

- **nit** — `crates/core/tests/gate_contract.rs:392-404`. The content-marker assertion
  searches the whole file (`scan.contains`), so a comment mentioning `"runs"` could
  satisfy it while the guard step regressed; and the ban covers only the literal
  `[ ! -s` spelling. A future rewrite could reintroduce a size test (`[ -s ]` inverted,
  `stat -c%s`, `wc -c`) and still pass. Fix (next touch): scope the marker search to
  the guard step's run block (split on its step name), and assert the guard uses a
  content `grep` of the marker rather than only banning one size-test spelling.

## Checklist notes

1. Boundary input: `parse_budget.rs` new tests pin the stream path refusing non-UTF-8
   with typed `NotUtf8` (`toml.rs:93` `from_utf8` — no lossy decode) — security-positive,
   correctly targets existing APIs.
2/3. Path/shell: fixed literal redirect and grep pattern in the workflow; env vars
   quoted; no untrusted data interpolated into shell.
4. Secrets/PII: none in the diff; the artifact carries only SARIF advisory/license data.
   `an_io_refusal_names_its_file_only_through_source_path` additionally pins that the
   local path stays out of `Display` (`toml.rs:66-71` keeps it on the variant) while
   `source_path()` carries it for operators — the redaction contract, both halves.
5. New dependency: `actions/upload-artifact` — justified (finding 3), full-SHA pinned,
   SHA verified against the signed v7.0.2 release commit; no known advisory.
   cargo-deny stays pinned to 0.20.2 on an independent scanner toolchain.
6. Filesystem writes: workflow writes one report file in the runner workspace; no
   product write paths changed.
7. Malformed/hostile input: rejects remain typed and clean; the new tests strengthen
   this rather than weaken it.

## FYI (out of seat, no severity)

- Pre-existing and outside this diff range: `crates/core/src/toml.rs:303`
  (`digits.replace('_')`, E0061 — `str::replace` takes a replacement) makes
  `bindforge-core` fail to compile, so **no** gate command — including the two test
  files changed here — can currently run green. Verified identical at base commit
  `42d019b` (no commit in the range touches `toml.rs`), so it is not a finding against
  this change, but it blocks execution-verification of this pass's fixes and belongs to
  the correctness seat. (An unrelated `unused_mut` at `toml.rs:78` and `cargo fmt
  --check` diffs in 11 files are also pre-existing gate noise.)
- Design-text divergence, also pre-existing: the proposal's Gates paragraph still says
  the scheduled job runs `cargo deny check advisories`, while the workflow runs the
  whole `cargo deny check` and the gate test forbids the one-section spelling. The
  superset is security-favorable (every configured `deny.toml` section has a consumer);
  the proposal text, not the workflow, is what lags.
