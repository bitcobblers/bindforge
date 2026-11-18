# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/research.md @ c978d4b12120
- started: 2026-11-18T02:44:54.767Z
- ended: 2026-11-18T02:49:29.929Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":68971,"output":15025,"reasoning":0,"cache.read":502580,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/tasks/T-001-steam-linux-discovery.md for this pipeline run.
Write your findings to docs/designs/tasks/001-steam-linux-discovery/01-research.md.

Your read set is: the source document at docs/tasks/T-001-steam-linux-discovery.md; the directory
listings of `docs/adrs/` and `docs/tasks/` (those listings are the ADR and
task indexes — there are no index files); the ADRs that source cites by
number — for a task, the ADR named by its `associated_adr` frontmatter; and,
if this design directory already holds them, the immediately prior run's
`01-research.md` and `02-proposal.md`.
Outside that set, read at most 25 more files. The ceiling is the point: reads
beyond the sources are context the run pays for twice.
Never open docs/designs/tasks/001-steam-linux-discovery/reviews/ or docs/designs/tasks/001-steam-linux-discovery/transcripts/ — run records
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
{"summary":"Rewrote docs/designs/tasks/001-steam-linux-discovery/01-research.md for T-001 (Linux Steam discovery plugin), re-verifying the repo seam first-hand — the two-method DiscoveryPlugin trait, five-field DiscoveredGame with Platform::SteamLinux, normalize_slug, the error-swallowing DiscoveryEngine, DummyPlugin-only CLI, the 32-package Cargo.lock with no VDF parser or tempfile, and the CLI test that asserts `[]` — and re-fetching every external citation this run. Format evidence is consistent: ACF text KeyValues rooted at AppState carrying appid/name/StateFlags/installdir, composition rule <library>/steamapps/common/<installdir>, and StateFlags bit 4 = StateFullyInstalled per the Lutris table. New evidence added over the prior pass: three current Rust scanners (fragcap-steam, steamie-core, yandere-core) that hand-roll a small KeyValues reader instead of taking keyvalues-parser+pest, the pre-2021 vs modern libraryfolders.vdf shapes that already caused a silent-zero-libraries bug, per-game Proton prefix directories holding look-alike libraryfolders.vdf copies, and divergent backslash-escape handling across implementations. Lenses applied: prior art, platform notes (root candidate list, Flatpak sandbox visibility, arbitrary mounts, Rust 1.85/edition-2024 and --locked gate), and security landscape under Risks & unknowns (install_path as a privilege hand-off, whole-scan blast radius, diagnostic leakage). The designer's decisions are enumerated as 19 open questions: root policy, test seam and fixtures, dependency posture, grammar subset and escaping, non-game filtering, StateFlags and missing-name handling, path validation, per-file failure policy, ordering/dedup, error taxonomy, and performance budget.","commitMessage":"docs: research Linux Steam discovery plugin for T-001"}
```
