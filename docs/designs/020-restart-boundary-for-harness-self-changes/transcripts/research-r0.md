# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/research.md @ 35133f43f622
- started: 2026-11-13T19:55:33.880Z
- ended: 2026-11-13T19:58:53.765Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":151233,"output":25217,"reasoning":0,"cache.read":1250912,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/020-restart-boundary-for-harness-self-changes.md for this pipeline run.
Write your findings to docs/designs/020-restart-boundary-for-harness-self-changes/01-research.md.

Your read set is: the source document at docs/adrs/020-restart-boundary-for-harness-self-changes.md; the directory listing
of `docs/adrs/` (that listing is the ADR index — there is no index file); the
ADRs that source cites by number; and, if this design directory already holds
them, the immediately prior run's `01-research.md` and `02-proposal.md`.
Outside that set, read at most 25 more files. The ceiling is the point: reads
beyond the sources are context the run pays for twice.
Never open docs/designs/020-restart-boundary-for-harness-self-changes/reviews/ or docs/designs/020-restart-boundary-for-harness-self-changes/transcripts/ — run records
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
{"summary": "Wrote docs/designs/020-restart-boundary-for-harness-self-changes/01-research.md. Read the source ADR 020, its cited ADRs (006, 007, 008, 010, 016), the ADR index listing, and the harness code that defines the restart boundary's context: prompts.ts (loadTemplate, strict render, TEMPLATE_DEFAULTS, templateHash), runner.ts (stage/pass commit sites, ownership sweep with per-commit path lists, lock re-assertion, StopDriving, finalizeEscalate, status renderer, stage-entry template pinning), state-store.ts (ADR-006 checkpoint grammar and status enum), types.ts (RunStatus, reserved names), states.ts (planStop sticky-record precedent), cli.ts (exit codes 0/2/1), stages.ts, and docs/workflows.md — 8 files beyond the mandated set. Applied the prior-art and platform-notes lenses with verified primary sources (Node module cache docs, Erlang code loading, systemd ExecReload/notify-reload, Kubernetes rolling updates, nodemon, webpack HMR, sysexits(3)). Key research outputs: the detection hook likely reuses enforceOwnership's git diff-tree path lists; the new state collides with hard-coded running|done|escalated enums in types.ts and the checkpoint zod schema; the CLI currently maps every non-done status to exit 2; commit-site coverage must include review-pass and revert commits; the planStop sticky-field is a representation precedent. Open questions for the designer cover enum-vs-field choice, exit code, boundary count limit, agent-permission-file scope, and marker/checkpoint authority.", "commitMessage": "docs: add research for ADR-020 restart boundary"}
```
