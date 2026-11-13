# Transcript: design-review:security r1c0

## Provenance
- stage: design-review:security (round r1c0)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T13:11:19.153Z
- ended: 2026-11-13T13:12:33.485Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/013-game-manifests/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/013-game-manifests.md,
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
re-open one only if it contradicts docs/adrs/013-game-manifests.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/013-game-manifests/reviews/design/security-r1c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → The contract — entry table row `presets` and the **Budgets** bullet: \"`MAX_MANIFEST_ROWS = 256` bounds each of `actions`, `dead-keys`, and `docked-actions`\"","problem":"New. `presets` is the only array in the format with no cardinality cap: the entry table gives it none, the Budgets bullet omits it, and the Risk row's mitigation (\"The four named budgets refuse before any join\") therefore does not cover it. Under A14 it is untrusted input, and the \"each preset id appears once\" rule is a global uniqueness pass over it, so one hostile document inside the 1 MiB budget carries on the order of 10^5 slug strings that get uniqueness-checked, stored, and re-serialized by every consumer.","fix":"Add the cap in the same sentence as the others — \"`MAX_PRESET_IDS = 64` bounds `presets`\" — mirror it inline in the entry table the way `exclusive-groups` already states \"at most 64 groups\", and extend the invalid corpus (`presets-past-cap.toml`) plus the Test strategy #4 budget cases."},{"severity":"minor","location":"Chosen design → The contract → **Row encoding** — \"a native name of 1–128 octets with no ASCII control character\"","problem":"New. The ban is ASCII-only, so C1 controls (U+0080–U+009F) and Unicode format characters (U+202E RIGHT-TO-LEFT OVERRIDE, U+200B, U+FEFF) are legal native names under the spec's own wording. Native names are exactly the design's declared echo payload (\"every echoed native name goes through the escape-and-truncate error path\") and the content shown in the manifest PR diff that is this design's only review gate, so a hostile manifest can reorder or mask what a reviewer's terminal displays. Since `spec.md` + fixtures are the third-party contract, an out-of-repo reader implements the ASCII-only ban verbatim.","fix":"Change the one clause to \"no Unicode control or format character (`Cc`/`Cf`)\" and pin it with one invalid fixture (e.g. `name-format-char.toml` carrying U+202E) beside `name-control-char.toml`."},{"severity":"minor","location":"Chosen design → Load-time validation: the registry join — \"the two cross-file rules (join, location) live at load, where the registry and the directory are in hand\"","problem":"New. The design splits validation across two levels but never states what makes the split safe for consumers. The `document.rs` edit adds a public `Document::Manifest(Manifest)` arm to the existing generic `parse`, and `Manifest`'s fields are `pub`, so a consumer can obtain and act on a `Manifest` whose `actions[].action` / `docked_actions[].action` never went through `registry.contains`. ADR-013's Decision requirement (\"ids are validated against the loaded registry, so a manifest cannot invent an action the taxonomy does not name\") and ADR-015's target refusals then rest on a value that skipped the only guard, and nothing in the design forbids that path.","fix":"Name the invariant once in this section: a `Manifest` not produced by `load_manifest` is not contract-valid and is not a legal input to `seeded-from` or to 015's refusals — and say where it is enforced. If enforcement rather than convention is intended, keep the constructor crate-internal (`Manifest` fields `pub(crate)`, `pub` accessors) so the generic arm cannot hand out a consumable unjoined value."},{"severity":"minor","location":"Chosen design → Load-time validation, step 1 — \"`parse_file(game_dir.join(\"manifest.toml\"))` — size budget before the read\"","problem":"New. A14 classifies the whole `data/manifests/` tree as arriving from elsewhere, so it can carry `manifest.toml` as a symlink or a FIFO/character device; such an entry reports a small or zero size, so the pre-read size budget that every other hostile-shape mitigation depends on is bypassed and the read is unbounded or never-ending.","fix":"In step 1 require a regular file: refuse a symlink or non-regular entry as `InvalidValue` naming `manifest.toml` (never the resolved path, per the existing \"never echoes a local path\" rule), and pin it in Test strategy #4 beside the directory-segment case."},{"severity":"nit","location":"Chosen design → The contract — entry table `revision`: \"the data revision, `>= 1`, monotone per game\"","problem":"New. The field has a stated lower bound and no stated upper bound, and the enumerated integer edge cases are \"`revision = 0`, a negative revision, and a string revision\"; an out-of-range decimal (`revision = 99999999999999999999`) against an `i64` field has neither a rule nor a fixture, unlike every other shape in the section.","fix":"State the upper bound (\"must fit `i64`\", or narrow the field to `u32`) and add one invalid fixture so third-party readers refuse the same documents this reader does."}]}
```
