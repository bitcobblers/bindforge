# Security Review — code/security-r3c0

## Scope
Diff range 1bda5c34f1561afd79fc395d63855b3d1194d08d…HEAD excluding docs/designs.
Paths touched:
- tools/orchestration/tests/pipeline/ledger-corpus.test.ts
- tools/orchestration/tests/pipeline/ledger-join-gates.test.ts
- tools/orchestration/tests/pipeline/ledger.test.ts

Design reference: docs/designs/021-decision-unit-ledger/02-proposal.md
ADR reference: docs/adrs/021-decision-unit-ledger.md

## Security checklist applied
- No secrets, credentials or tokens introduced.
- No new network egress or external service calls.
- Command execution limited to `git init/add/commit` via `execFileSync` with static argument arrays; cwd is a freshly created tmp dir from `mkTmpDir`. No user-supplied command strings.
- File system writes confined to tmp dirs created by `mkTmpDir` and to the test repo under `docs/adrs` with static names. Paths built with `node:path.join`; no path traversal inputs.
- Dynamic import of `../../src/pipeline/ledger.js` is intentional per design and guarded by type checks that throw if exports are missing.
- JSON parsing limited to test-generated `.pipeline.json` created in the same tmp repo; no untrusted deserialization.
- No markdown parser or external parser added; CRLF handling uses regex splits per A13.
- No changes to production gate logic in this diff; only test scaffolding for `checkLedgers`, `ledgerGate`, `parseLedger`, `extractHandles`.

## Findings
None. The added test files follow the design's Test strategy #1–#12, use only static fixtures, and do not introduce security-sensitive operations. Gate evidence captured by runner shows all checks green:
- npm run check exit 0
- cargo fmt --all -- --check exit 0
- cargo clippy --workspace --all-targets -- -D warnings exit 0
- cargo test --workspace --locked exit 0

## Verdict
Approve.
