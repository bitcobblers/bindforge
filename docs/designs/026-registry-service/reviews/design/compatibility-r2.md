# compatibility review r2

verdict: approve

Scope: the single editorial confirmation pass of round 2 — the diff at
`reviews/design/editorial-r2.diff`, read against `02-proposal.md`,
`docs/adrs/026-registry-service.md`, and the proposal's Revision log. Previous
findings for this seat: none (the editorial phase is one language pass per
round), so the pass asks one question: did the editorial reflow break a
compatibility claim or make a section false? It did not.

## Checklist walk

1. **Compatibility matrix** — the diff does not touch the matrix; every row
   still carries a status (Linux Yes, macOS Yes, Windows Yes, Other/exotic
   Unix No). No `unknown`, no empty cell. The paragraph under it ("the three
   `Yes` rows … are the rows the tasks owe behavior to") is unchanged.
2. **OS input APIs (X11/Wayland/Raw Input/HID)** — not applicable: the design
   is a plain-HTTP service over `std::net::TcpListener`; no platform input API
   is in scope. The `winreg` edge the proposal names is disclosed, not silent
   (verified: `crates/discovery/Cargo.toml` puts `winreg` under
   `[target.'cfg(windows)'.dependencies]`, so the `--upstream`/`Source` reuse
   pulls it only on Windows, and §Deferred retires the edge with the lift).
3. **Node version floor** — not applicable: no Node/JS surface in the diff or
   in `tools/orchestration` is touched.
4. **Native modules** — as above: the only native edge is the disclosed
   `cfg(windows)` `winreg` transit through `discovery`; no new native
   dependency is introduced by the changed text.
5. **Keyboard reality (non-QWERTY, dead keys, IME, non-ASCII names)** — not
   touched by the diff; `name` stays octet-bounded (`MAX_NATIVE_NAME_OCTETS`,
   verified = 128 in `crates/core/src/manifest.rs`) and the decoded-slug rule
   still refuses only control bytes / invalid UTF-8, so non-ASCII keys and
   display text remain admissible opaque strings.
6. **File paths** — unchanged by the diff. Re-checked the shipped rule the
   matrix cites: `apps/discovery-cli/src/matches.rs` (`data_dir`, `store_path`)
   uses `PathBuf` joins and `StoreLayout::Windows` → `APPDATA` with
   drive-prefix/absolute test, so "relative = unset" and the
   `<data>\bindforge\registry\{cache-v1,catalog-v1}.json` spelling hold;
   `host_layout()` picks `StoreLayout::Unix` off Windows, which is what the
   macOS row's "same `<data>` rule" depends on.
7. **Platform test per supported row** — unchanged: #19 names Linux via the
   cargo gates and Windows/macOS via the #13 `StoreLayout` env-closure seam
   plus case-unique fixture names, and states the no-native-runner limit
   plainly rather than claiming CI coverage it does not have.

## Verification run (read-only)

- "core's thirteen-constant family" (the diff only re-wrapped this phrase):
  `grep '^pub const' crates/core/src` returns exactly 13 — the claim holds.
- "`026-S1` is partially satisfied" (diff changed only the adverb placement):
  `data/manifests/` contains only `README.md`, so A14's basis for the partial
  satisfaction and the §Rollout step 6 ledger annotation still stands.
- "`probe()` drops the `is_loopback()` … refusal" (was "deletes"): matches the
  shipped site `apps/discovery-cli/src/catalog.rs:248` and ADR `026-L1`'s own
  word ("drops its loopback-resolution refusal and keeps the fail-closed
  connect"). No scope drift from the ADR.
- "answers exactly one request before `http.rs` closes the connection" (was
  "the one request §http.rs closes"): consistent with the module text ("one
  answer per connection") and with the shipped client, whose request is built
  with `Connection: close` (`catalog.rs:438`), so single-shot connections cost
  the shipped pair nothing.
- A3/A19/A11 anchors re-confirmed: `MAX_DISCOVERY_SLUG_OCTETS = 128`,
  `MAX_RESPONSE_OCTETS = 65_536`, `MAX_CANDIDATES = 256`, `percent_encode` /
  `candidates_url`, `LOOPBACK_HOSTS` all present where the proposal cites them.

## Findings

None. No blocker, no major, no minor.

The one edit in the diff that carries meaning rather than rhythm is Test-strategy
#10: "would pass `MAX_CATALOG_DOCUMENT_OCTETS`" → "would exceed". That edit
removes a latent contradiction instead of creating one — as written before, the
named `422` would have pinned the opposite of §The admin face ("one over
`MAX_CATALOG_DOCUMENT_OCTETS` … is the named `422`") and §Budgets ("checked
against the exact post-`revision`-bump buffer"). #1's "as a named refusal" and
the seed's "partially satisfied" likewise restate what the sections and the
`026-S1` ledger annotation already say.

## FYI (no severity)

- For the editorial seat only: the `MAX_CONCURRENT_CONNECTIONS` re-wrap in
  §Budgets left a ragged three-line break ("so ≤64 live / threads and ≤64
  in-flight / upstream hops") and dropped the `§` marker from the `http.rs`
  cross-reference, which every other reference in the document carries. No
  effect on meaning or on any platform claim; formatting only.
