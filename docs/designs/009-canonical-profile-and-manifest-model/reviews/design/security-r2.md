# security review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial pass), read against
`02-proposal.md` and `docs/adrs/009-canonical-profile-and-manifest-model.md`.
Ratification pass: my previous security findings were none, so the bar here is
"did the changed text break a security guarantee, or make a cited section false".
Checklist walked item by item over every hunk in the diff.

- **Item 3 (path handling)** — the only path-security text the diff touches is the
  game-id grammar clause: "which excludes `..`, `/`, `\` and a leading `.`" → "…`\`,
  and a leading `.`". Oxford comma only; the exclusion set is byte-identical, and the
  "this grammar, not a path rule, is what gates a pin's halves" claim is intact.
  The Id → file paragraph (index lookup, `DirEntry::file_type()`, cap on bytes read,
  ids string-compared only) is not in any hunk.
- **Item 2 / 7 (untrusted parsing, fail closed)** — the Adversarial posture hunk only
  inserts "are" ("New `CoreError` variants **are** pinned by a `fixtures/invalid/`
  document **only where `parse` can see the condition**"); the parse-visible
  enumeration, the suite-level enumeration, the full variant set, and the
  `InvalidValue`-as-`Err` rule for a malformed policy invocation are unchanged.
- **Item 4 (secrets/PII)** — the only touched PII-adjacent text is the Revision-log
  mention of `generated-by`/`author:alice`, which matches the body's closed producer
  list (`policy|preset|generator`) and its "publishes no free-text attribution" claim.
- **Item 5 (dependencies)** — design names no dependency for the shipped slice; the
  diff's serde/`toml_edit` sentence is unchanged context ("nothing in this design adds
  a dependency", Compatibility matrix). Checked RustSec/OSV for `serde` (only
  `serde-json-wasm` RUSTSEC-2024-0012, a different crate) and `toml_edit` (no advisory
  found); nothing named carries a known unfixed advisory.
- **Cross-reference renumbering (§n → contract n)** — verified each new target: the
  stem rule and `MAX_INDEX_ENTRIES`/`MAX_INDEX_DEPTH` are in §3; the canonical
  emphasis statement is in §1; `apply_policy`'s `Err` channel is in §4. No reference
  was pointed at the wrong contract, so no security claim was re-aimed.
- Items 1 (trust boundaries) and 6 (least privilege) are untouched by the diff.

## Findings

- [minor][new] Chosen design → Profiles are chains / Editor notes — the diff's one
  net-new line, `"for` is required at parse" is ambiguous; author to clarify" — leaves
  an unresolved ambiguity in the clause that is the fail-closed gate for pins. The
  body sentence it flags reads, in the same paragraph, "`for` … absent = a profile not
  bound to a game", so "required at parse" has two readings: presence mandatory
  (which rejects the unbound profile the same paragraph, and Migrations' "`for` absent
  = not bound to a game", exist to serve), or grammar-validated when present (which is
  what Test strategy #6's "a malformed `for` pin is `BadPin`" already asserts). Both
  readings fail closed, so nothing is unsafe; what is unsettled is the enforcement
  coverage of the guard the r3 log credits with making `resolve`'s `seeded-from` check
  and the drift gate non-vacuous ("one rule closes both") — and the proposal ships
  carrying an open editor question about it. Fix: replace "`for` is required at parse`"
  with the rule the suite already pins — "`for`, when present, is grammar-checked at
  parse (a malformed pin is `BadPin`) and its `game` + `revision` must name a manifest
  document in the index (`UnresolvedAncestor`); a profile with no `for` is unbound" —
  then delete the Editor notes line. No new mechanism: the text replaces an ambiguous
  clause with the two rules Test strategy #6 and #9 already assert.

## FYI (no severity)

- The added `Editor notes:` line is an author-directed TODO sitting in the proposal
  body under the Revision log; hygiene for the authoring pass, not a security claim.
- Revision log → Transforms: "which enforces no free-text attribution" reads as the
  negation of its intent; the body's "publishes no free-text attribution" is the
  accurate shape. Language, not security.
- The `docked` hunk now reads "apply-time work owned by ADR-012 (Deferred)", which
  matches the Deferred clause naming `docked`-honoring and the ADR's "Not decided
  here: codec implementations". Consistent deferral — not re-opened.
