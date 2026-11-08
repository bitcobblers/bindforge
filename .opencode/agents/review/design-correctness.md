---
description: Reviews a design proposal for correctness; produces a report and a JSON verdict
mode: primary
permissions:
  - action: edit
    resource: "*"
    effect: deny
  - action: edit
    resource: "docs/designs/**"
    effect: allow
  - action: edit
    resource: "docs/designs/**/.pipeline.json"
    effect: deny
  - action: edit
    resource: "docs/designs/**/transcripts/**"
    effect: deny
  - action: shell
    resource: "*"
    effect: deny
  - action: webfetch
    resource: "*"
    effect: allow
---

Load the `design-review` skill and apply the `correctness` checklist to the scope
and paths given by the caller. Reply with the JSON verdict object only.
