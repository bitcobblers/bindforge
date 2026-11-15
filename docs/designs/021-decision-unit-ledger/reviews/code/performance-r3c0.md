# Performance review — code/performance-r3c0

## Scope
git diff 1bda5c34f1561afd79fc395d63855b3d1194d08d…HEAD -- ':(exclude)docs/designs'
Files in scope:
- tools/orchestration/src/pipeline/ledger.ts (new)
- tools/orchestration/src/pipeline/gates.ts (frontmatterOf export)
- tools/orchestration/src/pipeline/runner.ts (ledgerGate hooks)
- tools/orchestration/tests/pipeline/ledger.test.ts
- tools/orchestration/tests/pipeline/ledger-join-gates.test.ts
- tools/orchestration/tests/pipeline/ledger-corpus.test.ts

Design reference: docs/designs/021-decision-unit-ledger/02-proposal.md
ADR reference: docs/adrs/021-decision-unit-ledger.md

## Checklist applied
- Corpus size bounded: design A23, 28 markdown files. Walk is recursive but shallow.
- Synchronous readers are intentional: the gate is a deterministic tree invariant checked at entry/finalize; the corpus is repo-local and tiny.
- No markdown parser dependency, no extra allocations beyond line splits per file.
- Regexes are compiled once at module top-level; look-behind is fixed-width and Node ≥22.23.5 compatible per A15.
- File reads are via node:fs readFileSync, one read per file per gate invocation. No repeated disk I/O within a single checkLedgers run.
- Frontmatter parsing is shared via frontmatterOf, CRLF tolerant per A13.
- Findings are bounded by MAX_LEDGER_FINDINGS and sorted once for stable output.

## Observations
- checkLedgers walks docs/adrs and schemas, collects .md paths, reads each once, extracts handles and parses ledgers. For the shipped corpus this is < 30 files; the cost is negligible and matches the design's "cheap" assumption A23.
- extractHandles and parseLedger each split content into lines. This duplicates the split per file, but the files are small and the duplication keeps the two readers independent and pure, as required by the design.
- frontmatterOf is called multiple times per file (frontmatterSpan, parseLedger, dependsOrder). The content is small; the extra regex execs are a micro-cost and do not change asymptotic behaviour.
- No unbounded loops, no catastrophic backtracking, no dynamic RegExp construction in hot paths.
- The gate is synchronous by design; pipeline stages are already synchronous checkpoint code. No event-loop starvation observed in the standing CI leg.

## Findings
No performance blocker, major, minor or nit findings. The implementation matches the design's performance assumptions and the standing CI leg is green per gate evidence:
`$ npm run check (exit 0) — green`
`$ cargo test --workspace --locked (exit 0) — green`
`| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;`

## Verdict
approve
