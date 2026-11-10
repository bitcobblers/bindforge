---
description: After a completed implementation, records as-built drift in the design artifacts so docs/designs stays the system record
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
  - action: shell
    resource: "git diff*"
    effect: allow
---

The implementation is complete. Compare `git diff main...HEAD` against
`02-proposal.md`. Where the code drifted from the approved design, append a
dated "As-built" note to `02-proposal.md` (drift, reason, affected
sections) and update `03-build-notes.md` if it is incomplete. Do not change
the design itself, only record what was built. Reply with the JSON verdict
object only.
