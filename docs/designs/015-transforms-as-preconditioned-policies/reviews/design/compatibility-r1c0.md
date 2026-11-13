# Compatibility review r1

verdict: request-changes

Scope: whole document (c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/015-transforms-as-preconditioned-policies.md`, the Revision log
(r0: initial draft, nothing rejected yet). No prior compatibility findings.

## Findings

- [major][new] Compatibility matrix (and the Risks row that points at it) —
  the matrix never states the ANSI-US-only keyboard-geometry floor, but the
  Risks table promises exactly that: `"Policies are effectively ANSI-US-tested (A20) | High | Low–Med | The same floor 012 and 014 shipped; ISO/JIS legality needs new table files, not logic; stated in the Compatibility matrix"`.
  The matrix's eight rows are Linux CI runner, Windows 10/11 contributor,
  macOS contributor, toolchain floor, pre-015 core, existing formats,
  third-party reader, product artifacts — none of them carries the geometry
  fact, and the closest one (`"Existing formats, fixtures, loaders, resolve | yes (untouched)"`,
  which names `keyboard-geometry` only as byte-exact) is about not editing a
  format, not about which geometry a policy is valid on. This is checklist
  item 5 (non-QWERTY layouts) and it is not decoration: `relocate-cluster`'s
  whole gate is grid-dependent — `"the source keys not connected by rook-adjacency (|Δcol| + |Δrow| = 1) in the loaded grid"`
  plus `placement_is_legal(..., tables, geometry)` — so "a v1 policy is only
  validated against one grid" is a compatibility property of the layer, not a
  test note, and the document itself cites a matrix row that is not there.
  Fix: add one matrix row and leave the Risks cross-reference pointing at it —
  e.g. `| Keyboard geometry | yes (ansi-us only) | relocate-cluster's
  contiguity and legality read the loaded grid; exactly one geometry ships
  (A20). ISO/JIS is new table files, not logic, and no policy may claim
  non-ANSI support until they land |`. No new mechanism: this states the A20
  fact the `ISO/JIS grid tables` Deferred entry already carries.

- [minor][new] Deferred / out of scope + Compatibility matrix — IME is the one
  keyboard-reality sub-item (checklist item 5) the document never addresses in
  any section. Non-QWERTY is handled (A20, Risks, the ISO/JIS deferral), dead
  keys are handled by the untouchable guard (`"reads or writes an input
  appearing in docked-actions, dead-keys, or any exclusive-groups member"`),
  non-ASCII names are handled by the slug/tag grammars (`"slug grammar, ≤ 63
  octets (A15)"`, `<prefix>` `[a-z][a-z0-9-]*`) — but composition/IME appears
  nowhere, not even as an explicit non-goal, so a reader cannot tell whether
  it was considered. The honest answer is cheap and true: `apply` and
  `load_policy` see only action ids and closed-vocabulary physical input ids
  (A19), never a composed character. Fix: one clause on the new
  keyboard-geometry row above (or a Deferred line): `IME/composition — out of
  scope for v1: the closed keyboard: vocabulary names physical inputs (A19)
  and no text-entry input is modeled or matched`.

- [nit][new] Compatibility matrix — `"Product artifacts (CLI, desktop, hub) | not applicable | This stage ships no binary"`
  justifies the absence of shipped binaries, but the matrix nowhere names the
  OS input APIs the checklist asks about (X11, Wayland, Windows Raw Input,
  macOS HID) for the layer that *is* shipped — the core/transforms library —
  so its "no platform-specific input behaviour" position is implied rather
  than stated. Fix: extend that row's note (or the `Third-party reader, non-Rust`
  row) with `v1 touches no OS input API: the contract is parsed documents in,
  parsed documents out`.

## FYI (no severity)

- The row grammar states `"1..=MAX_RELOCATE_TARGETS, no repeats"`, yet the
  invalid-fixture list and Test strategy #1 pin no repeat-target refusal,
  while the contract asserts `"every grammar refusal has its fixture"`.
  Grammar-closure/correctness seat's call.
- The Options decision that preconditions are carried by the kind rather than
  declared per row sits against ADR-015's `"Each transform declares: ... preconditions"`;
  I read it as consistent with the ADR's chosen alternative (`"A fixed
  precondition vocabulary over loaded data keeps a policy reviewable as data"`)
  — the row still declares a precondition parameter (`<bound>`) — so it is not
  a finding here, but a seat reading ADR-fidelity should confirm that reading.
- Checklist items evaluated with no finding: every matrix row carries a
  status (no `unknown`, no empty cell); no Node floor applies — the analog
  floor row (`Toolchain floor (1.85.0, edition 2024)`) is std-only and names
  no above-floor API; no native modules (`crates/transforms` `"depends on
  bindforge-core and nothing else"`, `Cargo.lock` gains exactly one package);
  paths are slug-checked before any filesystem question and every on-disk name
  is a lowercase-ASCII slug with case-unique fixture names, so separators and
  case-insensitive filesystems are covered by a grammar, not concatenation;
  per-OS config-dir conventions (XDG/AppData) are not introduced here —
  `load_policy(dir, id)` takes the directory from the caller, and storage is
  deferred to ADR-017, which matches ADR-015's `"Not decided here: ... the CLI
  to author and apply policies (ADR-017)"`, so the deferral stands; Test
  strategy #11 names a leg for each platform the matrix claims supported
  (Linux CI runner, Windows contributor) while macOS is marked `no (v1)`, so
  no supported platform is untested; the deferrals to ADR-016/017/018, the
  AI-generated-bases note, and the closed-vocabulary widening all match the
  ADR's Decision and "Not decided here" list.
- `supported_majors()` being `"unchanged"` does not contradict ADR-015's
  consequence `"the policy format joins the conformance corpus and
  supported_majors"`: `format_majors("policy")` answers `[v1]` and the union
  is already `["v1","v2"]` (A11), so the format joins the gate and the
  invariant value is unaffected.
