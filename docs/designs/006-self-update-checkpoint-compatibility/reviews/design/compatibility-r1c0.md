# compatibility review r1

verdict: request-changes

Scope: whole document (02-proposal.md), inputs limited to the proposal, the ADR,
and the Revision log. Revision log: initial authoring, no rejected findings —
all findings below are `new`.

## Findings

- [major][new] Compatibility matrix — "| Windows | unknown | No CI workflow exists to exercise it" — checklist item 1 requires an `unknown` cell to appear as a finding: the matrix is the document's compatibility contract, and an `unknown` cell ships an undecided answer instead of a decision. The design itself makes Windows-differing primitives load-bearing (`rename`, `'wx'` in §4; the repair-command print in §3), so operators and packagers get no Windows posture from the document that decides this. This does not reopen the Deferred item ("Windows CI verification … no follow-up yet") — deferring the CI *work* is consistent with the ADR; the *status column* must still record a decision. Fix: change the cell to an explicit status — e.g. `no (unverified)` — keeping the existing note text; the change touches only the status cell. A dated Windows smoke job later may flip it to `yes`.

- [major][new] Compatibility matrix / Test strategy (Platform) — "macOS | yes | POSIX semantics hold; not exercised by CI in this repo" plus "the macOS, Windows, and network-FS rows are not exercised by this run" — checklist item 7: the Test strategy must name a platform test for every platform the matrix claims as supported. macOS is claimed `yes` while no named test run covers it; the POSIX-parity claim rests on documentation (A5/A6) rather than an exercised check. Fix: add one line to the Platform paragraph naming a concrete macOS run of the existing suites (a dated manual/dev-machine run recorded in the document also names a platform test and costs no CI), or demote the macOS row to the untested status the network row already uses (`partial`).

- [minor][new] Test strategy (Platform) vs Assumption register A12 — "Linux CI exercises all of the above" vs A12's "nothing in this repo exercises it (no CI workflow found)" — the document contradicts itself about whether a CI runner exists. If no CI exists, "Linux CI" names a non-existent artifact and the Linux platform-test claim (checklist item 7) has no named runner; if CI does exist, A12's basis is stale. Fix: replace "Linux CI exercises all of the above" with the actual runner — e.g. "the existing `tools/orchestration` test suite, run on Linux" — or add a Linux CI workflow as a named deliverable in Rollout (and correct A12).

- [minor][new] §1 Fence — "return `false` when the path's last segment is `STATE_FILE` (`.pipeline.json`), starts with `${STATE_FILE}.`" — the rule never says how the "last segment" is derived or how case is compared (checklist item 6: separators and case handled by a library, not string work). On a case-insensitive filesystem — macOS is claimed `yes` — a write reported as `.PIPELINE.JSON` passes an exact-string fence while the filesystem routes it to the same file as the checkpoint, breaking §1's "neither the worktree copy nor the committed copy can ever again carry model-authored bytes" claim; naive segment splitting also misparses Windows path shapes. Fix: one sentence binding the matcher to path-library semantics — `path.basename` comparison, case-insensitive comparison on case-insensitive platforms — reusing the normalization already in the moves validator (moves.ts:107–131) rather than introducing new string logic.

- [nit][new] §3 finalizeEscalate — "prints the repair commands — `git checkout HEAD -- <designDir>/.pipeline.json`" (and the `rm -r …` variant) — the printed commands are POSIX-shell only. That is consistent while Windows is undecided, but if the Windows cell ever becomes a support claim, the operator-facing message has no Windows form. Fix (conditional): only if Windows becomes supported, name the PowerShell equivalents of the repair commands alongside the POSIX ones.

## FYI (no severity)

- §4 / Test strategy #10 — "two concurrent `acquireLock` calls in a fixture repo admit exactly one winner" — whether a single-process test actually provokes the `wx` race (or just the sequential EEXIST path) is a correctness/test-rigor question, not this seat's.

## Checklist coverage

- 1 Matrix statuses: findings — Windows `unknown` (major); no empty cells; all other rows have statuses.
- 2 OS input APIs: N/A — the design touches no input API; it is runner file/lock semantics only.
- 3 Node version floor: floor named (`>= 22.23.4`, A11); `rename`/`'wx'`/`mkdir`/errno handling sit far below it — no finding.
- 4 Native modules: N/A — pure built-in `fs` use, no native dependency introduced, silent or otherwise.
- 5 Keyboard reality: N/A — no key, layout, or binding-name surface in this design.
- 6 File paths: finding — fence matcher segment/case handling unspecified (minor); config-dir conventions N/A (checkpoint lives inside the repo worktree).
- 7 Platform test per supported row: findings — macOS `yes` with no named platform test (major); Linux runner naming contradicts A12 (minor).
