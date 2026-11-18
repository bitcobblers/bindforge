# Build Notes

## Progress

- **Completed Behaviors** (this round — as-built reconciliation after the
  code-review r1 ratification):
    - The routed conflict is **closed**: `test-update` r1 (c9fb37b7) re-pinned
      `apps/discovery-cli/tests/cli_tests.rs:124` to
      `assert_eq!(obj["platform"], "SteamLinux")`, exactly what the correctness
      (finding 1) and testing (F-1) seats prescribed, and the runner's gate for
      that tree is green on all four legs (see *Gate Posture*). *Test Contract
      Conflicts* below is closed.
    - **Code-review round 1 is ratified in every seat**: correctness r1c2,
      compatibility r1c2 and testing r1c2 approve with 0 blocker / 0 major;
      security r1c3 and performance r1c3 ratify with 0 blocker / 0 major. What
      remains open is minors and nits only (listed in *Remaining Work*).
    - `crates/discovery/src/game.rs` at HEAD is byte-identical to `main`
      (confirmed independently by the correctness, compatibility and security
      seats), so the merged change set is purely additive on the data model, and
      `crates/discovery/Cargo.toml`, `apps/discovery-cli/Cargo.toml` and
      `Cargo.lock` appear nowhere in `git diff 4caa6889…HEAD` — §10's dependency
      posture held.
    - **All deviation items below are now mirrored in `02-proposal.md` →
      *As-built* (2026-11-17)**, the proposal-side record of the same drift
      (drift, reason, affected sections). No design decision was changed to match
      the code; the two document-owner items (the §5 NUL clause / Risks row 3,
      and §7's skip-count noun) are recorded there as edits the document owner
      owns, not as implemented changes.
    - **Verification pass completed 2026-11-17 (HEAD a6619bb0).** The whole
      *As-built* list (items 1–13) and every open item in *Deviations from the
      design* 2–14 were re-checked line-by-line against the source at HEAD —
      `keyvalues.rs`, `steam_linux.rs`, `main.rs`, both test files, the two
      fixture trees, the two agent-rights files and `flows.ts` — and every
      claim holds; the reconciliation note under *As-built* in `02-proposal.md`
      carries the per-item evidence and file:line anchors. No item was
      inaccurate, none closed as designed was reopened, and the pass found no
      drift beyond the recorded thirteen (notably: `from_host` and both caps
      referenced by no test, exactly as item 13 states; the minimal fixture's
      VDF carries only relative `path` values, which is why *Remaining Work*
      lists the absolute-`path` arm as never exercised). No new deviation item
      was added and no design decision was amended.
    - All T-001 design scope remains complete: `keyvalues.rs`, `steam_linux.rs`,
      the `--steam-root` CLI seam, the exports (commits 2bc4c2eb, fba0321c,
      b2adcc9c, the r1 fixes 77e24c72 and 752096e9, and the test update c9fb37b7).
    - **Carried record from the r1c1 fix stage (historical; all four bullets below
      were confirmed closed or unchanged by the r1c2/c3 seats):**
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
- **Remaining Work** (nothing gates; all of it is what the r1 seats left open):
    - Testing seat's five minors (non-blocking, coverage-only, all routed away
      from this stage): §2 cross-root "one set per `discover()` call" untested;
      the `(slug, install_path)` tie-break / duplicate-slug row untested (§5's
      accepted cost); §6's Proton / runtime accepted cost unpinned; the 1 MiB
      per-file cap untested on both the `parse` leg and the `read_text_file` leg;
      the absolute-`path` arm of `library_paths` never exercised (the only
      fixture VDF is relative). Three nits: no `"\0"` case in
      `parse_manifest_rejects_unusable_installdir`, nothing pins §7's skip count
      or message tail beyond the root name, and the redundant
      `!self.roots.is_empty()` clause is untestable by construction.
    - Security minor (carried, unchanged): `read_text_file`'s pre-read
      `metadata` length cap has a TOCTOU window and the fixed
      `libraryfolders.vdf` leg has no `is_file()` gate.
    - Performance nits (carried, advisory): `canonical_paths` runs twice per
      library (per-root at `steam_linux.rs:162`, merged set at `:68`), and every
      manifest is stat'd twice (`is_file` at `:218`, then `metadata` at `:291`)
      before it is opened.
    - Correctness / compatibility nit for the **document owner**, not the code:
      §5's reject list and Risks row 3 do not mention the interior-NUL refusal
      the code enforces, and §7's "file(s)" noun is narrower than the counter it
      prints. Both are recorded in *As-built* items 4 and 7; amending §5/§7 is
      outside implementation ownership.
    - Rollout step 5 (a contributor's real `discovery list` and
      `discovery list --json` output from a Linux machine with a real Steam
      install, plus one real `appmanifest_*.acf` and the machine's
      `libraryfolders.vdf` if shareable) stays an open human step; it is the task's
      Done-when check and the only remaining evidence for A6/A7.
