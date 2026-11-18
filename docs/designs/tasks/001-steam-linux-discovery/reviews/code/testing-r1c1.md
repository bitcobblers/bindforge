# Code review — testing seat — round 1, dissent pass c1

- Change under review: `git diff 752fd5abc9e4e88b57a98ef1c23c31daa3983c13..HEAD -- ':(exclude)docs/designs'`
- Scope of the revision: **one hunk only** — `crates/discovery/src/steam_linux.rs`
  (`discover()` §7 error path): the condition gains a `!self.roots.is_empty()`
  clause and the message is re-worded `file(s) skipped` → `entries skipped`.
  Full-branch posture (`git diff --stat main...HEAD`, paths outside docs/designs)
  otherwise unchanged: `crates/discovery/src/game.rs` still carries the +3
  kebab-case serde lines; no test file was touched by this revision.
- Inputs used: the diff, `02-proposal.md` (full current document),
  `docs/tasks/T-001-steam-linux-discovery.md`, the runner gate record, my r1
  findings. Verified current state at HEAD of `game.rs`, `game_tests.rs`,
  `steam_linux_tests.rs`, `cli_tests.rs`, `keyvalues_tests.rs`.

## Gate evidence (supplied by the runner, not re-executed)

Gate at 2026-11-18T04:31:16Z, green for the code under review:
`npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked` all exit 0. A failing required gate would
block; none fails. No check needed re-execution.

## Checklist

