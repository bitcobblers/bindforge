# correctness review r1 (pass c0)

verdict: approve

Scope: whole document. Inputs used: `02-proposal.md` (incl. Revision log, r0),
`docs/adrs/015-transforms-as-preconditioned-policies.md`, and the code/contract
artifacts named as Basis columns inside the proposal's own assumption register
(action.rs, placement.rs, document.rs, chain_ops.rs, manifest.rs, input.rs,
error.rs, conformance.rs, registry.toml, specs, ADR-004, ADR-008). Revision log
carries no closed findings, so every finding here is labeled `new`.

Verification notes (checklist 1–2): every `verified` register row I spot-checked
is supported by its cited code — A2 (`action.rs:52-64` enum comments verbatim),
A3 (registry.toml: exactly 4 discrete-lateral + 1 continuous-lateral + 35 none
= 40), A4 (`placement_is_legal` signature and `PlacementCheck` fields verbatim,
hands-as-inputs and answer-why rationale at `placement.rs:16-25`), A5, A6
(manifest caps 2..=16/≤64 confirmed at `manifest.rs:367,388`; spec.md:9 "a
transform's preconditions read this document's constraints" verbatim), A7, A8
(`error.rs:3` counts policy among untrusted; `conformance.rs:378-385` walk
comment), A9, A10/A12 fixture existence, A11 (`supported_majors()` union
`["v1","v2"]` at `document.rs:187-189`), A14 (80-char `MESSAGE_VALUE_CHARS`),
A15, A16 (`Cargo.toml` `members = ["crates/*"]`, ADR-008 clause 2 names
`crates/transforms`), A18 (`is_label_char` permits `:`; 128-octet cap), A19
(mouse 7 members; 48 keyboard members = 26+10+11+space, no tab/esc/arrows),
A21 (`cluster` occurs only in `placement.rs` doc text; no rook-adjacency
computation shipped), A24 (taxonomy `home,thumb,stretch,far,off-reach` verbatim).
Checklist 2: the proposal itself cites no research URLs; A23 is honestly marked
`unverified`, so I spot-checked its most load-bearing external claim directly —
RFC 6902 §3 "Operations are applied sequentially in the order they appear in
the array" — true, and the "shipped patch-level default" half of the same
sentence is fixture-pinned in-repo (A10). The invalid-fixture grammar set
(`table-header`, `dotted-key`, `multi-line-array`, `float-value`, `hex-integer`)
matches the refusal set every shipped spec.md states, so the corpus is
internally consistent with the shipped subset.
Checklist 6: the design answers the ADR's open points (grammar, cluster-as-data,
output in a chain, stack order, report shape) and defers exactly the ADR's
"Not decided here" list (016/017/018) — no deferral contradicts the ADR.

## Findings

- [minor][new] Chosen design → The stack / The match rule + per-kind checks —
  "the produced `set` list preserves the source base's element order in place"
  and "`swap-mouse-buttons` — finds the actions currently bound to
  `mouse:left` and `mouse:right`" — the shipped layout model is
  action→input (`chain_ops.rs:167` `BTreeMap<ActionId, InputId>`), and
  `profile/v2` never refuses one input carrying two actions: only the action
  side is a function, while reach tables and geometry *do* explicitly refuse
  repeated inputs (`document.rs:1231,1276`). Duplicate set elements are also
  legal data ("Duplicates are data", `profile/v2/spec.md:81`). So on a
  grammar-valid — and by the design's own untrusted-input posture, reachable —
  source base, "the action currently bound to `mouse:left`", "a target already
  bound to an action outside the match set", and "element order in place" for a
  duplicated action id are each underdetermined, which is the one corner where
  "same inputs ⇒ byte-identical output" and the closed reason vocabulary are
  not fully pinned. Fix: one sentence in The stack — apply first collapses the
  base's `set` to resolve's own action→input map (last element wins), and a
  source base with more than one action bound to one input is an entry refusal
  reusing `InvalidValue` naming the input (the A9 "could not evaluate"
  channel); pin each with one invalid-base test in `apply.rs`.

- [nit][new] Assumption register A17 — "ADR-004 binds a test tree to mirror
  `src/`: a module-named test file is that module's unit test", Basis
  `docs/adrs/004-mirror-src-hierarchy-in-tests.md:23-33` — the cited range is
  ADR-004's Context (the TS `pipeline-` prefix list), and ADR-004's title and
  Decision are scoped to "the orchestration test tree"; the naming rule the
  row states is at lines 49-53 ("Naming rule going forward… New subsystems
  follow the same shape"). The claim is true as repo practice — core already
  ships module-mirrored Rust tests (`tests/placement.rs`, `tests/manifest.rs`,
  `tests/chain_ops.rs`) — but the Basis citation does not contain it. Fix:
  cite `004:49-53` and add the shipped mirrored Rust test files as the
  practice anchor.

- [nit][new] Assumption register A13 — "the exported budget family is nine
  constants — … `MAX_CHAIN_DEPTH` 32, the four manifest caps, the two chain
  caps" — the nine exported `MAX_` constants are the two toml caps, the four
  manifest caps, `MAX_CHAIN_DEPTH`, and the two 014 profile caps
  (`MAX_PATCH_DELTA_ELEMENTS`, `MAX_GENERATED_BY_OCTETS` — `document.rs:165,170`);
  the enumeration both double-counts `MAX_CHAIN_DEPTH` and labels the 014
  profile caps as "chain caps", and the cited basis omits `document.rs`.
  The nine-count itself is confirmed. Fix: replace "the two chain caps" with
  "the two `profile/v2` caps from 014" and add `crates/core/src/document.rs`
  to the Basis.

## FYI (no severity)

- Chosen design → The output: the produced base copies `for` and sets
  `generated-by`, but a source base's `seeded-from` provenance is silently
  dropped (it is legal on bases and not in the produced-document recipe).
  A sentence either way would be completeness — consistency/compatibility's
  surface.
- A23 stays `unverified` in this seat because the run's `01-research.md` was
  not a permitted input; the two prior-art claims leaning on shipped behavior
  (RFC 6902 sequentiality, patch-order fixtures) both check out independently.
- `crates/transforms/tests/zz_api_probe.rs` names `apply`, `Applied`,
  `TransformReport`, `NotApplicableReason` but not `Outcome`/`NotApplicable`,
  which are equally new public names — whether the probe must cover every
  re-export is a test-strategy seat's call, not a contradiction.
