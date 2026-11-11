# Transcript: design-review:complexity r1c0

## Provenance
- stage: design-review:complexity (round r1c0)
- agent: review/design-complexity
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:25:23.713Z
- ended: 2026-11-11T18:28:32.968Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/008-rust-for-the-product.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/008-rust-for-the-product.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/design/complexity-r1c0.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design — \"Formats contract: `schemas/` and the conformance harness\": \"The scaffold ships with at least one valid and one invalid fixture per format; ADR-009's build seeds the real cases.\" (repeated in Test strategy #1 and Rollout 3)","problem":"The harness asserts every valid fixture \"parses and re-serializes to a semantically equal value\", so the first merged slice must implement working parse+serialize for all four formats (profile, patch, manifest, policy) — while Rollout 2 creates `crates/core` as a \"package skeleton\" whose \"public APIs and model content follow ADR-009\" and Non-goals hand \"chains, policies, manifest contents\" to ADR-009. The slice either invents four format models now and re-does them when ADR-009 lands, or the scaffold fixtures are placeholders and Test #1/#5 pass vacuously. Either way it is more model implementation than ADR-008's four clauses require, and it duplicates ownership of that surface with ADR-009.","fix":"Narrow the three sentences (Harness, Test #1, Rollout 3) to the formats this design's own Goals exercise — `profile/v1` (needed by `resolve_chain` and the chain-guard fixtures) and `schemas/reach-classes/v1/` — and state that the other three format directories arrive with ADR-009, picked up automatically by the harness's existing `schemas/*/v*/fixtures/{valid,invalid}` walk with no code change. This removes work instead of adding a mechanism."},{"severity":"minor","location":"Chosen design — \"Workspace shape\", root Cargo.toml: \"opt-level = 3                       # A6; applies once from the workspace root\", plus \"clap = { version = \\\"4.6.7\\\", features = [\\\"derive\\\"] }\" and \"toml_edit = \\\"0.25\\\"\"","problem":"A build profile and dependency entries with no consumer in this design: nothing in the slice uses sqlx (the adjacent comment itself defers \"axum, sqlx (A2, A3) enter [workspace.dependencies] with the hub era\"), `clap` is consumed only by `apps/cli` (ADR-012), and `toml_edit`'s justification — comment-preserving writes — is the codec/write-policy work Deferred to ADR-012; reading `schemas/` TOML needs only a plain serde TOML reader. Config knobs and deps ahead of their consumers (checklist items 4 and 6).","fix":"Delete the `[profile.dev.package.sqlx-macros]` block and the `clap`/`toml_edit` lines from the root Cargo.toml in this design; each is added by the ADR that creates its first consumer, which the existing \"Later, with the hub era\" rollout line already instructs."},{"severity":"minor","location":"Chosen design — Gates, \"Canonical sqlx rule (hub-era; everything else cross-references this)\": \"`SQLX_OFFLINE` is irrelevant until `apps/hub` exists.\"","problem":"The paragraph specifies the full hub-era mechanism (committed root `.sqlx/`, `cargo sqlx prepare --workspace` per PR, `SQLX_OFFLINE=true` in every gate job, a dedicated Postgres `--check` job), and Test strategy #7 plus a Risks row carry it, yet a workspace with no sqlx member needs no DB by construction. This is a future document's design written down here, and it is not in the Deferred list (which sends hub schema, migrations and upload caps to ADR-014 but not this) — a larger design than the Goals need (checklist items 3 and 5).","fix":"Keep the one-sentence decision in Options considered (offline cache over Postgres-service or macro ban) and move the mechanism wording, Test #7 and the risk row into the existing ADR-014 Deferred bullet, appending \"plus the sqlx offline cache and its `--check` job\"."},{"severity":"minor","location":"Options considered — \"Flatpak in the packaging matrix: ship or exclude.\" and Compatibility matrix row \"Windows 10 (1803+) / 11 — desktop era\"","problem":"This design decides Flatpak exclusion, NSIS-vs-MSI, WebView2 pre-install posture and macOS/Android status, and carries assumptions A13–A19 (Tauri, WebView2 distribution, Flatpak sandbox, Proton paths) for a surface its Non-goals assign to \"the desktop design\" and its Deferred list already defers with \"no follow-up yet\". One option entry, four matrix rows and one risk row are spent outside the Goals; the smaller design named by checklist item 3 keeps workspace, gates, `schemas/` harness and the TypeScript boundary only.","fix":"Collapse the Flatpak option entry and the Windows/macOS/Android/Flatpak rows into one matrix row (\"desktop packaging — the desktop design decides\") or a one-line decision note, and drop assumptions A13–A19 unless a surviving row cites them."},{"severity":"minor","location":"Non-goals — \"The model's semantics (taxonomy, reach tables, chains, policies, manifest contents) — owned by ADR-009.\" vs Chosen design — Adversarial guard: \"`ChainTooDeep` (chain longer than `MAX_CHAIN_DEPTH = 32`) … the constant is documented in `schemas/README.md` so third-party readers can conform\"","problem":"Chain semantics are declared out of scope, but this design fixes a normative depth cap, publishes it as part of the third-party conformance contract, and Test #3 freezes the boundary at exactly 32. Two documents own one public number, and no text gives a basis for 32, so ADR-009 inherits a constraint it was told it owns.","fix":"Add one clause to the Adversarial guard paragraph: chain semantics are ADR-009's, and this design fixes only the two error variant names and where the constant lives; either state the basis for 32 or mark it \"initial value, ADR-009 may set it\", and pin Test #3 to the constant rather than the literal 32."},{"severity":"nit","location":"Chosen design — \"Workspace shape\": \"[workspace.lints.rust]\" and \"[workspace.lints.clippy]\" (both empty) next to the claim \"the floor and lint policy are declared exactly once\"","problem":"The two lint tables contain no entries, so no lint policy is actually declared, and the gate already escalates with `cargo clippy --workspace --all-targets -- -D warnings`. An abstraction with no consumer behind it (checklist item 6).","fix":"Omit both tables until a specific lint belongs there (e.g. an `unused = \"deny\"` entry), or name the one lint they hold."},{"severity":"nit","location":"Chosen design — member table: \"`crates/transforms` | `bindforge-transforms` | policy stacks | this design; contents per ADR-009\"","problem":"A second empty workspace member created now; no Goal, the harness, or the gate exercises it, and the `members = [\"crates/*\"]` glob already accepts it unchanged whenever content exists.","fix":"Create `crates/transforms` in the ADR-009 slice, or keep the row and state plainly that it is a placeholder the gate compiles empty."}]}
```
