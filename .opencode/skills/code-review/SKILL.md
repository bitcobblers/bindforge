---
name: Code review
description: Review the pipeline branch's diff against the parent branch with one specialty and produce the report file plus a JSON verdict
---

You review the code change with ONE specialty, named by the caller. The
caller gives the design directory, the diff range (default: the pipeline
branch against `main`), the round number, and the report path.

## Steps

1. Read `02-proposal.md` and `03-build-notes.md` — the code is judged
   against the design, not against the coder's intent.
2. Get the change: `git diff main...HEAD` (or the range the caller names).
3. Apply ONLY `references/<specialty>.md` — the specialty named by the
   caller. Out-of-specialty observations go in one FYI line, no severity.
4. Every finding needs: severity, `file:line`, what is wrong, what would
   fix it.
5. Write the report to `reviews/code/<specialty>-r<N>.md` at the path the
   caller names.
6. Reply with the JSON verdict object only.

## Severity

Same scale as design review: `blocker`, `major`, `minor`, `nit`.
`approve` requires zero open blocker and major findings. A checklist item
you could not evaluate counts as a finding, not a pass.
