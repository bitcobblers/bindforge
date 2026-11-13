# performance review r1

verdict: request-changes

Scope: whole document (full-panel pass c0). Inputs: `02-proposal.md` (incl. its
Revision log), `docs/adrs/014-profiles-as-chains.md`. Revision log reads "r1:
none yet", so every finding here is new. Checklist item 6 evaluated against the
baseline the proposal itself quotes (A20/A21); `01-research.md` is outside this
seat's named inputs, so the research's own numbers are read only as quoted.

## Findings

- [major][new] Cross-file loading: `load_chain` in `chain_ops.rs` (step 2) +
  Resolution ("The composite cost is bounded and now stated") —
  "Walk `derives` nearest-first: the ancestor `<id>.toml` is looked up
  *directly in that same directory*" and "a resolution reads at most
  `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB) across at most 32 documents".
  The load cost is file opens, and the "at most 32 documents" half of the
  composite bound rests on a linearity assumption the document never states.
  `derives` is inherited from v1 as an *array of ancestors, nearest first* (A1,
  A15), the model keeps `derives: Vec<String>`, and A6 records that "nothing
  else bounds array sizes" — so one 1 MiB document can carry tens of thousands
  of ancestor ids. Nothing says whether the walk follows `derives[0]` only,
  follows every element, or refuses the extra ones. The `Chain { links: Vec }`
  comment ("deepest base first … head last") and the per-position `kind` rule
  imply a single path, but the design also requires `spec.md` to state the
  load-level rules "so a non-Rust implementer can conform to the whole chain
  contract" — an implementer reading "an array of ancestors, nearest first"
  will traverse all of them, and the per-file caps then evade the composite
  bound through file references, the exact failure mode the design's own
  prior-art finding names ("TF2 shows per-file caps evaded once files reference
  files", A20). Fix: in `load_chain` step 2, add one clause naming the shape of
  the walk — the walk follows `derives[0]` only; any further element is carried
  as data and never traversed (or refused, if that is the intent) — and restate
  the composite as bounded in file opens: "`≤ MAX_CHAIN_DEPTH` documents opened
  per resolution, regardless of `derives` array length". That replaces the
  implicit reading, not the caps; no new mechanism.
- [minor][new] Options ("Composite chain budget") + Chosen design → Delta
  elements ("the cap the manifest row cap already prices") + Resolution —
  "each ≤ 256 delta elements per list — the per-document budgets plus the link
  bound, with no new aggregate cap". The statement that the cost "is bounded
  and now stated" stops at bytes, link count and per-list count; it never
  states the derived totals that actually drive `resolve`'s work (≤ 32 links ×
  3 lists × 256 = 24,576 delta elements per chain; resolved bindings ≤ the
  joined manifest's action-row count), and the one number it leans on — the
  "manifest row cap" — is named but never quantified anywhere in the document.
  That omission is load-bearing for the chain operations, not just for
  bookkeeping: `diff` emits "a `set` for every action whose binding differs, a
  `clear` for every action `b` leaves unbound" and `merge` emits one patch over
  "every action either touches", so their outputs are bounded by the action-row
  count — cap-safe only while that row cap is ≤ `MAX_PATCH_DELTA_ELEMENTS`. The
  Risks table prices raising the delta cap but never the opposite direction
  (raising the manifest row cap), which would let `diff`/`merge` emit documents
  that fail the design's own per-list parse rule, with no refusal rule stated
  and no test in Test strategy #5 for a cap-sized input. Fix: name the manifest
  row-cap constant with its value at first use, add the derived total to the
  composite sentence, and add one clause to the `diff`/`merge` bullets saying
  their output obeys `MAX_PATCH_DELTA_ELEMENTS` because the joined action table
  is capped — with a refusal if that ever stops holding. No new aggregate cap.
- [minor][new] Resolution ("Load-time joins") + determinism rules —
  "every action id in any `set`/`clear`/`swap` must be a row of that manifest's
  `actions` table" and "`set` replaces any binding for that action (last writer
  wins)". The cost of the load path depends on representation, and the document
  fixes only the ordering, which pushes toward arrays: the action table is an
  array whose "row order is data" (rule 5), deltas apply "sequentially in array
  order", and `fork` also "refused if it collides with any link id in the
  chain". Array membership for the action join, for "any binding for that
  action", and for `swap`'s two lookups is O(deltas × rows) — worst case ~6M
  comparisons per resolution, bounded but quadratic in the caps. Fix: one
  sentence in `resolve`'s contract: action-row membership and the
  action→binding layout are held in a lookup built once per `load_chain`/
  `resolve` call; the manifest's `actions` array order is used only to order
  output, never for membership. This is an implementation statement, not a new
  mechanism, and it does not touch determinism rule 5.
- [minor][new] Cross-file loading: `load_chain` + Resolution → Load-time joins
  ("Manifest identity") — "a file whose `id` entry disagrees with its name is
  `InvalidValue` — the location rule, one rule". The design gives an
  invalidation rule for exactly one side of the resolution input — a bumped
  manifest revision is caught by pin equality — and none for the other. Because
  id↔filename equality is a contract rule and links are looked up by id in one
  directory, a consumer (ADR-017 storage, ADR-018 hub) that identifies a chain
  by its head id and caches the resolved layout goes stale silently whenever a
  link document is edited in place under an unchanged pin and unchanged ids.
  Deferring caching to 017/018 is fine; leaving the chain's identity definition
  open is what makes a fast consumer cache wrong rather than merely absent.
  Fix: one line in the `load_chain` section (mirrored in `spec.md`, since the
  spec carries the load rules): a chain's identity is the contents of its link
  documents — id equality pins the lookup, not the value — and pin equality
  invalidates only against manifest changes.

## FYI (no severity)

- Checklist item 1 (event path): this stage ships no binary and no keystroke or
  event path, so there is no per-event allocation, rebuild or synchronous IO to
  police; `load_chain`/`resolve` are load-time only, and the storage/CLI paths
  that could put resolution on an event path are deferred (ADR-017/018). The
  document therefore has no latency budget of any kind — only adversarial size
  maxima — which is why the composite statement above is the one to fix.
- Checklist item 3 otherwise passes: `id`/`derives` ≤ 63 octets,
  `generated-by` ≤ 128 octets with control chars refused, per-list delta caps,
  `MAX_CHAIN_DEPTH` reused unchanged, iterative walk refusing before descent —
  every string and structure the design adds is bounded.
- For the correctness seat: `MAX_GENERATED_BY_OCTETS` appears in Delta
  inventory and Rollout 2 but not in the entry table row, which states the
  value only as prose ("1–128 octets"); the composite/budget paragraph in
  `schemas/README.md` names it, so this reads as a naming-consistency point
  rather than a cost question.
- `diff`'s round-trip property test over corpus pairs costs two resolutions
  per pair; that is test-suite time, not a shipped cost.
