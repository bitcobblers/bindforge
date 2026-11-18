# Code review — testing seat — round r1, pass c5 (ratification)

Design: `docs/designs/024-game-slug-identity/02-proposal.md` (ADR-024)
Scope (given): `git diff 73ef6ca37c8e8205a51f37586425e971db9a4fd9..HEAD -- ':(exclude)docs/designs'`,
read against the full current proposal. Pass type: ratification — the later
fixes are checked only for damage to what this seat approved in round 1.

## What the increment contains

Six files, all on the `crates/core` + corpus side:

| Path | Change |
|---|---|
| `crates/core/src/placement.rs` | doc-comment reversal (geometry name is now a full rider) + the new `name.len() > MAX_SLUG_OCTETS` arm in `load_geometry`, before the join |
| `crates/core/tests/placement.rs` | `a_geometry_name_past_the_shared_slug_bound_loads_as_known_current_behaviour` re-pinned as `a_geometry_name_past_the_shared_slug_bound_is_refused_beside_the_grammar` |
| `crates/core/tests/ledger_annotations.rs` | new suite (4 tests) — Test strategy #13(a)/(b)/(c) |
| `crates/core/tests/policy.rs` | doc-comment only (the removed scope-narrowing sentence → the rider-set sentence) |
| `docs/adrs/013-game-manifests.md` | `amended-by 024-B1 (ADR-024, per 021-L4).` on the `013-L5` unit |
| `schemas/README.md` | bound bullet re-states `024-B1`'s full rider set, narrowing clause deleted |

`apps/discovery-cli/**` and `crates/discovery/**` are untouched by this range:
byte-identical to what this seat approved, so both of my round-1 nits stand
exactly as written and are re-recorded below unchanged.

## Ratification result

Nothing this seat approved was loosened:

- The **only pin removed** anywhere in the increment is
  `a_geometry_name_past_the_shared_slug_bound_loads_as_known_current_behaviour`
  — the exact pin Test strategy #14 orders re-pinned ("re-pins the case that
  today asserts a past-bound geometry name *loads*"). Its replacement is
  strictly stronger: refusal at `MAX_SLUG_OCTETS + 1` as `InvalidValue`, no
  path, no document content, one printable line, **plus** a new positive leg
  that a name at exactly 128 still loads (`crates/core/tests/placement.rs:376-395`),
  which is the boundary half the deleted test never stated.
- Every other discovery/CLI/core pin from round 1 is present and unchanged, and
  the two new strategy items the escalation repair added are both cited by real
  tests: #13 by `crates/core/tests/ledger_annotations.rs:4, 50, 72, 113, 129`,
  #14 by `crates/core/tests/placement.rs:323`.
- The revision-round items (checklist 3) now have gate teeth: the ledger
  annotation is asserted against the corpus (`ledger_annotations.rs:56-70`), the
  spec-literal discipline against the four `spec.md` files
  (`:79-111`, `:118-127`), the README's one stated home and rider set against
  drift (`:135-151`), and the geometry rider against behaviour (`placement.rs:335`).
  None of these existed as a gate when I approved the earlier shape.
- The new arm is not dead code (checklist 7): it is the branch the re-pinned
  test drives, and the shipped grammar path cannot reach it (a name with a
  separator or control character is grammar-refused first), so the raw echo in
  `format!("a geometry name is at most {MAX_SLUG_OCTETS} octets: `{name}`")`
  is bounded by the grammar check ahead of it.
- Determinism (checklist 4) holds: the new suite reads checked-in corpus files
  through `common::repo_root()` (`crates/core/tests/common/mod.rs:50-57`,
  `CARGO_MANIFEST_DIR`-anchored) and `common::fixture_text` (`:171-173`, panics
  on a missing file, so no vacuous pass), writes nothing, no clock, no network,
  no ports. Test names (checklist 5) state the claim, not the steps.
- Traceability of the new tests (checklist 1) is better than the branch average:
  module header plus a `proves:` line on each of the four tests, and each names
  the strategy sub-clause it pins.

## Gate evidence

Supplied by the runner (at `2026-11-18T21:37:49.945Z`, green):
`npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked` — all exit 0.

Executed by this seat at HEAD, to confirm the two new/changed legs really run
(the supplied evidence shows only one `test result:` line):

