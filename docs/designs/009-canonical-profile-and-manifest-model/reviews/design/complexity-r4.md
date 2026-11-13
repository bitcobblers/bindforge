# Complexity review r4

Scope: `reviews/design/editorial-r4.diff` (the round-4 editorial pass over `02-proposal.md`).
Inputs used: the diff, `02-proposal.md`, `docs/adrs/009-canonical-profile-and-manifest-model.md`,
the proposal's Revision log. No review reports or transcripts opened. Previous complexity
findings: none, so no re-raised or regression labels apply; every entry below is `new`.

verdict: approve

## Checklist walk (the diff, item by item)

1. **New package / module / interface, reuse in repo** — the diff adds zero: no new path, crate,
   module, type, function signature, entry, constant, or `CoreError` variant appears on any `+` line.
   The only structural additions in the whole diff are prose (see finding 1). Nothing to check
   against `tools/`, and nothing to flag for reuse.
2. **Every moving part traces to a Goal** — all 14 hunks restate text that already traced to a Goal
   (the five contracts, computed placement legality, the drift gate, the seed). The one part added by
   the diff that is not design text is the trailing `Editor notes:` line — finding 1.
3. **A 30–50 % smaller design** — not applicable to a language pass; the diff neither adds nor removes
   a mechanism, so the size it leaves is the size r3 left.
4. **New dependencies** — none. The `'(serde-based)'` hunk is quote-style only; the dependency-free
   `crates/core` choice and its deviation from the ADR's parenthetical stay as recorded in
   Revision log → Options considered / register (r0, amended r1, r4). Not re-litigated here.
5. **Defer the second half** — unchanged: `fork`/`merge` and their suite still land with the ADR-012
   caller, `diff` ships in the first slice, `merge`'s rule stays a `patch/v1/spec.md` statement.
   The `Deterministic chain resolution and a shipped `diff`` Goal line now matches §3 verbatim.
6. **Knobs with no consumer / one-implementation abstractions** — unchanged by the diff. The
   no-consumer entries (`fingerprint`, `docked`, `generated-by`, `presets`, `codec-family`) keep the
   consumers and deferrals the Revision log already records; the diff touches none of them.
7. **Value check** — proportionate, and the diff is complexity-neutral: it adds no mechanism, knob,
   limit, error variant, format, or dependency. Two hunks reduce future surface by tightening a claim
   (`is rejected with the shipped InvalidValue` names the actor instead of implying the document is
   the error; `UnresolvedAncestor`/`DuplicateId` wording is untouched). The Context hunk's
   `contracts, numbered 1–5 in the order listed` was verified against every `contract N` cross-reference
   in the body — §1 emphasis, §2 authoring bar and `mouse:*`, §3 id-index/stem/`MAX_CHAIN_DEPTH` unit,
   §4 invocation table, §5 drift mechanism — and all five resolve to the section the numbering names.
   `US-ANSI` in the two reworded hunks matches the Compatibility matrix rows and A17.

## Findings

- [minor][new] Revision log (trailing `Editor notes:` line, after the last log entry) —
  "Editor notes: Chosen design → Encoding rules → Nested arrays for structured lists — … is ambiguous; author to clarify." —
  this is the only moving part the diff adds, and it traces to no Goal: it is a process TODO carried
  inside the design artifact, addressed to an unnamed "author", with no consumer, no gate, and no
  place in the Revision log's decision record, so it survives merge as dead weight. The ambiguity it
  points at is real, which is why the fix is not to keep the note: the sentence "…are each a leading
  id (action, group) followed by its values" names five row families (`stack`, `swap`, `action`,
  `default`, `exclusive`) and two roles (action, group), and `stack`'s leading element is a transform
  name (`"relocate-cluster"`), which is neither, so the parenthetical does not cover the list it
  describes. Fix: in that one bullet, state the leading element per family — `stack` leads with the
  transform name, `swap` and the manifest's `action`/`default` lead with an action id, `exclusive`
  leads with a group id — and delete the `Editor notes:` line. Net surface: same sentence length,
  one line removed.

## FYI (no severity)

- Adversarial posture heading (diff hunk `@@ -215,16 +215,16 @@`) — the rename to "supports Decisions
  3 and 5" narrows a scope claim that the paragraph body outgrows: the same paragraph pins
  `GeometryConflict`, `PlacementNotIsometric` and `PlacementDegraded` (Decision 2 / contract 2)
  alongside the chain and manifest conditions. "Decisions 2, 3 and 5" would match the enumeration.
  Claim accuracy, not complexity — for the seat that owns it.
- Style only: §4's "core's reader" lowercases a name the body otherwise writes `bindforge-core`;
  it matches the Revision log's shorthand, so it is consistent-with-precedent rather than not.
