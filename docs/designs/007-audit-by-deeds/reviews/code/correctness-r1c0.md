# Code review — correctness, round 1, full-panel pass c0

Scope reviewed: `git diff 7844c17…HEAD -- ':(exclude)docs/designs'` as recorded in
`03-build-notes.md` "Files touched" + the design-007 test batch. Judged against
`02-proposal.md` (and `docs/prompts/audit-by-deeds.md` as its source).
Previous seat findings: none (first pass).

Note on method: this seat has no shell, so `npm run check` and the literal diff
could not be executed. The review reads the final state of every file the build
notes name, greps the whole package for design-007 markers to catch anything
touched outside them, and checks the build notes' *checkable* claims statically.

## Checklist

1. **Components and interfaces named in the proposal exist and match.** Pass.
   `subagent: []` (independence.ts:97–98); `PATH_ACCESS_KEYS` with exactly the
   nine keys the proposal lists (independence.ts:106–116); the node-capped,
   breadth-first `pathKeyValues` replacing `stringValues(input)` for unknown
   tools (independence.ts:230–257); both doc-comment updates (header
   scope-discipline block lines 29–37, the `ACCESS_FIELDS` block 73–83);
   `listChildSessionMessages(client, sessionID, requestTimeoutMs)` with the
   proposal's signature and BFS/visited-set/one-list-at-a-time shape
   (runner.ts:585–687); parent-first audit at the call site with the exact
   `` `${violation} (via child session ${sessionID})` `` suffix
   (runner.ts:1070–1083); the zero-descendant tripwire (1084–1092) and the
   caught/logged walk failure (1093–1110); `StageHandle.failureNote` (853–854)
   with the "Previous attempt's gate failure (log excerpt) … Do not repeat …"
   block gated on `resume === undefined` (923–930); `failureNote` set at both
   fresh-session attempt-2 sites (1539, 1748) — and grep confirms those are the
   only two `attempt: 2` sites; the three prompt lines (design-review.md:9–11,
   test-authoring.md:8–9, test-update.md:5–6) and their rendered-content pins
   (prompts.test.ts:207–212, 359–372).
2. **Assumption register honored.** Pass. A7: child lists are audited with the
   same `auditOptions`, `since` judging not gating the fetch
   (runner.ts:1064–1083), pinned by child-session-deeds.test.ts:138–154.
   A8: no reliance on session-creation timestamps. A9: quoting forbidden paths
   into attempt 2's note is safe and pinned. A10: infra faults degrade to a
   parent-only audit rather than a stage failure. A4's "which route" question is
   answered at call time (deviation 1, declared).
3. **Deviations declared.** Pass on the substance. Runtime route probing, the
   unimplemented third (`GET /session/:id/children`) fallback, the gate name
   `independenceAudit`, `MAX_PATH_SCAN_NODES = 256`, exporting `brief`, the
   feedback block's placement before `replyFormat`, the error-message wrapping,
   `{ cause: error }`, and discarding descendant violations on a later walk
   error are all declared (03-build-notes.md:8–77, 102–139) and match the code.
   One wording claim is off — finding 3.
4. **Scope creep.** None found. The src footprint is `independence.ts` and
   `runner.ts` only; the prompt footprint is the three named templates; no agent
   config, no `PipelineState` schema change (`failureNote` is handle state), no
   runner-executed formatter, budget/nudge/escalation/retryability flow
   unchanged (1524–1551, 1733–1761 mirror the pre-existing shape).
5. **Build-notes claims true.** Mostly verified: the nine pre-existing fakes
   really expose only `session.create/prompt/remove` (so deviation 2's premise —
   every audited attempt takes the neither-route throw — holds), no pre-existing
   test asserts gate-log counts or `pipeline status`/warn output, so the
   "no verdict changes" conclusion is sound; the ESLint `cause` fix is present at
   all three walk wrappers (563–577, 604–614, 640–654); the four-line→one-line
   `SessionChildApi.children?` form is 98 chars as claimed. Unverifiable from
   this seat: `npm run check` green / 241/241. See findings 3 and 7.
6. **Error paths exist and fail as designed.** Pass: walk fetch errors are
   caught, warned, gate-logged naming the session, and the attempt is audited
   parent-only; the neither-route case throws into that same handler; the
   matcher still fails closed on real accesses (child reads/globs/writes pinned
   at independence.test.ts:296–346 and end to end in child-deeds-retry.test.ts).
7. **Previous round's blocker/major findings addressed.** N/A — first pass.

