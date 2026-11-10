# security review r2

verdict: approve

Pass type: editorial confirmation (round 2). Scope: `reviews/design/editorial-r2.diff`
only. Inputs: `02-proposal.md`, `docs/prompts/audit-by-deeds.md`, the Revision log,
and my previous security findings — which are none (the editorial phase runs a
single language pass per round). Per the ratification rule, I report only if the
editorial revisions broke something previously approved or made a cited section
false.

## Checklist pass over the diff (scope = the diff's hunks)

Every hunk in `editorial-r2.diff` was checked against the security checklist for
any change in technical meaning. All nine hunks are language-only:

1. Goals — "never evidence from their input text" → "never judged on their input
   text". Equivalent to the design's `subagent: []` rule (§1) and to the one-line
   principle ("judges where a tool looks, never what text it carries"); does not
   admit text back as evidence. No checklist item touched.
2. Non-goals — "re-affirmed rejected" → "rejection re-affirmed". Grammar only;
   the run-003 formatter rejection (A11) stands.
3. Options/word-based exemption — "keeps" → "perpetuates" the perverse loop.
   Argument unchanged; the rejection of vocabulary curation stands.
4. Options/depth-1 walk — "for the price of zero extra machinery" → "closing it
   costs no extra machinery: a BFS…". Same conclusion; the subtree-BFS coverage
   (grandchildren reached, blind spot closed) is preserved and clearer. The
   malformed-parentID-visited-set guard (§2) is untouched.
5. Deeds of children — "is caught, logged" → "is caught and logged". The
   fail-open-on-infrastructure-fault posture (parent-only audit, `console.warn`
   + `gateLog` line, zero-descendant tripwire) is semantically identical to the
   r1 revision-log closure. Checklist item 7 (fail closed) was already accepted
   for this case at r1: it is a logged detection gap behind the fence as primary
   control (A10), not a crash or partial application — the diff changes nothing
   here.
6. Retry feedback — "and `handle.resume` is absent" → ", if `handle.resume` is
   absent". The original was ungrammatical; the condition set is identical, so
   the `brief(printable(...))` bounding of `failureNote` (A9 posture) is intact.
7. Validation posture — "its fixes" → "the design's fixes". Antecedent clarity
   only; dogfooding posture under the current matcher unchanged.
8. Risks table — "the attempt audited parent-only" → "the attempt is audited
   parent-only". Grammar only.
9. Rollout — "stage" → "stages". Grammar only.

No hunk alters a data-source classification (item 1), untrusted-input parsing —
the node-capped queue and paged fetches are untouched (item 2), path handling
(item 3), logging/secrets posture (item 4), any dependency claim (item 5 —
`@opencode/client ^2.1.4` text is not touched by the diff), privilege surface
(item 6), or fail-open/fail-closed behavior (item 7). The Revision log's r1
closures (paged enumeration, snapshot semantics, gate-logged partial audits,
tripwire, single `failureNote` seam) are all still present in the current text;
the diff did not disturb them.

## Findings

(none)

## FYI (no severity)

- The re-wrapped sentence in Validation posture now runs ~82 chars, marginally
  over the document's usual prose wrap; existing text (e.g. the Context links
  line) already exceeds 80, so this is at most a language-seat style call.
