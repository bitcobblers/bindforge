# correctness review r4

verdict: approve

Scope: `reviews/design/editorial-r4.diff` (the r4 editorial pass over `02-proposal.md`).
Inputs used: `02-proposal.md`, `docs/adrs/009-canonical-profile-and-manifest-model.md`, the
proposal's Revision log, the named diff, and this seat's prior findings (none — the editorial
phase is a single language pass per round).

## What the diff was checked for

Every changed line in the diff was read against the section it touches:

- **Context five-contract sentence** — the new "numbered 1–5 in the order listed" reading matches
  ADR-009's section order (taxonomy, input vocabulary/reach, chains, transforms, manifests) and the
  Chosen design's section order, so every `contract N` cross-reference in the body
  (`(contract 2)` authoring bar, `(contract 3)` index limits and stem rule, `(contract 4)`
  `apply_policy`'s `Err`, "the emphasis fact, contract 1") resolves without inference.
- **"US-ANSI grid"** — the two renames (Options, reach row) collapse the last two variant spellings;
  the document now uses `US-ANSI` in Options, §2, and the compatibility matrix, consistent with
  ADR-009's "ANSI geometry" / "ANSI-US only at first". No same-thing-two-names residue (checklist 5).
- **`rung = chain.len()` takes the whole chain** — agrees with Test strategy #7 and with the
  `MAX_CHAIN_DEPTH` unit stated in §3 and Risks.
- **"rejected with the shipped `InvalidValue` naming the tree"** — agrees with Test strategy #9's
  planted-document case and with Revision log (r3).
- **lowercase "core's reader"** — agrees with the Revision log's "`apply_policy`, not core's reader"
  and with the crate names used in prose elsewhere.
- **`'(serde-based)'`** — matches ADR-009's Decision text verbatim; the deviation is disclosed.
- **"every grading change carries a named reviewer (`git log`)"** — agrees with Rollout 6.
- **`(supports Decisions 3 and 5)`**, "cannot under-claim anything", "reports itself and no-ops",
  the added conjunctions and commas — meaning-preserving; no interface, data-flow, or edge-case
  statement changed.

Anti-hallucination spot-check (checklist 2) over the citations the changed sentences rely on:
A16's replacement citation `https://en.wikipedia.org/wiki/Home_row` is live and states the QWERTY
home keys are `ASDF` / `JKL;` (the r4 register fix); `https://toml.io/en/v1.1.0` confirms the
bare-key charset `A-Za-z0-9_-`, quoted keys, dotted keys as table-creating syntax, and nested arrays
(A2, A12, A13); `https://github.com/libsdl-org/SDL/blob/main/include/SDL3/SDL_scancode.h` is live and
confirms usage page 0x07 scancodes plus explicit ANSI/ISO key-count differences (A17). Nothing dead
or contradicting.

## Findings

- [minor][new] Chosen design → Encoding rules → Nested arrays for structured lists — "`stack`, `swap`, and the manifest's `action`/`default`/`exclusive` rows are each a leading id (action, group) followed by its values" — the bullet's parenthetical enumerates the leading element as an action id or a group id and counts `stack` among those rows, but §4 Transforms pins the opposite shape for it: "`stack` — an ordered array of invocations, each a non-empty array of strings whose first element is the transform name", a name from the closed invocation table, and Test strategy #10 rejects an unknown transform name as `Err(InvalidValue)`. So the canonical encoding rule under-describes one of the five row kinds it lists (checklist 3/5), and the diff's trailing editor note flags this same passage. This is not a regression of anything this seat previously approved — the bullet is context in the diff, and the mismatch is confirmed, not newly introduced, by the note. Fix: name the third kind in the existing parenthetical, replacing "(action, group)" with "(an action id, a group id, or, for `stack`, a transform name from the closed invocation table)" — no new mechanism, and §4 stays the authority for arity.

## FYI (no severity)

- Non-goals — "Anything the Deferred / out of scope section assigns to a named owner — ADR-012's codecs and CLI, ADR-011's preset set, ADR-014's hub, and layout generation": layout generation has no named owner in Deferred (its entry reads "no follow-up yet"), so the inserted "and" makes it read as a named-owner item. Pre-existing in the list; wording only, editorial seat.
- The diff appends a trailing "Editor notes: … author to clarify." line to the proposal. That is a process artifact, not design content: the revise round should resolve the flagged passage (the minor finding above) and delete the line, so the shipped document carries no open instruction to the author. The Revision log already records each r4 decision.
- The ADR's "(serde-based)" deviation, the fingerprint recipe (A21), the budget estimate (A24) and the index limits (A25) remain disclosed deviations/unverified rows with their basis stated; they are consistent with the ADR's Decision and Deferred owners, so they are not findings.
