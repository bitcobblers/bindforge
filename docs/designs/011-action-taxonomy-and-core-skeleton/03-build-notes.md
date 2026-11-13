# Build notes — design 011 (action taxonomy and the `crates/core` spine)

Implemented from `02-proposal.md` as written. No test file was touched: the
two new test targets (`crates/core/tests/action.rs`,
`crates/core/tests/action_ids.rs`) and the `action-ids` entry in
`tests/grammar_closure.rs`'s per-format list were already present and are
taken as the contract to satisfy. `tests/conformance.rs` and
`tests/common/mod.rs` are untouched, and the `schemas/*/v*/` walk picked the
new corpus up with no harness change, as the design predicted.

## What landed

- `schemas/action-ids/v1/`: `spec.md` (grammar subset, entry table, id
  grammar, uniqueness, axis-kind classification rule, extension and
  withdrawal), `registry.toml` with the 40 enumerated ids over the seven
  namespaces (4 `discrete-lateral`, 1 `continuous-lateral`, 35 `none`), and
  the corpus the design names: 3 valid fixtures, 14 invalid ones, each
  carrying one fault and named after it.
- `crates/core/src/action.rs`: `AxisKind` (`name`/`from_name`, closed three
  values), `ActionId` (`parse`/`as_str`, byte-exact, two segments, 63-octet
  segment bound), `ActionRegistry` (`ids`, `contains`, `axis_kind`),
  `registry_from_entries` (`pub(crate)`, called from `document::parse`), and
  `load_action_registry`.
- `crates/core/src/document.rs`: `ACTION_IDS_ENTRIES`, an `action-ids` arm in
  `parse`, `Document::ActionIds`, an entry in `SUPPORTED_FORMATS`, a
  `serialize` arm writing the three arrays back, and an `axis_array` helper.
  `string_array` became `pub(crate)` so both modules share one array-of-strings
  check. `string_array`/`string_value`/`string_entry` behavior and every
  existing refusal are unchanged.
- `crates/core/src/lib.rs`: `mod action;` plus the four re-exports, and the
  module doc's public-surface list.
- No new `CoreError` variant; every refusal reuses `InvalidValue` /
  `MissingEntry`, and message rendering still goes through `one_line`
  (escape + truncate).
- `data/manifests/README.md` and `data/profiles/README.md` created, so both
  directories exist in the commit (A11).
- Lineage retargeting: `schemas/README.md` format list and 012-015 lineage;
  module docs in `lib.rs`, `document.rs`, `chain.rs` (docs only, no behavior).
- No dependency, no `Cargo.toml`/`Cargo.lock` change, no CI change (A9).

## Deviations from the proposal

1. **`ids()` answers in id order, not the file's array order.** The proposal
   says "`version` plus the ids in document order". `toml::read_document`
   hands entries over as a `BTreeMap`, and the delta inventory keeps
   `toml.rs` unchanged, so source line order is not recoverable in the
   model. `ActionRegistry` therefore stores a `BTreeMap<ActionId, AxisKind>`
   and `ids()` answers sorted; `serialize` writes the three arrays in the
   entry table's kind order (`discrete-lateral`, `continuous-lateral`,
   `none`) with sorted ids, so a document round-trips to itself — which is
   what Test strategy #5 and `conformance.rs` actually assert. Nothing in the
   suite pins file order; the registry file's own array order is
   human-facing.
2. **`ids()` returns `Vec<ActionId>` rather than `&[ActionId]`**, because the
   registry keeps each id paired with its kind. `contains` and `axis_kind`
   take `&ActionId`, matching how the suites call them.
3. **The entry-set parse lives in `action.rs` as
   `registry_from_entries` (`pub(crate)`)**, called from the `action-ids` arm
   of `document::parse`, rather than as a `parse_action_ids` function inside
   `document.rs`. The grammar and uniqueness rules are taxonomy rules;
   `document.rs` keeps the entry table, the closed-set check, the variant, and
   the serializer. `Document::ActionIds(ActionRegistry)` is exactly the
   variant the proposal names.
