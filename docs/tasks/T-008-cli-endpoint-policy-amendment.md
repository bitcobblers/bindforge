---
doc_type: task
status: pending
date: 2026-11-19
type: enhancement
functional_area: application
associated_adr: ADR-026
depends_on: []
flow: light
---

# T-008: Open the CLI catalog endpoint policy (ADR-026 amendment)

ADR-026 `026-L1` amends `025-C4` and `025-L1`: the CLI's catalog endpoint
admits a literal-`http://` URL with any parseable host — loopback or remote
— and the loopback-only admission and refuse-non-loopback-resolution rules
retire. The connectivity gate itself, the response budgets, and the
fail-closed discipline are untouched, and the endpoint still comes only
from `--catalog-url` / `BINDFORGE_CATALOG_URL` and nothing else.

Changes, confined to `apps/discovery-cli`:

- `catalog.rs::parse_endpoint`: drop the `LOOPBACK_HOSTS` literal check;
  keep the literal-`http://` scheme requirement, the authority parse (no
  `@`, non-empty host, digits-only port), and the normalized
  `<scheme>://<host>:<port>` return. Refusal lines keep naming both config
  sources and, instead of the loopback spelling, the ADR-026 policy.
- `HttpCatalog::probe`: keep resolve-then-connect (fail closed, named), drop
  the "refuse unless every address is loopback" leg. A host that resolves
  to no address or answers no connect keeps its current named `Unreachable`.
- `CatalogError::line()` wording updates to name the ADR-026 endpoint policy
  (still one line, still naming `--catalog-url` and `BINDFORGE_CATALOG_URL`).
- Re-pin the suites that pin the retiring refusals: the `catalog.rs` unit
  tests for the admitted/refused spellings (`http://catalog.example` moves
  from refused to admitted; `https://`, `ftp://`, bare authorities, `@`
  credentials, and bad ports stay refused) and the gate suite's endpoint-
  policy section. Gate behavior tests (refusal before enumerating, response
  refusals) keep their loopback fakes — test hygiene keeps every request on
  loopback even though the policy widened.

Explicitly NOT changed: the request shape, `percent_encode`,
`read_response` budgets, `partition_candidates`, the match store, and the
JSON output contract.

Done when:
- The unit and gate suites pass with the re-pinned policy, and no test
  performs a non-loopback request.
- A subprocess test points `--catalog-url` at a loopback listener spelling
  the widened policy admits (an arbitrary Host header) and sees the gate
  pass, proving admission end to end without touching the network.
- `cargo test --workspace --locked` passes.
