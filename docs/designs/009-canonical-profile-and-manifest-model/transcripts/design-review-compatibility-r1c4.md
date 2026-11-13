# Transcript: design-review:compatibility r1c4

## Provenance
- stage: design-review:compatibility (round r1c4)
- agent: review/design-compatibility
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:11:13.559Z
- ended: 2026-11-11T23:15:09.523Z
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
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "Compatibility matrix — Windows 10/11 contributor machine row: \"exercised leg (Test strategy #13)\"",
    "problem": "The r1 renumbering moved the platform leg to Test strategy #14; #13 is now \"Integration — harness pickup and the version gate\", which tests nothing about a case-insensitive filesystem. Checklist item 7 requires the matrix row for each supported platform to name its platform test, and Rollout 7 already cites #14, so the matrix cell is the stale one. Fix: that one cell.",
    "fix": "Change \"exercised leg (Test strategy #13)\" to \"exercised leg (Test strategy #14)\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Input vocabulary + Files created (README row) — \"`schemas/README.md` carries the admitted keyboard-name list — the single enumeration the graded set below and Test strategy #3 iterate\" and the README row's \"(the enumeration Test strategy #3 iterates)\"",
    "problem": "The same revision rewrote Test strategy #3 to iterate the graded set, which the Authoring bar defines by a different rule (\"every key the shipped placements and the seed manifest's `default` rows reference, extended only by review\"). No CI check any longer enumerates README's admitted keyboard-name list, so the list's role is unstated and the US-ANSI `yes (v1)` matrix row's evidence trail points at a sentence that no longer matches the test it cites. The narrowing itself is the Revision log's r1 decision and is not re-opened; the dangling cross-reference is what the closure introduced.",
    "fix": "Name README's list as the admissible-name namespace and the graded set as its reviewed subset: drop \"and Test strategy #3 iterate\" from Input vocabulary and the parenthetical from the README row (leaving Test #3 to state it iterates the graded set). If the README list is meant to be graded in full, instead restore in Test #3 a clause that every admitted keyboard name is graded in both hands."
  },
  {
    "severity": "minor",
    "location": "Adversarial posture — \"New `CoreError` variants, each pinned by a fixture: … `DuplicateId`, `UnresolvedAncestor`, `MergeConflict`, `BadPin`, `IndexTooLarge`, `IndexTooDeep`\"",
    "problem": "A fixture is one document under `vN/fixtures/{valid,invalid}` (schemas/README.md:23,41-42; crates/core/tests/conformance.rs:47-78), which cannot carry these conditions. `IndexTooLarge`/`IndexTooDeep` (added by this revision) are tree-shape conditions, not document conditions; and `MergeConflict`'s implementation and suite moved to the ADR-012 slice in this revision, so it has no v1 code path and no v1 pin. Rollout 1's \"a valid+invalid fixture pair per format\" cannot ship those cases as the sentence claims.",
    "fix": "Split the sentence: variants a single document pins are pinned by an `fixtures/invalid/` case; `DuplicateId`, `UnresolvedAncestor`, `IndexTooLarge`, `IndexTooDeep` are pinned by the multi-document / scratch-tree unit tests Test strategy #1 and #9 name; `MergeConflict` ships and is pinned in the ADR-012 slice with its rule in `patch/v1/spec.md`."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Manifests §5 (Entries) — \"`default` = `[id, input]` (the emphasis fact, contract 1)\"",
    "problem": "Contract 1 is the action taxonomy, from which ADR-009 Decision 1 and this proposal's own §1 (\"the game fact is the action's default input, recorded as a manifest `default` row (contract 5)\") both exclude emphasis. The rewritten entry sentence cites the wrong contract.",
    "fix": "Cite contract 5 for that row."
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix — Windows row: \"tree file names follow the shipped case-unique stem rule\"",
    "problem": "The shipped case-uniqueness check runs over `schemas/*/v*/fixtures/{valid,invalid}` only (crates/core/tests/conformance.rs:277-292); `data/profiles/` and `data/manifests/` file names are unenforced. Harmless in substance, exactly because ids never become path components, but the row's wording claims coverage the shipped test does not have.",
    "fix": "Scope the wording to fixture names and state that `data/` file names are hygiene, not load-bearing."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/compatibility-r1c4.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"nit","location":"Compatibility matrix — Windows 10/11 contributor machine row: \"tree file names follow the shipped case-unique stem rule\"","problem":"re-raised. The shipped case-uniqueness assertion runs over fixture-directory names only (crates/core/tests/conformance.rs:277-311, \"fixtures/{kind} of {} holds names that differ only by case\"), and schemas/README.md:26-27 scopes the rule to \"fixture file names\". This revision widened the sentence the row rests on: §3 now states the extension rule for \"the `data/` manifest, placement, and policy documents\", so the row claims case-uniqueness coverage over trees no shipped test enforces. Harmless in substance — ids never become path components, the design's own reason — so it stays a nit.","fix":"In that one cell, scope the claim: \"fixture names follow the shipped case-unique stem rule; `data/` file names follow the stem rule as hygiene, not a load-bearing check\"."},{"severity":"nit","location":"Chosen design → Adversarial posture (final sentence) + Files created → `schemas/README.md` row: \"`schemas/README.md`'s guard section gains those two constants and their rejections beside the shipped three, so its 'public names for these limits' claim stays true\"","problem":"The README's canonical sentence is \"The three constants are the reference implementation's public names for these limits\" (schemas/README.md:64-65). Adding MAX_INDEX_ENTRIES / MAX_INDEX_DEPTH and their rejections to that section leaves \"the three constants\" naming three of five limits, so the claim stays true only if that sentence is amended — and the design nowhere says it is (Rollout 1 and the Files created cell record only the addition).","fix":"Name the edit: record in Rollout 1 and the Files created cell that the guard section's \"three constants\" sentence becomes a five-constant sentence."},{"severity":"nit","location":"Chosen design → Profiles are chains → Resolution, API bullet: \"`for` is required at parse and a `for` naming no manifest document in the index is `UnresolvedAncestor`\"","problem":"As written this reads as making `for` a required entry, which contradicts the same section's \"`profile/v1` entry set grows by **optional** entries only, so the shipped valid fixtures stay valid (no narrowing)\", its \"absent = a profile not bound to a game\" (the ADR's accessibility and alternative-controller case), and Rollout's Defaults line \"`for` absent = not bound to a game\". The rule actually being added is the parse-time one from §3's entry list (`seeded-from` requires `for`).","fix":"Write \"`seeded-from` requires `for` at parse\" (or \"`for`'s grammar is checked at parse\"), leaving `for` optional."}]}
```
