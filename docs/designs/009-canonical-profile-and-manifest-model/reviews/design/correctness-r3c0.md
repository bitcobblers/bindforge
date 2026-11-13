# correctness review r3c0 (ratification pass over r4)

verdict: approve

Scope: the `f7521536 → 4d07da4d` diff of `02-proposal.md` plus the full current
document. Inputs: the proposal, `docs/adrs/009-canonical-profile-and-manifest-model.md`,
the Revision log, and my r3 findings. Checklist: `references/correctness.md`.

## What the r4 diff was checked for

- **Context "contracts 1–5, in the order listed"** — the listed order
  (taxonomy, input vocabulary + reach classes, profiles as chains, transform
  policies, manifests) matches both the Chosen design sections §1–§5 and
  ADR-009's own Decision section numbering, so every `contract N` cross-reference
  resolves: line 100 `contract 5` (manifest `default` row), 176 `contract 2`
  (`mouse:*` parses), 182 `contract 3` (stem rule), 201 `contract 1` (emphasis —
  the canonical statement is §1's "Emphasis is not in the taxonomy" paragraph),
  227 `contract 4` (`apply_policy`'s `Err`) and `contract 3, A25` (index limits),
  233 `contract 2` (authoring bar), 236 `contract 5` (drift), 237 `contract 3`
  (chain-depth unit). No `§N` reference survives (grep for `§`: no matches), so
  the rename left no mixed-reference style behind. Clean.
- **A16's new basis** — `https://en.wikipedia.org/wiki/Home_row` is live
  (redirects to *Touch typing*, `#Home_row`) and states verbatim what the row
  claims it states: "On the most common type of English language keyboard, the
  QWERTY layout, 'A S D F' and 'J K L ;' are the home keys on the home row."
  `docs/adrs/009…:28-31` is exactly the "community practice disagrees … IJKL and
  Returnal's built-in southpaw OKL; …" passage, and ADR-009:86-88 supplies
  "IJKL and OKL; are two legal placements of the WASD shape", which is the
  row's other half. The `verified` status is now supported. The r3 note that the
  previously cited article was unreachable is resolved, not papered over.
- **`resolve` / `seeded-from` (my r3 #3)** — closed for real, not reworded:
  line 143 makes `seeded-from` **require** `for` at parse (`InvalidValue`,
  pinned by Test strategy #6), so the per-document pairing my fix asked for is
  in the entry set; the `Some`-whenever claim and the "never runs against
  nothing" guarantee are relocated to Id → file, where their actual support
  (`UnresolvedAncestor` on a pin naming no manifest document) lives, and §3's
  sentence now points there instead of asserting it locally. The Revision log's
  reason for not taking my global `for`-required form (it narrowed the entry set
  and made every unbound profile unparsable — the ADR's accessibility case) is
  sound, so I do not re-raise the multi-pin wording.
- **Punctuation/wording edits** (serial comma in the id-shapes bullet,
  `valid/invalid` pair, `Platform —`, `;` in the Options bullets) — no change of
  meaning anywhere; the Adversarial posture's variant-to-test mapping is
  byte-identical to what r3 approved.

## Findings

- [minor][new] Chosen design → Manifests state game facts (Drift mechanism),
  with the same wording in Risks — "The `cargo test` gate resolves **every
  profile in `data/profiles/`** against the manifest revision **it pins**" /
  "The canonical drift mechanism (contract 5) resolves **every `data/profiles/`
  chain** against **its pinned revision** inside `cargo test`" — the r4
  narrowing made these two universals refer to nothing for part of their domain.
  r4 withdrew r3's "`for` is required at parse" and put `for` back as an
  optional entry (line 143: "`absent` = a profile not bound to a game, which is
  what serves the ADR's accessibility and alternative-controller case";
  Revision log: "a rule local to `seeded-from`, since `for` stays optional";
  Defaults: "`for` absent = not bound to a game"). Those two sentences were
  written while every profile pinned a manifest, and were not amended with the
  narrowing: a profile/patch in `data/profiles/` with no `for` pins no revision,
  so "the manifest revision it pins" has no referent for exactly the case r4
  says the model exists to serve. The gate's behaviour for an unbound profile
  (resolve + `validate` with `manifest: None`; the `seeded-from` check cannot
  arise, by line 143) is inferable from §3, so the mechanism is not
  under-specified in a blocking way — but the drift suite's scope is now set by
  inference rather than by the sentence that defines it, and Test strategy #12
  already carries the correct scope ("for every profile that pins it"), which
  makes §5 and the Risks row the only two unscoped statements left. Fix: scope
  both by the key that drives the lookup — §5: "resolves every profile in
  `data/profiles/` **that declares `for`** against the manifest revision it
  pins (a profile with no `for` is resolved and `validate`d with
  `manifest: None`, and cannot carry `seeded-from`)"; Risks: "resolves every
  `data/profiles/` chain **that pins `for`** against its pinned revision". No
  new mechanism; it replaces the universal quantifier with the one the gate
  actually reads.

## FYI (no severity)

- Authoring bar (r3 #1) — r4 touched only the verb ("every rejected condition
  **is** pinned by Test strategy #3's model tests"); the substance is unchanged,
  so the two halves of my r3 finding are neither fixed nor broken: the
  grid-distance-monotonicity condition in that CI-enforced list is still named
  by no Test strategy clause, and the bar governs `placement/v1/spec.md` too
  while Adversarial posture maps `PlacementNotIsometric`/`PlacementDegraded` to
  #4. Not a regression, so not in this verdict.
- Test strategy #4 and #8 (r3 #2) — "a non-isometric **fixture** is rejected
  `PlacementNotIsometric`" / "each with its own **fixture**" are untouched by
  r4; the loader-vs-`validate` distinction #3 carries ("in the loader, not in a
  fixture document") is still not carried across to the two `validate`-produced
  sites. Unchanged, so not in this verdict.
- Register spot-check outside the diff: A15's
  `https://pubmed.ncbi.nlm.nih.gov/25029899/` is behind a cookie wall for the
  fetcher, so its content was not readable this pass. It is not a changed row
  and its claim is a negative one ("no standard grades per-key, per-hand
  reach"), so nothing in the diff depends on reading it; A16's replacement
  citation — the one r4 changed — verified live.
