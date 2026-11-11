# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/research.md @ 6fe69f965daf
- started: 2026-11-11T18:15:04.536Z
- ended: 2026-11-11T18:18:30.320Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/008-rust-for-the-product.md for this pipeline run.
Write your findings to docs/designs/008-rust-for-the-product/01-research.md.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":3}},"required":["summary"],"additionalProperties":false}
```

## Raw reply
```text
{"summary": "Researched ADR-008 (Rust for the product, TypeScript confined to the pipeline) and wrote docs/designs/008-rust-for-the-product/01-research.md. Verified all fixed framework choices against primary sources as of Nov 2026: cargo workspaces (cargo check --workspace, flat-members glob, workspace inheritance), clap 4.6.7, axum 0.8.9/0.9-in-dev, sqlx 0.9 (now under transact-rs), Tauri v2 (2.12 current, stable since the tauri-2.0 release), TOML v1.1.0, and toml_edit for format-preserving codecs. Applied three lenses: prior art (kanata, xremap, Modrinth App, Goverlay, Heroic, JSON Schema Test Suite), platform notes (Proton compatdata prefixes, WebView2 distribution, WebKitGTK deps, Flatpak sandbox vs filesystem scanning), and standards (formats-as-contract, evdev key naming, axum body limits). Key findings: sqlx compile-time checks complicate the single-gate story, WebView2 is not guaranteed on all Windows 10 devices, and the deferred Tauri webview UI creates an explicit TypeScript boundary question for the designer. Every claim carries an inline source URL.", "commitMessage": "docs: add research artifact for ADR-008 Rust product decision"}
```
