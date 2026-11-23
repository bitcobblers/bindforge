# bindforge

BindForge is an open framework for creating, sharing, and translating game input bindings.

## Conventions

- **Semantic commits**: every commit message follows the Conventional Commits format — `<type>: <short imperative description>` (e.g. `feat: add retry strategy to sync client`). Common types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `build`.
- **Branching**: main checkout stays on `main`. Pipeline runs must be launched from a dedicated worktree. Never move the main checkout off `main` for a run.
- **Formatting & lint**:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- **Test gates**:
  - `cargo test --workspace --locked`
  - `npm ci && npm run check` in `tools/orchestration` (Node 22.23.4)
- CI runs the cargo trio and the npm check. See `.github/workflows/check.yml`.

## Repo layout

- Product core: Rust crates, pinned by `rust-toolchain.toml`.
- Orchestration: `tools/orchestration` Node CLI that drives a running OpenCode server via `@opencode/client` SDK.
- Schemas: `schemas/` versioned specs with `spec.md` per version – profile, manifest, policy, action-ids, reach-classes, keyboard-geometry.
- Data: `data/profiles/`, `data/manifests/`.
- Docs: `docs/adrs/` (all ADRs), `docs/tasks/` (all tasks), `docs/designs/`, `docs/workflows.md`. ADRs and tasks stay in these folders permanently — accepted ADRs and completed tasks are retained as historical record, never removed.

## Development

- Use worktrees for pipeline work: `pipeline run <source>` and `pipeline resume <slug>` must be issued from the run's worktree.
- The runner owns commits and file moves. Stage agents do not commit.
- Stage ownership is enforced by the runner: changed paths outside a stage's ownership are reverted after each stage.

## Pipeline usage

- Entry points:
  - `pipeline run <source-path>` to start or continue a run
  - `pipeline resume <design-slug>` to continue from checkpoint
  - `pipeline status <design-slug>` for current state
- Source documents are ADRs (`docs/adrs/NNN-slug.md`), tasks (`docs/tasks/T-NNN-slug.md`, ADR-023), or briefs (`docs/prompts/*.md`).
  - An ADR must have frontmatter `status: proposed` with non-empty Context and Decision to enter the pipeline.
  - A task must have frontmatter `doc_type: task`, `status: pending`, `type`, and `functional_area` with a non-empty body; optional `associated_adr` and `depends_on` (task names, each must exist and be `done` before the run starts).
  - Every source document carries a `doc_type` key (`adr`/`task`); a `doc_type` that disagrees with the document's path is a usage error.
- The pipeline flips the source's `status` itself on completion: ADR `proposed` → `accepted`, task `pending` → `completed`.
- Light flow is selected with frontmatter `flow: light` (ADR-010). Otherwise the full flow runs.
- Design artifacts live under `docs/designs/<NNN-slug>/` (task runs under `docs/designs/tasks/<NNN-slug>/`) with `01-research.md`, `02-proposal.md`, `03-build-notes.md`, reviews, transcripts, `.pipeline.json`.

## Agent / shell rights

- Stage agents have read-only inspection and toolchain allowlists:
  - Allowed: `cargo *`, `rustfmt *`, `npm run *`, `npm test *`, `npx vitest *`, read-only git/ls/cat/grep/find.
  - Denied: commits, branch state changes, push, `rm`/`mv`, `tee`/`sed -i`/`dd`, writes to `transcripts/`, `reviews/**`, `.pipeline.json`.
- The runner owns moves per ADR-005 and reasserts `.pipeline.lock` at every stage boundary.

## Review norms

- Design review panel: correctness, compatibility, security, performance, complexity.
- Funneled passes: c0 is full panel, subsequent passes review only dissenting seats, then ratification.
- Code review seats: full flow = 5 seats; light flow = correctness + testing.
- Verdicts and severities are normalized before validation. Reports and transcripts are the run record, not review input.

## References

- `README.md` for contributor prerequisites and system dependencies.
- `docs/workflows.md` for detailed pipeline mechanics, budgets, and escalation.
- ADRs in `docs/adrs/` for architectural rationale.
