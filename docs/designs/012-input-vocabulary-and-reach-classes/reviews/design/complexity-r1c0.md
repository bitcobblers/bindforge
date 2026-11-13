# complexity review r1

verdict: approve

## Findings

None at any severity. Checklist walk:

1. **New packages/modules/interfaces vs existing reuse.** No new package: the
   delta inventory adds two modules (`input.rs`, `placement.rs`) to
   `crates/core` and two `schemas/*/v1|v2` data directories. `tools/` holds
   only the orchestration pipeline (Node prompts) — nothing reusable for an
   id grammar or geometry loader. The design reuses what exists: `parse_file`
   and the size budget, `quoted` escaping, the entry-set table pattern
   (`REACH_CLASSES_V2_ENTRIES` "beside the v1 list"), `hand_matches_file_name`
   mirrored by `geometry_matches_file_name`, the version gate, and the
   untouched `schemas/*/v*/` conformance walk. Fixture names duplicated across
   directories (`duplicate-input.toml` in both new corpora) are already an
   established pattern — `unknown-entry.toml` and `non-string-element.toml`
   recur across three existing format directories — so no harness churn.
2. **Each moving part traces to a stated Goal.** Verified individually: the
   v2 tables and id grammar (Goals 1–2, ADR Decisions One–Three), the
   geometry format (Goal 3, ADR Consequences verbatim "new table files, not
   new logic"), `placement.rs` and `GradeBound` (Goal 4, Decision Four
   clauses a and b), even the single `keys-entry.toml` v1 fixture (the
   Compatibility-matrix row "refuses `keys` in a v1 document naming the
   entry"). No orphan moving part found.
3. **Smaller design?** Every 30–50%-smaller variant contradicts the ADR's
   words: coordinates as Rust constants contradicts "adding a keyboard
   geometry is new table files, not new logic"; grading a key subset
   contradicts Decision Two's "both are filled in for ANSI-US geometry";
   skipping clause (a) contradicts Decision Four. Within the ADR's envelope
   the design repeatedly picks the smaller option (shared `classes` order
   over per-key grade numbers or Rust-side order; hand-level grade over a
   finger model; literal-glyph ids over a fifth naming scheme).
4. **New dependencies:** none — std-only, `Cargo.toml`/`Cargo.lock` in the
   Unchanged list; no in-repo alternative even needed.
5. **Deferred second half?** The geometry format plus predicate is the
   design's whole reason to exist per ADR Decision Four ("This ADR defines
   that predicate") — deferring it leaves the ADR unfulfilled, so it cannot
   be cut from this document without contradicting the source.
6. **Knobs with no consumer; one-implementation abstractions.**
   `keyboard-geometry/v1` ships one geometry, but the ADR itself names that
   seam as the extension point, so it is mandated, not speculative.
   `GradeBound` is not a config knob — it is the caller-supplied "declared
   bound" of clause (b). The canonical `classes` per-document rule *removes*
   cross-file loader logic rather than adding a layer.
7. **Value check:** the complexity looks proportionate. The bulky parts
   (96 authored grades, a 48-entry grid, ten invalid fixtures) are contract
   data the ADR explicitly requires as data, and the code delta is two
   small modules plus edits to existing arms.

## FYI (no severity)

- `input.rs` model section — `InputGrade` stores `class: String` while
  `ReachTable::grade` yields `Option<&ReachClass>`; whether the pair should
  hold the typed class is a typing/correctness call, not a complexity one.
- Test strategy #5 — "WASD→OKL" with delta `(7,0)` implies a fourth landing
  key (`keyboard:;`) the name "OKL" does not spell out; worth a wording
  check for the correctness seat.
