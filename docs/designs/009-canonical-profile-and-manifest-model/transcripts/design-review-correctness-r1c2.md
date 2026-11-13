# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:00:17.541Z
- ended: 2026-11-11T23:05:26.506Z
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
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Transforms (policy entry set) + Rollout 3 + Test strategy #10 — \"Each invocation must match its registry entry's declared arity and the grammars of its arguments (hand ∈ `left|right`, a placement id, a trailing-`*` action pattern); a mismatch is the shipped `InvalidValue { key, message }`, one `fixtures/invalid/` case per shape\" / \"is rejected at parse (`InvalidValue`), before `apply_policy` runs\"",
    "problem": "The parse-time check requires the registry table, and the design places that table on the far side of the one-way dependency edge: Rollout 3 creates the \"registry — each entry carrying its declared arity and argument grammars\" in crates/transforms (src/registry.rs, §4 file list), whose only edge is bindforge-transforms → bindforge-core. The rejections are pinned as schemas/policy/v1/fixtures/invalid/ cases, and the harness rejects fixture text through bindforge_core::parse alone (crates/core/tests/common/mod.rs:237-243, crates/core/tests/conformance.rs:138-144). So bindforge-core's reader must decide arity and the left|right grammar without the registry: as written the rule is unimplementable, and implementing it duplicates the registry in core, creating two sources for one fact.",
    "fix": "Declare the v1 registry table (the three transform names, their arity, and their argument grammars) in bindforge-core — crates/core/src/registry.rs, added to the Files-created row's core list and to Rollout 2 — with bindforge-transforms implementing the entries against it. The per-shape fixtures, the InvalidValue reuse, and Test #10's parse clause then need no change, and the registry keeps one home."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Input vocabulary — \"`schemas/README.md` carries the admitted keyboard-name list — the single enumeration the graded set below and Test strategy #3 iterate\"",
    "problem": "Test strategy #3 as revised iterates the graded set (\"every key in the graded set is graded in both hands, and a key outside a hand's table reads as that hand's `off-reach`\"), and the graded set is defined the other way round: \"every key the shipped placements and the seed manifest's `default` rows reference, extended only by review\". Nothing consumes the admitted keyboard-name list any more — input-id validity is grammar-only (`<part>` = lowercase words joined by `-`) and an ungraded key reads as off-reach, so `keyboard:banana` is a valid, total, ungraded input. The r0 clause that checked the list (\"every admitted keyboard name is graded in both hands\") was replaced, leaving a claim about a test that does not exist and an enumeration that gates nothing.",
    "fix": "State the list's enforcement in the reach-classes loader rules — \"a `keyboard:` key outside the admitted list is `InvalidValue`\" — and describe the list as the finite set the graded set draws from, dropping the Test strategy #3 attribution (Files-created row's parenthetical too)."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Adversarial posture — \"New `CoreError` variants, each pinned by a fixture: … `MergeConflict` … `IndexTooLarge`, `IndexTooDeep`\"",
    "problem": "Two of the listed variants have no fixture that pins them. MergeConflict's rule ships as patch/v1/spec.md text only: §3 defers fork/merge to the ADR-012 slice and Test strategy #7 defers their suite, so no v1 fixture or test produces it. IndexTooLarge/IndexTooDeep are tree-walk conditions; the harness walks fixtures one file at a time (conformance.rs:48-78), so an over-wide/over-deep walk cannot be a fixture document, and Test strategy #1 exercises them from a synthetic tree. The claim over-promises coverage the rollout does not ship.",
    "fix": "Qualify the sentence: \"each pinned by a fixture, except `MergeConflict` (its suite lands with `fork`/`merge` in the ADR-012 slice) and the two index limits (pinned by Test strategy #1's walk)\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains → Id → file — \"`data/manifests/` gets the same treatment keyed on each document's declared `game` + `revision`, so a `for` pin is an index lookup\"",
    "problem": "The collision case for the new manifest key is unstated. Two manifest documents declaring the same `game` + `revision` (their `id` values are free to differ) leave the pin lookup ambiguous, and the drift gate (§5, Test strategy #12) resolves every profile against exactly that key, so an ambiguous key makes the gate's verdict non-deterministic. The id case has a rule (\"`DuplicateId` fires on two documents declaring one id\"); the manifest key has none.",
    "fix": "One clause mirroring the id rule — \"two documents declaring one `game` + `revision` is `DuplicateId`\" — plus a fixture in Test strategy #9. No new error variant."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains (`resolve`) and Manifests — \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare, naming the profile id\"",
    "problem": "The check runs against a pin that may be absent: \"`for` … absent = a profile not bound to a game\", `Resolved { …, manifest: Option<ManifestPin> }`, and the drift gate resolves each profile \"against the manifest revision it pins\". An unpinned profile carrying `seeded-from` names a preset that no manifest is ever consulted against, so a dangling `seeded-from` passes CI silently — the residual hole left by moving the rule out of validate.",
    "fix": "State the precondition at parse where the entries are declared: \"`seeded-from` requires `for`; a `seeded-from` with no `for` is `InvalidValue`\", and mirror it in Test strategy #6. No new mechanism."
  },
  {
    "severity": "minor",
    "location": "Files created (`schemas/README.md` row) + Adversarial posture — \"`MAX_INDEX_ENTRIES`, `MAX_INDEX_DEPTH` in `index.rs`, re-exported from `lib.rs`\" vs the README row listing only \"format list, the canonical encoding rules, the admitted keyboard-name list …, `version`/`revision`, and the data-homes rule\"",
    "problem": "The design adds two public limit constants and two new rejection variants, but no contract location states them. The shipped guard section those limits live in reads \"The three constants are the reference implementation's public names for these limits\" and \"The `fixtures/invalid/` corpora are the contract for which condition produces which rejection\" (schemas/README.md:64,72-73); after this design there are five limit constants and two rejections whose condition has no contract home (they are not fixture-expressible).",
    "fix": "Extend the Files-created README row (and Rollout 1) to add the two limits and the two rejections to schemas/README.md's \"Input budget and chain bound\" section, replacing the \"three constants\" sentence's scope."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Manifests — \"then the three nested-array rows of the Encoding rules: `default` = `[id, input]` (the emphasis fact, contract 1)\" and \"The entry set stays the shipped closed allow-list — no prefix rule (Risks)\"",
    "problem": "Two pointers are stale. §1's canonical emphasis statement cites the same row as \"a manifest `default` row (contract 5)\" and states that the taxonomy (contract 1) carries no emphasis — so labeling the emphasis fact contract 1 points at the contract that disclaims it. And the prefix-rule Risk row that \"(Risks)\" cites was deleted in this revision.",
    "fix": "Cite contract 5 (or §1) for the emphasis row, and drop \"(Risks)\"."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Profiles are chains → Id → file — \"the extension stated separately (`.bindprofile` for profile/patch content, `.toml` for `schemas/` fixtures)\"",
    "problem": "The extension enumeration omits the data/ documents that use .toml: data/manifests/<game>/manifest.toml (§5), data/placements/*.toml (§2), and data/policies/ (§4), so the file-name rule under-covers the seed tree it governs.",
    "fix": "\"`.toml` for `schemas/` fixtures and for the `data/` manifest, placement, and policy documents\"."
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
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → Transforms (§4) — \"`apply_policy(base: &Resolved, policy: &PolicyDoc, ctx: &Ctx) -> PolicyOutcome` with `PolicyOutcome { base: Resolved, skipped: Vec<Skipped> }`\" + \"`apply_policy` validates every invocation against the table — its registry carries exactly that table — and a mismatch is the shipped `InvalidValue { key, message }` naming the invocation, raised before any transform runs\" (Test strategy #10: \"rejected by `apply_policy` (`InvalidValue`)\")","problem":"new — the r2 closure moved the invocation-table check out of core's parse and into `apply_policy`, but left `apply_policy`'s declared signature unchanged, returning `PolicyOutcome`, whose only field is `skipped: Vec<Skipped>`. There is no error channel: the `InvalidValue` the design now requires `apply_policy` to raise (Adversarial posture, §4, Test #10) has nowhere to go, and `skipped`'s stated semantics are the report of an *inapplicable* transform that no-ops, not a typed refusal. As written, a signature the design's own Goals promise as concrete Rust cannot express a rejection three sections pin, and a caller cannot distinguish \"stack rejected\" from \"stack applied with skips\". This is the defect the closure of my r1 #1 introduced: the parse-time check needed no error channel, the apply-time one does.","fix":"Make the return `Result<PolicyOutcome, CoreError>` (`bindforge-transforms` already depends on `bindforge-core`, so `CoreError` is in scope), keep `skipped` for the inapplicable-transform no-op, and state that the whole `stack` is validated against the invocation table before the first invocation runs. Test #10's wording then holds unchanged."},{"severity":"major","location":"Chosen design → Profiles are chains (`resolve`) — \"`resolve` also rejects a `seeded-from` the pinned manifest does not declare, naming the profile id — a profile fact read against a manifest, so not `validate`'s; `for` is required at parse, so the check always has a manifest.\"","problem":"new — `for` being required at parse guarantees a pin, not a manifest: the pin's halves are grammar-checked only (`BadPin`), and §3, §5, Test #9 and Test #12 state nothing about a grammatical `game` + `revision` that names no document in the manifest index. Under the design's own shapes that case is silently green: `Resolved { …, manifest: Option<ManifestPin> }` is documented as `None` for \"a profile not bound to a game\", so an unresolvable pin answers `None` as well — one shape carrying two facts — and both `resolve`'s `seeded-from` check and the drift gate (\"resolves every profile … against the manifest revision it pins — a read of the manifest index\") have no manifest to check against, so a dangling `seeded-from` on a profile pinning a manifest the seed does not ship passes CI, which is the residual hole the r2 closure states it closed. The claim as written is simply false: a pin is not a manifest.","problem_extra":""},{"severity":"minor","location":"Chosen design → Adversarial posture + Authoring bar (§2) — \"New `CoreError` variants, each pinned by a fixture in this slice except `MergeConflict` … and the two index limits\" and \"one invalid fixture per rejected condition\"","problem":"re-raised (narrowed from my r1 #3) — the exception list carves out only `MergeConflict` and the walk limits, but the shipped harness pins an invalid fixture by refusing its text through `bindforge_core::parse` (plus `resolve_chain` for a profile), one file at a time (`crates/core/tests/conformance.rs:137-154`, `crates/core/tests/common/mod.rs:237-243`). The remaining listed conditions are invisible to `parse`: `GeometryConflict` (needs both hand tables), the r2-added admitted-list `InvalidValue` and the class-vocabulary rule (reach loader, not parse), `DuplicateId` (two documents), `UnresolvedAncestor` (document + index), `AxisKindConflict`/`UnknownAction`/`UnknownInput` (manifest vs taxonomy), `PlacementNotIsometric`/`PlacementDegraded` (placement vs reach tables), `DuplicateDelta` (patch application). Worse, the authoring bar's \"one invalid fixture per rejected condition\" now applies to reach-classes loader conditions r2 added: an author who puts a `keyboard:banana` reach table in `schemas/reach-classes/v1/fixtures/invalid/` produces a parse-clean document and the shipped harness panics \"{label} is an invalid fixture and must be rejected\".","fix":"Qualify the posture sentence once: \"pinned by a fixture document where `parse` can see the condition; `GeometryConflict`, `DuplicateId`, `UnresolvedAncestor`, `AxisKindConflict`, `PlacementNotIsometric`, `PlacementDegraded`, `UnknownAction`, `UnknownInput`, `DuplicateDelta` are pinned over fixture data by the model suites that call the loader, index and validators (#3, #4, #6, #8, #9, #10)\" — and in the authoring bar state that reach-classes' rejected conditions are pinned by Test #3's model tests, `fixtures/invalid/` carrying only parse-refusable cases (Rollout 1's refused-TOML-construct cases). No new mechanism, no harness change."}]}
```
