# compatibility review r2

verdict: approve

Scope: whole document (`02-proposal.md`, r4 revision state). This round re-verifies closure of the r1 findings against the tree and re-walks the checklist.

## Findings
- [nit] Chosen design — `package.json` bullet (02-proposal.md:90) with the Node toolchain matrix row (02-proposal.md:115) — the new `"engines": { "node": ">=22.12.0" }` encodes a *toolchain* floor (the Vitest 5 requirement, A21) into the field runtime consumers of the published package read at install time. The shipped `dist/` output uses only `Date.now`, `Number.isInteger`, `setTimeout`, and `class extends Error`, all far below 22.12.0, so a consumer pinned to e.g. Node 20 now sees an `EBADENGINE` warning — or a hard install failure if it sets `engine-strict` — for code that actually runs there. Non-blocking: the design itself labels the floor as toolchain and explains why no lower runtime floor can be anchored (nothing in the tree establishes one), and the r1 nit explicitly recommended this exact fix. Fix: one sentence in the `package.json` bullet (or README) noting the declared floor reflects the test toolchain and the published runtime output tolerates lower Node, or declare a true runtime floor if one is ever established.

## Checklist walk-through
- 1 Matrix statuses: pass — all six rows carry an explicit status (02-proposal.md:110–117); no `unknown`, no empty cells. The macOS/Windows rows' "Yes — unverified" is the qualified status r1's major offered as the fix, not an `unknown`.
- 2 OS input APIs: pass — no input-API surface in scope (HTTP-polling CLI); the OS-neutral abstraction is justified (A12) and re-verified this round: `opencode-runner.ts` imports only `node:buffer`, `@opencode/client`, and `zod` — no platform modules.
- 3 Node version floor: pass — closed r1's nit. The floor is now named (Node ≥ 22.12.0, A21) and anchored: `vitest ^5.0.5` at package.json:40 re-verified; the Vitest 5.0 release post was re-fetched live this round and still reads verbatim "Vitest 5 requires Vite >= 6.4.0 and Node.js >= 22.12.0". package.json re-confirmed to have no `engines` key (47 lines) as the design describes, and the design's one-line addition addresses it. Every API the design uses predates the floor — no API above the floor. See the nit above on toolchain-vs-runtime flooring.
- 4 Native modules: pass — the change set adds no dependency at all (two source files, one package.json line, README, tests); the existing dep tree (`@opencode/client`, `yargs`, `zod`) is unchanged and pure JS; packaging story unchanged (Rollout #4: `npm run build` → `dist/`, `files: ["dist"]` at package.json:8–10). No silent native dependency.
- 5 Keyboard reality (layouts/dead keys/IME/non-ASCII names): not applicable — the change touches no keyboard or binding-name surface.
- 6 File paths: not applicable — no path construction or per-OS config-dir handling is introduced.
- 7 Platform test per claimed-supported platform: pass — closed r1's major. Test strategy #12 names the gate that actually exists, `npm run check` (verified at package.json:23 = lint + format:check + test), executed on Linux locally, and states plainly why it is local: A20's claim of zero CI re-verified by repo-wide glob for `**/*.{yml,yaml}` — zero files. macOS/Windows no longer make bare "Yes" support claims; they are downgraded to "Yes — unverified (OS-neutral path, A12)" with where/when they get checked (`macos-latest`/`windows-latest` CI leg or a manual run) — exactly the alternative fix r1 offered, the other remedy (adding CI) being a stated non-goal of the change. Adding CI is correctly kept outside `tools/orchestration` scope.

## FYI (no severity)
- The r1 FYI about the "existing tests stay green" chain resting on Vitest's `toThrow` substring semantics (A5) remains a correctness/test-strategy concern; this round it is additionally pinned by Test #1 (the existing test passing with no edits), so nothing more for this checklist.
- A8's unverified assumption that yargs passes junk numeric flag input through as a non-finite value drives the resolver's validation order; that is a correctness/test-strategy matter, not a platform-compatibility one.
