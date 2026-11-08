# complexity review r2

verdict: approve

Scope: whole document (`02-proposal.md`, r4 state). Checklist: `references/complexity.md`.
Prior round: `complexity-r1.md` — both minors verified closed in the body: the
`readonly timeoutMs` payload is gone (`ReplyTimeoutError` explicitly data-free), and the
upper bound is now explicitly labelled a product decision (A16, Options considered,
README `1–3 600` realigned).

## Findings

- [minor] Chosen design → `package.json` bullet (also Goals/Non-goals, Compatibility
  matrix Node-toolchain row, Rollout #1/#4) — the `engines: node >= 22.12.0` line is the
  only moving part that traces to **no stated Goal**. Goals 1–5 cover the flag/env, the
  fail-closed API boundary, the report, green tests, and the README; the design itself
  concedes the line "adds no enforcement burden" and that every API it uses predates the
  floor — its sole stated purpose is to give the Compatibility matrix a checkable anchor,
  which is a review artifact, not a product goal. Checklist item 2 (goal trace) / item 5
  (deferrable half). Severity minor rather than YAGNI-major: it is one advisory line
  documenting the already-verified `vitest ^5.0.5` fact (A21), not new functionality.
  Fix: drop the `package.json` edit from this change set and let the compatibility row
  anchor on the verified `vitest ^5.0.5` declaration alone (the A21 basis already does
  this) — or add one explicit goal line ("declare the toolchain floor in `package.json`")
  so the change traces, and keep Rollout #1/#4's references consistent either way.
- [nit] Options considered → "Wiring in `cli.ts`" (A19) / Test strategy #10 — the
  fallback `clientFactory` seam on `main()` is a dependency-injection abstraction with a
  single (test-only, speculative) consumer, and the design never says the two routes are
  mutually exclusive. As written, an implementer whose `vi.mock` spike stumbles can end up
  with both the mock-based test and a production seam — exactly the one-implementation
  abstraction the checklist warns about. Fix: one sentence stating the fallback *replaces*
  route (a) rather than being added alongside it, and that if the seam is taken, A19 is
  updated to record the spike outcome as the sole basis.
- [nit] Revision log, line 160 (r3 entry) — the "Deliberately not changed this round"
  paragraph is duplicated verbatim twice in the same line; the same decision content then
  reappears again at line 165 (r4 re-summarises the r3 list). This bloats an already
  ~25-line revision log and slows every downstream reviewer for zero information. Fix:
  delete the second copy on line 160; optionally let r4's not-changed list cross-reference
  r3's instead of restating it.

## Checklist walk-through (basis for the verdict)

1. **Reuse:** no new package or module; edits stay in the two existing source files, the
   README, the test file, and the one `package.json` line. Re-checked `tools/` for a
   reusable resolver/validator/error base: still none — the only numeric-validation
   precedent is `z.number().int().min(0)` (`pipeline/types.ts:36`, internal config, not a
   CLI resolver; see FYI), and the only error-class precedent is the module-local
   `StopDriving extends Error {}` (`pipeline/runner.ts:549`), so `ReplyTimeoutError` has
   nothing to reuse. The `DEFAULT_BASE_URL` export pattern, the wall-clock deadline
   pattern (A10), and the fake-timer test pattern are all correctly reused rather than
   reinvented, and the `DEFAULT_REPLY_TIMEOUT_MS`/`MAX_REPLY_TIMEOUT_MS` exports keep both
   bounds single-sourced. Pass.
2. **Goal trace:** every moving part traces to Goals 1–5 — the resolver, `EnvValues`
   (serves the one validated `CliConfig.timeoutSeconds`), the runner-side `TypeError`
   (Goal 2's "not only in the CLI"), `ReplyTimeoutError` (Goal 3), README (Goal 5) —
   except the `engines` line (minor above). Pass with minor.
3. **Smaller design:** no 30–50% smaller variant meets the Goals as r2 restates them. The
   growth since r1 (`EnvValues`, boundary check, format gate, two integration tests) each
   closes a correctness/security major from r1 — evidence-driven, not speculative. The
   still-trimmable parts (`engines` line, fallback seam) are ~10%. Pass.
4. **New dependencies:** none. yargs, vitest, and zod are already `package.json`
   dependencies; `vi.mock` is vitest built-in; the format gate and `JSON.stringify`
   escaping are stdlib. Pass.
5. **Deferrability:** the change is one thin, cohesive slice and splitting the two source
   files across branches would only add churn. The genuinely separable piece is the
   `engines` line (finding 1); Test #10's spike-gated integration test is already
   correctly sequenced as implementation-time work. Pass.
6. **Knobs/abstractions:** no knob without a consumer — the poll interval deliberately
   stays hard-coded (explicit non-goal), `MAX_REPLY_TIMEOUT_MS` has two real consumers
   (resolver + runner boundary), and `resolveReplyTimeoutSeconds` is exported for unit
   tests, not as an indirection layer. The only one-implementation abstraction is the
   conditional `clientFactory` fallback (nit above). Pass with nit.
7. **Value check:** proportionate. Roughly the same ~40-line production core r1
   estimated, plus validation hardening that was demanded by concrete r1 majors on other
   specialties; the 12 tests mirror the stated contract surface 1:1 rather than
   gold-plating. The feature is a flag + env default on a bounded smoke-test CLI, and the
   design spends its complexity on backward-compatible reporting and fail-closed
   validation — appropriate. The two trims above are housekeeping, not structural
   over-engineering.

## FYI (no severity)

- Reuse preference (unchanged from r1): `cli.ts` already imports zod and the repo has the
  `z.number().int().min(0)` precedent; the hand-rolled resolver remains the better choice
  because errors must name their source (`--timeout` vs env) — preference note, not a
  finding.
- Editorial/other specialties: the revision log (lines 141–165) now carries r1–r4
  verifications with heavy restatement across r3/r4 entries; whether to compress it is an
  editing question outside the complexity checklist, and anchor accuracy inside those log
  entries is correctness's business, not mine.
