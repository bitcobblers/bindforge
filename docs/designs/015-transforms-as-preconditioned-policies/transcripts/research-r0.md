# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/research.md @ 6fe69f965daf
- started: 2026-11-13T15:58:13.833Z
- ended: 2026-11-13T16:02:04.297Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/015-transforms-as-preconditioned-policies.md for this pipeline run.
Write your findings to docs/designs/015-transforms-as-preconditioned-policies/01-research.md.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary": "Researched ADR-015 (transforms as named, preconditioned policies) and wrote docs/designs/015-transforms-as-preconditioned-policies/01-research.md. Verified the internal substrate the ADR stands on (AxisKind/registry, placement_is_legal's structured PlacementCheck, manifest constraint entries, chain_ops' shipped load/resolve/fork/diff/merge, and the contract tree's pre-announced `policy` format slot), and surveyed primary-source prior art under three lenses: XKB options/variants, DualSense Edge profiles, Xbox button mapping, PowerToys, SharpKeys, xremap/input-remapper, Steam Input, Terraform preconditions, CSS @supports, RFC 6902 test-op, SDL_GameControllerDB, HID usage tables/evdev, plus security classes for untrusted policy documents. Key gaps flagged for the designer: precondition vocabulary grammar, cluster/handedness not being data in core today, tag-glob semantics, transform output's representation in a chain, stack-order semantics, and report shape/budgets. Every claim carries an inline source (local path or verified URL).", "commitMessage": "docs: add research artifact for ADR-015 transform policies"}
```
