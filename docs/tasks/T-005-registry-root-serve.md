---
doc_type: task
status: pending
date: 2026-11-19
type: feature
functional_area: backend
associated_adr: ADR-026
depends_on: [T-004-catalog-document-and-seed]
flow: light
---

# T-005: Root registry mode — serve candidates, health, and the 404s

ADR-026 (`026-C1`, `026-C2`, `026-L3`, `026-L6`) fixes the registry's
serving face; ADR-025's `025-C4` already pins the candidate response shape
the shipped CLI client speaks. No service implements it yet.

Create the package `apps/registry` (binary `registry`, std-only HTTP/1.1
serving, clap for argument parsing — clap is already a workspace pattern
from `discovery-cli`) and land root mode:

- `registry serve --root --corpus <path>` loads the catalog through
  `bindforge-core::load_catalog` (refusal at startup is the named line, non-
  zero exit), binds `--host` (default `127.0.0.1`) and `--port` (default
  ephemeral; the chosen URL is printed at start).
- `GET /v1/candidates?slug=<percent-encoded>` answers the join: the games
  whose `source_keys` contain the decoded slug, in document order, as
  `[{game_id, name}]`. The response framing is exactly what `catalog.rs`
  accepts: `200`, one decimal `Content-Length`, no `Transfer-Encoding`,
  `Connection: close`, body within the `025` budgets. An unknown slug is a
  `200 []` — "never heard of it" is a true answer from the root.
- `GET /v1/health` answers `200` with `{status, mode: "root", upstream:
  "none", local_entries, cached_slugs: 0, last_upstream_fetch: null}`.
- Every other route or method, and a malformed query, is a `404`. No route
  ever emits the whole catalog document (`026-L3`).
- A `query <slug>` subcommand is NOT part of this task (it exercises the
  caching policy; T-007 owns it).

Done when:
- `apps/registry` builds as a workspace member; the binary starts, prints
  its URL, and serves the two routes.
- Subprocess integration tests (loopback, per the `discovery-cli` suite
  pattern) pin: the candidates join against a fixture corpus, the exact
  response framing a client sees, empty-array for unknown slug, the health
  shape, and the 404 set.
- A test drives the real `discovery list` against the registry via
  `--catalog-url` with the existing scratch `SteamRoot` fixtures, proving the
  shipped client accepts the framing end to end.
- `cargo test --workspace --locked` passes.
