# Code review — security seat — round 1, pass c0

- Run: 024-game-slug-identity (ADR-024, design 02-proposal.md)
- Scope: `git diff e786de8f6820d71dbe919300e247f72bd59ef7f3...HEAD -- ':(exclude)docs/designs'`
- Inputs used: the diff, `02-proposal.md`, `docs/adrs/024-game-slug-identity.md`, the
  runner's gate record (green, 2026-11-18T19:08:59Z), and my own re-run of
  `cargo test --workspace --locked` (all binaries green; the gate record's lone
  `0 passed` line is the `bindforge-core` lib unittest binary, which has no inline
  tests — every integration binary reports real passing counts, including the new
  `slug_bound_corpus` (5), `collision_tests` (7), `emission_tests` (9),
  `slug_tests` (10), `error` (4), `cli_slug_refusal_tests` (4)).
- No previous findings (first pass). No other review reports opened.

## Checklist walkthrough

### 1. External input validated at the boundary — PASS

The new untrusted surface is (a) store-supplied display names/install dirs inside
`appmanifest_*.acf`, and (b) rows from any future `dyn DiscoveryPlugin`. The
derivation (`crates/discovery/src/slug.rs:76`) emits pure ASCII confined to
`[a-z0-9-]` by construction, never truncates, and returns `None` on nothing-derivable.
The value is then re-checked against the shared predicate and bound at the plugin
construction seam (`crates/discovery/src/steam_common.rs:245`) and again at the CLI
render seam (`apps/discovery-cli/src/main.rs:88-102`) before either rendering. A
third-party plugin's slug cannot bypass both seams. Shared binding-file parsing is
unchanged; `Namespace` deserialisation is closed to the single v1 value
(pinned in `crates/discovery/tests/game_tests.rs`).

### 2. Path handling — PASS

The grammar now rejects `/`, `.`, `..` and every separator shape for slugs
(`crates/core/src/placement.rs:89`, refusals pinned in `crates/core/tests/placement.rs`
and `chain_ops.rs`). `install_dir` reaches `library.join(..)` only through the
unchanged `usable_install_dir` gate; the new slug fallback `derive_slug(&install_dir)`
can only output ASCII slug characters. No absolute path enters a join from untrusted
data; no string-interpolated shell command anywhere in the diff (the only
`process::Command` uses are integration tests spawning the built binary with an
argument vector, no shell).

### 3. Injection — PASS

No regex is added, so no catastrophic-backtracking surface. Echoes of hostile values:
the CLI refusal line runs the offending slug through `escape_and_truncate`
(`crates/core/src/error.rs:190`) — control characters (including `\n`, `\r`, ESC,
NEL) become `\uXXXX` escapes, backslashes double, the value is cut at the message
budget; pinned in `crates/core/tests/error.rs`. The new `action.rs`/`document.rs`/
`manifest.rs` bound refusals deliberately lead with the interpolated number so the
truncated echo cannot hide the bound it names. Plugin-side refusals are count-only —
nothing per-row is echoed (`steam_common.rs:212-220`, `crates/discovery/src/steam_common.rs`
skip path), matching `024-L1`.

### 4. Secrets and PII — PASS

Diff-wide grep finds no secrets/tokens/keys. The refused-row stderr line names the
slug only, never the install path (design's `024-L1`); the root-wide failure
diagnostic names roots and counts but never the refused game's text — pinned by
`apps/discovery-cli/tests/cli_slug_refusal_tests.rs`. `install_path` output is the
pre-existing ADR-022 behavior, unchanged here.

### 5. New dependencies — PASS

`Cargo.lock` adds no new packages: only two dependency edges to the existing
workspace-internal `bindforge-core`, and `serde_json` as a `crates/discovery`
dev-dependency (the same already-locked version the CLI uses, justified by the
crate's first serde shape pins). No known unfixed advisory applies. The dependency
direction (`discovery → core`) matches the proposal's chosen option and keeps core
root-zero-dependency.

### 6. Filesystem writes — PASS

Product code adds no writes. Test helpers write only under the system temp dir with
pid+nonce+time uniqueness (`crates/discovery/tests/common/mod.rs:41-44`) and remove
their trees on `Drop`.

### 7. Clean rejection — PASS

No new panic paths: the `as_bytes()[0]` index in `slug.rs` is guarded by the prior
empty check; the collision-suffix probe (`steam_common.rs:278`) terminates because
candidates strictly grow and the loop breaks at the bound; refused rows are dropped
with a single named stderr line, and the test suite pins `no "panicked at"` on the
refusal paths.

## Notes (no severity)

- FYI: `docs/adrs/013-game-manifests.md` is not in the diff and still carries no
  `amended-by 024-B1` annotation on `013-L5` — the ledger amendment the proposal's
  "Specs, fixtures, ledger" and Rollout item 3 require appears to be missing; that
  is a process/correctness observation outside this seat, noted for the panel.
- FYI: `escape_and_truncate` passes U+2028/U+2029 through unescaped (not Rust
  "control" characters); on the stderr terminal this seat evaluates they cannot
  break a line, so no finding — noted only in case a future consumer treats the
  echo through a JavaScript-flavored line-splitting lens.

## Verdict

approve — zero open blocker or major security findings. All seven checklist items
pass on the cited evidence.
