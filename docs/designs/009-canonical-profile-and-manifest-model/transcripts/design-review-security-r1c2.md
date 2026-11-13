# Transcript: design-review:security r1c2

## Provenance
- stage: design-review:security (round r1c2)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:00:17.543Z
- ended: 2026-11-11T23:04:31.951Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 7b2ddc6d10df95a81e738c734fe5c9673d530181 5bbcc09d65524a3e2c522e58501451480f23d8ee` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design → Transforms produce bases / Adversarial posture / Test strategy #1 / Rollout 3 — \"Each invocation must match its registry entry's declared arity and the grammars of its arguments (hand ∈ `left|right`, a placement id, a trailing-`*` action pattern); a mismatch is the shipped `InvalidValue { key, message }`, one `fixtures/invalid/` case per shape\"",
    "problem": "The invocation guard my r1 finding asked for is fully specified in content (every shape enumerated, shipped InvalidValue reused, one fixture per shape, asserted \"before `apply_policy` runs\") but its enforcing component is unnamed, and the two candidate crates cannot both satisfy it as written. `schemas/policy/v1/spec.md` is \"parsed by `bindforge-core`, because the harness requires a reader for every format directory\", and Test #1 requires \"each `fixtures/invalid/` case is rejected with the pinned `CoreError` variant\" — which the shipped harness performs through core's reader (crates/core/tests/conformance.rs:138-150, :223-232). So the arity/grammar fixtures must be refused by core. Yet the knowledge needed to refuse them (transform names, arity, per-argument grammars) is stated to live only in bindforge-transforms' registry (§4 \"The v1 registry holds exactly\"; Rollout 3 \"the registry — each entry carrying its declared arity and argument grammars\"), and policy/v1/spec.md is never told t… [elided by the checkpoint cap]",
    "fix": "State the closed invocation table (transform name, arity, per-argument grammar) in schemas/policy/v1/spec.md as part of the format contract, enforced by core's policy reader, and add one clause that bindforge-transforms' registry carries exactly that table, with a test asserting registry == spec table. One canonical statement, no new CoreError variant, no new mechanism; Rollout 1 lists the table as spec content."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains (index walk) / Manifests state game facts — \"`data/manifests/` gets the same treatment keyed on each document's declared `game` + `revision`, so a `for` pin is an index lookup and no path is ever built from a document string\"",
    "problem": "The new manifest index's key is declared content, and no collision rule is named for it. Profile ids have DuplicateId; a manifest carries no `id` (entry set is format, version, game, revision, …), so nothing rejects two documents under data/manifests/ declaring one game + revision. A `for` pin then resolves to one of them by walk order — exactly the manifest the drift gate (contract 5), the axis-kind cross-check, and the seeded-from check run against. A shadowing manifest is a determinism and pin-integrity gap, and the fail-closed checklist item wants a rejection, not an iteration order.",
    "fix": "One clause: the manifest index rejects two documents declaring one `game` + `revision` as DuplicateId on that declared key (no new variant), and Test strategy #9 adds a shadowing-manifest case beside \"a valid pin resolves to a manifest by index lookup on declared `game` + `revision`\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains — \"`generated-by` (an id in the `<namespace>:<part>` shape … a value outside the shape is `InvalidValue`, so a shared profile publishes no author name, handle, or email)\"",
    "problem": "The mechanism adopted my r1 fix and free-text attribution is gone, but the justification added with it is stronger than the shape enforces. generated-by's namespace is explicitly open (\"there the namespace is a producer label, never an input namespace\") and `<part>` is any lowercase hyphen-joined token, so `author:alice-smith`, `mike:handle`, `studio:alicegmail-com` are in-shape and InvalidValue cannot refuse them. That is the \"what leaves the machine when bindings are shared\" case, and a profile is the artifact that travels, so the absolute claim in the revised sentence is false as written.",
    "fix": "Make the producer namespace a closed list in profile/v1/spec.md (e.g. `policy|preset|generator`), mirroring the closed input-namespace set, and soften the sentence to what the shape enforces (\"publishes no free-text attribution\"); ADR-014 keeps identity bound to a key (Deferred, A22)."
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
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/security-r1c2.md. That file is the only path this session
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "Chosen design → Profiles are chains (Resolution, `resolve` bullet) / Profiles are chains → Id → file / Test strategy #9 — \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare, naming the profile id … `for` is required at parse, so the check always has a manifest.\"", "problem": "The new clause is true of the pin, not of the manifest. `for` required at parse guarantees a well-formed `game` + `revision` string; the manifest itself is found by index lookup, and no rejection is named for a lookup miss. `UnresolvedAncestor` is scoped to \"a `derives` entry with no document\", `BadPin` is a grammar failure, and Test #9's pin cases cover only malformed halves (`BadPin`) and a valid lookup. A profile pinning a deleted, renamed, or never-authored manifest revision is an unruled state, and `Resolved { manifest: Option<ManifestPin> }` offers the silent reading — proceed with `manifest: None`, preset check skipped — which re-opens one level up the vacuously-green condition the r2 revision says it closed (\"unpinned, the check was vacuously green\"), and lets the drift gate's \"resolves every profile … against the manifest revision it pins\" pass on a pin nothing satisfies.", "fix": "One clause in §3: a `for` pin with no manifest document in the index is `UnresolvedAncestor`, that variant's rule stated once as \"a document reference with no document\" (no new `CoreError` variant, no new mechanism; the choice cannot be left to implementation because the variant list is the shipped compatibility promise), plus one Test #9 case beside the valid-pin lookup: a well-formed pin naming no manifest is rejected, so `resolve`'s `seeded-from` check and the drift gate are never vacuous."}, {"severity": "minor", "location": "Chosen design → Profiles are chains → Id → file — \"`crates/core/src/index.rs` walks `data/profiles/` and parses every document, indexing by its **declared** `id`; `data/manifests/` gets the same treatment keyed on each document's declared `game` + `revision`\"", "problem": "The keying presumes every document in a walked tree declares the format that tree holds, and no such rule appears anywhere in the proposal. Readers dispatch on the document's own `format` (the shipped `UnknownFormat` covers only formats the crate does not parse), so a document under `data/manifests/` declaring `format = \"policy\"`, or one under `data/profiles/` declaring `format = \"manifest\"`, parses cleanly and reaches the index with the key entry the tree needs absent (`id`, or `game` + `revision`). The behavior on hostile or misfiled content is left to the implementer — the same fail-closed family as the manifest-key collision closed this round.", "fix": "One clause: each walked tree accepts only the format(s) it holds (`data/profiles/` → `profile`/`patch`, `data/manifests/` → `manifest`), a document declaring another is refused at index time; add a Test #9 case planting a manifest-format document in `data/profiles/`."}, {"severity": "nit", "location": "Chosen design → Adversarial posture — \"New `CoreError` variants, each pinned by a fixture in this slice except `MergeConflict` … and the two index limits, pinned by Test strategy #1's synthetic walk, not by a fixture document: … `DuplicateDelta`, `DuplicateId`, …\"", "problem": "The exception list is one variant short. `DuplicateId` is an index-level, two-document rejection, and the shipped harness asserts per fixture file (crates/core/tests/conformance.rs:138-154), so no single `fixtures/invalid/` document can pin it; Test strategy #9 is what pins it, as the two sentences rewritten this round already acknowledge for `MergeConflict` and the index limits.", "fix": "Add `DuplicateId` to the same exception clause, naming Test strategy #9 as its pin."}]}
```
