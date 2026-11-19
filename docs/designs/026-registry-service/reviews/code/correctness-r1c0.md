# Code review — correctness seat — round 1, pass c0

- Change under review: `git diff 217c46fda9eb95230c8df62233eca8ef7b6ffb71...HEAD -- ':(exclude)docs/designs'`
  (63 files, +12,163 / −92: `apps/registry` new, `apps/discovery-cli` endpoint amendment, `schemas/catalog/v1/`,
  `data/catalog/`, core + discovery harness carve-outs).
- Design: `docs/designs/026-registry-service/02-proposal.md`; contract: `docs/adrs/026-registry-service.md`.
- Inputs used: that diff, the proposal, ADR-026, the runner's gate record, my own prior findings (none — first pass).
  `03-build-notes.md`, other seats' reports and transcripts were not opened, per the caller's input rule; the two
  checklist rows that read build notes are recorded below as **not evaluable within this pass's inputs**.
- Gate record (runner, 2026-11-19T21:53:05.514Z, green): `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` all exit 0.
  The only `test result:` tail quoted (`ok. 0 passed; 0 failed`) is a zero-test leg, so the quoted tail evidences
  "nothing failed", not "the registry suites ran"; nothing in the code contradicts the green exit.

## Checklist walk

1. **Components and interfaces named by the proposal exist and are wired.** Pass, with file-plan deviations
   (finding 7). All nine modules exist (`main.rs`, `serve.rs`, `http.rs`, `corpus.rs`, `admin.rs`, `cache.rs`,
   `upstream.rs`, `endpoint.rs`, `constants.rs`); four more were added (`keys.rs`, `persist.rs`, `state.rs`,
   test-only `wire.rs`) and none are missing. The CLI surface is complete and matches `026-C1`:
   `serve --root --corpus`, `serve --upstream`, `--host` (default `127.0.0.1`), `--port` (default `0`, printed
   value is the contract, `main.rs:235-248`), `--admin-token`/`BINDFORGE_REGISTRY_ADMIN_TOKEN`,
   `--refresh-interval` (bare unsigned seconds, `main.rs:155-164`), `query <slug>` (writes the cache in caching
   mode, `caching_policy.rs:371-419`; exits non-zero on the `026-L4` cold case, `main.rs:293-298`). Health carries
   exactly the six `026-C2` fields (`state.rs:394-423`); `stats` exactly the three `026-C4` counts
   (`state.rs:428-445`); no route answers a whole document (`admin.rs:59-94`, `response_shape.rs:139-159`).
