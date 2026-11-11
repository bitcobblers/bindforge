# security review r3

verdict: approve

Scope: `reviews/design/editorial-r3.diff` only (ratification pass). Inputs: the diff,
`02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`, the Revision log.
No previous security findings carried into the editorial phase, so the test is
whether any editorial revision broke approved text or made cited sections false.

## Checklist walk (against the diff)

1. **Trust boundaries** — untouched by the diff. Reply `moves` remains
   model-supplied data validated before use; the Non-goal hunk only adds the word
   "at" ("Any agent git write access, at any scope"), if anything tightening the
   prose prohibition inherited from ADR-005.
2. **Parsing untrusted data** — the size caps survive verbatim in the Reply field
   hunk: `.min(1).max(200)` per endpoint, `.max(64)` per list. The reworded
   "a list over either cap fails `safeParse`" preserves the fail-closed `safeParse`
   outcome pinned by Test strategy #1. No limit was weakened or removed.
3. **Path handling** — rules 1–7 pass through the diff unchanged except rule 5
   ("`to` does not exist anywhere" for "`to` exists nowhere" — identical meaning)
   and rule 6 ("What remains" for "What is left"). Canonicalization/realpath
   containment (rule 7), the reserved-name and case-folded `.git` segment rule
   (rule 2), and the reject-`..`/absolute rule (rule 1) are untouched.
4. **Secrets/PII** — nothing in the diff touches logging or data egress.
5. **Dependencies** — no dependency text in the diff; the only named git call
   (`git cat-file -t`) is outside the changed hunks and unchanged.
6. **Least privilege** — the Apply hunk's re-wraps keep `fs.renameSync` under the
   `.pipeline.lock` single-writer invariant and keep the runner as sole executor;
   no privilege widening slipped in with the prose.
7. **Fail closed** — the retry-wiring hunk replaces em-dashes with commas around
   "a returned issue or a throw from a rule" without changing which branch a
   validator throw takes (still `invalid-reply: moves`, not a crash); the Apply
   hunk keeps the guard re-check, reverse rollback, `moves-apply-failed`, and the
   `moves-rollback-failed` operator escalation exactly as approved in r2.

Deferred items re-checked only against the ADR: the read-only-git deferral matches
ADR-005's explicit "Deferred — read-only git in stage shells" clause, and the
checkpoint-trust carve-out matches ADR-005's Consequences ("ADR-006's
checkpoint-trust scope"). No contradiction; nothing re-opened.

## Findings

None. Every hunk is language-only; no security behavior changed.

## FYI (no severity)

- Reply field — "a list over either cap fails `safeParse`" is loose: the 200-char
  cap bounds an endpoint string, not the list. Test strategy #1 states the precise
  behavior ("an endpoint over the length cap, and a list over the pair cap fail
  `safeParse`"), so nothing security-relevant is ambiguous; flagging only as a
  language-precision note for the editorial record.
