# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/research.md @ 6fe69f965daf
- started: 2026-11-13T11:25:41.888Z
- ended: 2026-11-13T11:29:17.229Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/012-input-vocabulary-and-reach-classes.md for this pipeline run.
Write your findings to docs/designs/012-input-vocabulary-and-reach-classes/01-research.md.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary": "Wrote docs/designs/012-input-vocabulary-and-reach-classes/01-research.md. Research covered the shipped reach-classes/v1 contract (closed entry set, grammar subset, 1 MiB/16-deep budgets, version gate, placeholder ids and the tests that pin them), the input-vocabulary prior art across USB HID usage page 0x07, Windows scan codes, W3C KeyboardEvent code/key, XKB components, evdev KEY_/BTN_, QMK glyph keycodes with language-specific alias sets, and the W3C Gamepad API plus SDL's controller DB; per-hand reach grading has no external standard, with the touch-typing home row (ASDF/JKL;) as the closest asymmetry-encoding baseline. Three lenses applied (Prior art, Standards, Platform notes). Noted two ADR-vs-tree discrepancies (right.toml's 'cross' id; comments referencing ADR-012 not ADR-009) and surfaced open questions on the v2 keys encoding, class ordering for the legality predicate, table coverage of modifiers/Space, and geometry addressing. Every factual claim carries an inline source URL.", "commitMessage": "docs: add research artifact for ADR-012 input vocabulary and reach classes"}
```
