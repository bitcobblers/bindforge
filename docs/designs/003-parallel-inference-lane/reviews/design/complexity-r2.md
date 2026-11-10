# complexity review r2

verdict: approve

Scope: ratification/confirmation pass over `reviews/design/editorial-r2.diff` only.
Previous complexity findings: none. Per the pass rules, a finding is reported only
if the revised text broke something previously approved or made a cited section
false.

## Findings

None. Each hunk in the diff was checked against the complexity checklist:

1. No new package, module, or interface appears or changes in the diff — the only
   new module in the design (`src/pipeline/concurrency.ts`) is untouched by the
   editorial pass and remains justified by the rejected-`p-limit` option and ADR-003
   §Execution 3.
2. Moving parts unchanged: the escalation bullet's rewrite ("releases its permit and
   returns without prompting") preserves the same acquire/release semantics already
   specified; it adds no new mechanism.
3. No text the editorial pass touched changes the design's size or shape; every
   change is wording-level (hyphenation, articles, adjective order, tense:
   "dominant wall-clock cost", "semantics stay", "stays in runner code", "has
   today's fallback shape", "a range check", "the pass produces one commit").
4. No dependency changes.
5. No deferral/slice boundary changes.
6. No configuration-knob or abstraction changes; `--max-parallel` /
   `$OPENCODE_MAX_PARALLEL` and `maxParallelReviews` wording is unchanged in meaning.
7. Value check: proportionality verdict unchanged — the design remains a ~20-line
   semaphore plus a two-phase split and a drained writer, sized to the wall-clock
   goal; the editorial diff does not alter that assessment.

The rewritten escalation sentence matches ADR-003 §Execution 3 (escalate after
siblings settle, one sweep, one `finalizeEscalate`); "complete in-policy sibling
reports" keeps the same meaning as before. Nothing in the diff contradicts the
ADR or the revision log's r1 closures.

## FYI (no severity)
- "returns without prompting" in the escalation bullet could be misread as
  "without notifying"; the correctness/proposal-editor seats may want "returns
  without sending a prompt" — language nuance, not a complexity issue.
