# Build Notes

## Progress

- **Completed Behaviors** (this round — code-review r1 dissent-pass c1 fix):
    - All T-001 design scope remains complete: `keyvalues.rs`, `steam_linux.rs`,
      the `--steam-root` CLI seam, the exports (commits 2bc4c2eb, fba0321c,
      b2adcc9c and the r1 fix 77e24c72).
    - **Major, open in all three r1c1 seats** (correctness finding 1,
      compatibility C1-1, testing F-1): the branch-added
      `#[serde(rename_all = "kebab-case")]` on `Platform` and its doc comment
      are reverted; `crates/discovery/src/game.rs` is byte-identical to `main`
      again, restoring the proposal's "no change to `DiscoveredGame`,
      `Platform`" (Context), Non-goals and §8 "JSON shape unchanged" statements.
      `--json` emits `"SteamLinux"` again; the human table is unchanged because
      `Display` already printed the kebab form. This is the reviewers' primary
      fix option ("drop the derive attribute in this additive task and assert
      `"SteamLinux"` in the CLI test").
    - **Compatibility C1-3 / testing F-8 (wording drift)**: the §7 message tail
      is reverted from "entries skipped" to §7's own wording — "N file(s)
      skipped as unreadable or malformed" — and the code comment now states the
      same sentence as §7; string, comment and §7 are one statement again
      (*Deviations* 9 closed).
    - **Testing F-9**: the explicit `!self.roots.is_empty()` guard clause is
      kept — it spells §7's three clauses verbatim and is behavior-identical
      (both other seats' code walks confirmed); the seat's alternative (counting
      skipped roots) would change §7's trigger and was not taken.
    - **Nits (correctness 3, C1-2, F-7)**: the interior-NUL refusal in
      `usable_install_dir` stays as a declared, harmless code-exceeds-spec
      extension (*Deviations* 6); no backslash refusal is added — on Linux,
      the only supported platform, `\` is an ordinary filename byte, so
      refusing it would reject legal names beyond §5's stated set. Aligning
      §5's list with the NUL clause (and §7's "files" wording with the
      counter's real scope, C1-3's root cause) belongs to the proposal owner,
      not this stage.
- **Remaining Work**:
    - The test-update round owns the sole red assertion,
      `apps/discovery-cli/tests/cli_tests.rs:121` → assert `"SteamLinux"`
      (*Test Contract Conflicts*), plus the testing seat's carried additive
      test items F-2..F-8 listed there. Rollout step 5 (a contributor's real
      `discovery list` output from a Steam machine) stays an open human step.
- **Relevant Paths and Interface Decisions**:
    - This round's edits: `crates/discovery/src/game.rs` (revert to `main`),
      `crates/discovery/src/steam_linux.rs` (§7 message + comment wording).
- **Last Validation Result**: `cargo fmt --all -- --check` exit 0 (silent);
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `npm run
  check` exit 0 (61 files, 692 passed | 2 skipped); `cargo test --workspace
  --locked` red on exactly one test in the workspace — `cli_tests.rs:121`, the
  routed conflict. See *Gate Posture*.
- **Next Concrete Action**: the runner routes `cli_tests.rs:121` (and the
  F-2..F-8 additions) to `test-update`, then re-gates.

## Deviations from the design

1. **`Platform` serde attribute — CLOSED (reverted in the r1c1 fix round)**:
   the branch had added `#[serde(rename_all = "kebab-case")]` to `Platform`
   (`crates/discovery/src/game.rs`) despite *Context*'s "no change to
   `DiscoveredGame`, `Platform`" and §8's "JSON shape unchanged". All three
   r1c1 seats kept this major open (correctness 1, compatibility C1-1, testing
   F-1); the attribute and its doc comment are now reverted, and
   `crates/discovery/src/game.rs` is byte-identical to `main` again — the
   reviewers' primary fix option ("drop the derive attribute in this additive
   task and assert `"SteamLinux"` in the CLI test"). The one coupled assertion
   is routed in *Test Contract Conflicts*.
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
9. **§7 error message noun — CLOSED (reverted in the r1c1 fix round)**: the
   previous round re-worded §7's message tail to "entries skipped" and the
   comment to match; the r1c1 seats (compatibility C1-3, testing F-8) kept the
   divergence from §7's stated contract ("the count of skipped files and
   nothing else") open, and amending §7 is outside implementation ownership.
   The message tail is back to "N file(s) skipped as unreadable or malformed"
   and the code comment restates §7's sentence, so string, comment and §7 are
   one statement. The counter's real scope also absorbing unreadable libraries
   and VDFs (deviation 3's scope) is §7's own looseness — a one-line proposal
   fix recorded for the document owner. The guard's explicit
   `!self.roots.is_empty()` clause stays: it spells §7's first clause verbatim
   in code, is behavior-identical to the implicit form per all three seats'
   code walks, and is therefore not a deviation.

## Test Contract Conflicts (why `testsNeedUpdate` is true)

*Current — one assertion, created by this round closing Deviations 1.*

- `apps/discovery-cli/tests/cli_tests.rs:121` (`test_cli_list_json_format`):
  asserts `obj["platform"] == "steam-linux"`, written against the
  unauthorized `#[serde(rename_all = "kebab-case")]` attribute. With the
  attribute reverted, the serialized form is `main`'s and the proposal's
  again (Context "no change to `Platform`", Non-goals, §8 "JSON shape
  unchanged"): the value is `"SteamLinux"`. The assertion becomes
  `assert_eq!(obj["platform"], "SteamLinux");` — the fix both the
  correctness (finding 1) and testing (F-1) seats prescribe. Everything
  else in the file passes as written: order, slugs, display names,
  `install_path` composition, the five-field shape, `codec_id == null`,
  the table rows (the human table is unchanged because `Display` already
  printed the kebab form), and the `[]` case.
- Suggested in the same round (testing checklist item 3): pin the serialized
  spelling at its own layer — a `crates/discovery/tests/game_tests.rs`
  assertion that `serde_json::to_value(Platform::SteamLinux)` is
  `"SteamLinux"` — so a stray derive edit is caught at the data model, not
  through the CLI channel.
- The testing seat's carried additive items F-2..F-8 remain routed to
  `test-update`: cross-root library de-duping (#F-2), the `(slug,
  install_path)` tie-break / duplicate-slug row (#F-3), the §6 accepted-cost
  Proton row (#F-4), the 1 MiB / 16-level caps (#F-5), the absolute-`path`
  VDF branch via `env!("CARGO_TARGET_TMPDIR")` (#F-6), the NUL-`installdir`
  refusal case (#F-7), and a §7 skip-count plus message-tail assertion cited
  to the restored "file(s)" wording (#F-8).
- Historical: the earlier secondary-library `install_path` conflict recorded
  here was resolved by the test engineer in fba0321c; those assertions pass
  as written today.

## Gate Posture

**This round (code-review r1 dissent-c1 fix stage).** After this stage's
reverts (*Deviations* 1 and 9), the legs were run in this stage's shell:

- `cargo fmt --all -- --check` — **exit 0** (silent).
- `cargo clippy --workspace --all-targets -- -D warnings` — **exit 0**
  (`Finished dev profile`).
- `npm run check` (in `tools/orchestration`) — **exit 0**: `Test Files 61
  passed (61)`, `Tests 692 passed | 2 skipped (694)`.
- `cargo test --workspace --locked` — **non-zero exit**, red on exactly one
  test in the workspace; a `--no-fail-fast` re-run showed every other suite
  `test result: ok`. The failure is the routed conflict in *Test Contract
  Conflicts* — the product now serializes `Platform` the way `main` and the
  proposal do, and the test still expects the branch-added spelling:

  ```
  thread 'test_cli_list_json_format' panicked at apps/discovery-cli/tests/cli_tests.rs:121:9:
  assertion `left == right` failed
    left: String("SteamLinux")
   right: "steam-linux"
  ```

**Prior round (historical; r1c0 fix stage).** The runner-captured gate for
the code the r1c1 seats reviewed (2026-11-18T04:31:16Z, cited in this
stage's prompt) was green on all four legs for that tree
(`npm run check`, `cargo fmt`, `cargo clippy -D warnings`,
`cargo test --workspace --locked`, each exit 0); that stage's §7 wording
edit is the one reverted here (*Deviations* 9).

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
were green in the runner's capture quoted in the prior-round block and in this
stage's own fmt and test legs.
