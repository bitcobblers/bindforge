# compatibility review r1

verdict: request-changes

Scope: whole document (panel pass c0, round 1). Inputs: 02-proposal.md,
docs/adrs/003-parallel-inference-lane.md, the Revision log (initial proposal,
no prior decisions to honor). No previous compatibility findings.

Checklist evaluation:
1. Compatibility matrix — evaluated: two rows carry `unknown` → major finding below. No empty cells.
2. OS input APIs (X11/Wayland/Windows Raw Input/macOS HID) — N/A: the design touches no OS input API; its only OS-facing surfaces are existing `spawnSync` git and fs rename, already covered by the matrix finding.
3. Node version floor — pass: A10 records the `node >= 22.12.0` floor; the primitives the design adds (hand-rolled counter + FIFO waiter queue, `allSettled`, an enqueue-ordered promise chain) are all far below that floor. No API above the floor is named.
4. Native modules — pass: explicitly dependency-free ("~20 lines, no new dependency"); git remains an external binary via pre-existing `spawnSync`. No silent native dependency.
5. Keyboard reality (layouts, dead keys, IME, non-ASCII names) — N/A: no input surface; report/transcript keys are fixed ASCII specialty slugs.
6. File paths — pass at design level: the design introduces no path construction; it reuses existing `reportPathFor` and the existing tmp+rename in `state-store.ts`. It specifies no string concatenation.
7. Platform test for claimed platforms — pass as written: Linux is the only row claimed `yes` (test #10, Linux CI). macOS/Windows are not claimed, so no platform test is owed until the matrix finding resolves them; if they are promoted to `yes`, a platform test becomes owed.

Deferred section — checked against ADR-003: every deferral (ADR-004 checkpoint items, parallel mutating stages / multi-run worktrees, per-stage model/endpoint routing) matches the ADR's own Deferred/Alternatives carve-outs. No contradiction.

## Findings
- [major][new] Compatibility matrix — "| macOS | unknown | Platform-agnostic Node code; nothing exercised here proves it |" and the Windows row "| Windows | unknown | Same; git stays `spawnSync` (A6) |" — checklist rule 1: every platform row must carry a definite status, and `unknown` is a deferral, not a status. Nothing in Rollout or Test strategy ever resolves these rows, so the design ships with two platform-support boundaries left undecided at implementation time. Fix: commit to a status. Given Linux-only CI (Test strategy #10) and no new OS-specific surface, the smallest change is marking both rows "no — untested; no OS-specific code added"; alternatively promote them to `yes` only by adding the matching platform test to Test strategy (checklist rule 7). Either way `unknown` must go.
- [minor][new] Configuration and CLI — "add `maxParallelReviews?: number` to `PipelineConfig` (optional so existing configs compile, mirroring `requestTimeoutMs`)" — ADR-003 §Configuration states "`PipelineConfig.maxParallelReviews: number`, default **1**" (required, not optional). The relaxation is probably right for compiling existing config literals, but the proposal does not record the deviation from the source ADR, leaving an unexplained mismatch for whoever diffs implementation against the ADR and an ambiguity about whether the ADR's default-1 guarantee is enforced by the type or only by `DEFAULT_CONFIG`. Fix: add one clause — "the ADR's field type is deliberately relaxed to optional so existing config literals compile; `DEFAULT_CONFIG.maxParallelReviews = 1` is the sole default authority" — or keep the field required as the ADR writes it.

## FYI (no severity)
- The escalation path skips `enforceOwnership` and lets `finalizeEscalate`'s `commitStage` land siblings' partial reports in a widened escalation commit ("Accept this widened commit"). ADR-003 is silent about finalize on a failed pass, so this is not a compatibility contradiction — but the changed failure-path commit shape is worth a correctness seat's eye.
- The "Run with N > 1 | yes, conditional on the server exposing ≥ N slots" row rests on A1/A2 (slot count unverifiable, oversubscription behavior unspecified in-repo); the condition is honest but unverifiable — relevant to the performance/rollout seats.
