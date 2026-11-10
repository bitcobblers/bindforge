# Code review — security seat, round 1, pass c0

Diff under review: `git diff 7844c17171465dd779f8db6671801eb058bd2eb3...HEAD -- ':(exclude)docs/designs'`
(9 files: `tools/orchestration/prompts/{design-review,test-authoring,test-update}.md`,
`src/pipeline/{independence,runner}.ts`, `tests/pipeline/{child-deeds-retry,child-session-deeds,independence,prompts}.test.ts`).
Judged against `02-proposal.md` and the source `docs/prompts/audit-by-deeds.md`, security checklist only.

## Checklist walk

1. **External input validated at the boundary** — the new server-supplied records
   (`session.children`/`session.list` responses) go through `sessionEntries`
   (runner.ts:525–552) and `pageCursorNext` (runner.ts:555–561), which shape-check
   `id`/`parentID`/`cursor.next` before use. No new binding-file surface. **Pass.**
2. **Path handling** — no traversal, no absolute paths from untrusted data, no
   interpolated shell. The matcher only string-matches recorded values. **Pass.**
3. **Injection** — the segment regexes (independence.ts:138–142) are fixed
   literals, linear, never built from input; `PATH_ACCESS_KEYS` values are matched,
   never executed. The attempt-2 `failureNote` enters the prompt only through
   `brief(printable(...))` (runner.ts:1539, 1748), control bytes stripped, 140-char
   cap; prompt text is not audit input (A9), so quoting forbidden paths back is
   safe. `gateLog` sanitizes the new walk-failure reason (runner.ts:1425). **Pass,
   one nit (finding 3).**
4. **Secrets/PII** — none in the diff; fixtures use placeholder identity
   (`f@f`, `ses_*`). **Pass.**
5. **New dependencies** — none: no `package.json`/lockfile change in the range.
   **Pass.**
6. **Filesystem writes** — none added. **Pass.**
7. **Clean rejection of malformed/hostile input** — the walk is visited-set
   guarded against a cyclic `parentID` graph (pinned at
   tests/pipeline/child-session-deeds.test.ts:118–132), and the path scan is
   node-capped (independence.ts:230–257). But the child-walk catch discards
   violations already found — finding 1. **Fail.**

## Findings

### 1. major — `runner.ts:1097` — the child-walk catch erases violations already detected

The walk is entered only when the parent audit is clean, and `for await` pushes
child violations into `violations` as descendants are audited. If a *later*
enumeration or message fetch in the same walk then fails (after `bounded()`'s
single retry — a timeout counts), the catch runs `violations.length = 0`
(runner.ts:1097) and the attempt proceeds clean, gate-logged as merely
"walk incomplete". A deed that WAS seen is thrown away — strictly worse than the
design's stated failure mode, which is a detection gap for deeds *unseen*
(Risks row "Child-walk fetch fails… detection gap"), not erasure of evidence in
hand. The bypass is agent-shaped: this audit's own threat model (module header,
independence.ts:6–16) is a model routing around the fence; a stage that delegates
one transcript-reading child plus a second child engineered to stall its message
fetch past `requestTimeoutMs` (huge message list — fetches are paged and
time-boxed, so slow-fetch is reachable) converts a detected violation into a
pass. It also fires benignly: any transient server hiccup mid-walk silently
un-fails a real violation.

Fix: keep the violations collected before the failure — delete
`violations.length = 0;` (the parent list is already in the `violations.length
=== 0` branch, so the reset can only drop child findings), keep the
`console.warn`/`gateLog` partial-audit record, and let the surviving child
violations reach the `violations.length > 0` branch. Pin: a fake client whose
first child reads a `transcripts/` file and whose second fetch throws must still
fail the attempt with the violation and the walk-incomplete gate line.

### 2. minor — `runner.ts:629–634` — the compatibility matrix's third enumeration route is not implemented

The matrix row "OpenCode server/SDK without a children method" promises the
chain `session.children` → `session.list` + `parentID` filter → direct
`GET /session/:id/children` against the configured `baseUrl` with the client's
auth headers, and states "the walk always has a route (A4 decides which, not
whether)". The implementation probes only the first two and throws when neither
client method exists — with `subagent: []` in `ACCESS_FIELDS`, that throw means
every delegation's evidence goes unseen on such a client (logged, but a clean
verdict). Coverage posture for this seat, and a deviation from the approved
design text. Fix: implement the direct-HTTP third route, or amend the design's
matrix/rollout to state that two client routes are the shipped envelope (a
decision for the editor, not this seat) — either closes the gap between the
reviewed claim and the shipped audit.

### 3. nit — `runner.ts:1098–1102` — the walk-failure `console.warn` interpolates the server-supplied error string without `printable()`

The module's `printable` posture (independence.ts:270–281) exists so no
control/escape byte from server- or model-chosen text reaches the operator
terminal; the same string is sanitized again in `gateLog` (runner.ts:1425), so
this is consistency, not a new leak (the pre-existing `bounded()` warn at
runner.ts:727 has the same raw interpolation). Fix: wrap the error text in
`brief(printable(...))` in the warn.

### 4. nit — `runner.ts:639–668` — the `session.list` crawl has no anti-loop bound on `cursor.next`

A server that keeps returning the same non-empty `cursor.next` pages forever
(each request individually time-boxed, but the iteration is unbounded and
`byParent` re-accumulates duplicates). This exactly mirrors the pre-existing
`listSessionMessages` loop (runner.ts:485–501) — flagging because new code
should not clone the flaw. Fix: stop when a cursor repeats, or cap total pages.

## What holds (security-relevant confirmations)

- The `subagent: []` text exemption is safe *because* the child walk and its
  pins exist: pin (a)/(b) exemptions and pin (c) detections are all pinned
  (independence.test.ts new block; child-session-deeds.test.ts; the end-to-end
  `(via child session …)` failure in child-deeds-retry.test.ts), matching the
  source's "the new detection must be pinned, not just the new exemption."
- The zero-descendant tripwire fires as designed and is pinned
  (runner.ts:1086–1092; child-session-deeds.test.ts:286–328).
- `since` applies unchanged to child lists — the walk fetches everything, the
  matcher judges only the attempt's window (A7 pinned at
  child-session-deeds.test.ts:138–154); no double-punishment, no widened reach.
- `failureNote` is set only on the two non-resumable attempt-2 sites and
  suppressed whenever `resume` is set; budget, nudge loop, and escalation flow
  untouched — no new channel of model-controlled text into prompts beyond the
  existing `repairPrompt` precedent, and it is sanitized and length-bounded.
- BFS termination: visited-set guard on both routes; self-parent entries skipped
  (runner.ts:656).
- The prompt lines and their pins match the design text verbatim; no security
  content involved.

## Verdict

request-changes — one open major (finding 1); fix is a deleted line plus one pin.
