---
name: Plan note
description: Write the light flow's plan document with its five fixed sections
---

Read the source document at the path given by the caller. Structure the plan
exactly like `references/template.md`; keep every heading, and say nothing
the source document does not support.

## Rules

1. The plan is the run's only design document: it is written once, before any
   code, and judged by two code seats — a `correctness` seat against the
   approach and a `testing` seat against the Test plan.
2. Name components, files, and test files concretely, matching the real
   repository layout; the coder implements exactly what Files to touch lists.
3. The Test plan states what the check gate must prove; it is the section the
   testing seat checks the implementation against.
4. The plan's line cap is named in your prompt, and it is the gate's number:
   a plan over it stops the run. Fit the five sections inside it.
5. Write only the output path named by the caller. Reply with the JSON
   verdict object only.
