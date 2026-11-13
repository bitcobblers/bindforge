# complexity review r3c0 (ratification pass)

verdict: approve

Scope: the blob diff `f752153 → 4d07da4` of `02-proposal.md` (the r4 revision), plus the
full current document. Inputs: the proposal, `docs/adrs/009-canonical-profile-and-manifest-model.md`,
the Revision log, my r3 findings. Complexity checklist only.

## What the r4 diff does, complexity-wise

Every changed passage is net-neutral or net-reducing on complexity; nothing in it adds a
mechanism, a module, a dependency, a knob, or a deferral:

- **Context** — adds only the cross-reference key "contracts 1–5, in the order listed". Checked
  against the section numbering and against ADR-009's own Decision numbering (§1 taxonomy,
  §2 input vocabulary + reach, §3 chains, §4 transforms, §5 manifests): both orders agree, so
  every `contract N` reference in the body resolves. Spot-checked all five:
  `contract 5` (manifest `default` row, §1), `contract 2` (`mouse:*` names, §4; authoring bar,
  Risks), `contract 3` (stem rule, §5; `apply_policy`'s `Err` → `contract 4`, §Adversarial;
  `MAX_INDEX_*` + `MAX_CHAIN_DEPTH` unit, Risks), `contract 1` (the emphasis fact, §5). Each
  lands in the section its number names. No `§N` reference survives the relabel, so the document
  carries one convention, not two.
- **A16 basis** — the dead `Cursor_home_key` citation is replaced by
  `https://en.wikipedia.org/wiki/Home_row`, which is live (resolves to *Touch typing*,
  anchor `#Home_row`) and states verbatim what the parenthetical claims: "the QWERTY layout,
  'A S D F' and 'J K L ;' are the home keys on the home row". `docs/adrs/009…:28-31` says what
  the cell attributes to it — community practice splits on IJKL vs Returnal's built-in OKL;.
  The `verified` row now rests on two checkable sources. Sound closure, no added mechanism.
- **Profiles are chains → `resolve`** — the r3 clause "`for` is required at parse" is narrowed to
  a `seeded-from`-local rule and the pin-lookup/`UnresolvedAncestor` statement is *delegated* to
  Id → file instead of restated. De-duplication, not addition. Traced the whole invariant set for
  consistency: entry set (line "`for` … absent = a profile not bound to a game"; "`seeded-from` …
  **requires `for`** … is `InvalidValue`") → §3 resolve bullet → Id → file ("**or a `for` pin whose
  `game` + `revision` names no manifest document in the index**, which is what makes
  `Resolved.manifest` `Some` … and keeps the `seeded-from` check and the drift gate from running
  against nothing") → drift gate ("against the manifest revision it pins") → Test strategy #6
  (`seeded-from` with no `for` = `InvalidValue`), #9 (well-formed pin naming no manifest =
  `UnresolvedAncestor`), #12 ("for every profile that pins it") → Defaults ("`for` absent = not
  bound to a game") → Files table ("optional entries added; shipped fixtures unchanged"). Every
  link present; the removed "never runs against nothing" guarantee is preserved verbatim in
  Id → file, so no claim lost its backer. The rationale cites ADR-009 Context 3
  (disabilities / alternative controllers), which is in the ADR; nothing in the Decision requires
  a mandatory pin.
- **Adversarial posture** — the sentence fragment "New `CoreError` variants, pinned by …" is made
  a real sentence ("New `CoreError` variants **are** pinned by …"). The enumeration of producers,
  the parse-visibility rule, the thirteen-variant set, and the ADR-012 `MergeConflict` deferral are
  byte-identical to the text I approved.
- **Remaining hunks** (§2 word, Authoring bar verb, §1/§5/Encoding/Options/Rollout/Test #14,
  Revision log) are punctuation, voice, and cross-reference relabels. Meaning unchanged in each;
  the entry-set, fixture-obligation, and grading obligations are untouched.

## Findings

None. The r4 revisions broke nothing I approved at round 3 and made no section I cited false.

## FYI (no severity)

- My r3 findings 1 and 2 (Authoring bar: "grades are non-decreasing in grid distance … every
  rejected condition is pinned by Test strategy #3's model tests" — Test strategy #3's
  enumeration still carries no monotonicity clause, and the clause still claims #4's
  placement conditions; Input vocabulary: the admitted keyboard-name list's machine-readable home
  is unnamed, Test strategy #3 pins the out-of-list behavior only) are **unchanged in substance**
  by this round — r4 touched only the verb ("… is pinned by") and dropped one "admitted". Under
  the ratification rule that is not a breakage, so they are not re-reported here; they remain the
  residue of the r3 fixture-obligation closure and each has a one-clause fix.
- My r3 nit (Rollout 2's blanket "the `CoreError` variants named in Adversarial posture" vs that
  section's `MergeConflict`-to-ADR-012 deferral) is likewise untouched by the rewrite of that
  sentence.
- The Revision log records no rebuttal for any of the three; the r4 entries added are Context,
  Profiles are chains, Options/register (A16). Worth the reviser knowing which are open, not
  declined.

## Checklist value statement (item 7)

Proportionate, and this round moved it down, not up: one duplicated rule statement removed, one
entry-set narrowing withdrawn (an unbound profile is parsable again), one dead citation replaced
by a live one, and a cross-reference convention introduced that resolves against numbering the
document already had. No new package, module, interface, knob, abstraction layer, dependency, or
deferral appears in the diff; nothing in it needs a slice thinning beyond what Rollout already
sequences.
