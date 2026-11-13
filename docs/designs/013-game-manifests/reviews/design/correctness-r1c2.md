# correctness review r1 (ratification pass c2)

verdict: approve

Scope: the diff `bbebe40..fe5671d` of `02-proposal.md`, plus the full current
document, checked against `docs/adrs/013-game-manifests.md`, the Revision log,
and this seat's round-1 findings. Only text changed since the last pass was
examined for regressions.

## What the revision did

Both r1 entries land in Chosen design → Rules → **Budgets**: `MAX_MANIFEST_ROWS`
is widened from three arrays to four (`actions`, `dead-keys`, `docked-actions`,
`presets`), no new constant, with a new invalid fixture
(`fixtures/invalid/presets-past-cap.toml`), a new Test strategy #4 case (a 257th
preset id), a reworded Goals bullet, and a reworded Risks mitigation row. The
change is internally consistent: the constant count stays four (matching
`lib.rs`'s "the four budget constants" and Rollout 4's "the four new
constants"), the entry-table caps for `exclusive-groups` (2..=16 members, at
most 64 groups) still match `MAX_GROUP_MEMBERS`/`MAX_CONSTRAINT_GROUPS` and
Test strategy #2/#4, the new fixture refuses at `parse` alone as the conformance
posture requires, and "The four named budgets cap every array in the entry
table" is true once `exclusive-groups` is read as covered by the group caps. No
round-1 closure here breaks anything this seat approved, and none of the
sections this seat cited (Delta inventory, Identity/revision, the example,
Options → A19) was touched.

## Findings

- [minor][new] Chosen design → Rules → Budgets — "Each cap sits well above the
  baselines surveyed for it (40 registry ids, A7; TF2-scale bind lists and
  multi-word native names, A9, A13)" — the new justification claims all four
  caps and supports two. The register surveys a baseline for the row cap (A7,
  40 ids) and, loosely, for the native-name cap (A9), but no register row
  surveys a group-count or group-size baseline, so nothing backs "each cap" for
  `MAX_CONSTRAINT_GROUPS = 64` and `MAX_GROUP_MEMBERS = 16`; the only
  constraint-shape size the document does state, A8's closed vocabulary
  (48 keyboard + 7 mouse + 18 gamepad = 73 nameable input ids), sits *above* the
  64-group cap, so the sentence's own direction of comparison is not available
  for it. The "multi-word native names" half of the citation also rests on A13,
  which the register itself marks `unverified` (and the verified A9 survey names
  keybinds, not multi-word display names). Fix: scope the sentence to the caps
  that have a surveyed baseline — "The row and native-name caps sit well above
  the baselines surveyed for them (40 registry ids, A7; TF2-scale bind lists,
  A9); the two group caps are review ceilings over A8's 73-member closed
  vocabulary, not survey-derived." — or drop "each" and name which caps the
  citation covers.

- [nit][new] Chosen design → Rules → Budgets — "`MAX_MANIFEST_ROWS = 256`
  bounds each array-shaped entry (`actions`, `dead-keys`, `docked-actions`,
  `presets`)" — `exclusive-groups` is also an array-shaped entry in the entry
  table ("2..=16 members; at most 64 groups"), so the category phrase is
  contradicted by its own enumeration: the fifth array rides a different cap.
  Low blast radius (Test strategy #4 pins "a 65th group"), but this is the
  sentence `spec.md` inherits. Fix: "`MAX_MANIFEST_ROWS = 256` bounds each flat
  row array (`actions`, `dead-keys`, `docked-actions`, `presets`);
  `exclusive-groups` is bounded by `MAX_CONSTRAINT_GROUPS` and
  `MAX_GROUP_MEMBERS`."

## FYI (no severity)

- Closure record: the Revision log now carries only the two Budgets decisions.
  This seat's round-1 minors and nits — `placement.rs`'s predicate listed as both
  Edited and Unchanged; the loader credited with enforcing revision
  monotonicity; the load-time join's missing CI enforcement owner; `version` vs
  `revision` in the ADR's "id and version" pin; the A19 parenthetical in Options
  — are unchanged in the text and recorded neither as accepted nor as rejected,
  so a later reader cannot tell they were considered. Re-raising them is out of
  scope for a ratification pass; recording them in the log is not.
- Naming: `MAX_MANIFEST_ROWS` now spans row arrays and an id array (`presets`)
  under one exported contract name, and one shared 256 lets a document carry
  4 × 256 elements while the Risks row reads as a single-shape cap. Naming and
  constant granularity look like the complexity/compatibility seats' call.
- The 63-octet slug bound (Chosen design → Identity grammar) is a fifth limit
  that is not one of "the four budget constants" re-exported in `lib.rs` or named
  in `schemas/README.md` per Rollout 4. It is inherited from the action-id
  grammar rather than new, so the broadened Goal wording ("every new shape —
  each array and the native name") still holds as written, but the appositive is
  looser than the four-item list it replaced.
- Anti-hallucination spot-check: A9's basis (wiki.teamfortress.com/wiki/Scripting)
  is live and its "List of key names" section confirms `ESC`, `F1`–`F12`, `Tab`,
  arrows and `MOUSE1`–`MOUSE5`, and separately notes a 1 MiB config-file limit —
  supporting the budget posture. A13's basis
  (partner.steamgames.com/doc/features/steam_controller/iga_file) is live but its
  localization examples show single-word titles (`"move" "Move"`), so A13's
  `unverified` status in the register is accurate and should not be quoted as a
  surveyed baseline.
