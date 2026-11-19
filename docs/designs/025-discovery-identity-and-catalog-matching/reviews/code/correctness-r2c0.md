# Code review — correctness seat, round 2 pass c0

- Scope: `git diff 7b036617dfa2d7895888b73c9b9412387b1d7e08...HEAD -- ':(exclude)docs/designs'`
- Inputs used: the diff, `02-proposal.md`, `docs/adrs/025-discovery-identity-and-catalog-matching.md`, the runner's gate record (green at 2026-11-19T03:30:34Z), the code at branch HEAD for context around diff hunks. No review reports or transcripts opened.
- Previous findings for this seat: none.
- Verdict: **approve** (0 blocker, 0 major, 5 minor, 2 nit).

## Checklist walk

1. **Components and interfaces exist, wired, names/signatures match.** Nearly all:
   `Source`/`as_str`/`lookup_key` (`game.rs`), `friendly_name`, the six-field
   `DiscoveredGame` (`slug, friendly_name, display_name, install_path, platform,
   codec_id`), `AppManifest.appid` + `parse_manifest`'s 1–20-digit appid rule with
   case-insensitive read, the bound-only gates in `steam_common::discovered_game`
   and `main.rs::refuse_invalid_slugs` (refusal line text unchanged), the `(slug,
   install_path)` sort kept (`steam_common.rs:128`), `MAX_DISCOVERY_SLUG_OCTETS`
   beside `MAX_SLUG_OCTETS` (`placement.rs:85-96`) re-exported in `lib.rs` with the
   thirteenth-constant paragraph, the README sibling bullet with the not-a-grammar
   clause and twelve→thirteen, `MatchStore`/store path rule/load/save,
   `HttpCatalog`/`parse_endpoint`/`candidates_url`/`probe`/`read_response`/
   `parse_candidates`/`partition_candidates`, `selection`, the five-column table,
   the eight-key JSON row, both API probes re-pinned, the ledger gates and
   `id-source-key.toml` fixture. Deviations in the named *surface* of
   `CatalogClient`, `parse_endpoint`, and `Candidate`: findings 4.
2. **Assumption register honored.** A3 (case-insensitive appid read, no
   `keyvalues.rs` change), A5 (grammar exemption lands: `steam/1` survives the
   boundary, `!is_slug_name` pinned), A7 (five columns, one JSON document, fixed
   key set — `cli_json_mode_tests`), A10 (no `Cargo.lock` change — verified in the
   diff), A13 (amended-by shape on the four 024 units + the 024-R1 filled/superseded
   line, gated in `ledger_annotations.rs`), A16 (XDG/HOME/`%APPDATA%` env-seam rule),
   A20 (corpus sweep over `data/` + valid fixtures). No silently changed assumption.
3. **Declarations of deviations.** `03-build-notes.md` is outside this seat's
   permitted inputs, so the declaration of the retirement-shape deviation could not
   be read directly; the in-tree texts (`slug.rs:1-5`, `crates/discovery/Cargo.toml`
   comment, `retirement_tests.rs` docs) all assert it is recorded there. See findings
   1 and the evidence limit below.
