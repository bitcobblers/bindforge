# complexity review r1c2

verdict: approve

Ratification pass over `git diff bbebe40..fe5671d` of 02-proposal.md, checked against the complexity checklist. No prior complexity findings. The revisions (extend `MAX_MANIFEST_ROWS` to `presets` instead of a new constant, one new invalid fixture, one new Test strategy #4 case, a baseline-justification sentence, a tightened Goals bullet, a tightened example parenthetical) did not break anything the r1 document got right on this axis:

- Reuse (item 1): the extension reuses the existing exported budget constant rather than adding a fifth; the slug-grammar reuse and the `load_action_registry` pattern are untouched.
- Traceability (item 2): the extended cap traces to the revised Goal bullet ("each array and the native name", A6) — every array in the entry table now has exactly one cap, and the risk-table line "The four named budgets cap every array in the entry table" is true with the four constants as written.
- Size (items 3, 5): the design is one format directory, one module, four exported constants, zero new dependencies (items 4, 7 proportionate); deferrals are unchanged and consistent with ADR-013's Decision and "Not decided here" clauses.
- The new fixture `presets-past-cap.toml` and the "257th preset id" case refuse at parse level, matching the stated split; the 63-octet slug bound borrows an existing grammar bound, adding no new constant.

## Findings

- [nit][new] Chosen design → Budgets — "`MAX_MANIFEST_ROWS = 256` bounds each array-shaped entry (`actions`, `dead-keys`, `docked-actions`, `presets`)" — `exclusive-groups` is also an array-shaped entry and is bounded by `MAX_CONSTRAINT_GROUPS = 64`, not by this constant; "each array-shaped entry" overstates the sentence and the parenthetical list reads as if `exclusive-groups` were uncapped. The r1 wording ("bounds each of `actions`, `dead-keys`, and `docked-actions`") did not have this ambiguity. Fix: reword to "`MAX_MANIFEST_ROWS = 256` bounds `actions`, `dead-keys`, `docked-actions`, and `presets`; `MAX_CONSTRAINT_GROUPS = 64` bounds `exclusive-groups`" — same constants, no new mechanism.

## FYI (no severity)

- The revision log records the `presets` extension as an accepted r1 major; choosing the shared 256 constant over a fifth constant was the lower-complexity of the available closures — noted as an observation, no action.
