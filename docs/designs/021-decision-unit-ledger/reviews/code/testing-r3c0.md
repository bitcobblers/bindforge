# Code review — testing seat, r3c0

## Scope
git diff 1bda5c34f1561afd79fc395d63855b3d1194d08d...HEAD -- ':(exclude)docs/designs' against 02-proposal.md, ADR-021, and gate evidence.

## Summary
The testing surface implements the Test strategy #1–#12 from the proposal. The ledger readers, join gate, entry hook, finalize hook and standing CI leg are present and exercised by the module-named unit tests and the cross-module integration suites. The gate record shows `npm run check` green and `cargo fmt/clippy/test` green. No testing regressions are observed against the design.

## Checklist evidence

### Unit readers
* Anchored unit-id grammar accepts `013-C1`, `021-R12`, multi-id spans and cell-final ids; refuses `1234-C1`, `013-C1x`, `013-C123`, `013-X1`, `v013-C1`. `ledger.test.ts` #1.
* Handle grammar returns ids for trailing code spans, multi-id spans and table-cell finals; returns nothing for mid-sentence, fenced, frontmatter or trailing-word ids. `ledger.test.ts` #2.
* CRLF tolerance yields identical handle sets and findings for LF/CRLF twins. `ledger.test.ts` #3.
* `parseLedger` accepts the 013 shape, reports `pointer-missing` for pre-run shape, `unit-kind-mismatch`, `unit-misplaced`, `unit-duplicate`, `ledger-empty`. `ledger.test.ts` #4.
* Reservation owners: `owner-missing`, `owner-unknown`, clean forms `ADR-019`, `` `unassigned` ``, bare `unassigned`, `this ADR's run`. `ledger.test.ts` #5.
* Total join both directions: `handle-dangling`, `unit-unhandled` except unassigned, cross-ledger resolution. `ledger.test.ts` #6.
* `depends-order` on `depends-on` larger than own number, checked regardless of `ledger:` key. `ledger.test.ts` #7.
* `note-missing` for handle-bearing file without `**Unit handles.**` note; silent for prose-only ids. `ledger.test.ts` #8.

### Integration
* `checkLedgers` end-to-end over scratch corpus, `ledgerGate` denial bounded by `MAX_LEDGER_FINDINGS` with `+N more`. `ledger-join-gates.test.ts` #9.
* Entry hook `runPipeline` escalates red tree before first stage, gate-logs under `ledger-join`, no window opened; brief source refused; green tree proceeds. `ledger-join-gates.test.ts` #11.
* Finalize hook `finalizeDone` escalates on unhandled unit, does not flip status; flips to accepted on green tree. `ledger-join-gates.test.ts` #12.
* Standing CI leg: `checkLedgers` over this repository returns `[]`; 013 has 24 units, 021 has 10 units; template never parsed as ledger. `ledger-corpus.test.ts` #10.

### Design conformance
* `parseLedger`, `extractHandles`, `checkLedgers`, `ledgerGate` exported with the taxonomy `ledger-empty`, `unit-malformed`, `unit-duplicate`, `unit-kind-mismatch`, `unit-misplaced`, `pointer-missing`, `owner-missing`, `owner-unknown`, `handle-dangling`, `unit-unhandled`, `note-missing`, `depends-order`.
* `MAX_LEDGER_FINDINGS` exported and used to bound denial reason.
* `frontmatterOf` shared with `gates.ts`, BOM dropped, CRLF handled.
* Corpus walk covers `docs/adrs/**` and `schemas/**`, excludes `docs/designs/**`, template excluded from ledger parsing.
* Entry hook after `sourceAdmissible`, finalize hook before `flipAdrStatus`, gate name `ledger-join`.

## Gate evidence
`npm run check` exit 0 — green
`cargo fmt --all -- --check` exit 0 — green
`cargo clippy --workspace --all-targets -- -D warnings` exit 0 — green
`cargo test --workspace --locked` exit 0 — green
`test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;`

No testing findings.
