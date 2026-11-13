# compatibility review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` — the single editorial confirmation pass
of round 2, compatibility seat. Ratification criterion applied: report only what
the revisions broke of previously approved text or made false in cited sections.
This seat carried no findings in the prior pass.

## Findings

None.

Evaluation notes:

- All six hunks are language-only and none touches compatibility surface:
  the Goals bullet ("Precondition outcomes **as** → **are** verdict data"),
  two cross-reference corrections ("(Options)" → "(Options considered)", which
  now point at the actual `## Options considered` heading), "mapped one-to-one
  onto" → "to", "exactly as `set`/`swap`/manifest rows do today" → "as …
  already do", and the typo repair "the answer-why a bool cannot give" →
  "the answer a bool cannot give", which restores the claim A4 supports.
  Every post-diff line was confirmed present in the current `02-proposal.md`;
  the diff and the document agree.
- The compatibility matrix is untouched by this diff and still passes the
  checklist: every row carries a status (no empty cell, no `unknown`); the OS
  input APIs named by the checklist (X11, Wayland, Windows Raw Input, macOS
  HID) are explicitly named as untouched by anything shipping ("touches no OS
  input API … parsed documents in, parsed documents out"); the toolchain floor
  row (1.85.0, edition 2024, std-only, `Cargo.lock` gains exactly one package)
  keeps the native-module question pure — no prebuilds/node-gyp surface at all;
  keyboard reality (single `ansi-us` grid, IME/composition out of scope,
  closed physical-input vocabulary per A19) is stated by the Keyboard geometry
  row; file-path safety rides the slug-grammar-before-filesystem rule in
  `load_policy`.
- Test strategy #11 names a platform leg for every matrix row claiming
  support: the Linux CI runner (`check` gate's `rust` job) and the one-time
  Windows contributor run; macOS is `no (v1)`, so it needs no leg. Unchanged
  by this diff.
- Deferred items were checked against `docs/adrs/015-transforms-as-preconditioned-policies.md`:
  the handoffs to ADR-016/017/018 track the ADR's "Not decided here" list, and
  the AI-generated-base deferral matches the ADR's "gated by validation, not
  trusted by provenance" alternative. No contradiction found.

## FYI (no severity)

- The diff appends an unresolved "Editor notes:" line to the proposal flagging
  Test strategy #1's "aliasing `transforms_`" sentence as ambiguous, author to
  clarify. Its subject is grammar-closure wording, not compatibility — noted
  only because it leaves an open author action inside the document; it belongs
  to the editorial/complexity seats' attention, not this verdict.
