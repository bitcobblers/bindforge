# complexity review r1

verdict: approve

## Findings

None. The design is complexity-clean against the checklist; the walk below
records how each item was evaluated so a later round can see it was not
skipped.

## Checklist walk

1. **Reuse before new package/module/interface.** Checked `tools/` — it holds
   only `tools/orchestration` (the Node/TS design-pipeline tooling, nothing
   transform-related). Checked `crates/core`: the design reuses rather than
   re-invents — `placement_is_legal`/`GradeBound`/`PlacementCheck` as the
   precondition gate (verified public in `crates/core/src/placement.rs:98-127`),
   `KeyboardGeometry::coordinate` for the grid lookups the rook-adjacency check
   needs (public, `placement.rs:62-64` — the check itself is ~10 lines of
   arithmetic over that lookup, proportionate to leave it in `transforms` and
   out of core's frozen public surface), `is_slug_name` for the loader
   (`pub(crate)`, same crate — no new slug helper), the shipped pair encoding,
   `check_dir_version`, the `generated-by` label grammar, and 014's
   pin-equality rule "cross-referenced rather than restated." The new
   `crates/transforms` crate is ADR-015's own Decision ("`crates/transforms`
   implements the stack"), not invented here.
2. **Every moving part traces to a Goal.** Walked the delta inventory: the
   `Document::Policy`/parse/serialize arms trace to the corpus-and-walk goal
   (A8); `load_policy` to the spec's load rules; `apply`/`Applied` to the
   stack goal; `TransformReport`/`NotApplicableReason` to the "per-row verdict
   reports" goal; the two budget constants to parse-level refusals that have
   fixtures (`transforms-past-cap.toml`, `relocate-target-past-cap.toml`). The
   three transform kinds are exactly the ADR's three (A1). Nothing orphaned.
3. **Design 30–50% smaller?** No defensible cut. Dropping prefix tags breaks
   the ADR's own example (`relocate-cluster(move.steer-*, …)`); folding apply
   into core contradicts the ADR's named crate; a precondition-list grammar
   would be *bigger* and the design already refused it in Options ("Precondi-
   tions: a separate author-declared list, or carried by the kind"); a new
   `CoreError` family for verdicts would be bigger and is refused (A9). The
   byte-order pairing of match set to targets looks like a candidate for
   explicit `id=input` pairs, but that collapses relocate to exact ids only —
   it loses the ADR-exemplified prefix form — so the chosen rule is the
   cheaper one and the surprise risk it creates is already enumerated and
   fixture-pinned in Risks.
4. **New dependencies.** None external: `Cargo.lock` "gains exactly one pack-
   age" is the workspace's own `crates/transforms`; the toolchain row states
   std-only. Verified the workspace `crates/*` glob posture in A16.
5. **Defer the second half?** The natural split (grammar+corpus first, stack
   second) is tempting but a grammar-only merged slice would leave a "supported"
   format with no runtime implementation, against the repo's precedent that a
   supported format ships its behavior; and deferring apply past this run would
   drop the run's own Goal ("`crates/transforms` … the stack"). The Rollout is
   already staged (spec/corpus → model/loader → crate → tests), so the first
   *merged slice* stays reviewable. Answer: no, and not worth a finding.
6. **Knobs without consumers / abstraction with one implementation.** The two
   caps have parse-level consumers and pinned fixtures. The report abstraction
   is one struct set, not a layer stack: no traits, no plugin seam — kinds are
   a compiled closed enum, matching the Options choice. `MAX_*` constants and
   `spec.md`'s composite budget mirror the shipped nine, 014's posture (A13).
7. **Value check.** Proportionate. The complexity is nearly all *contract
   surface* (one small row grammar, a closed ten-spelling reason vocabulary,
   two caps) sitting on mechanisms that already ship; the genuinely new logic
   is the match rule, the rook-adjacency contiguity check, and the untouchable
   guard — each traceable to a clause of ADR-015's Decision ("reports and
   no-ops; never mangles", "hard-docked or exclusive-group key is never a
   target"). The document repeatedly chose the smaller option (preconditions
   carried by the kind; no aggregate cap; no new error variants; materializing
   through 014's pin equality "not a new machinery"). Nothing reads YAGNI.

## Revision log

Only the r0 entry exists — no rejected findings to respect, and nothing in
this pass is re-raised.

## Deferred / out of scope

Checked against the ADR: ADR-015's "Not decided here" list names exactly
ADR-016 (seed set), ADR-017 (CLI), ADR-018 (hub/voting); the proposal's
deferrals match and none contradicts the ADR's Decision or Consequences.

## FYI (no severity)

- The interaction of "a translation can walk onto its own vacated keys" with
  `placement_is_legal`'s clause (a) distinctness check is subtle and lands in
  Test strategy #4 ("every failure leaves the working layout byte-equal"
  pins atomicity but not that clause); the correctness seat may want to
  confirm the ordering of those two checks is pinned — outside my specialty.
