# Complexity review r1 (pass c0, full panel)

verdict: request-changes

Scope: whole of `docs/designs/008-rust-for-the-product/02-proposal.md`. Inputs:
the proposal, `docs/adrs/008-rust-for-the-product.md`, the Revision log (empty —
"No findings yet — r0 initial draft", so every finding below is `new`), and
`tools/` checked for reuse per checklist item 1: `tools/orchestration` is the
OpenCode pipeline harness (TS, NodeNext, vitest) with nothing reusable for
TOML parsing, schema conformance, or a cargo gate, so no finding here claims a
missed in-repo reuse.

## Findings

- [major][new] Chosen design → "Formats contract: `schemas/` and the conformance
  harness" — "The scaffold ships with at least one valid and one invalid fixture
  per format; ADR-009's build seeds the real cases." (same wording in Test
  strategy #1 and Rollout 3) — the harness asserts "every valid fixture parses
  and re-serializes to a semantically equal value", so the first merged slice
  must ship working parse **and** serialize for all four formats (`profile`,
  `patch`, `manifest`, `policy`). But Rollout 2 creates `crates/core` as a
  "package skeleton" whose "public APIs and model content follow ADR-009", and
  the Non-goals hand "chains, policies, manifest contents" to ADR-009. So the
  design either forces four format models to be invented here and re-done when
  ADR-009 lands (implement-and-throw-away), or the scaffold fixtures are
  placeholders and Test #1/#5 pass vacuously — either way the slice is heavier
  than ADR-008's four clauses require, and it duplicates ownership of the model
  surface. Fix: narrow the three sentences (Harness, Test #1, Rollout 3) to the
  formats this design's own Goals exercise — `profile/v1` (needed by
  `resolve_chain` and the chain-guard fixtures) and
  `schemas/reach-classes/v1/` — and state that the other three format
  directories arrive with ADR-009, which the harness's existing
  `schemas/*/v*/fixtures/{valid,invalid}` walk picks up with no code change.
  This removes work rather than adding a mechanism.

