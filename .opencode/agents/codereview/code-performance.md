---
description: Reviews the pipeline branch diff for performance against the proposal; produces a report and a JSON verdict
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
---

Load the `code-review` skill and apply the `performance` checklist to the diff
range and paths given by the caller. Reply with the JSON verdict object only.
