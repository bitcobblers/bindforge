# correctness review r2

verdict: approve

Scope: ratification of `reviews/design/editorial-r2.diff` (single editorial
confirmation pass). Previous correctness findings carried into this pass:
none. I checked only the diff hunks plus the surrounding rewritten sentences
in `02-proposal.md`, against `docs/adrs/003-parallel-inference-lane.md` and
the Revision log.

## Findings

None. Every hunk is language-only; I checked each rewritten sentence for a
change of technical meaning:

- Context — "dominant wall-clock cost", "every other state-machine
  semantics stay as written": still matches ADR-003 ("Every other
  state-machine semantic ... stays as written"). No change.
- Options — "The cap stays in runner code": still matches ADR-003
  ("the fan-out stays runner code, never agent prompts"). Grammar only.
- Chosen design / finalize — "the message has today's fallback shape with
  one value": identical claim to the previous wording; still consistent
  with A8 (no `commitMessage` in review replies).
- Pass-level fan-out / escalation bullet — "releases its permit and
  returns without prompting, so no seat that today's `StopDriving`
  (runner.ts:1158–1166) would skip ever gets run": same semantics as
  "releases it and returns unprompted", and consistent with the pseudo-code
  line "release + skip if escalationPending". The runner.ts line reference
  is unchanged by the diff and still bounds the `StopDriving` throw
  (runner.ts:1166). "only complete in-policy sibling reports" is an
  adjective-order swap only.
- Configuration — "a range check (integer ≥ 1)": unchanged contract, still
  matches ADR-003 §Configuration (strict fail-closed, bad env is a config
  error even when the flag overrides it).
- Test strategy #9 — "the pass produces one commit": tense fix; assertion
  unchanged and still matches ADR-003 (exactly one commit per review pass).

Diff-application check: all `+` lines match the current proposal verbatim;
no `-`-side wording survives in the document. The revisions broke nothing
previously approved and made no cited section false. The Revision log shows
no closed item reopened by this diff, and no Deferred item touches ADR-003.

## FYI (no severity)

- Pass-level fan-out bullet — the rewrap leaves "then" dangling at the end
  of its source line ("...enforceOwnership` sweep, then" / next line
  "`finalizeEscalate` once ..."). Renders fine in Markdown; purely a source
  raggedness for the editorial seat.