2. **Assumption register.** Honored: A3/A4/A5 (`keys.rs:72-94`, `corpus.rs:286-288`, core's 128-octet bounds);
   A6/A8 (`persist.rs:63-71`, `cfg(unix)` modes); A7 (`persist.rs:183-218`: fixed `<name>.tmp`, `create_new(true)`,
   rename, re-`chmod`, stale tmp a hard failure — and `persist.rs:366-380` pins it); A9/A10 (harness:
   `common/mod.rs:419-432`, `conformance.rs:81-84/212-215/300-303` guard only the parse/rejection loops, the
   fixture-name test stays ungoverned through the directory-aware `is_fixture_name`, `version_gate.rs` untouched);
   A11/A12/A15 (`LOOPBACK_HOSTS` and the constant are gone, `probe()` keeps resolve→refuse-empty→connect before
   any plugin, the refusal line drops "loopback", and the subprocess suite keeps only refused spellings — no test
   dials a remote host); A13 (`Cargo.toml` untouched, `apps/*` picks the crate up); A16 (the closed token set is
   reached through `Source::from_token`, which is exactly `spells_lookup_key`'s body, `crates/discovery/src/game.rs:84-89`);
   A19 (`keys.rs:52-63` is the shipped escape-everything rule, used to build the upstream target, `upstream.rs:53`);
   A22 (`persist.rs:236-252` metadata-gated before read, for both documents); A23 (one answer per connection,
   `http.rs:310-329`, `serve.rs:114-125`). One gap: A1's cap is enforced on the **inbound** hop only — finding 1.
3. **Deviations declared in `03-build-notes.md`.** Not evaluable in this pass (file not in the allowed inputs).
   The deviations I found are findings 2 and 7; if the build notes declare them they are closed there.
4. **Scope creep.** No behavior in the diff is outside the proposal's specification; the *vehicle* is — finding 7.
5. **`03-build-notes.md` claims.** Not evaluable in this pass. One claim in a **shipped** file is false:
   `data/catalog/README.md:12-14` says "`026-S1` carries the partially-satisfied line for it" —
   `docs/adrs/026-registry-service.md:207-211` carries no such annotation (finding 2).
6. **Error paths exist and fail closed.** Good overall: every `026-C3`/`026-C7` refusal is named, one line, and the
   file is untouched (`catalog_document.rs:29-76`, `cache.rs:407-477`); framing refusals precede any mutation
   (`http.rs:218-260`); the over-cap upsert is the named `422` measured on the post-`revision`-bump buffer with the
   model, `revision` and file untouched and no tmp (`state.rs:456-479`); the cold case is the named `503` with a
   legible reason phrase and never `[]` (`serve.rs:219-229`); the token fold does not early-return on length
   (`admin.rs:151-162`) and a disabled face is a `404`, never a `401` oracle (`serve.rs:172-181`). Three holes:
   findings 1, 4 and 5.
7. **Previous rounds.** First code pass; nothing to re-check.

## Findings

- **[major]** `apps/registry/src/serve.rs:206-215` with `apps/registry/src/corpus.rs:218-231` — the serving face
  caps its answer by candidate **count** (`MAX_CANDIDATES`) but never by **octets**, so it can emit a response the
  contract's own client refuses. `constants.rs:15` defines `MAX_RESPONSE_OCTETS = 65_536` and `grep` shows its only
  uses are `upstream.rs:134/193` (the inbound hop); `write_response` (`http.rs:317-320`) promises whatever
  `body.len()` it is handed. The corpus contract explicitly blesses records on every bound
  (`schemas/catalog/v1/spec.md:78-80` and `fixtures/valid/on-every-length-bound.json`), so 256 postings at
  `game_id` = 128 octets and `name` = 128 octets serialize to ≈71,900 octets (> `MAX_RESPONSE_OCTETS`, and over
  the shipped client's 5-digit `Content-Length` limit at `apps/discovery-cli/src/catalog.rs:348`). The shipped
  client then answers `the body promises N octets over the 65536-octet budget` → `Unreachable`, and so does another
  registry (`upstream.rs:193-198`), turning a legal root answer into a caching registry's `stale`/`503`. `026-C2`
  requires "exactly the response shape and **budgets** the shipped client pins (A1)"; the test that pins the count
  cap uses ~45-octet candidates (`candidates_join.rs:135-178`), so the hole is invisible to the gate.
  *Fix:* bound the answer by the same stable prefix — in `candidates_answer` (or `Catalog::candidates`), serialize
  and drop trailing postings while the body exceeds `MAX_RESPONSE_OCTETS`, keeping the first postings in `game_id`
  byte order so truncation stays stable — and pin it with a 256-record max-bound corpus; or record the acceptance
  explicitly in `spec.md`/build notes and in a named risk, since the proposal's §Budgets reconciles `026-B1`×`026-B2`
  but never prices the response side.

- **[major]** `docs/adrs/025-discovery-identity-and-catalog-matching.md:96-111` (unchanged by this diff) — the
  `026-L1` amendment ships in this branch (`apps/discovery-cli/src/catalog.rs:36-45,152-160,240-260`: the loopback
  allowlist and the non-loopback-resolution refusal are deleted) without the ledger annotations §Rollout step 6 of
  the proposal says **this run** ships: `025-C4` and `025-L1` have no `amended-by 026-L1` line (grep for `026` in
  that file: no matches), `025-R1` does not record ADR-026 as filling its catalog half, and `026-S1`
  (`docs/adrs/026-registry-service.md:207-211`) carries no partially-satisfied line even though the seed mints
  minted ids and `data/catalog/README.md:12-14` asserts the annotation exists. After this branch the ledger states
  the retired loopback-only endpoint policy as current law for shipped code that no longer enforces it, and a
  shipped contract README points at an annotation that is not there (`026-S1` also still promises "game ids drawn
  from the shipped manifest ids"). *Fix:* land the three `amended-by`/record lines (ADR-026's own Rollout item 6 and
  `021-L4`, the precedent being `docs/adrs/024-game-slug-identity.md:66-90`), or state in this run's record who
  lands them and why this branch is allowed to leave the ledger contradicting shipped behavior; correct
  `data/catalog/README.md` to match whatever actually ships.

- **[minor]** `schemas/catalog/v1/spec.md:56-57` ("The entry set is closed: any other entry, a missing entry, or a
  wrong-typed entry is a named refusal") against `apps/registry/src/corpus.rs:123-175` and `:292-346` — the loader
  reads the three names it wants (`fields.get(...)`) and silently ignores every other entry, at document level and
  at record level, and the admin upsert shares that same gate, so
  `{"game_id":…,"name":…,"source_keys":[…],"gameId":"typo"}` is accepted and stored. No `fixtures/invalid/` case
  covers it (the 17 shipped fixtures cover the other 12 rows of the spec's refusal table), so nothing can catch the
  divergence. Because `spec.md:10-15` names `corpus.rs` as this format's conformance surface, the contract asserts a
  refusal the only conformance checker does not perform. *Fix:* either reject unknown entries in `from_document` and
  `record_from_value` and ship the fixture the table row implies, or narrow the spec sentence to "a missing or
  wrong-typed entry is a named refusal; extra entries are ignored".

- **[minor]** `apps/registry/src/corpus.rs:101-109` with `apps/registry/src/main.rs:227-230` — a root instance
  pointed at a path that does not exist starts, serves, and answers `200 []` for every slug. The "absent file is the
  empty document" rule is stated for the caching instance's local-entries document ("created empty if absent",
  §Chosen design → CLI surface) and for the cache (`026-C7`); extending it to `--corpus` turns an operator typo in
  the only content argument a root has into a silently empty public catalog rather than the named startup refusal
  every other corpus fault gets. No suite covers it (no test in `apps/registry/tests/` starts a root with a missing
  corpus). *Fix:* have `State::root` require the corpus to exist (an `--allow-empty-corpus` flag or a load-with-
  `absent_is_error` parameter), keeping absent-is-empty for the local-entries and cache documents; pin it beside
  `catalog_document.rs`'s other startup refusals.

- **[minor]** `apps/registry/src/serve.rs:162-194` — for a `/v1/admin/**` path, `Dispatch::WrongMethod` is answered
  (a `405` plus `Allow`) before enablement is consulted, so on a routable bind with no token — the state `026-C5`
  defines as "admin **disabled**" — `PUT /v1/admin/stats` and `POST /v1/admin/games` answer `405` with their method
  lists while every carried method answers `404`. `026-C5` and §The admin face and auth say the disabled case is
  flatly "`404`" ("never a `401` oracle"), and a `405`+`Allow` is an existence oracle for the face the instance just
  said it does not have. The suite is aware of the seam and steps around it rather than closing it
  (`admin_auth.rs:14-16`: "Only methods a route actually carries are asked without a credential").
  *Fix:* in `admin_route`, return the named `404` for every dispatch result when `!admin.is_enabled()`, keeping the
  `405`/`Allow` behavior for an enabled face, and extend `a_non_loopback_bind_without_a_token_disables_the_admin_face`
  with one non-carried method.

- **[nit]** `apps/registry/src/admin.rs:74-85` — a parameterized path is a route only for `DELETE`, so
  `PUT`/`POST`/`GET /v1/admin/games/<valid-game-id>` falls to `Dispatch::NotARoute` → `404`. `026-C4`'s "A `405`
  names any method the route does not carry" reads onto that route too, and the code is inconsistent about its own
  rule: the collection path (`/v1/admin/games`) does answer `405` with `Allow: POST`. *Fix:* either answer `405`
  with `Allow: DELETE` once the segment is `game_id`-valid, or state the "parameterized paths are method-specific,
  others are `404`" rule in `spec.md`/the route doc so the two shapes are deliberate.

- **[minor]** run scope — `apps/registry/**` (≈5.0k src + ≈5.7k test lines) and the `026-L1` CLI amendment land in
  this branch, while `02-proposal.md:30-37` ("Non-goals (**scope guards for code review**): No product code in this
  run — the registry binary, loader, and CLI amendment ship as `T-004`…`T-008`"), `docs/adrs/026-registry-service.md:245-248`
  (Rollout 7: "no product code") and §Rollout step 5's accepted "contract ahead of code" divergence window all say
  they do not. The five task sources the proposal names as the vehicle (`docs/tasks/T-004`…`T-008`) do not exist in
  the tree (`docs/tasks/` holds only `T-001`…`T-003`), so the change ships outside the record the contract names,
  and the divergence window the proposal and Risks table describe as *open* is in fact closed by this branch.
  *Fix:* amend this run's record (build notes / the Rollout) to state that the implementation and the `026-L1`
  amendment landed here rather than as `T-004`…`T-008`, or move the code to those task runs; escalate this to
  blocking if `03-build-notes.md` neither declares the change nor the two deviations above (it is outside the inputs
  this pass was given, so I could not check).

- **[nit]** file plan and dependency wording — §`apps/registry` fixes a nine-module plan; the crate adds
  `keys.rs`, `persist.rs`, `state.rs` and a `#[cfg(test)]`-only `wire.rs` (599 lines of in-process wire suite
  living in `src/`), and `keys.rs:72-94` re-expresses the key rule through `Source::from_token` rather than calling
  `Source::spells_lookup_key`, which §`apps/registry` names as the reused predicate. Both are behavior-equivalent
  (`from_token` is `spells_lookup_key`'s only predicate, `crates/discovery/src/game.rs:84-89`). *Fix:* note the
  extra modules where the module list lives, or call `spells_lookup_key` directly so the closed-set reuse is one
  call rather than a reconstruction.

## Out of specialty (FYI, no severity)

`http.rs:234-260` buffers a declared `Content-Length` (up to `MAX_REQUEST_BODY_OCTETS` = 16 MiB, by the proposal's
own §Budgets identity) before the admin credential is checked, and `http.rs:288-305` rescans the accumulated header
block byte-by-byte for `\r\n\r\n` where the shipped client it copies uses an O(1) `ends_with` — both are
budget/performance-seat questions against a routable bind, not design mismatches.
