# Code correctness review — T-001 Linux Steam discovery (round 1, pass c0)

Seat: correctness. Scope: `git diff 4caa6889...HEAD -- ':(exclude)docs/designs'`
(source and test paths only). Inputs: that diff, `02-proposal.md`,
`docs/tasks/T-001-steam-linux-discovery.md`, the runner's gate record, and my own
prior findings (none — first pass). `03-build-notes.md` was **not** among the
permitted inputs, so deviation-declaration could not be confirmed (see F-1 note).

Gate record (runner, green): `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked` all exit 0.

## Checklist 1 — components/interfaces named in the proposal exist and match

PASS, with one deviation. `crates/discovery/src/lib.rs` exports `keyvalues` and
`steam_linux` alongside the existing five modules (lib.rs:1-7). `keyvalues.rs`
exposes `KeyValueError{line,message}`, `Value::{Str,Section}`,
`Section{children}` with case-insensitive first-match `get_str`/`get_section`,
and `parse` — names/signatures match §1. `steam_linux.rs` exposes
`SteamLinuxPlugin{roots}`, `from_host`, `from_roots`, `DiscoveryPlugin` (
`name() -> "steam-linux"`), pure `host_candidates(home, env)`,
`libraries_in_root`, `AppManifest{display_name,install_dir}`, `parse_manifest` —
all match §1. No `new()`/`Default` (clippy-clean). `DummyPlugin` is deleted and
replaced by `SteamLinuxPlugin` in `apps/discovery-cli/src/main.rs:33-38` with the
`--steam-root` seam (§8). **F-1 below** is the only interface-level divergence.

## Checklist 2 — assumption register honored

PASS for A1–A20 that the code touches. A9: `from_host` reads `HOME` via
`std::env::var_os` (`steam_linux.rs:39-41`), not the deprecated `home_dir()`;
unset/empty/relative home yields no candidates (`host_candidates` line 101-103).
A3/A12: only numeric-keyed blocks read; legacy keys dropped. A4: VDF read at one
fixed path, `appmanifest_*.acf` read only as direct children of `steamapps/`;
compatdata decoys unreachable. A8 candidate order matches §2 exactly
(`steam_linux.rs:105-134`). A11: results canonicalized/sorted/de-duped via
`BTreeSet` (`canonical_paths`) and output sorted by `(slug, install_path)`
(`steam_linux.rs:89`). A16/A4 caps enforced in `read_text_file` and `parse`.
A17: `discover(&self)` no-args held; roots stored on the plugin. No assumption is
silently reversed by the plugin logic. (The serialized-`Platform` issue is a
deviation, not an assumption-register row — filed under F-1.)

## Checklist 3/5 — deviations vs proposal and build-notes claims

**F-1 (major).** `crates/discovery/src/game.rs:7-8` adds
`#[serde(rename_all = "kebab-case")]` to `Platform`. The proposal states in three
places that `Platform` is not to change and that the JSON shape is unchanged:
Context line 14 ("no change to `DiscoveredGame`, `Platform`, ..."), Non-goals
line 35, and §8 line 262 ("the printing code is untouched, so the table header and
**JSON shape are unchanged**"). The attribute changes the serialized value of
`DiscoveredGame.platform` from `"SteamLinux"` to `"steam-linux"`, and the new CLI
test `apps/discovery-cli/tests/cli_tests.rs:121` hard-codes `"steam-linux"`, so the
test only passes because of this change — it is a coupled deviation, not a
pre-existing behavior. This is a user-visible JSON output change that the proposal
explicitly guaranteed would not happen.

*Not escalated to blocker:* ADR-022 (docs/adrs/022-game-discovery.md:34) documents
the platform name as `steam-linux`, and the `Display` impl already prints
`steam-linux`, so the change aligns serialization with the documented contract and
the table; the prior `DummyPlugin` always returned an empty list, so no live
consumer observed a PascalCase value, and the gates are green. I could not confirm
whether the deviation was declared in `03-build-notes.md` (not a permitted input);
if it is undeclared, this checklist item's "undeclared deviation" rule would make
it a blocker.

## Checklist 4 — scope creep

- **F-2 (nit).** Non-product files ride in the branch that the proposal never
  requested: `tools/orchestration/src/pipeline/flows.ts:92` (light-flow code-stage
  `stageTimeoutMs` 15→12 min) and `.opencode/agents/code/coder.md` /
  `test-engineer.md` (transcript read/glob/grep/shell `allow`→`deny`). These are
  pipeline-harness config, not Steam-discovery source or tests; they do not affect
  plugin behavior. Flagging only to confirm they belong to this run's change set.
- **F-3 (nit).** `usable_install_dir` (`steam_linux.rs:273-280`) also rejects
  values containing an interior NUL byte, which is beyond §5's rejection list
  ("absent, empty, `.`, `..`, absolute, or contains `/`"). Harmless hardening that
  the code comment openly acknowledges.

## Checklist 6 — error paths exist and fail closed

PASS. §7 skip-and-continue is implemented: unreadable/non-UTF-8/oversize files,
parse failures, and missing/malformed `libraryfolders.vdf` are skipped
(`read_text_file`, `library_paths`, `parse_manifest`) and the scan continues. The
single `Err` path fires only when zero games were emitted **and** at least one file
was skipped (`steam_linux.rs:75-87`); with no selected roots the scan never skips,
so "not installed"/no-home returns `Ok(vec![])`. The message names the roots and
the skip count and carries no file contents (verified against the `all-broken`
fixture leak-checks in `steam_linux_tests.rs:476-488`). `install_dir` is rejected
before unvalidated path composition; library paths are gated by canonicalization +
`steamapps`-dir check. One labeling nit (no finding): a failed `read_dir` on a
library is folded into the "file(s) skipped" count
(`steam_linux.rs:199-204`), so the §7 message's "file(s)" wording can over-count a
directory; behavior is still fail-closed and within §7's intent.

## Checklist 7 — prior-round blocker/major addressed

N/A — first pass, no prior findings.

## Out-of-specialty FYI

KeyValues reader byte slicing is panic-safe (token boundaries are ASCII
delimiters; multi-byte UTF-8 never contains `"`/`\\`/brace bytes, so
`from_utf8_lossy` is lossless) and cannot loop without progress — noted for the
security/performance seats, not a correctness finding.

## Findings summary

- F-1 major — `game.rs` `Platform` serde rename deviates from the proposal's
  explicit "no change to `Platform`" / "JSON shape unchanged" and changes
  user-visible JSON; the CLI test depends on it. Revert the attribute (and assert
  `"SteamLinux"`) to stay additive, or treat the data-model change as out of T-001
  scope and amend §8/Context to declare it.
- F-2 nit — non-product harness edits (`flows.ts`, `.opencode/agents/*`) in branch.
- F-3 nit — NUL-byte rejection in `usable_install_dir` beyond §5's rejection set.

Verdict: request-changes (one open major).
