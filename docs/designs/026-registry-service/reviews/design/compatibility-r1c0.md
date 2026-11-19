# Compatibility review r1 (pass c0, full panel, scope: whole document)

verdict: request-changes

Inputs used: `docs/designs/026-registry-service/02-proposal.md` (incl. its Revision
log), `docs/adrs/026-registry-service.md`, and read-only inspection of the shipped
code the proposal's compatibility claims rest on (`apps/discovery-cli/src/{catalog,matches}.rs`,
`apps/discovery-cli/Cargo.toml`, `apps/discovery-cli/tests/cli_catalog_gate_tests.rs`,
`crates/core/src/{placement,manifest,lib}.rs`, `crates/core/tests/{conformance,version_gate,slug_bound_corpus,common/mod}.rs`,
`crates/discovery/src/game.rs`, root `Cargo.toml`, `rust-toolchain.toml`,
`schemas/README.md`). No other run record was opened. No shell was available to
this seat (permission denied), so no cargo/npm leg was re-run; every code claim
below was established by reading the shipped source.

## Findings

- [major][new] Compatibility matrix — "Other / exotic Unix (BSD et al.) | unknown |
  Treated by the Unix data-dir branch; `std::net` assumed portable; not exercised by
  any suite here." — The seat's checklist requires every platform row to carry a
  status, and `unknown` is not one: it is the absence of a decision. The matrix's own
  footer makes the gap load-bearing — "the matrix describes the target
  `apps/registry` behavior the tasks must meet" — so a row with no status means
  `T-004`…`T-008` carry no promise (and no refusal) for that row, while the row is
  simultaneously the one the matrix says is handled by the shipped Unix data-dir
  branch. The cell is not empty, so this is major rather than a blocker. Fix:
  replace the row's status `unknown` with the decision `No` — Notes: "not a v1
  target: the Unix data-dir branch and `std::net` may carry it, nothing exercises or
  promises it" — and extend the matrix footer from "the matrix describes the target
  `apps/registry` behavior the tasks must meet" to name the three `Yes` rows
  (Linux/other-Unix-CI, macOS, Windows) as the rows the tasks owe behavior to. This
  replaces text; it adds no mechanism, and it leaves Test strategy #19 unchanged
  because that row is claimed as unsupported rather than supported.

