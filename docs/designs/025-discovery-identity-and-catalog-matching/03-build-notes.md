# Build notes — design 025 (discovery identity and catalog matching)

## Progress

- **Gate-fix round 6 (this pass) — the gate is green.** The runner's
  capture at `2026-11-19T03:25:31.784Z` was the eighth red on
  `cargo test --workspace --locked`, the same two ledger pins; between that
  capture and this round the operator landed Rollout 5's five ledger
  annotations in `docs/adrs/024-game-slug-identity.md` — that file now
  stands modified-uncommitted in the working tree (ADRs are runner/
  operator-owned; this stage never read or wrote it, Deviation 2). All
  four legs were re-run in this shell against the tree as it stands, each
  exit 0: `cargo test --workspace --locked` (every target
  `test result: ok`; `ledger_annotations` `7 passed; 0 failed`),
  `cargo fmt --all -- --check` (silent), `cargo clippy --workspace
  --all-targets -- -D warnings` (`Finished dev` profile), and `npm run
  check` in `tools/orchestration` (Prettier clean, 61 files,
  `692 passed | 2 skipped`). No leg failed on a formatting or lint rule,
  so `npm run format` has nothing to land. Quoted under Gate posture.
- **Completed behaviors** (unchanged since the implementation pass;
  re-verified green in this shell): identity (`Source` +
  `lookup_key`/`spells_lookup_key`, `DiscoveredGame { slug, friendly_name,
  display_name, install_path, platform, codec_id }`, `Namespace` deleted,
  `friendly_name()` normalization, appid rule, bound-only plugin gate); the
  `MAX_DISCOVERY_SLUG_OCTETS = 128` bound with its `schemas/README.md`
  sibling bullet and refused invalid fixture; and the CLI's catalog client,
  match store, and rewritten output contract (`crates/discovery/src/game.rs`,
  `crates/discovery/src/steam_common.rs`, `crates/core/src/placement.rs`,
  `apps/discovery-cli/src/{catalog.rs,matches.rs,main.rs}`).
- **Remaining work inside stage ownership: none.** **Next concrete
  action**: the runner commits the operator's
  `docs/adrs/024-game-slug-identity.md` change — as its own commit, per
  Rollout 5's attributability clause — and `pipeline resume`s; every gate
  leg is green at the tree as it stands, so no further stage edit can
  improve the posture.
- **Observed tree state this stage did not author and left untouched**:
  `docs/designs/025-.../ESCALATION.md` is an unstaged deletion (`D` in
  `git status`) — the runner-authored budget-exhaustion marker whose state
  the operator's landing resolves; restoring it would reinstate a stale
  record, and no gate leg reads it. The only other dirty paths are the
  operator's `M docs/adrs/024-game-slug-identity.md` and this stage's own
  build-notes edit.

## Gate posture

- Failing leg — the runner's gate capture at `2026-11-19T03:25:31.784Z`:
  `cargo test --workspace --locked` → **exit 101**, one target
  (`-p bindforge-core --test ledger_annotations`, `5 passed; 2 failed`);
  the failure-carrying tail lines, quoted line-selectively:

  ```text
  ---- the_024_r1_reservation_records_adr_025_as_filling_its_slot stdout ----
  thread 'the_024_r1_reservation_records_adr_025_as_filling_its_slot' (1452632) panicked at crates/core/tests/ledger_annotations.rs:306:5:
  `024-R1` must record that the publisher-namespace half is superseded while ADR-025 fills the resolver slot (design 025 Rollout 5):
  ---- the_024_amended_units_carry_the_adr_025_annotations stdout ----
  thread 'the_024_amended_units_carry_the_adr_025_annotations' (1452631) panicked at crates/core/tests/ledger_annotations.rs:281:9:
  `024-C1` must carry `amended-by 025-C1` (design 025 Rollout 5, per `021-L4`): ADR-025 replaces the identity half of ADR-024, and the amended units' ledger rows must say so. Unit entry as shipped:
  failures:
      the_024_amended_units_carry_the_adr_025_annotations
      the_024_r1_reservation_records_adr_025_as_filling_its_slot
  test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  error: test failed, to rerun pass `-p bindforge-core --test ledger_annotations`
  ```

  (No quoted line carries a credential or an absolute path — the only path
  in them is the repo-relative `crates/core/tests/ledger_annotations.rs`.
  The capture's elided passing lines and its trailing "note: run with
  `RUST_BACKTRACE=1`" line are dropped as non-carrying.)
- Reading of the two panics (now closed): both read only
  `docs/adrs/024-game-slug-identity.md` via `fixture_text` at a
  compile-time `repo_root()`, with no stage-reachable lever; the
  `024-R1` entry already carried the `ADR-025` name, so only the
  `superseded` clause and the four `amended-by` rows were missing. Both
  halves are corpus lines in the runner-owned ADR — no code, formatter,
  or suite change was involved in closing them.
- Green posture at the tree as it stands (this shell, exit 0 each):
  `cargo test -p bindforge-core --test ledger_annotations --locked` →
  `test result: ok. 7 passed; 0 failed` (both formerly-failing tests
  `ok`); `cargo test --workspace --locked` → every target
  `test result: ok`, zero `FAILED` results workspace-wide;
  `cargo fmt --all -- --check` → no output; `cargo clippy --workspace
  --all-targets -- -D warnings` → `Finished dev profile` with no warnings;
  `npm run check` in `tools/orchestration` → `eslint .` clean, `All
  matched files use Prettier code style!`, `Test Files 61 passed (61)`,
  `Tests 692 passed | 2 skipped (694)`. `Cargo.lock` unchanged.

## Deviations from the proposal

1. **Retired files not deleted** (Rollout 1): stage denies `rm`/`mv` and the
   `moves` contract rejects `tests/**` endpoints. `crates/discovery/src/slug.rs`
   is emptied to a retirement comment (contains no retired token, so the
   retirement name-sweep passes it); `tests/slug_tests.rs` and
   `tests/collision_tests.rs` still ship but are excluded from every build via
   a never-enabled `required-features = ["retired-adr-024-suites"]` gate in
   `crates/discovery/Cargo.toml`. **Operator**: delete the three files and the
   gate; the retirement suite is re-pinned (`0f25a167`) and passes with the
   gate in place, so the deletion is hygiene, not gate posture.
2. **Rollout 5 — closed by the operator between the `03:25:31` capture and
   this round (was open in fix-rounds 2–5)**: the ledger annotations in
   `docs/adrs/024-game-slug-identity.md` are read- and write-denied to every
   stage of this run (ADRs are runner-owned), so they could not land in a
   stage commit, and no stage-side lever existed (`fixture_text` is a plain
   read at a compile-time `repo_root()`; the assertions match design 025's
   Test strategy #16 word for word, so they were never `test-update`
   candidates). The operator landed the round-5 hand-off payload: this round
   `git status` shows that file modified-uncommitted, and the 7/7 green
   `ledger_annotations` run in this shell (Gate posture) proves the landed
   lines satisfy both pins — without this stage reading or writing the ADR.
   Rollout 5's "its own commit" clause now belongs to the runner's commit of
   the operator's change; post-commit re-verification is
   `cargo test -p bindforge-core --test ledger_annotations --locked`.
3. **Escaped echo keeps the file name**: `match store unreadable` echoes
   `escape_and_truncate(parent) + escape_and_truncate(file_name)` so the
   operator always sees `matches.json` (a whole-path `escape_and_truncate`
   would cut long XDG paths at 80 chars). Store-refusal and prompt lines match
   the suites' pins.
4. **Table cells escape only when hostile**: ACF-sourced cells run through
   `escape_and_truncate` only when they carry control characters; clean values
   (long temp install paths) render untruncated, per the install-path cell pins
   in the output suites.
5. **CLI dev-dependencies added** (`bindforge-core`, `discovery`, `serde_json`)
   because the committed suites import them directly; `Cargo.lock` needed no
   change (edges already existed).
6. **025-R1 deferral honored**: no ledger work beyond item 2, per Rollout 6.

## As-built addendum (2026-11-18 as-built pass — verified against
`git diff 7b03661…HEAD`; drift note appended to `02-proposal.md` §As-built)

