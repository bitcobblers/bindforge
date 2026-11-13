# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T03:54:34.546Z
- ended: 2026-11-13T03:56:17.066Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/011-action-taxonomy-and-core-skeleton/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 118d4d24f1852b11a4bab1607525db043b343856 f23e3e46775472749f40dcd6952c48fec3982eca` (both are blob ids of 02-proposal.md), then read the full current document.
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
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "Test strategy #3 — \"a document with exactly the four defined entries parses\"",
    "problem": "The format defines five entries: the spec entry table lists format, version, and the three axis arrays (all required), and the registry.toml shown in Chosen design carries exactly those five keys; the same test says a \"fourth axis array\" is an unknown entry, implying three defined arrays plus format and version. As written, a document with only four of the five required entries would have to parse, contradicting the entry table and the missing-axis-arrays fixture.",
    "fix": "Change \"four\" to \"five defined entries\"."
  },
  {
    "severity": "minor",
    "location": "Delta inventory — \"Unchanged: `conformance.rs`, `common/mod.rs`, `chain.rs`, `reach.rs`, …\"",
    "problem": "Contradicts Placeholders and lineage / Rollout 5, which edit \"the `ADR-009` references in the module docs of `crates/core/src/lib.rs`, `document.rs`, and `chain.rs`\" (chain.rs:3 and :16 hold such references). chain.rs is an edited file listed as unchanged.",
    "fix": "Move chain.rs into the Edited list annotated \"(module docs only)\", or split the Unchanged list into unchanged files and docs-only edits."
  },
  {
    "severity": "minor",
    "location": "Goals / Non-goals — \"the stale ADR-009 lineage in the contract docs retargeted to the 011-015 series\"",
    "problem": "The delta inventory and Rollout 5 retarget only schemas/README.md plus the lib.rs/document.rs/chain.rs module docs, but stale ADR-009 ownership lineage also sits in schemas/reach-classes/v1/spec.md (\"owned by ADR-009 … pending ADR-009\"), schemas/profile/v1/spec.md, and crates/core/src/reach.rs (\"ADR-009 owns the taxonomy the ids name\"), so the Risks row \"Stale ADR-009 lineage misleads the next contributor\" under-covers its own mitigation.",
    "fix": "Add those three files' lineage sentences to the Rollout 5 retarget list (doc-only, no deferred substance touched), or scope the Goal to the files actually edited."
  },
  {
    "severity": "nit",
    "location": "Chosen design — fixtures/invalid list, \"missing-axis-arrays.toml\"",
    "problem": "The proposal commits to toml-test's rule (\"one fault each, file named after the fault (A6)\"), but the plural name implies one document missing several required axis arrays — multiple faults in one fixture.",
    "fix": "Make the fixture omit exactly one required axis array and name it missing-axis-array.toml; the all-arrays-absent case is already covered by unit Test strategy #3."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/011-action-taxonomy-and-core-skeleton.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/011-action-taxonomy-and-core-skeleton/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "Chosen design — The contract, entry table — \"`version` (string, required, equal to the version segment of the directory holding the file)\"", "problem": "The new constraint is written as a per-document entry rule, but fixtures are `action-ids` documents held in `schemas/action-ids/v1/fixtures/{valid,invalid}/`, whose holding directories carry no version segment; the check itself exists only in `load_action_registry` (\"the declared `version` equals `dir`'s version segment\") and the unchanged harness parses fixtures without it. A third-party conformance reader applying the entry table literally to each fixture must either reject every valid fixture or invent the unstated rule \"match the nearest ancestor directory named `v<N>`\", and a version-mismatch fixture becomes unexpressible — the same rule appears under two shapes in spec.md (document constraint) and The model (loader-side check).", "fix": "Scope the entry-table parenthetical to the registry document and name the check site: \"`version` (string, required; `load_action_registry` refuses a `registry.toml` whose `version` differs from the version segment of the versioned directory holding it; fixtures declare `v1` and are gated by the version rule, not by their directory)\" — replacing the document-level constraint with a registry-load constraint."}]}
```
