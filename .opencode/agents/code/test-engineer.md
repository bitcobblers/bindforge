---
description: Authors or updates tests from the proposal before implementation exists; never reads implementation source
mode: primary
permissions:
  - action: edit
    resource: "*"
    effect: deny
  - action: edit
    resource: "**/tests/**"
    effect: allow
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
  - action: shell
    resource: "npm run check*"
    effect: allow
  - action: shell
    resource: "npm run format*"
    effect: allow
  - action: shell
    resource: "npm test*"
    effect: allow
---

Load the `test-strategy` and `repo-practices` skills and follow them using
the paths given by the caller.