## Findings

1. **minor — runner.ts:1070–1111: the child walk re-runs once per nudge cycle,
   not "once per attempt".** The audit block sits inside `executeInference`'s
   outer loop, and every nudge path (`continue` at 1148/1174/1190/1212) re-enters
   it. The proposal's cost paragraph fixes it as "once per attempt after the
   reply, off the poll loop". Consequences, all new side effects that the
   pre-existing pure matcher did not have: up to `MAX_NUDGES + 1` = 9
   `independenceAudit` gate entries per attempt when the walk route is missing
   (which is every audited attempt of the nine pre-existing fakes, and of any
   client with neither SDK route), crowding the six-entry "recent gates" view
   (runner.ts:2300–2302) and the persisted checkpoint with near-identical lines;
   and a repeat of the whole session-list crawl plus one full paged message fetch
   per descendant, per nudge. *Fix:* run the walk at most once per attempt —
   hoist a `walkAudited` flag (or compute the descendant violations before the
   parse/nudge paths and reuse them), keeping the parent-list audit where it is.
2. **minor — runner.ts:637–668: the `session.list` fallback pagination has no
   non-advancing-cursor guard.** The crawl breaks only when
   `pageCursorNext(page)` is absent. If the endpoint accepts no `cursor`
   parameter (A4/A8 leave it unverified) yet answers with a `cursor.next`, the
   `for (;;)` never terminates: the BFS visited set cannot save it because the
   pagination loop is separate, and `byParent` just accumulates the same page's
   ids forever — the attempt hangs in the post-reply audit with only per-request
   timeouts. *Fix:* break when `next` equals the cursor just sent (or has been
   seen before), and consider a page/node cap for the enumeration.
3. **minor — 03-build-notes.md:28 and 99–100 describe the change set wrongly
   about tests.** "test files are not editable at this stage" / "No test files
   were modified" while the change under review carries
   `tests/pipeline/child-session-deeds.test.ts`,
   `tests/pipeline/child-deeds-retry.test.ts` and extended pins in
   `tests/pipeline/prompts.test.ts:207–212,359–372` — i.e. the Test strategy
   #1–#8 pins do exist, but the notes' "Files touched" omits them and the wording
   invites a reviewer to conclude the pins are missing. *Fix:* list the test
   files in the change set (noting the test-authoring stage authored them) so the
   notes describe the diff a reviewer is handed.
4. **nit — runner.ts:585: `listChildSessionMessages` is exported.** The
   proposal's snippet keeps it module-private; the export exists so
   `child-session-deeds.test.ts:80–86` can drive the walk against fake clients,
   which Test strategy #5 requires. Undeclared widening of the module's surface.
   *Fix:* add one line to the build notes (or drive the walk only through the
   audit call site).
5. **nit — independence.ts:97–98 (+ independence.test.ts:249–262): the
   delegation exemption keys on the part name `subagent`, which only fixtures
   establish.** If a real server records the delegation part under another name,
   `subagent: []` is inert: the false positive is still cured (the unknown-tool
   path never scans payload keys such as `prompt`/`task`/`agent`), but pin (a)
   would then pin nothing production hits. *Fix:* confirm the recorded tool name
   from one real message list and map that name too.
6. **nit — runner.ts:1084–1092: the zero-descendant tripwire reads only the
   parent list.** §2 says "when an audited message list holds a completed
   `subagent` part"; a *child* list that itself holds a completed delegation
   whose own descendants were not enumerated warns about nothing. *Fix:* run
   `hasCompletedDelegation` over each audited child list too, or record the
   narrower reading in the build notes.
7. **nit — claims this seat could not evaluate (not a code defect).** Shell was
   denied, so `npm run check` (build notes: "lint clean, prettier clean, 241/241
   tests passing") and the exact diff boundary were not executed here; `typecheck`
   is not part of `check` (package.json:23), so a type slip in the new code would
   not fail the gate the notes cite. Everything else above was checked statically.
   *Fix:* none for the change; the next seat with shell should re-run the gate and
   `npm run typecheck` once.

## FYI (no severity)

The walk's enumeration in the `session.list` route is server-wide (no
location/directory filter) before the parent-scoped BFS narrows it; per-attempt
cost on a busy server follows the proposal's accepted cost paragraph, so it is
not a finding — noted only so the first post-merge dogfood run can time it.

## Verdict

approve — no blocker or major findings; findings 1–3 are worth a fix, 4–7 are
records.
