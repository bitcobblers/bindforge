# Complexity review r2

Scope: the diff `reviews/design/editorial-r2.diff` (editorial confirmation pass, round 2).
Inputs read: `02-proposal.md`, `docs/adrs/009-canonical-profile-and-manifest-model.md`,
the proposal's Revision log, and the diff itself. No review report or transcript opened.
Previous complexity findings: none.

verdict: request-changes

## Findings

- [major][new] Chosen design → Profiles are chains (resolution bullet, `resolve` API) —
  "`for` is required at parse" — the diff adds the trailer
  "`Editor notes: Chosen design → Profiles are chains — \"`for` is required at parse` is ambiguous; author to clarify.`",
  and the flag is sound: the clause is a surplus global constraint that contradicts the
  section it lives in. (a) Same section, same list: the entry set "grows by **optional**
  entries only, so the shipped valid fixtures stay valid (no narrowing)" and `for` is
  defined "`absent` = a profile not bound to a game". (b) Under the literal required-at-parse
  reading, the shipped valid fixture `schemas/profile/v1/fixtures/valid/basic-profile.toml`
  (`format`, `version`, `id`, `derives` — no `for`) stops parsing, which falsifies that
  "no narrowing" claim, the Goals' "machine-checkable by the existing harness with no
  harness change", and Rollout → Migrations' "the `profile/v1` entry additions are
  optional, so every shipped fixture resolves unchanged". (c) No consumer needs the global
  rule: the sentence's own stated consumer is the `seeded-from` check ("so the check never
  runs against nothing"), and the Revision log's recorded r3 decision is the local rule —
  "`seeded-from` requires `for` at parse, and a `for` naming no manifest in the index is
  `UnresolvedAncestor`" — which closes the dangling-pin gap without touching unbound
  profiles. (d) Strictly read, every profile is bound to one game revision, which works
  against the very case the same clause cites (the ADR's accessibility /
  alternative-controller profile, ADR-009 Consequences: "authoring a personal
  handedness/accessibility policy once and re-applying it to every game BindForge supports").
  Test strategy #6 and #9 pin `BadPin` and pin-valid resolution only, so the suite gives an
  implementer no signal on requiredness — the ambiguity lands in `profile/v1`, the format
  implementation starts from.
  Fix: delete "`for` is required at parse`" and state the rule once, locally, in the
  bullet it governs: "`for` is optional (`absent` = not bound to a game); a present `for`
  must be grammatical (`BadPin`) and name a manifest document in the index
  (`UnresolvedAncestor`), and a `seeded-from` requires `for` (`InvalidValue`)`". This
  replaces the global requirement with the recorded r3 rule, keeps the drift gate and the
  preset check from running against nothing, and restores the unbound-profile case. Then
  remove the `Editor notes:` trailer, whose flag this closes.

## Complexity checklist walk-through (diff-scoped)

1. New package/module/interface: none in the diff — no path, type, or API line changes.
2. Moving parts traced to a Goal: the only content-bearing additions are the `Editor notes`
   trailer and the normative clause it flags, above; nothing else in the diff adds a part.
3. Design 30–50% smaller: not applicable to a language-only diff; no text in the diff
   changes the design's size. The one shrink the diff does surface is the finding above —
   dropping a global rule in favour of the local one already recorded in the Revision log.
4. New dependencies: none (`bindforge-core` stays depends-on-nothing in every touched line).
5. Deferral so the first merged slice stays thin: unchanged by the diff; the "valid+invalid"
   → "valid/invalid" and "Platform:" → "Platform —" edits are style only.
6. Knobs with no consumer / one-implementation abstraction: the mandatory-`for` rule (above).
7. Value check: the diff is language-only and adds no mechanism, dependency, or knob, so the
   complexity ratified in earlier rounds is untouched — and, at seed scale, with reuse of the
   shipped grammar, budget, version gate, chain guard and harness, it reads proportionate to
   what the five contracts buy. The one complexity-relevant artifact the diff surfaces is
   the surplus mandatory-`for` clause.

Cross-reference normalization checked for regressions: `§1`→`contract 1` (emphasis fact, §1
canonical statement), `§3`→`contract 3` (stem rule, index constants), `§4`→`contract 4`
(`apply_policy`'s `Err`) all resolve to the same sections the `§` forms pointed at — no
cited section was made false, and no section I approved was broken by the diff.

## FYI (no severity)

- Manifests, opening line: "`<game>` follows the game-id grammar and the stem rule
  (contract 3)" — the game-id grammar is canonical in **Encoding rules**; contract 3 carries
  only the stem rule. The imprecision predates the diff (the `§3` form pointed the same way),
  so the editorial pass normalized a reference it did not create — editorial/formatting seat.
- The `Editor notes:` trailer is a process artifact sitting in the proposal body; removing it
  (once the flagged clause is resolved) belongs to the editorial seat, not to the design.
