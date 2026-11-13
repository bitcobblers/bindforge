# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T14:16:29.927Z
- ended: 2026-11-13T14:17:56.426Z
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
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Cross-file loading: `load_chain` in `chain_ops.rs` — \"Walk `derives` nearest-first: the ancestor `<id>.toml` is looked up *directly in that same directory*\" (with \"Resolution: deterministic\" — \"a resolution reads at most `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB) across at most 32 documents\")",
    "problem": "The walk text is singular but the entry table keeps v1's plural `derives` array (\"ancestor chain, nearest first\") and `merge` refers to `derives[0]`, so multi-element lists are expected to be legal; nothing states what `derives[1..]` means at v2. Under an untrusted-input format this collapses the design's own mitigation: if the walk follows every element the chain is a tree, `MAX_CHAIN_DEPTH` no longer bounds documents opened (the ≤32 × 1 MiB figure the Risks row \"Cross-file machinery is new attack surface\" leans on is false) and \"more than `MAX_CHAIN_DEPTH` links\" is left to the implementer, so two implementers disagree about how much hostile input they read. If only `derives[0]` is followed, every later element is an ancestor claim never resolved, so `ChainLinkMissing`'s completeness posture stops covering part of the document.",
    "fix": "State one rule at v2: a `patch` has exactly one `derives` element, a `base` none (v1's array untouched per A15/A1; the ADR's \"nearest ancestor first\" reading survives), pin it with a new `fixtures/invalid/derives-two-elements.toml`, and rewrite the composite-cost sentence to count documents opened per resolution under that rule. Alternative if the array stays: say all elements are followed, give the walk a visited-set plus a documents-opened counter, and replace the ≤32 × 1 MiB sentence with the resulting bound — do not leave both readings available."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Cross-file loading: `load_chain` — \"the ancestor `<id>.toml` is looked up *directly in that same directory*\"",
    "problem": "This is the only point where untrusted document content becomes a filesystem path, and the security property is never stated. It holds only incidentally because `derives` elements are slug-grammar-checked at parse (excluding `/`, `.`, `..`); the design never pins the constructed path's parent to the head's directory, never says no other entry may name a location, and Test strategy #3 has no traversal-shaped case — `derives-element-uppercase.toml` tests case, not path.",
    "fix": "Add one sentence to the load rule: the filesystem path is `<major dir>/<parse-validated slug>.toml`, no other entry names a location, and the constructed path must be a direct child of the head's directory; add the parse case that a `derives` element of `..` or `a/b` refuses as `InvalidValue` (the slug refusal already yields it — name it so a non-Rust implementer conforms)."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Cross-file loading / Deferred — \"A hub upload bundle is exactly such a directory (ADR-018 ships the transport; this is the shape it transports\"",
    "problem": "The only source classified as untrusted is `data/profiles/` (A12; Risks says \"`data/profiles/` posture unchanged\"). This design defines a second, network-sourced input — a directory of chain documents arriving from a hub — and writes `load_chain(head: &Path)` for a third, a player's personal chain directory; neither is classified. (Deferring trust-over-a-channel to ADR-018 is consistent with the ADR's Decision Four and Consequences, so this is not re-opening a deferral — it is the missing classification of sources whose guards are decided here.)",
    "fix": "One sentence where the bundle shape is recorded (or in A12's row): every document `load_chain` opens — fixtures, `data/profiles/`, a hub bundle directory, a personal chain directory — is parsed under the same version gate, byte budget and chain guards, source-independent; trust stays with validation and explicit adoption."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Chain operations — \"`diff` requires the two chains to share their deepest link id — the same base\"",
    "problem": "Base identity is the id string, not the document. Chains loaded from different directories can name the same base id with different bytes (the id↔filename rule is unique only within one directory), so `diff` asserts a property — \"the chain `a` extended by the diff resolves to `b`'s resolved layout\" — that is false for a consumer applying the emitted patch over its own base, and pin equality at load will not catch it because the ids match. `diff`'s output is precisely the artifact that gets shared.",
    "fix": "Make the shared-base test content-bearing: require the deepest link's parsed `Profile` (or its `serialize` bytes) to be equal for both chains, refusing with a named error when the ids match and the documents differ, and state that in the `diff` bullet instead of \"share their deepest link id\"."
  },
  {
    "severity": "minor",
    "location": "Entry table / Provenance labels — \"`generated-by` | string | no | 1–128 octets, no ASCII control characters: a label naming what produced the document\"",
    "problem": "This is the format's only free-text field and the checklist's \"what may be logged, what leaves the machine when bindings are shared\" is unanswered for it. It is exported with the document in the bundle shape defined here, echoed into logs via the `one_line` path on every refusal (A13), and its grammar allows arbitrary non-ASCII, so it can carry a personal name or email or a bidi-override/homoglyph string that renders as an authority name in a UI. Decision Four bounds the damage but does not make the label non-public or non-spoofable.",
    "fix": "Add one sentence in Provenance labels: the label is a public, shared artifact, must not be treated as secret or displayed as authority, and reaches logs only through `one_line`; tighten the grammar to exclude default-ignorable and bidi control characters, reusing the existing `generated-by-control-char.toml` fixture shape."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Provenance labels / The contract entry table — \"`generated-by` | string | no | 1–128 octets, no ASCII control, default-ignorable, or bidi-control characters\"","problem":"The r1 closure adds two new character classes to the label grammar but neither is defined nor pinned: the invalid-fixture list still names only `generated-by-past-cap.toml` and `generated-by-control-char.toml`, and Test strategy #1 still tests only \"`generated-by` past 128 octets or control-character-bearing\". The compatibility matrix makes `profile/v2/spec.md` + fixtures the whole contract for a \"Third-party reader, non-Rust\", so an undefined Unicode class with no fixture leaves two implementers free to disagree about whether a bidi override or a default-ignorable codepoint in a label is refused — the renders-as-authority case the ban was added to close (checklist: what may be logged / what leaves the machine when bindings are shared).","fix":"Extend the existing pair with one fixture carrying a bidi formatting character (or add `generated-by-bidi-control.toml`) and name it in Test #1's `generated-by` clause, and state the classes in the entry row implementably without a Unicode table: `Default_Ignorable_Code_Point` plus the explicit ranges U+202A–U+202E and U+2066–U+2069. No mechanism changes; this pins the rule the revision added."},{"severity":"nit","location":"Cross-file loading: `load_chain` in `chain_ops.rs`, step 2 — \"the constructed path is always a direct child of the head, `<major dir>/<parse-validated slug>.toml`\" and \"mirroring `load_manifest`'s id↔directory equality, which pins the lookup but not the value\"","problem":"Two wording slips in the sentence that now carries the path-containment property: \"a direct child of the head\" names a document where a directory is meant, and \"but not the value\" follows immediately after a rule whose subject is the `id` value, so it can be read as waiving the id↔filename check that stops one file answering for another link's id.","fix":"Rewrite as \"a direct child of the head's directory\", and replace \"which pins the lookup but not the value\" with \"which pins the lookup, not the document: equal ids in different directories are not the same link\" — the sentence's own closing clause (\"a chain's identity is its documents' contents\") already states that."}]}
```
