# compatibility review r1

verdict: request-changes

## Findings
- [major] Test strategy item 10 (02-proposal.md:113) vs Compatibility matrix rows Linux/macOS/Windows (02-proposal.md:95–97) — checklist item 7 requires a platform test for every platform the matrix claims supported. The matrix marks macOS and Windows "Yes", but the test strategy names an executed test only on Linux and explicitly declines any macOS/Windows test ("need no dedicated platform tests"). The OS-neutrality argument (A12) is sound as an argument, but the repo contains no CI configuration at all (no `.github/workflows`, no yaml anywhere), so even the "must pass on Linux (CI)" run is unenforced and the macOS/Windows support claims rest on nothing checked. Fix: add a test-strategy item that runs `npm test` on a CI matrix including `macos-latest` and `windows-latest` (matching the Rollout #1 gate), or downgrade those two matrix rows to "Yes — unverified (OS-neutral path, A12)" and state where/when they will be exercised.
- [nit] Compatibility matrix, Node toolchain row (02-proposal.md:98) — checklist item 3 asks for the Node version floor; the design evaluates compatibility against the `ES2022` compile target instead, and no Node floor is declared anywhere (`tools/orchestration/package.json` has no `engines` field), so "Supported: Yes" has no floor to check new APIs against. Every API the design uses (`Date.now`, `setTimeout`, `Number.isInteger`, `class extends Error`) is far below any plausible floor, so nothing is actually broken. Fix: name the floor in the row (e.g. "Node ≥ 20, matching CI") and/or add `engines.node` to `tools/orchestration/package.json`, so future reviews have an anchor.

## Checklist walk-through
- 1 Matrix statuses: pass — all six rows carry an explicit status; no `unknown`, no empty cells (02-proposal.md:93–100).
- 2 OS input APIs: pass — no input-API surface in scope; the OS-neutral abstraction is justified (A12, matrix notes) and verified against `tools/orchestration/src/opencode-runner.ts` (HTTP polling + `setTimeout` only, no platform APIs).
- 3 Node version floor: no API above the floor is used; see the nit above about the unstated floor.
- 4 Native modules: pass — no new dependencies, pure TS/JS, packaging story in Rollout #4 (`dist/` via `npm run build`, `files: ["dist"]` already in package.json).
- 5 Keyboard reality (layouts/dead keys/IME/non-ASCII names): not applicable — the change touches no keyboard or binding-name surface.
- 6 File paths: not applicable — no path construction or config-dir handling is introduced.
- 7 Platform test per supported platform: fail — see the major finding.

## FYI (no severity)
- Rollout step 1 and Test strategy item 10 both lean on a CI gate that does not exist in this repository; that belongs to rollout/complexity, not to this checklist.
- The "existing tests stay green" guarantee for API consumers rests entirely on Vitest's substring semantics for `toThrowError(string)` (A5, 02-proposal.md:31); correctness/test-strategy may want to confirm it directly rather than by citation.
