# correctness review r1c4 (dissent pass on my r1 findings, after revision r4)

verdict: approve

Scope as directed: `git diff 1e8f240 e1e63cb` (the `02-proposal.md` blobs), then the
full current document. Inputs: `02-proposal.md`, `docs/adrs/026-registry-service.md`,
the Revision log, my r1 findings. Code checked read-only to verify what the changed
text claims about the shipped code.

## Closures checked (my two open findings)

1. **[major, r1] outbound request target unspecified — CLOSED for real, not reworded.**
   The revision adds the shipped request builder to the copy list (§`apps/registry`:
   "six invariants are **copied** … the response reader and the shipped request
   builder `percent_encode`/`candidates_url` (#7)"), states the outbound spelling in
   the policy step (§Caching mode step 4: "target built from the *decoded* slug via
   the copied `percent_encode` (A19)"), pins it in #7 ("a space- or %-bearing `slug`
   reaches upstream only percent-encoded from the decoded key (A19)"), and follows the
   count through §Deferred ("six copied invariant blocks") and the Revision log's
   `r1: apps/registry dependency line` entry. Verified the basis, not just the words:
   `apps/discovery-cli/src/catalog.rs:174` `percent_encode` maps every byte outside
   the RFC 3986 unreserved set to `%XX` (space → `%20`, `%` → `%25`, `/` → `%2F`),
   `catalog.rs:190` `candidates_url` composes `<base>/v1/candidates?slug=<enc>`, and
   `catalog.rs:435-438` sends exactly that tail as `GET {target} HTTP/1.1` — so the
   bytes the upstream percent-decodes are byte-identical to the key this instance
   asked about, which is the round trip §The serving face step 1 ("Percent-decode
   `slug` **first**") requires. The widened alphabet really does admit the keys at
   issue: `crates/discovery/src/game.rs:84` `spells_lookup_key` requires only a
   closed-set token and a non-empty external id, and step 1's extra refusals are a
   further separator, `.`/`..`, control bytes, invalid UTF-8 — a space or `%` is
   none of them, so the fix was load-bearing rather than decorative. The copy
   premise also holds: `apps/discovery-cli/Cargo.toml` declares no `[lib]`, so the
   builder cannot be shared, only copied.
2. **[minor, r1] checked buffer ≠ renamed buffer — CLOSED.** The bump now precedes the
   check: §The admin face and auth tests "the model with the upsert and the next
   `revision` value … serialized whole — the exact buffer renamed", §Budgets checks
   "the exact post-`revision`-bump buffer an admin write renames into place", and #10
   pins "an upsert whose serialized corpus with its bumped `revision`". The
   9→10/99→100 one-octet overflow that would let an accepted upsert persist a corpus
   its own loader refuses is gone, so both absolutes this revision wrote (§Budgets "no
   accepted mutation can persist a corpus its own loader refuses"; §Concurrency "a
   restart cannot lose an accepted mutation") now hold. `DELETE` cannot reintroduce
   it: removing a record strictly shrinks the document, and §Concurrency's
   "serializes the new model, writes the fixed tmp name, renames it into place, and
   swaps the model in, **all under the one write lock**" makes the admin buffer and
   the swapped-in model the same object. No contradiction between the reworded bullet
   and §Concurrency, #10 or #11.

Nothing else in the changed text falsifies a section I approved: the "five"→"six"
edits are complete (no stale count survives anywhere in the document), the
enablement bullet's "not one of the copied blocks" stays consistent with the r1 entry
that made it the registry's own judgment, and the Revision log's r4 lines record both
closures and no rejected finding of mine that I can show is unsound.

## Findings

- [nit][new] §Test strategy #10 (**Admin mutations**) — "an upsert whose serialized
  corpus with its bumped `revision` would pass `MAX_CATALOG_DOCUMENT_OCTETS` is that
  named `422`" — this line was rewritten this round and "would pass <a budget>"
  reads two ways: "exceed it" (the intent, matching §The admin face's "one over
  `MAX_CATALOG_DOCUMENT_OCTETS` … is the named `422`" and §Budgets) or "satisfy it",
  which inverts the rule; every other site in the document uses "over" for the
  refusal, so this is the one spot where a test author working from the item alone
  could pin the opposite of the contract. Fix: "would exceed
  `MAX_CATALOG_DOCUMENT_OCTETS`".

## Checklist notes

- Assumption register: the diff touches no register row. The one row the changed text
  leans on, A19, is `verified` and I re-verified it against the cited code above.
- Research-URL spot-check (anti-hallucination gate): not applicable to this pass —
  the diff adds or changes no cited source; A20/A21 remain the document's own
  `unverified` rows, correctly worded as reversible standards-conformance choices
  whose binding obligations are `026-C4`/`026-C5`'s own unit texts.
- Interfaces (§item 5): the copied builder is named by the same identifiers
  (`percent_encode`/`candidates_url`) in §`apps/registry`, step 4, #7 and the
  Revision log; the document cap is one constant at both ends of a document's life.
- Deferred: re-opened nothing. The six-block lift deferral does not contradict the
  ADR — `026-C1` fixes `apps/registry` as its own std-only package and nothing in
  ADR-026 requires the client's blocks to be shared.
