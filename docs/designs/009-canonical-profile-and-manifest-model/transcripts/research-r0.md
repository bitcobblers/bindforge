# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/research.md @ 6fe69f965daf
- started: 2026-11-11T22:24:38.174Z
- ended: 2026-11-11T22:28:21.411Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/009-canonical-profile-and-manifest-model.md for this pipeline run.
Write your findings to docs/designs/009-canonical-profile-and-manifest-model/01-research.md.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary": "Researched ADR-009's five-contract model (action taxonomy, reach classes, profile chains, transform policies, game manifests) and wrote docs/designs/009-canonical-profile-and-manifest-model/01-research.md using three lenses: Prior art (Unity/Unreal/Godot named actions, QMK + Steam Input layer overlaying, Lutris declarative manifests, SDL_GameControllerDB, kanata/xremap, Returnal's handedness presets — no incumbent cross-game binding format exists, so schemas/ fills a gap), Standards (HID usage page 0x07, SDL3 scancodes as the layout-independent naming axis vs the ADR's layout-dependent glyph naming, evdev/X11 keysyms/Windows VK, W3C standard gamepad, TOML v1.1.0 + SemVer), and Security landscape (untrusted-deserialization DoS advisories RUSTSEC-2019-0025/-2024-0012 vs the repo's existing MAX_NESTING_DEPTH/MAX_DOCUMENT_BYTES/MAX_CHAIN_DEPTH guards, path traversal via path-like based-on refs, SLSA provenance-is-not-trust). Key findings for the designer: dotted action ids collide with TOML dotted keys and the repo's v1 grammar refuses dotted keys (format decision needed); reach-class tables have no external standard dataset and become an authored compatibility promise; the 'IJKL breaks the home-row anchor' claim is contestable and belongs to the reach tables; merge semantics over RFC 6902-style op chains are unspecified.", "commitMessage": "docs: add research for canonical profile and manifest model"}
```
