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
   `reviews/design/`, then append to the Revision log: round number, which
   findings you addressed, and what you did NOT change and why.
7. Write only the output path named by the caller. Reply with the JSON
   verdict object only.
