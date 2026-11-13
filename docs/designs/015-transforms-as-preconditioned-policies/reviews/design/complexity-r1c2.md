# complexity review r1c2 (ratification pass)

verdict: approve

Scope: the diff `fc0a1f9..0dacb96` of `02-proposal.md`, then the full current
document. My r1 verdict held no findings, so the test here is only: did the
revisions break something I approved, or make a section I cited false? No.

The diff touches four places and adds no runtime surface:
- Risks → "Policies are effectively ANSI-US-tested (A20)": mitigation re-pointed
  at the new matrix row instead of restating "new table files, not logic".
- Compatibility matrix → new `Keyboard geometry` row (ansi-us only; ISO/JIS is
  table files not logic; IME/composition out of scope for v1).
- Compatibility matrix → `Product artifacts`: one sentence — what ships touches
  no OS input API.
- Deferred → `ISO/JIS grid tables` bullet shortened to a pointer to that row.
- Revision log rewritten to the two r1 entries.

## Checklist

1. **Reuse / new package, module, interface.** The diff introduces none. It adds
   no grammar entry, no knob, no type, no crate, no loader — the ANSI-US floor is
   closed by *documenting* the shipped single geometry (A20) rather than by
   adding a `geometry` entry to `policy/v1` or a geometry-selection parameter to
   `apply`. That is the complexity-correct closure, and it keeps the Chosen
   design's surface exactly as I approved it: `apply(policy, base, manifest,
   registry, tables, geometry)` unchanged, `KeyboardGeometry` still an input the
   caller supplies.
2. **Every moving part traces to a Goal.** No new moving part. The two new matrix
   rows describe behavior already implied by rows I approved: A19 (closed input
   vocabulary — "never a composed character" follows from ids, not characters)
   and A20 (one geometry ships). Neither row contradicts ADR-015, whose Decision
   says only that preconditions are checked against 012 legality and 013
   manifest constraints — it makes no geometry or IME claim.
3. **A design 30–50% smaller?** Unchanged by the diff; the slice is still
   grammar + loader + one crate, and the diff did not add code the document then
   has to test. Test strategy is untouched, so no test obligation was created by
   the prose.
4. **New dependencies.** None. The new `Product artifacts` sentence actively
   *narrows* the dependency story (no X11/Wayland/Raw Input/HID), consistent with
   the unchanged Toolchain row ("std-only", `Cargo.lock` gains exactly one
   package = the workspace crate) and Rollout 3.
5. **Could the second half be deferred so the first slice stays thin?** The diff
   strengthens this: ISO/JIS tables and any OS-level interaction stay behind
   ADR-016/017/018, and no v1 mechanism is added in anticipation of them.
6. **Knobs with no consumer; abstraction with one implementation.** Nothing added.
   The one sentence worth watching: "no policy may claim non-ANSI support until
   they land" — the v1 entry set (`format`, `version`, `id`, `transforms`) has no
   field where a policy *could* claim geometry, so there is no claim to validate
   and no consumer for a future gate. Keep it read as a review norm for ADR-016's
   data, not as a hook for a `geometry` entry in `policy/v2`; adding such an
   entry now, or in v1, would be the knob-with-no-consumer mistake.
7. **Value check.** Proportionate, and the diff improves it: three places used to
   carry the ANSI-only fact (A20, Risks, Deferred); now the matrix row is the
   single statement and Risks and Deferred both cite it. Less repetition, same
   coverage.

## Findings

None. No blocker, major, minor or nit. Nothing I approved was broken: the Goals,
the Options choices (closed kind set, kind-carried preconditions, declared hands,
materialized base, sequential stack, verdict-data report, per-structure caps),
the model, loader, stack, report, budget, Rollout and Test strategy are byte-for-
byte what r1 approved; the Revision log's two entries match the diff exactly.

## FYI (no severity)

- Compatibility matrix → Keyboard geometry — "IME/composition is out of scope for
  v1" is the document's only place where that exclusion appears: it is not in
  Non-goals and has no entry in Deferred / out of scope, where every other
  exclusion names an owner. The sentence is true and complexity-neutral (it adds
  no mechanism); the placement is a structure/consistency question, not a
  complexity one.
- Revision log — the r0 entry is gone and the log records only the two changes
  that landed. If any r1 finding was deliberately rejected, that decision is not
  indexed. Process/record question for the run's editor, not for this checklist.
