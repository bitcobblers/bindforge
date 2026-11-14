---
name: Code review
description: Review the pipeline branch's diff against the parent branch with one specialty and produce the report file plus a JSON verdict
---

You review the code change with ONE specialty, named by the caller. The
caller gives the design directory, the diff range (default: the pipeline
branch against `main`), the round number, and the report path. The caller's
allowed inputs, diff scope, pass type, and output path are authoritative.
Use injected previous findings; do not open earlier review reports.

## Steps

1. Read `02-proposal.md` and the caller's allowed supporting inputs. Read
   `03-build-notes.md` only when the caller permits it.
2. Inspect exactly the caller's diff scope, including its path exclusions.
   Only when no scope is supplied, use `git diff main...HEAD`. On dissent or
   ratification passes, honor the caller's incremental scope rather than
   restarting a whole-branch review. Expand into surrounding source only as
   needed to establish a concrete finding.
3. Apply ONLY `references/<specialty>.md` — the specialty named by the
   caller. Out-of-specialty observations go in one FYI line, no severity.
4. Every finding needs: severity, `file:line`, what is wrong, what would
   fix it.
5. Write the report only to the exact path the caller names.
6. Reply with the JSON verdict object only.

## Severity

Same scale as design review: `blocker`, `major`, `minor`, `nit`.
`approve` requires zero open blocker and major findings. For each blocking
finding, identify a violated requirement or concrete failure scenario and cite
the evidence. Mark irrelevant checklist items not applicable. Record missing
evidence explicitly; make it blocking only when you identify the required
property that cannot be established and why that prevents proceeding. Do not
invent a defect to complete a checklist or treat a preference as major.