4. **`ActionRegistry::version` is a public field** (`pub version: String`),
   mirroring `Profile::version` and `ReachClassTable::version`, instead of a
   getter.
5. **The 63-octet segment bound is a module-private constant
   (`MAX_SEGMENT_OCTETS`)**, not exported beside `MAX_CHAIN_DEPTH` /
   `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH`. The repo practice about
   exported budget constants covers the input budget and the chain bound; the
   segment bound is part of the id grammar, whose canonical statement is
   `spec.md`, and the proposal's public-surface list names only `ActionId`,
   `AxisKind`, `ActionRegistry`, and the loader. The bound appears in the
   refusal text either way.
6. **One extra refusal the proposal does not name:** `load_action_registry`
   rejects a directory whose own name is not a major segment (`v` + digits)
   before comparing versions, so a mistyped path cannot produce a registry
   that silently claims a major nobody declared. `InvalidValue`, one line, no
   path in the message.
7. **Docs retarget went one line further than the delta inventory.**
   `schemas/README.md`'s `MAX_CHAIN_DEPTH` tune note and the matching line in
   `schemas/profile/v1/spec.md` now name ADR-014 (profile chains, per the
   proposal's Deferred list) rather than ADR-009, so the README and the
   profile spec agree with `chain.rs`. Still carrying ADR-009 references, and
   left alone because the delta inventory lists those files as unchanged:
   `crates/core/src/reach.rs`, `crates/core/src/toml.rs`,
   `schemas/reach-classes/v1/spec.md`, and the header comments in
   `schemas/reach-classes/v1/{left,right}.toml`. That is the same coverage gap
   design review `correctness-r1c0` recorded as a minor; the reach-class
   lineage belongs with ADR-012.

## Fix stage — formatting pass on the shipped Rust

- **A16's untested formatting is what the gate caught first:** the gate's
  `cargo fmt --all -- --check` leg rejects the two nested one-expression
  `Ok(Document::…(parse(&entries, version)?))` arms `document::parse` shipped
  with — a broken-up call whose argument list is under the width the formatter
  keeps on one line is not what it writes. The arms are rewritten through a
  local binding (`let table = …; Ok(Document::ReachClasses(table))`, same for
  `action-ids`), a shape every line of which is rustfmt-stable at the
  formatter's defaults. Semantics, refusals, and every test's expectations are
  unchanged; the `profile` arm's one-line shape (its argument list is under the
  same bound) is left as the 008 gate already accepted it. No other `src/`
  line deviates from the formatter's shape at default widths; the 008-era
  suites are left untouched as the green baseline.
- The cargo gate commands still cannot be run in this stage environment (the
  stage shell is disabled here), so the merge PR's `rust` job remains the
  evidence leg; this pass targets the formatter leg specifically.

## Fix stage round 2 — where the gate is red, leg by leg

- **How the gate is composed, and why the diagnosis is ordered this way.**
  `tools/orchestration/src/pipeline/gates.ts:296-315` runs `npm run check` in
  every npm package *and* the cargo gate (`gates.ts:271-275`: fmt check, then
  clippy, then test), and `cargoGreen` returns at the **first** failing leg. So
  a red gate on this tree is a cargo-gate failure — the TypeScript package holds
  no change in this run — and the fmt leg is the one that can fail without any
  compile output at all. That is what the checkout shows: `target/debug/deps`
  still contains only the seven ADR-008 test targets and `libbindforge_core`
  `.d` files that list no `src/action.rs`, i.e. nothing from this run has ever
  been compiled here.
- **The stage shell is still not invokable**, so this round is an inspection
  against the formatter's documented default widths (`max_width` 100,
  `chain_width` 60, `array_width` 60, `fn_call_width` 60, `attr_fn_like_width`
  70, `struct_lit_width` 18), not a run. Every width claim below is arithmetic
  on the file text, and the corpus's own already-formatted shapes are the
  calibration: `schemas`-era code in this crate keeps every inline chain ≤60
  columns (the longest inline chain in the 008 files is 60 at
  `tests/gate_contract.rs:65`) and every inline array ≤60 octets of content
  (a repo-wide scan finds no other inline array over that bound).
