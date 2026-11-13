# security review r3 (ratification pass c0)

verdict: approve

Scope: `git diff f7521536 4d07da4d` of `02-proposal.md` (r4 revisions), then the full
current document, plus `docs/adrs/009-canonical-profile-and-manifest-model.md` and the
Revision log. Inputs limited to those named by the caller; no review report or
transcript opened.

## What r4 changed, security view

The r4 diff is almost entirely referent hygiene — `§N` → `contract N` (anchored by the
new Context clause "contracts 1–5, in the order listed"), `Test #N` → `Test strategy #N`,
one Oxford comma in the id-shape sentence, four punctuation edits, the A16 citation swap,
and one substantive edit in `resolve`. Checked:

- **Contract numbering resolves as claimed.** Context's list (taxonomy, input
  vocabulary/reach, profiles-are-chains, transform policies, manifests) matches the
  section order of Chosen design §1–§5 and ADR-009's own `### 1`–`### 5`. Every
  `contract N` cross-reference lands on the right section: contract 5 (manifest `default`
  row, drift mechanism), contract 1 (emphasis fact), contract 2 (`mouse:*` parses,
  authoring bar CI), contract 3 (stem rule, `MAX_INDEX_ENTRIES`/`MAX_INDEX_DEPTH`,
  `MAX_CHAIN_DEPTH` unit), contract 4 (`apply_policy`'s `Err`). No dangling `§` reference
  survives in the document.
- **The r3 `for`/pin closure survives the relocation.** `resolve` no longer states
  "`for` is required at parse"; the guarantee now reads from two places that together
  keep it total — the entry set ("`seeded-from` … **requires `for`** … a `seeded-from`
  with no `for` is `InvalidValue`") and Id → file ("`UnresolvedAncestor` fires … **or a
  `for` pin whose `game` + `revision` names no manifest document in the index** … which is
  what makes `Resolved.manifest` `Some` whenever the profile declares `for` and keeps the
  `seeded-from` check and the drift gate from running against nothing"). Both are pinned:
  Test strategy #6 asserts the `InvalidValue`, #9 asserts the well-formed-pin
  `UnresolvedAncestor`. The r4 narrowing is an improvement on the security axis: it keeps
  the unbound (accessibility) profile parsable, which a global `for`-required entry set
  would have destroyed, without weakening fail-closed behavior on the pinned path.
- **Path/traversal family (A18) untouched.** The id-shape sentence is the only edit inside
  it and is punctuation only; "this grammar, not a path rule, is what gates a pin's
  halves", "no path is ever built from a document string", `DirEntry::file_type()` with
  the cap on bytes read, and Test #9's escape-carrying-id case all stand.
- **Budget/fail-closed family untouched.** 1 MiB pre-read, 16-level nesting, closed entry
  sets, bounded one-line messages with no local path, `MAX_CHAIN_DEPTH` unchanged,
  `MAX_INDEX_ENTRIES`/`MAX_INDEX_DEPTH` named in the README guard section,
  `BadPin` listed among the parse-visible conditions.
- **PII/attribution untouched.** `generated-by` stays a `<namespace>:<part>` id over the
  closed list `policy|preset|generator`, so a shared document publishes no free-text
  attribution — consistent with ADR-009's "recorded, never trusted".
- **Dependencies: none added.** core stays depends-on-nothing, `bindforge-transforms`
  depends on the path crate only; the diff adds no dependency, so the advisory/CVE item
  has nothing to fetch. The one new external reference (A16) is a citation, not a
  dependency — verified live: it redirects to the Touch typing article and states
  "the home row keys are: 'ASDF' for the left hand and 'JKL;' for the right hand", which
  is exactly the parenthetical the row claims, and `docs/adrs/009…:28-31` does carry the
  practice-split/Returnal-OKL; observation. The `verified` row is checkable again.

No r4 edit breaks anything approved at r3, and no section cited in my r3 findings was made
false by the new text. The three r3 items below are still open — none of them is a
regression, and none gates.

## Checklist coverage

1. Trust boundaries — pass. Shared binding files remain the untrusted class, read through
   the same reader, index, and closed entry sets; no new data source is introduced.
2. Parsing untrusted data — pass. Budgets, closed entry sets, refused-TOML fixtures, no
   code evaluation, no compression; `IndexTooLarge`/`IndexTooDeep` bound the aggregate.
3. Path handling — pass. Ids are string-compared only; pins are grammar-gated then
   index-looked-up; canonicalization is Deferred (ADR-012) and consistent with it.
4. Secrets/PII — pass. `generated-by` closed producer list; no rejection message carries a
   local path.
5. Dependencies — pass (vacuous: no dependency is named by the design).
6. Privileges — pass. Read-only walk of `data/` plus pinned contract paths; no device,
   network, cwd, or database access (Test strategy #14).
7. Fail closed — pass, with the chain-pin gap in finding 1 as the one place where a
   validation verdict for one profile id is not single-valued.

## Findings

- [minor][re-raised] Chosen design → Profiles are chains → Resolution (`resolve` bullet) +
  entry-set paragraph — "whose `manifest` is `Some` whenever the profile declares `for`" /
  "`patch/v1` … carries the same optional chain entries" / Revision log (r4): "a rule local
  to `seeded-from`, since `for` stays optional". The pin's *existence* is total; *which*
  manifest a chain pins is still unruled, and r4 makes that state plainly legal: `for` is an
  optional entry on **every** chain document (base and patch alike), while `Resolved`
  carries one `manifest` field and the design speaks of one "pinned manifest" (§5, "the
  manifest revision it pins"). A chain whose base declares `for = "remnant2@0.1"` and whose
  tip declares a valid `for = "remnant2@0.2"` is a fully valid document set with no stated
  rule, so `resolve(root, 1)` and `resolve(root, chain.len())` evaluate the `seeded-from`
  check against different manifests, and the drift gate reports a different revision for one
  profile id depending on which document's pin the implementation reads. That is the same
  walk-order/implementation-dependent determinism loss the adjacent `DuplicateId` clause
  exists to prevent ("an ambiguous pin key would make the drift gate's verdict walk-order
  dependent"), on the mechanism ADR-009 names as the only trust mechanism — "Provenance
  labels the chain; it never substitutes for validation". Fix: one clause in the §3
  entry-set paragraph — the pin is a chain fact: a document whose `for` differs from the
  `for` of the document its `derives[0]` names is the shipped `InvalidValue` (no new
  `CoreError` variant, no new mechanism, and an unbound chain declares `for` in neither
  place, so the accessibility case is untouched) — plus one Test strategy #9 case beside the
  existing pin-lookup sentences, so `Resolved.manifest` has exactly one referent.
- [nit][re-raised] Chosen design → Profiles are chains → Id → file — "Each tree accepts
  only the formats it holds — `data/profiles/` `profile`/`patch`, `data/manifests/`
  `manifest`". The gate is stated per tree and enumerates two of the four `data/` trees the
  design walks and loads (`data/{manifests,profiles,policies,placements}/`, Files created;
  `data/placements/` is load input for Test #3/#4 and `data/policies/` for #10/#11), while
  `schemas/README.md`'s data-homes rule says `data/` "holds content discovered by walking".
  A document misfiled in the two unlisted trees is dispatched on its own declared `format`,
  with no gate named — the fail-closed family this clause closed for the indexed trees.
  Unchanged by r4, so not a regression. Fix: extend the parenthetical to `data/placements/`
  → `placement` and `data/policies/` → `policy`, same shipped `InvalidValue` at load.
- [nit][re-raised] Test strategy #8 — "a manifest naming an unknown action id, an axis kind
  that disagrees with the taxonomy (`AxisKindConflict`), … is rejected, each with its own
  fixture". The r3 posture rule ("pinned by a `fixtures/invalid/` document **only where
  `parse` can see the condition** … and otherwise, over fixture data, by the model suite")
  excludes these conditions from a fixtures directory, and #8's `AxisKindConflict` /
  `UnknownAction` / `UnknownInput` cases are exactly the ones Adversarial posture routes to
  #8 as model-suite cases; untouched "each with its own fixture" reads as `schemas/*/v1/
  fixtures/invalid/` placement, where the shipped walk asserts every file is refused through
  `parse` — the walk then fails on documents that parse cleanly. Unchanged by r4. Fix:
  one phrase — "each with its own case in the model suite over fixture data", never under
  `schemas/*/v1/fixtures/invalid/`.

## FYI (no severity)

- Rollout 5 lists the three suites in a different order than the three test files it names
  ("the model-walk, manifest-drift, and transform-semantics suites (Test strategy #10, #11,
  #12) as `crates/core/tests/{model_walk.rs,manifest_drift.rs}` and
  `crates/transforms/tests/policy_semantics.rs`") — #10/#11/#12 map to the third/first/second
  file. Inferable, and the prose pairs each suite by name to its file; consistency seat's call.
- The A16 citation is `en.wikipedia.org/wiki/Home_row`, which is a redirect to the
  *Touch typing* article anchored at `#Home_row`; the cited sentence is present, so the row
  is checkable — a reader following it lands on a differently-titled page.
- The A16 row's added ADR pointer is `docs/adrs/009…:28-31`; ADR-009's Context observation 1
  does carry both halves of the claim (practice split; Returnal's built-in OKL;). Line
  anchors into an ADR age less well than the section anchors used elsewhere in the register.
