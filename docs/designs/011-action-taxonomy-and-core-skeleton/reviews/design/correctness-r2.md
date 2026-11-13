# correctness review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (6 hunks, all applied — every `+` line matches
the current `02-proposal.md`: Goals, Options considered, The model error rule,
Placeholders and lineage, Test strategy #1, Context). Inputs read: the diff,
`02-proposal.md` (incl. Revision log), `docs/adrs/011-action-taxonomy-and-core-skeleton.md`.
No earlier correctness report was consulted.

All six hunks are language-only: Oxford commas, a hyphenated compound
(`build-script-generated`), a verb fix (`gains … to` → `adds … to`), a possessive
(`ADR-014's profiles`), and an em-dash appositive replacing a restrictive `that`
clause. No name, shape, count, rule, citation, or assumption row changed, so
checklist items 1, 5, 6 and 7 have nothing new in scope to break:
- The appositive in Placeholders and lineage keeps the same subset of `ADR-009`
  references retargeted ("the ones that describe future model ownership"), still
  matching Delta inventory's "three module-doc lineages" and Rollout 4.
- The Test strategy #1 comma removes a mis-parse (`…cycle-target` and a 63-octet
  segment) rather than introducing one; the accept/reject lists are unchanged.
- r1 closures the diff touches are intact: the `action-ids/v2` grammar-widening
  rule (Stability × Risks) and the escaped-id echo rule (The model) still say
  what the Revision log records.

Checklist 2 (anti-hallucination spot-check), done on URLs cited in the sections
the diff touches or nearby: `https://www.rfc-editor.org/rfc/rfc1035` (A5) live and
supporting — §2.3.1 `<label> ::= <letter> [ [ <ldh-str> ] <let-dig> ]` with
let-dig-hyphen interior and letter-or-digit last, "no significance is attached to
the case"; `https://github.com/toml-lang/toml-test` (A6) live and supporting —
"test one thing and one thing only", "named after the fault"; `https://semver.org/`
(A12) live and supporting — "issue a new minor release with the deprecation in
place … remove the functionality in a new major release". No dead or
contradicting source found.

## Findings
- [nit][new] The model (error rule) — "with an offending id echoed, escaped to printable
  characters" — the new comma turns one predicate into two, so `escaped` can be read
  as a property of the rejection message rather than of the echoed id, which is
  weaker than the r1 closure the Revision log records ("an offending id is echoed
  escaped to printable characters"). Fix: replace "echoed, escaped to printable
  characters" with "echoed after escaping to printable characters" — one clause,
  same guarantee, no new mechanism.

## FYI (no severity)
- Assumption register A5 — "RFC 1035 §2.3.1 … ≤63 octets": the 63-octet label bound
  is RFC 1035 §2.3.4, not §2.3.1 (the character and case rules are §2.3.1/§2.3.3).
  Citation precision only; the cited URL supports the row. This text is untouched by
  the r2 editorial diff, so it is not a scope finding and gates nothing.
- The diff adds no `action-ids` interface name; `lib.rs` "the four new names" still
  matches `AxisKind`, `ActionId`, `ActionRegistry`, `load_action_registry` — no
  two-names-for-one-thing risk was introduced (compatibility/complexity seats, not mine).
