# security review r2

verdict: approve

Scope: the editorial round-2 diff
(`docs/designs/014-profiles-as-chains/reviews/design/editorial-r2.diff`), read
against `02-proposal.md` and `docs/adrs/014-profiles-as-chains.md`. This is a
language-only pass, so the security question asked of every hunk is one thing:
did the rewording move, weaken, or make false a trust-boundary, untrusted-input,
path-handling, logging, budget, privilege, or fail-closed statement? Every hunk
is punctuation, line re-wrapping, or verb agreement. No hunk adds, removes, or
renames a guard, cap, refusal, or fixture pin.

Previous security findings: none (the editorial phase is one language pass per
round), so there is nothing to re-check as a closure regression.

## Findings

None. No blocker or major finding open.

## Checklist coverage (all seven items evaluated)

1. Trust boundaries — unchanged. The diff hunk at "Where ancestor documents
   come from" only adds a comma ("ADR-018's, and storage location is explicitly
   undecided (A11)"); the hub stays ADR-018's and every opened source stays
   behind "one version gate, byte budget, and chain guards, source-independent".
2. Parsing untrusted data — unchanged, and one statement got tighter:
   "one per link — totalling at most `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES`
   (32 MiB)". The arithmetic still checks out against A6 (1 MiB) and
   `MAX_CHAIN_DEPTH = 32`, and the diff did not touch `MAX_PATCH_DELTA_ELEMENTS`.
3. Path handling — unchanged. The only touched sentence in that passage is
   "Absent is `ChainLinkMissing`" → "An absent file is `ChainLinkMissing`",
   which names the same refusal for the same condition; the constructed path is
   still `<major dir>/<parse-validated slug>.toml` with the slug grammar
   refusing `..` and `/`, and nothing anywhere interpolates a path into a shell.
4. Secrets and PII — improved by the diff: the wrapped fixture name is repaired
   from "`forged-log-line .toml`" to "`forged-log-line.toml`", which now matches
   the corpus listing in the contract section, so the log-injection pin
   (single-line, control-free, truncating refusal for newline-bearing values,
   A13) still points at a fixture that is actually declared.
5. Dependencies — nothing to check: no hunk and no proposal section names a
   dependency; the design is std-only with `Cargo.lock` byte-unchanged.
6. Privileges — unchanged. "core builds documents, not filesystems" survives
   the re-wrap at the `fork` bullet; core stays read-only.
7. Fail closed — unchanged. The re-wrapped `merge` and Test strategy #5 text
   still refuses the whole result (`InvalidValue` naming the colliding action
   id) and refuses the identity-cancelling composition as the empty-patch case;
   `load_chain`'s refusal of `fork`'s empty-delta shell survives the added
   "that" in "a shell that `load_chain` refuses".

## FYI (no severity)

- **Chain operations / `fork`** — "a shell that `load_chain` refuses as
  `patch-empty-deltas`", and the parallel "refuses as the empty-patch case" in
  Test strategy #5, name a *fixture* where the design elsewhere promises a typed
  refusal ("Every other v2 refusal is `InvalidValue` or `MissingEntry`"). The
  wording predates this diff — the editorial pass only inserted "that" — so it is
  not a regression from this round; flagging it for the contract/correctness seat
  to decide whether a refusal label or the fixture name belongs there.
- The diff's hunk at "`forged-log-line.toml`" is the one edit here with security
  bite, and it is a repair rather than a break: an un-fixed name would have left
  the newline-injection pin unanchored to the corpus.
