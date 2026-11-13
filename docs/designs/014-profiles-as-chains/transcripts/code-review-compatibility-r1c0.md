# Transcript: code-review:compatibility r1c0

## Provenance
- stage: code-review:compatibility (round r1c0)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T15:15:06.742Z
- ended: 2026-11-13T15:17:56.272Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/014-profiles-as-chains/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 4cfcd8db4477247e8e6ecb26ecade40451e0ad91...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/014-profiles-as-chains.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/code/compatibility-r1c0.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "crates/core/src/chain_ops.rs:347-375 (open_link: try_exists at :348, id check at :363-373; lookup built at :131, stem at :379)", "problem": "The id↔file-name equality rule compares the document's declared `id` with the stem of the path `load_chain` itself constructed, never with the name the filesystem matched. On a case-insensitive filesystem (the matrix's Windows row, 'Supported? yes'), a directory holding `Game-Base.toml` with `id = \"game-base\"` satisfies a `derives = [\"game-base\"]` lookup and loads, while the same chain refuses `ChainLinkMissing` on the Linux CI runner. The Windows matrix row rests on 'profile-id ↔ file-name equality is case-unique', which the loader does not enforce in that direction; no test pins a case-only mismatch (tests/chain_ops.rs:1354 differs by more than case), and the Windows leg of Test strategy #9 is the one leg that would miss it.", "fix": "Pin the byte-exact on-disk name before opening the link: in `open_link`, require `fs::read_dir(directory)` to contain an entry whose `file_name()` equals `{expected}.toml` byte-for-byte (one scan per link, inside the existing MAX_CHAIN_DEPTH budget), and add the case-only mismatch to the load_chain guard cases (Test strategy #3). Alternatively state the caveat contract-side — one sentence in schemas/profile/v2/spec.md §'Loading a chain' rule 3 plus the matrix's Windows row — but the platform divergence must be refused or written down, not left implicit."}, {"severity": "minor", "location": "schemas/profile/v2/spec.md:46 and §Per-link rules; enforcing code crates/core/src/document.rs:1074 (check_generated_by) with is_label_char at :1101", "problem": "spec.md states the `generated-by` character rule as 'no ASCII control or bidi-control characters', but the reference reader also refuses the default-ignorable/joiner set the proposal names (soft hyphen, U+034F, U+061C, U+180E-180F, U+200B-200F, U+202A-202E, U+2060-206F, U+FEFF, U+FFF9-FFFB). Only the two halves the spec states are corpus-pinned (generated-by-control-char.toml, generated-by-bidi-char.toml — the latter's comment asserts the default-ignorable rule, which is not contract text), and no fixture or test carries a non-ASCII label at all, so the accept side of the boundary is unpinned too. Under the matrix's 'third-party reader, non-Rust — spec.md + fixtures are the contract' row, a conforming reader accepts documents bindforge-core refuses.", "fix": "Restore the proposal's three-way wording in spec.md:46 and state the excluded set once as is_label_char's comment enumerates it; then pin the boundary in tests/chain_ops.rs grammar cases: one refusal for a U+00AD label, one for U+200B, and one accept case for a plain non-ASCII label. Keep the design's fixture list as written; if the panel prefers corpus pins (generated-by-soft-hyphen.toml, a valid unicode-label fixture), edit the design's fixture list in the same PR."}, {"severity": "minor", "location": "crates/core/src/document.rs:1051 (check_profile_slug), used as a file name at crates/core/src/chain_ops.rs:131 and :348; fresh ids via chain_ops.rs:581", "problem": "The slug grammar admits Windows reserved device basenames (`con`, `aux`, `prn`, `nul`, `com1`-`com9`, `lpt1`-`lpt9`), which on Windows are not file names: they cannot be created, git refuses to check them out, and Path::try_exists answers through the device. A chain with `id = \"con\"` loads on Linux and is unbuildable/unfetchable for the Windows contributor the matrix marks supported, and a hub bundle could carry one. Nothing in the diff addresses it. Precedent note: placement::is_slug_name already turns manifest and geometry ids into path components with the same grammar and no guard, so this is a widening of existing exposure, not new.", "fix": "Decide it once where the grammar lives: either reject a reserved Windows basename inside check_profile_slug (naming the rule, with a grammar/chain_ops case pinning it) and open the same question for placement::is_slug_name as a follow-up, or record it under Deferred/out of scope with an owner and add one sentence to spec.md §'Id grammars' plus the matrix's Windows row that the id grammar does not exclude platform-reserved names."}]}
```
