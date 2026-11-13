# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/research.md @ 6fe69f965daf
- started: 2026-11-13T03:35:41.761Z
- ended: 2026-11-13T03:40:07.541Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/011-action-taxonomy-and-core-skeleton.md for this pipeline run.
Write your findings to docs/designs/011-action-taxonomy-and-core-skeleton/01-research.md.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary":"Researched ADR-011 (action taxonomy + crates/core skeleton) and wrote docs/designs/011-action-taxonomy-and-core-skeleton/01-research.md. Applied three lenses — prior art (Steam Input IGA files, Unity Input System actions/GUIDs, SDL GameControllerDB, evdev key-code registry, libretro remap layering, XAG 107), standards/conformance (JSON Schema Test Suite and toml-test golden-fixture harnesses, RFC 1035 label grammar, semver deprecation vocabulary, HID/evdev lineage), and platform notes (cargo workspace globs + virtual-manifest resolver, edition 2024 ⇒ Rust 1.85, rust-toolchain.toml pinning, CI gates). Every claim carries a source URL. Key findings for the designer: 'actions not inputs' identity layers are industry-proven; published id sets are one-way doors needing written extension/deprecation discipline; and the branch already contains crates/core and schemas/ scaffolding that contradicts the ADR's greenfield framing — flagged as the top risk with open questions on the ~40-id list, id grammar, and harness portability.","commitMessage":"docs: add research for action taxonomy and core skeleton"}
```
