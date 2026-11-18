# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/research.md @ c978d4b12120
- started: 2026-11-18T05:25:05.658Z
- ended: 2026-11-18T05:28:45.523Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":97671,"output":22433,"reasoning":0,"cache.read":650115,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/tasks/T-002-steam-windows-discovery.md for this pipeline run.
Write your findings to docs/designs/tasks/002-steam-windows-discovery/01-research.md.

Your read set is: the source document at docs/tasks/T-002-steam-windows-discovery.md; the directory
listings of `docs/adrs/` and `docs/tasks/` (those listings are the ADR and
task indexes — there are no index files); the ADRs that source cites by
number — for a task, the ADR named by its `associated_adr` frontmatter; and,
if this design directory already holds them, the immediately prior run's
`01-research.md` and `02-proposal.md`.
Outside that set, read at most 25 more files. The ceiling is the point: reads
beyond the sources are context the run pays for twice.
Never open docs/designs/tasks/002-steam-windows-discovery/reviews/ or docs/designs/tasks/002-steam-windows-discovery/transcripts/ — run records
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
{"summary": "Research artifact written to docs/designs/tasks/002-steam-windows-discovery/01-research.md. It covers the ADR-022 Windows backend contract, the existing SteamLinuxPlugin seams to share (keyvalues parser with \\\\-unescape, libraries_in_root, parse_manifest, canonical-path rule, degraded-scan/error semantics), the community-converged host-probing convention (HKCU\\Software\\Valve\\Steam SteamPath stored forward-slash-delimited, HKLM SOFTWARE\\Valve\\Steam InstallPath in the 32-bit view, %ProgramFiles(x86)%/%ProgramFiles% defaults), WOW64 registry view flags (KEY_WOW64_32KEY/64KEY) and HKCU's shared nature, Windows path hazards (backslash separator justifying the mandated installdir refusal, reserved device names, \\?\\ canonicalize prefixes), the winreg-vs-windows-core dependency choice gateable via [target.'cfg(windows)'], and cross-compile compile-verification with cargo check --target x86_64-pc-windows-msvc. Every claim carries an inline source; key unknowns flagged for the designer: steamapps/ vs config/ libraryfolders.vdf location, registry value type/expansion, emitted-path \\?\\ handling, and the shared-core decomposition shape. Prior art: steam-game-path, Lutris config.py, Playmoir KB, Whisky docs, plus the repo's own Linux plugin. Sources read within budget: 8 repo files plus the ADR/task pair; external citations verified via targeted web lookups.","commitMessage":"docs: add research for T-002 Windows Steam discovery"}
```
