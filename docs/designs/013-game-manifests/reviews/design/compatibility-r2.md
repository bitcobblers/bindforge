# compatibility review r2

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r2.diff` (9 hunks), read
against `02-proposal.md`, `docs/adrs/013-game-manifests.md`, and the Revision
log. Previous compatibility findings: none (the editorial phase runs one
language pass per round).

All nine hunks are language-only. Each post-image was confirmed present in the
proposal as it now stands, so no pre-image wording survives anywhere to
contradict a changed section:

- Goals → row encoding: "A row encoding that is unambiguous inside the
  one-line string-array grammar, even when a game-native name contains spaces"
  — same claim, moved modifier. The encoding it points at (split at the first
  space) is untouched.
- Options → action row: `, and` → `;` before the contract-posture clause;
  "honored" → "honoured". The refusal of a third axis-kind column is unchanged,
  and `ActionRegistry::axis_kind` remains the single source (A7).
- Options → manifest identity: "or a lowercase slug" → "or use a lowercase
  slug" — no change to the slug↔directory-name equality that carries the
  Windows case-collision posture.
- Example note: comma dropped before "and Deferred" — the deferral target is
  unchanged and still matches the ADR's deferral of preset/vocabulary growth.
- Load-time validation → Registry join: "must be `registry.contains`" →
  "must pass `registry.contains`; one that does not is refused, naming the id"
  — grammatically clearer, same `contains` check, same refusal shape.
- What the series reads: "in any `exclusive_groups` member" → "as any
  `exclusive_groups` member" — the set-membership lookup ADR-015 needs is
  unchanged.
- Test strategy #4: "files a ... as a refusal" → "reports a ... as a refusal"
  — still the `Result<Manifest, CoreError>` refusal named in #4's own text.
- Test strategy #7: "behavior" → "behaviour"; Rollout/example note spacing.
  The pinned statement that pre-existing refusals stay byte-unchanged,
  including the `is_slug_name` rename, still holds.

Checklist walk (all seven evaluable from the named inputs; none regressed by
the diff):

1. Compatibility matrix — every row carries a status; no `unknown`, no empty
   cell (Linux CI yes, Windows yes, macOS no (v1), toolchain floor yes,
   pre-013 core yes-refuses, non-Rust yes-data-only, product artifacts n/a,
   Steam Deck no (v1)).
2. OS input APIs — the design reads no input API; the abstraction is justified
   in the matrix rows ("nothing here adds a macOS-specific requirement" /
   "nothing here adds platform-specific work") and the capture side belongs to
   ADR-017, exactly as the ADR states.
3. Language floor — Rust, not Node; the floor row (1.85.0, edition 2024) is
   matched by std-only code, no new dependency, `Cargo.lock` byte-unchanged;
   nothing used sits above the floor.
4. Native modules — none; pure std Rust, so no prebuild/node-gyp/CI-matrix
   story is owed and none is implied.
5. Keyboard reality — native names are octet-capped (128) and may hold spaces,
   `#`, and any glyph the closed escape table decodes, which is what A13's
   localized/multi-word names need; the closed input vocabulary's inability to
   name `Tab`/`Esc` is a named Deferred item riding the 012 vocabulary-major
   mechanism, consistent with the ADR's Decision, so it is not re-opened here.
6. File paths — `game_dir.join("manifest.toml")`, no string concatenation;
   lowercase-only slug grammar plus byte-equal `id`/directory rule is what
   makes case collisions visible on a case-insensitive filesystem; per-OS
   config-directory conventions are ADR-017's, matching the ADR.
7. Platform tests — a leg is named for every row the matrix calls supported:
   Test strategy #8 names the Linux CI gate leg and the Windows contributor
   machine leg (also Rollout 7); the no-(v1) rows need none.

Revision log: both r1 entries (budget extended to `presets`, caps tied to their
baselines) are reflected in the current Budgets text; neither was rejected, and
nothing in this diff re-opens either.

## Findings
- none

## FYI (no severity)
- The two Britishisms the diff introduces ("honoured", "behaviour") now match
  the ADR's own "honour" and the proposal's remaining instances (lines 120,
  209, 230), so spelling is consistent document-wide; `serialize`/`re-serialize`
  stay American because they name code identifiers. Editorial seat's call, not
  a compatibility concern.
