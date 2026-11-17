---
doc_type: task          # the source document's kind: task (ADR-023) — required
status: pending         # pending | completed — the pipeline flips pending -> completed on completion
date: YYYY-MM-DD
type: feature           # feature | enhancement | defect | ...
functional_area: tooling # tooling | application | backend service | ...
associated_adr: ADR-NNN # optional — the ADR this task serves
depends_on: []          # optional — task names (T-NNN-slug); each must exist and be done
disposition:            # optional, operator-managed — blocked | cancelled | deferred | ...
flow: light             # optional — full (default) | light (ADR-010)
---

# T-NNN: <short imperative title>

Name the file `T-NNN-slug.md` after this template: the number is the task
series' next free number (tasks count from 001, separately from ADRs), and
the slug is the lowercased, dash-joined title.

Tasks are the units of work beneath an ADR: small, concrete changes the
system requires. A task that serves an ADR names it in `associated_adr`; a
task that needs another task's work first lists it in `depends_on` — the
pipeline refuses to start this task while any dependency's run is not
`done`.

State the change in a few paragraphs: what exists today, what the task
changes, and how "done" is recognizable. No Context/Decision sections are
required; the body must simply be non-empty.
