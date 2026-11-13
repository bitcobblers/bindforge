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
  # ADR-016 Decision 3: read-only inspection, star form so a bare command name
  # never grants reach. A compound command is scanned leg by leg, so
  # `cd docs && wc <file>` needs the `cd *` leg too. The toolchain stays out:
  # the leading `shell *` deny above keeps `cargo` and `npm` denied.
  - action: shell
    resource: "cd *"
    effect: allow
  - action: shell
    resource: "git status *"
    effect: allow
  - action: shell
    resource: "git diff *"
    effect: allow
  - action: shell
    resource: "git log *"
    effect: allow
  - action: shell
    resource: "git show *"
    effect: allow
  - action: shell
    resource: "ls *"
    effect: allow
  - action: shell
    resource: "cat *"
    effect: allow
  - action: shell
    resource: "head *"
    effect: allow
  - action: shell
    resource: "tail *"
    effect: allow
  - action: shell
    resource: "wc *"
    effect: allow
  - action: shell
    resource: "grep *"
    effect: allow
  - action: shell
    resource: "rg *"
    effect: allow
  - action: shell
    resource: "find *"
    effect: allow
  # The run's records are denied in the permission layer as well as audited.
  # Coarse on purpose — a shell rule judges raw command text and a grep rule the
  # requested regex — so these stop the plain-spelling access and the deed audit
  # stays the backstop.
  - action: shell
    resource: "*transcripts*"
    effect: deny
  - action: shell
    resource: "*reviews*"
    effect: deny
  - action: glob
    resource: "docs/designs/**/transcripts/**"
    effect: deny
  - action: glob
    resource: "docs/designs/**/reviews/**"
    effect: deny
  - action: grep
    resource: "*transcripts*"
    effect: deny
  - action: grep
    resource: "*reviews*"
    effect: deny
  # The checkpoint is a run record too: every seat's verdict and findings and the
  # gate record live there, and only the runner reads or writes it (ADR-006 §1).
  # Same coarse rule shape as the two directories above, same deed-audit backstop.
  - action: shell
    resource: "*pipeline.json*"
    effect: deny
  - action: read
    resource: "docs/designs/**/.pipeline.json"
    effect: deny
  - action: glob
    resource: "docs/designs/**/.pipeline.json"
    effect: deny
---

Load the `design-proposal` skill and follow it using the paths given by the
caller, including the review files to address in a revision round. Reply
with the JSON verdict object only.
