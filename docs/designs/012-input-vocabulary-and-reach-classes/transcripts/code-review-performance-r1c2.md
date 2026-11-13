# Transcript: code-review:performance r1c2

## Provenance
- stage: code-review:performance (round r1c2)
- agent: codereview/code-performance
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T12:49:04.101Z
- ended: 2026-11-13T12:50:30.741Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 2682fb1f740770fdb588db1a3ff69d66e9d1f3d4..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/012-input-vocabulary-and-reach-classes.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (performance seat):
[
  {
    "severity": "minor",
    "location": "crates/core/src/reach.rs:107 (and crates/core/src/reach.rs:118)",
    "problem": "The graded lookup is a linear scan — `self.grades.iter().find(|entry| entry.input == *input)` — and `placement::degradation` (placement.rs:274-281) then re-finds the class index in `classes` by string via `rank_of`, so each move costs two scans of 48 `InputId` byte-comparisons plus two scans of 5. This is the public predicate ADR-015 preconditions its transforms on (A5), so a placement search runs it per action per candidate placement. The same change indexes the identical id→value lookup with a `BTreeMap<InputId, _>` for coordinates (placement.rs:40), and the crate's other closed id set uses a map for its 'listed once' rule (action.rs:149,200). Cost today is sub-microsecond (a 4-move check is roughly 400 short byte-comparisons, bounded by the closed 48-member `keyboard:` set), so this is a cost-contract/consistency finding rather than a measured regression.",
    "fix": "Index the grades at load — `grades: BTreeMap<InputId, ReachClass>` (`InputId` already derives `Ord`, so `grade(&InputId) -> Option<&ReachClass>` keeps its signature) — or keep the document-ordered `Vec` and store the class rank beside each `InputGrade` at parse time so a move costs one probe and no rank scan; either way state in the doc comment that the scan is bounded by the closed 48-member set."
  },
  {
    "severity": "nit",
    "location": "crates/core/src/document.rs:502 and crates/core/src/document.rs:553",
    "problem": "The 'each id appears once' rules are `iter().any(...)` scans inside the parse loop, making a k-element `keys` array O(k²). The closed member set caps it at ~2,300 comparisons per document, so there is no live cost; it is flagged because `registry_from_entries` (action.rs:200) enforces the same rule in O(log k) via `BTreeMap::insert(...).is_some()`, and matching that shape removes a scan class from the parser instead of re-justifying the bound at each site.",
    "fix": "Insert into a `BTreeMap<InputId, _>`/`BTreeSet<InputId>` as elements are parsed and refuse when the insert reports a repeat, keeping the current message text."
  },
  {
    "severity": "nit",
    "location": "crates/core/src/document.rs:186",
    "problem": "The version gate calls `supported_majors()` just to test membership, allocating a fresh `Vec<String>` — now two heap Strings, up from one — on every `parse` call, including every fixture in the conformance walk.",
    "fix": "Add `const MAJORS: &[&str] = &[MAJOR_V1, MAJOR_V2]` and test against it, leaving `supported_majors()` as a thin map/collect wrapper so the pinned public shape and the `supported_majors()`-equals-dirs-on-disk assertion (Test strategy #6) are unchanged."
  },
  {
    "severity": "nit",
    "location": "crates/core/src/placement.rs:327-333",
    "problem": "`targets_are_distinct` allocates a `Vec<&InputId>` and calls `seen.contains(&target)`, an O(m²) scan over a caller-supplied move list; for shipped tables the list cannot pass 49 moves without a repeat (closed-set bound), so no measured impact, but it is the third identity lookup in the change done as a scan next to `KeyboardGeometry`'s map. Related: `KeyboardGeometry::inputs()` (placement.rs:55-57) clones all 48 ids into a fresh `Vec` per call.",
    "fix": "Use a `BTreeSet<&InputId>` for the distinctness check, and have `inputs()` return an iterator over `self.coordinates.keys()` instead of a cloned `Vec`."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/012-input-vocabulary-and-reach-classes/reviews/code/performance-r1c2.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict": "approve", "findings": [{"severity": "nit", "location": "crates/core/src/document.rs:232 (with :114-116, :227, and the new gate at :125-148)", "problem": "parse now checks version membership twice: `if !supported_majors().contains(&version)` (232) and `check_format_major(format, &version)` (241). The new half is allocation-free (`format_majors` returns `Option<&'static [&'static str]>`), but the old half still builds a fresh `Vec<String>` of two heap Strings per parse call just to test membership — the same site round 1 flagged, left in place while a static-list gate was added next to it. For any format `format_majors` knows, the union test is now implied by the per-format test, so it is a duplicated lookup and the only allocation on the parse happy path, paid once per fixture in the conformance walk.", "fix": "Hoist the union into `const MAJORS: &[&str] = &[MAJOR_V1, MAJOR_V2]` (or derive it from `format_majors`) and test `MAJORS.contains(&version.as_str())`, keeping the union check before `check_format_major` so an unknown format naming an unshipped major still gets `UnsupportedVersion` rather than `UnknownFormat`. Leave `supported_majors()` as a thin map/collect wrapper so the public shape and the majors-equals-dirs-on-disk assertion (Test strategy #6) are unchanged."}, {"severity": "nit", "location": "crates/core/tests/common/mod.rs:110-116, called at crates/core/tests/version_gate.rs:88,103,128,197 and crates/core/tests/action.rs:517,535", "problem": "The new `format_majors` helper answers a two-element question by calling `format_dirs()`, i.e. a fresh two-level `fs::read_dir` walk of `schemas/` (sorted PathBuf plus String per entry) on every call. `version_gate.rs:128` calls it inside a 3-iteration loop, so one `cargo test` run performs roughly six full walks (~36 read_dir calls plus their allocations). Test-only and sub-millisecond, so no live cost, but the same suite already walks once and passes the list down (`zero_fixture_violation(dirs)` at tests/common/mod.rs:157, conformance.rs:79), and this is the only new fs cost the delta adds.", "fix": "Take the walked list — `pub fn format_majors(dirs: &[FormatDir], format: &str) -> Vec<String>` — so each test calls `format_dirs()` once, or memoise the walk in a `OnceLock` inside `format_dirs()`; the assertion content is unchanged either way."}]}
```
