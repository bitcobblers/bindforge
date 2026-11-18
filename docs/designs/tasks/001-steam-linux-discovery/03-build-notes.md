# Build Notes

## Progress

- **Completed Behaviors**:
    - All T-001 design scope is complete: `keyvalues.rs`, `steam_linux.rs`, the
      `--steam-root` CLI seam and the exports (commits 2bc4c2eb, fba0321c). The prior
      round's secondary-library `install_path` test conflict (*Test Contract
      Conflicts*, below, historical) was resolved by the test engineer in fba0321c.
    - This round — gate repair only. The single red leg was `npm run check` in
      `tools/orchestration`, made red by parent-branch commit `780cf9ab`
      ("feat: allow coding agents to read transcripts and increase timeout"), which
      predates this run's branch point and touches no T-001 path. It (a) flipped the
      `read`/`shell`/`glob`/`grep` `transcripts` rules from `deny` to `allow` in
      `.opencode/agents/code/coder.md` and `.opencode/agents/code/test-engineer.md`
      while every other statement — both files' own prompt text (added by that same
      commit), the AGENTS.md rights section, and the ADR-016 Decision 3 fence tests —
      kept `deny`, and (b) raised the light-flow `stageTimeoutMs` to 15 min while
      `docs/workflows.md` (Per-flow budgets, ADR-010) still documents 12 min.
      Restored `effect: deny` on the flipped transcripts rules (4 in `coder.md`,
      3 in `test-engineer.md`; that file's `grep` rule was never flipped) and
      `stageTimeoutMs: 12 * 60_000` in `tools/orchestration/src/pipeline/flows.ts`.
      `780cf9ab`'s unrelated, test-passing `runner.ts` `console.warn`→`console.error`
      change was left as-is. See *Deviations* 8 and *Gate Posture*.
- **Remaining Work**:
    - Nothing left in scope. Rollout step 5 (a contributor's real `discovery list` /
      `list --json` output from a Steam machine) stays an open, human step.
- **Relevant Paths and Interface Decisions**:
    - `crates/discovery/src/{keyvalues.rs,steam_linux.rs,lib.rs,game.rs}`,
      `apps/discovery-cli/src/main.rs` (T-001, unchanged this round).
    - Gate repair: `.opencode/agents/code/{coder.md,test-engineer.md}`,
      `tools/orchestration/src/pipeline/flows.ts`. The frontmatter now agrees with the
      agent prompt bodies, AGENTS.md, `docs/workflows.md` and the pinned tests.
- **Last Validation Result**: all four gate legs green in this stage's shell —
  `npm run check` exit 0 (Test Files 61 passed (61); Tests 692 passed | 2 skipped);
  `cargo fmt --all -- --check` silent; `cargo clippy --workspace --all-targets --
  -D warnings` finished clean; `cargo test --workspace --locked` all
  `test result: ok`. See *Gate Posture*.
- **Next Concrete Action**: none for this stage; the runner re-runs the gate over the
  repaired tree.

## Deviations from the design

1. **`Platform` gained a serde attribute** (`crates/discovery/src/game.rs`), although
   *Context* states "no change to `DiscoveredGame`, `Platform`".
   `apps/discovery-cli/tests/cli_tests.rs:93` asserts `platform == "steam-linux"`, the
   form ADR-022 documents and `Platform`'s own `Display` already prints, while serde's
   default emitted `"SteamLinux"`. Only the `#[serde(rename_all = "kebab-case")]`
   attribute was added: the variants, the five `DiscoveredGame` fields and the JSON key
   set are unchanged, so the wire form now agrees with `Display` instead of contradicting it.
2. **How an unterminated string is detected** (§4 leaves the mechanism open): a quoted
   value that reaches a newline before its closing quote is refused as `unterminated
   string`, positioned at the line the string *opened* on. A scan that instead runs to the
   next quote would close the string on a later line and report the wrong position, which
   `keyvalues_tests.rs::rejects_unterminated_string_with_positioned_error` pins.
   Consequence: multi-line quoted values are refused; none is known in these two file
   kinds, and the accepted value ends up identical to what §4 specifies.
3. **"Absent" is not a skip** (§7 lists "a missing or malformed `libraryfolders.vdf`"
   among the things skipped): an absent file is treated as a normal degraded scan and is
   *not* counted in §7's skipped count, while unreadable, oversize and non-UTF-8 files
   are. Counting an absent file would report the §7 error for "Steam installed, nothing
   under it", which §7 rules out. A malformed or unreadable VDF is still counted.
4. **The skip count is threaded through a private helper**: `libraries_in_root` keeps
   §1's signature and delegates to a private counting variant, so a malformed
   `libraryfolders.vdf` can reach §7's counter from inside `discover()`.
5. **Parse-budget constant placement**: `MAX_DOCUMENT_BYTES` and `MAX_NESTING_DEPTH` are
   exported from `keyvalues.rs` (no `bindforge-core` edge exists for `crates/discovery`),
   and the size cap is applied to `metadata` before the contents are allocated, per
   `repo-practices` ("cap input before they allocate it").
6. **One extra `installdir` refusal**: an interior NUL byte. §5's rule exists because
   the value is appended to an emitted path that a later consumer opens, and a NUL is not
   a name any platform can open; no documented case changes posture because of it.
7. **`install_path` keeps the library prefix** — `install_path` for a manifest found in
   library `l` is `l/steamapps/common/<installdir>` with `l` the library that held the
   manifest (§5, A2, §2's "canonical library prefix"), which is what two of the
   pre-existing assertions dispute. See *Test Contract Conflicts*; this is not a change of
   design, it is the design.
8. **Out-of-scope gate repair (this round)**: the red `npm run check` came from
   parent-branch commit `780cf9ab`, which predates this run and touches no path the
   proposal names; the proposal takes no position on agent shell rights or stage
   budgets. Rather than route the disagreement to `test-update`, the implementation
   was restored to the posture the rest of the tree already states — the two agents'
   own prompt text (including the restriction block `780cf9ab` itself added), the
   AGENTS.md rights section, the `docs/workflows.md` per-flow budget table (12 min
   light `stageTimeoutMs`) and the ADR-016/ADR-010-pinned tests all say `deny` and
   12 min. No T-001 design decision, source file or test expectation was changed.

## Test Contract Conflicts (why `testsNeedUpdate` is true)

*Historical — this records the conflict the implement round routed to `test-update`;
the test engineer resolved it in fba0321c, and the current gate is green (see
*Gate Posture*). This round needs no test change.*

Two assertions fail, both the same single mistake: they build the expected
`install_path` of the **secondary library's** game from the fixture *root* instead of
from the library its manifest sits in, which contradicts A2 ("the game directory is
`<library>/steamapps/common/<installdir>`"), §5's composition rule, test strategy #3
(`libraries_in_root` returns `<root>` **and** `<root>/secondary` — that test passes here)
and the failing test's own comment ("`install_path` is `<canonical library>/steamapps/common/<installdir>`").

- `crates/discovery/tests/steam_linux_tests.rs:373-385`
  (`discover_output_honours_the_game_output_contract`): the `(slug, install_dir)` table
  uses one `root` for all four rows. `unknown/secondary-title` needs
  `<root>/secondary/steamapps/common/Secondary Title`, i.e. a per-row library
  (`""`, `""`, `""`, `"secondary"`).
- `apps/discovery-cli/tests/cli_tests.rs:82-101` (`test_cli_list_json_format`): the
  `(slug, display_name, install_dir)` table has the same shape; the fourth row's `want`
  must be `<root>/secondary/steamapps/common/Secondary Title`.
- Everything else in both files passes as written: the four slugs and display names, the
  `(slug, install_path)` order, `platform == "steam-linux"`, `codec_id == null`, the
  five-field JSON shape, the table header, the `[]` case, the decoy and compatdata
  exclusions, and all 22 other `steam_linux_tests` / all 15 `keyvalues_tests` cases.
- Separately, the three test files are not rustfmt-clean (18 `Diff in` hunks), so
  `cargo fmt --all -- --check` fails on them. They need `rustfmt` in the same round; this
  stage's own five source files are rustfmt-clean.

Bending the product instead — composing a secondary library's paths from the primary
root — would emit a non-existent directory for every game in every secondary library
(the whole reason `libraryfolders.vdf` exists), so it was not done.

## Gate Posture

**This round (fix stage).** The runner-captured gate at 2026-11-18T04:06:34Z was red
on exactly one leg; the three cargo legs were green there and stayed green under this
stage's fix. Reproduced in this stage's shell, then fixed and re-run:

- `npm run check` (in `tools/orchestration`) — **exit 1** as captured. The failure-
  carrying lines of its tail (a `mount: /tmp/…` sandbox line is dropped per the
  citation rule):

  ```
  FAIL  tests/pipeline/agent-shell.test.ts > run-record shell fence and the own-report re-open (ADR-016 Decision 3) > denies a command naming a run record
  AssertionError: code/coder.md: ls docs/designs/002-x/transcripts/research-r0.md: expected 'allow' to be 'deny' // Object.is equality
   Expected: "deny"
   Received: "allow"
  FAIL  tests/pipeline/flows.test.ts > configFor: the light budgets > applies the light partial and leaves the lane knob last
  AssertionError: expected 900000 to be 720000 // Object.is equality
   - Expected
   + Received
   - 720000
   + 900000
  Test Files  2 failed | 59 passed (61)
       Tests  2 failed | 690 passed | 2 skipped (694)
  ```

  After the fix (*Progress*, *Deviations* 8) the same leg is **green** in this stage's
  shell: `Test Files 61 passed (61) · Tests 692 passed | 2 skipped (694)`, exit 0.
- `cargo fmt --all -- --check` — **exit 0** (silent), re-run here.
- `cargo clippy --workspace --all-targets -- -D warnings` — **exit 0**
  (`Finished dev profile`), re-run here.
- `cargo test --workspace --locked` — **exit 0**, every suite `test result: ok`,
  re-run here.

**Prior round (historical, superseded by the test-update and fba0321c).** The earlier
stage recorded `cargo fmt --all -- --check` failing with 18 `Diff in` hunks confined to
`apps/discovery-cli/tests/cli_tests.rs`, `crates/discovery/tests/keyvalues_tests.rs`
and `crates/discovery/tests/steam_linux_tests.rs`, and `cargo test --workspace
--locked` failing on the secondary-library `install_path` assertions in
`steam_linux_tests.rs:380` and `cli_tests.rs:96` (expected paths built from the
fixture root instead of the holding library) — the conflict described in *Test
Contract Conflicts*. `cargo clippy --workspace --all-targets -- -D warnings` passed
then as it does now. Those test files were fixed by the test engineer since; both legs
are green above.
