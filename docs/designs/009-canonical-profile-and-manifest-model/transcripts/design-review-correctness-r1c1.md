# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:49:38.640Z
- ended: 2026-11-11T22:56:47.349Z
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
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Encoding rules (Round-trip split); Test strategy #1 — \"pure lookup data (`taxonomy`, `reach-classes`, `placement`) is asserted by load, using the exemption `reach-classes/v1/spec.md` already states\"",
    "problem": "The shipped harness grants the load-asserted exemption only to format \"reach-classes\" (crates/core/tests/conformance.rs:63 gates it on dir.format == \"reach-classes\"); every other format's valid fixtures are re-serialized to an equal value by the unmodified harness (conformance.rs:47-78, exactly as verified assumption A8 states). Taxonomy and placement fixtures authored under the exemption the rule claims for them will fail the harness, contradicting Goals (\"machine-checkable by the existing harness with no harness change\") and Test strategy #14 (\"requires no change to crates/core/tests/conformance.rs\"). The split also leaves `policy` in neither half, though the harness round-trips it and it is a person-edited document.",
    "fix": "State in the Encoding rules and Test strategy #1 that taxonomy, placement, and policy fixtures must re-serialize to an equal value (as the harness enforces today for every format except reach-classes), and reserve \"asserted by load\" for the seed data files (actions.toml, left.toml/right.toml), which live outside fixtures/ and are load-checked by the model tests. This replaces the claimed exemption mechanism; it adds nothing."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Manifests (validate); Test strategy #9 — \"`validate(manifest, &Taxonomy)` enforces: … a profile's `seeded-from` names a declared preset\"",
    "problem": "The rule is a cross-document check (profile's seeded-from against the manifest's presets), but validate's declared inputs are the manifest and the taxonomy only, so it cannot evaluate this condition. Test strategy #9 repeats the mismatch by folding the seeded-from case into \"manifest validation\" with its own fixture.",
    "fix": "Move that condition out of validate(manifest, &Taxonomy) into the chain-resolution/drift step: \"`resolve` rejects a `seeded-from` the pinned manifest does not declare, naming the profile id\" — resolve already loads the pinned manifest, so no new mechanism; mirror the sentence in Test #9."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Encoding rules (`version` bullet) vs Profiles are chains / Manifests — \"a document's own revision is a dotted `<major>.<minor>[.<patch>]` string\"",
    "problem": "The profile section defines revision as \"a `<major>.<minor>` string\" and the `for` pin validates its revision half as \"a dotted `<major>.<minor>`\". A manifest declaring the encoding-rule-permitted patch component (revision = \"0.1.2\") can never be named by a game@revision pin: the same fact appears in two shapes and one is unpinnable.",
    "fix": "Drop the `[.<patch>]` option from the Encoding rule: the v1 revision grammar is `<major>.<minor>` for every document that can be pinned. This replaces the current revision sentence."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Transforms (relocate-cluster) — \"the matched actions' current inputs form a contiguous cluster for `<hand>` in `base`\"",
    "problem": "\"Contiguous cluster\" is a predicate no section defines over the data the design ships (reach tables carry class+row/col; placement::validate computes isometry and reach bound), and it decides apply-vs-Skipped. The design's own Options argument rejects review-only legality claims because \"two implementations could disagree\" — an undefined precondition predicate has the same defect.",
    "fix": "Define the precondition in existing data: the matched actions' current input set must be isometric (same offset vectors) to the named placement's source-key set, else Skipped — replacing the word \"contiguous\"."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Profiles are chains (Chain guard) and Risks row — \"32 links is a base plus 31 patches\"",
    "problem": "The shipped resolve_chain rejects only when derives.len() > MAX_CHAIN_DEPTH (crates/core/src/chain.rs:27), so the accepted maximum chain is 32 derives links = 33 documents — a base plus 32 patches. Test strategy #5's boundary (\"a chain of exactly MAX_CHAIN_DEPTH links resolves and one link deeper is ChainTooDeep\") is only true when links means derives entries, which makes the \"base plus 31\" phrasing off by one.",
    "fix": "State the unit once — MAX_CHAIN_DEPTH counts derives links; the maximum chain is a base plus 32 patches (33 documents) — and carry it into the Risks row."
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
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → Transforms (policy entry set) + Rollout 3 + Test strategy #10 — \"Each invocation must match its registry entry's declared arity and the grammars of its arguments (hand ∈ `left|right`, a placement id, a trailing-`*` action pattern); a mismatch is the shipped `InvalidValue { key, message }`, one `fixtures/invalid/` case per shape\" / \"is rejected at parse (`InvalidValue`), before `apply_policy` runs\"","problem":"The parse-time check requires the registry table, and the design places that table on the far side of the one-way dependency edge: Rollout 3 creates the \"registry — each entry carrying its declared arity and argument grammars\" in crates/transforms (src/registry.rs, §4 file list), whose only edge is bindforge-transforms → bindforge-core. The rejections are pinned as schemas/policy/v1/fixtures/invalid/ cases, and the harness rejects fixture text through bindforge_core::parse alone (crates/core/tests/common/mod.rs:237-243, crates/core/tests/conformance.rs:138-144). So bindforge-core's reader must decide arity and the left|right grammar without the registry: as written the rule is unimplementable, and implementing it duplicates the registry in core, creating two sources for one fact.","fix":"Declare the v1 registry table (the three transform names, their arity, and their argument grammars) in bindforge-core — crates/core/src/registry.rs, added to the Files-created row's core list and to Rollout 2 — with bindforge-transforms implementing the entries against it. The per-shape fixtures, the InvalidValue reuse, and Test #10's parse clause then need no change, and the registry keeps one home."},{"severity":"minor","location":"Chosen design → Input vocabulary — \"`schemas/README.md` carries the admitted keyboard-name list — the single enumeration the graded set below and Test strategy #3 iterate\"","problem":"Test strategy #3 as revised iterates the graded set (\"every key in the graded set is graded in both hands, and a key outside a hand's table reads as that hand's `off-reach`\"), and the graded set is defined the other way round: \"every key the shipped placements and the seed manifest's `default` rows reference, extended only by review\". Nothing consumes the admitted keyboard-name list any more — input-id validity is grammar-only (`<part>` = lowercase words joined by `-`) and an ungraded key reads as off-reach, so `keyboard:banana` is a valid, total, ungraded input. The r0 clause that checked the list (\"every admitted keyboard name is graded in both hands\") was replaced, leaving a claim about a test that does not exist and an enumeration that gates nothing.","fix":"State the list's enforcement in the reach-classes loader rules — \"a `keyboard:` key outside the admitted list is `InvalidValue`\" — and describe the list as the finite set the graded set draws from, dropping the Test strategy #3 attribution (Files-created row's parenthetical too)."},{"severity":"minor","location":"Chosen design → Adversarial posture — \"New `CoreError` variants, each pinned by a fixture: … `MergeConflict` … `IndexTooLarge`, `IndexTooDeep`\"","problem":"Two of the listed variants have no fixture that pins them. MergeConflict's rule ships as patch/v1/spec.md text only: §3 defers fork/merge to the ADR-012 slice and Test strategy #7 defers their suite, so no v1 fixture or test produces it. IndexTooLarge/IndexTooDeep are tree-walk conditions; the harness walks fixtures one file at a time (conformance.rs:48-78), so an over-wide/over-deep walk cannot be a fixture document, and Test strategy #1 exercises them from a synthetic tree. The claim over-promises coverage the rollout does not ship.","fix":"Qualify the sentence: \"each pinned by a fixture, except `MergeConflict` (its suite lands with `fork`/`merge` in the ADR-012 slice) and the two index limits (pinned by Test strategy #1's walk)\"."},{"severity":"minor","location":"Chosen design → Profiles are chains → Id → file — \"`data/manifests/` gets the same treatment keyed on each document's declared `game` + `revision`, so a `for` pin is an index lookup\"","problem":"The collision case for the new manifest key is unstated. Two manifest documents declaring the same `game` + `revision` (their `id` values are free to differ) leave the pin lookup ambiguous, and the drift gate (§5, Test strategy #12) resolves every profile against exactly that key, so an ambiguous key makes the gate's verdict non-deterministic. The id case has a rule (\"`DuplicateId` fires on two documents declaring one id\"); the manifest key has none.","fix":"One clause mirroring the id rule — \"two documents declaring one `game` + `revision` is `DuplicateId`\" — plus a fixture in Test strategy #9. No new error variant."},{"severity":"minor","location":"Chosen design → Profiles are chains (`resolve`) and Manifests — \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare, naming the profile id\"","problem":"The check runs against a pin that may be absent: \"`for` … absent = a profile not bound to a game\", `Resolved { …, manifest: Option<ManifestPin> }`, and the drift gate resolves each profile \"against the manifest revision it pins\". An unpinned profile carrying `seeded-from` names a preset that no manifest is ever consulted against, so a dangling `seeded-from` passes CI silently — the residual hole left by moving the rule out of validate.","fix":"State the precondition at parse where the entries are declared: \"`seeded-from` requires `for`; a `seeded-from` with no `for` is `InvalidValue`\", and mirror it in Test strategy #6. No new mechanism."},{"severity":"minor","location":"Files created (`schemas/README.md` row) + Adversarial posture — \"`MAX_INDEX_ENTRIES`, `MAX_INDEX_DEPTH` in `index.rs`, re-exported from `lib.rs`\" vs the README row listing only \"format list, the canonical encoding rules, the admitted keyboard-name list …, `version`/`revision`, and the data-homes rule\"","problem":"The design adds two public limit constants and two new rejection variants, but no contract location states them. The shipped guard section those limits live in reads \"The three constants are the reference implementation's public names for these limits\" and \"The `fixtures/invalid/` corpora are the contract for which condition produces which rejection\" (schemas/README.md:64,72-73); after this design there are five limit constants and two rejections whose condition has no contract home (they are not fixture-expressible).","fix":"Extend the Files-created README row (and Rollout 1) to add the two limits and the two rejections to schemas/README.md's \"Input budget and chain bound\" section, replacing the \"three constants\" sentence's scope."},{"severity":"nit","location":"Chosen design → Manifests — \"then the three nested-array rows of the Encoding rules: `default` = `[id, input]` (the emphasis fact, contract 1)\" and \"The entry set stays the shipped closed allow-list — no prefix rule (Risks)\"","problem":"Two pointers are stale. §1's canonical emphasis statement cites the same row as \"a manifest `default` row (contract 5)\" and states that the taxonomy (contract 1) carries no emphasis — so labeling the emphasis fact contract 1 points at the contract that disclaims it. And the prefix-rule Risk row that \"(Risks)\" cites was deleted in this revision.","fix":"Cite contract 5 (or §1) for the emphasis row, and drop \"(Risks)\"."},{"severity":"nit","location":"Chosen design → Profiles are chains → Id → file — \"the extension stated separately (`.bindprofile` for profile/patch content, `.toml` for `schemas/` fixtures)\"","problem":"The extension enumeration omits the data/ documents that use .toml: data/manifests/<game>/manifest.toml (§5), data/placements/*.toml (§2), and data/policies/ (§4), so the file-name rule under-covers the seed tree it governs.","fix":"\"`.toml` for `schemas/` fixtures and for the `data/` manifest, placement, and policy documents\"."}]}
```
