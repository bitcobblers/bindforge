# Security code review — dissent pass c1 (round 1 follow-up)

Seat: security. Scope: `git diff ea54086ca6562fb2b1bf69a25bf071479cb09b27..HEAD -- ':(exclude)docs/designs'`,
read against the full current tree, `02-proposal.md`, and `docs/adrs/008-rust-for-the-product.md`.
This pass checks the revision written to answer my round-1 findings first; anything else is reported
only because the changed code makes it true.

## Verdict: request-changes

Six of seven round-1 findings are resolved. One is not: the scheduled lockfile scan is still not a
running control — the toolchain half of the fix is right, but the command line as written is not a
form cargo-deny 0.20.2 (the pinned version) accepts, so the scan produces no report and the weekly
job fails red every week instead of scanning.

## Round-1 findings — disposition

### 1. (major) advisories.yml dead control — NOT RESOLVED (new mechanism, see open finding below)

The round-1 defect is genuinely fixed in part: `SCAN_TOOLCHAIN: "1.88.0"` installed via
`rustup toolchain install` with `RUSTUP_TOOLCHAIN` on both install and run (advisories.yml:30-34,
46-56), which is exactly cargo-deny 0.20.2's crates.io `rust_version` (re-verified: crates.io lists
MSRV 1.88.0 for 0.20.2; 0.20.2, published 2026-07-09, is still the newest release, and no
RustSec/GHSA advisory exists for cargo-deny itself). The header comment records the deliberate
independence from the product pin, and the "Fail if the scan produced no report" step
(advisories.yml:57-66) makes a dead control loud. But the scan step's `cargo deny check --format
json` puts `--format` after the subcommand, which the pinned 0.20.2 rejects at parse time — see the
open finding. The control still never runs.

### 2. (major) Open grammar — RESOLVED

`PROFILE_ENTRIES` / `REACH_CLASSES_ENTRIES` closed sets (document.rs:38-43) enforced by
`reject_unknown_entry` (document.rs:151-165) on every known format before format-specific parsing;
a typed `InvalidValue` names the offending entry. The alias case (`derives_`) is refused, so the
fail-open on the chain guards' own input is gone. Pinned in the corpus for both formats
(`schemas/profile/v1/fixtures/invalid/unknown-entry.toml`,
`schemas/reach-classes/v1/fixtures/invalid/unknown-entry.toml`) and in tests
(grammar_closure.rs:71-123); spec.md states the closed set and the rationale
(profile/v1/spec.md:30-34, reach-classes/v1/spec.md:39-40). The revision additionally closed the
syntax half I flagged adjacent to it: table headers, dotted keys, multi-line arrays, floats, and hex
integers are refused (toml.rs:111-116, 171-189, 282-315, 351-358) each with its own invalid fixture
and a corpus test (grammar_closure.rs:130-152). I verified each fixture against the reader by
hand: dotted-key (`author.name`) fails at "expected `=` after the key"; table-header fails at the
`[` guard; multi-line-array fails as unterminated array / key-with-`=`; float and hex fall out at
`finish()` ("unexpected `.`/`x` after the value").

### 3. (minor) Unbounded/unescaped untrusted content in Display — RESOLVED

`one_line` (error.rs:141-155) truncates untrusted fragments at 80 chars with an ellipsis and escapes
backslashes and control characters; every untrusted interpolation in the `Display` impl routes
through it (error.rs:88-130). `Io.path` is out of the message entirely, reachable only via
`source_path()` for server-side logs (error.rs:59-76), and `reach.rs` refusals name only the file
stem. Corpus + unit coverage: `every_invalid_fixture_is_rejected_with_a_single_line_message`
(conformance.rs:215-238) and `a_rejection_truncates_an_untrusted_value_rather_than_echoing_it`
(conformance.rs:244-264), with the `forged-log-line.toml` fixture. schemas/README.md:67-73
advertises exactly this bound. Residual (not a finding): `Malformed` messages embedding an offending
single character (toml.rs:201, 356) also pass through `one_line`, so control characters smuggled via
a quoted key's duplicate-key or trailing-garbage errors are escaped too.

### 4. (minor) Unexported budget constants, cap applied after allocation — RESOLVED

`MAX_DOCUMENT_BYTES` and `MAX_NESTING_DEPTH` are re-exported beside `MAX_CHAIN_DEPTH`
(lib.rs:35) and named in the public-surface doc (lib.rs:17-21). `parse_file`/`parse_reader`
(document.rs:134-145) front the crate's own enforcement: `read_file` metadata-checks length before
allocating (toml.rs:53-61), `read_reader` takes at most `MAX_DOCUMENT_BYTES + 1` bytes (toml.rs:77-94).
`load_reach_tables` now goes through `parse_file` (reach.rs:103-108), closing the unchecked
`fs::read_to_string` I flagged. schemas/README.md:51-57 states the pre-read rule and the hub
upload-policy carry-over for ADR-014. (The metadata-then-read TOCTOU on a locally-writable tree is
out of scope: the attacker already controls that filesystem.)

### 5. (minor) Budget implemented but not pinned by tests — RESOLVED

`crates/core/tests/parse_budget.rs` boundary-tests both halves against the exported constants:
acceptance at exactly `MAX_DOCUMENT_BYTES`, `DocumentTooLarge` at +1 (in memory and via
`parse_file` on a scratch file), `NestingTooDeep` at +1 array level, non-firing at exactly 16 —
mirroring chain_guard.rs. `schemas/profile/v1/fixtures/invalid/nesting-past-limit.toml` (17 levels)
puts the rule in the language-neutral corpus, and grammar_closure.rs:163-172 requires it to reject
as `NestingTooDeep`. Deleting either check in toml.rs now turns the gate red.

### 6. (nit) No timeout-minutes — RESOLVED

`timeout-minutes: 15` on both check jobs (check.yml:21, 39), `30` on advisories (advisories.yml:41),
pinned by `gate_contract.rs:200-241` (every job must declare a timeout and `permissions: contents:
read`; every action pinned to a 40-hex-char SHA — both SHAs verified correct for
checkout@v4.2.2 and setup-node@v4.4.0).

### 7. (nit) deny.toml policies configured but unrun — RESOLVED

The scan runs the whole `cargo deny check` (advisories.yml:51-56; per cargo-deny docs, a bare
`check` runs all four sections), and `gate_contract.rs:293-306` pins that a subcommand-restricted
form may not come back while any deny.toml section exists. The deny.toml header also corrects the
proposal's `skip`/`ignore` spellings (deny.toml:8-10) — config sections now match what actually
runs.

## Open findings

### major — `.github/workflows/advisories.yml:54-56` (with `crates/core/tests/gate_contract.rs:288-291`)

**Problem.** The scan step runs `cargo deny check --format json > cargo-deny-report.json || true`.
In cargo-deny 0.20.x — the pinned `CARGO_DENY_VERSION: "0.20.2"` — `--format` is a **root-level**
argument only: the 0.20.2 CLI source defines `FORMAT` on the top-level `Command` (with `-f`), not on
the `check` subcommand, and without `Arg::global(true)`; the 0.20.0 CLI refactor (PR #881) moved the
previously duplicated per-subcommand options to the root and removed the deprecated per-subcommand
copies, and the book's common-options page now says the shared options are used *before* the
subcommand, while the `check` page's option list (WHICH, `-A/-D/-W`, `--audit-compatible-output`,
`--feature-depth`, `-g`, `--hide-inclusion-graph`, `-s`) contains no `--format`. clap therefore
exits with "unexpected argument `--format` found" before any check runs: stdout is empty, the
redirect leaves `cargo-deny-report.json` at 0 bytes, and the "Fail if the scan produced no report"
step turns the job red **every week** with no scan ever having run. This is round 1's finding 1
still true by a different mechanism: the design's commitment ("the only recurring look at the
lockfile", Chosen design → Gates) and the named trigger for the toolchain security-advisory bump
remain unmet — no advisories, bans, licenses, or sources check executes against `Cargo.lock`. Unlike
round 1 it is now loud rather than silent, and the toolchain half of the fix is correct, but the
control is absent. The test suite does not catch it: `gate_contract.rs:289` only asserts the string
`cargo deny check` appears, and the command line as written satisfies it; the suite stays green
while CI would be weekly-red.

**Fix.** Move the flag to the root position the pinned tool accepts:
`cargo deny --format json check > cargo-deny-report.json || true`. Then update
`gate_contract.rs:288-291` to pin the working form, e.g. assert
`scan.contains("cargo deny --format json check")` (or at minimum that `--format` appears before
`check`), so the regression the suite currently lets through becomes a test failure rather than a
weekly red job nobody bisects. A one-line header comment noting that in 0.20 the shared options sit
before the subcommand would prevent the next edit from re-introducing this.

## Checklist notes (items that passed)

- Boundary validation, size limits, no code eval: closed grammar + closed entry sets + budget,
  hostile input always a typed `CoreError`, never a panic; malformed-hostile-input rejection is clean.
- Path handling: no traversal, no untrusted-derived paths (`load_reach_tables` joins fixed stems);
  no shell anywhere.
- Injection: no shell, no user-controlled regex; log forging closed (finding 3).
- Secrets/PII: none in source, fixtures, or logs; Io paths kept out of user-visible text.
- Dependencies: the diff adds no workspace dependency (in-tree reader keeps the scaffold
  dependency-free); cargo-deny 0.20.2 pinned and current with no known advisory; both actions pinned
  to verified SHAs.
- Filesystem writes: none in product code (ScratchDir is test-only, cleaned in `Drop`).
