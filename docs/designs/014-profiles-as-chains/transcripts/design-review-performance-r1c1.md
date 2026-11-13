# Transcript: design-review:performance r1c1

## Provenance
- stage: design-review:performance (round r1c1)
- agent: review/design-performance
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T14:17:56.477Z
- ended: 2026-11-13T14:18:54.743Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/014-profiles-as-chains/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff ac9cb8acf87eeb9634dab42010276b843f39e219 ec0ea4774b9da2f62f24e139b0b9617a34e4c145` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/014-profiles-as-chains.md,
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
Your own previous findings (performance seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Cross-file loading: `load_chain` in `chain_ops.rs`, step 2, with the Resolution paragraph — \"Walk `derives` nearest-first: the ancestor `<id>.toml` is looked up *directly in that same directory*\" / \"a resolution reads at most `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB) across at most 32 documents\"",
    "problem": "The composite load bound counts documents, but the document never states that the walk is a single path. `derives` is inherited from v1 as an array of ancestors, nearest first (A1, A15), the model keeps `derives: Vec<String>`, and A6 records that nothing bounds array sizes — so one 1 MiB document can carry tens of thousands of ancestor ids. Whether `load_chain` follows `derives[0]` only, follows every element, or refuses extras is unstated, and the design requires `spec.md` to carry the load-level rules so a non-Rust implementer can conform; such an implementer reading \"an array of ancestors, nearest first\" will traverse all of them, evading the \"at most 32 documents\" bound through file references — the exact mechanism the design's own prior-art finding names (\"TF2 shows per-file caps evaded once files reference files\", A20).",
    "fix": "In `load_chain` step 2 add one clause fixing the walk's shape: the walk follows `derives[0]` only; any further element is carried as data and never traversed (state refusal instead if that is the intent). Restate the composite as \"`≤ MAX_CHAIN_DEPTH` documents opened per resolution, regardless of `derives` array length\". This replaces the implicit reading; it adds no mechanism and no new cap."
  },
  {
    "severity": "minor",
    "location": "Options → \"Composite chain budget\", Chosen design → The contract → Delta elements (\"the cap the manifest row cap already prices\"), Resolution composite paragraph, and Chain operations → `diff`/`merge`",
    "problem": "The claim that cost \"is bounded and now stated\" stops at bytes, link count and per-list count. It omits the derived totals that drive `resolve`'s work (≤ 32 links × 3 lists × 256 = 24,576 delta elements per chain; resolved bindings ≤ the joined manifest's action-row count) and never quantifies the \"manifest row cap\" it justifies the 256 cap by mirroring. That number is load-bearing for the chain operations: `diff` emits one element per differing/unbound action and `merge` composes \"over every action either touches\", so their outputs are capped only while the manifest row cap ≤ `MAX_PATCH_DELTA_ELEMENTS`. The Risks table prices raising the delta cap but not the opposite direction, which would let `diff`/`merge` emit documents that fail the design's own per-list parse rule — a case with no stated refusal and no case in Test strategy #5.",
    "fix": "Name the manifest row-cap constant with its value at first use, add the derived element total to the composite sentence, and add one clause to the `diff`/`merge` bullets that their output obeys `MAX_PATCH_DELTA_ELEMENTS` because the joined action table is capped, with a refusal if that ever stops holding."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Resolution → Load-time joins (\"every action id in any `set`/`clear`/`swap` must be a row of that manifest's `actions` table\") and determinism rule 3 (\"`set` replaces any binding for that action (last writer wins)\")",
    "problem": "The load-path cost depends on representation, and the document fixes only ordering, which pushes toward arrays: the action table's \"row order is data\" (rule 5), deltas apply sequentially in array order, and `fork` is \"refused if it collides with any link id in the chain\". Array membership for the action join, for \"any binding for that action\", and for `swap`'s two lookups is O(deltas × rows) — bounded but quadratic in the caps (worst case ~6M comparisons per resolution).",
    "fix": "One sentence in `resolve`'s contract: action-row membership and the action→binding layout are held in a lookup built once per `load_chain`/`resolve` call, and the manifest's `actions` array order is used only to order output, never for membership. No new mechanism; determinism rule 5 is untouched."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Cross-file loading: `load_chain` (\"a file whose `id` entry disagrees with its name is `InvalidValue` — the location rule, one rule\") and Resolution → Load-time joins (\"Manifest identity\")",
    "problem": "The design states an invalidation rule for only one side of the resolution input: pin equality catches a bumped manifest revision. Nothing covers the other side. Because id↔filename equality is a contract rule and links are looked up by id in one directory, a consumer (ADR-017 storage, ADR-018 hub) that identifies a chain by its head id and caches the resolved layout goes stale silently on an in-place link edit under an unchanged pin and unchanged ids. Deferring caching to 017/018 is consistent with the ADR; leaving chain identity undefined is what turns a fast consumer cache into a wrong one.",
    "fix": "One line in the `load_chain` section, mirrored in `spec.md` since the spec carries the load rules: a chain's identity is the contents of its link documents — id equality pins the lookup, not the value — and pin equality invalidates only against manifest changes."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/014-profiles-as-chains.md.
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/design/performance-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"nit","location":"Chain operations → `diff`/`merge` — \"`diff` … a `set` per differing action, a `clear` per action `b` unbinds\" and \"`merge` … composes the two delta lists in chain order … over every action either touches\"","problem":"re-raised residue of my r1 finding #2, downgraded. The bullets still carry no output-cap clause. The substance is closed: with `MAX_MANIFEST_ROWS = 256` now named and equal to `MAX_PATCH_DELTA_ELEMENTS`, every loadable input's touched actions are rows of the shared pin's capped action table, so output lists are provably within the per-list cap. The only remaining path is `merge` over parse-valid, action-join-invalid patches (each can touch up to 1024 distinct action ids across its three lists), which can return Ok with a 257th element — caught later by the load's own parse rule rather than by `merge` itself.","fix":"One clause per bullet: output lists are capped because touched actions are rows of the shared pin's `MAX_MANIFEST_ROWS`-capped action table; state that `merge` refuses when the composition would exceed a list (or that its inputs are taken to be loadable against the shared pin)."}]}
```
