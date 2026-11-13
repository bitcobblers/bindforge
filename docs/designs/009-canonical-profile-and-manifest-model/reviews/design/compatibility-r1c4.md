# compatibility review r1 (ratification pass c4)

verdict: approve

Scope: the blob diff `7b2ddc6 → f752153` of `02-proposal.md`, then the full current
document, read against `docs/adrs/009-canonical-profile-and-manifest-model.md` and
the compatibility checklist. Inputs kept to the named paths; no review report or
transcript opened.

## Closure of my r1 findings (checked first)

- **#13 → #14 matrix cell (minor)** — closed. The Windows row now reads
  "exercised leg (Test strategy #14)", and #14 is the platform leg
  ("green on the Linux CI runner and on a Windows contributor machine"), which is
  what Rollout 7 cites. Checklist item 7 now holds for every row the matrix claims
  as supported: Linux (#14 + the cargo gate), Windows (#14), Rust 1.85.0 (the gate
  adds no dependency), US-ANSI geometry (#3 grades the ANSI grid).
- **README keyboard-list / Test #3 dangling reference (minor)** — closed. Input
  vocabulary now names the list as "the admitted `keyboard:` vocabulary the reach
  tables draw from, enforced by the reach loader (a key outside it is
  `InvalidValue`), and carrying no grading obligation of its own", the README row
  drops the stale parenthetical, and Test #3 both iterates the content-derived
  graded set and pins the list ("a reach-table key outside the admitted keyboard
  list is `InvalidValue`"). The list has an enforcing check again and the US-ANSI
  row's evidence trail points at text that matches it.
- **"Each pinned by a fixture" (minor)** — closed, and closed the way the fix
  asked: fixture-pinned only where `parse` can see the condition, and every other
  variant named to the suite that calls its producer (#1 synthetic walk, #3, #4,
  #6, #8, #9, `MergeConflict` to the ADR-012 slice). I re-walked each pin against
  the Test strategy as it now stands: all 13 variants are accounted for and each
  cited test states that condition. The Risks row's "plus per-condition invalid
  fixtures" and the authoring bar's "one invalid fixture per rejected condition"
  were removed with the claim, and the authoring bar now routes rejected reach
  conditions to #3 — consistent with Test #1, which asserts fixture cases only.
- **`default` row's contract citation (nit)** — closed: "the emphasis fact, §1",
  which is the section carrying the canonical emphasis statement. "no prefix rule
  (Risks)" was dropped with the deleted Risks row.
- **Case-unique stem rule coverage (nit)** — not addressed, and the revision
  widened the sentence it rests on; re-raised below as a nit.

I also checked the reach loader's newly restated "the file's name agrees with its
in-file `hand`": it is shipped behavior (`crates/core/src/reach.rs:103-131`
refuses it as `InvalidValue { key: "hand" }`, naming the file name, no path) and
is pinned by the shipped per-hand suite that Test #3 preserves, so it is not an
unpinned new rejection — no finding.

Nothing in the diff breaks the ADR. `seeded-from` requiring `for` is consistent
with ADR Decision 3/5 (a built-in preset belongs to a game); the closed
`policy|preset|generator` producer list covers the ADR's reserved
`generated-by` and its AI-generation case; deferring "honoring a manifest's
`docked` actions" to ADR-012 is consistent with ADR Decision 5, which requires
the manifest to record hard-docked constraints, not that v1 enforce them at apply
time; the personal-policy-location and config-path deferrals match the ADR's
"Not decided here" list.

## Findings

- [nit][re-raised] Compatibility matrix — Windows row, "tree file names follow the
  shipped case-unique stem rule" — the shipped case-uniqueness assertion runs over
  `fixtures/{valid,invalid}` names only
  (`crates/core/tests/conformance.rs:277-311`, "fixtures/{kind} of {} holds names
  that differ only by case"), and `schemas/README.md:26-27` scopes the rule to
  fixture file names. The revised §3 sentence now states the extension rule for
  "the `data/` manifest, placement, and policy documents" too, so the row's
  wording claims case-uniqueness coverage over trees no shipped test enforces.
  Substantively harmless — ids never become path components, which is the design's
  own reason — so this stays a nit. Fix: in that one cell, scope the claim
  ("fixture names follow the shipped case-unique stem rule; `data/` file names
  follow the stem rule as hygiene, not a load-bearing check").
- [nit][new] Chosen design → Adversarial posture (last sentence) and Files created
  → `schemas/README.md` row — "`schemas/README.md`'s guard section gains those two
  constants and their rejections beside the shipped three, so its 'public names
  for these limits' claim stays true" — the README's canonical sentence is "The
  three constants are the reference implementation's public names for these
  limits" (`schemas/README.md:64-65`); adding `MAX_INDEX_ENTRIES` /
  `MAX_INDEX_DEPTH` to that section leaves "the three constants" naming three of
  five, so the claim stays true only if that sentence is amended, which the
  design never says happens. Fix: name the edit — Rollout 1 and the Files created
  cell record that the guard section's "three constants" sentence becomes a
  five-constant sentence.
- [nit][new] Chosen design → Profiles are chains → Resolution, API bullet — "`for`
  is required at parse and a `for` naming no manifest document in the index is
  `UnresolvedAncestor`" — as written this reads as making `for` a required entry,
  which contradicts the same section's "`profile/v1` entry set grows by
  **optional** entries only, so the shipped valid fixtures stay valid (no
  narrowing)", its "absent = a profile not bound to a game" (the ADR's
  accessibility case), and Rollout's Defaults line "`for` absent = not bound to a
  game". The rule being added is the parse-time one from §3's entry list. Fix:
  write "`seeded-from` requires `for` at parse" (or "`for`'s grammar is checked at
  parse"), leaving `for` optional.

## FYI (no severity)

- Test strategy #10's "the registry's names, arity, and argument grammars equal
  the `policy/v1/spec.md` invocation table" is not mechanically checkable against
  a Markdown table; the testable shape is registry == a pinned expected table in
  the test, with `spec.md` agreement carried by review. Test-strategy seat.
- The graded set is content-derived (`data/placements/` + the seed manifest's
  `default` rows) while the tables it grades live in the contract tree, so a
  `data/` content PR can now fail a `schemas/`-tree test (#3). That coupling is
  the r1/r2 decision and I do not re-open it, but `reach-classes/v1/spec.md`
  should state it so a data author is not surprised. Docs seat.
