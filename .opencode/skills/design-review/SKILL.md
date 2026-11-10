---
name: Design review
description: Review a design proposal or an editor's diff with one specialty checklist and produce the report file plus a JSON verdict
---

You review with ONE specialty, named by the caller, against ONE scope (the
whole proposal, or a diff file). The caller also gives the round number and
the report path. Earlier reports for every specialty — including yours —
live under the run's `reviews/` directory; check your own previous report
before calling a finding new.

## Steps

1. Read the scope target and `01-research.md`.
2. Apply ONLY `references/<specialty>.md` — the specialty named by the
   caller. Findings outside your specialty are not your verdict's business;
   note them in one line as "FYI" without severity.
3. Every finding needs: severity, location, what is wrong, and what would
   fix it. In `fix`, give the smallest change that resolves the finding; if it
   introduces a mechanism, name the text or behavior it replaces. A fix that
   only adds has usually moved the problem rather than solved it. Anchor the
   location by **section name plus a short verbatim quote** of the passage. Line numbers are optional: every revise round
   renumbers the document, so a `file:line` anchor is stale before the
   report is even read. Confirm a location once; do not re-read the same
   passage to re-derive the same number.
4. Check the Revision log at the end of `02-proposal.md`: one current entry
   per decision the author has made, including what they deliberately did not
   change. Re-report a rejected finding only if you can show the rebuttal
   itself is unsound — and say that in `problem`. Conversely, closures
   introduce defects: report what newly revised text breaks even when the
   earlier theme is closed.
5. `02-proposal.md` carries a `## Deferred / out of scope` section: concerns
   the design hands to another document. Report one only when it contradicts
   the source document's Decision or its stated requirements, and name the
   clause it contradicts. A deferral consistent with the source document is
   not a finding, and is not re-opened by repeating it.
6. Label each finding `new`, `re-raised` (also open in the previous round)
   or `regression` (was closed, now back in changed text). The label
   changes no gate outcome; it tells the reviser where to spend the round.
7. Write the report to the path named by the caller (shape below).
8. Reply with the JSON verdict object only.

Inside `execute`, `fetch` is HTTP-only; use `read`, `grep` and `glob` for
repository files.

## Severity

- `blocker` — the design cannot proceed as written
- `major` — must be fixed before implementation starts
- `minor` — should be fixed; does not block
- `nit` — preference

## Report shape

```md
# <specialty> review r<N>

verdict: approve | request-changes

## Findings
- [blocker][re-raised] <section> — "<quoted snippet>" — <what is wrong>. Fix: <suggestion>
- [major][new] <section> — "<quoted snippet>" — <what is wrong>. Fix: <suggestion>

## FYI (no severity)
- <observation for another specialty>
```

`approve` requires zero open blocker and major findings. A missing checklist
item you could not evaluate counts as a finding, not a pass.
