# security review r1 (dissent pass c1)

verdict: approve

Scope: `git diff ce95f3b f4f64f9` on `02-proposal.md`, then the full current
document. Checklist: security (trust boundaries, parsing, path handling,
logging, dependencies, least privilege, fail-closed).

## Round-1 findings — disposition

- **[major] running-descendant snapshot gap — CLOSED.** §2 now states "The walk
  is a point-in-time snapshot taken after the reply: a descendant still running
  at that moment contributes only its completed parts (Risks)", and the new
  Risks row names the net-vs-today loss, the one-attempt scope, and the A10
  fence mitigation. The author took the stated-semantics branch of my fix rather
  than the settle re-fetch; that is the branch I offered, and the accepted gap
  is now visible in the run record instead of silent. Residual wording nit
  below (finding 4), not a re-opening.
- **[major] unguarded fallback enumeration — CLOSED.** The fallback is now
  "one `client.session.list()` paged with `cursor.next` like
  `listSessionMessages` (A6) and indexed by `parentID` into a Map the BFS
  walks", so the page-1-only hole is gone, and the tripwire — "an audited
  message list holds a completed `subagent` part yet the walk enumerated zero
  descendants" — closes the empty-but-successful case, with a matching
  Test strategy #5 pin. Log-only posture, no new failure mode: as asked.
- **[minor] partial audit invisible in the run record — CLOSED.** "`console.warn`
  plus one `gateLog` line naming the session it could not enumerate", and the
  fetch-failure Risks row now reads "name the partial audit in the run record".
  Stage still not failed, no schema change: as asked.
- **[minor] false-positive direction untracked / stale Risks cross-reference —
  CLOSED.** The stale "(Risks row 2)" pointer is gone; §1 now states the FP
  outright — "query-text `pattern` on an unknown tool is a known false
  positive — the violation names key and value, so a mis-scan is a one-line
  allowlist fix" — which is the mitigation I named, carried in the section that
  makes the decision rather than a second Risks row. Acceptable.
- **[minor] unbounded `failureNote` into a fresh prompt — CLOSED.** The note is
  now `brief(printable(out.failure ?? 'no reply'))` at both sites and enters as
  "Previous attempt's gate failure (log excerpt): …", labelled and length-bounded
  through the existing `brief()` helper. No structured field demanded. Trust
  boundary now matches the rest of the runner's log posture.
- **[nit] unbounded matcher recursion — CLOSED.** "collect string values under
  an allowlist of access-naming keys, via a node-capped queue that drops the
  deepest nodes", and the Deferred list picks up "values beyond the §1 walk's
  node cap" as a named residual. Dropping rather than throwing is what I asked
  for; the audit call site can no longer be taken down by an enormous tool input.
- **[nit] session-list read scope — OPEN (re-raised, finding 3).**

## Findings

- [minor][new] Chosen design §2 Deeds of children — "A child-walk fetch error
  after `bounded()`'s single retry is caught, logged … and the audit proceeds on
  the parent list alone" — that sentence was accurate for the r0 array-returning
  helper (all-or-nothing) but the revised helper is now a lazy
  `AsyncGenerator`, so a late failure leaves the violations of every descendant
  already fetched sitting in hand; read literally, the sentence instructs the
  implementer to discard them and audit the parent alone, silently shrinking the
  audited set while the gate log says "partial audit". The zero-descendant
  tripwire does not cover it (it only fires on *zero*). Fix: one clause in the
  same bullet — "violations already collected from descendants fetched before
  the error are kept; the catch skips only the un-fetched remainder" — replacing
  the flat "proceeds on the parent list alone".
- [nit][new] Compatibility matrix, row "OpenCode server/SDK without a children
  method" — "else a direct `GET /session/:id/children` against the configured
  `baseUrl` with the client's auth headers" — new text adds a hand-rolled
  network route outside the SDK client with three unspecified details: where the
  auth headers come from (it should be the same config already used to construct
  `OpenCodeClient`, not a second read of a credential), that the id taken from
  the walked `SessionInfo` records is percent-encoded into the URL path
  (`encodeURIComponent`) instead of interpolated, since the BFS trusts those
  records, and that the headers are never logged — the §2 gateLog line names the
  session only. Fix: one clause on the row naming the credential source, id
  encoding, and no-logging, and stating that this route falls into the same §2
  fetch-error catch as the other two (A4 is still marked unverified, so the row
  should not read as if that route's reachability were verified).
- [nit][re-raised] Chosen design §2 Deeds of children — "one
  `client.session.list()` paged with `cursor.next` … and indexed by `parentID`
  into a Map the BFS walks" — the revision widened this from a filtered call per
  node to one full paged enumeration of every session on the server per walk, and
  the design still never scopes the rest of that data. Unrelated sessions'
  metadata (titles, timestamps) is now in the runner's memory at audit time, and
  nothing states it cannot reach the gate log or the attempt-2 note. Fix: append
  to that bullet — "the list result is used only to build the descendant set; it
  is never logged or persisted" — no behavior change.
- [nit][new] Risks, row 2 — "A descendant still running at the snapshot
  contributes **no** completed parts, so its deeds go unseen for that attempt"
  contradicts §2, which says such a descendant "contributes only its completed
  parts". The gap is only the deeds completed after the snapshot. Fix: align the
  row's wording with §2 ("its deeds after the snapshot go unseen for that
  attempt"), since the row, not §2, is what a reader of the gate record leans on
  to judge the accepted gap.

## FYI (no severity)

- §2 now states "`since` bounds what the matcher judges, not what the walk
  fetches" and the cost bullet concedes "including entirely pre-`since` lists on
  the resumable lane". Source §1 asks to enumerate children "created at or after
  the attempt's `since` stamp"; the design's superset fetch + message-level
  filter is equal-or-stronger for detection, so it is not a contradiction, but
  it is the shape that lets a child spawned by attempt 1 on the resumable lane
  have its post-`since` deeds charged to attempt 2. The misattribution Risks row
  and the Deferred identity-guards line both already name that class — cost and
  identity are correctness/performance seats' business.
- The unbounded per-attempt fan-out (one full paged message fetch per descendant,
  N uncapped) is restated, not newly introduced, by the cost bullet — performance
  seat's territory.
