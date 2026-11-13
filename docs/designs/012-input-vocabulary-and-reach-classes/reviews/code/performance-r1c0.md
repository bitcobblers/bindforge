# Performance review — design 012, code, round 1, seat c0

Scope: `git diff c26bf7f02371a1535a4d956d00294479b94d5532...HEAD -- ':(exclude)docs/designs'`
(38 files, +3496/−79: `crates/core/src/{input,placement,reach,document,lib}.rs`, eight test
targets, `schemas/reach-classes/v2/`, `schemas/keyboard-geometry/v1/`, one v1 invalid fixture,
`schemas/README.md`). Judged against `02-proposal.md` and ADR-012. Specialty: performance only.

## Checklist walk

1. **Event/keystroke path.** Clean. There is no event path in this stage, and the change does
   not create one: `placement_is_legal` (`placement.rs:173-206`) is a pure function over the
   loaded `ReachTables` and `KeyboardGeometry` the caller already holds — no fs, no network, no
   re-parse, no table rebuild per call. File reads stay in the explicit loaders
   (`load_reach_tables` `reach.rs:145`, `load_geometry` `placement.rs:136`), one `parse_file`
   per call, as the proposal's "pure function over loaded data" requires. The predicate
   pre-reserves both output vectors (`placement.rs:183-184`) and formats refusal strings only on
   the refusal path.
2. **Complexity.** No unbounded quadratic. The only scans are over `grades` (48), `classes` (5)
   and the caller's move list; the `grades` and geometry-key vectors cannot grow past 48 because
   `InputId::parse` accepts only the closed member set (`input.rs:179-191`, 48 `keyboard:`
   members), so a 49th element must repeat and the parse refuses there
   (`reach-classes/v2/fixtures/invalid/duplicate-input.toml`,
   `keyboard-geometry/v1/fixtures/invalid/duplicate-input.toml`; the shipped-set bound is pinned
   by `reach_v2.rs:470`). Where identity lookup was intended the code nonetheless scans in three
   places while indexing the identical lookup with a map elsewhere — findings 1-4.
3. **Unbounded growth.** Clean. No cache, registry, or global was added (`grep` for
   `static`/`OnceLock`/`Lazy`/`RefCell`/`Mutex` in `crates/core/src` finds only `&'static str`
   rule text). The new collections are per-load values bounded at 48; the new contract data is
   small — `wc -c`: `reach-classes/v2/left.toml` 1,522 B, `right.toml` 1,430 B,
   `keyboard-geometry/v1/ansi-us.toml` 1,260 B, i.e. ~4.2 KB parsed for a full v2 + geometry
   load against v1's 242 B per hand.
4. **Startup.** Clean. Nothing eager: no `static` initializer, no lazy loader, no build-time
   read. Boot parses nothing it did not parse before; a consumer that adopts v2 pays three extra
   small files at a load it chooses.
5. **Cache invalidation.** Not applicable, not a gap: there is no cache in the crate or in this
   change, and the loaded model (`ReachTables`, `KeyboardGeometry`) is an immutable value, so
   there is no invalidation rule to write or pin. Re-loading is the only refresh path, and it
   re-reads from disk (`placement.rs:144`, `reach.rs:145`), which is why a stale value cannot
   outlive a table edit.
6. **Benchmark-like check.** The change ships none, so there was nothing to run: the only hit for
   `benchmark`/`Instant`/`SystemTime`/`elapsed` across the diff is
   `schemas/reach-classes/v2/spec.md:131`, which is A8's data-provenance note ("no external
   benchmark validates them"), not a timing note. The figures in the findings are `wc -c` sizes
   and closed-set arithmetic, not measurements I ran; a measured number would need a bench target
   this stage deliberately does not have ("this stage ships no binary").

## Findings

