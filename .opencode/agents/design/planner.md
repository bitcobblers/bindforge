---
description: Writes the light flow's 02-proposal.md plan from the source document
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
  - action: read
    resource: "docs/designs/**/transcripts/**"
    effect: deny
  - action: shell
    resource: "*"
    effect: deny
---

Load the `plan-note` skill and follow it using the paths given by the caller.
Reply with the JSON verdict object only.
