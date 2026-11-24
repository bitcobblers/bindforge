# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: basalt/qwen-flash-next#medium
- template: tools/orchestration/prompts/research.md @ c978d4b12120
- started: 2026-11-24T03:06:45.494Z
- ended: 2026-11-24T03:12:42.453Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":95196,"output":24587,"reasoning":0,"cache.read":1229013,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/017-keybinding-extraction-codecs.md for this pipeline run.
Write your findings to docs/designs/017-keybinding-extraction-codecs/01-research.md.

Your read set is: the source document at docs/adrs/017-keybinding-extraction-codecs.md; the directory
listings of `docs/adrs/` and `docs/tasks/` (those listings are the ADR and
task indexes — there are no index files); the ADRs that source cites by
number — for a task, the ADR named by its `associated_adr` frontmatter; and,
if this design directory already holds them, the immediately prior run's
`01-research.md` and `02-proposal.md`.
Outside that set, read at most 25 more files. The ceiling is the point: reads
beyond the sources are context the run pays for twice.
Never open docs/designs/017-keybinding-extraction-codecs/reviews/ or docs/designs/017-keybinding-extraction-codecs/transcripts/ — run records
are not research input, and the runner audits tool calls and fails the attempt
on any access to them.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary":"Wrote docs/designs/017-keybinding-extraction-codecs/01-research.md for ADR-017 (codec families, config-path discovery, bindings/v1 snapshot, read-only `discovery bindings` face). Read set: the ADR plus its cited ADRs 008, 009, 012, 013, 022, 023, 025, the ADR/task directory listings, and 13 additional repo files (crates/discovery, crates/core/src/input.rs, schemas/README.md, schemas/manifests/v1/spec.md, apps/discovery-cli) plus web sources for Steam userdata/controller_config layouts, Proton prefix layout and symlink fragility, Unreal INI/config hierarchy and Enhanced Input persistence, Source cfg grammar, and Unreal text encoding. No prior run artifacts existed in the design directory. Key research outputs: today's scan cannot name the main Steam root (roots are unioned and flattened) so 017-C2 is new plumbing; the closed ADR-012 input vocabulary (48 keyboard glyphs, no modifiers/named keys/axes) cannot spell most real bindings, which collides with 017-C3/L3; 013-B4 is the wrong bound for a codec family id (the spec uses the shared MAX_SLUG_OCTETS); bindings/v1's JSON 'document verbatim' makes it the second JSON format unless schemas/README is amended; 025-L5 forbids source keys in shipped artifacts, which the snapshot's lookup key and SteamID may violate; 013-R2/R3 name ADR-017 as owner and the ADR never mentions them; and reuse of list's scan drags 025-L1's mandatory catalog probe into a face the ADR calls local-by-construction. 15 open questions are handed to the designer.","commitMessage":"docs: add 017 research on keybinding extraction codec families"}
```
