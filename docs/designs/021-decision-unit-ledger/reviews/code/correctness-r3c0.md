# Correctness review — r3c0

## Scope
git diff 1bda5c34f1561afd79fc395d63855b3d1194d08d...HEAD -- ':(exclude)docs/designs'

## Summary
The ledger readers and join gate implement the components named in 02-proposal.md:
* `tools/orchestration/src/pipeline/ledger.ts` exports `parseLedger`, `extractHandles`, `checkLedgers`, `ledgerGate`, `MAX_LEDGER_FINDINGS` and the twelve finding codes.
* `gates.ts` exports `frontmatterOf` shared with `parseSource`.
* `runner.ts` hooks `ledgerGate` at the single `sourceAdmissible` call site and at `finalizeDone` before `flipAdrStatus`, gate-logged as `ledger-join`.

The implementation follows the anchored id grammar, handle grammar, CRLF/BOM posture, fence/frontmatter skipping, pointer detection outside code spans, reservation owner forms, `depends-order`, note-missing, and the two-direction join with `unassigned` exception. Error paths deny closed.

Deviations named in 03-build-notes.md are present and documented:
* `parseLedger` third optional `file` parameter for finding attribution.
* Per-kind number non-increasing reported as `unit-duplicate`.
* `unit-misplaced` units remain registered.
* `depends-order` ignores non-three-digit entries.
* Bullet text includes indented continuations.
* Conformance split between coder-owned `schemas/**` and runner-owned `docs/adrs/**`.

Assumption register rows A1–A24 are honoured by the readers; the corpus walk is `docs/adrs` + `schemas`, paths are built with `node:path`, no writes occur, and the gate runs at entry and finalize.

No undeclared deviation was found. No scope creep beyond the documented prompts fix in build notes deviation 8, which is recorded as a prior-round editorial correction and does not alter the ledger contract.

## Findings
None.

## Verdict
approve
