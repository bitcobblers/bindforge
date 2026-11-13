# Compatibility review r3 c0 (ratification)

verdict: approve

## Scope

Ratification pass. Read: the diff `f7521536a6bca93f4f9b4d4a4cfce3446567cdc7` →
`4d07da4d5c80e4400db6d7f78e4ac2a8879824ce` of `02-proposal.md`, then the full
current document, the ADR, and the Revision log. Checked first: the three open
compatibility nits from round 3, then only whether the r4 text broke anything
approved.

The r4 diff is: the Context's contract-ordering clause (enabling the `§N` →
`contract N` cross-reference normalization, applied everywhere — no `§`
reference survives in the document), A16's citation basis, the narrowing of the
`for`-required-at-parse rule to a `seeded-from`-local rule, and punctuation.
Nothing in the compatibility surface moved: the matrix rows, the Test strategy
platform leg, the toolchain/dependency rows, the encoding/grammar rules, and
the path rules are byte-identical to what round 3 approved.

## r3 finding closures

- **Closed** — Profiles are chains → Resolution, API bullet. The bullet now
  reads "the pin's index lookup, and the `UnresolvedAncestor` for a pin naming
  no manifest document, are canonical in Id → file", so it no longer reads as
  making `for` a required entry. Verified consistent end-to-end: §3's entry
  list keeps `for` optional with "absent = a profile not bound to a game" and
  puts the parse-time requirement on `seeded-from` (`InvalidValue`); Test
  strategy #6 asserts "a `seeded-from` with no `for` is `InvalidValue`";
  Rollout's Defaults keeps "`for` absent = not bound to a game"; and the
  never-run-against-nothing guarantee is carried where the rule lives, in
  Id → file ("`UnresolvedAncestor` fires on any document reference with no
  document — a `derives` entry, **or a `for` pin whose `game` + `revision`
  names no manifest document in the index**, which is what makes
  `Resolved.manifest` `Some` whenever the profile declares `for` and keeps the
  `seeded-from` check and the drift gate from running against nothing"). The
  Revision log records the narrowing and its reason. No compatibility
  narrowing of the entry set remains.
- **Open, unchanged (nit)** — Compatibility matrix Windows row; see F1. The
  row's text is untouched by the diff, and §3's stem-rule sentence is the same
  one round 3 assessed, so nothing newly broke.
- **Open, unchanged (nit)** — `schemas/README.md` guard section; see F2. The
  Adversarial posture constant sentence, the Files created row, and Rollout 1
  are unchanged in substance (only `§4` → `contract 4` and `valid+invalid` →
  `valid/invalid`).

## Findings

- [nit][re-raised] Compatibility matrix — Windows 10/11 contributor machine row:
  "tree file names follow the shipped case-unique stem rule" — the shipped
  case-uniqueness assertion walks `format_dirs()` and asserts over
  `fixtures/{valid,invalid}` only (`crates/core/tests/conformance.rs:277-315`,
  "fixtures/{kind} of {} holds names that differ only by case"), and
  `schemas/README.md:26-27` scopes the rule to "Fixture file names …
  case-unique within a fixture directory". The row claims the coverage over
  `data/` trees, which no shipped test enforces. Substance is safe — ids never
  become path components, the design's own reason — so it stays a nit. Fix: in
  that one cell, scope the claim — "fixture names follow the shipped
  case-unique stem rule; `data/` file names follow the stem rule as hygiene,
  not a load-bearing check".
- [nit][re-raised] Chosen design → Adversarial posture (final sentence) +
  Files created → `schemas/README.md` row + Rollout 1 — "`schemas/README.md`'s
  guard section gains those two constants and their rejections beside the
  shipped three, so its 'public names for these limits' claim stays true" —
  the README's canonical sentence is "The three constants are the reference
  implementation's public names for these limits" (`schemas/README.md:64-65`).
  Adding `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH` leaves "the three constants"
  naming three of five limits, so the claim stays true only if that sentence is
  amended, and Rollout 1 and the Files created cell record only the addition.
  The Revision log's r2 reason ("the README's 'three constants' claim needed
  the new limits in it") states the intent, not the edit. Fix: name the edit in
  Rollout 1 and the Files created cell — the guard section's "three constants"
  sentence becomes a five-constant sentence.

## Checklist (this pass)

1. Matrix rows — every row carries a status; no `unknown`, no empty cell; the
   hub row's `no (v1) — out of scope` is the r1 closure. Unchanged. Pass.
2. OS input APIs — the design targets none: Non-goals disclaim any
   input-event API, and the matrix's game-config row is `n/a` ("no game config
   I/O here"). Unchanged. Pass.
3. Node version floor — N/A (Rust); the pinned-toolchain row states the floor
   (1.85.0) and that nothing adds a dependency. Unchanged. Pass.
4. Native modules — none: `bindforge-core` stays depends-on-nothing,
   `bindforge-transforms` depends on the path crate only. Unchanged. Pass.
5. Keyboard reality — glyph naming fixed to US-ANSI with the non-US-layout,
   ISO/JIS, `pos:`, and gamepad rows all `no (v1)` with the v2 path named;
   `keyboard:semicolon` / `keyboard:apostrophe` cover the punctuation names,
   the manifest's `dead-keys` is the game-ignores-input fact. A16's basis is
   now a live citation — verified: `https://en.wikipedia.org/wiki/Home_row`
   resolves (redirect) to Touch typing, which states the QWERTY home keys are
   `ASDF` / `JKL;`, plus ADR-009:28-31 for the practice split. This strengthens
   the row; it breaks nothing. Pass.
6. File paths — id → index, never id → path; the game-id grammar (Encoding
   rules) excludes `..`, `/`, `\`, leading `.`; `DirEntry::file_type()` with
   the cap on bytes read; no rejection message carries a local path. The r4
   retarget of the §5 pointer to "(Encoding rules)" is more accurate than the
   section number it replaces, and "(contract 3)" for the stem rule resolves to
   Id → file, where it is stated. Pass.
7. Platform test per supported platform — Test strategy #14 covers Linux CI and
   the Windows contributor machine, the matrix's two `yes` runtime rows; macOS
   is `no (v1)`, so no leg is owed. Unchanged. Pass.

## FYI (no severity)

- The `contract N` cross-reference set resolves: Context's ordering matches the
  Chosen design section numbering (1 taxonomy, 2 input vocabulary/reach/
  placements, 3 chains, 4 transforms, 5 manifests), and every cited reference
  lands in the section that canonically states the cited rule.
- Contract 5's "`default` = `[id, input]` (the emphasis fact, contract 1)" and
  contract 1's "recorded as a manifest `default` row (contract 5)" point at
  each other. That is one canonical statement with two directions, not a
  contradiction; a drafting seat may want contract 5 to say "the emphasis
  statement is canonically contract 1" to remove the round trip.
- `https://en.wikipedia.org/wiki/Home_row` is a redirect to the Touch typing
  article; the citation is live and supports the parenthetical as quoted.
