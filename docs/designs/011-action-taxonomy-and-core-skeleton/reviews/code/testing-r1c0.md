# Code review — testing seat, round 1 pass c0

Scope: `git diff 4aeb57c2188fbdfc1ca68d99080e4d2f3950d945...HEAD -- ':(exclude)docs/designs'`
(30 files: `crates/core/src/{action,document,lib,chain}.rs`,
`crates/core/tests/{action,action_ids,grammar_closure}.rs`, the
`schemas/action-ids/v1/` corpus + `spec.md`, `schemas/README.md`,
`schemas/profile/v1/spec.md`, two `data/**/README.md`).
Judged against `02-proposal.md` (Test strategy #1-#7) and ADR-011. First pass,
no prior findings to re-test.

## Checklist 6 — the gate leg (not evaluable in this seat)

The checklist asks me to run the package's check command and paste the tail.
The rust gate for this package is (`.github/workflows/check.yml:29-33`)
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo test --workspace --locked`. **This review seat's shell access
was denied** (every `shell` call returned `permission.rejected`), so I could
execute none of them and have no output to paste. This is the same limitation
the proposal already records as A16 ("No stage of the 008 run could invoke a
toolchain"), and the proposal's Risks row "The new suite never runs locally"
names the merge PR's `rust` job as the evidence leg. I therefore record item 6
as **not evaluated**, per the skill's rule that an item I could not evaluate is
a finding and not a pass — logged below as F1, not as a pass and not as a red
gate. The runner verifies it independently.

What I could do statically, and what it showed (no substitute for the run):

- Import/usage audit of both new test targets against `-D warnings`: every
  import (`ActionId`, `ActionRegistry`, `AxisKind`, `CoreError`, `BTreeMap`,
  `BTreeSet`, `PathBuf`, and all nine `common` helpers) is used; no test-local
  helper or const is dead. No obvious clippy trip (no needless lifetime —
  `tests/action.rs:65` needs its explicit `'a` because the output borrows one
  of two input lifetimes; no `result_large_err`-style pattern the default set
  flags).
- Fixture-name audit against `common::is_fixture_name` (`tests/common/mod.rs:352`)
  and the case-unique rule (`tests/conformance.rs:277`): all 3 valid + 14
  invalid names pass, and no two differ only by case — the Windows half of #7
  is not newly endangered by this corpus.
- Expected-result trace of each new assertion against the shipped `src/` (see
  "Cross-checks" below): I found no test whose expectation the shipped
  implementation contradicts, and no fixture the reader would accept.

## Checklist 1 — Test strategy coverage (each numbered item cited by a test)

| Item | Cited by | Verdict |
|---|---|---|
| #1 id grammar, one case per rule | `tests/action.rs:164` (`accepts_a_well_formed_two_segment_id`: `move.strafe-left`, `item.slot-6`, `aim.cycle-target`, 63-octet segment), `:189` (`rejects_an_id_that_breaks_the_segment_grammar`: all ten listed rejections incl. empty string and 64-octet) | covered |
| #2 axis kind total + pinned | `tests/action.rs:232`, `:266`; `tests/action_ids.rs:435` (one array per id), `:480` (`discrete-lateral` set == the four endpoints), `:510` (steering/camera/forward grading) | covered |
| #3 closed entry set | `tests/action.rs:316` (defined entries parse), `:329` (`emphasis`, fourth axis array, refusal names the entry), `:353` (`MissingEntry` per array), `:376` (non-string element), `:387` + `:405` (duplicate in one array / across arrays, names the id) | covered |
| #4 version gate for the new format | `tests/action.rs:493` (no version), `:511` (`v99`), `:528` (declared ≠ directory segment); `tests/action_ids.rs:322` (declared majors == majors under `schemas/*/v*/`) | covered |
| #5 harness unchanged, corpus exercised, zero-fixture rule, `grammar_closure` list | `tests/action_ids.rs:161`, `:195`, `:226`, `:264`; `tests/grammar_closure.rs:104` (format list now `["profile", "reach-classes", "action-ids"]`) | covered |
| #6 shipped registry loads: 40 ids, once each, seven namespace counts, missing/foreign `registry.toml` | `tests/action_ids.rs:355`, `:390`, `:411`, `:536`, `:560` (+ `:587` over-budget) | covered |
| #7 three gate commands green on Linux, once on Windows | no test-file leg; stated as such at `tests/action_ids.rs:20-26`, same treatment 008's `gate_contract.rs:4-17` gives its CI/platform legs | see FYI |

