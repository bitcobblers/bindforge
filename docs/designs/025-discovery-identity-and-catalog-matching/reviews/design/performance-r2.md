# performance review r2

verdict: approve

Scope: this is the single editorial confirmation pass of round 2; the reviewed
surface is `reviews/design/editorial-r2.diff` (11 hunks, language-only), read
against `02-proposal.md`, `docs/adrs/025-discovery-identity-and-catalog-matching.md`,
and the proposal's Revision log. No prior performance findings were carried
into this round, so the pass checks one thing: whether the editorial revisions
broke a performance property or made a cited section false. They did not.

Diff-by-diff check against the performance checklist:

1. Input-path latency / per-event cost — not applicable to this diff; no hunk
   adds allocation, table rebuild, or synchronous IO. The reflowed
   "refusal ... before enumeration" hunk (Match store — Load) preserves the
   gate-before-work ordering that `025-L1` and Scan-flow step 1/2 pin.
2. Structure costs (O(n²)) — no hunk touches iteration or join structure. The
   step-6 retag from a nested-backtick tuple to (`slug`, `game_id`, friendly
   name of the first sharing row...) keeps the record one-per-distinct-slug,
   matching the store shape `{slug, game_id, friendly_name}`; no new per-row
   work is implied.
3. Memory growth / bounds — the Test strategy 10 change "over 65 536 bytes" →
   "over 65 536 octets" aligns the test text with the client spec's
   "`65_536` body octets" and "header block over the same `65_536`-octet
   budget"; same numeric cap, strictly a terminology fix, and it removes a
   byte/octet drift the design body had not used. The 256-candidate cap and
   the store read budget are untouched.
4. Startup cost — no hunk changes file reads or schema parsing at boot; the
   store-load refusal still precedes enumeration.
5. Caching / invalidation — the `probe()` hunk ("every returned
   `SocketAddr`'s `IpAddr::is_loopback()`") is a possessive-clarity fix; the
   address-based loopback property and the "slug the store resolves is never
   queried" rule are unchanged.
6. Research baseline gap — no hunk touches any expected-number claim, so no
   gap can have been introduced. No `01-research.md` baseline citation lives
   in any changed passage.

Revision log: all entries are r1 or Initial decisions; none of the edited
passages contradicts a recorded author decision, and no rejected finding is
re-opened by this diff.

## Findings

None.

## FYI (no severity)

- The shell tool was permission-denied for this session, so the read-only
  cargo/npm legs could not be run to re-verify code claims; the editorial diff
  introduces no new code claim, so no required verification was skipped.
