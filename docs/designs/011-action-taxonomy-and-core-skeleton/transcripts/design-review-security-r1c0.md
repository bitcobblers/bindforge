# Transcript: design-review:security r1c0

## Provenance
- stage: design-review:security (round r1c0)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T03:47:43.592Z
- ended: 2026-11-13T03:48:45.859Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/011-action-taxonomy-and-core-skeleton/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/011-action-taxonomy-and-core-skeleton.md,
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
re-open one only if it contradicts docs/adrs/011-action-taxonomy-and-core-skeleton.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/011-action-taxonomy-and-core-skeleton/reviews/design/security-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design — The model: `crates/core/src/action.rs` (with Stability): \"`load_action_registry(dir: &Path) -> Result<ActionRegistry, CoreError>`\"","problem":"[new] No data source in the document is classified (security checklist 1). The registry entry point takes a caller-chosen `dir`, and the same section adds \"an `action-ids` arm in `parse`\" plus an entry in `SUPPORTED_FORMATS` — the shared parse path the proposal itself ties to \"the guard ADR-008's design decided for the hub's upload path\". Nothing states which `action-ids` document is authoritative, so a document arriving over the untrusted path is indistinguishable from the shipped registry, and `contains()`/`axis_kind()` — the attribute the ADR makes load-bearing (\"Only `discrete-lateral` actions may have polarity flipped by a transform (015)\") — could be answered from counterparty-chosen ids, re-grading arbitrary actions into the flip-eligible set and undermining the stability promise in Stability. A17's \"nothing acts on the attribute\" bounds today's damage but leaves the boundary unrecorded, so ADR-015 inherits it by accident.","fix":"Add one clause to spec.md's Extension and withdrawal rule and to `load_action_registry`'s doc: the registry bindforge-core answers from is the versioned directory shipped with the crate (the app-pinned path); an `action-ids` document that arrives from elsewhere is parsed as data only — it is never passed to `load_action_registry` and never consulted for `axis_kind`. That replaces the current silence on source authority rather than adding a mechanism."},{"severity":"minor","location":"Test strategy 4 (with The model, `load_action_registry`): \"an `action-ids` document declaring no version, or `v99`, is refused with the error naming the supported majors\"","problem":"[new] The loader is specified as \"`parse_file(dir.join(\"registry.toml\"))` and the variant check\", which checks the declared major is supported but never compares it to the version segment of the directory it was loaded from — and `conformance.rs`'s `schemas/*/v*/` walk derives format and major from the path. A document declaring a later-supported major sitting in `action-ids/v1/` parses under v1 rules, so spec and corpus drift with no refusal (checklist 7, fail closed).","fix":"State in the spec's entry table, and add as a case in Test strategy 4, that a declared `version` unequal to the version segment of the directory being loaded is an `InvalidValue` refusal."},{"severity":"nit","location":"Chosen design — Placeholders and lineage: \"one paragraph saying what lands there and which ADR populates it (013 and 014), that nothing reads them yet\"","problem":"[new] The README content spec covers the directories that will hold shared binding files from elsewhere, the one artifact class this checklist treats as untrusted, yet never says so; the classification appears nowhere in the proposal.","fix":"Add one sentence to the README spec: anything dropped into `data/profiles/` or `data/manifests/` is a file from elsewhere and is untrusted input once 013/014 read it — a classification, not a restatement of the stability rule the README already declines to repeat."},{"severity":"nit","location":"Chosen design — The contract: `schemas/action-ids/v1/`, spec.md entry table: \"`discrete-lateral` / `continuous-lateral` / `none` (arrays of strings, all required, may be empty)\"","problem":"[new] The entry table bounds element type but not array length, so for an `action-ids` document reaching the parser from outside the repo the only cap on id count is the parse budget; uniqueness (\"an id appears once across the three arrays\") and document-order construction then cost whatever that budget permits, and the budget's magnitude is never stated for this format.","fix":"Either state an explicit max id count in the entry table refused with `InvalidValue`, or state plainly that the existing parse budget is the bound for this format too — either replaces the implicit-only limit."},{"severity":"nit","location":"Chosen design — The model: \"every rejection keeps the single-line, truncating, path-free message the contract already promises\"","problem":"[new] The spec also requires refusals that name the offender (\"a repeat is a rejection naming the id and the array\"). For a malformed id the offending bytes are by construction outside the grammar charset and may carry terminal control sequences; \"single-line\" bounds newlines, not escapes, so an echoed hostile id can inject into a log or terminal line.","fix":"One clause on the error rule: an offending id is echoed escaped to printable characters, so the naming requirement cannot inject control sequences into a single-line message."}]}
```
