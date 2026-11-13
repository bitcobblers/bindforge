# performance review r4

verdict: approve

Scope: `reviews/design/editorial-r4.diff` (the r4 editorial language pass over
`02-proposal.md`), read against `02-proposal.md` and
`docs/adrs/009-canonical-profile-and-manifest-model.md`. Inputs used: the
proposal, the ADR, the Revision log, the named diff. No review reports or
transcripts opened.

## Checklist walk (diff-scoped)

1. **Input-path latency budget** — no hunk in this diff adds, removes, or
   re-words any per-event cost. The diff touches Context, Goals, Non-goals,
   Options, Encoding-rule cross-reference wording, the taxonomy `spec.md`
   sentence, the reach-row and Authoring-bar sentences, the Resolution/`resolve`
   bullets, the Id → file paragraph, `core`'s reader sentence, the
   inapplicable-transform sentence, the manifest entry list, the Drift
   mechanism sentence, the README row, the Adversarial-posture heading and
   first paragraph, Test strategy #12, the Revision log, and the appended
   Editor notes. None of these is on a keystroke/event path, and the
   event-path scope is unchanged (Non-goals: "Any input-event API (injection,
   capture)"; Compatibility matrix: "this design reads and writes only
   `schemas/` and `data/` documents; no game config I/O here").
   Hunk 154/155 hyphenates "refused-TOML constructs" — the 1 MiB pre-read cap,
   16-level nesting cap, closed entry sets and bounded messages the paragraph
   carries are byte-for-byte intact.
2. **Structure costs / O(n²)** — the diff's only complexity-bearing sentences
   are untouched: `resolve(index, root, rung)` and chain order (hunk 97/98 adds
   one comma and expands "the chain whole" to "the whole chain"), and
   `MAX_CHAIN_DEPTH = 32 accepts 32 links — a base plus 32 patches, 33
   documents` (context line, unchanged). Test strategy #11's linearity claim
   ("build one `Index` per test binary and share it, so the gate is linear in
   files, not tests") is not in the diff, and the Drift-mechanism and #12 edits
   (hunk 134/135, 163/164) add only the comma before "naming", leaving
   "resolves every profile in `data/profiles/` against the manifest revision it
   pins — a read of the manifest index" — the gate's per-bump cost — as written.
3. **Memory growth** — `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH`, their
   `IndexTooLarge` / `IndexTooDeep` rejections, and A25's product-as-aggregate
   bound appear in the diff's context lines unchanged; hunk 102/103 changes
   "is the shipped `InvalidValue`" to "is rejected with the shipped
   `InvalidValue`" and adds commas only. No bound is softened, dropped, or
   re-scoped.
4. **Startup cost** — the fixed-path-vs-walk split (data-homes rule) survives
   hunk 143/144, which only adds the conjunction "and": the contract tree still
   holds lookup tables read at a fixed path, `data/` still discovered by
   walking. The taxonomy `load` sentence (hunk 68/69) is prose-level
   ("states that ids are added only by manifest-review PRs"), so
   `Taxonomy::load` stays pinned to the one data file.
5. **Caching / invalidation** — the invalidation story (a manifest bump
   re-resolves every pinning profile and re-runs `validate` plus the
   `seeded-from` check, `DuplicateId` keeps a pin unambiguous so the drift
   verdict is not walk-order dependent) is untouched in substance; the
   "walk-order dependent" clause is a context line in hunk 102/103.
6. **Research baseline vs expected numbers** — the baseline quoted in Context
   ("the 1 MiB / 16-level / 32-link adversarial guard") is unmodified by the
   diff, and the two rows that carry the design's own numbers (A24, A25) are
   outside the diff and stay marked `unverified` by the author. No new expected
   number is introduced by this pass, so no unexplained gap is created.

Every checklist item was evaluable within the given scope; none was a gap.

## Findings

None. This diff is language-only: word order, punctuation, quote style,
capitalization (`Core's` → `core's`), and two "ANSI-US"/"ANSI-grid" →
"US-ANSI" normalizations. No hunk changes a cost claim, a bound, a
complexity claim, a read/discovery path, or an invalidation rule, and no
hunk makes a section the performance seat previously approved false.

## FYI (no severity)

- The appended Editor notes line flags the Encoding rules' nested-array
  sentence — "`stack`, `swap`, and the manifest's `action`/`default`/`exclusive`
  rows are each a leading id (action, group) followed by its values" — as
  ambiguous. That is the editorial seat's to resolve; from this seat the
  performance-relevant half of that bullet ("the reader accepts it at nesting
  level 2, inside the 16-level budget (A6)") is untouched and stays bounded,
  so the clarification must keep depth fixed at 2 and must not turn `exclusive`'s
  variadic row into a nesting axis.
- A24 and A25 are the design's own `unverified` rows for format size and
  walked-tree scale. Already recorded as assumptions with the seed-scale
  argument, so not a finding here; worth keeping in mind for the implementation
  round, where the constants get concrete values.