Independent count check: `schemas/action-ids/v1/registry.toml` carries 4 + 1 +
35 = 40 ids, and the namespace split is move 8, aim 3, combat 8, interact 3,
item 8, ui 6, system 4 — so `SHIPPED_IDS` (`tests/action_ids.rs:48`) and
`NAMESPACE_COUNTS` (`:96`) genuinely pin the proposal's enumeration rather than
being padded to whatever the file says. `INVALID_FIXTURES` (`:113`) names
exactly the 14 files the proposal's contract tree names, and each file exists.

## Checklist 2 — both layers

Unit layer `crates/core/tests/action.rs` (grammar, closed set, version gate,
scratch dirs) and integration layer `crates/core/tests/action_ids.rs` (shipped
registry, corpus walk, harness rules) both exist, matching the proposal's split
and Rollout 2. The two new targets need no `Cargo.toml` change (A8: each
`tests/*.rs` is its own target).

## Checklist 3 — revision rounds

Not applicable: no previously rejected blocker/major in this run to re-pin.

## Checklist 4 — determinism

No new test reads the wall clock, opens a socket, binds a port, or depends on
cwd: the contract tree is reached through `env!("CARGO_MANIFEST_DIR")`
(`tests/common/mod.rs:50`), scratch trees through `ScratchDir` (`:170`,
pid+counter+nonce, `Drop`-cleaned), and fixture iteration is sorted
(`read_dir_sorted`, `:329`). The `SystemTime` in `ScratchDir::new` is
pre-existing (`common/mod.rs` is untouched by this diff) and only names a
directory, never an assertion. `tests/action_ids.rs:587` derives its oversized
size from `bindforge_core::MAX_DOCUMENT_BYTES` rather than a literal, which is
the repo's stated habit. Deterministic: pass.

## Checklist 5 — names describe behavior

Every new test name states the behavior it pins
(`refuses_an_action_ids_document_missing_an_axis_array_naming_the_entry`,
`the_shipped_discrete_lateral_set_is_exactly_the_four_lateral_endpoints`,
`the_loader_refuses_a_version_directory_without_a_registry_file`), not a call
sequence. Pass.

## Checklist 7 — untestable additions and dead branches

Two additions in this diff have no consumer in the suite; both are findings
(F1/F2 below plus F4). The one genuinely unreachable code path in the new
`src/` is the `!is_major_dir(major)` arm of `load_action_registry`
(`crates/core/src/action.rs:233-238`), which no test reaches — F2.
`AxisKind::name`/`from_name`/`ALL`, `SEGMENT_*_RULE`, `invalid_id`,
`name_the_entry`, `is_well_formed_segment`, `is_major_dir`'s true arm,
`axis_array` (`document.rs:247`) are all exercised.

## Cross-checks that the new expectations match the shipped code

These are the traces I did by hand since I could not run the suite; each is
where a wrong expectation would have gone red:

- `move..steer` reaches the segment-count rule (via `split_once` + `second
  .contains('.')`, `action.rs:114-119`), and `move.` / `.forward` reach the
  segment-shape rule; `möve.forward` fails the first-byte test
  (`action.rs:280`), so all three rejections in `tests/action.rs:195` land on
  `InvalidValue` as asserted.
