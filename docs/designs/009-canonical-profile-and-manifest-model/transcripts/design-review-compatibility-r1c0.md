# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:33:55.058Z
- ended: 2026-11-11T22:40:18.836Z
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

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → 5. Manifests state game facts / Encoding rules (canonical) — \"content `data/manifests/<game>/manifest.toml` (fixture-name rule for `<game>`)\" and \"The `cargo test` gate resolves every profile in `data/profiles/` against the manifest revision it pins\"","problem":"Manifests are the only cross-document reference the design leaves path-shaped. Profiles got a walked index and the canonical rule \"Ids never appear in a path\", which the Encoding rules state for every format; a manifest has no index, so the sole location given is a directory component derived from the profile's `for = \"<game>@<revision>\"` pin — interpolating an untrusted document's field into a filesystem path, the class A18 forbids. The named guard, \"its halves validated against the game-id grammar\", names a grammar the proposal never defines (the canonical rules enumerate exactly \"Two id shapes\": taxonomy ids and input ids). Same missing text has a second consequence: the pin carries a `revision`, the store is one file per game, and Test strategy #13 expects a revision bump to fail \"for every profile that pins it\", so no rule says how a pin naming a superseded revision resolves for a published profile — v1 has no revision-addressable manifest lookup, which is the ADR's \"detecting binding drift per manifest version\" promise from the compatibility side.","fix":"In `manifest/v1/spec.md`, state the resolution rule: walk `data/manifests/` and string-compare each document's declared `game` and `revision` (index lookup, no id in any path); add `PinNotSatisfied` (pinned by a fixture) when a pin's revision matches no shipped document's `revision`. Add the third id shape to the canonical Encoding rules: `game`, `codec-family` and preset names are `[a-z0-9-]+`, the fixture-name grammar. Keep `data/manifests/<game>/manifest.toml` as an authoring convention that load validates, not a lookup."},{"severity":"major","location":"Compatibility matrix — \"| Hub upload path (wire budget, server-side chain pre-resolution) | unknown |\"","problem":"Checklist item 1: `unknown` in a matrix this design uses as its compatibility statement is a finding. The row also mixes the ADR-014 deferral with the model's own contribution, and the model's half is already answered and tested (the id-index rule, Test strategy #10), so `unknown` understates it.","fix":"Split the row: \"Model contribution to the upload path (id-index rule, client-side guards) | yes | Test strategy #10; the shipped budget/closed-entry/bounded-message guards\", and move the wire-budget and server-side pre-resolution wording into the Deferred bullet that already owns it (ADR-014), leaving no row with `unknown`."},{"severity":"major","location":"Chosen design → 5. Manifests state game facts / Compatibility matrix — \"`action:<id>` = the game-native action name (the join)\"","problem":"The manifest is the first format whose values are human, game-authored strings, and for a localized game those strings are non-ASCII (accents, Cyrillic, CJK). The design pins its value grammar only to A2 (\"values are basic strings…\") and its keyboard rows to US-ANSI glyph naming, so the non-ASCII case is stated nowhere, fixture-covered nowhere, and has no matrix row — while the manifest is in the round-trip set A8 grades (\"`profile`, `patch`, `manifest` re-serialize to an equal value\"), making byte-stable re-serialization of non-ASCII basic strings a load-bearing property of the shipped grammar that the Options claim of adding \"nothing to\" it papers over. Checklist item 5 (non-ASCII binding names).","fix":"One sentence in `manifest/v1/spec.md` and the Encoding rules: string values are full TOML basic strings, non-ASCII codepoints and `\\uXXXX` escapes admitted, with byte-stable re-serialization pinned by the fixture corpus. Add one valid `manifest/v1` fixture with a non-ASCII `action:<id>` value, and one matrix row: \"Localized (non-ASCII) game-native action names | yes (v1)\"."},{"severity":"minor","location":"Compatibility matrix — \"| Windows 10/11 contributor machine | yes | exercised leg (Test strategy #14): TOML is case-sensitive…\"","problem":"The platform leg is Test strategy #15 (\"Platform: all of the above is green on the Linux CI runner and on a Windows contributor machine\"); #14 is \"harness pickup and the version gate\". Checklist item 7 checks the matrix-to-test mapping, and Rollout #7 cites the same leg as #15, so the matrix is the inconsistent citation.","fix":"Cite Test strategy #15 in that row."},{"severity":"minor","location":"Chosen design → 5. Manifests / 2. Input vocabulary — \"`dead-keys` (inputs the game ignores)\" with `dead-keys = [\"keyboard:caps-lock\"]`","problem":"In keyboard terminology a dead key is a composing key (US-International apostrophe, ISO accents) — a key that yields no character alone. Readers on the layouts the matrix's layout rows speak about read this field in that sense, and the design's actual composing-key case (an admitted id such as `keyboard:apostrophe` being a dead key on a non-US layout) is addressed nowhere. Checklist item 5 (dead keys).","fix":"Keep the ADR-009 field name; add one clause to `manifest/v1/spec.md` and the input-vocabulary paragraph: \"dead-keys = inputs the game ignores; layout composing (dead) keys are out of v1 scope and arrive with the layout axis, see Deferred\"."},{"severity":"minor","location":"Compatibility matrix — no IME row","problem":"The matrix enumerates the geometry axis, symbol-layout axis, gamepad, mouse and config-file rows, and Deferred closes layout, gamepad, mouse and accessibility devices; IME composition appears in no row, no non-goal and no Deferred bullet, so the abstention reads as an omission rather than a boundary. Checklist item 5 (IME).","fix":"Add one row: \"IME composition (CJK and other input-method layouts) | n/a (v1) | this design names inputs and has no input-event surface (Non-goals); a key id names the US-ANSI key, and the character a composing session yields is a capture concern (ADR-012)\"."},{"severity":"minor","location":"Chosen design → 2. Reach classes / Authoring bar — \"both hands graded over the full admitted keyboard name set\" with the matrix row \"US-ANSI keyboard geometry | yes (v1) | the only graded geometry\"","problem":"The admitted name set is `schemas/README.md`'s (the design invents no second table), so the authoring bar and Test strategy #3 commit to grading every name in it, including nav-cluster, numpad and function-row keys; the ADR's own emphasis example presumes an `F13`-class key (\"next to the index finger or on `F13`\"), which no ANSI 104-key grid carries. The coordinate space is specified only as \"row/col signed decimal integers on the ANSI-US grid\", so the matrix's `yes (v1)` has an undeclared extent and a manifest `default:` naming a function key becomes `UnknownInput` by accident, not by rule.","fix":"One sentence in `reach-classes/v1/spec.md` naming the grid extent v1 grades — the rows/cols covered and which admitted names fall outside it as `off-reach` — so the admitted set and the coordinate range close over each other."},{"severity":"nit","location":"Compatibility matrix — \"| Mouse bindings | names only | `mouse:*` parses and appears in profiles/patches/transforms…\"","problem":"\"names only\" is a note, not a status, and the matrix is read as a yes/no support table.","fix":"Make the status \"yes (names only)\" and keep the existing note."}]}
```
