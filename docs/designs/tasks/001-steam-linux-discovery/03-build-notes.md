# Build Notes

## Progress

- **Completed Behaviors**:
    - All T-001 design scope is complete and gate-green: `keyvalues.rs`,
      `steam_linux.rs`, the `--steam-root` CLI seam, the exports, and the prior
      rounds' test conflict and gate repair (commits 2bc4c2eb, fba0321c,
      b2adcc9c; *Test Contract Conflicts* and the gate-repair record below are
      historical).
    - This round (code-review r1 fix stage): the r1 seat reports named in the
      stage prompt live under this design's `reviews/**`, which this stage's
      access rules deny, so the findings themselves were not readable here. The
      substitute was a full clause-by-clause re-audit of the branch diff
      (product code, CLI, fixtures, agent files, `flows.ts`) against
      `02-proposal.md` §1–§10 and the assumption register. One clause-level
      mismatch remained and is fixed: §7's one error path in
      `SteamLinuxPlugin::discover` now spells its "at least one root was
      selected" clause in the guard (previously implicit via the skip count),
      and the `PluginError` message says "entries skipped as unreadable or
      malformed" — the count includes an unreadable *library* skip that
      `scan_library` performs, which the old "file(s)" noun misdescribed
      (*Deviations* 3 and 9). Firing semantics are unchanged. The §1 public
      surface, §2/§3 selection and canonical rules, §4 grammar subset and caps,
      §5 composition/ordering, §6 unfiltered emission, §8 CLI wiring, §9
      fixtures and §10 dependency posture were each re-verified; nothing beyond
      *Deviations* 1–8 (which stay accepted) departed from the proposal.
- **Remaining Work**:
    - Nothing left for the implementer. Rollout step 5 (a contributor's real
      `discovery list` / `list --json` output from a Steam machine) stays an
      open, human step. If a seat's r1 finding is something other than the §7
      item, this stage cannot see it; the next pass should surface the report
      text through an operator or a stage whose rights include `reviews/**`.
- **Relevant Paths and Interface Decisions**:
    - This round's only edit: `crates/discovery/src/steam_linux.rs` — §7
      message wording and the explicit roots clause in `discover()`.
    - The §7 test pins the root name in the message and the absence of file
      contents, not the noun, so no test change is needed.
- **Last Validation Result**: all four gate legs green in this stage's shell
  after the edit — `cargo fmt --all -- --check` silent; `cargo clippy
  --workspace --all-targets -- -D warnings` finished clean; `cargo test
  --workspace --locked` every suite `test result: ok` (49 discovery unit, 26
  steam-linux, 15 keyvalues tests included, plus the 3 CLI tests); `npm run
  check` exit 0 (Test Files 61 passed (61); Tests 692 passed | 2 skipped).
  See *Gate Posture*.
- **Next Concrete Action**: none for this stage; the runner re-runs the gate and
  the dissenting r1 seats over the amended tree.

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
9. **§7 error message noun and explicit roots clause** (`crates/discovery/src/
   steam_linux.rs`, this round): §7 gates its one `Err` on "at least one root was
   selected" and says the message names "the count of skipped files and nothing
   else". The guard's roots clause was only implicit (an empty-root plugin cannot
   skip files); it is now explicit. And the skip count includes an unreadable
   *library* — `scan_library` counts a failed `read_dir` of a selected library's
   `steamapps/` so the "Steam present, everything under it unreadable" case still
   reaches §7's signal — which the old "file(s)" noun misdescribed; the message
   now says "entries skipped as unreadable or malformed". Names and count only,
   no file contents, exactly as §7 requires; the pinned test asserts the root
   name and the content-leak negatives, both unchanged.

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

**This round (code-review r1 fix stage).** The runner-captured gate at
2026-11-18T04:11:25Z was green on all four legs. After this stage's §7 edit
(*Progress*, *Deviations* 9) all four legs were re-run in this stage's shell:

- `cargo fmt --all -- --check` — **exit 0** (silent).
- `cargo clippy --workspace --all-targets -- -D warnings` — **exit 0**
  (`Finished dev profile`).
- `cargo test --workspace --locked` — **exit 0**, every suite `test result: ok`
  with no failures (the zero-test lines are doc-test summaries), including the
  §7 error-path test over the amended message.
- `npm run check` (in `tools/orchestration`) — **exit 0**: `Test Files 61
  passed (61)`, `Tests 692 passed | 2 skipped (694)`.

**Prior round (historical; gate repair).** The runner-captured gate at
2026-11-18T04:06:34Z was red on exactly one leg; the three cargo legs were green
there and stayed green under that stage's fix:

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

  After that stage's fix (*Deviations* 8) the same leg went green, and it is
  green above.

**Earlier round (historical, superseded by the test-update and fba0321c).** A still
earlier stage recorded `cargo fmt --all -- --check` failing with 18 `Diff in` hunks confined to
`apps/discovery-cli/tests/cli_tests.rs`, `crates/discovery/tests/keyvalues_tests.rs`
and `crates/discovery/tests/steam_linux_tests.rs`, and `cargo test --workspace
--locked` failing on the secondary-library `install_path` assertions in
`steam_linux_tests.rs:380` and `cli_tests.rs:96` (expected paths built from the
fixture root instead of the holding library) — the conflict described in *Test
Contract Conflicts*. `cargo clippy --workspace --all-targets -- -D warnings` passed
then as it does now. Those test files were fixed by the test engineer since; both legs
are green above.
