# security review r1 (dissent pass c3)

verdict: approve

Scope reviewed: the r2→r3 diff of `02-proposal.md` (authoring bar, §3 `resolve` bullet + Id → file, §4 policy/`apply_policy`,
Adversarial posture, Risks row 1, Test strategy #1/#3/#9/#10, Revision log entries r3), then the full current document,
against `docs/adrs/009-canonical-profile-and-manifest-model.md`.

## Check on my r1 findings

1. **[major, r1] Dangling `for` pin was an unruled state; `Resolved.manifest: Option<…>` read it as an unbound profile.**
   **Closed.** §3 Id → file now states `UnresolvedAncestor` once as "any document reference with no document — a `derives`
   entry, **or a `for` pin whose `game` + `revision` names no manifest document in the index**", the `resolve` bullet states
   `manifest` is `Some` whenever the profile declares `for`, and Test #9 adds the well-formed-pin-naming-no-manifest case.
   No new `CoreError` variant and no new mechanism were added, and the closure propagates: a manifest deleted or bumped
   out from under a pin now fails Test #12, so the `seeded-from` check and the drift gate are never vacuous. What remains at
   that location is a narrower version of the same class — see finding 1 below.
2. **[minor, r1] Walked trees keyed on entries a foreign-format document lacks.** **Closed.** "Each tree accepts only the
   formats it holds — `data/profiles/` `profile`/`patch`, `data/manifests/` `manifest` — and a document declaring another is
   the shipped `InvalidValue` naming the tree", pinned by Test #9's two planted-document cases. Reuses a shipped variant;
   inherits the shipped `one_line` bounding (`crates/core/src/error.rs:24,155-169`), so the echoed value cannot forge log
   lines. Residual scope gap in the enumeration — finding 2.
3. **[nit, r1] Exception list one variant short (`DuplicateId`).** **Closed**, and closed better than asked: the clause is
   now the general rule ("pinned by a `fixtures/invalid/` document **only where `parse` can see the condition** … and
   otherwise … by the model suite that calls their producer") with a per-variant mapping. I checked the mapping against the
   full variant set: all 13 accounted for (`BadPin` is parse-visible, as claimed at `parse` in §3). New wording creates one
   tension in untouched text — finding 3.

## Findings

- [minor][new] Chosen design → Profiles are chains → Resolution (`resolve` bullet) — "`Resolved { bindings: …, chain: Vec<String>, manifest: Option<ManifestPin> }`, whose `manifest` is `Some` whenever the profile declares `for`" and "`resolve` also rejects a `seeded-from` the pinned manifest does not declare" — the closure made the pin's *existence* total, but not *which* manifest. `for` is an optional entry on every chain document (§3: `patch/v1` "carries the same optional chain entries"), and a chain has one `manifest` field and one "pinned manifest" (§5: "resolves every profile … against the manifest revision **it pins**"). A shared chain whose base pins `remnant2@0.1` and whose tip pins an existing `remnant2@0.2` is a valid-document state with no stated rule: `resolve(root, 1)` (base alone) and `resolve(root, chain.len())` then check `seeded-from` against different manifests, and the drift gate's verdict for one profile id is implementation-order dependent — the exact "walk-order dependent" determinism loss the `DuplicateId` clause in the same paragraph exists to prevent, on the one mechanism the design names as the trust mechanism (validation, not provenance). Fix: one clause in §3 — the pin is a chain fact: a chain's documents declare one `for`, and a document whose `for` differs from the chain's is `InvalidValue` (no new variant, no new mechanism) — plus one Test #9 case beside the pin lookups.
- [nit][new] Chosen design → Profiles are chains → Id → file — "Each tree accepts only the formats it holds — `data/profiles/` `profile`/`patch`, `data/manifests/` `manifest`" — the rule is stated as "each tree" but enumerates only the two indexed trees, while the design ships four walked `data/` trees (`data/placements/*.toml` is loaded in Test #3/#4, `data/policies/` carries the policy content Test #11 applies; `schemas/README.md`'s data-homes rule says all of `data/` "holds content discovered by walking"). A misfiled document in those two trees is again format-dispatched on the document's own declaration with no stated gate. Fix: extend the parenthetical to `data/placements/` → `placement` and `data/policies/` → `policy`, same `InvalidValue` at load; no new mechanism, and Test #9's planted-document sentence covers it.
- [nit][new] Test strategy #8 (with Adversarial posture) — "a manifest naming an unknown action id, an axis kind that disagrees with the taxonomy (`AxisKindConflict`), … is rejected, each with its own fixture" — the r3 pinning rule says these conditions are *not* carried by a `fixtures/invalid/` document (`parse` cannot see the taxonomy, and the harness asserts every `fixtures/invalid/` file is rejected by `parse` — `crates/core/tests/conformance.rs:138-154`), yet #8's untouched "each with its own fixture" reads as if they live there; a reader who places them under `schemas/manifest/v1/fixtures/invalid/` fails the harness walk, not the intended check. Fix: one phrase in #8 — its cases are data under the model suite (as the posture's "over fixture data" says), never under `schemas/*/v1/fixtures/invalid/`.

## FYI (no severity)

- The rewritten authoring bar claims CI enforces "grades are non-decreasing in grid distance from that hand's home-graded keys" and then attributes pinning to "Test strategy #3's model tests"; #3 has cases for cross-hand coordinate agreement, graded-set coverage, `off-reach`, and the admitted-name list, but no case asserting the monotonicity property. (correctness/complexity seat)
- The resolve bullet's rejection of a `seeded-from` the manifest does not declare names no `CoreError` variant, and no variant in the shipped set obviously owns it; only Test #12 (integration) asserts the failure. The variant list is the compatibility promise, so this is the compatibility seat's to pin. (compatibility seat)
- Security checklist items 5 and 6 evaluated cleanly: the design adds no third-party dependency (`bindforge-core` stays depends-on-nothing, `bindforge-transforms` depends on the path crate), so no advisory lookup applies, and the privileges stay filesystem reads of `schemas/` and `data/` with symlinks not followed and the size cap on bytes read.
