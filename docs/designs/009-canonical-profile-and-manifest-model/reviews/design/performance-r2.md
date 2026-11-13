# performance review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`) as the confirmation pass. Inputs used: `02-proposal.md`, `docs/adrs/009-canonical-profile-and-manifest-model.md`, the Revision log inside the proposal. Prior performance findings: none (the editorial phase is one language pass per round).

## Checklist, applied to the diff

1. **Input-path latency budget** — the diff touches no event-path text; the design keeps input-event work out of scope (Non-goals: "Any input-event API (injection, capture)"). Nothing added allocation, table rebuild, or synchronous IO on a keystroke path. Clean.
2. **Structure costs / O(n²)** — the only cost-bearing text in the changed lines is the Adversarial posture paragraph; the performance-bearing wording is byte-identical across the `-`/`+` pair (`The per-document budget is unchanged and `MAX_CHAIN_DEPTH` is reused as is; the tree walk adds the two public constants `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH``), with the only deltas being `§3`/`§4` → `contract 3`/`contract 4` and an inserted "are". The renamed cross-references resolve to the intended sections (contract 3 = Profiles are chains → Id → file, where the index limits and the stem rule live; contract 4 = Transforms, where `apply_policy`'s `Err` is defined; contract 1 = Action taxonomy, where the emphasis statement is), so no bound became unlocatable. Clean.
3. **Memory growth** — the bounded-table story (`BTreeMap` resolution, index capped by `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH`, A25 headroom) is outside the diff and unchanged. Clean.
4. **Startup cost** — `Taxonomy::load` pinned to one data file, the single walked `data/` index, and the "no new dependency / core stays depends-on-nothing" claims are untouched by the diff (the Options → serde paragraph appears in the diff as context only). Clean.
5. **Caching / invalidation** — the drift gate and the shared-one-`Index`-per-test-binary claim (Test strategy #11, "the gate is linear in files, not tests") sit in diff context lines, unmodified. Clean.
6. **Research baseline vs. expected numbers** — the research file is outside this pass's permitted inputs, so the baseline was read as quoted inside the proposal: A24 (new formats far under 1 MiB, "verified only against the shipped corpus") and A25 (seed-scale walk, "unverified", product of the two index constants as the aggregate bound). The gap is labelled as an assumption, not asserted as fact, and neither assumption is in the diff. Nothing for this pass.

No blocker, major, minor, or nit finding: every changed line is punctuation, an Oxford comma, article insertion, a `Test #N` → `Test strategy #N` normalization, `valid+invalid` → `valid/invalid`, `Platform:` → `Platform —`, and `§N` → `contract N`. None of these carries a cost, bound, budget, cache, or startup claim; the two lines that do carry such claims changed only in the ways just listed.

## Findings

None.

## FYI (no severity)

- Revision log / editorial: the diff appends the open note "Editor notes: Chosen design → Profiles are chains — "`for` is required at parse" is ambiguous; author to clarify." That is an editorial thread for the author, not a cost claim — the parse-time requirement it names has no per-event path (parsing happens at index/CI time).
- Correctness seat, pre-existing and untouched by this diff: Authoring bar — "every rejected condition is pinned by Test strategy #3's model tests". Per Adversarial posture, `PlacementNotIsometric` and `PlacementDegraded` are pinned by #4, not #3; the same paragraph ends by naming "#3 and #4", so the narrower reference reads as scope-limited to the reach-table conditions. The diff only inserted "is" and did not change its truth value.
