---
name: Design proposal
description: Write or revise the run's 02-proposal.md from the research, following the required section template
---

Read the source document and `01-research.md` at the paths given by the
caller. Structure `02-proposal.md` exactly like `references/template.md`;
keep every heading.

## Rules

1. Every assumption — of yours or inherited from the research — goes in the
   Assumption register with status `verified` or `unverified` and a basis
   (URL or file reference).
2. Body text may not state more than the register supports. Unverified
   assumptions stay worded as assumptions.
3. Name components, files, and interfaces concretely, matching the real
   repository layout.
4. Compatibility matrix: one row per platform; "unknown" is allowed, an
   empty cell is not.
5. Test strategy: state what unit tests and what integration tests must
   prove, numbered, so the test engineer can cite "Test strategy #N".
6. Revision rounds: address **every** open blocker and major finding from
   `reviews/design/`, then **maintain** the Revision log. It is an index of
   the run's decisions, not a transcript: one entry per finding that is still
   material, at most two lines, disposition first. When a round supersedes an
   earlier decision, replace that entry where it stands; never stack a
   correction underneath it. Anchor entries by section name, not line number —
   reviewers locate passages by section and quote, and every round renumbers
   the document. The full record already exists outside the proposal: one
   commit per revise round, the reports in `reviews/`, and the transcripts.
   Do not duplicate it inside the design.
7. Write only the output path named by the caller. Reply with the JSON
   verdict object only.
8. Each claim lives in exactly one place. When a finding touches a claim
   that is repeated across sections, consolidate it into one canonical
   statement and cross-reference it from the others — a reviewer reports
   the drift between copies, not the copy itself.
9. Build `oldString` from a fresh `read` of the target lines, never from
   memory or a `grep` preview: both pad the line-number prefix, so
   indentation counted from them is off by one and this document indents
   nested prose 1-8 spaces. Start `oldString` after a line's leading
   whitespace and take enough of the line to be unique. If an edit
   reports `Could not find oldString`, re-`read` the region - do not
   re-guess the indentation with anchored `^ {n}` patterns.
10. Inside `execute`, `fetch` is HTTP-only. Reach for `read`, `grep` and
   `glob` for repository files.
11. Choosing a move: resolve a finding with the smallest change that makes it
   stop being true — delete the passage or sub-system that produced it,
   rewrite or narrow it so the conflict cannot arise, defer the concern to
   `## Deferred / out of scope`, or add text. Take the first that works;
   addition is the last resort, not the default. Revise rounds aim for a
   net-zero or net-negative line count; if the diff is net positive, justify
   it in one sentence in the reply `summary`.
12. Trace every section to a clause of the source document's Decision. A
   passage that traces to nothing is either deleted or listed in
   `## Deferred / out of scope` with the document that owns it. Designing a
   concern into the proposal commits you to it; recording it as deferred does
   not.
