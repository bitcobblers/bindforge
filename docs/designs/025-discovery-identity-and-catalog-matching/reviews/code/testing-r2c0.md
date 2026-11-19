# Code review — testing seat, round 2 full panel (c0)

- Scope: `git diff 7b036617dfa2d7895888b73c9b9412387b1d7e08...HEAD -- ':(exclude)docs/designs'`
- Inputs used: the diff, `02-proposal.md`, `docs/adrs/025-discovery-identity-and-catalog-matching.md`, the runner's gate record, my own reading of the source and test trees. No other seat's report or transcript was used as review input.
- Checklist: `references/testing.md` (code testing), testing specialty only.

## Gate evidence

Supplied (runner, 2026-11-19T03:30:34Z, green): `npm run check` 0, `cargo fmt --all -- --check` 0,
`cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo test --workspace --locked` 0
(tail `test result: ok. 0 passed; 0 failed; ...`).

Executed by me (specific question: does `cargo test --workspace --locked` really build and run
everything it should — including the new `retirement_tests`, `boundary_tests`, ledger and API-probe
gates — while the two `required-features`-gated retired suites, which still import the deleted
`discovery::slug::derive_slug`, stay excluded; and is the runner's `0 passed` tail legitimate?):
`cargo test --workspace --locked` — exit green, ~45 test binaries, hundreds of assertions, all
`test result: ok.`; the trailing lines are `0 passed` binaries (doc-test legs), consistent with the
runner's captured tail. Had slug_tests/collision_tests been selected, the build would have failed
to compile; it did not. The gate record is corroborated, fresh, and green. No failing leg; nothing
blocked on evidence grounds.

## Checklist walk

1. **Every numbered strategy item cited.** Design-025 citations
   (`proves: design 025's Test strategy #N`) map cleanly: #1/#2 (`game_tests.rs`,
   `zz_api_probe.rs`, `plugin_tests.rs`), #3/#4 (`emission_tests.rs`, `steam_linux_tests.rs`,
   in-module `steam_common.rs` bound pins, in-module `main.rs` emit-boundary pins,
   `cli_slug_refusal_tests.rs`), #5–#7 (`cli_match_store_tests.rs` + in-module `matches.rs`),
   #8 (`cli_confirmation_flow_tests.rs` + in-module `selection` pin), #9/#10/#11
   (`cli_catalog_gate_tests.rs` + in-module `catalog.rs`), #12 (`cli_confirmation_flow_tests.rs`,
   `cli_tests.rs`), #13 (`cli_json_mode_tests.rs`, `cli_tests.rs`), #14 (`retirement_tests.rs`,
   both `zz_api_probe.rs`), #15 (harness discipline stated and observed: every listener binds
   `127.0.0.1:0`, no fixed ports, no non-loopback host is ever dialed — the named-case refusals
   for foreign endpoints are pure-parse), #16 (`boundary_tests.rs`, `ledger_annotations.rs`,
   `id-source-key.toml` + the existing conformance walk). One named sub-case is uncovered —
   finding 3.
2. **Unit and integration layers both present** where promised: in-module units (`game.rs` seam
   via `game_tests`, `steam_common.rs`, `matches.rs`, `catalog.rs`, `main.rs`) plus subprocess
   integration for the gate, store, confirmation flow, JSON contract, and retirement sweeps. The
   real `HttpCatalog` is driven over a socket by subprocess suites; pure halves are pinned
   in-module. Pass.
3. **Revision rounds.** No prior findings from this seat; the r1 additions (harness migration of
   `cli_tests`/`cli_output_polish_tests`/`cli_dual_plugin_tests`/`cli_slug_refusal_tests` behind
   the loopback fake, bound refusal moved in-module) are present and consistent. Pass.
4. **Determinism.** No wall-clock-dependent assertions (`SystemTime` appears only as
   uniqueness entropy in temp-dir names alongside pid + atomic counter); no real network (all
   listeners are in-process `TcpListener`s on ephemeral loopback ports; the only closed-port seam
   is bind-and-drop on `127.0.0.1:0`); `run_cli` strips `BINDFORGE_CATALOG_URL`, `XDG_DATA_HOME`,
   `HOME`, `APPDATA` from every child, and the host-probe suite uses `env_clear` — no test can
   touch a developer's store or dial an unconfigured endpoint. Pass.
5. **Names describe behavior.** Pass (e.g. `the_gate_refuses_before_enumerating`,
   `a_declined_confirmation_writes_no_store`, `the_rescan_prints_the_stored_id_and_asks_the_catalog_nothing`).
6. **Gate evidence** — assessed above; corroborated by an executed run.
7. **Untestable/unconsumed additions.** Two: the `retired-adr-024-suites` gate feature (finding 2)
   and the `Display`/`Error` impls on `DiscoveryError` (finding 4). Otherwise the new surface
   (`Source`, `lookup_key`, `spells_lookup_key`, `friendly_name`, `usable_appid`, `parse_endpoint`,
   `candidates_url`, `partition_candidates`, store path/load/save) is all consumed and pinned.

