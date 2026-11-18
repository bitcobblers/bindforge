# security review r1

verdict: approve

Scope: whole document (`02-proposal.md`), full-panel pass c0. Inputs: the proposal, `docs/adrs/024-game-slug-identity.md`, the Revision log (single initial entry — no rejected findings), and read-only verification against the current code.

## Checklist walk

1. **Trust boundaries** — Covered. Store-supplied display names and install-dir names, and rows from `dyn DiscoveryPlugin` (explicitly including "future third-party plugins"), are all treated as untrusted: grammar-and-bound is enforced at plugin construction (`steam_common::discovered_game` returns `None` on refusal) and again at the CLI emit boundary before any printing. No shared binding-file parsing enters this design (N/A).
2. **Parsing untrusted data** — No new parsers; the derivation is a pure total function whose output is refused (never truncated) past the bound. Verified `is_slug_name` (crates/core/src/placement.rs:74-84) is byte-wise and rejects non-ASCII, so the checked surface is closed ASCII. Fail-closed everywhere: refusal removes the row and increments `skipped`; no partial emission (per-row granularity is correct, not partial application).
3. **Path handling** — Verified in code: the grammar requires a leading ASCII lowercase letter and permits only `[a-z0-9-]`, so `/`, `.`, `..`, absolute paths and NUL cannot pass; Test strategy #2 pins exactly these shapes. The existing `usable_install_dir` rule (crates/discovery/src/steam_common.rs:258-264) already refuses `.`, `..`, absolute and nested install dirs before fallback derivation. No path is interpolated into a shell string anywhere in the design. The Windows reserved-name gap is deferred with a named owner and does not contradict the ADR's Decision (which changes the bound and identity shape, not the character class); the risk table's claim that "grammar already blocks traversal and separators" is confirmed against placement.rs.
4. **Secrets/PII** — The refusal echo goes through `escape_and_truncate` (verified at crates/core/src/error.rs:172-186: backslash/control escaping plus an 80-char bound, so no second-line injection and no unbounded echo). The CLI refusal line emits no install path of its own. No new data leaves the machine; the JSON row only gains the closed `namespace` field.
5. **Dependencies** — No new third-party dependency is named; `unicode-normalization` is explicitly rejected and the only new edge (`discovery → bindforge-core`) is in-workspace. Checklist item not applicable — nothing to advisory-check.
6. **Privileges** — No new privileges; the design adds a compile-time edge and reuses today's discovery read surface. Least privilege unchanged.
7. **Fail closed** — Confirmed: grammar/bound violation at construction → game never added; violation at CLI → row dropped with one named stderr line; collision candidate with no in-bound fit → refused and counted. `Namespace` is a closed serde enum, so no wire value can mint a namespace beyond `unknown`.

## Findings

- [nit][new] CLI boundary and output — "dropped, with one stderr line `error: refused game with invalid slug: <escape_and_truncate(value)>` and no path (024-L1)" — "and no path" overstates the mechanism if read as "no path-shaped text can appear on this line": `escape_and_truncate` (verified error.rs:172-186) escapes control characters and truncates but passes `/` and `.` through, so a hostile plugin that puts path-shaped text in the `slug` field has that text echoed (escaped, 80-char-bounded). What is actually guaranteed is single-line, bounded, escaped echo of the plugin's own value and no install path emitted by the CLI. Fix: tighten the clause to "and no CLI-sourced path; the echo is the escaped, length-bounded value only" — one reworded sentence, no new mechanism.

## FYI (no severity)

- Deferred Windows reserved-name refusal: consistent with the ADR (no grammar amendment mandated here) and named as known current behaviour by Test strategy #11; on the compatibility and complexity seats' side of the fence to weigh whether the eventual amendment ADR gets an owner before adoption (ADR-025) makes such ids land on disk.
- Correctness seats may want a look at the collision pass's "iterate distinct display names in byte order" interaction with rows sharing a display name across a plugin's libraries — the text asserts they share the assigned slug; security is indifferent since both rows are already grammar-checked.
