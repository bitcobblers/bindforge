# complexity review r2

verdict: approve

Scope: `docs/designs/013-game-manifests/reviews/design/editorial-r2.diff` (9 hunks),
read against `02-proposal.md`, `docs/adrs/013-game-manifests.md`, and the
proposal's Revision log. Inputs are those four only; no other review report or
transcript was opened.

All nine hunks are language-only: word order in the Goals bullet ("A row
encoding that is unambiguous inside the one-line string-array grammar"), a
semicolon where a conjunction ran on ("the drift channel the research flags;
the contract's own"), `honored` → `honoured`, `behavior` → `behaviour`, the
parallel "or use a lowercase slug", the comma dropped in "see Registry growth
order and Deferred", `must be registry.contains` → `must pass
registry.contains`, "in any `exclusive_groups` member" → "as any", and `files
a` → `reports a`. The post-images match the current proposal text at every
hunk. Nothing in the diff adds, removes, renames, or re-bounds a mechanism:
the entry table, the four budget constants, the four new structs, the loader
signature, the fixture list, the Deferred list and the ADR's five fact groups
are untouched. Nothing the diff rewrote contradicts the ADR's Decision, and
the two r1 Revision-log entries (budget extended to `presets`, caps tied to
baselines) are not reopened by this pass.

## Findings

None.

## Checklist coverage

1. **Reuse.** The diff introduces no new moving part. Across the design the
   reuse is real, and the diff leaves it intact: the slug grammar reuses the
   shipped `placement::is_geometry_name` (confirmed: defined at
   `crates/core/src/placement.rs:72`, called at `placement.rs:90,137` and
   `document.rs:586`) rather than adding a second name validator; `tools/`
   holds only the TypeScript orchestration pipeline, nothing reusable here.
2. **Moving parts trace to a Goal.** Unchanged by this diff; the entries the
   wording touched (`actions` rows, `exclusive-groups`, `docked-actions`,
   `presets`) each map to an ADR-013 fact group and a stated Goal.
3. **30–50% smaller design.** Not available for the cost: one format
   directory, one `Document` arm, one module and four exported caps is already
   the minimum that satisfies the ADR's five fact groups plus the harness's
   round-trip and zero-fixture rules. Cutting the optional `fingerprint` or
   the budgets would strand an ADR clause or a research requirement, not
   complexity.
4. **New dependencies.** None; std-only, `Cargo.toml`/`Cargo.lock` byte-unchanged. The diff touches neither.
5. **Defer the second half.** Already done (first game manifest = Rollout 6; CI matrix, preset contents, codec, vocabulary major all Deferred). The diff does not move anything back into scope.
6. **Knobs with no consumer; one-implementation abstractions.** No new knob
   appears in the diff. The four caps are parse-level refusals with named
   baselines, and each of the four structs is read by a stated consumer
   (`Manifest`/`ManifestAction` by 014's join, the constraint sets by 015's
   precondition lookup). The one entry nothing in this run consumes is
   `fingerprint` — it is mandated verbatim by ADR-013 ("a fingerprint of the
   config it was read from"), grammar-only, and its semantics are deferred to
   ADR-017 consistent with that ADR's own reservation, so it is not YAGNI.
7. **Value check.** Proportionate: the whole feature is one schema directory,
   one module, four constants and four flat structs on machinery that already
   ships, with no new error variant, no new dependency and no CI change. The
   editorial pass changed none of that.

## FYI (no severity)

- Delta inventory — "`crates/core/src/placement.rs` and both `is_geometry_name`
  call sites" counts two, while Rollout 2 says "its three call sites" and the
  tree has three (`placement.rs:90,137`, `document.rs:586`). Outside this
  diff's changed lines; correctness seat's, not complexity's.
