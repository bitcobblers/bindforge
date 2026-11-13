# Transcript: design-review:compatibility:attempt2 r1c0

## Provenance
- stage: design-review:compatibility:attempt2 (round r1c0)
- agent: review/design-compatibility
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:40:18.877Z
- ended: 2026-11-11T22:43:54.407Z
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
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/009-canonical-profile-and-manifest-model.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/compatibility-r1c0.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Previous attempt's gate failure (log excerpt): independence violation: read a review report: /home/mike/repos/bindforge/docs/designs/009-canonical-profile-and-manifest-model/reviews/desig….
Do not repeat the behavior that caused that failure.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"request-changes","findings":[{"severity":"major","location":"Compatibility matrix — \"Hub upload path (wire budget, server-side chain pre-resolution) | unknown\"","problem":"The row's status is `unknown`, so the matrix is not a decidable support record; the checklist requires every platform row to carry a status and treats `unknown` as a finding. This is matrix vocabulary only — the ADR-014 hub deferral itself is consistent with the ADR's \"Not decided here: … the hub service and voting\" and is not re-opened.","fix":"Change that status cell to `no (v1) — out of scope` and keep the existing note (\"ADR-014 owns it (A23); the client-side guards and the id-index rule are the model's contribution\"). One-cell change, no new mechanism."},{"severity":"major","location":"Chosen design → Encoding rules → Round-trip split (with Goals \"machine-checkable by the existing harness with no harness change\", Test strategy #1 \"`taxonomy`, `reach-classes`, `placement` are asserted by load\", and Test strategy #14 \"requires no change to `crates/core/tests/conformance.rs`\") — \"pure lookup data (`taxonomy`, `reach-classes`, `placement`) is asserted by load, using the exemption `reach-classes/v1/spec.md` already states\"","problem":"That exemption is not a harness policy keyed on format kind: it is a hardcoded format-name equality test at crates/core/tests/conformance.rs:63 (`if dir.format == \"reach-classes\"`). Every other format directory's fixtures/valid/ is subjected to parse → serialize → re-parse → equality. So load-asserting `taxonomy` and `placement` requires editing conformance.rs, which contradicts Goals' \"no harness change\" and Test strategy #14's \"no change to conformance.rs\"; the three claims cannot all hold. A8's cited basis (conformance.rs:322-335) supports directory discovery only, not the exemption.","fix":"Make all six formats round-trip (add serialize arms in Rollout 2's `Document` extension) and reserve the load-assert exemption for `reach-classes`, whose loader consumes a per-hand directory pair rather than one document. Replace \"taxonomy, reach-classes, placement are asserted by load\" in Test strategy #1 and the quoted sentence in Round-trip split accordingly. (If the exemption is widened instead, Goals' \"no harness change\" and Test strategy #14 must name that harness edit; round-tripping all six is the smaller change and keeps the harness frozen.)"},{"severity":"major","location":"Chosen design → Profiles are chains §3 + Manifests §5 — \"its halves validated against the game-id grammar\" and \"content `data/manifests/<game>/manifest.toml` (fixture-name rule for `<game>`)\"","problem":"The game-id grammar is the only gate between an untrusted `for` pin and a filesystem path component, and it is never defined: Encoding rules enumerate exactly two id shapes (taxonomy ids, input ids) and the quoted-key id list omits game ids. The rule leaned on exists only as a test helper, crates/core/tests/common/mod.rs:352 `is_fixture_name` (which strips a `.toml` suffix), so no shipped `crates/core` code can enforce it and `BadPin` has no specified predicate. This leaves open the same path-traversal class the design closes for profile ids under A18 (\"Ids never appear in a path\").","fix":"Add the game-id shape to the canonical Encoding rules as `[a-z0-9-]+` (lowercase, no dots, no `@` — the fixture-name stem rule), and state that manifest resolution looks the pin's game half up in a walked index keyed on each manifest's declared `game` entry, replacing construction of `data/manifests/<game>/manifest.toml` from the pin."},{"severity":"minor","location":"Chosen design → Input vocabulary — \"it arrives as a *new* prefix (`pos:<part>`), leaving `keyboard:` meaning 'US-ANSI glyph' permanently\"","problem":"The canonical grammar closes the namespace set (`<namespace>` ∈ `keyboard|mouse|gamepad`), so minting `pos` is a widening of a closed set in every format that carries input ids (profile, patch, manifest's `default:`/`dead-keys`, policy), not only in the reach tables. Deferred names only a `reach-classes/v2`, and the ADR's \"the id shape reserves it\" covers shape, not the version gate, so implementers must guess whether a v1 reader accepts `pos:a`.","fix":"One sentence in Input vocabulary (canonical): `pos` joins the `<namespace>` set only in a v2 of every format that carries input ids, and a v1 document naming a `pos:` input is `UnknownInput`."},{"severity":"minor","location":"Chosen design → Profiles are chains §3 — \"File names follow the shipped fixture-name rule (lowercase ASCII, `-`-joined)\"","problem":"The shipped rule as implemented requires a `.toml` suffix (is_fixture_name returns false for any other name), while profile content is `data/profiles/**/*.bindprofile` per this same section and ADR-009 Decision 3. The cited rule cannot be satisfied by the files it governs, and the Windows matrix row leans on it for its case-uniqueness claim.","fix":"Cite the stem rule — \"lowercase ASCII, `-`-joined stem\" — and state the extension separately: `.bindprofile` for profile/patch content, `.toml` for `schemas/` fixtures."},{"severity":"minor","location":"Chosen design → Input vocabulary + Files created by this design — \"`schemas/README.md` lists the keyboard names v1 admits\" (README row: \"format list, the canonical encoding rules, `version`/`revision` split\")","problem":"README is designated the single canonical enumeration of the v1 keyboard surface, and both the authoring bar (\"both hands graded over the full admitted keyboard name set\") and Test strategy #3 (\"every admitted keyboard name is graded in both hands\") iterate it, but the inventory row for README and Rollout 1 omit that list, and schemas/README.md contains no key-name enumeration today. The set the US-ANSI support claim is tested against has no named source.","fix":"Add the admitted keyboard-name list to that Files-created row's contents and name it as the enumeration Test strategy #3 iterates."}]}
```
