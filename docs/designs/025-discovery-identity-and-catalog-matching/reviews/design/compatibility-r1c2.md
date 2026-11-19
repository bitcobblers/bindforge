# compatibility review r1 — c2 ratification pass

verdict: approve

Scope: the blob diff `2efd6ade → 44618331` of `02-proposal.md` plus the full
current document, checked only for revisions that break what the compatibility
seat approved in round 1. Inputs: `02-proposal.md`, ADR-025, the Revision log,
this seat's r1 findings; repo verification via read-only inspection only
(`Cargo.toml` files, `crates/core/src/error.rs`, `apps/discovery-cli/src/main.rs`,
`crates/core/tests/ledger_annotations.rs`, `schemas/README.md`, the two test
directories named by Rollout 1 and 3). No review reports or transcripts opened.

## Diff-by-diff disposition against the r1 approved state

- A10 rewrite: conclusion ("no `Cargo.lock` change") re-verified true — the new
  modules' deps are a subset of the CLI's existing deps — but the enumeration
  regressed (finding 1).
- Bound / `schemas/README.md` bullet (added `MAX_NATIVE_NAME_OCTETS` borrowing
  clause): checked `ledger_annotations.rs` `bound_bullet` (selects the first
  line containing the literal `MAX_SLUG_OCTETS = 128`, tests/ledger_annotations.rs:238)
  — the new clause quotes no such literal; no test anchors on
  `MAX_NATIVE_NAME_OCTETS = 128` in the README. The r1 nit stays dodged-as-before;
  placement/phrasing still unpinned (unchanged, not a regression).
- Match store (one-record-per-slug, load-time budget + slug-shape validation,
  exclusive tmp + 0700/0600): the data-dir rule my r1 findings cited is
  byte-for-byte unchanged; Windows exempt from the mode rule via "where unix
  modes apply"; no new dependency or path convention. Capacity note below as FYI.
- Catalog client (`parse_endpoint` grammar, header rules, post-resolution
  loopback address check): host set unchanged from the approved
  `127.0.0.1`/`[::1]`/`localhost`; Test #9/#10 pins match the spec text
  (`@`-authority, `0177.0.0.1`, empty host, non-digit port all refused); std-only,
  toolchain-floor-clean.
- Scan flow step 6 (one record per slug) and step 4: consistent with the store
  bullet and `025-L3`; no stdout-contract delta beyond what #13/#14 re-pin —
  except step 8's new cell escaping (finding 2).
- Risks HTTP row: mirrors the parse spec; caps now match Test #10.
- Test strategy #4/#6/#7/#9/#10/#12/#14: each added pin traces to a spec bullet;
  #15 (platform) unchanged, so the macOS findings' subject is untouched.
- Rollout 1 and 3 re-pin/migration lists: every named suite exists in-tree
  (`crates/discovery/tests/{plugin_tests,game_tests,steam_linux_tests,steam_windows_tests,emission_tests}.rs`,
  `apps/discovery-cli/tests/{cli_tests,cli_output_polish_tests,cli_dual_plugin_tests,cli_slug_refusal_tests}.rs`);
  the `cli_slug_refusal_tests` route deletion is consistent with the bound-only
  boundary and Test #4's in-module pins.
- Revision log: two `r1:` entries (test-harness migration, table cells); neither
  claims my r1 findings were addressed or deliberately rejected — they simply
  were not acted on, and their sections are unchanged (see "Still open" below).

## Findings

- [minor][new] Scan flow and output — step 8: "the two ACF-sourced cells — `Display Name`, `Install Path` — render through `escape_and_truncate` in place of today's raw `row_cells` pass-through" — this r1-introduced rule reuses the refusal-echo renderer for routine table data, and that renderer (crates/core/src/error.rs:190-204, bound at :30) does two things beyond control-char escaping: it doubles every backslash and cuts at 80 characters with `…` (its own doc, error.rs:183-188, scopes it to rejection echoes). Consequence: every legitimate Windows `Install Path` renders with doubled separators (`C:\\Steam\\…`), and any install path or name past 80 chars is silently shortened in the shipped table on every platform; non-ASCII is preserved (only `is_control()` chars escape), so locale is fine but path display is not. Test #12's new case pins only the ESC-byte outcome; no pin or matrix note covers the backslash/length side effects, and the Windows matrix row's named gap is about test execution, not table rendering. Fix: replace "render through `escape_and_truncate`" with a cell rule that escapes control characters only (no backslash doubling, no 80-char cut) — naming `escape_and_truncate` as what it replaces — or keep the reuse and pin the Windows-path and long-path renderings in Test #12 plus a one-clause note on the Windows matrix row.
- [nit][new] Assumption register A10: "and those are the workspace's only external deps (`clap` and `serde_json` per-package, not workspace-level)" — the rewrite dropped `winreg`, which is a workspace external dep (Cargo.toml:23, consumed target-gated at crates/discovery/Cargo.toml:21-22) and is named by this proposal's own A9; the pre-revision A10 listed it correctly. The row's conclusion is unaffected (`winreg` is already in `Cargo.lock` and already compiles only for the Windows leg), and the matrix row it grounds ("Rust toolchain and dependencies | Yes") stays true. Fix: restore it — "…`serde`, `serde_json`, plus the target-gated `winreg` (A9), and those are the workspace's only external deps…".

## Still open from r1, unchanged by this revision (not re-reported as findings)

- macOS data-dir basis (matrix "macOS (APFS) | Yes, no CI leg" + Match-store path rule + A16) — text byte-for-byte unchanged.
- macOS platform-test level (Test strategy #15) — text unchanged.
- Missing "Breaking, in-tree only" matrix row for the `discovery` public API / CLI stdout contract — matrix untouched; the step-8 escaping change mildly widens the stdout-contract delta that row should one day enumerate.
- Bound-bullet placement/phrasing nit — still dodged by wording ("set equal to the two 128 bounds"); the added `MAX_NATIVE_NAME_OCTETS` clause introduces no new anchor collision (verified against `bound_bullet` and the README).

## FYI (no severity)

- Match store Load: "a file over a `65_536`-octet read budget" bounds the store at roughly 150 records at worst-case field sizes (3 × 128-octet fields + JSON punctuation) and ~400-800 at typical sizes; a breach's only stated remedy is deleting the file, losing every confirmed match. A one-sentence capacity statement (or a larger budget) would close it — capacity reasoning is correctness's call, not compatibility's.
- Step 6's prose has nested inline code (`` `(slug, game_id, friendly name of the first sharing row in `(slug, install_path)` order)` ``), which renders broken in Markdown — copy nit.