- `cargo test --workspace --locked --test ledger_annotations` → `test result:
  ok. 4 passed; 0 failed` (all four #13 legs by name).
- `cargo test --workspace --locked --test placement` → `test result:
  ok. 21 passed; 0 failed`, including
  `a_geometry_name_past_the_shared_slug_bound_is_refused_beside_the_grammar`.
- `cargo test --workspace --locked` (full) → 42 legs print `test result: ok`,
  0 legs print `FAILED`.
- `cargo fmt --all -- --check` → clean.

No required gate leg failed; nothing needed for the verdict is missing evidence.

## Testing checklist

1. Strategy coverage — met for #1–#14 (each item has at least one `proves:`
   cite; #13/#14 added by this increment). Findings 2 and 3 below are about
   *strength* of two of those cites, not absence.
2. Unit + integration layers as promised — met; unchanged here.
3. Previously rejected items now have a test that would have caught them — met
   (geometry pin reversed to the refusal; ledger + spec literals gained teeth).
4. Deterministic — met.
5. Behaviour-named tests — met.
6. Gate evidence — met (supplied green, independently re-run green).
7. No untestable additions — met (the new `load_geometry` arm is driven).

## Findings

### 1. minor — `crates/core/tests/ledger_annotations.rs:205-213` (with `:102-109`, `:187-203`)

**Problem.** The #13(b) "no bound sentence carries a literal `63` or `128`"
check is keyword-gated: a chunk counts as a bound sentence only if it contains
`MAX_SLUG_OCTETS`, `shared bound` or `slug bound` (`is_bound_sentence`). A bound
statement phrased without any of those — the realistic regression this gate
exists to catch, e.g. a re-added row cell "at most 128 octets" beside a slug
grammar mention — is never classified, so the literal-number assertion never
sees it and the gate stays green. The chunker at `:187-203` widens both error
directions over markdown tables: it rejoins all whitespace and cuts only at
". ", so consecutive table rows (which end in `|`, not a period) merge into one
chunk — today that happens to keep the `id`/`codec` rows' bound mentions clean
of the `MAX_NATIVE_NAME_OCTETS`/`generated-by` 128 rows, but any later row edit
that shifts a break can merge another budget's literal into a bound chunk
(spurious failure) or move a restated bound number out of one (silent pass).

**Fix.** Widen the classification and make the negative half not depend on it:
also treat a chunk as being about this bound when it pairs a slug-grammar or id
mention with an `octets` cap phrase (`is at most …octets`, `past …octets`), or
— simpler and stronger — assert additionally that no chunk in the four
`spec.md` files contains a standalone `63` or `128` token unless that chunk also
names one of the other budgets' constants (`MAX_NATIVE_NAME_OCTETS`,
`MAX_GENERATED_BY_OCTETS`, `MAX_POLICY_TRANSFORMS`, `MAX_RELOCATE_TARGETS`).
Splitting the chunker on table-row boundaries (`| … |` as a terminator) removes
the over-inclusive half.

### 2. nit — `crates/core/tests/ledger_annotations.rs:93-101`

**Problem.** The cross-reference half is existential (`bound.iter().any(...)`),
while the design's acceptance for #13(b) reads "each bound sentence names
`MAX_SLUG_OCTETS` and points at `schemas/README.md`". By this file's own
predicate the shipped corpus already contains bound sentences that satisfy
neither — `schemas/action-ids/v1/spec.md`'s "`segment-past-cap.toml`'s segment
one octet past the shared bound …" chunk is classified as a bound sentence and
names neither token — so an `all` reading of the design sentence fails today and
the gate is per-file existential. The gap is small (one un-cross-referenced
sentence per file passes), but the test's `proves:` line cites the design's
"each" wording, so the cite overstates what is asserted.

**Fix.** Either state the intended quantifier in the `proves:` line ("each of
the four states the bound in *a* sentence that cross-references the README
bullet"), or restrict the universal check to chunks that state a cap and leave
the prose mentions out of `is_bound_sentence`.

### 3. nit — `crates/core/tests/placement.rs:335-395`

**Problem.** The re-pinned geometry test pins the refusal's *variant* and its
negative properties (no path, no document content, one printable line) but never
asserts that the message names the bound it broke, i.e. interpolates the
exported constant. Every other numeric-bound refusal in core pins that:
`crates/core/tests/policy.rs:907-912`, `crates/core/tests/action.rs:251-259`,
`crates/core/tests/slug_bound_corpus.rs:144-150`. `load_geometry`'s new arm
deliberately mirrors `load_policy`'s message shape (the code comment at
`crates/core/src/placement.rs:169-173` says so), and that deliberate shape is
the one property of it no test holds.

**Fix.** Mirror the policy leg: one
`assert!(detail.contains(&bindforge_core::MAX_SLUG_OCTETS.to_string()), …)`
beside the existing negative assertions.

### 4. nit (carried from r1, unchanged by this increment) — `apps/discovery-cli/src/main.rs:206-313`, `crates/discovery/src/steam_common.rs:430-508`

**Problem.** Unchanged: the two in-source `mod tests` — the only pins of Test
strategy #9's filter half and #4's counter clause — still carry no
`// proves: design 024's Test strategy #N` marker at the module or its tests
(verified: no `proves:` occurrence in either file); the citations live only in
the neighbouring suites' headers (`apps/discovery-cli/tests/cli_slug_refusal_tests.rs:5`,
`crates/discovery/tests/collision_tests.rs:4`).

**Fix.** One marker per test (or per module) in the wording the suites use, e.g.
`// proves: design 024's Test strategy #9 (Chosen design: CLI boundary) — the
emit filter drops the row and answers one named, escaped line`, and
`// proves: design 024's Test strategy #4 (collision pass) — skipped grows by
the refused row count`.

### 5. nit (carried from r1, unchanged by this increment) — `apps/discovery-cli/src/main.rs:81-84` vs `:115-129`

**Problem.** Unchanged: nothing pins that `main`'s list path calls
`refuse_invalid_slugs` — deleting the call site leaves every suite green,
because the shipped plugins refuse bad rows one seam earlier
(`crates/discovery/src/steam_common.rs:245-247`) so the subprocess tests never
reach it. The known residual of the function-level + end-to-end split this seat
prescribed in r1; recorded as a nit, not reopened.

**Fix.** If a later round touches this seam, give `main` a test-reachable seam
for the emit step (a `#[cfg(test)] pub(crate) fn list_games(…) ->
(Vec<DiscoveredGame>, Vec<String>)` that `main` and an in-source test both
call). No change required now.

## FYI (no severity)

`schemas/README.md:143`'s "exports all twelve constants" claim has no count pin
against `crates/core/src/lib.rs`'s export list; it is outside this increment's
changed lines and outside the proposal's Test strategy list, so it is noted, not
raised.

## Verdict

`approve` — zero open blocker or major findings. The post-approval revisions
reversed one pin only where Test strategy #14 demands it, replaced it with a
stronger one, and added the two missing gate legs (#13, #14) with per-test
citations; round 1's approved surface is intact and the full workspace suite and
fmt are green at HEAD under this seat's own re-run.
