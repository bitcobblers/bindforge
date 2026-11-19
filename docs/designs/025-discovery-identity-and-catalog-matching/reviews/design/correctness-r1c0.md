# correctness review r1 (pass c0, full panel)

Scope: whole document. Inputs used: `02-proposal.md` (incl. its Revision log),
`docs/adrs/025-discovery-identity-and-catalog-matching.md`, and read-only
inspection of the code/CI/specs the assumption register cites. No review
report or transcript was opened.

verdict: request-changes

## Verification performed (basis for "no finding" on the register)

Checked and **supported**: A1 (`crates/discovery/src/game.rs:43-51` field set;
`Namespace` kebab-case serde enum, sole value `Unknown`, `Display` = `unknown`);
A2 (`steam_common::parse_manifest` returns only `display_name` + `install_dir`;
`steam_linux.rs:6` is the *only* "AppIDs are never read" statement in the
crate — grep for `appid|AppID` over `crates/discovery/src` returns that one
line); A3 (`keyvalues::Section::find` uses `eq_ignore_ascii_case`, `get_str`
exists; `appmanifest_900001.acf:3` carries a quoted `"appid"` inside
`AppState`); A4 (`discovered_game` derives + checks
`is_slug_name`/`MAX_SLUG_OCTETS`; `resolve_collisions` is in-module with
in-module tests; the `(slug, install_path)` ordering is at the call site, so
the design's "the sort stays" is right); A5 (`is_slug_name` is
`[a-z][a-z0-9-]*` over bytes — refuses `/`; `MAX_SLUG_OCTETS = 128`);
A6 (`MAX_NATIVE_NAME_OCTETS = 128` exported; core exports **exactly twelve**
constants, so "thirteenth" and "twelve→thirteen" are both correct);
A7 (`COLUMNS` is the five named columns; `--json` is one compact
`serde_json` array; T-003 §2 pins the key set); A8 (`DiscoveryError` has two
string variants; `DiscoveryEngine::discover_all` swallows plugin errors and
returns `Ok`); A9 (`crates/discovery/Cargo.toml` deps as stated);
A11 (`.github/workflows/check.yml`: one `ubuntu-latest` job,
`cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`,
`cargo test --workspace --locked`, no macOS leg);
A12 (`conformance.rs::every_invalid_fixture_is_rejected_with_a_structured_error`
walks every `fixtures/invalid/`, so `id-source-key.toml` needs no new test);
A13 (`ledger_annotations.rs` `ledger_unit(text, "013-L5")` +
`entry.contains("amended-by 024-B1")`; `013-game-manifests.md:117` carries the
literal `amended-by 024-B1 (ADR-024, per 021-L4).` shape);
A18 (RFC 3986 §3.4 allows `/` unencoded in a query — fetched);
A20 (grep for `steam/`, `epic/`, `local/` over `schemas/` and `data/`:
no matches, so the corpus sweep passes as designed).
Fetched A14's Steamworks *Applications* page: "Each App is represented by a
unique ID called an App ID … used with the Steamworks API and Steamworks Web
API" — supports the closed-token/`steam/<appid>` premise. `024-C1/C2/L1/L2/
B1/L3/R1` all exist as annotatable units. `apps/discovery-cli/Cargo.toml`
already carries `bindforge-core`, `serde` (derive), `serde_json`, `clap`, so
"No `Cargo.lock` change" holds. `steam_common::absolute_env` and
`steam_linux::host_candidates` exist as the env-lookup seam the store path
mirrors.

## Findings

- [major][new] Rollout item 3 + Risks (re-pin row) + Test strategy #4 —
  "`024`'s identity fixtures and tests re-pin a second time" / "the re-pinned
  CLI output contract (`Namespace` column out, `Game Id` in)" — the plan
  accounts for the new integration tests (#11–#13) and for named deletions,
  but not for the pre-existing `apps/discovery-cli` subprocess suites that the
  new step 1 breaks outright. Verified: `cli_tests.rs`,
  `cli_output_polish_tests.rs` and `cli_slug_refusal_tests.rs` all run the
  built binary (`CARGO_BIN_EXE_discovery-cli`) against fixture/temp Steam
  roots and assert a rendered table or JSON array; with the gate landing
  before any plugin is constructed, every one of those runs now exits
  non-zero with `error: catalog transport unavailable` and empty stdout. Each
  needs the loopback fake, and that harness work appears in no Option,
  Rollout item, or Risk. Separately, `cli_slug_refusal_tests.rs` pins the
  emission boundary end-to-end off a fixture whose *display name* is
  over-bound (`const SLUG_BOUND: usize = bindforge_core::MAX_SLUG_OCTETS;`),
  which is no longer producible: a `steam/<appid>` key is at most 26 octets
  under the design's own 1–20-digit appid rule, so no fixture can exceed
  `MAX_DISCOVERY_SLUG_OCTETS`. As written the gates cannot pass without
  unplanned re-tooling of suites the design never names.
  Fix: in Rollout item 3, name the migration — every existing
  `apps/discovery-cli` subprocess test runs behind a `TcpListener` fake (or an
  injected `CatalogClient`) — and state that the CLI emit-boundary bound
  refusal moves to the in-source `#[cfg(test)]` hand-built-row pin in
  `src/main.rs`, deleting the fixture route in `cli_slug_refusal_tests.rs`
  rather than leaving it to assert an unreachable condition.

