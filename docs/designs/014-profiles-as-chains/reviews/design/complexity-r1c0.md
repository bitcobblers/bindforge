# complexity review r1 (panel pass c0, whole document)

verdict: approve

## Findings

- [minor][new] The contract: `schemas/profile/v2/` — the `id` entry row, "the profile's identifier: slug grammar (A14), ≤ 63 octets" — the 63-octet cap has no stated provenance anywhere in the document: it is not in Options considered (which chose only the shipped slug grammar), not in the assumption register (A14 cites the shared slug grammar, no length cap), and Rollout 2's "two budget constants" (`MAX_PATCH_DELTA_ELEMENTS`, `MAX_GENERATED_BY_OCTETS`) omit it — so the cap is enforced (entry table, the Id grammars rule, `id-past-cap.toml`, Test strategy #1) but the number is unowned. Fix: one line — if it mirrors a cap already shipped for manifest/preset ids, add it to A14's basis; if it is new, name it beside the two rollout constants or state the in-text reason (the profile-id↔file-name directory rule wants a bounded path segment). The cap itself is not YAGNI — it traces to the directory rule — this is a traceability gap only.

## Checklist walk

1. Reuse: new surface is one module (`chain_ops.rs`), one `CoreError` variant, one selector arm — the document shows each reusing shipped machinery (`check_dir_version`, slug grammar, `ActionId::parse`/`InputId::parse`, `one_line`, the `Cycle`/`ChainTooDeep` posture, the `reach_classes_entries` pattern, `load_manifest`'s id↔name rule). No reuse flag raised.
2. Goal trace: every entry in the entry table and every operation (`fork`/`diff`/`merge`) is named in ADR-014 Decision One/Three; pin equality traces to the ADR's named re-pinning cost; only the 63-octet cap is under-traced (finding above).
3. Smaller design: none found that still meets Decision One–Four (see value check).
4. Dependencies: none — std-only, `Cargo.lock` byte-unchanged.
5. Deferral: `fork`/`diff`/`merge` are the natural second half, but Decision Three fixes them in this ADR, so cutting them contradicts the source document; the rollout already sequences format/parse (steps 1–2) before the ops module (step 3), and data shipping is deferred (Rollout 6). No finding.
6. Knobs/abstractions: both new constants have consumers (parse refusals, fixtures, Test strategy #1); no config knob, no one-implementation abstraction layer.
7. Value check: proportionate. The code surface is one module plus a version-branched arm; the document's bulk is refusal rules and fixtures that the repo's own discipline (A3, A16) requires, and the design repeatedly chose the smaller answer — states the composite budget instead of a new aggregate constant, no search path, action↔action-only `swap`, refusal over inference, "mount at any rung" as data rather than an API, migration as a documented step rather than a tool.

## FYI (no severity)

- "Mount at any rung" paragraph: "a suffix without its base is refused by `load_chain` at load" — `load_chain` takes only the head path and always walks the full chain, so what refuses is a missing link, not a consumer's slicing choice; the phrasing may confuse — correctness seat's call.
- Two similarly-named public functions will coexist: shipped `resolve_chain` (single-document derives guard, untouched per A15) and new `resolve(chain, manifest)` (chain + manifest → layout). The public-surface doc edit is on the change list; keeping the distinction crisp is a compatibility/correctness concern.
- `major-not-shipped.toml` (v2 corpus) and the `version_gate.rs` v3 case exercise overlapping refusal mechanisms (version gate vs `check_dir_version`); whether both pin one rule is compatibility-seat business.
