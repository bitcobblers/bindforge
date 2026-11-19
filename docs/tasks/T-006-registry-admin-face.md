---
doc_type: task
status: pending
date: 2026-11-19
type: feature
functional_area: backend
associated_adr: ADR-026
depends_on: [T-005-registry-root-serve]
flow: light
---

# T-006: Registry admin face — enablement rule, mutations, atomic persist

ADR-026 (`026-C4`, `026-C5`) defines how games enter a registry's catalog:
a token-or-loopback-gated admin surface that mutates the instance's own
catalog document. T-005's root serves a frozen corpus; this task makes it
editable under the rule.

Land in `apps/registry`:

- Enablement (`026-C5`): the admin routes exist iff the listener is bound
  loopback-only **or** a bearer token is configured (`--admin-token` or
  `BINDFORGE_REGISTRY_ADMIN_TOKEN`); a non-loopback bind without a token is
  a startup refusal naming the rule. Disabled admin routes answer `404`;
  enabled ones require `Authorization: Bearer <token>` under a constant-time
  comparison, answering `401` for missing or wrong tokens.
- `POST /v1/admin/games` upserts one whole record `{game_id, name,
  source_keys}`; the record is validated against the `026-C3` contract
  (reuse the `load_catalog` validation pieces) and refused by name; an
  accepted mutation bumps `revision` and persists the document atomically —
  the exclusive-create tmp file + rename + owner-only mode discipline of
  `apps/discovery-cli/src/matches.rs`.
- `DELETE /v1/admin/games/<game_id>` removes the record (unknown id is the
  named refusal, not silence) and persists the same way.
- `GET /v1/admin/stats` answers `{games, source_keys, revision}` counts for
  the loaded document. Whole-document reads do not exist anywhere
  (`026-L3`).
- Wrong method on an admin route is a `405` naming the route's methods.

Done when:
- The enablement rule is unit-tested through a bind-decision seam (loopback
  vs non-loopback, token set vs unset) and pin-tested at the subprocess
  level, including `401`/`404`/`405` shapes.
- Upsert (insert and replace), delete, refusal of invalid records (bad
  grammar, unknown source token, missing field), and revision bumping are
  pinned; a crash-discipline test asserts a pre-existing tmp entry is never
  written through and the corpus file is byte-identical on refusal.
- After an upsert + delete round trip, a fresh registry process serves the
  mutated catalog (the persist is the truth, not the in-memory copy).
- `cargo test --workspace --locked` passes.
