# correctness review r2

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r2.diff` (single editorial
confirmation pass, round 2). Inputs per the caller: `02-proposal.md`,
`docs/adrs/013-game-manifests.md`, the Revision log, no prior findings from
this seat. This is a ratification pass: findings only if the revisions broke
approved text or made a cited section false.

## Findings

None. Every diff hunk was checked for technical-meaning drift and none
changes a decision, an interface, or a rule:

- **Goals / row encoding** ("A row encoding that is unambiguous inside the
  one-line string-array grammar, even when a game-native name contains
  spaces"): rephrasing only; still agrees with the Chosen design's
  split-at-first-space encoding.
- **Options — action row** (comma→semicolon, "honored"→"honoured"): the
  single-source/registry argument and the (A7) citation are untouched; the
  British spelling now matches the ADR's "honour" and the proposal's other
  "honour" uses.
- **Options — manifest identity** ("or use a lowercase slug equal to the data
  directory name"): inserting the verb keeps the option identical to the
  chosen slug id, byte-equal to the directory name (entry table `id` row,
  load step 2 location rule).
- **Example note** ("see Registry growth order and Deferred"): both
  cross-reference targets exist — the **Registry growth order** paragraph
  under Chosen design → Identity, and the Deferred bullet on widening the
  closed input vocabulary for `Tab`/`Esc`.
- **Load-time join** ("must pass `registry.contains`; one that does not is
  refused, naming the id") and **Test strategy #4** ("`load_manifest`
  reports a grammar-valid-but-unregistered action id as a refusal naming
  the id"): these two edits *harmonize* the load rule and its test wording;
  both still agree with Goals and with A7's `ActionRegistry::contains`.
- **What the series reads** ("as any `exclusive_groups` member"): same
  set-membership rule as the entry table and the ADR-015 quote; no rule
  change.
- **Test strategy #7** ("behaviour"): spelling only.

Checklist coverage for this pass:

1. Assumption register — untouched by the diff; the `unverified` rows (A13,
   A17) remain flagged as such and are not restated as fact by the changed
   text. The diff preserves every A-number citation on the edited lines
   (A5/A13, A7).
2. Research spot-check — four URLs fetched: wiki.teamfortress.com/wiki/Scripting
   (A9: key-name table incl. `ESC`, `F1`–`F12`, `Tab`, arrows, `MOUSE1`–`MOUSE5`;
   the `+moveleft`/`+moveright` null-cancel passage backs the action-level
   exclusivity example), w3.org/TR/SRI (A16: `hash-algorithm "-" base64-value`,
   closed strength-ordered token set, §5.1 unauthenticated-channel and §5.3
   small-content brute-force cautions — the fingerprint risk row's basis is
   accurate), partner.steamgames.com IGA doc and github.com/ngollan/Elite-Binds
   (A13 and the identity section's "name↔location mismatch is what a validator
   exists to catch" — the Elite-Binds README describes exactly a
   preset-name-matches-filename validator). All live, none contradicting.
3. Internal contradictions — none introduced; each edited sentence still
   matches its counterpart sections (entry table, load steps, test cases).
4. Edge cases — the diff changes no rule, so the edge-case posture
   (empty arrays, caps, collisions, case-insensitive filesystems) is
   unchanged from the approved r1 text.
5. Interfaces — names unchanged by the diff; `registry.contains` /
   `registry.axis_kind` still match A7's `ActionRegistry::contains` /
   `ActionRegistry::axis_kind` (call-site vs method-path style only).
6. Source-document coverage — the diff answers no ADR requirement afresh;
   the five fact groups and the pin halves remain decided as approved.
7. Prior-round closure — the Revision log's r1 major (extend
   `MAX_MANIFEST_ROWS` to `presets` + `presets-past-cap.toml` + Test
   strategy #4 case) is present for real in the body — the constant's text
   bounds `presets`, the fixture is listed, and the "257th preset id" case
   exists — and the r1 nit tying caps to surveyed baselines is present
   ("Each cap sits well above the baselines surveyed for it...").

Deferred check: every Deferred bullet (017 codec/fingerprint semantics, 014
pin grammar and re-validation matrix, 016 seed contents, vocabulary-major
keys, 018 transport, authoring tooling) matches the ADR's own "Not decided
here" list and costs framing; none contradicts the ADR's Decision or stated
requirements, so none is re-opened.

## FYI (no severity)

- Chosen design → "What the series reads from here" / Options — the ADR
  literally lists "+ axis kind" inside the action-table fact group, and the
  proposal records it by registry answer rather than as manifest data. The
  proposal's rebuttal (A7: axis kind is a property of the id, single source)
  is internally consistent and predates this diff, so it is not a
  correctness finding here; an ADR-fidelity seat may want to weigh whether
  "honoured by the *answer*, not a copy" satisfies the ADR's wording.
