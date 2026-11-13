# Compatibility review r2

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`) against `02-proposal.md`.
Inputs: the diff, `02-proposal.md`, `docs/adrs/009-canonical-profile-and-manifest-model.md`, the Revision log. No prior report opened.

verdict: approve

## Findings

None. Every diff hunk was checked for compatibility effect; each is language-only, and the
two changes that touch compatibility-bearing text resolve correctly:

- **Cross-reference renames (`§N` → `contract N`)** — Encoding rules, Manifests, Adversarial
  posture, Risks. `Context` names the five contracts in the ADR's order, and `Chosen design`
  §1–§5 follow the same order, so each new citation lands on the text it cites: "the stem rule
  (contract 3)" → §3 *Id → file, never id → path*, which carries the stem rule and the
  "no path is ever built from a document string" rule; "`apply_policy`'s `Err` (contract 4)" →
  §4 Transforms; "`MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH` (contract 3, A25)" → §3;
  "the authoring bar and its CI checks (contract 2)" → §2; "The canonical drift mechanism
  (contract 5)" → §5. No reference became dangling, and the rename removed the only
  line-number-shaped anchor in the document.
- **Game-id grammar, Oxford comma** — Encoding rules: "which excludes `..`, `/`, `\`, and a
  leading `.`". The four-item enumeration is now unambiguous, so the claim that the grammar —
  not a path rule — gates a pin's halves is clearer, not changed.
- **Keyboard-name list gloss** — Input vocabulary: "the admitted keyboard-name list: the
  `keyboard:` vocabulary the reach tables draw from, enforced by the reach loader (a key
  outside it is `InvalidValue`)". Dropping the second "admitted" matches the glosses in Files
  created and Rollout 1 and keeps the loader-enforced closed set intact; the graded set stays
  content-derived, so no full-keyboard grading obligation returns.
- **Authoring bar clause** — "every rejected condition is pinned by Test strategy #3's model
  tests": finite-verb form of the same claim; the grading-side rejections it names
  (`GeometryConflict`, admitted-list `InvalidValue`) are in Test strategy #3, placement
  rejections stay with #4, as the following sentence says.
- **Test strategy #14 punctuation, Rollout 1 "valid/invalid", Deferred/Options comma edits,
  Revision log "Test #N" → "Test strategy #N"** — no platform, dependency, keyboard, or path
  claim moved.

## Checklist (compatibility)

1. **Matrix rows** — 11 rows, all with a status; no `unknown`, no empty cell. The hub row's
   `no (v1) — out of scope` matches Deferred (ADR-014) and the ADR's "Not decided here: the
   hub service and voting". Untouched by the diff.
2. **OS input APIs (X11/Wayland/Raw Input/HID)** — N/A by design and justified: the ADR names
   inputs "OS-neutrally", the Non-goals rule out "Any input-event API (injection, capture)",
   and the matrix row "A game's own config files (any OS, Proton prefix) | n/a" states the
   crate "reads and writes only `schemas/` and `data/` documents". No platform API to target.
3. **Version floor** — Node is not this repo; the floor is Rust 1.85.0 pinned (A9, matrix
   row). The diff adds no API surface, so nothing above the floor.
4. **Native modules** — none. Core stays depends-on-nothing, transforms depends on the path
   crate only, Rollout 2 states "No new dependency". The diff adds none.
5. **Keyboard reality** — non-US symbol layouts, ISO/JIS, `pos:` naming all stated `no (v1)`
   with the v2 path; input and taxonomy ids are lowercase ASCII, so non-ASCII binding names
   cannot arise, while game-native action names ride in `action` rows as string values.
   Consistent with ADR Decision 2. Unchanged by the diff.
6. **File paths** — grammar-gated ids, walked index keyed on declared `id` / `game` +
   `revision`, "never by joining a pin into a path", `DirEntry::file_type()` with no symlink
   following, case-unique lowercase stem rule; per-OS config dir is Deferred and matches the
   ADR's "where a user's personal policy document lives". The diff's rename points at this
   text correctly.
7. **Platform test per supported platform** — matrix supports Linux x86-64 and Windows
   10/11; Test strategy #14 names both, Rollout 7 gates the announcement on the Windows leg.
   macOS is `no (v1)`, so no leg is owed.

## FYI (no severity)

- End of document — new line "Editor notes: Chosen design → Profiles are chains — \"`for` is
  required at parse\" is ambiguous; author to clarify." This is an editor's open query left in
  the shipped text, so the document ends carrying an acknowledged ambiguity. The ambiguity it
  points at is real: §3 Profiles are chains says `for` is an **optional** entry with "absent =
  a profile not bound to a game" (and Defaults repeats it), while the `resolve` bullet states
  "`for` is required at parse" — the intended reading, per the Revision log's `seeded-from`
  entry, is "required when `seeded-from` is present". Language/clarity fix for the editorial
  seat: resolve the query (scope the requirement to the `seeded-from` case) and delete the
  note line. Not a compatibility finding.
- "contract" now carries two senses: the numbered ADR contracts and "the contract tree" /
  "the `patch/v1/spec.md` contract" (Files created, Revision log). "(contract 3, A25)" sits
  two lines under the Files-created table's "contract tree" cell; a reader could read the
  citation as a tree reference. Editorial-seat preference.
- Two citations for one fact: §5's "`default` = `[id, input]` (the emphasis fact, contract 1)"
  cites the taxonomy section, while §1 records the same row as "a manifest `default` row
  (contract 5)". Both readings are defensible; picking one direction per citation would read
  cleaner. Editorial-seat preference.
