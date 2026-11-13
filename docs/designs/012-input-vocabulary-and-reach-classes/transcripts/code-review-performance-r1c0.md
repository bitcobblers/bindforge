# Transcript: code-review:performance r1c0

## Provenance
- stage: code-review:performance (round r1c0)
- agent: codereview/code-performance
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T12:28:13.908Z
- ended: 2026-11-13T12:29:52.620Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff c26bf7f02371a1535a4d956d00294479b94d5532...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/012-input-vocabulary-and-reach-classes.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/012-input-vocabulary-and-reach-classes/reviews/code/performance-r1c0.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "crates/core/src/reach.rs:107 (and crates/core/src/reach.rs:118)", "problem": "The graded lookup is a linear scan — `self.grades.iter().find(|entry| entry.input == *input)` — and `placement::degradation` (placement.rs:274-281) then re-finds the class index in `classes` by string via `rank_of`, so each move costs two scans of 48 `InputId` byte-comparisons plus two scans of 5. This is the public predicate ADR-015 preconditions its transforms on (A5), so a placement search runs it per action per candidate placement. The same change indexes the identical id→value lookup with a `BTreeMap<InputId, _>` for coordinates (placement.rs:40), and the crate's other closed id set uses a map for its 'listed once' rule (action.rs:149,200). Cost today is sub-microsecond (a 4-move check is roughly 400 short byte-comparisons, bounded by the closed 48-member `keyboard:` set), so this is a cost-contract/consistency finding rather than a measured regression.", "fix": "Index the grades at load — `grades: BTreeMap<InputId, ReachClass>` (`InputId` already derives `Ord`, so `grade(&InputId) -> Option<&ReachClass>` keeps its signature) — or keep the document-ordered `Vec` and store the class rank beside each `InputGrade` at parse time so a move costs one probe and no rank scan; either way state in the doc comment that the scan is bounded by the closed 48-member set."}, {"severity": "nit", "location": "crates/core/src/document.rs:502 and crates/core/src/document.rs:553", "problem": "The 'each id appears once' rules are `iter().any(...)` scans inside the parse loop, making a k-element `keys` array O(k²). The closed member set caps it at ~2,300 comparisons per document, so there is no live cost; it is flagged because `registry_from_entries` (action.rs:200) enforces the same rule in O(log k) via `BTreeMap::insert(...).is_some()`, and matching that shape removes a scan class from the parser instead of re-justifying the bound at each site.", "fix": "Insert into a `BTreeMap<InputId, _>`/`BTreeSet<InputId>` as elements are parsed and refuse when the insert reports a repeat, keeping the current message text."}, {"severity": "nit", "location": "crates/core/src/document.rs:186", "problem": "The version gate calls `supported_majors()` just to test membership, allocating a fresh `Vec<String>` — now two heap Strings, up from one — on every `parse` call, including every fixture in the conformance walk.", "fix": "Add `const MAJORS: &[&str] = &[MAJOR_V1, MAJOR_V2]` and test against it, leaving `supported_majors()` as a thin map/collect wrapper so the pinned public shape and the `supported_majors()`-equals-dirs-on-disk assertion (Test strategy #6) are unchanged."}, {"severity": "nit", "location": "crates/core/src/placement.rs:327-333", "problem": "`targets_are_distinct` allocates a `Vec<&InputId>` and calls `seen.contains(&target)`, an O(m²) scan over a caller-supplied move list; for shipped tables the list cannot pass 49 moves without a repeat (closed-set bound), so no measured impact, but it is the third identity lookup in the change done as a scan next to `KeyboardGeometry`'s map. Related: `KeyboardGeometry::inputs()` (placement.rs:55-57) clones all 48 ids into a fresh `Vec` per call.", "fix": "Use a `BTreeSet<&InputId>` for the distinctness check, and have `inputs()` return an iterator over `self.coordinates.keys()` instead of a cloned `Vec`."}]}
```
