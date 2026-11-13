# Code review — performance seat, ratification pass c2

- Range reviewed: `git diff 2682fb1f740770fdb588db1a3ff69d66e9d1f3d4..HEAD -- ':(exclude)docs/designs'`
  (4 commits: `192fb72` docs, `eba75fc` fix — gate per-format majors + round-1 findings, `9c1f601` test,
  `cc12e72` docs). Read against the full current `02-proposal.md` and
  `docs/adrs/012-input-vocabulary-and-reach-classes.md`.
- Question for this pass: did the post-approval revisions break what this seat approved in round 1?

**Verdict: approve.** No blocker or major finding. The delta adds no per-event work, no unbounded
structure, and no new allocation on the parse hot path beyond one membership test that was already
there; the four round-1 findings sit at byte-identical code and are neither fixed nor worsened.

## What the delta actually does to cost

The change under review is the per-format version gate (`format_majors` / `check_format_major`,
`document.rs:118-148`), one shared directory-segment rule (`major_dir_segment`, `document.rs:687-699`,
adopted at `document.rs:675` and `action.rs:229`), the ASCII clarification of the keyboard member set
(`input.rs:179-198`, comment and rule string only — the predicate body is unchanged and is O(1)), a
serializer arm change (`document.rs:360-369`), two new invalid fixtures, and test/doc updates.

Cost deltas measured by reading, all neutral-to-positive:

- `format_majors` returns `Option<&'static [&'static str]>` — a match over ≤5 string literals plus a
  `contains` over ≤2 `&str`. **No allocation**, and the `UnsupportedVersion` error's `map(...).collect`
  (`document.rs:146`) runs only on the refusal path. This is the static-list shape round 1 asked for;
  it is used for the per-format half.
- `major_dir_segment` replaces two copies of the same `file_name` → `strip_prefix` → byte scan with one
  borrowing helper. Same O(name length), no allocation; `check_dir_version` and `load_action_registry`
  each now run one scan instead of one scan each plus a duplicated predicate.
- Dropping the `version != MAJOR_V1` guard from `parse_keyboard_geometry` moves that refusal earlier
  (into the shared gate); it removes a check from the format arm rather than adding one.
- Two new invalid fixtures add two `parse` calls to the conformance walk. Negligible.

## Checklist

1. **Event/keystroke path** — pass. No fs or network added; the only new fs cost is a test helper
   (finding 2). No per-event table rebuild; loading still happens once in `load_reach_tables` /
   `load_geometry` / `load_action_registry`.
2. **Complexity** — pass for the delta. The new gate is O(1) per parse. Nothing in the delta introduces
   an O(n²) over an unbounded n. The pre-existing scans (grades, duplicate-id rules, target distinctness)
   are unchanged; see carry-forward.
3. **Unbounded growth** — pass. No cache, registry, or array is added. Every new list is a `&'static`
   slice of at most two majors.
4. **Startup** — pass. `parse` gains one match and one ≤2-element `contains`; the allocation the old
   membership test did is not made worse (finding 1 asks that it be removed, since the new gate now
   subsumes it).
5. **Cache invalidation** — not applicable: the crate caches nothing, and the delta adds nothing
   cached, so there is no invalidation rule for a test to cover.
6. **Benchmark-like test or timing note** — nothing to run. A grep over `crates/core` for
   `Instant`/`bench`/`elapsed`/`duration`/`microsecond` finds only the `ScratchDir` nonce
   (`tests/common/mod.rs:189`), which is not a measurement. Every cost statement in this report is
   therefore analytic, bounded by the closed 48-member `keyboard:` set and by the ≤2-element majors
   lists; no number is quoted because the tree ships no timing harness to quote one from.

## Findings

### nit — the allocating majors membership test is now redundant beside the new per-format gate

