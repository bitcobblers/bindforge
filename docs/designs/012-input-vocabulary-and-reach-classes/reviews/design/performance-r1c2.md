# performance review r1c2 (ratification of the r1 revisions)

verdict: approve

## Scope

Only the diff `17aee59 → b08752d` of `02-proposal.md`, read together with the
full current document and the ADR. Checked against the performance checklist
and my own r1 findings only.

## Findings

None open.

## Closure check on my r1 finding

- [nit — closed] Chosen design — "The model: `crates/core/src/input.rs` and
  the reach edits" — my r1 finding flagged the parsed pair as
  `class: String` while the Options section settles ordinality as "integer
  rank arithmetic over table data", forcing a per-call scan/string-compare of
  the 5-entry classes array. The revision lands exactly the shape my fix
  asked for: the pair is now `input: InputId`, `class: ReachClass`, and the
  revision log records it ("`InputGrade.class` is `ReachClass`"). Since the
  canonical classes list is a parse-level rule pinned to one fixed five-id
  order ("the loader compares no files (the ordinality's source is one rule
  per document)"), the rank mapping is static and resolved at parse time;
  the per-call string lookup is gone. The Options claim now holds as
  written. Nothing to re-raise.

## What the revised text does to the checklist (all clear)

1. **Event-path cost** — nothing in the diff puts work on a keystroke/event
   path; the predicate stays "a pure function over loaded data", loads stay
   one-time under the existing budget.
2. **Structure costs** — the predicate remains O(moves) over bounded data
   (48 graded ids, one geometry file); "coordinates `0..=999`" plus "signed
   integers over the bounded coordinates" bound the arithmetic without
   adding cost. No new cross-product.
3. **Memory growth** — `keys: Vec<InputGrade>` is empty for v1 documents and
   bounded at 48 entries per hand for v2; `PlacementCheck` carries per-move
   deltas only in the non-agreement (illegal) case, bounded by the move
   count.
4. **Startup cost** — the version-segment equality check now applies to
   *every* reach table at load, v1 included: one string comparison per
   document at load, "v1-on-v1 passes trivially". Negligible, load-time not
   event-time.
5. **Caching** — no new cache; invalidation unchanged.
6. **Research baseline** — A8 ("No external benchmark") already states the
   absence of published numbers; the diff introduces no performance claim
   that gaps the research.

FYI (no severity): the `load_geometry` pre-join grammar check on `name` is a
path-traversal/safety concern, not a performance one — the grammar match is
negligible and strictly avoids a wasted file read; security/correctness
seats own the rule itself.