- [minor][new] Chosen design → "Workspace shape" (root `Cargo.toml`) —
  "opt-level = 3                       # A6; applies once from the workspace
  root" alongside "clap = { version = \"4.6.7\", features = [\"derive\"] }" and
  "toml_edit = \"0.25\"" — a build profile and dependency entries with no
  consumer in this design. Nothing in the slice uses sqlx (the very next comment
  says "axum, sqlx (A2, A3) enter [workspace.dependencies] with the hub era"),
  `clap` is consumed only by `apps/cli` (ADR-012), and `toml_edit`'s reason to
  exist — comment-preserving writes — is the codec/write-policy work the
  Deferred section hands to ADR-012; loading `schemas/` TOML needs only a
  plain serde TOML reader. Checklist items 4 and 6. Fix: delete the
  `[profile.dev.package.sqlx-macros]` block and the `clap`/`toml_edit` lines
  from the root `Cargo.toml` in this design; add each in the ADR that creates
  its first consumer (the hub-era paragraph and rollout's "Later, with the hub
  era" line already carry that instruction).

- [minor][new] Gates → "Canonical sqlx rule (hub-era; everything else
  cross-references this)" — the paragraph spells out the full offline-cache
  machinery (`cargo sqlx prepare --workspace`, committed root `.sqlx/`,
  `SQLX_OFFLINE=true` in every gate job, a dedicated Postgres `--check` job) and
  Test strategy #7 and a Risks row carry it, yet the same paragraph ends
  "`SQLX_OFFLINE` is irrelevant until `apps/hub` exists." A workspace with no
  sqlx member needs no DB by construction, so the mechanism is a hub-era design
  written down here, outside the Deferred list (which defers hub schema,
  migrations and upload caps to ADR-014 but not this). Checklist items 3 and 5.
  Fix: keep the one-sentence decision in Options considered (offline cache over
  Postgres-service or macro ban — that decision is cheap and closes a research
  question) and move the mechanism wording, Test #7 and the risk row into the
  existing ADR-014 Deferred bullet ("plus the sqlx offline cache and
  `--check` job").

- [minor][new] Options considered → "Flatpak in the packaging matrix: ship or
  exclude." plus Compatibility matrix row "Windows 10 (1803+) / 11 — desktop
  era" — this design decides Flatpak exclusion, NSIS-vs-MSI, WebView2
  pre-install posture and macOS/Android status, and carries assumptions A13–A19
  (Tauri, WebView2 distribution, Flatpak sandbox, Proton paths) for a product
  surface whose Non-goals say "Desktop webview UI framework, its npm toolchain,
  packaging pipelines — owned by the desktop design" and whose Deferred list
  already defers them with "no follow-up yet". That is a large share of the
  document's surface (one option entry, four matrix rows, one risk row) spent
  outside its Goals — the smaller design named by checklist item 3 keeps the
  workspace, gates, `schemas/` harness and the TypeScript boundary, and reduces
  desktop packaging to one Deferred row plus the CI-runner Linux row. Fix:
  replace the Flatpak option entry and the Windows/macOS/Android/Flatpak rows
  with a single matrix row ("desktop packaging — desktop design decides") or a
  one-line decision note, and drop A13–A19 unless a row that survives cites
  them.

- [minor][new] Non-goals vs Chosen design → "The model's semantics (taxonomy,
  reach tables, chains, policies, manifest contents) — owned by ADR-009." vs
  "`ChainTooDeep` (chain longer than `MAX_CHAIN_DEPTH = 32`) ... the constant is
  documented in `schemas/README.md` so third-party readers can conform" — chain
  semantics are deferred, but this design fixes a normative depth limit,
  publishes it as part of the third-party contract, and Test #3 freezes the
  boundary at exactly 32. Two documents then own one public number, and no text
  states why 32 rather than a derived bound, so ADR-009 inherits a constraint it
  was told it owns. Fix: one clause in the Adversarial guard paragraph — chain
  semantics are ADR-009's; this design fixes only the two error variant names
  and where the constant lives — and either state the basis for 32 or mark it
  "initial value, ADR-009 may set it", with Test #3 pinned to the constant
  rather than the literal.

- [nit][new] Chosen design → "Workspace shape" — "[workspace.lints.rust]
  [workspace.lints.clippy]" are empty tables, while the prose claims "the floor
  and lint policy are declared exactly once". There is no lint policy being
  declared, and `cargo clippy --workspace --all-targets -- -D warnings` already
  carries the escalation, so the two headers are a knob with no consumer
  (checklist item 6). Fix: omit both tables until a specific lint is moved there
  (e.g. a `unused = "deny"` entry), or state the one lint the sections hold.

- [nit][new] Chosen design → member table row "`crates/transforms` |
  `bindforge-transforms` | policy stacks | this design; contents per ADR-009" —
  a second empty workspace member created now; nothing in the Goals, the
  harness, or the gate exercises it, and `members = ["crates/*"]` accepts it
  unchanged whenever ADR-009 gives it content. Fix: create `transforms` in the
  ADR-009 slice, or keep it and say plainly that the row is a placeholder that
  the gate compiles empty.

## Value check (checklist item 7)

The core of the design is proportionate: one root manifest, one toolchain pin,
three skeleton members, a fixture tree plus one generic harness, and two CI jobs
is close to the minimum that satisfies ADR-008's four clauses, and the choices
that look like additions (`bindforge-` prefixes, edition 2024, committed
`Cargo.lock`) each replace a known failure mode rather than add a layer. The
excess is all at the edges — hub-era sqlx machinery, desktop packaging
decisions, and pre-declared dependencies for crates that do not exist yet — and
removing them shrinks the design roughly a third without touching a single
Goal.

## FYI (no severity)

- Versioning rule ("a document's declared version must name a major
  `crates/core` supports" vs "a file's presence in `vN/fixtures/valid/` is the
  definition of 'supported'") states two different mechanisms for one word —
  correctness/compatibility seat's call.
- Risks mitigation claims "Exact pins in `[workspace.dependencies]`" while the
  manifest shows `serde = { features = ["derive"] }` with "exact version pinned
  at merge" — a compatibility-seat consistency question about the one-gate
  promise on a fresh clone.
- `cargo test --workspace --locked` as the third gate command is consistent with
  ADR-008 clause 4 (`npm run check` includes tests) — checked, not a deviation.
