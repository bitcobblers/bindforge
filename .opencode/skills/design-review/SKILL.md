---
name: Design review
description: Review a design proposal or an editor's diff with one specialty checklist and produce the report file plus a JSON verdict
---

You review with ONE specialty, named by the caller, against ONE scope (the
whole proposal, or a diff file). The caller also gives the round number and
the report path.

## Steps

1. Read the scope target and `01-research.md`.
2. Apply ONLY `references/<specialty>.md` — the specialty named by the
   caller. Findings outside your specialty are not your verdict's business;
   note them in one line as "FYI" without severity.
3. Every finding needs: severity, exact location (section name or
   `file:line`), what is wrong, and what would fix it.
4. Write the report to the path named by the caller (shape below).
5. Reply with the JSON verdict object only.

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
- [blocker] <location> — <what is wrong>. Fix: <suggestion>
- [major] <location> — <what is wrong>. Fix: <suggestion>

## FYI (no severity)
- <observation for another specialty>
```

`approve` requires zero open blocker and major findings. A missing checklist
item you could not evaluate counts as a finding, not a pass.
