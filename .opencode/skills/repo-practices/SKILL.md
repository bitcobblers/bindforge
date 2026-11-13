---
name: Repository practices
description: BindForge repo conventions — TypeScript tooling, package layout, tests, formatting, semantic commits
---

Conventions for code landing in this repository.

## Packages

- Tools live under `tools/<name>/` with `src/`, `tests/`, `README.md`,
  `package.json`, `tsconfig.json`, `eslint.config.js`, `.prettierrc`.
  Copy an existing package's config files rather than writing new ones.
- ESM with NodeNext resolution: relative imports end in `.js` even in
  TypeScript source.
- TypeScript: `strict` plus `noUncheckedIndexedAccess`. No `any`; validate
  external data at the boundary with zod and type from the schema.

## Commands (from each package's package.json)

- `npm run check` — lint + format check + tests. This is the gate.
- **Run the gate legs yourself, in your own shell, before you reply** —
  `cargo fmt --all -- --check` and friends are allowed to every code stage.
  Quote the real output in the build notes. Never assert a formatting, lint or
  test posture you did not execute: a stage that reasons about rustfmt instead
  of running it spends a fix round discovering the gate disagrees.
- `npm test` — vitest; tests in `tests/**/*.test.ts`.
- `npm run format` — run before finishing; import order is
  `node:` externals, then relative, blank line between groups.

## Tests

- A directory under `tests/` mirrors a subsystem directory under `src/`;
  within it, a module-named file (`state-store.test.ts`) is that module's
  unit test and cross-module behavior suites keep descriptive names
  (`funnel.test.ts`). New subsystems follow the same shape.
- Unit tests must not touch the network or a live server — stub the
  client (see `tools/orchestration/tests/opencode-runner.test.ts`).
- Integration tests use temp directories and clean up after themselves.

## Commits

Conventional Commits (AGENTS.md): `<type>: <short imperative description>`,
types `feat fix refactor docs test chore build`, optional scope like
`feat(orchestration): ...`.

## Naming

Kebab-case file and directory names; ids lowercase kebab-case.

## Rust (ADR-008)

- Product code is Rust only: crates under `crates/<name>/`, apps under
  `apps/<name>/`, in the cargo workspace at the repo root. Workspace
  package names carry the `bindforge-` prefix (a bare `core` collides with
  Rust's implicit `core` name — cargo issue 7760). Edition, `rust-version`
  (the 1.85 floor), and license are inherited from `[workspace.package]`;
  the toolchain is pinned by `rust-toolchain.toml`. `members` lists only
  globs that match an existing member — a glob matching none aborts every
  gate command at manifest load, and git tracks no empty directories, so
  `apps/*` joins with the first app crate.
- The cargo gate mirrors `npm run check`:
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked`.
  `Cargo.lock` is committed; CI uses `--locked`. Node floor for the
  orchestration gate: 22.23.4 (`tools/orchestration` `engines.node`).
- Tests live under each crate's `tests/`, mirroring `src/` (ADR-004):
  a module-named file is that module's unit test, cross-module suites keep
  descriptive names; unit tests must not touch the network.
- Outside `tools/orchestration`, JS/TS source may live only in
  `apps/desktop`'s webview frontend, presentation only. Capability files
  under `apps/desktop/src-tauri/capabilities/` enumerate only the app's own
  commands; the `fs`, `shell`, and `dialog` plugins are not enabled — file
  access happens in workspace crates.
- The scheduled `.github/workflows/advisories.yml` scan (cargo-deny, pinned
  version, root `deny.toml`) is triage input, never a PR check; a waiver
  names the advisory id in `deny.toml`. It runs the full `cargo deny check`,
  so every section `deny.toml` configures has a consumer, and it builds the
  scanner on its own pinned toolchain: cargo-deny's MSRV sits above the
  product pin, so building it under `rust-toolchain.toml`'s channel fails and
  the scan silently stops running. Its shared options are root-level in 0.20,
  so they precede the subcommand (`cargo deny --format sarif check`); a flag
  after `check` is a parse error that exits before scanning. The report is
  captured on the stream the pinned tool actually writes it to: in 0.20,
  `--format json` sends every record to stderr and only SARIF goes to stdout.
  The guard step requires a SARIF document by content (`"runs"`), not mere
  non-emptiness, and the report is uploaded as a SHA-pinned
  `actions/upload-artifact` step — the week's triage record must outlive the
  runner.
- The `schemas/` grammars are documented subsets of TOML v1.1.0: the subset
  is stated in the contract tree (`schemas/README.md` and each `spec.md`),
  never only in the reader, and each format's entry set is closed — an entry
  outside it is a typed refusal, not an ignored key. The parse-budget
  constants (`MAX_DOCUMENT_BYTES`, `MAX_NESTING_DEPTH`) are exported beside
  `MAX_CHAIN_DEPTH` so callers cap input before they allocate it.