### 1. minor — `crates/core/src/reach.rs:107` (and `:118`): the graded lookup is a linear scan
`ReachTable::grade` is `self.grades.iter().find(|entry| entry.input == *input)`, and
`placement::degradation` (`placement.rs:274-281`) then re-finds the class's index in `classes` by
string through `rank_of` (`placement.rs:301-311`, `reach.rs:116-119`). Each move therefore costs
two scans of 48 `InputId` byte-comparisons plus two scans of 5, per call, and this is the public
predicate ADR-015 preconditions its transforms on (A5) — a placement search runs it once per
action per candidate placement. The same change indexes the identical id→value lookup with a map
for coordinates (`placement.rs:40`, `BTreeMap<InputId, (u32, u32)>`) and the crate's other closed
id set uses the map for its "listed once" rule (`action.rs:149`, `action.rs:200`). Cost today is
sub-microsecond (a 4-move check is roughly 400 short byte-comparisons), so this is a
cost-contract/consistency finding, not a measured regression.
**Fix:** index the grades at load — `grades: BTreeMap<InputId, ReachClass>` (`InputId` already
derives `Ord`, so `grade(&InputId) -> Option<&ReachClass>` keeps its signature), or keep the
document-ordered `Vec` and store the class's rank beside each `InputGrade` at parse time so a
move costs one probe and no rank scan. Either way, state in the doc comment that the scan is
bounded by the closed 48-member `keyboard:` set.

### 2. nit — `crates/core/src/document.rs:502` and `:553`: O(k²) "listed once" checks
`grades.iter().any(|grade| grade.input == input)` and `keys.iter().any(|other| other.input ==
key.input)` inside the parse loop make a k-element `keys` array quadratic. The closed member set
caps it at ~2,300 comparisons for a whole document, so this is not a live cost; it is flagged
because `registry_from_entries` (`action.rs:200`) enforces the same rule in O(log k) with
`registry.entries.insert(id, kind).is_some()`, and reusing that shape removes a scan class from
the parser rather than re-justifying the bound at each call site.
**Fix:** insert into a `BTreeMap<InputId, _>` (or `BTreeSet<InputId>`) as elements are parsed and
refuse on `is_some()`, keeping the existing message.

### 3. nit — `crates/core/src/document.rs:186`: the version gate allocates per parse
`if !supported_majors().contains(&version)` builds a fresh `Vec<String>` — now two heap
`String`s, up from one — on every `parse` call, purely for a membership test. Parse is the crate's
public entry point and the conformance walk calls it for every fixture.
**Fix:** keep `const MAJORS: &[&str] = &[MAJOR_V1, MAJOR_V2]` and test against it; leave
`supported_majors()` as a thin `MAJORS.iter().map(|major| (*major).to_string()).collect()` so the
pinned public shape and the `supported_majors()`-equals-disk assertion (Test strategy #6) are
untouched.

### 4. nit — `crates/core/src/placement.rs:327-333`: target distinctness by scan-plus-Vec
`targets_are_distinct` allocates a `Vec<&InputId>` and calls `seen.contains(&target)` — O(m²) over
a caller-supplied move list. For shipped tables the list cannot exceed 49 moves without a repeat
(the same closed-set bound), so no measured impact; the finding is that this is the third
identity lookup in the change done as a scan, next to `KeyboardGeometry`'s map. Related: the
public `KeyboardGeometry::inputs()` (`placement.rs:55-57`) clones all 48 ids into a fresh `Vec` on
every call.
**Fix:** a `BTreeSet<&InputId>` for the distinctness check (early exit needs no allocation beyond
the set), and return an iterator over `self.coordinates.keys()` from `inputs()`.

## Out of specialty, no severity (FYI)

`crates/core/tests/zz_api_probe.rs` is a fourth test target; the proposal's delta inventory names
three (`tests/input.rs`, `tests/placement.rs`, `tests/reach_v2.rs`). Not a performance matter —
its load calls are three small files in one test.

## Verdict

`approve` — no blocker or major performance finding. Nothing in the change puts fs, network, a
rebuild, or an unbounded allocation on a caller's hot path, nothing grows without a bound, and
the four findings are bounded-cost cleanups (map-vs-scan consistency in three spots, one
per-parse allocation).
