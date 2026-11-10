# Security review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`, c0, round 1). Inputs: proposal +
Revision log, `docs/prompts/audit-by-deeds.md`. No prior security findings of
mine to re-check. Revision log read first: none of the r0 decisions below are
re-opened; each finding is about a hole the r0 decision leaves unobservable or
unbounded, not about the decision itself.

## Findings

- [major][new] Chosen design §2 Deeds of children — "once per attempt after the
  reply — off the poll loop, bounded" — the walk is a point-in-time snapshot of
  the descendant subtree, and the design never says what happens to a descendant
  that is still running when that snapshot is taken. A `subagent` delegation run
  in the background (or any child that outlives the parent's reply) contributes
  no completed parts to the audited lists, so its read of a `transcripts/` file
  is never seen for that attempt — and since the attempt's session is the walk's
  root, that subtree is never re-audited by a later attempt. Today the crude
  delegation-text scan at least flags such an attempt (that is the false positive
  the design removes); after `subagent: []` the parent carries zero signal and
  the child's deeds are absent from the snapshot, so this is a *net* detection
  loss in exactly the class the mechanism exists to catch (source §1: "The
  child's deeds are the violation"). Nothing in §2, the Risks table (whose
  child rows cover misattribution and fetch failure), or Test strategy #5–#6
  acknowledges it — every child fixture is a completed one. Fix: state the
  snapshot semantics in §2 ("the audit does not wait for descendants still
  running at reply time") and add the matching Risks row with the A10
  mitigation, same treatment the design already gives the fetch-error fallback;
  if a mechanism is preferred, reuse the existing `bounded()` single-retry
  pattern for one settle re-fetch of descendants that had in-flight parts at
  snapshot time, replacing the flat "once per attempt" sentence.

- [major][new] Chosen design §2 Deeds of children — "else `client.session.list()`
  filtered by `parentID === sessionID`" — the fallback enumeration has no silent
  failure mode guard. The message fetch is explicitly paged (A6: "the existing
  paged `listSessionMessages` (asc order, `cursor.next`)") but the session-list
  fallback is specified with no paging and no result-shape assumption, so
  children beyond the first page — or a `SessionInfo` shape without `parentID`,
  which A8 already concedes is unverifiable here — yield an empty subtree and
  therefore *no* error and *no* log: the new detection is inert on every attempt
  while every stage passes. A throw from a missing `session.children` method at
  least lands in the §2 fetch-error catch; an empty-but-successful enumeration
  lands nowhere. Tests #5 and #6 cannot catch it: both drive fake clients that
  always return children. With `subagent: []` the walk is the only signal the
  audit has for delegated deeds, so inertness is invisible *and* consequential.
  Fix: in §2, page the fallback the same way as `listSessionMessages` (`cursor.next`
  until exhausted) and add one tripwire: when an audited message list contains a
  completed `subagent` part but the walk enumerated zero descendants,
  `console.warn` naming the enumeration gap — this is the same log-only posture
  the design already chose for fetch errors, so it adds no new failure mode.

- [minor][new] Chosen design §2 Deeds of children — "A child-walk fetch error
  after `bounded()`'s single retry is caught, logged (`console.warn`), and the
  audit proceeds on the parent list alone" — the r0 rejection of failing the
  stage on an infrastructure fault is sound and I do not re-argue it, but the
  degradation is recorded only on stdout, which is not the run's record: gate
  entries and the gate log are what the next round and the operator read. An
  attempt audited on a partial subtree is indistinguishable in the record from a
  fully audited clean attempt, and the design's own §2/§3 chain then feeds the
  "clean" verdict into the attempt-2 prompt. Fix: keep parent-only proceeding,
  and additionally write one gate-log line through the existing `gateLog`/
  `printable()` path (runner.ts:1144–1161) naming the partial audit and the
  session it could not enumerate — log only, no stage failure, no schema change.

- [minor][new] Chosen design §1 Matcher — "Unknown tools' `pattern` is treated as
  a glob-style access, matching the glob precedent; the grep-style reading of
  `pattern` as query text is a known residual (Risks row 2)." — the residual is
  a false positive (a search tool's regex query naming `transcripts` fails an
  attempt that looked nowhere), but Risks row 2 is the opposite direction: "An
  unknown tool names its access under a key outside `PATH_ACCESS_KEYS` → missed
  violation". The claimed mitigation therefore does not exist: the FP class that
  this document exists to eliminate (run-005's 977 s loss, the perverse
  quote-the-rule loop) is asserted and then left untracked. The same applies to
  `command`, whose value can name `transcripts/` inside a message string. Fix:
  keep the allowlist decision (r0) and correct the cross-reference by adding a
  Risks row for the false-positive direction, with the mitigation the design
  already has — the violation string names the offending key and value, so a
  mis-scanned key is a one-line allowlist fix — and note that the known
  search/glob tools are already in `ACCESS_FIELDS` with their exemptions pinned,
  so the FP only reaches genuinely unknown tools.

- [minor][new] Chosen design §3 Retry feedback — "each with `failureNote:
  printable(out.failure ?? 'no reply')`" and the appended "Do not repeat the
  behavior that caused that failure." — the note is the design's only new
  data-flow *into* an agent, and it is classified for one threat only: A9 shows
  quoting forbidden paths is safe for the audit, and `printable()` strips
  control bytes. Neither bounds content. For non-audit gates `out.failure` can
  carry verbatim tool or test output — text an injected or misbehaving agent
  influenced — now injected into a fresh session's prompt with no length bound,
  on top of the rendered template. This does not ask for the structured
  "rule broken" field that r0 rejected. Fix: build `failureNote` through the
  existing `brief()` helper (already used for violation text in §2) so the note
  is length-bounded, and label the appended block as a quoted log excerpt
  ("Previous attempt's gate failure (log excerpt): …") at the injection point.

- [nit][new] Chosen design §1 Matcher — "collect string values found under an
  allowlist of access-naming keys, recursively, in any nesting depth" — the walk
  into server-recorded, agent-produced tool input is unbounded recursion, and
  the §2 error catch covers fetch/parse faults, not a `RangeError` thrown while
  matching the parent's own list. Since this branch is being rewritten anyway,
  bound it: iterative queue with a node/depth cap (drop the deepest nodes, do not
  throw), so a hostile or merely enormous tool input cannot take the audit call
  site down.

- [nit][new] Chosen design §2 Deeds of children — "`client.session.list()`" reads
  every session on the server to select descendants, which is broader than the
  walk needs and is a new read the design does not scope. Fix: one clause stating
  the list is used only to build the descendant set and is never logged or
  persisted, so unrelated sessions' metadata cannot reach the gate log or the
  attempt-2 note.

## Checklist coverage

- 1 Trust boundaries: new reads (session list, children, descendant message
  lists) are all same-origin local OpenCode server records, already the audit's
  existing domain — no new trust domain; agent-produced tool-input strings are
  the untrusted payload the matcher judges. No finding, but see the §2 nit.
- 2 Parsing untrusted data: see the recursion-bound nit.
- 3 Path handling: the matcher only string-matches recorded fields; no
  canonicalization claim is made or needed, no path is interpolated into a shell
  string, nothing executes the values it reads (`command` is read, never run).
- 4 Secrets/PII: nothing leaves the machine; the new flows move runner-generated
  text into gate log/prompts already inside the run record. See the `failureNote`
  minor for content bounding.
- 5 Dependencies: `@opencode/client ^2.1.4` — no advisory on the client package
  itself (npm health: no known vulnerabilities; 2 deps, `@opencode/schema`,
  `@opencode/protocol`). `anomalyco/opencode`'s published advisories
  (GHSA-vxw4-wv6m-9hhh, GHSA-c83v-7274-4vgp, GHSA-632h-h47v-g4x4) target the
  server/CLI/web UI, not `@opencode/client`, and predate or are fixed outside the
  pinned line; no unfixed advisory blocks. `@trivago/prettier-plugin-sort-imports`
  (A12): no advisories. No new dependency is introduced.
- 6 Least privilege: no new filesystem/shell capability; the walk adds read-only
  server calls (§2 nit for scope).
- 7 Fail closed: parent-scoped BFS with a visited set terminates on a malformed
  `parentID` graph — good. The two silent-degradation paths are the two majors
  and the gate-log minor.

## FYI (no severity)

- Source §1 requires enumerating children "created at or after the attempt's
  `since` stamp"; §2 deliberately drops the creation-time filter (A8) and relies
  on the message-level `since` filter. Deviation from a source clause — correctness
  / compatibility seat's call, not mine; the security direction is a false
  positive, not a bypass.
- Rollout 1 runs `npm install` against `^2.1.4` while `@opencode/client` is at
  2.2.x; the surface the walk is written against (and the `SessionInfo` shape in
  A8) should be the lockfile-resolved version, not a caret range.
- Test strategy #9: extending nine fake clients "with the session-list method
  their flows need" — pin at least one fake whose list call *fails* or returns a
  second page, otherwise #5–#6 only ever exercise the happy path that both
  majors above hide.
- Applying the parent's `ownReportPath` to children is a good closing of the
  "delegate the forbidden write" loophole; note it can also flag a child writing
  scratch files, an FP question for the correctness seat.
