# complexity review r1

verdict: approve

Scope: whole document (`02-proposal.md`). Checklist: `references/complexity.md`.

## Findings

- [minor] Chosen design → `src/opencode-runner.ts` (`ReplyTimeoutError` bullet) — the
  `readonly timeoutMs` payload on `ReplyTimeoutError` has no consumer: `main()`'s catch
  only prints the error (cli.ts:120–123), the compatibility story for API consumers needs
  the *class* (which is defensible), and the effective duration is already embedded in the
  message. The design itself used "no consumer in this repository needs it" to reject exit
  code 124; the same argument applies to the field. Fix: drop `readonly timeoutMs` and keep
  the class with the extended message, or state one named consumer that would read the
  property rather than the message.
- [minor] Options considered → "Value semantics" / Chosen design → `resolveReplyTimeoutSeconds`
  / A16 — the `86_400` s upper bound traces to no stated Goal and the design admits no
  evidence for it (A16 "unverified"; "design judgement"). It is also the only cap that
  *restricts* configurability, and the design concedes huge values are technically safe
  under the wall-clock deadline (A9). It carries collateral surface: the cap constant,
  half of Test #7 (`86_400`/`86_401` boundary tests), the README's "1–86 400" wording, and
  the A16 register row. Fix: validate `Number.isInteger(value) && value >= 1` with no upper
  bound and drop A16/Test #7's upper boundary; or, if the typo guard is kept, say plainly
  it is a product decision, not an evidence-backed bound, and keep the README wording
  aligned.

## Checklist walk-through (basis for the verdict)

1. Reuse: no new package or module; changes land in the two existing files. Checked
   `tools/` for a reusable resolver/validator: none exists (only in-repo numeric-validation
   precedent is `z.number().int().min(0).default(0)` in `pipeline/types.ts:36`, an internal
   config default, not a CLI resolver). The wall-clock deadline pattern
   (`pipeline/runner.ts:326`) and the `DEFAULT_BASE_URL` export pattern are correctly reused,
   and `DEFAULT_REPLY_TIMEOUT_MS` avoids a second hardcoded default. Pass.
2. Goal trace: all moving parts trace to Goals 1–5 except the two minors above. Pass with minors.
3. Smaller design: no 30–50% smaller variant exists. The irreducible core — one option
   threaded through `cli.ts` + `opencode-runner.ts`, one exported resolver, README + tests —
   is already present; the trimmable parts (error payload, upper bound) are ~10%. Pass.
4. New dependencies: none; yargs and zod are already in `tools/orchestration`, and the
   resolver hand-rolls three-line validation instead of adding anything. Pass.
5. Deferrability: the change is a single thin slice (one commit, default unchanged);
   nothing in the second half could be deferred to thin the first merge. Pass.
6. Knobs/abstractions: `resolveReplyTimeoutSeconds` is exported for unit tests, not as a
   one-implementation abstraction layer; the deliberate absence of a yargs `default:` avoids
   A8's coercion pitfall rather than adding indirection; the poll interval stays hardcoded.
   The only consumer-less surface is the error payload (minor above). Pass with minor.
7. Value check: proportionate. Roughly 40 lines of production code (one exported constant,
   one optional field, one parameter, one error class, one resolver, one wiring change) for
   a bounded smoke-test CLI flag + env default, with the complexity consciously spent on
   backward-compatible reporting (message prefix, exit-code contract untouched). The two
   minors are trims, not structural over-engineering.

## FYI (no severity)

- `cli.ts` already imports zod and `pipeline/types.ts:36` has a `z.number().int().min(0)`
  precedent; the resolver could express the range declaratively with zod — hand-rolled
  validation is likely clearer here because errors must name the value's source
  (`--timeout` vs env), so this is a preference, not a finding.
- Correctness specialty: research flagged that a timeout shorter than one poll interval
  still yields one immediate `message.list` check; the proposal never states that outcome
  explicitly (it becomes moot for integer seconds ≥ 1 only if the reader notices). Worth a
  sentence.
