# Transcript: test-update r1

## Provenance
- stage: test-update (round r1)
- agent: code/test-engineer
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/test-update.md @ 3e580b3c3a3f
- started: 2026-11-13T12:39:35.749Z
- ended: 2026-11-13T12:44:23.111Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Update the package's tests to match the approved design at
docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md after the round 1 changes
(see: docs/designs/012-input-vocabulary-and-reach-classes/reviews/code/correctness-r1c0.md docs/designs/012-input-vocabulary-and-reach-classes/reviews/code/compatibility-r1c0.md).
Change tests only where the design or an accepted review finding changed.

To move or rename a file, do not touch the paths yourself —
declare it in `moves`, and the runner performs it.
You own the test files you write: before replying, run the package formatter
(`npx prettier --write`) on the files you touched.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"commitMessage":{"type":"string","minLength":3},"moves":{"default":[],"maxItems":64,"type":"array","items":{"type":"object","properties":{"from":{"type":"string","minLength":1,"maxLength":200},"to":{"type":"string","minLength":1,"maxLength":200}},"required":["from","to"],"additionalProperties":false}}},"required":["moves"],"additionalProperties":false}
```

## Raw reply
```text
{"commitMessage": "test(core): update the suite for the round-1 review fixes\n\nPer-format version gate (correctness + compatibility finding 1):\n- version_gate.rs pins a `profile`, `action-ids` and `keyboard-geometry`\n  document declaring `v2` refused as `UnsupportedVersion`, naming that\n  format's own supported majors, and pins the new corpus fixture\n  `profile/v1/fixtures/invalid/major-not-shipped.toml` by name.\n- `version_gate.rs` and `action.rs` now assert the refusal names the majors\n  the format's corpus carries (new `common::format_majors`), not the\n  crate-wide `supported_majors()` union; the equality case keeps the union.\n\nASCII member set (compatibility finding 2):\n- `input.rs` refuses `keyboard:é` — the closed set is written `{a-z}`/`{0-9}`.\n- `reach_v2.rs` pins `reach-classes/v2/fixtures/invalid/non-ascii-member.toml`\n  as refused naming the member.\n\nClause (a), targets distinct (correctness finding 3):\n- `placement.rs` runs the case on a move set that really shares one delta\n  (`a -> s` twice), asserting `delta == Some((1, 0))` beside `!shape_ok`, and\n  keeps the differing-delta set as the pin for the common-delta half.\n\nNot pinned: the serializer's version-only `keys` branch (correctness finding 2)\n— reach tables stay asserted by load, not re-serialization, and that state is\nreachable only from a hand-built model. The `zz_api_probe.rs` inventory nit is\nbookkeeping for the notes leg, not a test change.\n\nGate legs run here: `cargo fmt --all -- --check` clean, `cargo clippy\n--workspace --all-targets --locked -- -D warnings` clean, `cargo test\n--workspace --locked` green — 15 targets, 122 passed, 0 failed, 0 ignored.", "moves": []}
```
