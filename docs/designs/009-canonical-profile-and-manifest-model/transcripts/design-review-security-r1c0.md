# Transcript: design-review:security r1c0

## Provenance
- stage: design-review:security (round r1c0)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:33:55.058Z
- ended: 2026-11-11T22:39:56.570Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/009-canonical-profile-and-manifest-model.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/security-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → Manifests state game facts / Profiles are chains — \"content `data/manifests/<game>/manifest.toml` (fixture-name rule for `<game>`)\" and \"its halves validated against the game-id grammar and a dotted `<major>.<minor>` revision, else `BadPin`\" and \"Ids never appear in a path\"","problem":"The canonical path rule is scoped to profile ids (the id→file index over `data/profiles/`), and manifest resolution is left unspecified while the manifest layout puts a game id in a path component. The drift mechanism resolves a pin to a manifest with no stated lookup path, so the natural implementation joins the pin's game half — a string from a shared, untrusted profile — into `data/manifests/{game}/manifest.toml`, the path-interpolation class A18 exists to stop. The check the design leans on to prevent that is \"the game-id grammar\", which is defined nowhere: Encoding rules state only the taxonomy-id and input-id shapes, so BadPin's acceptance set cannot be tested. This does not re-open the Revision log's rejected charset narrowing of profile ids; the escape-carrying id fixture stays valid.","fix":"Name the game-id grammar as the shipped fixture-name rule (lowercase ASCII words joined by `-`; never a dot, `/`, `\\`, or leading `.`) in Encoding rules, and state in the Manifests section that a pin resolves through the same bounded walked index over `data/manifests/`, keyed by the document's declared `game` and `revision` — no path is built from a pin. Add a `..` pin and an absolute-path pin to Test strategy #10."},{"severity":"major","location":"Chosen design → Transforms produce bases — \"each is a non-empty array of strings, first element the transform name\"; Test strategy #11 \"an unknown transform name is rejected\"","problem":"The policy `stack` is validated by transform name only. Nothing pins invocation arity or argument grammar, so `[\"relocate-cluster\", \"move.steer-*\"]` (missing hand and placement id), `[\"swap-mouse-buttons\", \"extra\"]`, a hand outside `left|right`, a non-placement-shaped placement id, and a `*` in a non-trailing position all reach the registry, where reading `invocation[1]`/`invocation[2]` panics — against the shipped contract that every refusal is a typed variant, \"never a panic, never a bare string\", for documents that are untrusted at every entry point.","fix":"State in `policy/v1/spec.md` that each invocation must match its registry entry's declared arity and the grammars of its arguments (hand ∈ `left|right`, placement id, trailing-`*` action pattern), rejected as the shipped `InvalidValue { key, message }` (no new variant), one `fixtures/invalid/` case per shape, and extend Test strategy #11 to assert each is rejected before `apply_policy` runs."},{"severity":"minor","location":"Chosen design → Profiles are chains (index walk) — \"walks `data/profiles/` (bounded depth and entry count, each file size-checked by the shipped budget)\"","problem":"The shipped budget check is metadata-based (`fs::metadata(path).len()` compared to `MAX_DOCUMENT_BYTES`, then read), and metadata follows symlinks. A tree entry that is a symlink to `/dev/zero` (metadata length 0) or a FIFO passes the pre-read cap and then reads unbounded, so the walk's size guard fails open as a hang/OOM rather than rejecting cleanly. Here the walked tree is reviewed repo content, so this hardens the guard the model contributes to the hub path rather than fixing a live exposure.","fix":"One clause: the walk takes a directory entry's own type (`DirEntry::file_type().is_file()`, symlinks not followed) and rejects non-regular entries, and the cap applies to bytes read from the opened file (the `read_reader` take-N shape), not to metadata length."},{"severity":"minor","location":"Chosen design → Profiles are chains — \"`generated-by` (free-form string id, e.g. `policy:lefty-mouse-relocate`)\"","problem":"`generated-by` is the only profile field with no grammar, and a profile is the artifact that leaves the machine when bindings are shared, so free-form is where an author's name, handle, or email gets published in a shared/hub listing. The phrase itself is self-contradictory (free-form vs id).","fix":"Constrain the value to the existing `<namespace>:<part>` shape the example already follows, state it in `profile/v1/spec.md`, and reject a non-id-shaped value as `InvalidValue`."},{"severity":"nit","location":"Chosen design → Encoding rules / Transforms — \"`*` appears only in a policy action pattern, as a trailing suffix (`move.steer-*`)\"","problem":"A bare `*` is a trailing suffix on an empty prefix, so a shared policy can address the entire taxonomy; only each transform's precondition limits the blast radius, and a pattern that matches nothing is a quiet no-op.","fix":"Require a non-empty literal prefix before `*`, and record a zero-match pattern as a named `Skipped` entry so an adopter sees the policy did nothing."}]}
```