4. **Scope creep.** No unasked-for behavior shipped. Additive enablers —
   `DiscoveryError: Display + Error` (`registry.rs`), `Source::from_token`/
   `spells_lookup_key`/`ALL` (needed by the store's `<token>` closed-set load check)
   — serve named design behavior. One stray file: finding 2. One source-document
   edit beyond the runner's sanctioned flip: finding 5.
5. **Build-notes claims.** Not readable under the input restriction (see Evidence
   limits); nothing in the code contradicts the claims the diff itself references.
6. **Error paths exist and fail closed.** Gate before any plugin construction, one
   named line, empty stdout (`cli_catalog_gate_tests`); store refusal names the
   escaped path and rewrites nothing before enumerating; mid-scan query failure
   aborts with no partial table; non-200 / Transfer-Encoding (either casing) /
   missing-duplicate-non-numeric Content-Length / over-budget header block and body
   / 257-candidate / non-JSON all refuse (`catalog.rs` + hostile-response leg);
   hostile `game_id` dropped at parse, write, and load (`025-L5`); tmp opened with
   `create_new` and never reused; save happens only after a confirmation.
7. **Revision rounds.** My previous verdict held no findings. The proposal's r1
   revision-log items are present: the four pre-existing CLI subprocess suites run
   behind the loopback `TcpListener` fake, `cli_slug_refusal_tests`'s unproducible
   fixture route is deleted with the bound refusal pinned in-module in `main.rs`,
   and the ACF-sourced table cells pass through the escape path (`cell()`, pinned
   by the ESC-byte test).

## Findings

1. **minor — retirement landed as tombstones + feature-gate, not the deletion the proposal names.**
   `crates/discovery/src/slug.rs:1-5` (file emptied, not deleted),
   `crates/discovery/Cargo.toml:17-33` (`slug_tests`/`collision_tests` kept in-tree
   behind `required-features = ["retired-adr-024-suites"]`; `tests/slug_tests.rs:24`
   still spells `use discovery::slug::derive_slug;`). The proposal states
   "`crates/discovery/src/slug.rs` is deleted", Rollout 1 "the deletion of `slug.rs`
   … and their suites", and Test strategy #14 "tests/slug_tests.rs and
   tests/collision_tests.rs are gone." The shape is forced by the stage toolchain
   (stages cannot `rm`; the runner owns moves) and the functional property — the
   machinery compiles, builds, and asserts nowhere the gates run — is pinned by
   `retirement_tests.rs`, with the file removals left as a named operator follow-up.
   The deviation is claimed to be recorded in `03-build-notes.md` (unreadable to this
   seat, so I cannot establish it is *undeclared*; if it is absent there, checklist
   item 3 escalates this). Fix: have the runner delete `slug.rs`, `slug_tests.rs`,
   `collision_tests.rs` and the gate block at final integration, or confirm the
   build notes record the recorded-retirement shape.
2. **minor — stray artifact committed: `crates/discovery/tests/.rm-probe`.**
   A zero-byte new file, referenced nowhere in the tree, with no role in any target
   or test — it appears to be residue of a delete probe. The proposal never asked
   for it and it is not part of the retirement shape. Fix: delete it (runner/operator).
3. **minor — `crates/discovery/tests/retired/README.md:10-13` misdescribes the tree.**
   It states the two retired suites "are kept here" (under `tests/retired/`), but the
   README is the only file there; the suites stand at `tests/slug_tests.rs` and
   `tests/collision_tests.rs` gated off the build via the manifest, which is the
   shape `retirement_tests.rs` itself pins. Fix: correct the README to the
   gated-in-place shape (or reconcile it when finding 1's deletions land).
4. **minor — named surfaces diverge from the proposal's signatures in `catalog.rs`.**
   `catalog.rs:200-203`: `trait CatalogClient` carries only `candidates`; the
   proposal's `trait CatalogClient { fn probe(...); fn candidates(...); }` puts
   `probe` in the trait, and "the seam every behavior below is written against" is
   not the seam the flow uses — `main.rs` drives the concrete `HttpCatalog`.
   Related: `parse_endpoint` returns a normalized `String`, not the sketched
   `Endpoint`; `Candidate` is a plain struct with a hand-rolled parse rather than a
   `Deserialize` derive (behavioral equivalence — unknown fields ignored — holds).
   No behavior or contract is missing; the abstraction the proposal named was
   simplified. Fix: add `probe` to the trait (and have `main.rs` hold
   `&dyn CatalogClient`), or record the surface change as a declared deviation.
5. **minor — the branch edits its own source ADR beyond the sanctioned flip.**
   `docs/adrs/025-discovery-identity-and-catalog-matching.md:182-186`: Rollout item
   6 (the deferral record, `025-R1`) is added to the run's source document by this
   diff. AGENTS.md sanctions only the runner flipping the source `status` on
   completion; ADR-024's edits are sanctioned by Rollout 5, nothing sanctions an
   in-flight edit to ADR-025's own Rollout list. The added text only records the
   deferral the proposal already states (its Rollout 6 places the record in
   `03-build-notes.md`) and relaxes nothing. Fix: confirm the runner authored this
   edit (attribution), or move the record to `03-build-notes.md` alone as the
   proposal's Rollout 6 spells it.
6. **nit — phantom unit handles `025-C5` / `025-C6`.**
   `apps/discovery-cli/src/main.rs:334`, `apps/discovery-cli/src/catalog.rs:37`,
   `apps/discovery-cli/tests/cli_confirmation_flow_tests.rs:1`,
   `apps/discovery-cli/tests/cli_json_mode_tests.rs:1` cite `025-C5`/`025-C6`,
   which ADR-025 does not define (contracts stop at `025-C4`); the ADR's own
   unit-handle discipline says ids outside the ledger "govern nothing", so these
   provenance citations point at nothing. Fix: re-cite as `025-C4`/`025-L2` or drop.
7. **nit — `apps/discovery-cli/Cargo.toml:14-22` rationale is false about cargo.**
   The comment claims "integration tests only see dev-dependencies"; integration
   targets see the package's normal dependencies too, and all three crates listed
   (`bindforge-core`, `discovery`, `serde_json`) are already normal deps, so the
   block is redundant (harmless — no `Cargo.lock` change). Fix: drop the block or
   fix the comment.

## Evidence limits

- `03-build-notes.md` is not among this pass's permitted inputs, so checklist
  items 3 and 5 were checked only against the code and the diff's own in-tree
  claims; nothing observed contradicts those claims, and the one material deviation
  (finding 1) has its declaration asserted at three points in the diff itself.
- The gate record's single captured line (`test result: ok. 0 passed …`) is the
  renderer's tail of a green `cargo test --workspace --locked` leg (exit 0); the
  suite bodies at HEAD match the Test strategy #1–#16 claims, so I read the leg as
  green with a truncated capture, not as zero tests run.

## FYI (out of specialty, no severity)

- `HttpCatalog::candidates()` re-resolves the host per query and connects without
  re-applying probe's post-resolution loopback check; only the name `localhost`
  can drift between probe and query, so the loopback-only property assumes
  resolution stability mid-run. Noted for the security seat.
- The `Host:` header is composed bracket-stripped (`::1:8080`) for the `[::1]`
  spelling; no shipped peer checks it.