- **Progress items closed by the runner**: Deviation 2's operator change is
  committed — `c9983a81` carries the five `docs/adrs/024-game-slug-identity.md`
  ledger lines (inside the run's diff range, `ledger_annotations` 7/7 per Gate
  posture) and the `ESCALATION.md` deletion, so the Progress section's
  "modified-uncommitted" state and "next concrete action" are both superseded;
  the working tree now shows only the runner's `.pipeline.json`.
- **Additional drift verified this pass, recorded as proposal As-built items
  6–9** (each additive; no design decision altered):
  7. Catalog seam shapes: the `CatalogClient` trait carries only `candidates()`
     (`probe()` is an inherent `HttpCatalog` method); `parse_endpoint` returns
     a normalized `String` base, not an `Endpoint` struct; `Candidate` is
     parsed manually from `serde_json::Value` (no `Deserialize` derive; unknown
     fields still ignored, response order kept).
  8. No-data-directory degradation: `store_path` → `None` when no absolute env
     value resolves; the scan then runs on an empty, non-writable store
     (`MatchStore::writable()` — a non-writable store earns no prompt;
     `save()` no-ops). The design never named this state.
  9. Additive surface: `Source::ALL`/`from_token`/`spells_lookup_key` (the
     store's closed-set check) and `Display` + `std::error::Error` for
     `DiscoveryError` in `crates/discovery/src/registry.rs` — a file the design
     does not name; the CLI's plugin-failure line still quotes `Debug`.
  10. `crates/discovery/tests/.rm-probe` (0 bytes) is a stage toolchain probe
      that shipped by accident; **operator**: delete it in the same pass that
      deletes the three files of Deviation 1 and the `retired-adr-024-suites`
      gate.
  11. Doc-comment citations only: `main.rs`, `catalog.rs`,
      `cli_json_mode_tests.rs` and `cli_confirmation_flow_tests.rs` name
      `025-C5`/`025-C6`, handles no ledger defines (the intended handles are
      `025-C4`/`025-L2`); and Rollout 6's deferral record also entered the
      source ADR as Rollout item 6 via the run's research commit `8e005c31`.
- **Verified as designed**: the five-column table and eight-key JSON row; the
  bound-only emit boundary and its unchanged refusal text; appid 1–20 digits
  with skip-and-count and no fallbacks; `(slug, install_path)` sort with the
  collision pass gone; `MAX_DISCOVERY_SLUG_OCTETS` in `placement.rs`/`lib.rs`
  and the `schemas/README.md` sibling bullet with "thirteen constants"; the
  loopback endpoint policy, percent-encoded `?slug=`, header/body/count budgets,
  and post-resolution loopback check; the store's budget/shape refusals,
  sorted byte-stable save, exclusive tmp, 0700/0600 modes; the
  `id-source-key.toml` invalid fixture; the `data/` + valid-fixture `steam/`/
  `epic/`/`local/` sweep; Rollout 3's suite migration with the bound refusal
  pinned in-module in `main.rs`. No `Cargo.lock` change anywhere in the range
  (Deviation 5's dev-deps reuse existing edges).
