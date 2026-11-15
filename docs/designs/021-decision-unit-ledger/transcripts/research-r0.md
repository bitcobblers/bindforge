# Transcript: research r0

## Provenance
- stage: research (round r0)
- agent: research/researcher
- model: strata/qwen3.8-flash-next-nvfp4-nvfp4
- template: tools/orchestration/prompts/research.md @ 35133f43f622
- started: 2026-11-14T20:08:55.520Z
- ended: 2026-11-14T20:21:58.124Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":112152,"output":37940,"reasoning":0,"cache.read":3279854,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Research the topic of docs/adrs/021-decision-unit-ledger.md for this pipeline run.
Write your findings to docs/designs/021-decision-unit-ledger/01-research.md.

Your read set is: the source document at docs/adrs/021-decision-unit-ledger.md; the directory listing
of `docs/adrs/` (that listing is the ADR index — there is no index file); the
ADRs that source cites by number; and, if this design directory already holds
them, the immediately prior run's `01-research.md` and `02-proposal.md`.
Outside that set, read at most 25 more files. The ceiling is the point: reads
beyond the sources are context the run pays for twice.
Never open docs/designs/021-decision-unit-ledger/reviews/ or docs/designs/021-decision-unit-ledger/transcripts/ — run records
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
{"summary": "Wrote docs/designs/021-decision-unit-ledger/01-research.md for ADR-021 (the decision-unit ledger as the ADR system of record). Read set honoured: the source ADR, the docs/adrs/ listing as the index, the ADRs 021 cites by number (003, 004, 005, 006, 011, 012, 013, 014, 015, 016, 020) and the cited template.md; no prior 021 run artifacts exist, and no run records were opened. Four files beyond that set — gates.ts (parseSource, sourceAdmissible, stagePathAllowed, EXECUTABLE_SURFACE_PREFIXES, documentOutgrewFlow), runner.ts (entry path and ADR status flip), schemas/manifests/v1/spec.md (shipped handles and the Unit handles note), schemas/README.md. Two lenses applied (Prior art, Standards) under the Prior art section, with primary sources: adr-tools, MADR and its category ADR-0010, the Kubernetes KEP template, GitHub CODEOWNERS, Conventional Commits, RFC 8174, W3C 'Cool URIs don't change', and CommonMark. Findings are repo-local and dated to this branch: 021's own ten units carry no → pointer, so the gate it commissions fails its own source document; the shipped 013 pair already breaks 021-L2 and 021-L5 because 013-R5 has neither a handle nor an unassigned marker; 'handle at the end of a spec line' has fifteen table-row and three mid-sentence counterexamples in the only spec that ships handles; the manifests spec's prose cites ADR-016 where the ledger assigns ADR-019; nothing marks an ADR as ledger-bearing and no index gives 'reserved ADR number' a home; only implement/fix may write schemas/, so backfill write rights and 021-R1's unassigned owner are load-bearing; landing the gate crosses ADR-020's restart boundary. Eleven open questions handed to the designer, including handle grammar, gate scope during rollout, → target resolution, and the merged-tree wording of the finalize assert."}
```