- **Relevant Paths and Interface Decisions**:
    - This round's edits are documentation only: `02-proposal.md` (new *As-built*
      section, plus its 2026-11-17 re-verification note) and this file. The
      verification pass above read source, tests and fixtures but changed none
      of them: no source, test, fixture or schema path changed.
    - The built surface, unchanged since c9fb37b7: `crates/discovery/src/lib.rs`
      (+`keyvalues`, +`steam_linux`); `crates/discovery/src/keyvalues.rs`
      (`KeyValueError`, `Value`, `Section`/`get_str`/`get_section`, `parse`, plus
      the two `pub` caps); `crates/discovery/src/steam_linux.rs`
      (`SteamLinuxPlugin::from_host`/`from_roots`, `host_candidates`,
      `libraries_in_root`, `AppManifest`, `parse_manifest`, private scan/skip
      helpers); `apps/discovery-cli/src/main.rs` (`--steam-root`, `DummyPlugin`
      deleted).
- **Last Validation Result**: no toolchain leg was run in this stage — the
  documentation stage's shell rights exclude `cargo`/`npm`. The code at HEAD is
  unchanged since c9fb37b7, and the runner-captured gate for that tree
  (2026-11-18T04:41:08.659Z) was **green on all four legs**: `npm run check`,
  `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --
  -D warnings` and `cargo test --workspace --locked`, each exit 0. The
  correctness and compatibility seats independently re-ran `cargo test
  --workspace --locked` and `cargo test -p discovery-cli --test cli_tests
  --locked` at HEAD: 0 failures in every leg (three CLI tests pass). See *Gate
  Posture*.
- **Next Concrete Action**: none for the implementation — round 1 converged with
  every seat approving and no blocker or major open. The runner advances the
  flow; the open items in *Remaining Work* are either test-coverage additions,
  advisory hardening, document-owner edits, or the rollout step 5 human check.

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
   was routed in *Test Contract Conflicts* and re-pinned by `c9fb37b7` — closed.
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
   pre-existing assertions disputed before fba0321c. See *Test Contract Conflicts*; this is not a change of
   design, it is the design.
8. **Out-of-scope gate repair (the gate-repair round; the r1 seats left it as
    approved)**: the red `npm run check` came from
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
10. **Second fixture tree** (§9 draws one tree, `steam/minimal/`): the checked-in
    fixture set is two trees — `crates/discovery/tests/fixtures/steam/minimal/`
    exactly as §9 draws it, decoys included, plus
    `crates/discovery/tests/fixtures/steam/all-broken/` (two truncated manifests,
    no `libraryfolders.vdf`). *Why:* Test strategy #7 needs "a root that has a
    `steamapps` directory whose entries are all unparseable" and #3 needs a root
    whose VDF is absent, and §9 forbids test filesystem writes and took no
    dev-dependency, so a read-only on-disk tree is the only way to express either.
    `all-broken` is the fixture #3's "VDF absent" case reuses.
11. **"Empty" is trimmed** (§5 says "absent, empty"): `parse_manifest` trims
    before deciding, so a whitespace-only `name` falls back to `installdir` and a
    whitespace-only `installdir` is rejected. Same skip decision §5 states, just
    not reached by a literal byte-for-byte emptiness test.