1. Test-strategy citations (#1–#9 `proves:` markers): present — no regression.
2. Unit + integration layers: present — no regression.
3. **FAIL.** "Each previously rejected blocker/major now has a test that would
   have caught it." The r1 major (Platform JSON spelling) received no code or
   test response: `crates/discovery/src/game.rs:8` still carries
   `#[serde(rename_all = "kebab-case")]`; `crates/discovery/tests/game_tests.rs`
   (19 lines at HEAD) still has no serde assertion; the repo-wide only pin of
   `"steam-linux"` remains `apps/discovery-cli/tests/cli_tests.rs:121`, written
   against post-change behavior.
4. Determinism: the changed hunk adds none (no clock/network/env). OK.
5. Behavior-describing names: OK.
6. Gate evidence assessed first (above). OK.
7. Untestable additions: **new hunk adds one** — see F-9.

## Findings

### F-1 (major, carried from r1 #1 — OPEN)

- Location: `crates/discovery/src/game.rs:5-8` (asserted only at
  `apps/discovery-cli/tests/cli_tests.rs:121`)
- Problem: the r1 major is unaddressed by this revision. `Platform` still
  serializes as `"steam-linux"` against three current-document statements —
  Context:14-15 ("no change to `DiscoveredGame`, `Platform`"), Non-goals:35,
  §8:262 ("the printing code is untouched, so the table header and JSON shape
  are unchanged") — and the guard layer is unchanged: the data-model test file
  `crates/discovery/tests/game_tests.rs` contains no serialize/deserialize
  assertion at all, so the suite still cannot distinguish a ratified rename
  from an accidental derive edit, nor catch a revert of the derive or the
  dropped acceptance of the old spelling on deserialize. Checklist item 3
  requires a test that would have caught this finding; none was added.
- Fix (unchanged from r1): either drop the derive attribute in this additive
  task and assert `"SteamLinux"` at cli_tests.rs:121, or keep the ADR-022-
  aligned kebab form and add a cited test at its own layer
  (`crates/discovery/tests/game_tests.rs`:
  `serde_json::to_value(Platform::SteamLinux) == "steam-linux"` plus a
  round-trip, with `#[serde(alias = "SteamLinux")]` / `"SteamWindows"` if
  prior output must keep deserializing), correcting §8:262 in the same change.

### F-2 (minor, carried — OPEN)

- Location: `crates/discovery/tests/steam_linux_tests.rs` (missing test);
  property at `crates/discovery/src/steam_linux.rs:62-70`
- Problem: §2's per-`discover()`-call library de-duping ("no library is ever
  scanned twice, including by two roots sharing one") is still pinned only
  within one root (the `"0" → ".."` case); no
  `from_roots([fixture("minimal"), fixture("minimal/secondary")])` test and no
  repeated `--steam-root` CLI test exist. Hoisting `canonical_paths` to a
  per-root call would keep every test green.

### F-3 (minor, carried — OPEN)

- Location: `crates/discovery/src/steam_linux.rs:91` (sort key); fixtures
- Problem: the `install_path` tie-break of the `(slug, install_path)` sort and
  §5's "identical slugs from two libraries stay two rows" remain untested —
  the fixture set still yields four distinct slugs.

### F-4 (minor, carried — OPEN)

- Location: §6 cost vs. `scan_library`/fixture set
- Problem: the accepted cost "Proton builds and the Steam Linux Runtime appear
  in `discovery list` as rows" (Risks: High) is still unpinned; every fixture
  manifest still reads like a game with `"StateFlags" "4"`.

### F-5 (minor, carried — OPEN)

- Location: `crates/discovery/src/keyvalues.rs` `MAX_DOCUMENT_BYTES`;
  `steam_linux.rs` `read_text_file`
- Problem: the duplicated 1 MiB caps remain untested; inverting a comparison
  or drifting either constant breaks no test.

### F-6 (minor, carried — OPEN)

- Location: `crates/discovery/src/steam_linux.rs:154-160` (absolute `path`
  branch)
- Problem: the absolute-library-path branch — the shape every real
  `libraryfolders.vdf` uses — is still never exercised; §9's sanctioned
  `CARGO_TARGET_TMPDIR` escape hatch was not used.

### F-7 (nit, carried — OPEN)

- Location: `crates/discovery/src/steam_linux.rs` `usable_install_dir`
- Problem: the `contains('\0')` refusal still has no test case.

### F-8 (nit, carried and worsened by this revision — OPEN)

- Location: `crates/discovery/src/steam_linux.rs:84` (message);
  `crates/discovery/tests/steam_linux_tests.rs:464-492`
- Problem: §7's skip-count component is still unasserted — the all-broken test
  checks the root name and the no-contents half only. This revision then
  changed the one user-visible string §7 prescribes
  (`{} file(s) skipped as unreadable or malformed` →
  `{} entries skipped as unreadable or malformed`) with no test change: no
  test pins the count, the count-bearing noun, or any wording of the message,
  so the re-word from §7's stated "count of skipped files" to "entries" is an
  unpinned contract drift and a further wording edit would stay silent.
- Fix: assert the message contains `; 2 ` (all-broken fixture: two skipped
  manifests) and pin the fixed message tail, cited `// proves: Chosen design
  §7`, so both count and wording are stated expectations.

### F-9 (nit, new — introduced by the changed code)

- Location: `crates/discovery/src/steam_linux.rs:77`
  (`!self.roots.is_empty()` clause)
- Problem: the clause is unreachable-as-distinguishing: `skipped` is only
  incremented inside the `if !self.roots.is_empty()` scan block (lines 61-71),
  so `skipped > 0` already entails at least one selected root. No test can
  exercise a combination where the new clause changes behavior (roots empty +
  skipped > 0 is unconstructible), making it an untestable addition per
  checklist item 7, while the new comment claims it spells out a §7 clause the
  suite cannot verify.
- Fix: either drop the redundant clause (the scan-block guard plus
  `skipped > 0` already encode the three §7 clauses), or make the roots-empty
  precondition real (e.g. count skipped roots) so a test can pin each clause.

FYI (out of specialty, no severity): the §7 message's noun change
"file(s)" → "entries" may be more accurate for `skipped`'s actual contents
(VDF skips plus manifest skips) but drifts from §7's "count of skipped files"
wording — for the correctness seat to reconcile with the design text.

## Verdict

request-changes — the r1 major (F-1) received no code or test response in
this revision, and checklist item 3 therefore fails; the revision's only code
change adds an untestable clause (F-9) and re-words the §7 message with no
test (F-8).
