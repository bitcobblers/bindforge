---
doc_type: adr
status: accepted
date: 2026-11-10
---

# ADR 007: Judge the independence audit on deeds, not on text

Recorded retrospectively: run `007-audit-by-deeds` was commissioned directly
from `docs/prompts/audit-by-deeds.md`, not from an ADR, and has since
implemented and merged this decision end to end — the status above records
the merge, not a pending acceptance. Design artifacts:
`docs/designs/007-audit-by-deeds/` (proposal with as-built drift, build
notes, review reports).

## Context

The review-independence audit (`tools/orchestration/src/pipeline/
independence.ts`) inspects what each stage attempt's session actually did —
completed tool parts and shell messages — and fails the attempt on a
run-records access (`transcripts/`, and `reviews/` for review seats). It is
defense-in-depth over the agent-permission fence, with a stated posture of
"deterministic plain-text coverage, nothing more."

Every tool not in `ACCESS_FIELDS` — including `subagent` and `execute` —
fell back to a full scan of every string in the tool input. That judges
*what text a call carries*, not *where the tool looked*, and it failed
attempts that accessed nothing: a delegation prompt reciting the
prohibition, or test code whose string literals name `transcripts/`. Run
005 lost three stages to these false positives (a 977 s `implement` loss,
two review-seat failures, and the prettier escalation at 5/5 fix rounds).
Two sibling defects rode along: a fresh-session retry gave attempt 2 a
byte-identical prompt with nothing about why attempt 1 failed, and two
prompt lines proven necessary by runs had no test pins to keep them.

## Decision

One principle across all three changes: **the audit judges where a tool
looks, never what text it carries.**

1. **Matcher: paths only, not payloads.** `subagent` gets an explicit empty
   access list — delegation text is not an access; its evidence is the
   child walk. Unknown tools are judged only on string values under an
   allowlist of path-naming keys (`path`, `filePath`, `file`, `directory`,
   `dir`, `workdir`, `command`, `include`, `pattern`), scanned
   breadth-first under a shared node budget (`MAX_PATH_SCAN_NODES = 256`).
   Payload keys (`code`, `prompt`, `content`, …) are never scanned.
2. **Deeds of children.** A child session's deeds are its parent attempt's
   deeds. The runner walks the attempt session's full descendant tree
   (BFS, visited-set) and audits each descendant's message list with the
   same options, appending violations as `(via child session …)`. Cost
   guards: the walk runs only when the attempt recorded a `subagent` part,
   at most once per growth of that count, clamped to the stage deadline.
   On a mid-walk fetch failure, violations already collected still fail
   the attempt; only the unseen remainder is a gap, logged via
   `console.warn` and an `independenceAudit` gate line. A zero-descendant
   enumeration next to a recorded delegation trips a warning, so an empty
   walk never masquerades as a clean audit. The enumeration route is
   probed per client at call time: SDK `session.children`, else paged
   `session.list` indexed by `parentID`, else a direct
   `GET /api/session/:id/children` using a new optional in-memory
   `server: { baseUrl, username?, password? }` on
   `RunOptions`/`ResumeOptions`.
3. **Retry feedback.** `StageHandle` gains `failureNote`; a fresh-session
   attempt 2 appends "Previous attempt's gate failure (log excerpt): …
   Do not repeat the behavior that caused that failure" to the rendered
   prompt, bounded by `brief(printable(…))`. Set at both attempt-2 sites —
   `runWithRetry` and `runReviewPass`'s seat lane. Budget, nudge loop,
   escalation wording, resumability rule, and the checkpoint schema stay
   untouched.
4. **Pinned prompt lines.** The design-review prompt gains the bright line
   (listing/globbing/grepping a forbidden directory counts as opening it);
   `test-authoring` and `test-update` gain the formatter duty (run
   `npx prettier --write` on touched files before replying). Both land as
   reviewed artifacts pinned by `toContain` on rendered prompt content.

## Alternatives considered

**Word-based exemption (don't flag prohibition sentences):** rejected —
the same words-not-acts error from the other side; hardened prompts quote
the rule more, raising a content scan's false-positive rate.

**Drop the fallback scan with no replacement:** rejected — a nested value
under key `path` names where a tool looks and must still fail; the existing
pin for that had to survive.

**Value-shape heuristic (flag path-looking strings):** rejected —
reintroduces content judgment and makes the verdict unpredictable, which
the audit's stated posture forbids.

**Audit the children inside `independence.ts`:** rejected — the matcher
stays a pure function over message lists; fetching children needs the
runner's client and would create a module cycle.

**Depth-1 child walk:** rejected — two levels of delegation would restore
the blind spot for the cost of one extra children call.

**Failure note in `runWithRetry` only:** rejected — the motivating failure
happened on the review-pass lane; one `StageHandle` seam covers both.

## Consequences

Becomes possible: delegation text and test-code literals stop failing
honest attempts; the audit now sees deeds it was blind to — a child
reading a `transcripts/` file or globbing `reviews/**` fails its parent
attempt; attempt 2 repairs instead of repeating attempt 1's behavior
byte-identically.

Accepted costs: the walk adds per-attempt server calls (bounded by the
delegation guard, the once-per-growth rule, the page budget, and the stage
deadline); unknown tools naming their access under an off-allowlist key
are a documented miss class, same family as obfuscated shells; a
descendant still running at the snapshot contributes only its completed
parts.

Left open, non-blocking at merge (recorded in the run's build notes): the
direct-HTTP third enumeration route has no test pin; the walk guard keys
on the recorded part name `subagent` on fixture evidence; the walk's
deadline clamp is unpinned. First post-merge dogfooded run is the live
check of the child walk and the attempt-2 note.
