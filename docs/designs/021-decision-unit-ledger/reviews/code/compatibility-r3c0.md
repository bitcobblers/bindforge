# Compatibility review — r3c0

## Scope
Diff 1bda5c34f1561afd79fc395d63855b3d1194d08d…HEAD -- ':(exclude)docs/designs' against design 02-proposal.md and ADR-021.

## Compatibility checklist

### 1. Platform code matches Compatibility matrix
Supported platform is Linux per matrix. No OS-specific branches are introduced. `node:path` `join`/`basename` are used for filesystem access. No platform-only imports. macOS/Windows are listed as “Not tested — expectation only”, and the implementation relies on POSIX-style reporting paths which is the stated expectation.

### 2. Node floor
Package `engines.node` is `>=22.23.5`. Code uses look-behind regex `(?<![\dA-Za-z-])` and `\b`-style guards, `node:fs` sync APIs, `withFileTypes:true`. All available at the floor. No new dependencies.

### 3. Paths
Filesystem paths are built with `node:path.join`. Repo-root-relative reporting paths are emitted as POSIX `docs/adrs/...` strings per design ¶1 and the `file` field comment. The walk uses `join(root, entry.name)` recursively; `collectMarkdown` builds the reporting `rel` via template literal and later reconstructs the filesystem path with `join(repoDir, ...rel.split('/'))`. This is the design’s stated POSIX reporting convention. No case-sensitive filename assumptions are made; corpus names are lowercase kebab per repo-practices.

### 4. Guarded platform imports
No platform-only modules are imported. All imports are Node built-ins or project modules.

### 5. Locale / layout reality
Ledger IDs are ASCII `NNN-[CLBRS]\d{1,2}`. File reading is UTF-8 with BOM stripping and CRLF folding. Handle extraction is line-based and code-span based; non-ASCII content in prose does not affect the grammar. No binding-name handling is introduced.

### 6. Native dependencies
No native dependencies added. No packaging or CI matrix change.

### 7. Tests vs platforms
Test strategy #13 pins Linux execution for #1–#12; macOS/Windows rely on CRLF pin and lowercase-kebab names. No platform-specific tests are introduced and none are skipped. The standing CI leg `npm run check` runs on ubuntu-latest with Node 22.23.5 per A24.

## Findings
No compatibility blockers or majors identified. The implementation follows the design’s stated POSIX reporting convention and Linux execution matrix.
