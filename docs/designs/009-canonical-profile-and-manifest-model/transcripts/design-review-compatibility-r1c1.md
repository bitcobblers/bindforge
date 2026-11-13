# Transcript: design-review:compatibility r1c1

## Provenance
- stage: design-review:compatibility (round r1c1)
- agent: review/design-compatibility
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:49:38.641Z
- ended: 2026-11-11T22:56:21.160Z
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
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "Compatibility matrix — \"Hub upload path (wire budget, server-side chain pre-resolution) | unknown\"",
    "problem": "The row's status is `unknown`, so the matrix is not a decidable support record; the checklist requires every platform row to carry a status and treats `unknown` as a finding. This is matrix vocabulary only — the ADR-014 hub deferral itself is consistent with the ADR's \"Not decided here: … the hub service and voting\" and is not re-opened.",
    "fix": "Change that status cell to `no (v1) — out of scope` and keep the existing note (\"ADR-014 owns it (A23); the client-side guards and the id-index rule are the model's contribution\"). One-cell change, no new mechanism."
  },
  {
    "severity": "major",
    "location": "Chosen design → Encoding rules → Round-trip split (with Goals \"machine-checkable by the existing harness with no harness change\", Test strategy #1 \"`taxonomy`, `reach-classes`, `placement` are asserted by load\", and Test strategy #14 \"requires no change to `crates/core/tests/conformance.rs`\") — \"pure lookup data (`taxonomy`, `reach-classes`, `placement`) is asserted by load, using the exemption `reach-classes/v1/spec.md` already states\"",
    "problem": "That exemption is not a harness policy keyed on format kind: it is a hardcoded format-name equality test at crates/core/tests/conformance.rs:63 (`if dir.format == \"reach-classes\"`). Every other format directory's fixtures/valid/ is subjected to parse → serialize → re-parse → equality. So load-asserting `taxonomy` and `placement` requires editing conformance.rs, which contradicts Goals' \"no harness change\" and Test strategy #14's \"no change to conformance.rs\"; the three claims cannot all hold. A8's cited basis (conformance.rs:322-335) supports directory discovery only, not the exemption.",
    "fix": "Make all six formats round-trip (add serialize arms in Rollout 2's `Document` extension) and reserve the load-assert exemption for `reach-classes`, whose loader consumes a per-hand directory pair rather than one document. Replace \"taxonomy, reach-classes, placement are asserted by load\" in Test strategy #1 and the quoted sentence in Round-trip split accordingly. (If the exemption is widened instead, Goals' \"no harness change\" and Test strategy #14 must name that harness edit; round-tripping all six is the smaller change and keeps the harness frozen.)"
  },
  {
    "severity": "major",
    "location": "Chosen design → Profiles are chains §3 + Manifests §5 — \"its halves validated against the game-id grammar\" and \"content `data/manifests/<game>/manifest.toml` (fixture-name rule for `<game>`)\"",
    "problem": "The game-id grammar is the only gate between an untrusted `for` pin and a filesystem path component, and it is never defined: Encoding rules enumerate exactly two id shapes (taxonomy ids, input ids) and the quoted-key id list omits game ids. The rule leaned on exists only as a test helper, crates/core/tests/common/mod.rs:352 `is_fixture_name` (which strips a `.toml` suffix), so no shipped `crates/core` code can enforce it and `BadPin` has no specified predicate. This leaves open the same path-traversal class the design closes for profile ids under A18 (\"Ids never appear in a path\").",
    "fix": "Add the game-id shape to the canonical Encoding rules as `[a-z0-9-]+` (lowercase, no dots, no `@` — the fixture-name stem rule), and state that manifest resolution looks the pin's game half up in a walked index keyed on each manifest's declared `game` entry, replacing construction of `data/manifests/<game>/manifest.toml` from the pin."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Input vocabulary — \"it arrives as a *new* prefix (`pos:<part>`), leaving `keyboard:` meaning 'US-ANSI glyph' permanently\"",
    "problem": "The canonical grammar closes the namespace set (`<namespace>` ∈ `keyboard|mouse|gamepad`), so minting `pos` is a widening of a closed set in every format that carries input ids (profile, patch, manifest's `default:`/`dead-keys`, policy), not only in the reach tables. Deferred names only a `reach-classes/v2`, and the ADR's \"the id shape reserves it\" covers shape, not the version gate, so implementers must guess whether a v1 reader accepts `pos:a`.",
    "fix": "One sentence in Input vocabulary (canonical): `pos` joins the `<namespace>` set only in a v2 of every format that carries input ids, and a v1 document naming a `pos:` input is `UnknownInput`."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains §3 — \"File names follow the shipped fixture-name rule (lowercase ASCII, `-`-joined)\"",
    "problem": "The shipped rule as implemented requires a `.toml` suffix (is_fixture_name returns false for any other name), while profile content is `data/profiles/**/*.bindprofile` per this same section and ADR-009 Decision 3. The cited rule cannot be satisfied by the files it governs, and the Windows matrix row leans on it for its case-uniqueness claim.",
    "fix": "Cite the stem rule — \"lowercase ASCII, `-`-joined stem\" — and state the extension separately: `.bindprofile` for profile/patch content, `.toml` for `schemas/` fixtures."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Input vocabulary + Files created by this design — \"`schemas/README.md` lists the keyboard names v1 admits\" (README row: \"format list, the canonical encoding rules, `version`/`revision` split\")",
    "problem": "README is designated the single canonical enumeration of the v1 keyboard surface, and both the authoring bar (\"both hands graded over the full admitted keyboard name set\") and Test strategy #3 (\"every admitted keyboard name is graded in both hands\") iterate it, but the inventory row for README and Rollout 1 omit that list, and schemas/README.md contains no key-name enumeration today. The set the US-ANSI support claim is tested against has no named source.",
    "fix": "Add the admitted keyboard-name list to that Files-created row's contents and name it as the enumeration Test strategy #3 iterates."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/compatibility-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Compatibility matrix — Windows 10/11 contributor machine row: \"exercised leg (Test strategy #13)\"","problem":"The r1 renumbering moved the platform leg to Test strategy #14; #13 is now \"Integration — harness pickup and the version gate\", which tests nothing about a case-insensitive filesystem. Checklist item 7 requires the matrix row for each supported platform to name its platform test, and Rollout 7 already cites #14, so the matrix cell is the stale one. Fix: that one cell.","fix":"Change \"exercised leg (Test strategy #13)\" to \"exercised leg (Test strategy #14)\"."},{"severity":"minor","location":"Chosen design → Input vocabulary + Files created (README row) — \"`schemas/README.md` carries the admitted keyboard-name list — the single enumeration the graded set below and Test strategy #3 iterate\" and the README row's \"(the enumeration Test strategy #3 iterates)\"","problem":"The same revision rewrote Test strategy #3 to iterate the graded set, which the Authoring bar defines by a different rule (\"every key the shipped placements and the seed manifest's `default` rows reference, extended only by review\"). No CI check any longer enumerates README's admitted keyboard-name list, so the list's role is unstated and the US-ANSI `yes (v1)` matrix row's evidence trail points at a sentence that no longer matches the test it cites. The narrowing itself is the Revision log's r1 decision and is not re-opened; the dangling cross-reference is what the closure introduced.","fix":"Name README's list as the admissible-name namespace and the graded set as its reviewed subset: drop \"and Test strategy #3 iterate\" from Input vocabulary and the parenthetical from the README row (leaving Test #3 to state it iterates the graded set). If the README list is meant to be graded in full, instead restore in Test #3 a clause that every admitted keyboard name is graded in both hands."},{"severity":"minor","location":"Adversarial posture — \"New `CoreError` variants, each pinned by a fixture: … `DuplicateId`, `UnresolvedAncestor`, `MergeConflict`, `BadPin`, `IndexTooLarge`, `IndexTooDeep`\"","problem":"A fixture is one document under `vN/fixtures/{valid,invalid}` (schemas/README.md:23,41-42; crates/core/tests/conformance.rs:47-78), which cannot carry these conditions. `IndexTooLarge`/`IndexTooDeep` (added by this revision) are tree-shape conditions, not document conditions; and `MergeConflict`'s implementation and suite moved to the ADR-012 slice in this revision, so it has no v1 code path and no v1 pin. Rollout 1's \"a valid+invalid fixture pair per format\" cannot ship those cases as the sentence claims.","fix":"Split the sentence: variants a single document pins are pinned by an `fixtures/invalid/` case; `DuplicateId`, `UnresolvedAncestor`, `IndexTooLarge`, `IndexTooDeep` are pinned by the multi-document / scratch-tree unit tests Test strategy #1 and #9 name; `MergeConflict` ships and is pinned in the ADR-012 slice with its rule in `patch/v1/spec.md`."},{"severity":"nit","location":"Chosen design → Manifests §5 (Entries) — \"`default` = `[id, input]` (the emphasis fact, contract 1)\"","problem":"Contract 1 is the action taxonomy, from which ADR-009 Decision 1 and this proposal's own §1 (\"the game fact is the action's default input, recorded as a manifest `default` row (contract 5)\") both exclude emphasis. The rewritten entry sentence cites the wrong contract.","fix":"Cite contract 5 for that row."},{"severity":"nit","location":"Compatibility matrix — Windows row: \"tree file names follow the shipped case-unique stem rule\"","problem":"The shipped case-uniqueness check runs over `schemas/*/v*/fixtures/{valid,invalid}` only (crates/core/tests/conformance.rs:277-292); `data/profiles/` and `data/manifests/` file names are unenforced. Harmless in substance, exactly because ids never become path components, but the row's wording claims coverage the shipped test does not have.","fix":"Scope the wording to fixture names and state that `data/` file names are hygiene, not load-bearing."}]}
```
