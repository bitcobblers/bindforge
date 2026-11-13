# Transcript: design-review:performance r1c1

## Provenance
- stage: design-review:performance (round r1c1)
- agent: review/design-performance
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:49:38.640Z
- ended: 2026-11-11T22:54:09.593Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff cc78a21d26a8a556fb75f7206e403d8e8618c604 7b2ddc6d10df95a81e738c734fe5c9673d530181` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/009-canonical-profile-and-manifest-model.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Profiles are chains, \"Id → file, never id → path\"; also Adversarial posture and Rollout step 2 — \"walks `data/profiles/` (bounded depth and entry count, each file size-checked by the shipped budget)\", \"Chain and id resolution add no new budget: they reuse `MAX_CHAIN_DEPTH` and the tree-walk bounds\", \"No new dependency, no budget change\"",
    "problem": "There is no shipped tree-walk bound to reuse. The baseline the research and the proposal quote is per-document only — MAX_DOCUMENT_BYTES = 1 MiB pre-allocation, MAX_NESTING_DEPTH = 16, MAX_CHAIN_DEPTH = 32 (01-research Chains/patches/transforms; schemas/README.md \"Input budget and chain bound\"; the three public constants in crates/core/src/lib.rs). So the index's aggregate exposure is the 1 MiB per-file cap times an unnumbered entry count: total parse time and resident bytes at load are bounded by nothing the design pins, which is exactly the checklist's bounded-memory item (the Index is the one table with no bound) and startup-cost item (unbounded file reads at boot). The claim that the design \"add[s] no new budget\" and Rollout's \"no budget change\" are also false as written, since depth/entry-count limits are new budget that implementation must invent mid-coding.",
    "fix": "Name the walk bounds as public constants in index.rs beside the shipped ones (MAX_INDEX_ENTRIES, MAX_INDEX_DEPTH, re-exported from lib.rs like MAX_DOCUMENT_BYTES) with seed-scale headroom stated in A24's voice (the seed is one base + one patch chain). Correct the two claims: Adversarial posture becomes \"no change to the per-document budget; the tree walk adds MAX_INDEX_ENTRIES/MAX_INDEX_DEPTH\", Rollout 2 drops \"no budget change\" there, and Test strategy #1 pins an over-entry-count and over-depth walk case beside the over-size and over-nesting cases."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Action taxonomy, API line — \"`Taxonomy::load(dir: &Path) -> Result<Taxonomy, CoreError>`\", with data at `schemas/taxonomy/v1/actions.toml` and \"the refused-TOML-construct cases\" in the same directory",
    "problem": "A directory-taking loader has no stated file-selection rule, and that directory also ships spec.md and a fixtures/ tree that Rollout 1 requires to carry a valid+invalid pair per format plus the refused-construct cases. The boot read set is therefore undefined: a discover-every-*.toml implementation reads fixture files on every load and folds fixture ids into the taxonomy. placement is the mirror case — its format dir ships no data file at all (content is data/placements/*.toml), so a dir-shaped loader has nothing pinned to find. This is the checklist's startup-cost item: unscoped file reads at boot.",
    "fix": "Take the document path, not the directory: Taxonomy::load(path: &Path) pinned to actions.toml, mirroring the shipped load_reach_tables, which names its per-hand file and needs no discovery. State in taxonomy/v1/spec.md that fixtures/ is harness input and never load input."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Manifests state game facts, \"Drift mechanism\" (with Test strategy #13 and Rollout 5) — \"The `cargo test` gate resolves every profile in `data/profiles/` against the manifest revision it pins and re-runs `validate`\"",
    "problem": "Resolution requires an Index, and an Index is a full parse of every document under data/profiles/. The gate then costs (index-building tests) x (parse of every file), because model_walk.rs, manifest_drift.rs and the conformance walk are separate test binaries and each test function that builds its own Index reparses the whole tree. The API already supports build-once — resolve(index: &Index, root, rung) — so the repeated cost is unstated, not forced, and the design never says the gate's cost is linear in files.",
    "fix": "One sentence in Rollout 5: the drift and model-walk suites share a single Index built once per test binary (a shared fixture), so the gate stays linear in the number of files rather than in the number of tests."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/009-canonical-profile-and-manifest-model.md.
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/performance-r1c1.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Manifests state game facts (with Test strategy #8) — \"`validate(manifest, &Taxonomy)` enforces: … every `exclusive` and `docked` member appears in the `action` rows\"","problem":"The r1 rewrite replaced the role-qualified quoted keys (\"action:<id>\", a keyed map) with the nested-array rows `action = [[id, name]]`, so the action table is a list rather than a lookup. \"Appears in the `action` rows\" is therefore a scan per `exclusive`/`docked` member, and the only bound on the row count is the shipped per-file 1 MiB cap (A24, A25): an adversarial manifest with thousands of `action` rows and thousands of `exclusive` members makes validation quadratic (≈10⁹ comparisons) inside a document the budget accepts. Nothing in the design pins the indexing, so the cost is decided mid-coding. New in this revision, because the row encoding is the changed text.","fix":"One clause in §5, echoed in Test #8's voice: `validate` builds the `action`-row id set once as a `BTreeSet` and does membership lookups, so validation is `O(rows log rows)` in the document."},{"severity":"minor","location":"Chosen design → Transforms → `relocate-cluster` — \"the matched actions' current inputs in `base` are isometric to the named placement's source-key set (same offset vectors, per `placement::validate`'s shape rule over `<hand>`'s table)\"","problem":"\"Same offset vectors\" between two sets, read as the shape rule it cites states it (\"the offset vectors between source keys equal the offset vectors between the target keys\"), is an all-pairs comparison: O(k²) per `apply_policy` invocation, with a reach-table lookup per pair. For a placement document k is the pair count (4 in the shipped example) and the cost is trivial; this new call site sets k to every action matching `<action-prefix>`, bounded only by the base's entry count under the 1 MiB cap. Not the keystroke path (no event API in scope), so bounded-input cost, not per-event.","fix":"State the linear form in the shape rule both call sites reference — compare one anchor's offset plus the sorted coordinate multiset of the two key sets, O(k log k) — so the precondition's cost is pinned in the text `validate` and `relocate-cluster` both cite."},{"severity":"minor","location":"Files created → `schemas/README.md` row and Rollout 1 (the data-homes rule), read against Chosen design → Profiles are chains → \"Id → file, never id → path\" — \"`data/` holds content discovered by walking\"","problem":"The newly added data-homes rule makes every `data/` tree walked content, but `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH` are scoped to the two walks that name them (`data/profiles/`, `data/manifests/`). `data/placements/*.toml`, which is what the transforms' `Ctx { … placements: &Placements }` is built from, and `data/policies/` are discovered by a walk with no entry-count bound named and no pinned loader signature — the same aggregate-exposure plus unscoped-read shape that round 1 closed for the other two trees, now in text this revision introduced.","fix":"Extend §3's bound clause to name every walked `data/` tree (placements, policies) under the same `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH`, and name the placements loader (`Placements::load(dir)`) beside `Taxonomy::load` and `load_reach_tables`, so the read set behind `Ctx` is a pinned path rule rather than an implementation choice."}]}
```