- [minor][new] Catalog document and its spec + Test strategy #1 — "`schemas/catalog/v1/`
  ships `spec.md` + `fixtures/valid/*.json` + `fixtures/invalid/*.json`" — the shipped
  JSON fixtures end up counted and never parsed by anything. Every harness edit the
  proposal names *skips* them (`conformance.rs`: "guard `every_valid_fixture_…`,
  `every_invalid_fixture_…` and the single-line/truncation loops behind the
  core-TOML check"), the zero-fixture rule "counts files, not parses"
  (`common/mod.rs::zero_fixture_violation`, verified: it only tests emptiness), and
  the one suite that could consume them — Test strategy #1's catalog-document loader
  case — states its faults (wrong `version`, negative `revision`, grammar-invalid
  `game_id`, over-128 `name`, unspellable `source_key`) without naming the fixtures as
  its inputs. `schemas/README.md`'s discipline that "Loader refusals are the schema's,
  by name" (`026-C3`) is then unpinned for the new format, and third-party readers of
  the corpus — the audience `slug_bound_corpus.rs` cites for fixtures — get a contract
  tree that no suite proves. Fix: re-word Test strategy #1's inputs to the shipped
  corpus — "a valid corpus loads" becomes "every `fixtures/valid/*.json` loads, and
  each of … is a named refusal at `fixtures/invalid/*.json`" — and add one clause to
  the `common/mod.rs` bullet in §Catalog document and its spec recording that the
  JSON fixture directories stay under the existing fixture-name and case-uniqueness
  rule (`conformance.rs::fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique`,
  which walks every `format_dirs()` entry, verified) now that `is_fixture_name`
  accepts `.json` for external formats. No new mechanism; it points an existing suite
  at existing files.

## Checklist disposition

1. **Compatibility matrix** — finding (major, above). No empty cells.
2. **OS input APIs (X11/Wayland/Windows Raw Input/macOS HID)** — not applicable, and
   correctly so: the registry's surface is `std::net` HTTP/1.1 only and the CLI
   amendment touches no input path; the abstraction is justified in the text
   ("`std`-only for its serving face"). Verified against the shipped client, which is
   likewise `std::net`-only.
3. **Version floor** — no Node/JS code changes anywhere in scope, so the
   Node 22.23.4 floor is untouched (`tools/orchestration` is not edited by this
   design). Rust floor `1.85.0` (`rust-toolchain.toml`) is not exceeded by anything
   the design names (`std::net`, `std::fs`, `serde_json`), and the new crate needs no
   toolchain or workspace edit (`members = ["crates/*", "apps/*"]`, verified).
4. **Native modules** — pass: `serde_json` (already a shipped dependency of
   `apps/discovery-cli`, verified) is the only new-crate dependency named; no prebuilds,
   no `node-gyp`, no new CI leg.
5. **Keyboard reality (non-ASCII, IME, layout)** — pass: `game_id` is ASCII-only by
   `is_slug_name` (`[a-z][a-z0-9-]*`, verified in `crates/core/src/placement.rs`),
   `name` is unbounded-in-charset but octet-capped at 128 with the shipped
   char-boundary clamp precedent (`crates/discovery/src/game.rs::friendly_name`), and
   lookup keys carrying non-ASCII external ids are admitted by
   `Source::spells_lookup_key` (shape-only, verified) consistently with how the
   registry decodes-then-validates.
6. **File paths / config dirs** — pass: the reused precedent joins with `Path::join`
   (`matches.rs::store_path`), the XDG/`$HOME`/`%APPDATA%` rule and the
   relative-is-unset rule match the shipped `data_dir` exactly (verified), and the
   `<data>/bindforge/registry/{catalog-v1,cache-v1}.json` instances follow it.
7. **Platform test per supported platform** — pass: Test strategy #19 names Linux
   (cargo gates), Windows and macOS (the #13 `StoreLayout` env-closure seam plus
   case-unique fixture names) for the three rows the matrix claims as supported.

Revision log checked: all ten r0 entries are closures in this seat's neighbourhood
(format carve-out, budget closure, seed ids, upstream integrity, `026-C5` ambiguity,
two-writer refusal, request budgets, health posture, `T-008` divergence window). No
finding above is one the author rejected; the two findings are gaps the closures
introduced or left, not re-opened themes. Deferred section checked against the ADR:
the TLS/`026-R1`, `026-R2`/ADR-027, `026-R3`, real-manifest-id and `data_dir`-lift
deferrals are all consistent with the ADR's own Reservations and Rollout wording —
none re-opened here.

## FYI (no severity)

- Rollout ordering: `T-004`…`T-007` ship a service the shipped CLI cannot reach
  off-loopback until `T-008`, so integration cases #16/#17 (which drive
  `discovery list --catalog-url`) can only run against `127.0.0.1` in that window.
  Consistent with the ADR's own Rollout numbering, hence not a finding — but the
  tasks may want their loopback-vs-remote assertions sequenced explicitly.
- The byte-exactness obligation between the registry's serializer and the client's
  65,536-octet read cap is pinned by one seed-corpus case (#16). A name carrying a
  near-budget non-ASCII value is a legal `026-C3` record whose `serde_json` output
  expands under escaping; a correctness- or performance-seat case near the cap would
  close that.
- The A20/A21 standards claims are marked unverified and correctly subordinated to
  the ADR's binding texts; cross-checked against `026-C4` ("A `405` names any method
  the route does not carry") and `026-C5` (401/404 posture) — the proposal matches
  both, so the deviation notes stay non-blocking.
- A11's "three named shipped places" and both pinned suite names verified verbatim
  (`catalog.rs::LOOPBACK_HOSTS`, the `parse_endpoint` host check, `probe()`'s
  `is_loopback()` refusal, `catalog.rs`'s `the_policy_refuses_everything_beyond_the_loopback_spelling`,
  and the gate suite's `endpoint_policy_refuses_everything_beyond_the_loopback_spelling`),
  as was the refusal line naming "ADR-026 owns the real transport"; likewise A15's
  "no test performs a non-loopback request" header, the `2 s`/`5 s` timeouts, the
  `65_536`/`256` caps, and the `128`-octet slug and name bounds. A9/A10 hold as well:
  `fixtures()` returns files unfiltered so a JSON corpus would be TOML-parsed, and
  `version_gate.rs`'s majors-equality compares major-name sets (`v1` already
  supported), so "no change" is right.
