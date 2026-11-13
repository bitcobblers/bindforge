# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/research.md @ 6fe69f965daf
- started: 2026-11-13T12:58:34.592Z
- ended: 2026-11-13T13:04:15.444Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/013-game-manifests.md for this pipeline run.
Write your findings to docs/designs/013-game-manifests/01-research.md.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary": "Wrote docs/designs/013-game-manifests/01-research.md for ADR-013, scoped to the ADR's question (manifest as the game-facts contract, not codecs, presets, or authoring UX). Closest prior art is Valve's Steam Input IGA file and Action Manifest File (game id in the file name, closed action-kind vocabulary, major/minor revisions whose major flags breaking action-set changes), with SDL's GUID mapping line as prior art for a reserved codec family id, Unity's rename-surviving action id, Source-engine cfg structure, and Elite Dangerous .binds as the cautionary case; no surveyed system encodes exclusivity or docked-key facts as data. Repo findings pin what a manifests/v1 format inherits: one new Document arm plus SUPPORTED_FORMATS/format_majors entries (supported_majors() already declares v1 and v2, so the ADR's consequence overstates that change), automatic conformance-corpus pickup with a mandatory serialize round-trip, the flat one-line-array grammar that forces space-joined encodings against game-native names that contain spaces, the axis-kind duplication against registry.toml (40 ids), the 48-member keyboard: vocabulary that cannot name the keys a 'dead key' constraint is about, and no existing machinery for cross-file re-validation of dependents. Three lenses applied (Prior art, Standards, Security landscape); every external claim was fetched and verified, and claims whose sources could not be verified (Valve Developer Community, PCGamingWiki, a Steam Workshop item, steaminputdb.com) were dropped or re-sourced rather than cited.", "commitMessage": "docs: add research for ADR-013 game manifests"}
```
