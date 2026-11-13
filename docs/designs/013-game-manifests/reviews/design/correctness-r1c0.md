# Correctness review r1 (panel pass c0, scope: whole document)

verdict: approve

Inputs used: `docs/designs/013-game-manifests/02-proposal.md`,
`docs/adrs/013-game-manifests.md`, the Revision log inside the proposal
(single entry: `r0: initial proposal; no review findings addressed yet` —
nothing has been deliberately rejected, so no re-report filter applies), and
the external URLs cited by the assumption register. No other reports or
transcripts were opened.

## Checklist evaluation

1. **Assumption register.** A13 and A17 are honestly marked `unverified`, and
   the body leans on them only where an unverified basis is acceptable (the
   row encoding is sound even if A13 were false; A17 only downgrades
   confidence in authored data). The register's repo-code rows (A1–A10, A14,
   A18, A19) carry file-line citations I was not given as review inputs and
   did not open; they are taken at their cited status. The external rows were
   checked directly (see item 2). No unverified assumption is stated as fact
   in the body.
2. **Research-URL spot-check (anti-hallucination gate).** Five cited sources
   fetched: `wiki.teamfortress.com/wiki/Scripting` (live; its "List of key
   names" section really does enumerate `ESC`, F1–F12, `Tab`, arrows,
   `MOUSE1`–`MOUSE5`, `+moveleft`/`+moveright` — supports A9 and the
   null-cancel example); `w3.org/TR/SRI/` (live; alg-token + base64 digest,
   strength-ordered closed token set, §5.3 brute-forcing a small response
   against a known digest, §5.1 unauthenticated-channel warning — supports
   A16); `partner.steamgames.com/doc/features/steam_controller/iga_file`
   (live; "Move"/"Camera" with a localization block supports A13's example;
   the `major_revision`/`minor_revision` flags block supports the Options
   paragraph that rejects the pair); `github.com/ngollan/Elite-Binds` (live;
   README's validator "makes sure the preset name and version correspond to
   the file name, and that no duplicates are present" supports the
   name↔location lesson cited in Options and A13). Also fetched RFC 1035 for
   the body's "DNS-label bound RFC 1035 §2.3.1" claim: §2.3.1 publishes
   "Labels must be 63 characters or less" and §2.3.4 "labels 63 octets or
   less" — the 63-octet slug cap is correctly cited. No dead or
   contradicting source found.
3. **Internal contradictions.** One real contradiction found (finding 1) and
   two enforcement claims that conflict with the design's own stated
   capabilities (findings 2, 3). Entry count ("eleven") is consistent between
   the entry table, `MANIFEST_ENTRIES`, and Test strategy #5; the serialize
   order matches the entry-table order; budgets, caps, and the fixture list
   line up rule-for-rule with Test strategy #1–#4; the location rule's
   case-uniqueness argument matches the lowercase-only grammar.
4. **Missing edge cases.** Empty `actions` is allowed explicitly; absent
   platform (Windows/macOS posture) is handled; name collisions are handled
   by the case-unique slug rule; concurrent modification of data documents is
   not a load-time concept and is out of reach for a file-read loader — the
   bump rule is the design's answer and its enforcement is the subject of
   finding 2. The example manifest's digest decodes to exactly 32 octets
   (44-char base64), so it is grammar-valid.
5. **Interfaces / naming.** One name-ambiguity worth flagging (finding 4):
   the word "version" carries two meanings (the `version` entry vs the ADR's
   "id and version" pin half, which is `revision`).
6. **Does it decide what the ADR asked?** Yes. All five ADR fact groups have
   entries; the registry-join clause, the "key the game will not honour"
   clause, `seeded-from` ids, and the id+version pin are all answered. The
   ADR's "CI re-validates every dependent profile on each manifest bump" is
   deferred to ADR-014 with operands shipped — a deferral consistent with the
   ADR (not re-opened). "Not decided here: … how a manifest is authored or
   reviewed by a human" is deferred verbatim, no contradiction.
7. **Revision rounds.** Revision log holds only the r0 entry; nothing to
   verify as closed.

## Findings

- [minor][new] Delta inventory — "`crates/core/src/placement.rs` and both
  `is_geometry_name` call sites" (Edited) vs "`placement.rs`'s predicate"
  (Unchanged) — the same rename is listed as both an edit and unchanged in
  adjacent lists; an implementer reading only "Unchanged" could skip the
  rename the Edited list and Test strategy #7 require. Fix: qualify the
  Unchanged entry to "`placement.rs`'s predicate *logic* (name change only,
  per the Edited list)".
- [minor][new] Identity, revision, and the series pins — "the loader
  enforces monotonicity forward: a PR whose `revision` did not move while
  its facts did is a review refusal" — the sentence attributes enforcement
  to the loader in the same breath as conceding "core cannot pin a file's
  history"; no shipped or planned code reads file history, and a
  facts-changed/revision-static document loads cleanly. Fix: attribute the
  rule to manifest-PR review alone ("review enforces monotonicity forward;
  the loader enforces only the parse rules above").
- [minor][new] Identity, revision, and the series pins — "the load-time
  join otherwise fails the workspace test" — no artifact this design ships
  or names loads `data/manifests/` in CI: the conformance walk covers
  `schemas/*/v*/` fixtures only, Rollout 6 ships no game data, and the
  cross-file matrix is deferred to ADR-014. The join fires only where a
  caller invokes `load_manifest`, so the guarantee (repeated in the Risks
  row "the load-time join fails any manifest naming an id the registry does
  not back") is currently unbacked for real manifests. Fix: state who runs
  the load — e.g. extend Rollout 6: "the first manifest PR adds a
  `load_manifest` test over its `data/manifests/` entry", and qualify the
  Risks mitigation identically.
- [minor][new] Goals / Non-goals + Chosen design → Identity — "the pin
  ADR-014 spells" for the ADR's "manifest id **and version**" pin, while the
  document's `version` entry means the grammar major (`v1`) — the same word
  names two different values in the same design; a pin author could spell
  `remnant-2@v1` instead of `remnant-2@3`. Fix: one sentence in Identity:
  "the ADR's 'version' in 'id and version pin' is this document's
  `revision`, not its `version` entry (the grammar major)."
- [nit][new] Options considered — "(`#` comment-stripping happens outside
  quotes, A19)" — A19's pinned basis is that the reference reader strips
  `#` only at line start or after a complete entry; neither statement says
  the operative fact that a `#` inside a quoted array element is data,
  which is what Rules' "Names may contain spaces, `#`" actually relies on.
  Fix: align the parenthetical with the A19 wording.
- [nit][new] The contract: `schemas/manifests/v1/` — the example's
  `docked-actions = ["ui.inventory Tab"]` row cannot parse under the
  design's own rules today (`Tab` is outside the closed vocabulary;
  `ui.inventory` is not yet in the registry) — the passage is annotated as
  not shippable, but the document's one full illustrative manifest is one
  that `load_manifest` would refuse twice. Fix: make the example loadable
  with current-vocabulary rows and keep the deferred-vocabulary illustration
  in the parenthetical.

## FYI (no severity)

- The `fingerprint` entry ships grammar-only with "nothing consumes the
  value in this run"; the value of shipping a guard-less digest now rather
  than with ADR-017 is a design-taste question (complexity seat).
- The example manifest is decorative data that will never be validated
  against a real game; consistency of its `codec = "unreal-engine-5"`
  remnant-2 claim is outside what any fixture pins (compatibility seat).
