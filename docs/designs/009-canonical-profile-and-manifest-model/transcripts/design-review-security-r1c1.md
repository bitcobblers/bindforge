# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:49:38.641Z
- ended: 2026-11-11T22:55:28.531Z
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
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Manifests state game facts / Profiles are chains — \"content `data/manifests/<game>/manifest.toml` (fixture-name rule for `<game>`)\" and \"its halves validated against the game-id grammar and a dotted `<major>.<minor>` revision, else `BadPin`\" and \"Ids never appear in a path\"",
    "problem": "The canonical path rule is scoped to profile ids (the id→file index over `data/profiles/`), and manifest resolution is left unspecified while the manifest layout puts a game id in a path component. The drift mechanism resolves a pin to a manifest with no stated lookup path, so the natural implementation joins the pin's game half — a string from a shared, untrusted profile — into `data/manifests/{game}/manifest.toml`, the path-interpolation class A18 exists to stop. The check the design leans on to prevent that is \"the game-id grammar\", which is defined nowhere: Encoding rules state only the taxonomy-id and input-id shapes, so BadPin's acceptance set cannot be tested. This does not re-open the Revision log's rejected charset narrowing of profile ids; the escape-carrying id fixture stays valid.",
    "fix": "Name the game-id grammar as the shipped fixture-name rule (lowercase ASCII words joined by `-`; never a dot, `/`, `\\`, or leading `.`) in Encoding rules, and state in the Manifests section that a pin resolves through the same bounded walked index over `data/manifests/`, keyed by the document's declared `game` and `revision` — no path is built from a pin. Add a `..` pin and an absolute-path pin to Test strategy #10."
  },
  {
    "severity": "major",
    "location": "Chosen design → Transforms produce bases — \"each is a non-empty array of strings, first element the transform name\"; Test strategy #11 \"an unknown transform name is rejected\"",
    "problem": "The policy `stack` is validated by transform name only. Nothing pins invocation arity or argument grammar, so `[\"relocate-cluster\", \"move.steer-*\"]` (missing hand and placement id), `[\"swap-mouse-buttons\", \"extra\"]`, a hand outside `left|right`, a non-placement-shaped placement id, and a `*` in a non-trailing position all reach the registry, where reading `invocation[1]`/`invocation[2]` panics — against the shipped contract that every refusal is a typed variant, \"never a panic, never a bare string\", for documents that are untrusted at every entry point.",
    "fix": "State in `policy/v1/spec.md` that each invocation must match its registry entry's declared arity and the grammars of its arguments (hand ∈ `left|right`, placement id, trailing-`*` action pattern), rejected as the shipped `InvalidValue { key, message }` (no new variant), one `fixtures/invalid/` case per shape, and extend Test strategy #11 to assert each is rejected before `apply_policy` runs."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains (index walk) — \"walks `data/profiles/` (bounded depth and entry count, each file size-checked by the shipped budget)\"",
    "problem": "The shipped budget check is metadata-based (`fs::metadata(path).len()` compared to `MAX_DOCUMENT_BYTES`, then read), and metadata follows symlinks. A tree entry that is a symlink to `/dev/zero` (metadata length 0) or a FIFO passes the pre-read cap and then reads unbounded, so the walk's size guard fails open as a hang/OOM rather than rejecting cleanly. Here the walked tree is reviewed repo content, so this hardens the guard the model contributes to the hub path rather than fixing a live exposure.",
    "fix": "One clause: the walk takes a directory entry's own type (`DirEntry::file_type().is_file()`, symlinks not followed) and rejects non-regular entries, and the cap applies to bytes read from the opened file (the `read_reader` take-N shape), not to metadata length."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains — \"`generated-by` (free-form string id, e.g. `policy:lefty-mouse-relocate`)\"",
    "problem": "`generated-by` is the only profile field with no grammar, and a profile is the artifact that leaves the machine when bindings are shared, so free-form is where an author's name, handle, or email gets published in a shared/hub listing. The phrase itself is self-contradictory (free-form vs id).",
    "fix": "Constrain the value to the existing `<namespace>:<part>` shape the example already follows, state it in `profile/v1/spec.md`, and reject a non-id-shaped value as `InvalidValue`."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Encoding rules / Transforms — \"`*` appears only in a policy action pattern, as a trailing suffix (`move.steer-*`)\"",
    "problem": "A bare `*` is a trailing suffix on an empty prefix, so a shared policy can address the entire taxonomy; only each transform's precondition limits the blast radius, and a pattern that matches nothing is a quiet no-op.",
    "fix": "Require a non-empty literal prefix before `*`, and record a zero-match pattern as a named `Skipped` entry so an adopter sees the policy did nothing."
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
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → Transforms produce bases / Adversarial posture / Test strategy #1 / Rollout 3 — \"Each invocation must match its registry entry's declared arity and the grammars of its arguments (hand ∈ `left|right`, a placement id, a trailing-`*` action pattern); a mismatch is the shipped `InvalidValue { key, message }`, one `fixtures/invalid/` case per shape\"","problem":"The invocation guard my r1 finding asked for is fully specified in content (every shape enumerated, shipped InvalidValue reused, one fixture per shape, asserted \"before `apply_policy` runs\") but its enforcing component is unnamed, and the two candidate crates cannot both satisfy it as written. `schemas/policy/v1/spec.md` is \"parsed by `bindforge-core`, because the harness requires a reader for every format directory\", and Test #1 requires \"each `fixtures/invalid/` case is rejected with the pinned `CoreError` variant\" — which the shipped harness performs through core's reader (crates/core/tests/conformance.rs:138-150, :223-232). So the arity/grammar fixtures must be refused by core. Yet the knowledge needed to refuse them (transform names, arity, per-argument grammars) is stated to live only in bindforge-transforms' registry (§4 \"The v1 registry holds exactly\"; Rollout 3 \"the registry — each entry carrying its declared arity and argument grammars\"), and policy/v1/spec.md is never told to carry that table, so core's reader gains an undeclared dependency on a table in a crate that depends on it. Outcome as written: the arity fixtures are untestable through the harness path (Test #1 cannot pass), or the table is duplicated in two crates and drifts. For this checklist, the fail-closed boundary for untrusted policy documents has no stated owner.","fix":"State the closed invocation table (transform name, arity, per-argument grammar) in schemas/policy/v1/spec.md as part of the format contract, enforced by core's policy reader, and add one clause that bindforge-transforms' registry carries exactly that table, with a test asserting registry == spec table. One canonical statement, no new CoreError variant, no new mechanism; Rollout 1 lists the table as spec content."},{"severity":"minor","location":"Chosen design → Profiles are chains (index walk) / Manifests state game facts — \"`data/manifests/` gets the same treatment keyed on each document's declared `game` + `revision`, so a `for` pin is an index lookup and no path is ever built from a document string\"","problem":"The new manifest index's key is declared content, and no collision rule is named for it. Profile ids have DuplicateId; a manifest carries no `id` (entry set is format, version, game, revision, …), so nothing rejects two documents under data/manifests/ declaring one game + revision. A `for` pin then resolves to one of them by walk order — exactly the manifest the drift gate (contract 5), the axis-kind cross-check, and the seeded-from check run against. A shadowing manifest is a determinism and pin-integrity gap, and the fail-closed checklist item wants a rejection, not an iteration order.","fix":"One clause: the manifest index rejects two documents declaring one `game` + `revision` as DuplicateId on that declared key (no new variant), and Test strategy #9 adds a shadowing-manifest case beside \"a valid pin resolves to a manifest by index lookup on declared `game` + `revision`\"."},{"severity":"minor","location":"Chosen design → Profiles are chains — \"`generated-by` (an id in the `<namespace>:<part>` shape … a value outside the shape is `InvalidValue`, so a shared profile publishes no author name, handle, or email)\"","problem":"The mechanism adopted my r1 fix and free-text attribution is gone, but the justification added with it is stronger than the shape enforces. generated-by's namespace is explicitly open (\"there the namespace is a producer label, never an input namespace\") and `<part>` is any lowercase hyphen-joined token, so `author:alice-smith`, `mike:handle`, `studio:alicegmail-com` are in-shape and InvalidValue cannot refuse them. That is the \"what leaves the machine when bindings are shared\" case, and a profile is the artifact that travels, so the absolute claim in the revised sentence is false as written.","fix":"Make the producer namespace a closed list in profile/v1/spec.md (e.g. `policy|preset|generator`), mirroring the closed input-namespace set, and soften the sentence to what the shape enforces (\"publishes no free-text attribution\"); ADR-014 keeps identity bound to a key (Deferred, A22)."}]}
```
