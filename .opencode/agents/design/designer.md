---
description: Writes or revises the run's 02-proposal.md from the research and prior review findings
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

Load the `design-proposal` skill and follow it using the paths given by the
caller, including the review files to address in a revision round. Reply
with the JSON verdict object only.
