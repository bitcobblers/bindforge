# security review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial diff), read
against `02-proposal.md` and `docs/adrs/015-transforms-as-preconditioned-policies.md`.
No prior security findings to re-check; the pass asks only whether the
revisions broke approved security posture or made a cited section false.

## Checklist evaluation

1. **Trust boundaries** — the diff does not touch the classification. Policy
   documents stay untrusted (A8: "`error.rs` counts policy documents among the
   untrusted classes") and the Risks row "Policy is the fourth untrusted
   document class" is byte-unchanged. Pass.
2. **Parsing untrusted data** — hunk at Options considered → Row encoding
   ("exactly as `set`/`swap`/manifest rows do today" → "as
   `set`/`swap`/manifest rows already do") removes "exactly" but still asserts
   the same reuse of the shipped wrong-field-count refusal mechanism and its
   fixtures. Caps, closed entry set, and fixture-pinned refusals untouched. Pass.
3. **Path handling** — `load_policy`'s slug-before-filesystem rule (Goals,
   Loading) is outside the diff; nothing interpolated or reworded. Pass.
4. **Secrets/PII and echo posture** — the bounded-echo/escape-and-truncate
   posture (A14, The report) is untouched. Pass.
5. **Dependencies** — no third-party dependency is named or changed by the
   diff; `Cargo.lock` "gains exactly one package" refers to the new
   `crates/transforms` workspace crate, std-only per the Toolchain floor row.
   No advisory surface to check. Pass.
6. **Privileges** — the Compatibility matrix → Product artifacts row ("touches
   no OS input API ... parsed documents in, parsed documents out") is outside
   the diff and stays intact. Pass.
7. **Fail closed** — the three security-adjacent sentences the diff edits all
   preserve their claims: Goals "Precondition outcomes **are** verdict data on
   `PlacementCheck`'s channel, never new `CoreError` variants" (fragment made
   a sentence; the refusal/verdict split and A9 unchanged); "mapped one-to-one
   to a named verdict" (every check still lands a named verdict); and "exactly
   the answer a bool cannot give (A4)" — dropping "-why" keeps the claim true,
   since the carried `PlacementCheck` is itself the reason-carrying answer.
   No refusal or guard wording was weakened. Pass.

Deferred section: unchanged by the diff and consistent with ADR-015's "Not
decided here" list — no deferral contradicts the ADR.

## Findings

(none)

## FYI (no severity)

- The Goals bullet "Precondition outcomes are verdict data..." is now a full
  sentence among noun-phrase bullets; house style is the editorial seat's call.
- The appended "Editor notes:" line is a process artifact for the author to
  resolve in prose; no security content in it.