12. **The 1 MiB cap is enforced at two points** (§4 states one cap): `parse`
    refuses an over-cap `&str` as a positioned error at line 1, and
    `read_text_file` refuses on `std::fs::metadata` length before the contents are
    allocated (deviation 5's placement). The second gate is what keeps a huge file
    from being read into memory at all; the first keeps the pure parser's own
    bound testable without a filesystem.
13. **`get_str` / `get_section` are type-exclusive** (§1 lists them without saying
    what a type mismatch returns): a key whose value is a section has no string
    value and vice versa, so `get_str("apps")` on a `libraryfolders` block is
    `None`, not a coercion. Pinned by
    `keyvalues_tests.rs::parses_quoted_and_bare_tokens_in_declaration_order`.

14. **Snap base requires an absolute `$SNAP_USER_DATA`** (§2 says "when that
    variable is set"): a set-but-relative value is dropped and the
    `$HOME/snap/steam/common` base stands, through the same `absolute_env` helper
    that guards `$XDG_DATA_HOME`. *Why:* A9's "a relative value is invalid" rule
    applied uniformly to both variables; §2's ordered candidate list is otherwise
    reproduced exactly, which `host_candidates_snap_user_data_replaces_snap_trio_base`
    pins for the absolute case.

*As-built mirror:* every item above is mirrored, grouped, in `02-proposal.md` →
*As-built* (2026-11-17). The two
document-owner edits they imply — §5's list plus Risks row 3 for the NUL clause,
§7's noun for the skip counter — were deliberately not made from this stage.

## Test Contract Conflicts (why `testsNeedUpdate` is true)

**None open — this section is closed as of the test-update round (c9fb37b7) and
is kept as the run record.**

- ~~`apps/discovery-cli/tests/cli_tests.rs:121` (`test_cli_list_json_format`)~~ —
  **RESOLVED by the routed test update.** The assertion had been written against
  the unauthorized `#[serde(rename_all = "kebab-case")]` attribute, asserting
  `obj["platform"] == "steam-linux"`; the attribute was reverted as Deviations 1,
  so the serialized form is `main`'s and the proposal's again (Context "no change
  to `Platform`", Non-goals, §8 "JSON shape unchanged") and the value is
  `"SteamLinux"`. The test now reads
  `assert_eq!(obj["platform"], "SteamLinux");` at `cli_tests.rs:124` — exactly the
  fix both the correctness (finding 1) and testing (F-1) seats prescribe, and the
  guard those seats asked for at the layer where the contract is user-visible.
  Everything else in the file passed as written and still does: order, slugs,
  display names, `install_path` composition, the five-field shape,
  `codec_id == null`, the table rows (the human table is unchanged because
  `Display` already printed the kebab form), and the `[]` case. The runner's green
  `cargo test --workspace --locked` leg covers it, as does the correctness seat's
  own `cargo test -p discovery-cli --test cli_tests --locked` (3 passed, 0 failed).
- The `crates/discovery/tests/game_tests.rs` pin suggested while Deviations 1 was
  open (assert `serde_json::to_value(Platform::SteamLinux) == "SteamLinux"` at the
  data-model layer) was **withdrawn by the testing seat at r1c2**: with the derive
  attribute gone there is no crate-level serde attribute left to pin, `Platform`'s
  serde behavior is unchanged code outside this task's test strategy, and the CLI
  per-row assertion is the guard. Nothing was added to `game_tests.rs`.
- The testing seat's carried additive items F-2..F-8 were **not** treated as
  conflicts by the r1c2 seat, which approves with them open: they add assertions
  rather than change any existing expectation. They stand as non-blocking
  test-coverage additions — §2 cross-root de-duping (F-2), the `(slug,
  install_path)` tie-break / duplicate-slug row (F-3), the §6 accepted-cost Proton
  row (F-4), the 1 MiB cap on both legs (F-5, with the seat's correction that the
  two constants already share one definition and cannot drift), the absolute-`path`
  VDF branch via `env!("CARGO_TARGET_TMPDIR")` (F-6), the NUL-`installdir` refusal
  case (F-7), and a §7 skip-count plus message-tail assertion cited to §7's own
  "file(s)" wording (F-8). See *Remaining Work*.
- Historical: the earlier secondary-library `install_path` conflict recorded
  here was resolved by the test engineer in fba0321c; those assertions pass
  as written today.

## Gate Posture

**This round (as-built reconciliation, documentation-only stage).** This stage's
rights allow no toolchain leg: the `cargo` and `npm` invocations were refused by
the stage sandbox, so nothing was re-run and nothing is claimed beyond what was
captured elsewhere. The tree this stage documents is unchanged in code since
`c9fb37b7` (the only later commits touch `docs/designs/**`), and the
runner-captured gate for that tree — 2026-11-18T04:41:08.659Z, quoted by the
compatibility, testing, security and performance seats as their gate evidence — was
**green on all four legs**: `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings` and
`cargo test --workspace --locked`, each exit 0. Two seats additionally ran
`cargo test --workspace --locked` and `cargo test -p discovery-cli --test cli_tests
--locked` themselves at HEAD and report 0 failures in every leg (the CLI leg:
3 passed). The one `test result: ok. 0 passed` line inside that capture is a
zero-test doc-test leg, not the suite total, as the correctness seat notes. Code
review round 1 is therefore converged: correctness r1c2, compatibility r1c2 and
testing r1c2 approve (0 blocker, 0 major), and security r1c3 and performance r1c3
ratify with the same posture, carrying only minors and nits (*Remaining Work*).

**Prior round (historical; code-review r1 dissent-c1 fix stage).** After that stage's
reverts (*Deviations* 1 and 9), the legs were run in that stage's own shell:

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

  That single routed assertion was updated by the test-update round in `c9fb37b7`,
  and the same leg is green in the runner's capture quoted in this round's block
  above; no other leg was ever red on that tree.

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
