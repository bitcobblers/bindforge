# compatibility review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` only (editorial confirmation pass, round 2).
Previous compatibility findings: none. Inputs: the diff, `02-proposal.md` (final text),
the in-proposal Revision log, and `docs/adrs/006-self-update-checkpoint-compatibility.md`.

## Findings

None. All five hunks are language-only and semantics-preserving; none breaks a
compatibility claim or makes a cited section false:

- Goals bullet — "tmp pattern" → "tmp shape": matches §1's terminology for the
  `.pipeline.json.<8 hex>.tmp` shape (`state-store.ts:186`); matcher behavior unchanged.
- §2 May-refuse bullet — "that sentence" → "that wording": the refusal message still
  names the supported version plus the ADR's two interim postures
  (ADR-006:55–57); the v99 regex claim stands.
- §3 loadState bullet — reordered to "For a directory at the checkpoint path, … catch
  `EISDIR`/`ENOTDIR`/`ENOTEMPTY` alongside today's `ENOENT` handling": "with" →
  "alongside" is identical in meaning; errno handling unchanged and still consistent
  with Test #5 and Risk row 5.
- §4 lock bullet — added articles ("a live holder", "a dead holder", "and a racing
  loser"): the pid-liveness logic and the `'wx'` retry story are untouched; A6/A12
  (Node `'wx'`/rename semantics, no Windows exercise in-repo) still cover it.
- Test #12 — active→passive rewording: same assertion (checkpoint never written
  outside the runner's own write path); fence-related platform coverage unchanged.

Checklist re-check against the final text (unchanged by this diff): every matrix row
carries a status and no row is `unknown` (Windows is `no (unverified)` by the r1
revision, a declared non-support, not an unknown); Node primitives stay at or below
the `>= 22.23.4` floor (A11); no native modules; path matching is segment-wise and
case-insensitive via the moves-validator precedent, not string concatenation; the
POSIX-only repair commands are consistent with the Windows `no (unverified)` row;
the Test strategy names Linux coverage and records the macOS/Windows/network-FS rows
as untested, matching rows that are `partial`/`no`, not `yes`. Deferred items
(Windows smoke job, lock-ID nonce) are consistent with the ADR's Decision and
interim posture; not re-opened.

## FYI (no severity)
- Nothing for other seats from this diff; all hunks are wording-only.