- [minor][new] Test strategy #14 (retirement) — "`derive_slug`, `Namespace`,
  `namespace`, and `resolve_collisions` appear in no source under
  `crates/discovery` or `apps/discovery-cli`" — the textual-absence scope is
  far wider than the changes the Rollout schedules: grep finds 90 `namespace`
  occurrences in `crates/discovery` and 32 in `apps/discovery-cli`, spread
  across `plugin_tests.rs`, `game_tests.rs`, `steam_linux_tests.rs`,
  `steam_windows_tests.rs`, `emission_tests.rs`, `zz_api_probe.rs`,
  `cli_dual_plugin_tests.rs`, `cli_tests.rs`, `cli_output_polish_tests.rs`
  and `cli_slug_refusal_tests.rs`, none of which Rollout 1–5 names for change
  (only `slug.rs`, `slug_tests.rs`, `collision_tests.rs`, `resolve_collisions`
  and `Namespace` are named). It also catches prose: the doc comment above
  `COLUMNS` reads "(ADR-022, widened by ADR-024's namespace)".
  Fix: pin the compiled surface instead of the string — the `Namespace` type,
  the `namespace` field, `derive_slug` and `resolve_collisions` resolve
  nowhere in `src/` or in any assertion — and list the re-pinned suites in
  Rollout 1 and 3.

- [minor][new] Corpus boundary and ledger — "`!is_slug_name(slug)`,
  `!is_slug_name(friendly_name)` … — the structural proof that a source key
  cannot become a product id, because the grammar refuses it (A5)" — the claim
  is structural only for the slug (the `/` decides it). For `friendly_name`
  the normalization rule (trim, collapse whitespace, clamp at a `char`
  boundary) preserves an all-lowercase single-word name verbatim, and
  `display_name` falls back to `installdir`; every current fixture happens to
  carry a capital or a space ("Example Game", "Secondary Title",
  "BrokenMarker900202"), so the assertion passes by fixture accident and will
  fail confusingly for a future fixture named e.g. `doom`.
  Fix: restrict "the structural proof" to the slug, and either state the
  friendly-name assertion as a fixture-contingent pin or replace it with the
  property `025-L5` actually needs — a friendly name is never used as a
  manifest/profile `id` or a path component.

