---
doc_type: task
status: pending
date: 2026-11-19
type: feature
functional_area: backend
associated_adr: ADR-026
depends_on: []
flow: light
---

# T-004: Land the catalog document schema, loader, and seed

ADR-026 (`026-C3`, `026-S1`) defines the catalog document every registry
mode reads and mutates: `{version: 1, revision, games: [{game_id, name,
source_keys}]}`. Nothing in the repo carries that shape yet.

This task lands the contract document and its loader, before any registry
binary exists:

- `schemas/catalog/v1/spec.md` states the format per the schemas house rules
  (entry table, closed entry set, the shared bound from `schemas/README.md`),
  with `fixtures/valid/` and `fixtures/invalid/` covering every named
  refusal: unknown entry, bad `game_id` grammar (shared slug grammar via
  `bindforge-core::is_slug_name` and `MAX_SLUG_OCTETS`), `name` past
  `MAX_NATIVE_NAME_OCTETS`, a `source_key` outside the `025-C1` closed
  token set or past `025-B1`, empty `games`, a non-integer or negative
  `revision`.
- `bindforge-core` gains `load_catalog`, mirroring how `load_manifest` works:
  it reads a document against the v1 contract and refuses by name; the
  format is JSON (unlike manifests' TOML), so the loader validates a parsed
  value against the entry table and bounds.
- `data/catalog/v1/catalog.json` is the seed corpus (`026-S1`): a handful of
  real games, each carrying its `steam/<appid>` source key, valid against
  the new schema, committed beside a `README.md` in the repo's data-directory
  style.

Done when:
- `schemas/catalog/v1/` exists with `spec.md` and the fixture pair set.
- `load_catalog` in `crates/core` accepts every valid fixture and refuses
  every invalid fixture by name, unit-tested per ADR-004 layout.
- The seed file loads through `load_catalog` (a test asserts it).
- `cargo test --workspace --locked` passes.
