---
description: Reviews the pipeline branch diff for security against the proposal; produces a report and a JSON verdict
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
  - action: read
    resource: "docs/designs/**/reviews/**"
    effect: deny
  - action: shell
    resource: "*"
    effect: deny
  # Toolchain: the stage closes its own loop instead of burning a fix
  # round on a formatter it was never allowed to run.
  - action: shell
    resource: "cargo *"
    effect: allow
  - action: shell
    resource: "rustfmt *"
    effect: allow
  - action: shell
    resource: "npm run *"
    effect: allow
  - action: shell
    resource: "npm test *"
    effect: allow
  - action: shell
    resource: "npx vitest *"
    effect: allow
  # Read-only inspection. A compound command is scanned into subcommands
  # and every leg must pass, so `cd <dir> && cargo test` needs this one.
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
  # A review seat reads; it cannot rewrite the tree it is grading. The
  # read-only `--check` leg stays open so the seat can verify the gate.
  - action: shell
    resource: "cargo fmt *"
    effect: deny
  - action: shell
    resource: "cargo fmt *--check*"
    effect: allow
  - action: shell
    resource: "cargo fix *"
    effect: deny
  - action: shell
    resource: "cargo clippy --fix*"
    effect: deny
  - action: shell
    resource: "npm run format *"
    effect: deny
  # The runner owns commits, branch state and file moves (ADR-005), and a
  # shell write that bypasses the ownership audit stays closed. These come
  # last because the last matching rule wins.
  - action: shell
    resource: "git commit *"
    effect: deny
  - action: shell
    resource: "git push *"
    effect: deny
  - action: shell
    resource: "git pull *"
    effect: deny
  - action: shell
    resource: "git checkout *"
    effect: deny
  - action: shell
    resource: "git switch *"
    effect: deny
  - action: shell
    resource: "git reset *"
    effect: deny
  - action: shell
    resource: "git clean *"
    effect: deny
  - action: shell
    resource: "git rebase *"
    effect: deny
  - action: shell
    resource: "rm *"
    effect: deny
  - action: shell
    resource: "mv *"
    effect: deny
  - action: shell
    resource: "tee *"
    effect: deny
  - action: shell
    resource: "sed -i *"
    effect: deny
  - action: shell
    resource: "dd *"
    effect: deny
  - action: shell
    resource: "cargo publish *"
    effect: deny
  - action: shell
    resource: "npm publish *"
    effect: deny
  - action: webfetch
    resource: "*"
    effect: allow
  # ADR-016 Decision 3: the run's records are denied in the permission layer as
  # well as audited. Coarse on purpose — a shell rule judges raw command text
  # and a grep rule the requested regex — so these stop the plain-spelling
  # access and the deed audit stays the backstop.
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
  # The exception that keeps Decision 1 open, last so it wins: this seat may
  # re-read the report its own specialty writes, through the `git diff` allow.
  # The wildcard's extra reach over this seat's prior-round reports and another
  # design directory's same-named report is the recorded seam — the audit's
  # design-directory-scoped identity rule is what actually bounds it.
  - action: shell
    resource: "git diff *reviews/code/security-*"
    effect: allow
---

Load the `code-review` skill and apply the `security` checklist to the diff
range and paths given by the caller. Reply with the JSON verdict object only.