Not-in-specialty FYI (no severity): `HttpCatalog::probe` is an inherent method rather than the
trait method the Chosen design's `CatalogClient` sketch shows; behavior is pinned (subprocess gate
tests drive it), so this is a shape note for the correctness seat, nothing more.

## Findings

1. **minor — stray artifact committed: `crates/discovery/tests/.rm-probe`.**
   Location: `crates/discovery/tests/.rm-probe:1` (added in the diff, zero bytes).
   Problem: a delete-probe residue is committed inside the tests tree; it is referenced nowhere,
   is part of no build target, and is asked for by nothing in the proposal. Test-tree directories
   should not accumulate unreferenced binaries-of-any-kind; the committed tree is the run's record
   of what the change ships.
   Fix: runner/operator deletes the file at final integration (stage `rm` denial explains its
   presence but not its inclusion in the commit set).

2. **minor — the gated-retirement shape leaves a non-enabled knob and a retirement record that
   misdescribes the tree.**
   Location: `crates/discovery/Cargo.toml:24-33` and `crates/discovery/tests/retired/README.md:10-13`.
   Problem: (a) the `retired-adr-024-suites` feature is a knob with no possible consumer — enabling
   it cannot build, because the gated suites import the deleted `discovery::slug::derive_slug`; its
   only function is exclusion, which deletion would achieve permanently (checklist item 7). (b) The
   README states the two suites "are kept here" under `tests/retired/`, but that directory holds
   only the README; the files actually stand at `tests/slug_tests.rs` and `tests/collision_tests.rs`
   behind the feature gate. The retirement's own record is wrong about where the record-keepers are.
   The functional property — the retired machinery compiles, builds and asserts nowhere the gates
   run, and `retirement_tests.rs` pins both the gated shape and the deleted shape — does hold, and
   the operator follow-up is named in the manifest, so this is not a functional break.
   Fix: at final integration the runner deletes `slug.rs`, `tests/slug_tests.rs`,
   `tests/collision_tests.rs`, and the `[features]`/`[[test]]` block together; until then, correct
   the README to say the suites stand at their original paths, feature-gated.

3. **minor — Test strategy #9's "% / space encoded once, never double-encoded" case is not
   pinned.**
   Location: `apps/discovery-cli/src/catalog.rs:527-538` (the only pure `candidates_url` unit pin),
   with the subprocess pin at `apps/discovery-cli/tests/cli_catalog_gate_tests.rs:218-256`.
   Problem: strategy #9 names three encoding cases: `steam/900001` → `steam%2F900001` (pinned), a
   slug with a space, and a slug with a `%` (each "encoded once (never double-encoded)") — neither
   the space nor the `%` appears in any assertion. The implementation is correct
   (`percent_encode` maps `%`→`%25` before any other escape), but a future edit that pre-encoded
   the query or double-escaped would pass the suite.
   Fix: one more assertion in `the_request_url_encodes_the_separator_of_the_lookup_key`:
   `candidates_url(base, "local/a%b c")` → `?slug=local%2Fa%25b%20c`.

4. **minor — new public `Display`/`Error` impls on `DiscoveryError` have no consumer and no pin,
   and their doc comment misstates how the suites format the error.**
   Location: `crates/discovery/src/registry.rs:10-19`.
   Problem: the diff adds `impl Display` and `impl Error` for `DiscoveryError`; every formatter in
   the workspace uses the `Debug` shape (`engine.rs:28` `{e:?}`, `main.rs:148` `{error:?}`,
   `emission_tests.rs` asserts on `{error:?}`), and `registry_tests.rs` pins equality only. The
   added doc comment claims "design 025's suites format the failure with `{}`" — they do not. This
   is a checklist item 7 unconsumed addition whose only behavioral claim is unpinned (the exact
   `PluginError: <msg>` line shape is asserted nowhere).
   Fix: pin the line in `registry_tests.rs` (`assert_eq!(err.to_string(), "PluginError: error")`
   for both variants) and correct the comment to say the suites quote the `Debug` shape and the
   `Display` line is the error's voice for `dyn Error` consumers — or delete the impls until a
   consumer exists.

5. **nit — phantom unit handles cited in suite headers.**
   Location: `apps/discovery-cli/tests/cli_confirmation_flow_tests.rs:1` (`025-C5`),
   `apps/discovery-cli/tests/cli_json_mode_tests.rs:1` (`025-C6`),
   `apps/discovery-cli/src/catalog.rs:37` (`025-C6`).
   Problem: ADR-025's contracts end at `025-C4`; `025-C5`/`025-C6` name no unit. Per the ADR's
   own Unit-handles discipline, a handle that names nothing governs nothing — the suite-to-contract
   trace these headers exist to provide is broken for those lines (the confirmation flow and store
   are `025-C3`/`025-L2`/`025-L4`; candidate passthrough is `025-C4`).
   Fix: cite the real handles.

## Verdict

No blocker or major: the strategy's sixteen items are covered and cited, both layers exist, the
harness is deterministic and loopback-only, and the gates are green as supplied and as re-run.
All findings are cleanup, one missing named sub-case, and two unpinned minor additions — none
violates a required property in a way that blocks.
