# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/research.md @ 6fe69f965daf
- started: 2026-11-10T13:19:04.441Z
- ended: 2026-11-10T13:22:48.027Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/004-mirror-src-hierarchy-in-tests.md for this pipeline run.
Write your findings to docs/designs/004-mirror-src-hierarchy-in-tests/01-research.md.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary":"Researched ADR-004 (mirror src/ hierarchy in tools/orchestration/tests) and wrote docs/designs/004-mirror-src-hierarchy-in-tests/01-research.md. All of the ADR's load-bearing toolchain claims verified: tsconfig excludes tests/, no vitest config exists so the recursive default include discovers nested tests, gates.ts TEST_PATH_RE already matches nested paths, and all 26 test files only import ../src/... (never each other), so the move is git mv plus a uniform import-depth rewrite. Applied the prior-art and platform-notes lenses (Maven mirroring, Go colocation, Rust tests/; vitest 5 discovery+filtering, tsconfig, git rename detection). Flagged for the designer: the ADR says src/pipeline has 14 modules but it has 13; the 'import depth only' rule strands a stale path comment in pipeline-parallel-escalation.test.ts:45; and the flat tests/*.test.ts convention appears in two skill files (repo-practices:21 and test-strategy:27), not just the one the ADR names.","commitMessage":"docs: add research artifact for ADR-004 test-tree mirroring"}
```
