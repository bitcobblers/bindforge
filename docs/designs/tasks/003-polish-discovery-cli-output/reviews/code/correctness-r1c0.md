# Code review — correctness seat — round 1, pass c0

- Change under review: `git diff 092bdc29a602fd9711bf77eb039e9e1391a50969...HEAD -- ':(exclude)docs/designs'`
  (touches only `apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_tests.rs`,
  and the new `apps/discovery-cli/tests/cli_output_polish_tests.rs`).
- Design: `docs/designs/tasks/003-polish-discovery-cli-output/02-proposal.md`; task
  `docs/tasks/T-003-polish-discovery-cli-output.md`; the runner's gate record; my previous findings (none — first pass).
- Independent evidence: I re-ran `cargo test -p discovery-cli --locked`; all 8 tests pass,
  including the 3 new pins in `cli_output_polish_tests.rs`
  (`test result: ok. 3 passed` for that target; `test result: ok. 3 passed` for `cli_tests.rs`,
  `2 passed` for `cli_dual_plugin_tests.rs`). The gate record's lone
  `test result: ok. 0 passed` line is the renderer's tail (the last, empty doc-test target), not
  a sign the new tests skipped — `apps/discovery-cli/Cargo.toml` has no `autotests = false` and the
  new file is a top-level `tests/` target, and I observed it run and pass.

## Checklist walk

1. **Named components/interfaces exist and match.** ✓
   - `render_table(&[DiscoveredGame]) -> String` — private, in `main.rs`
     (`main.rs:106`), exactly the proposal's signature and name.
   - Columns stay Slug, Display Name, Install Path, Platform (`main.rs:31`).
   - `render_row` renders per column `"| "` + cell padded right to width (`main.rs:153-161`),
     matching the proposal's "`| ` + cell padded right".
   - `fn main() -> ExitCode` (`main.rs:33`); on the first `plugin.discover()` `Err`, one
     stderr line `error: plugin <name> failed: <DiscoveryError as Debug>` then
     `ExitCode::FAILURE` (`main.rs:67-70`).
   - CLI holds its own `Vec<Box<dyn DiscoveryPlugin>>` and registers the same plugins
     identically (`from_host` pair; cfg-gated `from_roots`) (`main.rs:42-59`); the
     aggregation loop appends in registration order, matching `DiscoveryEngine::discover_all`
     (`crates/discovery/src/engine.rs:20-33`, verified). `crates/` untouched (diff stat empty
     for `crates/`). ✓
2. **Assumption register.** The proposal has no assumption-register section; nothing to honor. N/A.
3. **Deviations declared in `03-build-notes.md`.** The caller's allowed inputs exclude
   `03-build-notes.md`, so declaration status cannot be established here — recorded as an
   evidence gap, not blocking (see nit below: the one literal-wording gap is a proposal-side
   arithmetic slip whose normative property — byte-offset alignment — is empirically
   established by the green alignment pins).
4. **Scope creep.** None. The diff is confined to the three files the proposal lists. The
   JSON `unwrap()` → stderr-message/`ExitCode::FAILURE` conversion is proposal §2 verbatim;
   no new flags, no output-format drift beyond §1-§3.
5. **`03-build-notes.md` claims.** N/A (outside the allowed inputs for this seat).
6. **Error paths exist and fail closed.** ✓ Plugin failure returns before any stdout write
   (stdout empty on failure, pinned). JSON serialization failure is a clean one-line stderr +
   non-zero exit (`main.rs:77-83`). No `unwrap()`/panic path remains on any in-scope failure
   in `main.rs`. Exit code pinned to 1 (`ExitCode::FAILURE`).
7. **Revision rounds.** First pass; no prior findings. ✓ N/A.

**Done-criteria cross-check:** aligned table + separator pinned by
`test_cli_list_table_columns_align_under_a_separator` (identical `|` byte offsets, separator
fills gaps, header + separator + exactly four fixture rows); header-only empty case pinned with
empty stderr; forced failure (non-UTF-8 `libraryfolders.vdf` → `PluginError`) pinned as one
stderr line naming `steam-linux`, exit 1, empty stdout, no `panicked at`, no engine-voice
`Error from plugin`. `test_cli_list_format` updated exactly as the proposal directs (old
unpadded header string replaced by padding/alignment assertions; slug, decoy and exactly-four
checks retained — `cli_tests.rs:111-134`). The `test_cli_list_json_*` tests are byte-unchanged
(the diff's only deletions in `cli_tests.rs` are the old header assertion and its comment).
Gate record: all four legs green.

## Findings

1. **[nit]** `apps/discovery-cli/src/main.rs:145` — `render_separator` emits `width + 1`
   dashes per column; proposal §1 says "`width + 2` dashes per column". `width + 1` is the
   arithmetically correct value for the same sentence's binding requirement ("every rendered
   line shares identical `|` byte offsets"): a body column occupies `1` space + `width`
   padded chars between delimiters, so `width + 2` dashes would push every following `|` one
   byte further and break the alignment the tests pin. The code satisfies the invariant; the
   proposal's dash count is the slip. Fix: record the arithmetic in `03-build-notes.md` (or
   correct the clause at editorial level) so the run record explains the one-word gap; no
   code change wanted. I could not verify declaration because `03-build-notes.md` is outside
   this seat's allowed inputs.

## FYI (no severity, outside this seat)

- The proposal pairs character-count column widths with byte-offset alignment; the two only
  coincide for ASCII cells. The code is faithful to the proposal as written (widths via
  `chars().count()`); non-ASCII display names would align visually but not by byte offset.
  Worth a future proposal-wording fix, not a code change.

## Verdict

approve — zero blocker/major findings; one nit.
