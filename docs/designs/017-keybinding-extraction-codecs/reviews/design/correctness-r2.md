# correctness review r2

verdict: approve

Scope: the editorial round-2 diff
(`docs/designs/017-keybinding-extraction-codecs/reviews/design/editorial-r2.diff`)
— five hunks, all language-only. Inputs: `02-proposal.md`, `docs/adrs/017-keybinding-extraction-codecs.md`,
the proposal's Revision log, and the diff. Previous correctness findings: none.

## Findings

None. No hunk changes a claim, a bound, an interface name or an ordering rule,
and none makes a section it touches — or a section those lines cite — false.

Hunk-by-hunk check (all clean):

1. §The codec face — `"at most" → "of at most"`. Grammar only. The claim in the
   changed sentence, `[a-z][a-z0-9-]*` bounded by `bindforge_core::MAX_SLUG_OCTETS`,
   still verifies: the constant exists at `crates/core/src/placement.rs:83`
   (`= 128`) and is re-exported at `crates/core/src/lib.rs:120`, so the path the
   sentence names is real and the bound is the one A6 records.
2. §Read discipline — `"ceilings the opens" → "caps the opens"`. Word choice only;
   it now agrees with the same sentence's "caps a tree's reads" and with the
   table's `MAX_CANDIDATE_FILES` row, and the arithmetic it states still holds:
   `(16 + 1) × 32 × 1 048 576` = 544 MiB, matching the `17 × 32` product.
3. Test strategy #3 — `"9 and 8 digit" → "9- and 8-digit"`. Hyphenation only; the
   count the sentence pins (9 + 8 = 17 trees, first 16 kept, 17th past
   `MAX_USER_DIRS_PER_GAME = 16`) is unchanged and agrees with §Read discipline's
   table and §Config-path resolvers' "at most `MAX_USER_DIRS_PER_GAME` … across
   all roots".
4. Test strategy #6 — `"other than the fixtures' others" → "other than those the
   fixtures use"`. The garbled comparative is replaced by the same proposition:
   an unlisted platform directory name is still located, no constant assumed —
   which is what §The first family states ("No platform-directory constant is
   assumed (A15)") and what A15's unverified status requires.
5. Test strategy #16 — `"and that the only Cargo.lock delta…" → "and pins that…"`.
   The verb now matches the sentence's own subject (the probe *pins* the delta),
   and the pinned fact agrees with §`crates/codecs`'s dependency list and A12:
   `discovery`, `bindforge-core` and `serde_json` are already packages in
   `Cargo.lock` (lines 141, 56, 317) and already dependencies of
   `apps/discovery-cli/Cargo.toml`, so a new package stanza plus the one
   `discovery-cli` edge is the whole delta.

Checklist disposition: (1) assumption register — no `verified` row's basis is
touched by the diff; the two rows the changed sentences lean on (A6, A12) were
re-checked against the tree and hold. (2) research-URL spot-check — not
applicable: the diff adds and changes no citation, and `01-research.md` was not
in this pass's allowed inputs. (3) internal contradictions — none introduced;
the five touched passages still agree with §`bindings/v1`'s bounds, §Determinism
and §The CLI face. (4) edge cases — no edge-case text changed. (5) interface
names — `MAX_SLUG_OCTETS`, `MAX_USER_DIRS_PER_GAME`, `MAX_CANDIDATE_FILES`,
`MAX_NATIVE_NAME_OCTETS`, `MAX_LOCATE_DEPTH`, `bindforge-codecs`,
`discovery-cli` appear under one name and one shape each. (6) does the design
decide what ADR-017 asked — unaffected by language-only edits; the divergences
it records remain the Revision log's nine documented readings. (7) prior-round
blocker/major closure — the caller gives no prior correctness findings for this
phase, so nothing to re-verify. Revision log: no entry is contradicted by the
diff; entries R1 (§Read discipline budgets, §Config-path resolvers) and the
initial §Codec family id entry are exactly the passages hunks 1–3 touch, and the
reworded text still states each decision as recorded. Deferred section: nothing
re-opened — the diff does not touch host-config, routing-data or vocabulary
deferrals, and none of them contradicts ADR-017's Decision.

## FYI (no severity)

- §Codec family id's "`data/manifests/…/manifest.toml`" is a forward-looking
  reference (A21 records that `data/manifests/` ships only a README today); the
  sentence is about legality of the id, not about a file that exists — wording
  only relevant to the complexity seat, and unchanged by this diff.