- [minor][new] Scan flow step 6 — "A chosen candidate writes the record
  `(slug, game_id, the row's friendly name)`" with "The answer applies to
  every row sharing the slug" — when several rows share one slug (which the
  Emission section relies on: "keeps the same game in two libraries as two
  rows"), "the row's friendly name" names no row, and the store — "keyed by
  `slug` alone" — holds one record where `025-C3` says "one record per
  confirmed install". The design acknowledges the sharing only for `local`
  ("Two install directories differing only in case collapse to one
  `local/<slug>`"), leaving the general case unstated.
  Fix: say the store holds one record per distinct slug, and name the row
  whose friendly name is stored (the first in the existing `(slug,
  install_path)` order).

- [minor][new] Assumption register, A10 — "The workspace's external deps are
  `serde`, `winreg`, `clap`, `serde_json` … verified | Cargo.toml:20-24" —
  the cited basis does not support the set: `[workspace.dependencies]` lists
  `bindforge-core`, `serde` and `winreg`, and comments that `clap` is among
  the deps that "enter when their consumer lands"; `clap` and `serde_json` are
  per-package deps of `apps/discovery-cli` (`clap = { version = "4", … }`,
  `serde_json = "1.0"`). The operative conclusion is safe — the CLI already
  has everything the new modules need, so no `Cargo.lock` change follows — but
  the row is marked `verified` against a basis that contradicts its wording.
  Fix: restate A10 as "the CLI's deps are `discovery`, `bindforge-core`,
  `clap`, `serde`, `serde_json`" citing `apps/discovery-cli/Cargo.toml:6-13`
  beside `Cargo.toml:20-24`.

- [minor][new] Bound — "`schemas/README.md` gains a sibling bullet in the
  exported budget family: ADR-025 adds `MAX_DISCOVERY_SLUG_OCTETS = 128`" —
  the new bullet covers only the new constant, while the design also gives the
  existing `MAX_NATIVE_NAME_OCTETS` a second, non-schema use as the
  `friendly_name` bound (`025-C2`'s borrowed sibling bound). Its stated homes
  keep describing it narrowly — the README bullet says "one game-native action
  name", `013-B4` says the same — so a reader of the budget family finds no
  statement of the discovery row's display bound.
  Fix: one clause in the new bullet — ADR-025 also borrows
  `MAX_NATIVE_NAME_OCTETS` as the `friendly_name` display bound, a non-schema
  use beside the lookup-key cap — leaving both existing bullets' text intact.

- [nit][new] Test strategy #4 — "The plugin-side skip case at the same bound
  is asserted separately" — through any plugin entry point that case is
  unreachable: the 1–20-digit appid rule caps a composed key at 26 octets, so
  only an in-module construction of `AppManifest` with a 130-digit `appid` can
  reach the `MAX_DISCOVERY_SLUG_OCTETS` skip in `discovered_game`.
  Fix: say so in the one line, and pin the case as an in-module
  `AppManifest` construction (defence in depth), or drop it and keep the
  `refuse_invalid_slugs` hand-built-row boundary pin.

## FYI (no severity)

- Compatibility seat: the loopback-authority-only client means no shipped
  invocation of `discovery list` can ever succeed outside a test process; the
  Revision log records the choice and its C-toolchain/`025-R1` reasoning, and
  the ADR's Consequences accept fail-closed discovery, so it is not re-opened
  here — but "Remote catalog service | No (named refusal)" is a shipped
  behavior change to the CLI's purpose, not just a deferral.
- Compatibility seat: `025-C2`'s "never end-user visible" sits beside the JSON
  row emitting `friendly_name`; the design flags it itself ("`friendly_name`
  is discovery output, which `025-L5` allows").
- Complexity seat: `refuse_invalid_slugs` keeps its name and its
  "invalid slug" line text while its body becomes bound-only and its ADR-024
  doc comment ("carries a slug in the shared grammar") goes false; the doc
  comment is not in the named edit list.
- Security seat: the client percent-encodes `/` in the query value, which
  RFC 3986 §2.2 makes *non*-equivalent to the literal form; the server-side
  reading is deferred to `025-R1` with an owner named, so the spelling is a
  provisional contract ADR-026 may have to honour.
