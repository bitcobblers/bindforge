# compatibility review r1c1 (dissent pass)

verdict: approve

Scope: `git diff fc0a1f9..0dacb96` of `02-proposal.md` (Risks ANSI-US row, new
Compatibility matrix → Keyboard geometry row, Product artifacts row note,
Deferred → ISO/JIS entry, Revision log), then the full current document.
Inputs: `02-proposal.md`, `docs/adrs/015-transforms-as-preconditioned-policies.md`,
the Revision log, my r1 findings.

## Findings

No open findings. No new findings — nothing in the changed text is false, and
nothing it asserts contradicts the ADR.

## Closures (my r1 findings, verified against the changed text)

- [major][closed] Compatibility matrix — the requested row landed verbatim in
  substance: `| Keyboard geometry | yes (ansi-us only) | relocate-cluster's
  contiguity and legality read the loaded grid ... exactly one geometry ships
  (A20) — ISO/JIS is new table files, not logic, and no policy may claim
  non-ANSI support until they land`. The Risks row now points at it ("the
  Compatibility matrix's Keyboard geometry row states what no v1 policy may
  claim"), closing the dangling cross-reference. Verified against the stack:
  the contiguity check reads "the loaded grid" and legality ends in
  `placement_is_legal(from, to, pairs, GradeBound(bound), tables, geometry)`,
  so the single-grid property is a real layer-level fact. Deferred → ISO/JIS
  now cross-references the row instead of restating it; no contradiction with
  ADR-015's Decision (which says nothing about geometries; A20 is a shipped-data
  fact).
- [minor][closed] IME/composition — handled in the one-clause form I offered:
  "IME/composition is out of scope for v1: `apply` and `load_policy` see only
  action ids and closed-vocabulary physical input ids (A19), never a composed
  character". Accurate: the row grammar admits only `<tag>` ids and
  `InputId::parse`-checked inputs; no text-entry input is modeled anywhere in
  the contract. With the geometry row, the untouchable guard's dead-keys clause,
  and the A15 slug grammar, all four sub-items of checklist 5 are now visible
  in the document.
- [nit][closed] OS input APIs — the Product artifacts row now names all four
  checklist-2 APIs and justifies the abstraction: "touches no OS input API
  (X11, Wayland, Windows Raw Input, macOS HID): parsed documents in, parsed
  documents out". Checklist item 2 satisfied by justification, as allowed.

## Checklist re-walk (against the changed sections)

1. Matrix statuses: every row non-empty; "yes (ansi-us only)" is the same
   qualified-yes spelling as the pre-existing "yes (refuses cleanly)". Pass.
2. OS input APIs: named + justified (above). Closed.
3. Version floor: Rust project — the Toolchain floor row (1.85.0, edition
   2024, std-only) is the analogue; no new text uses anything above it. Pass.
4. Native modules: none — std-only, `Cargo.lock` gains exactly one package. Pass.
5. Keyboard reality: all four sub-items now visible (above). Pass.
6. File paths: untouched by this diff; `load_policy`'s slug-before-filesystem
   rule stands. Pass.
7. Platform tests: matrix claims Linux CI and Windows; Test strategy #11 names
   both legs. The Keyboard geometry row's claim (ansi-us) is exercised by the
   relocate-cluster tests running against the shipped tables/geometry. Pass.

## FYI (no severity)

- None.