- **Location:** `crates/core/src/document.rs:232` (with `:114-116`, `:227`, and the new `:125-148`)
- **Problem:** `parse` now checks version membership twice: `if !supported_majors().contains(&version)`
  at 232, then `check_format_major(format, &version)` at 241. The second is allocation-free; the first
  builds a fresh `Vec<String>` of two heap `String`s per `parse` call purely to test membership — this
  is the same site round 1 flagged (`document.rs:186` then), and the delta left it while adding a
  static-list gate next to it. For every format `format_majors` knows, the union test is now implied by
  the per-format test, so it is a duplicated lookup *and* the only allocation on the parse happy path,
  paid once per fixture in the conformance walk.
- **Fix:** hoist the union into `const MAJORS: &[&str] = &[MAJOR_V1, MAJOR_V2]` (or derive it from
  `format_majors`) and test `MAJORS.contains(&version.as_str())`, keeping the union check *before*
  `check_format_major` so an unknown format naming an unshipped major still gets `UnsupportedVersion`
  rather than `UnknownFormat`. Leave `supported_majors()` as a thin map/collect wrapper so the public
  shape and the `supported_majors()`-equals-dirs-on-disk assertion (Test strategy #6) are untouched.

### nit — the new `format_majors()` test helper re-walks the contract tree on every call

- **Location:** `crates/core/tests/common/mod.rs:110-116`, called at
  `crates/core/tests/version_gate.rs:88,103,128,197` and `crates/core/tests/action.rs:517,535`
- **Problem:** `format_majors` builds its answer with `format_dirs()`, i.e. a fresh two-level
  `fs::read_dir` of `schemas/` (sorted `PathBuf` + `String` per entry) per call. `version_gate.rs:128`
  calls it inside a 3-iteration loop, so one test run performs roughly six full walks (~36 `read_dir`
  calls plus their allocations) to answer a question about two-element lists. Test-only and
  sub-millisecond — no live cost — but the same suite already walks once and hands the list to what
  needs it (`zero_fixture_violation(dirs)` at `tests/common/mod.rs:157`, `conformance.rs:79`), and this
  helper is the one new fs cost the delta adds.
- **Fix:** take the walked list — `pub fn format_majors(dirs: &[FormatDir], format: &str) -> Vec<String>`
  — and let each test call `format_dirs()` once; or memoise the walk in a `OnceLock` inside
  `format_dirs()`. The assertion content is unchanged either way.

## Round-1 carry-forward — nothing broke

The delta touches none of the four sites round 1 reported, so each is exactly as approved: still open,
not regressed. Line numbers have shifted only because of the module-doc and gate insertions above them.

| Round-1 finding | Current site | Status after the delta |
|---|---|---|
| minor — linear graded lookup plus the `rank_of` re-scan | `reach.rs:107`, `reach.rs:118`, `placement.rs:280-281` | unchanged; per-move cost is still two scans of ≤48 `InputId` comparisons plus two of 5 |
| nit — `iter().any` duplicate-id rules in the parser | `document.rs:555`, `document.rs:600` | unchanged; still O(k²) over a ≤48-element array |
| nit — allocating majors membership test | `document.rs:232` | unchanged, and now duplicated by the per-format gate → re-filed above as a nit |
| nit — `targets_are_distinct` Vec scan; `inputs()` clones 48 ids | `placement.rs:326-335`, `placement.rs:55-57` | unchanged |

The new `placement.rs` test case (`tests/placement.rs:448-502`) calls `placement_is_legal` twice on
48-entry tables instead of once — a few hundred extra byte comparisons in one test, not a cost finding.

## FYI (out of specialty, no severity)

`document.rs:666-669` still says the stand-in directories the conformance harness loads fixtures in are
*not* named for a major, while `schemas/reach-classes/v2/spec.md` (edited in this delta) and
`tests/conformance.rs:124-128` now name the stand-in for the fixture's own major — so `check_dir_version`
no longer short-circuits for the harness, and that sentence describes a tree that no longer exists.
Correctness/compatibility seats own whether that reads as drift.
