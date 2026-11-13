# correctness review r1 (c2, ratification)

verdict: approve

Scope: the diff `fc0a1f9 → 0dacb96` (Risks row "Policies are effectively
ANSI-US-tested", Compatibility matrix → new Keyboard geometry row + expanded
Product artifacts row, Deferred → ISO/JIS bullet, Revision log), then the full
current document. Checked only against `02-proposal.md` and
`docs/adrs/015-transforms-as-preconditioned-policies.md`, plus the shipped data
the new row's claims rest on (`schemas/keyboard-geometry/v1/` — exactly one
table file, `ansi-us.toml`; `crates/core/src/placement.rs:9-11`;
`crates/core/src/input.rs:1-18`).

## What the revisions do not break

- The stack, the match rule, the report vocabulary, the output/regeneration
  rules, the budget section and the assumption register are untouched by the
  diff, so the byte-identical-output and closed-reason-vocabulary contracts I
  approved in r1 stand unchanged. My r1 minor finding (collapse of a hostile
  source base's `set` to `resolve`'s action→input map, and the
  one-input-two-actions entry refusal) is still open in exactly the same
  wording, unregressed; the two r1 citation nits (A17 lines 23-33, A13's "two
  chain caps") are likewise still exactly as I quoted them.
- The Risks row's new pointer resolves: the Compatibility matrix now carries a
  row named "Keyboard geometry" that does state the claim restriction, so the
  mitigation is not a dangling cross-reference. The Deferred ISO/JIS bullet's
  pointer resolves the same way, and the requirement it used to state in place
  ("needed before a policy claims non-ANSI support") survives the move.
- `exactly one geometry ships (A20)` and `read the loaded grid` are confirmed
  against shipped data: `schemas/keyboard-geometry/v1/` holds one table file
  (`ansi-us.toml`) plus spec/fixtures, and `placement.rs:9-11` states the
  shape rule reads the grid. `closed-vocabulary physical input ids (A19)` is
  confirmed by `input.rs:1` ("The OS-neutral physical-input vocabulary").
- The Product artifacts addition ("touches no OS input API … parsed documents
  in, parsed documents out") is consistent with the Delta inventory (core edits
  + `crates/transforms`, std-only, no binary) and contradicts nothing; naming
  OS key-code registries in docs is not calling an OS API.

## Findings

- [nit][new] Compatibility matrix → Keyboard geometry — "ISO/JIS is new table
  files, not logic, and no policy may claim non-ANSI support until they land" —
  the moved sentence is now the authoritative place the Deferred bullet and the
  Risks row both point at, and in that seat it drops the qualifier the shipped
  geometry spec attaches to the same phrase ("A geometry that
  places a key the reach tables do not grade is a vocabulary addition, and it
  rides the same closed member sets"), which the proposal's own Deferred bullet
  already mirrors for other members ("widening the closed sets rides the
  `reach-classes` vocabulary major"). An ISO/JIS grid that places keys outside
  the 48-member ANSI-US-derived `keyboard:` set is therefore *also* a
  vocabulary major, not only table files — so the row, read as the answer to
  the ANSI-US risk, under-states the work and lets an implementer schedule
  ISO/JIS as data-only. Fix: inside the same clause, qualify it with the
  mechanism the document already names — "ISO/JIS is new table files, not
  logic, except where a grid places a member outside the closed `keyboard:`
  set, which rides the `reach-classes` vocabulary major (A19)" — rather than
  adding a new rule or a new section.
- [nit][new] Revision log — "Compatibility matrix → Keyboard geometry (r1):
  added the row Risks cites" — the log replaced the r0 placeholder with only
  the two matrix edits, so the r1 findings that the diff did not touch (this
  seat's collapse-rule/entry-refusal finding, and the A17 and A13 basis-citation
  nits) appear neither as changed text nor as a recorded rejection. A reader of
  the next round cannot tell "deliberately rejected" from "missed". Fix: append
  one line per unaddressed r1 finding with the author's reason (e.g. "stack
  collapse rule: unchanged — deferred to implementation with invalid-base tests
  in `crates/transforms/tests/apply.rs`"), leaving the two matrix entries as
  they are.

## FYI (no severity)

- Compatibility matrix rows are headed "Platform / environment"; "Keyboard
  geometry" is a data/feature axis, so the matrix now answers two different
  questions in one column pair — a formatting/taxonomy call, not a correctness
  one.
- "IME/composition is out of scope for v1" appears only in the matrix row; the
  Non-goals list and the Deferred section never carry it, so the exclusion has
  no home that survives the proposal. The existing Non-goals bullet on widening
  the closed input vocabulary (A19) would cover it without new text.
- The restriction "no policy may claim non-ANSI support" has no field to bite
  on in v1 (`format`/`version`/`id`/`transforms` carry no capability entry), so
  it is a claim discipline for ADR-016's shipped policy data, not a grammar
  rule — worth stating where its owner is if it stays load-bearing.
- Checklist item 2 (research URL spot-check): the diff introduces no new
  external source; A23's prior-art basis is `01-research.md`, which the caller
  excluded from this pass's inputs, so it was not re-walked here.
