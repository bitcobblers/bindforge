# Transcript: design-review:performance r1c4

## Provenance
- stage: design-review:performance (round r1c4)
- agent: review/design-performance
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:11:13.560Z
- ended: 2026-11-11T23:13:48.811Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the ratification pass c4: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 7b2ddc6d10df95a81e738c734fe5c9673d530181 f7521536a6bca93f4f9b4d4a4cfce3446567cdc7` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "severity": "minor",
    "location": "Chosen design → Manifests state game facts (with Test strategy #8) — \"`validate(manifest, &Taxonomy)` enforces: … every `exclusive` and `docked` member appears in the `action` rows\"",
    "problem": "The r1 rewrite replaced the role-qualified quoted keys (\"action:<id>\", a keyed map) with the nested-array rows `action = [[id, name]]`, so the action table is a list rather than a lookup. \"Appears in the `action` rows\" is therefore a scan per `exclusive`/`docked` member, and the only bound on the row count is the shipped per-file 1 MiB cap (A24, A25): an adversarial manifest with thousands of `action` rows and thousands of `exclusive` members makes validation quadratic (≈10⁹ comparisons) inside a document the budget accepts. Nothing in the design pins the indexing, so the cost is decided mid-coding. New in this revision, because the row encoding is the changed text.",
    "fix": "One clause in §5, echoed in Test #8's voice: `validate` builds the `action`-row id set once as a `BTreeSet` and does membership lookups, so validation is `O(rows log rows)` in the document."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Transforms → `relocate-cluster` — \"the matched actions' current inputs in `base` are isometric to the named placement's source-key set (same offset vectors, per `placement::validate`'s shape rule over `<hand>`'s table)\"",
    "problem": "\"Same offset vectors\" between two sets, read as the shape rule it cites states it (\"the offset vectors between source keys equal the offset vectors between the target keys\"), is an all-pairs comparison: O(k²) per `apply_policy` invocation, with a reach-table lookup per pair. For a placement document k is the pair count (4 in the shipped example) and the cost is trivial; this new call site sets k to every action matching `<action-prefix>`, bounded only by the base's entry count under the 1 MiB cap. Not the keystroke path (no event API in scope), so bounded-input cost, not per-event.",
    "fix": "State the linear form in the shape rule both call sites reference — compare one anchor's offset plus the sorted coordinate multiset of the two key sets, O(k log k) — so the precondition's cost is pinned in the text `validate` and `relocate-cluster` both cite."
  },
  {
    "severity": "minor",
    "location": "Files created → `schemas/README.md` row and Rollout 1 (the data-homes rule), read against Chosen design → Profiles are chains → \"Id → file, never id → path\" — \"`data/` holds content discovered by walking\"",
    "problem": "The newly added data-homes rule makes every `data/` tree walked content, but `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH` are scoped to the two walks that name them (`data/profiles/`, `data/manifests/`). `data/placements/*.toml`, which is what the transforms' `Ctx { … placements: &Placements }` is built from, and `data/policies/` are discovered by a walk with no entry-count bound named and no pinned loader signature — the same aggregate-exposure plus unscoped-read shape that round 1 closed for the other two trees, now in text this revision introduced.",
    "fix": "Extend §3's bound clause to name every walked `data/` tree (placements, policies) under the same `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH`, and name the placements loader (`Placements::load(dir)`) beside `Taxonomy::load` and `load_reach_tables`, so the read set behind `Ctx` is a pinned path rule rather than an implementation choice."
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
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/performance-r1c4.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Files created → `schemas/README.md` row, read against Chosen design → Profiles are chains → Id → file — \"the guard section's two new index limits and their rejections, and the data-homes rule: the contract tree holds lookup tables a loader reads at a fixed path, `data/` holds content discovered by walking\" / \"`.toml` for `schemas/` fixtures and for the `data/` manifest, placement, and policy documents\"","problem":"re-raised, not closed, and widened by the text this revision touched. The new README cell pairs the two index limits with the data-homes rule, and the rewritten §3 clause newly names the `data/` placement and policy documents as walked content, while the bound sentence those edits live in is scoped to exactly two walks (`MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH` on `data/profiles/` and `data/manifests/`). The guard text now states a bound for `data/` as a whole that covers two of its four trees, and A25's \"their product is the only aggregate bound on bytes read at load\" no longer names every tree read at load: `Ctx { … placements: &Placements }` is built from `data/placements/`, and policies from `data/policies/`, by a walk with no entry-count bound named and no pinned loader signature beside `Taxonomy::load(path)` / `load_reach_tables`. Bounded-input memory/read-volume cost, not a keystroke path (no event API in scope).","fix":"Extend §3's bound clause to name every walked `data/` tree — \"`placements` and `policies` included, under `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH`\" — and name `Placements::load(dir)` beside the two pinned loaders, so the README guard cell and A25's aggregate-bytes claim cover the read set behind `Ctx`. No new mechanism: one clause, one signature."}]}
```
