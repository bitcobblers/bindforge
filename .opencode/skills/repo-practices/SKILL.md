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
- `npm test` — vitest; tests in `tests/*.test.ts`.
- `npm run format` — run before finishing; import order is
  `node:` externals, then relative, blank line between groups.

## Tests

- Unit tests must not touch the network or a live server — stub the
  client (see `tools/orchestration/tests/opencode-runner.test.ts`).
- Integration tests use temp directories and clean up after themselves.

## Commits

Conventional Commits (AGENTS.md): `<type>: <short imperative description>`,
types `feat fix refactor docs test chore build`, optional scope like
`feat(orchestration): ...`.

## Naming

Kebab-case file and directory names; ids lowercase kebab-case.
