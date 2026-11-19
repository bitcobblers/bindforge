# Complexity review r1 (c5 ratification)

verdict: approve

Scope as instructed: `git diff 1e8f240 e1e63cb` over `02-proposal.md` (the
post-approval revisions), then the full current document, plus the ADR, the
in-document Revision log, and my r1 findings. Nothing outside those inputs was
opened.

## What the revisions did to what I approved

- **My r1 minor (buffer checked ≠ buffer written) is closed, and closed by the
  fix I named, at no complexity cost.** §The admin face and auth now reads "the
  model with the upsert and the next `revision` value is serialized whole — the
  exact buffer renamed", §Budgets now says "checked against the exact
  post-`revision`-bump buffer an admin write renames into place", and the r4
  Revision-log line states the same. One buffer, one check, one write; no new
  mechanism, no new pass over the corpus, and the two clauses no longer disagree
  about whether the bump is inside the checked serialization. Nothing here
  enlarges the moving-parts set the Goals pay for.
- **The new sixth copied invariant does not break what I approved.** The copy
  list (five → six), §Enablement's "not one of the copied blocks", §Deferred's
  "six copied invariant blocks", and the r1 dependency-log line are mutually
  consistent, and the list still counts six by groups (response reader, request
  builder, endpoint gate, `game_id_is_valid`, `<data>` rule, tmp-and-rename),
  each still pinned by the test item cited for it. Nothing was dropped from the
  five I approved.
- Reuse check (checklist item 1): the copied pair exists as claimed —
  `apps/discovery-cli/src/catalog.rs:174` (`percent_encode`) and `:190`
  (`candidates_url`), and the crate is bin-only (`main.rs`/`catalog.rs`/
  `matches.rs`, no `lib.rs`). `percent_encode` is not even `pub`, so no
  visibility tweak to the shipped binary would have avoided the copy; the
  smaller alternative is exactly the shared-crate lift §Deferred already names.
  `crates/discovery/Cargo.toml:39` carries `winreg = { workspace = true }`, as
  the text claims.
- Checklist items 2–6: no new package, module, dependency or config knob in this
  diff; no abstraction with one implementation added (a copy is the opposite
  trade, and its consolidation is deferred by name); the second half is already
  staged thin across `T-004`…`T-008` with no product code this run. Item 7, value
  check: the design is still proportionate — six small duplicated blocks in a
  contract-only run, against a deferred single-copy lift, is the cheaper of the
  two shapes available to it.

## Findings

- [nit][re-raised] §Test strategy #10 — "an upsert whose serialized corpus with
  its bumped `revision` would pass `MAX_CATALOG_DOCUMENT_OCTETS` is that named
  `422`" — the revision this round fixed the Revision-log half of my r1 nit (the
  budget line now says "an over-cap upsert the named `422`"), but this sentence
  was edited — "with its bumped `revision`" was inserted — and kept the inverted
  word: "would pass" reads as "satisfies the cap", pinning the acceptance
  boundary backwards, against the document's own refusal voice (#1 "document
  **over** `MAX_CATALOG_DOCUMENT_OCTETS`", #5 "load **over** `026-B2` is
  refused"). Fix: "would exceed `MAX_CATALOG_DOCUMENT_OCTETS`" — the same one-word
  change I proposed in r1, now applicable to this line alone.
- [nit][re-raised] §Deferred — "Lifting the six copied invariant blocks
  (§`apps/registry`) … into a shared crate so both binaries call one copy, which
  also gives `Source`/`spells_lookup_key` that shared home" — this round's line
  only renumbered five → six; the `winreg` tail I asked for is still missing, so
  §`apps/registry`'s unchanged promise that the "transitive Windows-only
  `winreg` edge [is] taken for that predicate and **retired by the deferred lift
  below**", and the r1 dependency-log line's "its `winreg` transitive retired by
  the deferred lift (§Deferred)", both still point at text that no longer makes
  the claim. Fix: restore the retired-transitive clause in the same §Deferred
  sentence (preferred; the claim is true — `crates/discovery/Cargo.toml:39`), or
  drop "and retired by the deferred lift below" from §`apps/registry`.

No blocker or major findings; the two open items are wording nits that do not
gate implementation.

## FYI (no severity)

- §Caching mode step 4's outbound encoding is a `025-C4` request-target contract
  question as much as a complexity one; the mechanism itself (re-encode from the
  decoded key rather than forwarding the raw query bytes) is the shape the
  shipped client uses, so the compatibility seat may want to confirm the
  byte-identity claim on the wire for a key carrying a literal `%`.
