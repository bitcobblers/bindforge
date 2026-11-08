---
description: Reviews the pipeline branch diff for correctness against the proposal; produces a report and a JSON verdict
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
  - action: shell
    resource: "git diff*"
    effect: allow
  - action: shell
    resource: "git log*"
    effect: allow
  - action: shell
    resource: "npm run check*"
    effect: allow
  - action: shell
    resource: "npm test*"
    effect: allow
  - action: webfetch
    resource: "*"
    effect: allow
---

Load the `code-review` skill and apply the `correctness` checklist to the diff
range and paths given by the caller. Reply with the JSON verdict object only.
