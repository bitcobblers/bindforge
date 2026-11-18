# Correctness review r2

verdict: approve

Scope: ratification pass over `reviews/design/editorial-r2.diff` only (the single
editorial confirmation pass of round 2). Previous correctness findings entering
this pass: none (the editorial phase is a single language pass per round).

## Findings

None. Every hunk was checked to confirm it is language-only and does not break
previously approved content or falsify any cited section:

- Context: "moves the namespace ... It raises the shared bound from 63 to 128
  octets with one stated home and one exported constant, amending `013-L5`" —
  sentence resegmentation only; the ADR-024 Decision clauses it encodes
  (single-segment id, namespace field, 63→128, `013-L5` amendment, `024-R1`
  reservation) all survive verbatim in substance.
- Goals: "Enforce the grammar and the bound before any `DiscoveredGame` leaves a
  plugin or the CLI" — identical obligation to `024-L1`; the escape-and-truncate
  echo clause is untouched.
- Options (fold table): "nothing is transliterated multi-character" → "nothing
  maps to more than one character (no `ß`→`ss`, no `æ`→`ae`)" — same constraint,
  still consistent with the canonical derivation's "no multi-character
  transliteration" and the Non-goals transliteration clause.
- Options (collision suffix): "the same named-refusal path used for an
  un-derivable name" — same fallback chain as `024-C2` and the Derivation
  section; option (a)/(b) rejections unchanged in substance.
- Chosen design (core): the reflowed bullet still deletes exactly the four
  private constants A2 names (three `MAX_SLUG_OCTETS` plus `action.rs`
  `MAX_SEGMENT_OCTETS`) and keeps "All four sites reference the exported
  constant" and the `format!` refusal-message rule.
- Identity shape: comma-only change; the `"unknown"` vs `"SteamLinux"`
  divergence statement is intact.
- Emission: "checks the final value against `bindforge_core::is_slug_name` and
  the bound" and "install-directory fallback" — same checks, same order; the
  de-comma'd "is the CLI-boundary line of the next section alone" still matches
  the Revision log decision that plugin-side refusals are count-only and
  `024-L1`'s named refusal exists only at the CLI boundary.
- CLI boundary / Specs / Rollout: backtick and connective-only edits
  ("and names the renamed `segment-past-cap.toml`", `024-L1`, `024-L3`); the
  action-id segment still "borrows only the bound number".
- Test strategy #1: "are reproduced exactly" — grammar fix; the pinned case set
  is untouched.
- Deferred: "A resolver that maps an `unknown`-namespaced slug onto a real
  manifest id — ADR-025 (reserved, `024-R1`)" — article addition only; still
  matches ADR-024 `024-R1`, so the deferral is not re-opened.

Checklist items 1 (assumption register), 2 (research URL spot-checks), 4
(new edge cases), and 7 (prior-round blocker/major closure) are not applicable
to this pass: the diff touches no register row, no cited source, no behavioral
clause, and no prior-round blocking finding is open.

## FYI (no severity)

- The diff's `---` header points at the editor's scratch copy
  (`tmp/bindforge-editorial-game-slug-identity-r2.md`); the applied file is the
  proposal itself — expected for an editorial pass, nothing for the reviser.
