# correctness review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` only (single editorial confirmation
pass, round 2). Correctness checklist applied to the diff against
`02-proposal.md`, the Revision log, and `docs/prompts/audit-by-deeds.md`.
No prior correctness findings to re-check; the Revision log entries (r0/r1)
were checked against the post-diff text and remain consistent with it.

## Findings

None.

Diff-by-diff check (all nine hunks):

- Goals, "never evidence from their input text" → "never judged on their input
  text" — preserves the source §1 requirement ("never treat the prompt text in
  the parent's tool input as evidence") and matches §1's `subagent: []`
  treatment (the part is inspected for completion by the §2 tripwire, but its
  input text is never judged). No meaning drift.
- Non-goals "rejection re-affirmed in run 003" — grammar-only; the A11 pin
  citation is untouched.
- Options, word-based exemption "perpetuates the perverse loop" — still
  matches the source's named "perverse feedback loop" mechanism.
- Options, depth-1 walk rewrite ("closing it costs no extra machinery: a BFS
  ... costs one children call per session in that subtree") — same argument;
  the two-level-delegation premise is supported by the cited
  https://opencode.ai/docs/agents/ (the `subagent` permission "controls which
  agents it may launch" in child sessions; the built-in `general` is
  singled out as one that "cannot launch more subagents", implying custom
  subagents can). Consistent with §2's BFS + visited set.
- §2 fetch-error "caught and logged" — grammar-only; still agrees with the
  Risks row "Child-walk fetch fails and the attempt is audited parent-only".
- §3 "when building `fullPrompt` (runner.ts:693–710), if `handle.resume` is
  absent and `failureNote` is set, append" — makes the existing conditional
  explicit; matches "Set on a fresh-session attempt 2" and the two non-
  resumable set-sites. No behavior change.
- Validation posture "the design's fixes" — clarifies "its"; still matches the
  source's Notes section.
- Risks row "the attempt is audited parent-only" — grammar fix, agrees with §2.
- Rollout 3 "design-review stages" (plural) — correct: the prompt lines touch
  test-authoring, test-update, and design-review.

Checklist coverage:
1. Assumption register — no assumption row or its basis citation is touched by
   the diff; register unchanged.
2. URL spot-check — the proposal's two cited URLs fetched:
   https://opencode.ai/docs/agents/ and https://opencode.ai/docs/server/.
   Both live and support the claims they back (subagents run in child
   sessions; `GET /session`, `GET /session/:id/children`, `parentID` in the
   session schema). No dead or contradicting source.
3. Internal contradictions — none introduced; each rewritten sentence still
   agrees with §1/§2/§3, Risks, and Compatibility matrix.
4. Edge cases — diff adds no mechanism, so no new edge cases.
5. Interfaces — no name/shape changed; `failureNote`, `PATH_ACCESS_KEYS`,
   `listChildSessionMessages`, `brief(printable(...))` phrasing untouched.
6. Source decisions — unchanged by an editorial-only pass.
7. Prior blocker/major closures — none open (no prior correctness findings).

## FYI (no severity)
- Assumption register A3 — its `server`-URL basis (https://opencode.ai/docs/server/)
  self-labels "These are the docs for OpenCode V1" (pointing V2 readers to
  /docs/llms.txt). Pre-existing text untouched by this diff, and the design
  already hedges the exposure (A4 unverified; Compatibility matrix row makes
  the route determinate at step 1 with three server-side fallbacks), so not a
  finding on this pass — flag for the next full-proposal correctness pass if
  A3's "verified" status is revisited.