- `tests/action.rs:473`'s escape case really does exercise id-echo escaping:
  the repo's subset reader decodes `\uXXXX` (`src/toml.rs:230`, `:264`), so the
  BEL reaches the message and `one_line` (`src/error.rs:155,164`) escapes it.
  `tests/action.rs:450`'s truncation case lands inside `one_line`'s 80-char
  cut, so the 4 KiB tail is genuinely cut, not echoed.
- Duplicate detection: `BTreeMap::insert` returning `Some` (`action.rs:200`)
  catches both the in-array and the cross-array case, and the message names the
  id, as `tests/action.rs:387`/`:405` assert.
- Version-mismatch case: `load_action_registry` compares the loaded
  `registry.version` with the directory segment (`action.rs:229-245`), so the
  `v2`-directory case yields `InvalidValue`, one line, path-free — matching
  `tests/action.rs:528`.
- All three valid fixtures round-trip through `serialize`
  (`document.rs:222-239`) into a `BTreeMap`-equal registry, which is what
  `tests/conformance.rs:47` and `tests/action_ids.rs:195` both demand with no
  harness special case — the proposal's claim that this format round-trips
  "like `profile`" holds.

## Findings

### F1 — minor — checklist item 6 not evaluable in this seat
Location: `.github/workflows/check.yml:29-33` (gate), report-wide.
Problem: this seat could not run `cargo fmt --all -- --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, or `cargo test --workspace --locked`
(shell access denied), so the checklist's "run it and paste the tail" leg is
unverified here and this report carries no `test result:` tail. Per the skill,
an item I could not evaluate is a finding, not a pass. Nothing in the static
review suggests a red gate, but nothing here proves green either, and the
proposal's own Risk row ("The new suite never runs locally", likelihood High)
makes this the stage's real evidentiary gap.
Fix: attach the merge PR's `rust` job `test result:` lines for
`crates/core/tests/action.rs` and `crates/core/tests/action_ids.rs` (plus the
fmt/clippy legs) to the outstanding evidence checklist that
`docs/designs/008-rust-for-the-product/03-build-notes.md` carries, as Rollout 5
says, before merge; run the Windows leg of Test strategy #7 once.

### F2 — minor — an unreachable-in-practice branch ships untested
Location: `crates/core/src/action.rs:229-238` (`is_major_dir` false arm in
`load_action_registry`), untested by either new target.
Problem: every `load_action_registry` call in the suite passes a directory
named `v1` or `v2` (`tests/action.rs:140`, `:533`;
`tests/action_ids.rs:131`, `:536`, `:560`, `:592`), so the refusal for a
directory that is not a major (`latest/`, a bare `action-ids/`, a root path via
`unwrap_or_default()` → `""`) is a branch no test can fail. Checklist 7 counts
that as a finding, and it is the one guard on the loader that the version-mismatch
test does not cover: a caller pointing the loader at the format directory
instead of the major directory currently produces no assertion anywhere.
Fix: add one case to `tests/action.rs` — `ScratchDir` with
`registry.toml` inside a directory named `latest` (or the format directory
itself), `load_action_registry` must return `InvalidValue` naming the
major-directory rule, one line, no path — or delete the branch and rely on the
version comparison alone.

### F3 — minor — stale suite headers claim the tests fail
Location: `crates/core/tests/action.rs:26-29`; `crates/core/tests/action_ids.rs:32-33`.
Problem: both headers still say the suite is "written from the proposal while
`crates/core/src/action.rs` does not exist yet, so they fail until the design
is implemented" / "before the format ships, so they fail until it does". In
this commit `src/action.rs`, `src/document.rs`'s `action-ids` arm, and the
whole corpus exist, so the statement is false about the tree a reader is
looking at, and it contradicts the `proves:` map above. It is exactly the class
of stale lineage this proposal itself treats as a risk ("Stale ADR-009 lineage
misleads the next contributor"), and a future fixer reading it has an excuse to
dismiss a red run as expected.
Fix: rewrite both sentences to state what the suite pins now (e.g. "written
from the proposal, kept green against the shipped `src/action.rs`; no
assertion may be weakened to make a change pass"), keeping the
written-from-the-proposal lineage.

### F4 — minor — one Test strategy #3 refusal is asserted weaker than the contract
Location: `crates/core/tests/action.rs:376-385`
(`refuses_a_non_string_element_in_an_axis_array`).
Problem: the test asserts only that `parse` returns *some* error. Test strategy
#3 groups the non-string element with the duplicate case as "refused naming
the id/entry", and `schemas/action-ids/v1/spec.md:40-42` types each array as
"array of strings". As written, a reader that refused a `7` for a reason
unrelated to the array contract (`Malformed`, a budget error, an
`UnknownFormat`) passes, and the surrounding `refuses_a_*` tests in the same
file do pin the variant, so this one is an outlier rather than a deliberate
looseness.
Fix: assert `matches!(err, CoreError::InvalidValue { .. })` and that the
rendered message names the array (`none`), mirroring
`refuses_a_duplicate_id_within_one_axis_array_naming_the_id`.

### F5 — nit — two Rollout 3 obligations are pinned only by prose
Location: `data/manifests/README.md`, `data/profiles/README.md` (new),
against proposal Rollout 3 + A11.
Problem: the tree facts this stage promises — both `data/` directories exist in
the commit, and the third format's version directory is present with its
`spec.md` + `registry.toml` — are asserted for fixtures
(`tests/action_ids.rs:161`) but not for the `data/` placeholders or the two
non-fixture files. git tracks no empty directory (A11), so the placeholder
guarantee is exactly the kind of tree fact this repo pins elsewhere
(`tests/gate_contract.rs:517` pins workspace members for the same reason).
Fix: extend the `tests/action_ids.rs:161` walk assertion with
`spec.md`/`registry.toml` presence, and add one cheap `repo_root()`-relative
assertion that `data/manifests/README.md` and `data/profiles/README.md` exist.

## FYI (out of specialty, no severity)

- The proposal describes `ActionRegistry` as "`version` plus the ids in
  document order"; the model stores a `BTreeMap<ActionId, AxisKind>` so
  `ids()` is id-sorted (`src/action.rs:149-159`, acknowledged in its own doc
  comment), and nothing tests order either way. Semantics are unaffected
  (round-trip equality holds) but the deviation from the design's sentence
  belongs to the correctness/API seat. `ActionRegistry::version` is also a
  public mutable field with no consumer in this diff.
- `schemas/profile/v1/spec.md:6` ("owned by ADR-009") and
  `crates/core/src/toml.rs:3`, `src/reach.rs:5,59` still carry ADR-009
  lineage. They are outside the three module docs + `schemas/README.md` the
  proposal scoped for retargeting, so this matches the design, but a reader
  following the "stale lineage" risk may expect them too.
- Test strategy #7's platform legs have no test-file citation, only the
  explanatory header at `tests/action_ids.rs:20-26`; that mirrors how 008's
  `gate_contract.rs:4-17` and `conformance.rs:17-23` handle the same class of
  leg, so I read it as consistent rather than as a gap.
- `tests/action_ids.rs:587` (over-budget registry) is a `proves:` cite to
  "Chosen design (The model, loader) + Adversarial guard" rather than to a
  Test strategy item; it is a legitimate extra leg, not a coverage hole.

## Verdict

No blocker or major finding: the Test strategy's items #1-#6 are each pinned by
at least one citing test, both promised layers exist, the corpus matches the
proposal's named fixture tree one-fault-each, the suite is deterministic, and
every trace I could do by hand matches the shipped implementation. Approval is
conditioned on F1's evidence leg landing with the PR (the checklist's gate leg
is unverified in this seat) and on F2/F3 being cleaned up.

verdict: approve — 0 blocker, 0 major, 4 minor, 1 nit.
