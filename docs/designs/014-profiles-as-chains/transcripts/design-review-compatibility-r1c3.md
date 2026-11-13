# Transcript: design-review:compatibility r1c3

## Provenance
- stage: design-review:compatibility (round r1c3)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T14:23:04.562Z
- ended: 2026-11-13T14:24:35.042Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/014-profiles-as-chains/02-proposal.md.
This is the ratification pass c3: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff ac9cb8acf87eeb9634dab42010276b843f39e219 2aae9c9eed298b076191c645f1686c764e0b8a01` (both are blob ids of 02-proposal.md), then read the full current document.
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
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "Chosen design → Delta inventory (Edited) — \"`PROFILE_ENTRIES` renamed `PROFILE_V1_ENTRIES`, new `PROFILE_V2_ENTRIES`\"",
    "problem": "The rename is presented as an ordinary edit, but the proposal never states whether PROFILE_ENTRIES is private or part of core's public surface; the zz_api_probe.rs edit is listed for 'the new public names' only, so nothing says whether existing downstream users of the old name break, and the Compatibility matrix has no core-API row.",
    "fix": "State in the Edited list (or a matrix row) that the rename is internal because PROFILE_ENTRIES is private — as the assumption register already does for placement::is_slug_name (pub(crate)) — or, if it is public, record the rename as a public-surface break in the matrix; do not add a compat alias."
  },
  {
    "severity": "minor",
    "location": "The contract: schemas/profile/v2/ — \"The `spec.md` also states the load-level rules below (the directory rule, pin equality, the joins)\"",
    "problem": "The keyboard-reality check is only half covered: non-ASCII binding names are refused (slug grammar, uppercase fixtures) and named keys are Deferred consistently with the ADR, but chain documents bake input ids like `keyboard:d` into artifacts shared across players (the example carries `set = [\"move.steer-left keyboard:d\"]`), and nowhere — contract, matrix, or Deferred — is it stated that the physical-key semantics of the closed `keyboard:` vocabulary (character vs position on non-QWERTY layouts) are inherited unchanged from the shipped input/reach-classes contract and are not this design's to define.",
    "fix": "One sentence appended to the spec.md load-rule list pinned in Rollout 1: input-id semantics are inherited unchanged from the shipped contract and a chain is layout-neutral data. It adds no mechanism; it clarifies the boundary the determinism rules already assume."
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix — \"macOS contributor | no (v1)\"",
    "problem": "The status cell \"no (v1)\" is cryptic: a first-time reader cannot tell which posture '(v1)' names or whether a 014-era contribution on macOS is in or out.",
    "fix": "Spell the cell out, e.g. \"no — outside design 008's supported matrix\", and drop the bare \"(v1)\" marker."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/design/compatibility-r1c3.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → The contract: `schemas/profile/v2/`, `generated-by` row — \"1–128 octets, no ASCII control, default-ignorable, or bidi-control characters\"","problem":"The revision adds two exclusion classes named as Unicode properties that the std-only story the matrix commits to (\"Toolchain floor (1.85.0, edition 2024) | yes | std-only, no new dependency, `Cargo.lock` byte-unchanged\") cannot answer: `char::is_control()` covers Cc only and std exposes no Default_Ignorable or bidi-category accessor (shipped code checks exactly that: document.rs:875, error.rs:164). The reference implementation therefore needs a hand-maintained range table the document never states, and since `profile/v2/spec.md` is the language-neutral contract (matrix row \"Third-party reader, non-Rust\"), a third-party reader cannot decide whether a label carrying U+FE0F (Default_Ignorable) or a soft hyphen is a valid shipped document — a validity disagreement over a shared public artifact. Only bidi and ASCII-control are fixture-pinned (`generated-by-bidi-char.toml`, `generated-by-control-char.toml`); \"default-ignorable\" has none.","fix":"In the entry row and `spec.md`, replace the property names with the closed code-point list — bidi-control U+061C, U+200E–U+200F, U+202A–U+202E, U+2066–U+2069 — and either drop \"default-ignorable\" or pin it to an explicit range list; add the matching invalid fixture so the third boundary is corpus-pinned like the other two. Adds no mechanism; it replaces two Unicode-property names with the ranges they mean."},{"severity":"nit","location":"Chosen design → Cross-file loading: `load_chain` in `chain_ops.rs`, step 2 — \"the constructed path is always a direct child of the head, `<major dir>/<parse-validated slug>.toml`\"","problem":"The link path is stated as a `/`-joined template; the compatibility checklist asks that separators be handled by a library rather than string formatting, and the Windows contributor row is `yes`, where the separator is `\\`. Traversal safety is already closed (slug grammar refuses `..` and `/`, pinned in Test strategy #1) — only the separator wording is missing.","fix":"One clause naming the std construction — \"the link path is `Path::join`ed from the head's parent directory\" — and drop the `/`-template spelling."},{"severity":"nit","location":"Chosen design → Chain operations: `fork`, `diff`, `merge` — \"empty delta lists — a shell `load_chain` refuses as `patch-empty-deltas` until the caller fills one element\"","problem":"`patch-empty-deltas` is a fixture name, not a refusal; every other promise to a caller names the typed variant (`InvalidValue`, `ChainLinkMissing`, `MissingEntry`), and the variant a caller must match is part of the public error surface. Test strategy #5 repeats the behavior without naming the variant.","fix":"Name the typed refusal (`InvalidValue` naming the link/element, the shape the Per-link rules already implies) and cite `patch-empty-deltas.toml` as the fixture that pins it."},{"severity":"nit","location":"Chosen design → Resolution, \"Mount at any rung\" — \"`pub struct Chain { links: Vec<Profile> }  // only load_chain builds one`\" with \"a consumer mounts a prefix by `load_chain`ing a lower link as head, or reads the documents singly with `parse_file`\"","problem":"The revision made the only read path into `Chain` private without adding an accessor, so a consumer holding a `Chain` cannot see its links; the mount-at-any-rung capability (ADR Decision Three, \"take its base and patch independently\") is then reachable only by re-parsing files and walking `Profile.derives` by hand, and `zz_api_probe.rs`'s \"new public names\" is undecidable before implementation.","fix":"One clause stating the intent: either a `Chain::links()` iteration accessor, or \"`Chain` is opaque by design; the rung ids come from the head document's `derives` read with `parse_file`\". The second option names the text the accessor would otherwise replace."}]}
```
