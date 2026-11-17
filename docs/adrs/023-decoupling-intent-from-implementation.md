---
doc_type: adr
status: proposed
date: 2026-11-16
ledger: yes
---

# ADR 023: Decouple architectural intent from implementation via tasks

This ADR is implemented out-of-band (023-R1): the pipeline cannot run the
change to its own entry gate, so a bootstrap lands it directly. A task run
carries its own task number into its design directory, `docs/designs/tasks/NNN-slug/`.

## Context

The BindForge pipeline is currently oriented around implementing Architectural Decision Records (ADRs). ADRs capture high-level architectural intent and rationale, which makes them difficult to use as single units of implementation work. ADRs often encompass a wide scope of changes, making progress tracking, dependency management, and granular code reviews cumbersome. Attempting to implement an entire ADR in a single pipeline run leads to scope creep and incomplete implementations.

## Decision

We will decouple architectural intent (ADRs) from implementation units (Tasks). The pipeline will become input-agnostic, operating on both ADRs and Tasks.

### Contracts

- **023-C1** The source document taxonomy. Every source document carries a `doc_type` field in its frontmatter: `adr` for architectural decisions, `task` for implementation units. The source path stays authoritative — `docs/adrs/` is an ADR, `docs/tasks/` is a task, anything else a brief — and a present `doc_type` that disagrees with the path is a usage error at entry. `doc_type: task` is required for a task document. → Rollout, item 1.
- **023-C2** Task source layout. Task source documents live in `docs/tasks/T-NNN-slug.md`, a dash separating the `T-` prefix from the number. → Rollout, item 2.
- **023-C3** Task artifact layout. Pipeline artifacts for a task run live in `docs/designs/tasks/NNN-slug/`, segregating them from ADR artifacts under `docs/designs/NNN-slug/`. → Rollout, item 2.
- **023-C4** Task frontmatter and admission. A task document carries `doc_type: task`, `status: pending`, `type`, and `functional_area`, plus optional `associated_adr`, `depends_on`, and `disposition`. The entry gate admits it only with `status: pending`, the required keys present, and a non-empty body. → Rollout, item 1.
- **023-C5** Task numbering. Tasks count in their own series from 001, separate from the ADR/design counter: the run's `NNN` is the task's number, read from the source file name, and a fresh number is allocated by scanning `docs/tasks/` and `docs/designs/tasks/`. → Rollout, item 2.

### Laws

- **023-L1** The runner detects the source kind. The runner reads the source path and frontmatter before any state is written and selects the lifecycle, the source gate, and the artifact directory from them. → Rollout, item 3.
- **023-L2** Task naming. A task source file is named `T-NNN-slug.md` — prefix, dash, three-digit number, dash, slug — and an entry from `docs/tasks/` that violates the name is refused at the source gate. → Rollout, item 3.
- **023-L3** Task lifecycle flip. When a task run completes, the runner flips the task's `status` from `pending` to `completed`; `disposition` is operator-managed and the runner never writes it. → Rollout, item 3.
- **023-L4** Task dependencies. A task's `depends_on` names tasks as `T-NNN-slug`; the entry gate refuses a malformed name, a missing dependency source file, or a dependency whose checkpoint is not `done`. → Rollout, item 3.

### Reservations

- **023-R1** Bootstrap implementation. The pipeline runner update this ADR commissions lands out-of-band: the pipeline cannot run the change to its own entry gate, so a bootstrap implements it directly, and the first in-pipeline work after the merge exercises the new path. Owner: `unassigned`. → Rollout, item 5.

### Seeds

- **023-S1** ADR backfill. `doc_type: adr` is added to every existing ADR and to `docs/adrs/template.md` so new ADRs carry the key, making the corpus invariant "every source document carries `doc_type`" hold for the ADR series. → Rollout, item 4.

**Unit handles.** The handles in this ADR sit at the end of the Rollout list items: one trailing code span of unit ids governs the line's claim, an id anywhere else governs nothing.

Rollout:

1. the taxonomy and the task frontmatter land with the source gate. `023-C1 023-C4`
2. the task layout, artifact layout and numbering land with identity derivation. `023-C2 023-C3 023-C5`
3. the detection, naming, flip and dependency laws land with entry and finalization. `023-L1 023-L2 023-L3 023-L4`
4. the ADR backfill lands. `023-S1`
5. the bootstrap implementation lands. `023-R1`

## Alternatives considered

**Keep ADRs as the sole unit of work.** This maintains the existing pipeline but continues the problem of large, unmanageable scopes and poor progress tracking.

**Use a separate tool for tasks.** This would fragment the workflow and lose the unified pipeline benefits for code review and artifact generation.

**Share the ADR counter for task numbers.** The first task would carry the next free ADR number (e.g. T-023), entangling two series that the `T-` prefix already separates. The separate series keeps each number meaningful in its own space; the segregated `docs/designs/tasks/` tree and the slug-carrying branch names keep the two series from colliding.

## Consequences

Becomes easy: implementation can be tracked at a granular level that maps to code changes and tests; design and code reviews can be scoped to specific tasks; failure in one task does not block an entire ADR; tasks can be ordered with `depends_on` and the runner refuses to start an unsatisfied one.

Costs: increased file count in `docs/tasks/` and `docs/designs/tasks/`; pipeline runner complexity increases; developers must learn to document work as tasks; `pipeline resume` must search two design roots and report clearly when a slug exists in both.

Not decided here: the exact set of values for `type` and `functional_area` (to be defined in a schema); the aggregation view for ADR-to-task progress.
