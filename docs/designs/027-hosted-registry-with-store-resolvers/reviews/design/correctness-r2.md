# Correctness review r2 (editorial confirmation pass, scope: `reviews/design/editorial-r2.diff`)

verdict: approve

Scope honored: the 14 hunks of `editorial-r2.diff`, `02-proposal.md` as it now
stands, `docs/adrs/027-hosted-registry-with-store-resolvers.md`, and the
proposal's Revision log. No other run record opened; no previous-round findings
were carried to this seat for the editorial phase, so nothing was re-raised and
no regression was possible against my own earlier text.

## Findings

None. Every hunk is a language-only change; none moved technical meaning, and
none made a section it touches false.

Hunk-by-hunk confirmation (section — change — meaning check):

- Goals (Issued entries) — "bookkeeping that deleted ids never return" →
  "bookkeeping ensuring that deleted ids never return" — matches the
  retired-ids design in §Issued entries verbatim in substance.
- §Hosted mode (`--upstream` bullet) — hyphenation reflow of "named-refusal
  family" — the term and the quoted refusal line are unchanged, and §Test
  strategy #8's "the retirement refusal (quoting the flag, naming ADR-027)"
  still matches it.
- §Hosted mode — "New flags:" → "New flag: `--no-steam`" — correct: `--no-steam`
  is the only new flag this design introduces (checked the shipped parser in
  `apps/registry/src/main.rs`: `--root`, `--corpus`, `--upstream`,
  `--refresh-interval` exist; no `--no-steam`), and `027-C3` names no other.
- §Hosted mode (`query` bullet) — comma before "including" — the bullet's
  content (load-or-pull, no refresh thread, provenance words, `(no candidates)`
  with provenance `none`) matches the shipped `query_command` and its
  `(no candidates)` line, and §Test strategy #15.
- §Hosted mode (health bullet) — comma before "and
  `steam_snapshot.fetched_at` is their successor" — health set still the
  enumerated `{status, mode, corpus_entries, steam_snapshot}`, still consistent
  with #9.
- §The store pull (`fetched_at`) — "overlapping only re-fetches" → "an overlap
  only re-fetches" — same claim about the appid merge absorbing re-fetched
  items; still consistent with the pre-pull wall-clock stamp rule.
- §The store pull (refresh thread) — comma after "tmp-and-rename" — sequence
  unchanged (rename, then publish under the snapshot `RwLock`), still consistent
  with §Concurrency's "the refresh thread takes only the snapshot lock".
- §Issued entries (retired ids) — "it is ordered insert" → "it is an ordered
  insert" — still consistent with the document's byte order and binary-search
  membership, and with #5.
- §Budgets (memory sentence) — "and a delta merge the old snapshot" → "and a
  delta merge **holds** the old snapshot" — supplies the missing verb only; the
  four items named are the ones the r2 Revision-log entry already committed to
  (old model, merged model, serialize buffer, page body) and they agree with the
  one-linear-merge-plus-one-index-build rule in §The store pull. The adjacent
  "144 MiB" sum still holds against the shipped constant
  `MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216` (verified in
  `apps/registry/src/constants.rs`) plus the design's own 64/32/32.
- §Test strategy #4 — serial comma — the persist-failure assertion set
  (model, `revision`, file untouched, nothing served) is unchanged and still
  matches §Issued entries' Ordering and #12's zero-rewrite case.
- §Test strategy #6 (transport assurance) — comma after "Without it" — the
  adversarial-argument it sets up is unchanged and still matches the seam
  description and the Revision log's r1→r2 entry.
- §Test strategy #8 — "retirement-refusal" → "retirement refusal" — same
  posture, same quoted-flag/naming-ADR-027 content, still matches §Hosted mode.
- §Rollout 4 — "the measured issued-document rewrite at 100k and 200k records
  recorded in §Budgets" → "the issued-document rewrite measured at 100k and 200k
  records, recorded in §Budgets" — the deliverable and its destination are
  unchanged, and §Budgets' closing sentence says exactly that.
- §Deferred (CLI self-resolution) — "the source document's Consequences stands
  outside its ledger" → "leaves outside its ledger" — now grammatical and still
  faithful to the ADR's Consequences ("a separate small amendment proposed
  (outside this ADR's ledger) letting the CLI self-resolve `local/<dir>` keys");
  it contradicts no ADR clause, so the deferral is not re-opened.

Cross-section interface check after the diff: refusal names (`registry data
directory is locked`, `mint identity held`, `mint identity over bound`, `mint
cap reached`, `retired id list full`, `steam snapshot stale`, `no steam
snapshot`, `steam resolver inactive`, `epic resolver inactive`, `steam pull
deadline`) each appear under one spelling across §Hosted mode, §The hosted
lookup policy, §The store pull, §Issued entries, §Budgets and §Test strategy;
no thing appears under two names or shapes. The mint's `6 + 122 = 128`
arithmetic still rests on verified constants: `MAX_SLUG_OCTETS = 128`
(`crates/core/src/placement.rs`), `MAX_DISCOVERY_SLUG_OCTETS = 128` (same file),
`MAX_NATIVE_NAME_OCTETS = 128` (`crates/core/src/manifest.rs`).

## Checklist applicability

1. Assumption register — no register row changed in this diff; the rows behind
   the touched passages (A2, A10, A13, A15, A16) are unchanged from the text I
   verified, and I re-checked the constants the touched arithmetic names.
2. Research-URL spot-check — not applicable to this pass: no citation, URL or
   `verified`/`unverified` status is added, removed or reworded by the diff. No
   evidence gap arises from the changed text, so none blocks.
3. Internal contradictions — none introduced; all 14 hunks traced above.
4. Missing edge cases — the diff adds no behavior, so no new edge case.
5. Interfaces — checked, no drift.
6. Decides what the source asked — the ADR's `027-C1`–`027-C6`, `027-L1`–`L5`,
   `027-B1`–`B4`, `027-R1`–`R3` and `027-S1` all remain mapped to sections; the
   diff touches none of those mappings.
7. Previous round's blocker/major closures — none carried to this seat for the
   editorial pass; the Revision log shows each earlier decision closed with a
   current entry, and the diff reopens none of them (in particular it does not
   revive the deleted `dropped_appids` subsystem, `024-L2` ordinals, or a
   "pure-Rust" TLS claim).

## FYI (no severity)

- §Budgets' memory sentence now asserts the delta merge holds old snapshot,
  merged model, serialize buffer *and* the current page body at once. If pages
  are reduced into "the pulled changes" as they arrive and their bodies dropped,
  the true peak is the larger of the pull phase and the merge phase, not the
  four-way sum — the figure is conservative (over-stated, never under-stated),
  so it is safe, but the performance seat may want the clause narrowed to name
  which phase peaks.
- §Goals still says "`serve --root --corpus` plus resolver flags" (echoing the
  ADR's plural "resolver flags of `027-C3`") while §Hosted mode now says "New
  flag" for the single `--no-steam`; language-only, the editorial seat's call.