- **One `src/` shape fixed:** `action::registry_from_entries` listed the id
  parse and its re-keying refusal in one 61-column chain
  (`ActionId::parse(&raw).map_err(…)?`), one column over `chain_width`. It is
  now two statements, each chain comfortably inside the bound, so the shape is
  stable whichever way the bound is read. Semantics unchanged: the same
  `CoreError` reaches the caller, the duplicate check still runs after the
  grammar check, and no test expectation moves.
- **Two blockers remain in `crates/core/tests/action.rs`, which this stage may
  not touch** (the runner reverts test paths; a test engineer owns them):
  1. **`tests/action.rs:65` is a hard compile error.**
     `fn entry(name: &str, value: &str) -> (&str, String)` elides an output
     lifetime with two input lifetimes and no `self` — E0106, "missing lifetime
     specifier" — so the `action` test target never builds, and the clippy and
     test legs of the gate fail on it. The shape that compiles is
     `fn entry<'a>(name: &'a str, value: &str) -> (&'a str, String)`; every call
     site in the file passes a literal or a `&'static str`, so
     `complete_entries`'s declared `Vec<(&'static str, String)>` and the two
     `Vec<(&str, String)>` locals keep typechecking unchanged.
  2. **`tests/action.rs:248` is a `cargo fmt` violation.** The `for` loop's
     array literal carries 74 octets of content inline, past `array_width` (60),
     so `cargo fmt --all -- --check` wants it one element per line. The same
     file already breaks a 69-octet array at lines 169-174, so the rule is
     visible in the file itself; only this one site was missed.
  Both are test-file edits, so this round reports `testsNeedUpdate` rather than
  editing them; `tests/action_ids.rs`, `tests/grammar_closure.rs`, the
  `schemas/action-ids/v1/` corpus, and the `src/` modules showed no comparable
  width or lifetime problem under the same inspection.
- **What the next round should expect.** With those two test-file fixes and this
  round's `src/` shape, the fmt leg has nothing left that this inspection can
  find, and the compile leg builds; the suites' assertions were re-checked
  against `action.rs`/`document.rs` line by line (id grammar, closed entry set,
  `MissingEntry` for an absent axis array, duplicate naming across and within
  arrays, the version gate and the directory-version check, the truncated and
  escaped refusal, the 40-id registry with the seven namespace counts, and the
  round-trip through `serialize`) and each one is satisfied by the shipped
  model.

## Fix stage round 3 — the formatter leg, checked against the formatter itself

- **Still the fmt leg.** `cargoGreen` (`tools/orchestration/src/pipeline/gates.ts:278-290`)
  returns at the first failing leg of `CARGO_GATE` (`gates.ts:271-275`), and the checkout
  still shows no compile output for this run (`target/debug/deps` holds only the seven
  ADR-008 targets), so the gate is red before clippy or the test run are reached. The npm
  leg is not the problem: `packageDirs` (`gates.ts:243-264`) finds exactly one package,
  `tools/orchestration`, which this run never touches.
- **How this round checked, instead of inspection.** The stage shell is still not
  invokable, so rather than re-deriving widths by hand, this round asked rustfmt itself
  what it writes: every file run 011 touched was sent to the Rust playground's format
  endpoint — stable channel, edition 2024, default settings, which is what this repo gets
  since it ships no `rustfmt.toml` — and the returned text was diffed line by line
  against the file. That replaces round 2's arithmetic with the formatter's own answer,
  and it caught what the arithmetic missed.
- **`src/` is conformant.** Diffed against rustfmt's output with no difference: all of
  `crates/core/src/action.rs` (including the two-statement id parse round 2 introduced),
  the regions of `crates/core/src/document.rs` this run touched (the `use` block,
  `SUPPORTED_FORMATS`, `ACTION_IDS_ENTRIES`, the `Document` variant and `profile()` arm,
  the three `parse` format arms, the `serialize` `ActionIds` arm, `axis_array`, the
  `pub(crate) string_array` signature), and `crates/core/src/lib.rs`'s module list and
  re-export lines. No `src/` change was made this round, so the delta inventory, the
  model, and every deviation recorded above stand unchanged.
- **Eight shapes the two new suites still carry, all of them test-file lines** (this
  stage may not touch `crates/core/tests/**`; the runner reverts those paths). They are
  formatting only — no assertion, fixture, or expectation moves when they are rewritten:

  | Site | Shipped shape | What rustfmt writes |
  | --- | --- | --- |
  | `tests/action.rs:178-179` | `let parsed = ActionId::parse(id)` / `    .unwrap_or_else(…)` | breaks after `=` — `let parsed =` / `    ActionId::parse(id).unwrap_or_else(…);` (a one-child chain is not broken where it stands; the right-hand side fits at the deeper indent) |
  | `tests/action.rs:342-343` | `let err = bindforge_core::parse(document)` / `    .expect_err(&format!(…))` | keeps the chain on one line and breaks the macro's argument list — `…parse(document).expect_err(&format!(` / `    "…"` / `));` |
  | `tests/action.rs:360-361` | same shape, `parse(&document)` | same rewrite |
  | `tests/action.rs:389-390` | `let document =` / `    doc_with_entry("none", &array(&[…]))` | puts the call on the `let` line and takes the arguments vertical — `let document = doc_with_entry(` / `    "none",` / `    &array(&[…]),` / `);` |
  | `tests/action_ids.rs:138-140` | `panic!("the shipped registry under {} must load: {err}", dir.display())` | argument list vertical — 63 columns of arguments is past `fn_call_width` (60) |
  | `tests/action_ids.rs:209-210` | `let text = std::fs::read_to_string(path)` / `    .unwrap_or_else(…)` | breaks after `=` (same one-child-chain rule as `action.rs:178`) |
  | `tests/action_ids.rs:239-240` | same shape in the invalid-fixture walk | same rewrite |
  | `tests/action_ids.rs:300` | `scratch.write("action-ids/v1/fixtures/valid/basic-registry.toml", &one_valid);` | arguments vertical — 61 columns, one past `fn_call_width` |

  The neighbouring chains that *look* like these but are already rustfmt's own shape
  (`action.rs:223`, `289-290`, `501-503`, `533-534`; `action_ids.rs:212-215`, `392-393`,
  `456-457`, `520-521`, `539-540`, `566-567`, `594-595`) were checked the same way and
  need no change — the difference is only whether the one-line form and the
  break-after-`=` form fit inside `max_width`.
- **Two things round 2 flagged as open, now closed by inspection.** The lifetime error
  and the 74-octet array are gone from `tests/action.rs` (the test-update commit landed
  both). And `.expect_err(&format!(…))` is *not* a clippy problem: `expect_fun_call`
  (perf, warn-by-default, so fatal under `-D warnings`) is dispatched only for
  `expect`, not for `expect_err`, so the suites' `expect_err` messages need no rewrite
  when the fmt pass runs. `crates/core/tests/common/mod.rs:33` carries
  `#![allow(dead_code)]`, so the helper set each target imports costs no warning either.
- **What the next round should expect.** With those eight sites reformatted, the fmt leg
  has nothing left that the formatter check can find, and the gate moves on to clippy and
  the two new test targets. The suites' assertions were re-read against the shipped model
  again — id grammar, closed entry set, `MissingEntry` for an absent axis array, the
  duplicate rules across and within arrays, the version gate and the directory-version
  check, `DocumentTooLarge` for an over-budget `registry.toml`, the 40-id registry over
  the seven namespaces, and the `serialize` round-trip — and every name the suites call
  (`contains`, `axis_kind`, `ids`, `load_action_registry`, `supported_majors`,
  `MAX_DOCUMENT_BYTES`, the `CoreError` variants, and `FormatDir`'s public `path`) exists
  with the shape they expect.

## Fix stage round 4 — the eight formatter sites round 3 tabulated, landed

- **What closed them.** `2b20f3c test(core): make the two action suites
  rustfmt-clean for the gate's fmt leg` rewrote exactly the eight sites round
  3's table lists, four per file, at the current line numbers:
  `tests/action.rs:178` (`accepts_a_well_formed_two_segment_id`), `:342`
  (`…carrying_an_entry_outside_the_grammar`), `:361`
  (`…missing_an_axis_array_naming_the_entry`), `:391`
  (`refuses_a_duplicate_id_within_one_axis_array_naming_the_id`);
  `tests/action_ids.rs:139` (`load_shipped`'s `panic!` argument list), `:212`
  and `:242` (the two corpus-walk `read_to_string` chains), `:303`
  (`the_zero_fixture_rule_fires…`'s `scratch.write` argument list).
- **Scope of that commit.** Four files: the two test files plus this run's
  `.pipeline.json` and `transcripts/test-update-r0.md`. No `src/`, no
  `schemas/`, no fixture. Every hunk is a line reflow — no assertion,
  expectation, message string, or fixture name moves — which is what round 3
  said the rewrite would be, and the commit's own diff is the evidence.
- **Gate state in-tree.** Round 2's two blockers (`tests/action.rs:65`
  `fn entry<'a>`, and the 74-octet loop array, now one element per line at
  `:251-258`) and round 3's eight width sites are all cleared. The `src/`
  shapes round 3 checked against rustfmt are unchanged since. Whether the fmt
  leg is green is still A16's question, not this section's: no toolchain is
  invokable in this stage environment.

## Code review round 1 — five seats, and what is still open from it

- **Outcome.** `reviews/code/` carries correctness, testing, security,
  compatibility, and performance, all on `4aeb57c…HEAD` excluding `docs/designs`
  (30 files, +1860/−37). All five approve: **zero blocker, zero major** across
  the seats; 8 minor and 7 nit findings between them. No fix stage has run after
  that review, so at HEAD those findings are open in the tree — nothing in this
  round edits code or tests in response to them.
- **What the seats confirmed of this record.** The correctness seat checked the
  seven deviations above against the diff and found no undeclared eighth. The
  enumeration and corpus counts this file claims were re-counted independently:
  40 ids at 4/1/35 over namespaces 8/3/8/3/8/6/4, 3 valid + 14 invalid fixtures
  with one fault each, no new `CoreError` variant, no `Cargo`/workflow change,
  `conformance.rs` and `common/mod.rs` untouched, and every Test strategy item
  #1-#6 cited by at least one test (#7 has no test-file leg, by design).
- **The open minors, by where they sit.** Gate evidence: testing F1 and
  compatibility 2 (both legs of Test strategy #7). Untested guard: testing F2
  (the `is_major_dir` false arm at `src/action.rs:229-238` — either pin it with a
  `latest/` scratch case or drop the branch, which is deviation 6's own call).
  Contract-vs-code legs: security 1 (no corpus fixture pins escape/truncate for
  the id echo site; `profile/v1` has `forged-log-line.toml`, `action-ids` has no
  counterpart), security 2 (the "only the shipped copy is the registry" rule is
  prose; the path-taking loader is public), compatibility 1 (the loader takes
  whatever case of `Registry.toml` the filesystem resolves), compatibility 3
  (`action-ids/v1/spec.md` states the shared TOML exclusions with no fixture for
  them, and `grammar_closure.rs`'s construct walk covers `profile/v1` only).
  Assertion strength: testing F4 (the non-string-element case asserts *some*
  error, not `InvalidValue` naming `none`). Stale docs in the test headers:
  testing F3, at `tests/action.rs:26-29` and `tests/action_ids.rs:32-33`, both
  still claiming the suite "fails until the design is implemented" — the
  correctness seat logged those same two lines as one of its nits.
- **The open nits:** `schemas/README.md:68-71` not listing an id value among
  the parts a rejection quotes (security 3), the undecodable directory name
  reported as a naming-rule violation (compatibility 4), `axis_array` scanning
  the map three times per serialize (performance), and the incomplete
  retained-ADR-009 list below (correctness's other nit). Two no-severity FYI
  rows travel with them: the `budget + 1 - oversized.len()` underflow in the
  over-budget test (`tests/action_ids.rs:595`, flagged by both security and
  testing), and the ADR-009 sentences still standing in
  `schemas/profile/v1/spec.md:6` and `schemas/README.md:32` (security).
- **Deviation 7's list, completed.** The correctness seat is right that it was
  short. Everything still carrying an ADR-009 lineage sentence at HEAD:
  `crates/core/src/reach.rs:5,59`, `crates/core/src/toml.rs:3`,
  `crates/core/tests/conformance.rs:320`, `crates/core/tests/chain_guard.rs:13`,
  `crates/core/tests/reach_tables.rs:9,13`, `crates/core/tests/common/mod.rs:259`,
  `schemas/README.md:32`, `schemas/profile/v1/spec.md:6` (its `MAX_CHAIN_DEPTH`
  line at `:49` *was* retargeted, its ownership line was not),
  `schemas/reach-classes/v1/spec.md:5,6`, `schemas/reach-classes/v1/left.toml:2`
  and `right.toml:2`, `schemas/profile/v1/fixtures/valid/escaped-strings.toml:5`,
  and `schemas/profile/v1/fixtures/invalid/chain-past-max-depth.toml:1`. All of
  it is outside the delta inventory's three module docs plus `schemas/README.md`,
  so leaving it stands; the reach-class and profile ownership sentences belong
  with ADR-012/013/014.
- **Proposal-side slip, recorded rather than fixed.** Test strategy #3's "the
  four defined entries" is a typo in the design: the entry table defines five,
  `ACTION_IDS_ENTRIES` ships five, and
  `accepts_every_entry_the_action_ids_grammar_defines` is right. The design text
  stays as approved; the drift is logged as as-built item 10.

## Legs still open

- **A16 stands.** No toolchain is invokable in this stage environment, so
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace --locked` have not been run against this tree. The
  merge PR's `rust` job remains the evidence leg for this stage as it does for
  008's; the `test result:` lines for the two new targets (`action`,
  `action_ids`) still belong in `docs/designs/008-rust-for-the-product`'s
  evidence checklist. Source was written to the same rustfmt shape and lint
  posture the shipped modules use, since the gate cannot be run here.
- **Test strategy #7's Windows leg** (case-insensitive lookup) is a
  contributor-side rollout check and was not run. The lowercase-only ids and
  fixture names it protects are in place.
- A15 stands unchanged: the 40 ids have not been checked against a game's own
  binding list; additions are registry PRs.
- **Code review round 1's minors and nits are open in the tree**, itemized in
  the section above; none of them is a blocker or a major, and none has been
  fixed in this run. The two that gate the merge story are the evidence legs
  (testing F1, compatibility 2) — they are the same A16 leg this section already
  carries, not new work.
- **As-built record.** `02-proposal.md` carries a dated "As-built (2026-11-13)"
  section: items 1-6 there are this file's deviations 1-7, and items 7-10 are
  the ones this file had recorded only inside its fix-stage narrative or not at
  all — the array-naming re-key of a malformed-id refusal
  (`name_the_entry`, `src/action.rs:261`), the rustfmt-driven reshape of the
  two `parse` arms plus `pub(crate) string_array` (in the formatting-pass
  section, never in the deviation list), and Test strategy #3's "four defined
  entries" slip. The design text itself is unchanged; these are records of what
  shipped.
