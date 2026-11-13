# Transcript: design-review:security r1c3

## Provenance
- stage: design-review:security (round r1c3)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:08:06.078Z
- ended: 2026-11-11T23:10:45.213Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 5bbcc09d65524a3e2c522e58501451480f23d8ee f7521536a6bca93f4f9b4d4a4cfce3446567cdc7` (both are blob ids of 02-proposal.md), then read the full current document.
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
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Profiles are chains (Resolution, `resolve` bullet) / Profiles are chains → Id → file / Test strategy #9 — \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare, naming the profile id … `for` is required at parse, so the check always has a manifest.\"",
    "problem": "The new clause is true of the pin, not of the manifest. `for` required at parse guarantees a well-formed `game` + `revision` string; the manifest itself is found by index lookup, and no rejection is named for a lookup miss. `UnresolvedAncestor` is scoped to \"a `derives` entry with no document\", `BadPin` is a grammar failure, and Test #9's pin cases cover only malformed halves (`BadPin`) and a valid lookup. A profile pinning a deleted, renamed, or never-authored manifest revision is an unruled state, and `Resolved { manifest: Option<ManifestPin> }` offers the silent reading — proceed with `manifest: None`, preset check skipped — which re-opens one level up the vacuously-green condition the r2 revision says it closed (\"unpinned, the check was vacuously green\"), and lets the drift gate's \"resolves every profile … against the manifest revision it pins\" pass on a pin nothing satisfies.",
    "fix": "One clause in §3: a `for` pin with no manifest document in the index is `UnresolvedAncestor`, that variant's rule stated once as \"a document reference with no document\" (no new `CoreError` variant, no new mechanism; the choice cannot be left to implementation because the variant list is the shipped compatibility promise), plus one Test #9 case beside the valid-pin lookup: a well-formed pin naming no manifest is rejected, so `resolve`'s `seeded-from` check and the drift gate are never vacuous."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains → Id → file — \"`crates/core/src/index.rs` walks `data/profiles/` and parses every document, indexing by its **declared** `id`; `data/manifests/` gets the same treatment keyed on each document's declared `game` + `revision`\"",
    "problem": "The keying presumes every document in a walked tree declares the format that tree holds, and no such rule appears anywhere in the proposal. Readers dispatch on the document's own `format` (the shipped `UnknownFormat` covers only formats the crate does not parse), so a document under `data/manifests/` declaring `format = \"policy\"`, or one under `data/profiles/` declaring `format = \"manifest\"`, parses cleanly and reaches the index with the key entry the tree needs absent (`id`, or `game` + `revision`). The behavior on hostile or misfiled content is left to the implementer — the same fail-closed family as the manifest-key collision closed this round.",
    "fix": "One clause: each walked tree accepts only the format(s) it holds (`data/profiles/` → `profile`/`patch`, `data/manifests/` → `manifest`), a document declaring another is refused at index time; add a Test #9 case planting a manifest-format document in `data/profiles/`."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Adversarial posture — \"New `CoreError` variants, each pinned by a fixture in this slice except `MergeConflict` … and the two index limits, pinned by Test strategy #1's synthetic walk, not by a fixture document: … `DuplicateDelta`, `DuplicateId`, …\"",
    "problem": "The exception list is one variant short. `DuplicateId` is an index-level, two-document rejection, and the shipped harness asserts per fixture file (crates/core/tests/conformance.rs:138-154), so no single `fixtures/invalid/` document can pin it; Test strategy #9 is what pins it, as the two sentences rewritten this round already acknowledge for `MergeConflict` and the index limits.",
    "fix": "Add `DuplicateId` to the same exception clause, naming Test strategy #9 as its pin."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/security-r1c3.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Profiles are chains → Resolution (`resolve` bullet) — \"`Resolved { bindings: …, chain: Vec<String>, manifest: Option<ManifestPin> }`, whose `manifest` is `Some` whenever the profile declares `for`\" / \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare\"","problem":"The r3 closure made the pin's existence total (a dangling `for` is `UnresolvedAncestor`) but left *which* manifest a chain pins unruled, so the check can run against the wrong manifest. `for` is an optional entry on every chain document (§3: `patch/v1` \"carries the same optional chain entries\"), while a chain has one `manifest` field and one \"pinned manifest\" (§5: \"resolves every profile … against the manifest revision it pins\"). A shared chain whose base pins `remnant2@0.1` and whose tip pins an existing `remnant2@0.2` is a fully valid-document state with no stated rule: `resolve(root, 1)` (base alone) and `resolve(root, chain.len())` then evaluate `seeded-from` against different manifests, and the drift gate's verdict for one profile id becomes implementation/chain-order dependent — the same \"walk-order dependent\" determinism loss the adjacent `DuplicateId` clause exists to prevent, on the mechanism the design names as the only trust mechanism (validation, not provenance). This is the residue of my r1 major at the same location, not the vacuity it described.","fix":"One clause in §3: the pin is a chain fact — a chain's documents declare one `for`, and a document whose `for` differs from the chain's is `InvalidValue` (no new `CoreError` variant, no new mechanism) — plus one Test #9 case beside the existing pin lookups, so `Resolved.manifest` has exactly one referent."},{"severity":"nit","location":"Chosen design → Profiles are chains → Id → file — \"Each tree accepts only the formats it holds — `data/profiles/` `profile`/`patch`, `data/manifests/` `manifest`\"","problem":"The new format gate is stated as \"each tree\" but enumerates only the two indexed trees, while the design ships four walked `data/` trees: `data/placements/*.toml` is loaded content (Test #3, #4) and `data/policies/` carries the policy content Test #11 applies, and `schemas/README.md`'s data-homes rule says all of `data/` \"holds content discovered by walking\". A misfiled document in those two trees is again dispatched on the document's own declared `format`, with no gate named — the fail-closed family this clause closed for the indexed trees.","fix":"Extend the parenthetical to `data/placements/` → `placement` and `data/policies/` → `policy`, same shipped `InvalidValue` at load; no new mechanism, and Test #9's planted-document sentence already covers the cases."},{"severity":"nit","location":"Test strategy #8 (with Adversarial posture) — \"a manifest naming an unknown action id, an axis kind that disagrees with the taxonomy (`AxisKindConflict`), … is rejected, each with its own fixture\"","problem":"The r3 pinning rule says such conditions are not carried by a `fixtures/invalid/` document (parse cannot see the taxonomy, and the shipped harness asserts that every file under `fixtures/invalid/` is rejected through `parse` — crates/core/tests/conformance.rs:138-154), yet #8's untouched \"each with its own fixture\" reads as if they live there. A reader who places #8's cases under `schemas/manifest/v1/fixtures/invalid/` makes the harness walk fail on documents that parse cleanly, instead of exercising the `validate` check intended.","fix":"One phrase in #8: its cases are data under the model suite (as the posture's \"over fixture data\" already says), never under `schemas/*/v1/fixtures/invalid/`."}]}
```
