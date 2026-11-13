# compatibility review r1 (ratification pass c2)

verdict: approve

## Scope

Blob diff `bbebe40..fe5671d` of `02-proposal.md` plus the full current
document, checked only for breakage of what r1 approved and for falsification
of sections cited by the r1 compatibility findings. Inputs per the caller:
`02-proposal.md`, `docs/adrs/013-game-manifests.md`, the Revision log, and my
own r1 findings.

## Findings

None. No blocker or major is open from this pass.

## Checklist re-verification of changed text

1. **Matrix completeness** — unchanged by the diff; every row carries a
   status; no `unknown`, no empty cell.
2. **OS input APIs** — unchanged; the design ships no input code ("this stage
   ships no binary", matrix row), so no per-platform API targeting is owed
   here; posture as approved in r1.
3. **Version floor** — toolchain floor row (1.85.0, edition 2024) unchanged;
   budgets are plain exported constants, std-only; no API above the floor
   introduced by the revision.
4. **Native modules** — unchanged: no new dependency, `Cargo.lock`
   byte-unchanged; the added budgets are named constants, not native code.
5. **Keyboard reality** — the diff does not touch Row encoding, the name
   grammar, or the valid-fixture set, so my r1 minor finding ("a native name
   of 1–128 octets with no ASCII control character" — raw non-ASCII native
   names unpinned; no localized-name fixture; Test strategy #1 exercises only
   spaces, `#`, `"`, `\`) remains exactly as filed: open, minor, not broken or
   falsified by the revision. It is not re-counted as a new finding this pass.
6. **File paths** — `game_dir.join("manifest.toml")` and the location rule's
   lowercase-slug byte-equality (case-unique on case-insensitive filesystems)
   are untouched.
7. **Platform tests vs matrix** — matrix supports Linux CI and Windows
   contributor; Test strategy #8 names both legs; unchanged.

Changed-text consistency checks that pass:
- `MAX_MANIFEST_ROWS = 256` now bounds `actions`, `dead-keys`,
  `docked-actions`, and `presets`, so the risk table's new "The four named
  budgets cap every array in the entry table" is true (`exclusive-groups` is
  covered by `MAX_CONSTRAINT_GROUPS`/`MAX_GROUP_MEMBERS`); "the four budget
  constants" in the `lib.rs` re-export line and Rollout 4 remain correct.
- The `presets-past-cap.toml` fixture and the Test strategy #4 addition
  ("a 257th preset id ... refuse as `InvalidValue`") match the extended cap.
- Goals bullet "Named parse budgets for every new shape — each array and the
  native name" now matches the Budgets bullet's coverage.
- The example's replacement note ("`ui.inventory` is not yet in
  `registry.toml` ... see Registry growth order, and Deferred") points to the
  Registry growth order section, which existed in the previously approved
  text and keeps the example consistent with the same-PR registry rule; the
  `Tab` vocabulary consequence stays deferred exactly as the ADR allows.
- Deferred section vs ADR-013: the CI re-validation deferral still tracks the
  ADR's Decision (`for` pin is ADR-014's entry; the matrix is a listed series
  cost, not a v1 gate here). No contradiction to re-open.
- Revision log records both r1 dispositions (the `presets` budget closure as
  major, the baseline tie-in as nit); no finding in my seat was recorded as
  rejected, and none in changed text was re-introduced.

## FYI (no severity)

- Budgets bullet, new sentence — "Each cap sits well above the baselines
  surveyed for it (40 registry ids, A7; TF2-scale bind lists and multi-word
  native names, A9, A13)": the assumption register supports baselines for the
  row and name caps, but surveys no baseline for preset counts or group
  sizes, and A13 is marked unverified. Whether that sentence over-claims its
  evidence is a correctness/complexity question, not a compatibility one.
